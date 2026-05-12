//! Façade de publication : `EventBus`.
//!
//! Le bus est la **seule** API pour émettre un événement. Il garantit
//! que l'événement est persisté dans l'outbox (table `events`) avant
//! d'être considéré comme accepté. La distribution vers les
//! [`Subscriber`]s est gérée par un [`crate::dispatcher::DispatcherWorker`]
//! qui s'exécute en arrière-plan.
//!
//! Cette séparation publish ↔ deliver est ce qui rend la livraison
//! **at-least-once** : l'écriture de l'événement ne dépend d'aucun
//! subscriber, et la consommation est rejouable.

use async_trait::async_trait;
use std::sync::Arc;

use crate::error::EventResult;
use crate::event::Event;
use crate::store::EventStore;

/// Bus d'événements.
///
/// Cloner le bus est cheap : c'est un wrap d'`Arc<EventStore>`.
#[derive(Clone)]
pub struct EventBus {
    store: Arc<EventStore>,
}

impl EventBus {
    pub fn new(store: Arc<EventStore>) -> Self {
        Self { store }
    }

    pub fn store(&self) -> &Arc<EventStore> {
        &self.store
    }

    /// Publie un événement (persistance immédiate dans l'outbox).
    ///
    /// Retourne l'événement enrichi de son `sequence`. Tant que cet appel
    /// n'a pas réussi, **l'événement n'existe pas**.
    pub async fn publish(&self, event: Event) -> EventResult<Event> {
        let persisted = self.store.append(&event).await?;
        tracing::debug!(
            event_id = %persisted.id,
            sequence = ?persisted.sequence,
            kind = persisted.kind.as_code(),
            flow_id = %persisted.flow_id,
            "Événement publié"
        );
        Ok(persisted)
    }
}

/// Consommateur d'événements.
///
/// Les implémentations doivent être **idempotentes** : le bus garantit
/// at-least-once, donc un événement peut être livré plusieurs fois (en cas
/// de crash entre le traitement et l'avancement du watermark).
///
/// L'identifiant `id()` doit être stable entre redémarrages — il sert de
/// clef dans `event_subscriptions`.
#[async_trait]
pub trait Subscriber: Send + Sync {
    /// Identifiant stable, persistant. Ex: `"webhook-dispatcher"`,
    /// `"audit-elastic"`, `"metrics-prom"`.
    fn id(&self) -> &str;

    /// Filtre côté subscriber. Renvoie `false` pour ignorer.
    ///
    /// **Attention** : un événement ignoré fait quand même avancer le
    /// watermark. C'est volontaire (on n'a pas envie de bloquer la
    /// progression pour des événements qu'on ne veut pas traiter).
    fn accepts(&self, _event: &Event) -> bool {
        true
    }

    /// Traite un événement. Renvoyer `Err` empêche l'avancement du
    /// watermark pour cet événement — il sera rejoué au prochain tick.
    async fn handle(&self, event: &Event) -> EventResult<()>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct CountingSub {
        id: String,
        count: AtomicUsize,
    }

    #[async_trait]
    impl Subscriber for CountingSub {
        fn id(&self) -> &str {
            &self.id
        }
        async fn handle(&self, _event: &Event) -> EventResult<()> {
            self.count.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    #[tokio::test]
    async fn subscriber_default_accepts_everything() {
        let s = CountingSub {
            id: "x".into(),
            count: AtomicUsize::new(0),
        };
        let e = Event::new(uuid::Uuid::new_v4(), crate::event::EventKind::Received);
        assert!(s.accepts(&e));
    }
}
