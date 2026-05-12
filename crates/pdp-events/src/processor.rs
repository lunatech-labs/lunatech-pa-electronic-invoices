//! Processors pipeline qui émettent vers le bus.
//!
//! Le `LifecycleProcessor` est le remplaçant de `pdp_trace::TraceProcessor` :
//! il publie un événement par étape franchie. Il y a un constructeur par
//! variante de [`pdp_core::model::FlowStatus`] pour faciliter l'usage
//! dans les routes (`RouteBuilder::process(...)`).
//!
//! Le processor ne modifie pas l'exchange — il observe et publie.

use async_trait::async_trait;
use std::sync::Arc;

use pdp_core::error::PdpResult;
use pdp_core::exchange::Exchange;
use pdp_core::model::FlowStatus;
use pdp_core::processor::Processor;

use crate::bus::EventBus;
use crate::event::{Event, EventKind};

/// Émet un événement de cycle de vie à chaque passage dans le pipeline.
pub struct LifecycleProcessor {
    bus: EventBus,
    kind: EventKind,
    step: String,
}

impl LifecycleProcessor {
    pub fn new(bus: EventBus, kind: EventKind, step: impl Into<String>) -> Self {
        Self {
            bus,
            kind,
            step: step.into(),
        }
    }

    pub fn received(bus: EventBus) -> Self {
        Self::new(bus, EventKind::Received, "reception")
    }
    pub fn parsing(bus: EventBus) -> Self {
        Self::new(bus, EventKind::Parsing, "parsing")
    }
    pub fn parsed(bus: EventBus) -> Self {
        Self::new(bus, EventKind::Parsed, "parsing")
    }
    pub fn validating(bus: EventBus) -> Self {
        Self::new(bus, EventKind::Validating, "validation")
    }
    pub fn validated(bus: EventBus) -> Self {
        Self::new(bus, EventKind::Validated, "validation")
    }
    pub fn transforming(bus: EventBus) -> Self {
        Self::new(bus, EventKind::Transforming, "transformation")
    }
    pub fn transformed(bus: EventBus) -> Self {
        Self::new(bus, EventKind::Transformed, "transformation")
    }
    pub fn distributing(bus: EventBus) -> Self {
        Self::new(bus, EventKind::Distributing, "distribution")
    }
    pub fn distributed(bus: EventBus) -> Self {
        Self::new(bus, EventKind::Distributed, "distribution")
    }
    pub fn waiting_ack(bus: EventBus) -> Self {
        Self::new(bus, EventKind::WaitingAck, "ack")
    }
    pub fn acknowledged(bus: EventBus) -> Self {
        Self::new(bus, EventKind::Acknowledged, "ack")
    }
    pub fn rejected(bus: EventBus) -> Self {
        Self::new(bus, EventKind::Rejected, "rejection")
    }
    pub fn cancelled(bus: EventBus) -> Self {
        Self::new(bus, EventKind::Cancelled, "annulation")
    }
    pub fn error(bus: EventBus) -> Self {
        Self::new(bus, EventKind::Error, "erreur")
    }

    /// Variante générique : choisit l'événement à émettre selon le `FlowStatus`
    /// courant de l'exchange. Utile pour un processor unique inséré à un point
    /// charnière du pipeline.
    pub fn from_status(bus: EventBus) -> StatusReflectingProcessor {
        StatusReflectingProcessor { bus }
    }
}

#[async_trait]
impl Processor for LifecycleProcessor {
    fn name(&self) -> &str {
        "LifecycleProcessor"
    }

    async fn process(&self, exchange: Exchange) -> PdpResult<Exchange> {
        let event = build_event(&exchange, self.kind, &self.step);
        if let Err(e) = self.bus.publish(event).await {
            tracing::error!(
                error = %e,
                flow_id = %exchange.flow_id,
                kind = self.kind.as_code(),
                "Publication d'événement échouée (non bloquant pour le pipeline)"
            );
        }
        Ok(exchange)
    }
}

/// Variante qui dérive l'événement du `FlowStatus` courant de l'exchange.
pub struct StatusReflectingProcessor {
    bus: EventBus,
}

#[async_trait]
impl Processor for StatusReflectingProcessor {
    fn name(&self) -> &str {
        "StatusReflectingProcessor"
    }

    async fn process(&self, exchange: Exchange) -> PdpResult<Exchange> {
        let kind: EventKind = exchange.status.clone().into();
        let step = step_for_status(&exchange.status);
        let event = build_event(&exchange, kind, step);
        if let Err(e) = self.bus.publish(event).await {
            tracing::error!(
                error = %e,
                flow_id = %exchange.flow_id,
                kind = kind.as_code(),
                "Publication d'événement échouée (non bloquant)"
            );
        }
        Ok(exchange)
    }
}

fn step_for_status(s: &FlowStatus) -> &'static str {
    match s {
        FlowStatus::Received => "reception",
        FlowStatus::Parsing | FlowStatus::Parsed => "parsing",
        FlowStatus::Validating | FlowStatus::Validated => "validation",
        FlowStatus::Transforming | FlowStatus::Transformed => "transformation",
        FlowStatus::Distributing | FlowStatus::Distributed => "distribution",
        FlowStatus::WaitingAck | FlowStatus::Acknowledged => "ack",
        FlowStatus::Rejected => "rejection",
        FlowStatus::Cancelled => "annulation",
        FlowStatus::Error => "erreur",
    }
}

fn build_event(exchange: &Exchange, kind: EventKind, step: &str) -> Event {
    let mut e = Event::new(exchange.flow_id, kind).with_step(step);

    if let Some(route_id) = exchange.get_header("route.id") {
        e = e.with_route(route_id.clone());
    }
    if let Some(siren) = exchange.tenant_siren() {
        e = e.with_tenant(siren.to_string());
    }
    if let Some(inv) = exchange.invoice.as_ref() {
        e = e.with_invoice_key(inv.key_string());
    }
    if let Some(filename) = exchange.source_filename.as_ref() {
        e = e.with_message(format!("{} — {}", step, filename));
    }
    if let Some(last_err) = exchange.errors.last() {
        let detail = last_err
            .detail
            .clone()
            .unwrap_or_else(|| last_err.message.clone());
        e = e.with_error(detail);
    }
    e
}

/// Construit un Arc<dyn Subscriber> à partir d'une closure async (utile pour les tests).
///
/// Garde-le dans `pdp-events` pour ne pas dépendre de `tokio_stream` ailleurs.
pub mod test_helpers {
    use super::*;
    use crate::bus::Subscriber;
    use crate::error::EventResult;
    use crate::event::Event as Ev;
    use std::sync::Mutex;

    /// Subscriber qui capture les événements en mémoire (utilisé par les tests).
    pub struct CaptureSubscriber {
        pub id: String,
        pub captured: Mutex<Vec<Ev>>,
    }

    impl CaptureSubscriber {
        pub fn new(id: impl Into<String>) -> Arc<Self> {
            Arc::new(Self {
                id: id.into(),
                captured: Mutex::new(Vec::new()),
            })
        }

        pub fn events(&self) -> Vec<Ev> {
            self.captured.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl Subscriber for CaptureSubscriber {
        fn id(&self) -> &str {
            &self.id
        }
        async fn handle(&self, event: &Ev) -> EventResult<()> {
            self.captured.lock().unwrap().push(event.clone());
            Ok(())
        }
    }
}
