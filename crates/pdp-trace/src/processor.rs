use async_trait::async_trait;
use pdp_core::error::PdpResult;
use pdp_core::exchange::Exchange;
use pdp_core::model::FlowStatus;
use pdp_core::processor::Processor;

use crate::store::TraceStore;
use std::sync::Arc;

/// Processor d'archivage : prend un snapshot de l'`Exchange` (XML, PDF,
/// métadonnées) dans Elasticsearch à un jalon du pipeline.
///
/// Depuis la V3 de la migration vers `pdp-events`, ce processor ne publie
/// plus d'événement de cycle de vie. Les événements sont émis sur le bus
/// par `pdp_events::LifecycleProcessor` et persistés par le subscriber
/// `TraceEventSubscriber` (qui appelle [`TraceStore::record_event`]).
///
/// Les constructeurs `received/parsed/validated/...` sont conservés pour
/// la lisibilité de la chaîne de pipeline et pour les logs `tracing`.
/// Ils ne diffèrent que par le `step_name`.
pub struct ExchangeSnapshotProcessor {
    store: Arc<TraceStore>,
    step_name: String,
}

impl ExchangeSnapshotProcessor {
    pub fn new(store: Arc<TraceStore>, step_name: &str, _status: FlowStatus) -> Self {
        Self {
            store,
            step_name: step_name.to_string(),
        }
    }

    pub fn received(store: Arc<TraceStore>) -> Self {
        Self::new(store, "reception", FlowStatus::Received)
    }

    pub fn parsed(store: Arc<TraceStore>) -> Self {
        Self::new(store, "parsing", FlowStatus::Parsed)
    }

    pub fn validated(store: Arc<TraceStore>) -> Self {
        Self::new(store, "validation", FlowStatus::Validated)
    }

    pub fn transformed(store: Arc<TraceStore>) -> Self {
        Self::new(store, "transformation", FlowStatus::Transformed)
    }

    pub fn distributed(store: Arc<TraceStore>) -> Self {
        Self::new(store, "distribution", FlowStatus::Distributed)
    }
}

#[async_trait]
impl Processor for ExchangeSnapshotProcessor {
    fn name(&self) -> &str {
        "ExchangeSnapshotProcessor"
    }

    async fn process(&self, exchange: Exchange) -> PdpResult<Exchange> {
        if let Err(e) = self.store.record_exchange(&exchange).await {
            tracing::error!(error = %e, "Snapshot d'exchange échoué");
        }
        tracing::debug!(
            step = %self.step_name,
            flow_id = %exchange.flow_id,
            exchange_id = %exchange.id,
            "Exchange archivé"
        );
        Ok(exchange)
    }
}
