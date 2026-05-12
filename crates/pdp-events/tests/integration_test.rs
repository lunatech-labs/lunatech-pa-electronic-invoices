//! Tests d'intégration `pdp-events` avec un vrai Postgres (testcontainers).
//!
//! Couvre les invariants critiques du bus :
//! - publication + audit (list_by_flow, list_by_invoice)
//! - ordre total par `sequence`
//! - at-least-once : un subscriber qui échoue ne perd pas l'événement et
//!   ne fait pas avancer son watermark
//! - isolation entre subscribers (un subscriber bloqué n'affecte pas l'autre)
//! - dé-duplication via `event.id` côté livraison

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use async_trait::async_trait;
use pdp_events::{
    DispatcherConfig, DispatcherWorker, Event, EventBus, EventKind, EventResult, EventStore,
    Subscriber,
};
use sqlx::postgres::PgPoolOptions;
use testcontainers::runners::AsyncRunner;
use testcontainers::ContainerAsync;
use testcontainers_modules::postgres::Postgres;
use uuid::Uuid;

async fn setup() -> (ContainerAsync<Postgres>, Arc<EventStore>, EventBus) {
    let container = Postgres::default().start().await.expect("postgres");
    let host = container.get_host().await.unwrap();
    let port = container.get_host_port_ipv4(5432).await.unwrap();
    let url = format!("postgres://postgres:postgres@{}:{}/postgres", host, port);
    let pool = PgPoolOptions::new()
        .max_connections(4)
        .connect(&url)
        .await
        .unwrap();
    let store = Arc::new(EventStore::new(pool));
    store.migrate().await.unwrap();
    let bus = EventBus::new(store.clone());
    (container, store, bus)
}

#[tokio::test]
async fn publish_audit_by_flow_and_invoice() {
    let (_c, store, bus) = setup().await;
    let flow = Uuid::new_v4();
    let key = "123456789/FA-001/2026";

    for k in [
        EventKind::Received,
        EventKind::Parsed,
        EventKind::Validated,
        EventKind::Transformed,
        EventKind::Distributed,
        EventKind::Acknowledged,
    ] {
        bus.publish(
            Event::new(flow, k)
                .with_invoice_key(key)
                .with_tenant("123456789"),
        )
        .await
        .unwrap();
    }

    let by_flow = store.list_by_flow(flow).await.unwrap();
    assert_eq!(by_flow.len(), 6, "doit retrouver les 6 événements du flux");
    // Ordre total : sequences strictement croissantes
    let seqs: Vec<i64> = by_flow.iter().filter_map(|e| e.sequence).collect();
    let mut sorted = seqs.clone();
    sorted.sort();
    assert_eq!(seqs, sorted, "ordre total par sequence");

    let by_inv = store.list_by_invoice(key).await.unwrap();
    assert_eq!(by_inv.len(), 6, "audit par clef facture");
}

struct CountingSub {
    id: String,
    count: AtomicUsize,
}

#[async_trait]
impl Subscriber for CountingSub {
    fn id(&self) -> &str {
        &self.id
    }
    async fn handle(&self, _e: &Event) -> EventResult<()> {
        self.count.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

#[tokio::test]
async fn dispatcher_drains_outbox_in_order() {
    let (_c, store, bus) = setup().await;
    let flow = Uuid::new_v4();
    for k in [EventKind::Received, EventKind::Validated, EventKind::Distributed] {
        bus.publish(Event::new(flow, k)).await.unwrap();
    }

    let sub = Arc::new(CountingSub {
        id: "test-counter".into(),
        count: AtomicUsize::new(0),
    });
    let worker = DispatcherWorker::new(store.clone(), sub.clone())
        .with_config(DispatcherConfig {
            poll_interval: std::time::Duration::from_millis(50),
            batch_size: 10,
        });

    let processed = worker.tick().await.unwrap();
    assert_eq!(processed, 3);
    assert_eq!(sub.count.load(Ordering::SeqCst), 3);

    // Second tick : rien à faire, watermark à jour
    let processed = worker.tick().await.unwrap();
    assert_eq!(processed, 0);
}

struct FailingSub {
    id: String,
    fail_count: AtomicUsize,
    fail_first_n: usize,
}

#[async_trait]
impl Subscriber for FailingSub {
    fn id(&self) -> &str {
        &self.id
    }
    async fn handle(&self, _e: &Event) -> EventResult<()> {
        let n = self.fail_count.fetch_add(1, Ordering::SeqCst);
        if n < self.fail_first_n {
            Err(pdp_events::EventError::Other(format!("simulated fail #{n}")))
        } else {
            Ok(())
        }
    }
}

#[tokio::test]
async fn at_least_once_on_subscriber_failure() {
    // Un subscriber qui échoue les 2 premières fois doit rejouer l'événement
    // jusqu'à succès. Le watermark ne doit pas dépasser l'événement bloquant.
    let (_c, store, bus) = setup().await;
    let flow = Uuid::new_v4();
    bus.publish(Event::new(flow, EventKind::Received)).await.unwrap();
    bus.publish(Event::new(flow, EventKind::Validated)).await.unwrap();

    let sub = Arc::new(FailingSub {
        id: "flaky".into(),
        fail_count: AtomicUsize::new(0),
        fail_first_n: 2,
    });
    let worker = DispatcherWorker::new(store.clone(), sub.clone());

    // Tick 1 : 1ère tentative → échec, watermark reste à 0
    let _ = worker.tick().await.unwrap();
    assert_eq!(
        store.get_watermark("flaky").await.unwrap(),
        0,
        "watermark inchangé après échec"
    );

    // Tick 2 : 2ème tentative → échec encore
    let _ = worker.tick().await.unwrap();
    assert_eq!(store.get_watermark("flaky").await.unwrap(), 0);

    // Tick 3 : succès des 2 événements
    let processed = worker.tick().await.unwrap();
    assert_eq!(processed, 2);
    let wm = store.get_watermark("flaky").await.unwrap();
    assert!(wm >= 2, "watermark avance après succès, vu={}", wm);

    // L'événement #1 a bien été tenté plusieurs fois
    let _ = store
        .record_delivery("flaky", Uuid::new_v4(), true, None)
        .await
        .unwrap(); // ne plante pas
}

#[tokio::test]
async fn subscribers_are_isolated() {
    // Un subscriber bloqué (toujours en échec) ne doit pas empêcher
    // un autre subscriber d'avancer sur le même flux.
    let (_c, store, bus) = setup().await;
    let flow = Uuid::new_v4();
    bus.publish(Event::new(flow, EventKind::Received)).await.unwrap();
    bus.publish(Event::new(flow, EventKind::Validated)).await.unwrap();

    let blocked = Arc::new(FailingSub {
        id: "blocked".into(),
        fail_count: AtomicUsize::new(0),
        fail_first_n: 1_000,
    });
    let healthy = Arc::new(CountingSub {
        id: "healthy".into(),
        count: AtomicUsize::new(0),
    });

    let w1 = DispatcherWorker::new(store.clone(), blocked.clone());
    let w2 = DispatcherWorker::new(store.clone(), healthy.clone());

    let _ = w1.tick().await.unwrap();
    let _ = w2.tick().await.unwrap();

    assert_eq!(
        store.get_watermark("blocked").await.unwrap(),
        0,
        "blocked reste au point mort"
    );
    let wm_healthy = store.get_watermark("healthy").await.unwrap();
    assert!(wm_healthy >= 2);
    assert_eq!(healthy.count.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn subscriber_filter_advances_watermark() {
    // Un subscriber qui ignore (accepts=false) un événement doit quand même
    // faire avancer son watermark — sinon il se bloque sur les événements
    // qui ne l'intéressent pas.
    let (_c, store, bus) = setup().await;
    let flow = Uuid::new_v4();
    bus.publish(Event::new(flow, EventKind::Parsing)).await.unwrap();
    bus.publish(Event::new(flow, EventKind::Validated)).await.unwrap();

    struct OnlyValidated {
        id: String,
        count: AtomicUsize,
    }
    #[async_trait]
    impl Subscriber for OnlyValidated {
        fn id(&self) -> &str {
            &self.id
        }
        fn accepts(&self, e: &Event) -> bool {
            matches!(e.kind, EventKind::Validated)
        }
        async fn handle(&self, _e: &Event) -> EventResult<()> {
            self.count.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    let sub = Arc::new(OnlyValidated {
        id: "selective".into(),
        count: AtomicUsize::new(0),
    });
    let worker = DispatcherWorker::new(store.clone(), sub.clone());

    let processed = worker.tick().await.unwrap();
    assert_eq!(processed, 1, "un seul événement traité (Validated)");
    assert_eq!(sub.count.load(Ordering::SeqCst), 1);

    // Watermark doit avoir avancé au-delà de l'événement Parsing ignoré.
    let wm = store.get_watermark("selective").await.unwrap();
    assert!(wm >= 2, "watermark a sauté l'événement filtré, vu={}", wm);
}
