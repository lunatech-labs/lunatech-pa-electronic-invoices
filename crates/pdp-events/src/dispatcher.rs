//! `DispatcherWorker` : fan-out de l'outbox vers les subscribers.
//!
//! Pour chaque subscriber enregistré, le worker :
//! 1. Lit le watermark courant (`event_subscriptions.last_sequence`).
//! 2. `fetch_after` un lot d'événements de l'outbox.
//! 3. Pour chaque événement : `accepts` puis `handle`.
//! 4. Avance le watermark à la séquence du dernier événement **traité avec succès
//!    ou explicitement ignoré**.
//!
//! Sémantique **at-least-once** : si `handle` renvoie `Err`, le watermark
//! n'avance pas pour cet événement — il sera rejoué au tick suivant. L'erreur
//! est tracée dans `event_deliveries` pour ne pas bloquer indéfiniment
//! l'observabilité.
//!
//! Le worker maintient des subscribers indépendants : un subscriber lent ou
//! en erreur ne bloque pas les autres.

use std::sync::Arc;
use std::time::Duration;

use tokio::sync::Notify;
use tokio::task::JoinHandle;

use crate::bus::Subscriber;
use crate::error::EventResult;
use crate::store::EventStore;

/// Configuration du dispatcher.
#[derive(Debug, Clone, Copy)]
pub struct DispatcherConfig {
    /// Intervalle entre deux polls de l'outbox.
    pub poll_interval: Duration,
    /// Nombre maximum d'événements lus par tick.
    pub batch_size: i64,
}

impl Default for DispatcherConfig {
    fn default() -> Self {
        Self {
            poll_interval: Duration::from_millis(500),
            batch_size: 100,
        }
    }
}

/// Dispatcher tournant en tâche tokio. Une instance par subscriber.
pub struct DispatcherWorker {
    store: Arc<EventStore>,
    subscriber: Arc<dyn Subscriber>,
    config: DispatcherConfig,
    notify: Arc<Notify>,
}

impl DispatcherWorker {
    pub fn new(store: Arc<EventStore>, subscriber: Arc<dyn Subscriber>) -> Self {
        Self {
            store,
            subscriber,
            config: DispatcherConfig::default(),
            notify: Arc::new(Notify::new()),
        }
    }

    pub fn with_config(mut self, config: DispatcherConfig) -> Self {
        self.config = config;
        self
    }

    /// Permet à un producteur de réveiller le worker (réduit la latence).
    pub fn waker(&self) -> Arc<Notify> {
        self.notify.clone()
    }

    /// Démarre le worker en tâche tokio. La tâche s'arrête quand le handle
    /// est abandonné (drop).
    pub fn spawn(self) -> JoinHandle<()> {
        tokio::spawn(async move { self.run().await })
    }

    /// Boucle principale. Tick périodique ou réveil par `notify`.
    pub async fn run(self) {
        let sub_id = self.subscriber.id().to_string();
        tracing::info!(subscriber = %sub_id, "DispatcherWorker démarré");

        loop {
            if let Err(e) = self.tick().await {
                tracing::warn!(
                    subscriber = %sub_id,
                    error = %e,
                    "Erreur dans le tick du dispatcher (retry au prochain cycle)"
                );
            }

            tokio::select! {
                _ = tokio::time::sleep(self.config.poll_interval) => {}
                _ = self.notify.notified() => {}
            }
        }
    }

    /// Exécute un cycle. Public pour les tests qui pilotent à la main.
    pub async fn tick(&self) -> EventResult<usize> {
        let sub_id = self.subscriber.id();
        let watermark = self.store.get_watermark(sub_id).await?;
        let batch = self.store.fetch_after(watermark, self.config.batch_size).await?;

        if batch.is_empty() {
            return Ok(0);
        }

        let mut highest_ok: Option<i64> = None;
        let mut processed = 0usize;

        for event in &batch {
            let seq = match event.sequence {
                Some(s) => s,
                None => continue,
            };

            if !self.subscriber.accepts(event) {
                // Accept = false : on avance quand même (l'événement n'intéresse pas).
                highest_ok = Some(seq);
                continue;
            }

            match self.subscriber.handle(event).await {
                Ok(()) => {
                    let _ = self
                        .store
                        .record_delivery(sub_id, event.id, true, None)
                        .await;
                    highest_ok = Some(seq);
                    processed += 1;
                }
                Err(e) => {
                    tracing::warn!(
                        subscriber = %sub_id,
                        event_id = %event.id,
                        sequence = seq,
                        error = %e,
                        "Échec de handle, le watermark restera bloqué"
                    );
                    let _ = self
                        .store
                        .record_delivery(sub_id, event.id, false, Some(&e.to_string()))
                        .await;
                    // On stoppe le batch : at-least-once + ordre → on ne saute pas.
                    break;
                }
            }
        }

        if let Some(seq) = highest_ok {
            self.store.set_watermark(sub_id, seq).await?;
        }
        Ok(processed)
    }
}
