# Bus d'événements interne et journal d'audit — `pdp-events`

Le crate `pdp-events` est le système nerveux de la PDP : chaque transition du cycle de vie d'un flux (facture, CDAR, e-reporting) produit un événement immuable, persisté dans PostgreSQL puis distribué aux abonnés internes (webhooks, archivage, métriques).

## Pourquoi un bus ?

Avant `pdp-events`, deux problèmes coexistaient :

1. **Observabilité interne** dispersée — chaque processeur appelait directement `pdp-trace` pour écrire dans Elasticsearch. Pas d'API unifiée pour consommer l'historique d'une facture.
2. **Webhooks couplés au pipeline** — la dispatcher webhook était appelée en ligne dans les processeurs, donc impossible d'ajouter un consommateur (métriques, audit fiscal, replay) sans modifier le code métier.

Le bus résout les deux : **un seul point de publication, plusieurs consommateurs indépendants, un journal durable pour l'audit**.

## Garanties

| Propriété | Comportement |
|-----------|--------------|
| **Durabilité** | Toute publication réussie est persistée dans la table `events`. L'événement existe avant tout fan-out. |
| **At-least-once** | Chaque subscriber reçoit chaque événement au moins une fois. Un crash entre handle et watermark peut entraîner un rejouage. Les subscribers doivent être idempotents (clef de déduplication : `event.id`). |
| **Ordre total par subscriber** | Chaque subscriber consomme les événements dans l'ordre du `sequence` global. Un échec bloque uniquement ce subscriber, pas les autres. |
| **Isolation** | Un subscriber lent ou en erreur ne ralentit pas les autres. Chaque subscriber a son propre worker et son propre watermark. |
| **Audit complet** | `EventStore::list_by_invoice(key)` renvoie l'historique chronologique complet d'une facture. |

## Granularité — 14 événements

Une variante par valeur de `FlowStatus`, en correspondance 1-1 :

| `EventKind` | Étape métier |
|-------------|--------------|
| `Received` | Réception |
| `Parsing`, `Parsed` | Parsing XML/PDF |
| `Validating`, `Validated` | Validation XSD + Schematron |
| `Transforming`, `Transformed` | Transformation UBL/CII/Factur-X |
| `Distributing`, `Distributed` | Distribution (PPF, PEPPOL, autre PDP) |
| `WaitingAck`, `Acknowledged` | Attente et réception de l'ack destinataire |
| `Rejected`, `Cancelled`, `Error` | États terminaux d'échec |

Cette granularité fine sert à la fois l'audit (chaque transition est tracée) et l'observabilité (chaque consommateur filtre ce qu'il veut).

## Architecture

```
┌──────────────┐   publish()    ┌──────────────┐
│  pipeline    │ ─────────────► │  EventStore  │  ← table events (outbox)
│ (Processor)  │                │   PostgreSQL │
└──────────────┘                └──────┬───────┘
                                       │ fetch_after(watermark)
                                       ▼
                              ┌──────────────────┐
                              │ DispatcherWorker │  une instance par subscriber
                              └──────┬───────────┘
                                     │ handle()
                  ┌──────────────────┼──────────────────┐
                  ▼                  ▼                  ▼
        ┌─────────────────┐  ┌─────────────┐  ┌─────────────────┐
        │ WebhooksSub     │  │ MetricsSub  │  │ ArchiveESSub    │
        │ AFNOR XP Z12-013│  │ Prometheus  │  │ Elasticsearch   │
        └─────────────────┘  └─────────────┘  └─────────────────┘
```

## Schéma PostgreSQL

Trois tables. Tout est idempotent — `EventStore::migrate()` peut être appelé à chaque démarrage.

```sql
events                  -- journal d'audit (outbox)
├── id            UUID PRIMARY KEY
├── sequence      BIGSERIAL UNIQUE     -- ordre total
├── flow_id       UUID
├── kind          TEXT                 -- code stable: "received", "validated", ...
├── invoice_key   TEXT                 -- "SIREN/NUMERO/ANNEE"
├── tenant_siren  TEXT
├── route_id      TEXT
├── step          TEXT
├── message       TEXT
├── error_detail  TEXT
├── payload       JSONB
├── occurred_at   TIMESTAMPTZ
└── created_at    TIMESTAMPTZ

event_subscriptions     -- watermark par subscriber
├── subscriber_id TEXT PRIMARY KEY
├── last_sequence BIGINT
└── updated_at    TIMESTAMPTZ

event_deliveries        -- traces de livraison (observabilité)
├── subscriber_id    TEXT
├── event_id         UUID
├── attempts         INT
├── last_attempt_at  TIMESTAMPTZ
├── delivered_at     TIMESTAMPTZ      -- NULL si jamais livré
├── last_error       TEXT
└── PRIMARY KEY (subscriber_id, event_id)
```

Index :
- `events(flow_id)`, `events(invoice_key)`, `events(tenant_siren)` pour les requêtes d'audit
- `events(kind)`, `events(occurred_at)` pour les filtres
- `events(sequence)` pour le fan-out
- Partiel sur `event_deliveries` pour les livraisons en échec

## Composants

### `EventStore` ([store.rs](../crates/pdp-events/src/store.rs))

Couche de persistance. API principale :
- `append(event)` — insère et renvoie l'événement avec son `sequence`
- `list_by_flow(flow_id)` / `list_by_invoice(key)` — audit
- `fetch_after(seq, n)` — utilisé par le worker
- `get_watermark(sub_id)` / `set_watermark(sub_id, seq)`
- `record_delivery(sub_id, event_id, success, error)`

### `EventBus` ([bus.rs](../crates/pdp-events/src/bus.rs))

Façade de publication. Seule API utilisée par le pipeline pour émettre un événement.

```rust
let bus = EventBus::new(store.clone());
bus.publish(Event::new(flow_id, EventKind::Validated)
    .with_invoice_key("123456789/FA-001/2026")
    .with_tenant("123456789")
    .with_step("validation")).await?;
```

### `Subscriber` trait

```rust
#[async_trait]
pub trait Subscriber: Send + Sync {
    fn id(&self) -> &str;                       // stable, persistant
    fn accepts(&self, event: &Event) -> bool;   // filtre rapide
    async fn handle(&self, event: &Event) -> EventResult<()>;
}
```

Les implémentations doivent être idempotentes. L'`id` sert de clef de watermark dans `event_subscriptions` — il doit être stable entre redémarrages.

### `DispatcherWorker` ([dispatcher.rs](../crates/pdp-events/src/dispatcher.rs))

Une instance par subscriber. Boucle :
1. Lit le watermark.
2. Récupère un batch d'événements postérieurs.
3. Filtre via `accepts`, traite via `handle`.
4. Avance le watermark à la séquence du dernier événement traité **avec succès ou explicitement ignoré**.

En cas d'échec sur `handle`, le watermark reste bloqué et l'erreur est tracée dans `event_deliveries`. Le batch s'arrête (préserve l'ordre).

### `LifecycleProcessor` ([processor.rs](../crates/pdp-events/src/processor.rs))

Adapter `pdp_core::Processor` qui publie un événement à chaque passage. Un constructeur par variante :

```rust
builder
    .process(Box::new(LifecycleProcessor::validated(bus.clone())))
    .process(Box::new(LifecycleProcessor::transformed(bus.clone())))
    .process(Box::new(LifecycleProcessor::distributed(bus.clone())));
```

Variante générique : `LifecycleProcessor::from_status(bus)` dérive le `EventKind` du `FlowStatus` courant de l'exchange.

## Subscribers de référence

### `WebhooksSubscriber` ([webhooks_subscriber.rs](../crates/pdp-app/src/webhooks_subscriber.rs))

Branche le bus aux webhooks AFNOR XP Z12-013 §5.4. Mapping :

| `EventKind` | Événement webhook | `ackStatus` |
|-------------|-------------------|-------------|
| `Received` | `flow.received` | `Pending` |
| `Validated`, `Distributed`, `Acknowledged` | `flow.ack.updated` | `Ok` |
| `Rejected`, `Error` | `flow.ack.updated` | `Error` |
| Autres | ignoré (le subscriber filtre via `accepts`) | — |

Habituellement deux instances par PDP : une pour `flow_direction="In"` (réception) et une pour `Out` (émission).

## Migration depuis `pdp-trace`

L'objectif est que `pdp-events` remplace `pdp-trace` comme source de vérité d'audit. La migration se fait en trois temps :

1. **V1 — bus disponible (livré)**. Le crate `pdp-events` coexiste avec `pdp-trace`. Le bus est testé en isolation.
2. **V2 — wiring + double consommation (livré)**. Le bus est créé dans `main.rs` quand PostgreSQL est dispo. Deux subscribers sont branchés en arrière-plan :
   - [`WebhooksSubscriber`](../crates/pdp-app/src/webhooks_subscriber.rs) — dispatch des webhooks AFNOR
   - [`TraceEventSubscriber`](../crates/pdp-trace/src/event_subscriber.rs) — réplication des événements vers Elasticsearch (champ `events` des documents `pdp-{siren}`)

   Les processeurs du pipeline appellent `LifecycleProcessor::from_status(bus)` aux jalons (`Validated`, `Transformed`, `Distributed`). Quand le bus est `None` (pas de Postgres), le pipeline retombe sur l'ancien chemin (`WebhookAckProcessor` + dispatch direct dans `server.rs`).
3. **V3 — découplage complet (livré)**. `TraceProcessor` a été renommé en [`ExchangeSnapshotProcessor`](../crates/pdp-trace/src/processor.rs) et ne fait plus que l'archivage XML/PDF (`record_exchange`). Les événements de cycle de vie sont exclusivement publiés via le bus, et `TraceEventSubscriber` les réplique vers Elasticsearch.
   - Plus de duplication d'événements (avant V3, l'événement était écrit deux fois : par `TraceProcessor` puis par le subscriber).
   - `record_event` est désormais **idempotent** côté ES : le script Painless vérifie l'`event.id` avant d'ajouter dans le tableau `events`. Un rejouage at-least-once ne crée pas de doublon.
   - Un alias rétrocompat `#[deprecated] pub type TraceProcessor = ExchangeSnapshotProcessor;` reste exporté le temps que les usages externes migrent.

## Usage côté code

### Boot

```rust
let store = Arc::new(EventStore::new(pg_pool));
store.migrate().await?;
let bus = EventBus::new(store.clone());

// Démarrer les subscribers
let webhook_sub_in  = WebhooksSubscriber::new(webhook_store.clone(), "In");
let webhook_sub_out = WebhooksSubscriber::new(webhook_store.clone(), "Out");
let _h1 = DispatcherWorker::new(store.clone(), webhook_sub_in).spawn();
let _h2 = DispatcherWorker::new(store.clone(), webhook_sub_out).spawn();
```

### Insertion dans le pipeline

```rust
builder = builder
    .process(Box::new(ValidateProcessor::new()))
    .process(Box::new(LifecycleProcessor::validated(bus.clone())))
    .process(Box::new(TransformProcessor::new(target_format)))
    .process(Box::new(LifecycleProcessor::transformed(bus.clone())));
```

### Audit d'une facture

```rust
let history = store.list_by_invoice("123456789/FA-001/2026").await?;
for e in history {
    println!("{} | {} | {:?}", e.occurred_at, e.kind.as_code(), e.message);
}
```

## Tests

- **Unitaires** : 4 tests dans `src/` (mapping `EventKind` ↔ `FlowStatus`, builder, contrat `Subscriber`).
- **Intégration testcontainers** : 5 tests dans [tests/integration_test.rs](../crates/pdp-events/tests/integration_test.rs) couvrant :
  - publication + audit par flow_id et invoice_key
  - dispatcher en ordre, watermark progresse
  - at-least-once : un subscriber qui échoue rejoue
  - isolation entre subscribers (un bloqué ne gêne pas les autres)
  - filtre `accepts` qui avance le watermark sans traiter

```bash
cargo test -p pdp-events              # tout
cargo test -p pdp-events --lib        # unitaires uniquement (sans Postgres)
```

## Conformité audit

L'événement est immuable une fois persisté. Les colonnes `id`, `sequence`, `created_at` ne sont jamais mises à jour. Pour répondre à un audit fiscal (Art. 242 nonies A CGI) :

```sql
SELECT occurred_at, kind, step, message, error_detail, payload
FROM events
WHERE invoice_key = '123456789/FA-2026-001/2026'
ORDER BY sequence;
```

renvoie l'historique complet, dans l'ordre, avec les détails techniques de chaque étape.
