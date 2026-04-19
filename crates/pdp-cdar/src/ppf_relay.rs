//! Relais CDV vers le PPF (Flux 6).
//!
//! Certains CDV reçus de l'acheteur ou du vendeur doivent être relayés au PPF
//! via le Flux 6 (CDV statuts obligatoires, code interface FFE0654A).
//!
//! D'après l'onglet "Acteurs CDV" (XP Z12-012 Annexe A V1.2), les CDV
//! qui doivent être transmis au PPF sont :
//! - **210** Refusée (émis par l'acheteur)
//! - **212** Encaissée (émis par le vendeur)
//!
//! Les autres statuts (204, 205, 206, 207, 208, 209, 211, 214, 220) ne sont
//! PAS relayés au PPF.
//!
//! Note : les CDV 200 et 213 sont déjà gérés par le pipeline (CdarProcessor),
//! pas par ce relay.

use async_trait::async_trait;
use std::sync::Arc;

use pdp_core::endpoint::Producer;
use pdp_core::error::PdpResult;
use pdp_core::exchange::Exchange;
use pdp_core::processor::Processor;

/// Codes CDV qui doivent être relayés au PPF via Flux 6
const CDV_RELAY_TO_PPF: &[u32] = &[210, 212];

/// Processeur qui relaye certains CDV reçus vers le PPF via Flux 6.
///
/// Doit être placé APRÈS le `CdvReceptionProcessor` dans le pipeline.
/// Quand un CDV 210 (Refusée) ou 212 (Encaissée) est reçu, il est
/// transmis au PPF via le producer PPF SFTP configuré pour Flux 6.
///
/// Le pipeline n'est pas bloqué en cas d'échec d'envoi.
pub struct CdvPpfRelayProcessor {
    /// Producer PPF pour envoyer le CDV (PpfSftpProducer ou mock)
    ppf_producer: Arc<dyn Producer>,
}

impl CdvPpfRelayProcessor {
    pub fn new(ppf_producer: Arc<dyn Producer>) -> Self {
        Self { ppf_producer }
    }

    /// Vérifie si un code CDV doit être relayé au PPF
    fn should_relay(status_code: u32) -> bool {
        CDV_RELAY_TO_PPF.contains(&status_code)
    }
}

#[async_trait]
impl Processor for CdvPpfRelayProcessor {
    fn name(&self) -> &str {
        "CdvPpfRelay"
    }

    async fn process(&self, exchange: Exchange) -> PdpResult<Exchange> {
        // Vérifier qu'un CDV a été reçu (parsé par CdvReceptionProcessor)
        let is_cdv_received = exchange
            .get_property("cdv.received")
            .map(|v| v == "true")
            .unwrap_or(false);

        if !is_cdv_received {
            return Ok(exchange);
        }

        // Récupérer le code statut
        let status_code: u32 = match exchange.get_property("cdv.status_code") {
            Some(code) => match code.parse() {
                Ok(c) => c,
                Err(_) => return Ok(exchange),
            },
            None => return Ok(exchange),
        };

        // Vérifier si ce CDV doit être relayé au PPF
        if !Self::should_relay(status_code) {
            tracing::debug!(
                exchange_id = %exchange.id,
                cdv_status = status_code,
                "CdvPpfRelay: CDV {} non concerné par le relais PPF",
                status_code
            );
            return Ok(exchange);
        }

        let cdv_doc_id = exchange
            .get_property("cdv.document_id")
            .cloned()
            .unwrap_or_else(|| exchange.id.to_string());

        tracing::info!(
            exchange_id = %exchange.id,
            cdv_status = status_code,
            cdv_doc_id = %cdv_doc_id,
            "CdvPpfRelay: relais du CDV {} vers le PPF (Flux 6 FFE0654A)",
            status_code
        );

        // Créer un Exchange dédié pour le Flux 6
        let mut relay_exchange = Exchange::new(exchange.body.clone());
        relay_exchange.set_header("document.type", "cdar");
        relay_exchange.set_header("source.protocol", "internal");
        relay_exchange = relay_exchange.with_flow_id(exchange.flow_id);

        // Métadonnées Flux 6
        relay_exchange.set_property("ppf.code_interface", "FFE0654A");
        relay_exchange.set_property("flow.syntax", "CDAR");
        relay_exchange.set_property("cdv.status_code", &status_code.to_string());
        relay_exchange.set_property("cdv.document_id", &cdv_doc_id);

        // Nom de fichier pour le Flux 6
        relay_exchange.source_filename = Some(format!("CDV_{}_{}.xml", status_code, cdv_doc_id));

        // Envoyer au PPF
        match self.ppf_producer.send(relay_exchange).await {
            Ok(_) => {
                tracing::info!(
                    flow_id = %exchange.flow_id,
                    cdv_status = status_code,
                    "CDV {} relayé au PPF via Flux 6",
                    status_code
                );
            }
            Err(e) => {
                tracing::error!(
                    flow_id = %exchange.flow_id,
                    cdv_status = status_code,
                    error = %e,
                    "Erreur lors du relais CDV {} au PPF",
                    status_code
                );
                // On ne bloque pas le pipeline
            }
        }

        Ok(exchange)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pdp_core::error::PdpError;
    use std::sync::Mutex;

    struct MockProducer {
        sent: Mutex<Vec<Exchange>>,
        fail: bool,
    }

    impl MockProducer {
        fn new() -> Self {
            Self { sent: Mutex::new(Vec::new()), fail: false }
        }
        fn failing() -> Self {
            Self { sent: Mutex::new(Vec::new()), fail: true }
        }
        fn sent_count(&self) -> usize {
            self.sent.lock().unwrap().len()
        }
        fn last_exchange(&self) -> Exchange {
            self.sent.lock().unwrap().last().unwrap().clone()
        }
    }

    #[async_trait]
    impl Producer for MockProducer {
        fn name(&self) -> &str { "mock-ppf" }
        async fn send(&self, exchange: Exchange) -> PdpResult<Exchange> {
            if self.fail {
                return Err(PdpError::RoutingError("PPF mock error".into()));
            }
            self.sent.lock().unwrap().push(exchange.clone());
            Ok(exchange)
        }
    }

    fn make_cdv_exchange(status_code: u32) -> Exchange {
        let mut ex = Exchange::new(b"<CdvResponse>CDV XML</CdvResponse>".to_vec());
        ex.set_property("cdv.received", "true");
        ex.set_property("cdv.status_code", &status_code.to_string());
        ex.set_property("cdv.document_id", &format!("CDV-{}-001", status_code));
        ex
    }

    // === CDV 210 (Refusée) — doit être relayé au PPF ===

    #[tokio::test]
    async fn test_relay_cdv_210_refusee_to_ppf() {
        let producer = Arc::new(MockProducer::new());
        let processor = CdvPpfRelayProcessor::new(producer.clone());

        let exchange = make_cdv_exchange(210);
        let result = processor.process(exchange).await.unwrap();

        assert_eq!(producer.sent_count(), 1);
        let sent = producer.last_exchange();
        assert_eq!(sent.get_property("ppf.code_interface").unwrap(), "FFE0654A");
        assert_eq!(sent.get_property("flow.syntax").unwrap(), "CDAR");
        assert_eq!(sent.get_property("cdv.status_code").unwrap(), "210");
        assert!(sent.source_filename.as_ref().unwrap().contains("CDV_210"));

        // L'exchange original est retourné inchangé
        assert_eq!(result.body, b"<CdvResponse>CDV XML</CdvResponse>");
    }

    // === CDV 212 (Encaissée) — doit être relayé au PPF ===

    #[tokio::test]
    async fn test_relay_cdv_212_encaissee_to_ppf() {
        let producer = Arc::new(MockProducer::new());
        let processor = CdvPpfRelayProcessor::new(producer.clone());

        let exchange = make_cdv_exchange(212);
        let result = processor.process(exchange).await.unwrap();

        assert_eq!(producer.sent_count(), 1);
        let sent = producer.last_exchange();
        assert_eq!(sent.get_property("ppf.code_interface").unwrap(), "FFE0654A");
        assert_eq!(sent.get_property("cdv.status_code").unwrap(), "212");
        assert!(result.body == b"<CdvResponse>CDV XML</CdvResponse>");
    }

    // === CDV 204 (Prise en charge) — ne doit PAS être relayé ===

    #[tokio::test]
    async fn test_no_relay_cdv_204() {
        let producer = Arc::new(MockProducer::new());
        let processor = CdvPpfRelayProcessor::new(producer.clone());

        let exchange = make_cdv_exchange(204);
        processor.process(exchange).await.unwrap();

        assert_eq!(producer.sent_count(), 0);
    }

    // === CDV 205 (Approuvée) — ne doit PAS être relayé ===

    #[tokio::test]
    async fn test_no_relay_cdv_205() {
        let producer = Arc::new(MockProducer::new());
        let processor = CdvPpfRelayProcessor::new(producer.clone());

        let exchange = make_cdv_exchange(205);
        processor.process(exchange).await.unwrap();

        assert_eq!(producer.sent_count(), 0);
    }

    // === CDV 207 (En litige) — ne doit PAS être relayé ===

    #[tokio::test]
    async fn test_no_relay_cdv_207() {
        let producer = Arc::new(MockProducer::new());
        let processor = CdvPpfRelayProcessor::new(producer.clone());

        let exchange = make_cdv_exchange(207);
        processor.process(exchange).await.unwrap();

        assert_eq!(producer.sent_count(), 0);
    }

    // === CDV 200 (Déposée) — ne doit PAS être relayé (déjà géré par le pipeline) ===

    #[tokio::test]
    async fn test_no_relay_cdv_200() {
        let producer = Arc::new(MockProducer::new());
        let processor = CdvPpfRelayProcessor::new(producer.clone());

        let exchange = make_cdv_exchange(200);
        processor.process(exchange).await.unwrap();

        assert_eq!(producer.sent_count(), 0);
    }

    // === Pas de CDV reçu — skip ===

    #[tokio::test]
    async fn test_skip_when_no_cdv_received() {
        let producer = Arc::new(MockProducer::new());
        let processor = CdvPpfRelayProcessor::new(producer.clone());

        let exchange = Exchange::new(b"<Invoice/>".to_vec());
        processor.process(exchange).await.unwrap();

        assert_eq!(producer.sent_count(), 0);
    }

    // === Erreur PPF ne bloque pas le pipeline ===

    #[tokio::test]
    async fn test_ppf_error_does_not_block() {
        let producer = Arc::new(MockProducer::failing());
        let processor = CdvPpfRelayProcessor::new(producer.clone());

        let exchange = make_cdv_exchange(210);
        let result = processor.process(exchange).await;

        assert!(result.is_ok(), "Le pipeline ne doit pas être bloqué par une erreur PPF");
    }

    // === Vérifier should_relay pour tous les codes ===

    #[test]
    fn test_should_relay_codes() {
        // Doivent être relayés
        assert!(CdvPpfRelayProcessor::should_relay(210));
        assert!(CdvPpfRelayProcessor::should_relay(212));

        // Ne doivent PAS être relayés
        assert!(!CdvPpfRelayProcessor::should_relay(200));
        assert!(!CdvPpfRelayProcessor::should_relay(201));
        assert!(!CdvPpfRelayProcessor::should_relay(202));
        assert!(!CdvPpfRelayProcessor::should_relay(203));
        assert!(!CdvPpfRelayProcessor::should_relay(204));
        assert!(!CdvPpfRelayProcessor::should_relay(205));
        assert!(!CdvPpfRelayProcessor::should_relay(206));
        assert!(!CdvPpfRelayProcessor::should_relay(207));
        assert!(!CdvPpfRelayProcessor::should_relay(208));
        assert!(!CdvPpfRelayProcessor::should_relay(209));
        assert!(!CdvPpfRelayProcessor::should_relay(211));
        assert!(!CdvPpfRelayProcessor::should_relay(213));
        assert!(!CdvPpfRelayProcessor::should_relay(214));
        assert!(!CdvPpfRelayProcessor::should_relay(220));
        assert!(!CdvPpfRelayProcessor::should_relay(221));
        assert!(!CdvPpfRelayProcessor::should_relay(501));
    }
}
