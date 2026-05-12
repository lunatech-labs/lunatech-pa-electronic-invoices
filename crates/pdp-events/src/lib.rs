//! # pdp-events — Bus d'événements interne et journal d'audit durable
//!
//! Ce crate est le **système nerveux** de la PDP : chaque transition du
//! cycle de vie d'un flux (facture, CDAR, e-reporting) produit un événement
//! immuable, persisté dans PostgreSQL (pattern *outbox*), puis distribué
//! aux abonnés (webhooks, métriques, archivage, etc.).
//!
//! ## Garanties
//!
//! - **At-least-once** : un événement persisté est distribué au moins une fois
//!   à chaque subscriber. Les subscribers doivent être idempotents (clef de
//!   déduplication : `event.id`).
//! - **Ordre total par subscriber** : chaque subscriber consomme dans l'ordre
//!   du `sequence` global. Un échec bloque l'avancement du watermark de **ce**
//!   subscriber uniquement, pas des autres.
//! - **Audit** : la table `events` est le journal d'audit. Elle répond à
//!   « qu'est-il arrivé à la facture X ? » via [`store::EventStore::list_by_invoice`].
//!
//! ## Vue d'ensemble
//!
//! ```text
//!   ┌──────────────┐   publish()    ┌──────────────┐
//!   │  pipeline    │ ─────────────► │  EventStore  │  ← table `events`
//!   │ (Processor)  │                │  (outbox)    │
//!   └──────────────┘                └──────┬───────┘
//!                                          │ fetch_after()
//!                                          ▼
//!                                  ┌──────────────────┐
//!                                  │ DispatcherWorker │  une instance / subscriber
//!                                  └──────┬───────────┘
//!                                         │ handle()
//!                          ┌──────────────┼──────────────┐
//!                          ▼              ▼              ▼
//!                    ┌─────────┐    ┌─────────┐    ┌────────────┐
//!                    │webhook  │    │ metrics │    │ archive ES │
//!                    └─────────┘    └─────────┘    └────────────┘
//! ```
//!
//! ## Exemple minimal
//!
//! ```no_run
//! # use std::sync::Arc;
//! # async fn ex(pool: sqlx::PgPool) -> Result<(), Box<dyn std::error::Error>> {
//! use pdp_events::{EventStore, EventBus, Event, EventKind};
//!
//! let store = Arc::new(EventStore::new(pool));
//! store.migrate().await?;
//! let bus = EventBus::new(store.clone());
//!
//! // Émettre un événement
//! let flow_id = uuid::Uuid::new_v4();
//! bus.publish(Event::new(flow_id, EventKind::Received)
//!     .with_invoice_key("123456789/FA-001/2026")
//!     .with_tenant("123456789"))
//!     .await?;
//! # Ok(()) }
//! ```

pub mod bus;
pub mod dispatcher;
pub mod error;
pub mod event;
pub mod processor;
pub mod store;

pub use bus::{EventBus, Subscriber};
pub use dispatcher::{DispatcherConfig, DispatcherWorker};
pub use error::{EventError, EventResult};
pub use event::{Event, EventKind};
pub use processor::{LifecycleProcessor, StatusReflectingProcessor};
pub use store::EventStore;
