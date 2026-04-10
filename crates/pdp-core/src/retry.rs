//! Mécanismes de résilience pour les producers : retry avec backoff exponentiel
//! et dead-letter queue pour les messages en échec.

use async_trait::async_trait;
use std::path::PathBuf;

use crate::endpoint::Producer;
use crate::error::PdpResult;
use crate::exchange::Exchange;

// ---------------------------------------------------------------------------
// RetryProducer
// ---------------------------------------------------------------------------

/// Producer wrapper qui réessaie l'envoi en cas d'erreur transitoire.
///
/// Le délai entre chaque tentative suit un backoff exponentiel :
/// `delay = min(initial_delay_ms * multiplier^attempt, max_delay_ms)`
pub struct RetryProducer {
    inner: Box<dyn Producer>,
    max_retries: u32,
    initial_delay_ms: u64,
    max_delay_ms: u64,
    multiplier: f64,
}

impl RetryProducer {
    /// Crée un nouveau `RetryProducer` avec les paramètres de backoff.
    pub fn new(
        inner: Box<dyn Producer>,
        max_retries: u32,
        initial_delay_ms: u64,
        max_delay_ms: u64,
        multiplier: f64,
    ) -> Self {
        Self {
            inner,
            max_retries,
            initial_delay_ms,
            max_delay_ms,
            multiplier,
        }
    }

    /// Crée un `RetryProducer` avec les paramètres par défaut :
    /// 3 tentatives, délai initial 1s, max 30s, facteur 2.0.
    pub fn with_defaults(inner: Box<dyn Producer>) -> Self {
        Self::new(inner, 3, 1000, 30_000, 2.0)
    }

    /// Calcule le délai pour une tentative donnée (0-indexed).
    fn delay_for_attempt(&self, attempt: u32) -> u64 {
        let delay = self.initial_delay_ms as f64 * self.multiplier.powi(attempt as i32);
        (delay as u64).min(self.max_delay_ms)
    }
}

#[async_trait]
impl Producer for RetryProducer {
    fn name(&self) -> &str {
        self.inner.name()
    }

    async fn send(&self, exchange: Exchange) -> PdpResult<Exchange> {
        let mut last_error = None;

        for attempt in 0..=self.max_retries {
            match self.inner.send(exchange.clone()).await {
                Ok(result) => {
                    if attempt > 0 {
                        tracing::info!(
                            producer = self.inner.name(),
                            exchange_id = %exchange.id,
                            attempt = attempt + 1,
                            "Envoi réussi après {} tentative(s)",
                            attempt + 1,
                        );
                    }
                    return Ok(result);
                }
                Err(err) => {
                    last_error = Some(err);

                    if attempt < self.max_retries {
                        let delay = self.delay_for_attempt(attempt);
                        tracing::warn!(
                            producer = self.inner.name(),
                            exchange_id = %exchange.id,
                            attempt = attempt + 1,
                            max_retries = self.max_retries,
                            delay_ms = delay,
                            error = %last_error.as_ref().unwrap(),
                            "Échec de l'envoi, nouvelle tentative dans {} ms",
                            delay,
                        );
                        tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
                    } else {
                        tracing::error!(
                            producer = self.inner.name(),
                            exchange_id = %exchange.id,
                            max_retries = self.max_retries,
                            error = %last_error.as_ref().unwrap(),
                            "Toutes les tentatives épuisées, abandon",
                        );
                    }
                }
            }
        }

        Err(last_error.unwrap())
    }
}

// ---------------------------------------------------------------------------
// DeadLetterProducer
// ---------------------------------------------------------------------------

/// Producer qui écrit les exchanges en échec dans un répertoire dead-letter.
///
/// En cas d'erreur du producer interne :
/// - Le body de l'exchange est sauvegardé dans `{dead_letter_path}/{exchange_id}_{filename}`
/// - Un fichier JSON sidecar contient les détails de l'erreur, headers et propriétés
/// - L'erreur originale est retournée (non masquée)
pub struct DeadLetterProducer {
    inner: Box<dyn Producer>,
    dead_letter_path: PathBuf,
}

impl DeadLetterProducer {
    /// Crée un nouveau `DeadLetterProducer`.
    pub fn new(inner: Box<dyn Producer>, dead_letter_path: PathBuf) -> Self {
        Self {
            inner,
            dead_letter_path,
        }
    }

    /// Écrit l'exchange dans le répertoire dead-letter.
    fn write_dead_letter(
        &self,
        exchange: &Exchange,
        error: &crate::error::PdpError,
    ) {
        // Créer le répertoire si nécessaire
        if let Err(e) = std::fs::create_dir_all(&self.dead_letter_path) {
            tracing::error!(
                path = %self.dead_letter_path.display(),
                error = %e,
                "Impossible de créer le répertoire dead-letter"
            );
            return;
        }

        let filename = exchange
            .source_filename
            .as_deref()
            .unwrap_or("unknown");
        let base_name = format!("{}_{}", exchange.id, filename);

        // Écrire le body
        let body_path = self.dead_letter_path.join(&base_name);
        if let Err(e) = std::fs::write(&body_path, &exchange.body) {
            tracing::error!(
                path = %body_path.display(),
                error = %e,
                "Impossible d'écrire le body dead-letter"
            );
            return;
        }

        // Écrire le sidecar JSON avec les métadonnées
        let sidecar = serde_json::json!({
            "exchange_id": exchange.id.to_string(),
            "flow_id": exchange.flow_id.to_string(),
            "source_filename": exchange.source_filename,
            "error": error.to_string(),
            "error_debug": format!("{:?}", error),
            "headers": exchange.headers,
            "properties": exchange.properties,
            "status": format!("{}", exchange.status),
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });

        let sidecar_path = self.dead_letter_path.join(format!("{}.json", base_name));
        if let Err(e) = std::fs::write(&sidecar_path, serde_json::to_string_pretty(&sidecar).unwrap_or_default()) {
            tracing::error!(
                path = %sidecar_path.display(),
                error = %e,
                "Impossible d'écrire le sidecar dead-letter"
            );
            return;
        }

        tracing::warn!(
            producer = self.inner.name(),
            exchange_id = %exchange.id,
            dead_letter_path = %body_path.display(),
            "Exchange écrit dans la dead-letter queue"
        );
    }
}

#[async_trait]
impl Producer for DeadLetterProducer {
    fn name(&self) -> &str {
        self.inner.name()
    }

    async fn send(&self, exchange: Exchange) -> PdpResult<Exchange> {
        match self.inner.send(exchange.clone()).await {
            Ok(result) => Ok(result),
            Err(err) => {
                self.write_dead_letter(&exchange, &err);
                Err(err)
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::PdpError;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    // --- Mock Producer qui échoue N fois puis réussit ---

    struct FailingProducer {
        fail_count: u32,
        attempts: Arc<AtomicU32>,
    }

    impl FailingProducer {
        fn new(fail_count: u32) -> (Self, Arc<AtomicU32>) {
            let attempts = Arc::new(AtomicU32::new(0));
            (
                Self {
                    fail_count,
                    attempts: Arc::clone(&attempts),
                },
                attempts,
            )
        }
    }

    #[async_trait]
    impl Producer for FailingProducer {
        fn name(&self) -> &str {
            "mock-failing"
        }

        async fn send(&self, exchange: Exchange) -> PdpResult<Exchange> {
            let attempt = self.attempts.fetch_add(1, Ordering::SeqCst);
            if attempt < self.fail_count {
                Err(PdpError::SftpError(format!(
                    "Erreur simulée (tentative {})",
                    attempt + 1
                )))
            } else {
                Ok(exchange)
            }
        }
    }

    // --- Mock Producer qui échoue toujours ---

    struct AlwaysFailProducer;

    #[async_trait]
    impl Producer for AlwaysFailProducer {
        fn name(&self) -> &str {
            "mock-always-fail"
        }

        async fn send(&self, _exchange: Exchange) -> PdpResult<Exchange> {
            Err(PdpError::SftpError("Erreur permanente".to_string()))
        }
    }

    // --- Tests RetryProducer ---

    #[tokio::test]
    async fn test_retry_succes_immediat() {
        let (producer, attempts) = FailingProducer::new(0);
        let retry = RetryProducer::new(Box::new(producer), 3, 10, 100, 2.0);

        let exchange = Exchange::new(b"<Invoice/>".to_vec());
        let result = retry.send(exchange).await;

        assert!(result.is_ok());
        assert_eq!(attempts.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_retry_succes_apres_echecs() {
        // Échoue 2 fois puis réussit
        let (producer, attempts) = FailingProducer::new(2);
        let retry = RetryProducer::new(Box::new(producer), 3, 10, 100, 2.0);

        let exchange = Exchange::new(b"<Invoice/>".to_vec());
        let result = retry.send(exchange).await;

        assert!(result.is_ok());
        // 2 échecs + 1 succès = 3 tentatives
        assert_eq!(attempts.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_retry_toutes_tentatives_epuisees() {
        let (producer, attempts) = FailingProducer::new(10);
        let retry = RetryProducer::new(Box::new(producer), 3, 10, 100, 2.0);

        let exchange = Exchange::new(b"<Invoice/>".to_vec());
        let result = retry.send(exchange).await;

        assert!(result.is_err());
        // 1 essai initial + 3 retries = 4 tentatives
        assert_eq!(attempts.load(Ordering::SeqCst), 4);

        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Erreur SFTP"));
    }

    #[tokio::test]
    async fn test_retry_with_defaults() {
        let retry = RetryProducer::with_defaults(Box::new(AlwaysFailProducer));
        assert_eq!(retry.max_retries, 3);
        assert_eq!(retry.initial_delay_ms, 1000);
        assert_eq!(retry.max_delay_ms, 30_000);
        assert!((retry.multiplier - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_delay_backoff_exponentiel() {
        let (producer, _) = FailingProducer::new(0);
        let retry = RetryProducer::new(Box::new(producer), 5, 1000, 30_000, 2.0);

        assert_eq!(retry.delay_for_attempt(0), 1000);   // 1000 * 2^0 = 1000
        assert_eq!(retry.delay_for_attempt(1), 2000);   // 1000 * 2^1 = 2000
        assert_eq!(retry.delay_for_attempt(2), 4000);   // 1000 * 2^2 = 4000
        assert_eq!(retry.delay_for_attempt(3), 8000);   // 1000 * 2^3 = 8000
        assert_eq!(retry.delay_for_attempt(4), 16000);  // 1000 * 2^4 = 16000
        assert_eq!(retry.delay_for_attempt(5), 30_000); // 1000 * 2^5 = 32000 → plafonné à 30000
    }

    // --- Tests DeadLetterProducer ---

    #[tokio::test]
    async fn test_dead_letter_ecrit_fichiers_en_echec() {
        let dir = tempfile::tempdir().unwrap();
        let dl_path = dir.path().join("dead-letter");

        let producer = DeadLetterProducer::new(
            Box::new(AlwaysFailProducer),
            dl_path.clone(),
        );

        let exchange = Exchange::new(b"<Invoice>DLQ</Invoice>".to_vec())
            .with_filename("facture_test.xml");

        let result = producer.send(exchange.clone()).await;

        // L'erreur originale est retournée
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Erreur permanente"));

        // Vérifier que le répertoire dead-letter a été créé
        assert!(dl_path.exists());

        // Vérifier le body
        let body_file = dl_path.join(format!("{}_facture_test.xml", exchange.id));
        assert!(body_file.exists(), "Le fichier body dead-letter doit exister");
        let body_content = std::fs::read(&body_file).unwrap();
        assert_eq!(body_content, b"<Invoice>DLQ</Invoice>");

        // Vérifier le sidecar JSON
        let sidecar_file = dl_path.join(format!("{}_facture_test.xml.json", exchange.id));
        assert!(sidecar_file.exists(), "Le fichier sidecar JSON doit exister");
        let sidecar_content = std::fs::read_to_string(&sidecar_file).unwrap();
        let sidecar: serde_json::Value = serde_json::from_str(&sidecar_content).unwrap();

        assert_eq!(sidecar["exchange_id"], exchange.id.to_string());
        assert_eq!(sidecar["source_filename"], "facture_test.xml");
        assert!(sidecar["error"].as_str().unwrap().contains("Erreur permanente"));
        assert!(sidecar["timestamp"].as_str().is_some());
    }

    #[tokio::test]
    async fn test_dead_letter_succes_pas_de_fichier() {
        let dir = tempfile::tempdir().unwrap();
        let dl_path = dir.path().join("dead-letter");

        let (inner, _) = FailingProducer::new(0); // réussit toujours
        let producer = DeadLetterProducer::new(
            Box::new(inner),
            dl_path.clone(),
        );

        let exchange = Exchange::new(b"<Invoice/>".to_vec())
            .with_filename("ok.xml");

        let result = producer.send(exchange).await;
        assert!(result.is_ok());

        // Aucun fichier dead-letter ne doit exister
        assert!(!dl_path.exists());
    }

    #[tokio::test]
    async fn test_dead_letter_sans_filename() {
        let dir = tempfile::tempdir().unwrap();
        let dl_path = dir.path().join("dead-letter");

        let producer = DeadLetterProducer::new(
            Box::new(AlwaysFailProducer),
            dl_path.clone(),
        );

        // Exchange sans filename
        let exchange = Exchange::new(b"body".to_vec());

        let result = producer.send(exchange.clone()).await;
        assert!(result.is_err());

        // Vérifie que le fichier utilise "unknown" comme nom
        let body_file = dl_path.join(format!("{}_unknown", exchange.id));
        assert!(body_file.exists());
    }

    #[tokio::test]
    async fn test_dead_letter_headers_et_properties() {
        let dir = tempfile::tempdir().unwrap();
        let dl_path = dir.path().join("dead-letter");

        let producer = DeadLetterProducer::new(
            Box::new(AlwaysFailProducer),
            dl_path.clone(),
        );

        let mut exchange = Exchange::new(b"body".to_vec())
            .with_filename("test.xml");
        exchange.set_header("route.id", "route-sftp");
        exchange.set_property("format", "CII");

        let result = producer.send(exchange.clone()).await;
        assert!(result.is_err());

        // Vérifier que le sidecar contient les headers et properties
        let sidecar_file = dl_path.join(format!("{}_test.xml.json", exchange.id));
        let sidecar: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&sidecar_file).unwrap()).unwrap();

        assert_eq!(sidecar["headers"]["route.id"], "route-sftp");
        assert_eq!(sidecar["properties"]["format"], "CII");
    }

    // --- Test combiné : RetryProducer + DeadLetterProducer ---

    #[tokio::test]
    async fn test_retry_puis_dead_letter() {
        let dir = tempfile::tempdir().unwrap();
        let dl_path = dir.path().join("dead-letter");

        // AlwaysFailProducer → RetryProducer (2 retries, délai court) → DeadLetterProducer
        let inner = RetryProducer::new(
            Box::new(AlwaysFailProducer),
            2,  // max_retries
            10, // initial_delay_ms (court pour le test)
            50,
            2.0,
        );
        let producer = DeadLetterProducer::new(Box::new(inner), dl_path.clone());

        let exchange = Exchange::new(b"<Invoice/>".to_vec())
            .with_filename("echec_total.xml");

        let result = producer.send(exchange.clone()).await;
        assert!(result.is_err());

        // Le fichier dead-letter doit exister
        let body_file = dl_path.join(format!("{}_echec_total.xml", exchange.id));
        assert!(body_file.exists());
    }
}
