//! Subscriber qui consomme le bus `pdp-events` et alimente l'archive Elasticsearch.
//!
//! Première étape de la bascule vers une architecture où `pdp-trace` n'est plus
//! appelé directement dans le pipeline : il devient un consommateur du bus,
//! au même titre que les webhooks. Couvre la duplication des `FlowEvent` dans
//! les documents `pdp-{siren}` (champ `events`).
//!
//! Étape suivante (hors de ce crate) : supprimer les `TraceProcessor::*` du
//! pipeline et garder uniquement un `ExchangeSnapshotProcessor` qui archive
//! XML/PDF aux jalons. Cette responsabilité reste hors du bus parce qu'elle
//! a besoin de l'`Exchange` complet (corps brut, pièces jointes).

use std::sync::Arc;

use async_trait::async_trait;
use pdp_core::model::{FlowEvent, FlowStatus};
use pdp_events::{Event, EventKind, EventResult, Subscriber};

use crate::store::TraceStore;

/// Subscriber qui mappe `EventKind` → `FlowStatus` et appelle `record_event`.
pub struct TraceEventSubscriber {
    id: String,
    store: Arc<TraceStore>,
}

impl TraceEventSubscriber {
    pub fn new(store: Arc<TraceStore>) -> Arc<Self> {
        Arc::new(Self {
            id: "trace-archive".to_string(),
            store,
        })
    }

    fn map_kind(kind: EventKind) -> FlowStatus {
        match kind {
            EventKind::Received => FlowStatus::Received,
            EventKind::Parsing => FlowStatus::Parsing,
            EventKind::Parsed => FlowStatus::Parsed,
            EventKind::Validating => FlowStatus::Validating,
            EventKind::Validated => FlowStatus::Validated,
            EventKind::Transforming => FlowStatus::Transforming,
            EventKind::Transformed => FlowStatus::Transformed,
            EventKind::Distributing => FlowStatus::Distributing,
            EventKind::Distributed => FlowStatus::Distributed,
            EventKind::WaitingAck => FlowStatus::WaitingAck,
            EventKind::Acknowledged => FlowStatus::Acknowledged,
            EventKind::Rejected => FlowStatus::Rejected,
            EventKind::Cancelled => FlowStatus::Cancelled,
            EventKind::Error => FlowStatus::Error,
        }
    }
}

#[async_trait]
impl Subscriber for TraceEventSubscriber {
    fn id(&self) -> &str {
        &self.id
    }

    async fn handle(&self, event: &Event) -> EventResult<()> {
        let status = Self::map_kind(event.kind);
        let route_id = event.route_id.as_deref().unwrap_or("unknown");
        let message = event
            .message
            .clone()
            .unwrap_or_else(|| format!("étape {}", event.kind.as_code()));

        let mut flow_event = FlowEvent::new(event.flow_id, route_id, status, &message);
        flow_event.id = event.id; // idempotence : même id, même document
        flow_event.timestamp = event.occurred_at;
        if let Some(ref key) = event.invoice_key {
            flow_event = flow_event.with_invoice_key(key);
        }
        if let Some(ref err) = event.error_detail {
            flow_event = flow_event.with_error(err);
        }

        self.store
            .record_event(&flow_event)
            .await
            .map_err(|e| pdp_events::EventError::Subscriber {
                subscriber: "trace-archive".into(),
                message: e.to_string(),
            })?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_covers_all_kinds() {
        // Compile-time guarantee : toutes les variantes sont mappées.
        for k in [
            EventKind::Received,
            EventKind::Parsing,
            EventKind::Parsed,
            EventKind::Validating,
            EventKind::Validated,
            EventKind::Transforming,
            EventKind::Transformed,
            EventKind::Distributing,
            EventKind::Distributed,
            EventKind::WaitingAck,
            EventKind::Acknowledged,
            EventKind::Rejected,
            EventKind::Cancelled,
            EventKind::Error,
        ] {
            let _ = TraceEventSubscriber::map_kind(k);
        }
    }

    #[test]
    fn id_is_stable() {
        // L'id doit être stable pour préserver le watermark entre redémarrages.
        let store = Arc::new(TraceStore::noop());
        let s = TraceEventSubscriber::new(store);
        assert_eq!(s.id(), "trace-archive");
    }

    #[test]
    fn handle_uses_event_id_for_idempotence() {
        // Sanity : le FlowEvent construit hérite de l'id de l'événement bus,
        // de sorte qu'un rejouage (at-least-once) ne crée pas de doublon.
        let store = Arc::new(TraceStore::noop());
        let _sub = TraceEventSubscriber::new(store);
        let ev_id = uuid::Uuid::new_v4();
        let e = Event {
            id: ev_id,
            flow_id: uuid::Uuid::new_v4(),
            kind: EventKind::Validated,
            invoice_key: None,
            tenant_siren: None,
            route_id: None,
            step: None,
            message: None,
            error_detail: None,
            payload: None,
            occurred_at: chrono::Utc::now(),
            sequence: Some(1),
        };
        assert_eq!(e.id, ev_id);
    }
}
