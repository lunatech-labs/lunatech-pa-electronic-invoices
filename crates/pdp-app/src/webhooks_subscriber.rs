//! Subscriber qui relie le bus `pdp-events` aux webhooks AFNOR.
//!
//! Mapping `EventKind` → événement AFNOR webhook (XP Z12-013 §5.4) :
//!
//! | EventKind                              | webhook event       | ackStatus |
//! |----------------------------------------|---------------------|-----------|
//! | `Received`                             | `flow.received`     | `Pending` |
//! | `Validated`, `Distributed`, `Acknowledged` | `flow.ack.updated`  | `Ok`      |
//! | `Rejected`, `Error`                    | `flow.ack.updated`  | `Error`   |
//! | autres                                 | (ignoré)            | —         |
//!
//! Les événements intermédiaires (`Parsing`, `Transforming`, etc.) ne
//! déclenchent pas de webhook AFNOR : ils restent disponibles dans le journal
//! d'audit `events` (cf. `EventStore::list_by_flow`).
//!
//! Ce subscriber dé-duplique implicitement : le watermark garantit qu'un même
//! événement ne sera pas dispatché deux fois en condition nominale, et la
//! clef `event.id` dans `event_deliveries` rend les rejouages détectables.

use std::sync::Arc;

use async_trait::async_trait;
use pdp_events::{Event, EventKind, EventResult, Subscriber};

use crate::webhooks::{WebhookDispatcher, WebhookEventType, WebhookStore};

/// Subscriber « webhooks AFNOR ». À enregistrer auprès d'un
/// [`pdp_events::DispatcherWorker`].
pub struct WebhooksSubscriber {
    id: String,
    store: Arc<WebhookStore>,
    /// Direction du flux pour ce subscriber (« In » ou « Out »).
    /// On instancie habituellement deux subscribers (un par direction).
    flow_direction: String,
}

impl WebhooksSubscriber {
    pub fn new(store: Arc<WebhookStore>, flow_direction: impl Into<String>) -> Arc<Self> {
        let dir = flow_direction.into();
        Arc::new(Self {
            id: format!("webhooks-{}", dir.to_lowercase()),
            store,
            flow_direction: dir,
        })
    }

    fn map_event(kind: EventKind) -> Option<(WebhookEventType, &'static str)> {
        match kind {
            EventKind::Received => Some((WebhookEventType::FlowReceived, "Pending")),
            EventKind::Validated | EventKind::Distributed | EventKind::Acknowledged => {
                Some((WebhookEventType::FlowAckUpdated, "Ok"))
            }
            EventKind::Rejected | EventKind::Error => {
                Some((WebhookEventType::FlowAckUpdated, "Error"))
            }
            _ => None,
        }
    }
}

#[async_trait]
impl Subscriber for WebhooksSubscriber {
    fn id(&self) -> &str {
        &self.id
    }

    fn accepts(&self, event: &Event) -> bool {
        Self::map_event(event.kind).is_some()
    }

    async fn handle(&self, event: &Event) -> EventResult<()> {
        let (webhook_event, ack_status) = match Self::map_event(event.kind) {
            Some(m) => m,
            None => return Ok(()),
        };

        let flow_type = event
            .payload
            .as_ref()
            .and_then(|p| p.get("flow_type"))
            .and_then(|v| v.as_str())
            .map(String::from)
            .or_else(|| {
                // Fallback raisonnable basé sur l'invoice_key.
                event.invoice_key.as_ref().map(|_| "Invoice".to_string())
            })
            .unwrap_or_else(|| "Unknown".to_string());

        let dispatcher = WebhookDispatcher::new(self.store.clone());
        dispatcher
            .dispatch(
                webhook_event,
                &event.flow_id.to_string(),
                &flow_type,
                &self.flow_direction,
                Some(ack_status),
            )
            .await;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pdp_events::Event as Ev;
    use uuid::Uuid;

    #[test]
    fn maps_lifecycle_to_afnor() {
        assert!(matches!(
            WebhooksSubscriber::map_event(EventKind::Received),
            Some((WebhookEventType::FlowReceived, "Pending"))
        ));
        assert!(matches!(
            WebhooksSubscriber::map_event(EventKind::Validated),
            Some((WebhookEventType::FlowAckUpdated, "Ok"))
        ));
        assert!(matches!(
            WebhooksSubscriber::map_event(EventKind::Rejected),
            Some((WebhookEventType::FlowAckUpdated, "Error"))
        ));
        assert!(WebhooksSubscriber::map_event(EventKind::Parsing).is_none());
        assert!(WebhooksSubscriber::map_event(EventKind::Transformed).is_none());
    }

    #[test]
    fn accepts_filters_unmapped_kinds() {
        let sub = WebhooksSubscriber::new(Arc::new(WebhookStore::new()), "In");
        let e_ok = Ev::new(Uuid::new_v4(), EventKind::Validated);
        let e_skip = Ev::new(Uuid::new_v4(), EventKind::Parsing);
        assert!(sub.accepts(&e_ok));
        assert!(!sub.accepts(&e_skip));
    }
}
