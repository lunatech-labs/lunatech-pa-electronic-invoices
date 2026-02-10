use async_trait::async_trait;
use pdp_core::error::PdpResult;
use pdp_core::exchange::Exchange;
use pdp_core::model::{FlowEvent, FlowStatus};
use pdp_core::processor::Processor;

use crate::store::TraceStore;
use std::sync::Arc;

/// Processor de traçabilité : enregistre chaque passage dans le pipeline
pub struct TraceProcessor {
    store: Arc<TraceStore>,
    step_name: String,
    status: FlowStatus,
}

impl TraceProcessor {
    pub fn new(store: Arc<TraceStore>, step_name: &str, status: FlowStatus) -> Self {
        Self {
            store,
            step_name: step_name.to_string(),
            status,
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
impl Processor for TraceProcessor {
    fn name(&self) -> &str {
        "TraceProcessor"
    }

    async fn process(&self, exchange: Exchange) -> PdpResult<Exchange> {
        let route_id = exchange
            .get_header("route.id")
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());

        let invoice_key = exchange.invoice.as_ref().map(|i| i.key_string());

        let message = format!(
            "Étape '{}' - fichier: {} | statut: {}",
            self.step_name,
            exchange.source_filename.as_deref().unwrap_or("N/A"),
            self.status,
        );

        let mut event = FlowEvent::new(exchange.flow_id, &route_id, self.status.clone(), &message);
        if let Some(ref key) = invoice_key {
            event = event.with_invoice_key(key);
        }

        // Enregistrer l'événement
        if let Err(e) = self.store.record_event(&event).await {
            tracing::error!(error = %e, "Erreur d'enregistrement de l'événement de trace");
        }

        // Enregistrer l'état de l'exchange
        if let Err(e) = self.store.record_exchange(&exchange).await {
            tracing::error!(error = %e, "Erreur d'enregistrement de l'exchange");
        }

        tracing::debug!(
            step = %self.step_name,
            flow_id = %exchange.flow_id,
            exchange_id = %exchange.id,
            "Trace enregistrée"
        );

        Ok(exchange)
    }
}
