//! Processeur de renvoi CDV vers l'émetteur de la facture.
//!
//! Après que le [`CdarProcessor`](crate::processor::CdarProcessor) a généré un CDV
//! (stocké comme propriété `cdv.xml`), ce processeur crée un Exchange dédié contenant
//! le CDV et l'envoie vers l'émetteur via un Producer configuré.
//!
//! Le pipeline principal n'est pas bloqué en cas d'échec d'envoi du CDV.

use async_trait::async_trait;
use std::sync::Arc;

use pdp_core::endpoint::Producer;
use pdp_core::error::PdpResult;
use pdp_core::exchange::Exchange;
use pdp_core::processor::Processor;

/// Processeur qui envoie le CDV généré vers l'émetteur de la facture.
/// Doit être placé après le CdarProcessor dans le pipeline.
pub struct CdvReturnProcessor {
    /// Producer pour renvoyer le CDV (AFNOR Flow, SFTP, fichier, etc.)
    cdv_producer: Arc<dyn Producer>,
}

impl CdvReturnProcessor {
    pub fn new(cdv_producer: Arc<dyn Producer>) -> Self {
        Self { cdv_producer }
    }
}

#[async_trait]
impl Processor for CdvReturnProcessor {
    fn name(&self) -> &str {
        "CdvReturn"
    }

    async fn process(&self, exchange: Exchange) -> PdpResult<Exchange> {
        // Vérifier qu'un CDV a été généré
        let has_cdv_header = exchange
            .get_header("cdv.generated")
            .map(|v| v == "true")
            .unwrap_or(false);

        let cdv_xml = match exchange.get_property("cdv.xml") {
            Some(xml) if has_cdv_header => xml.clone(),
            _ => return Ok(exchange),
        };

        // Créer un Exchange dédié au CDV
        let mut cdv_exchange = Exchange::new(cdv_xml.into_bytes());
        cdv_exchange.set_header("document.type", "cdar");
        cdv_exchange.set_header("source.protocol", "internal");
        cdv_exchange = cdv_exchange.with_flow_id(exchange.flow_id);

        // Copier les métadonnées de routage retour
        if let Some(v) = exchange.get_property("cdv.status_code") {
            cdv_exchange.set_property("cdv.status_code", &v.clone());
        }
        if let Some(v) = exchange.get_property("cdv.document_id") {
            cdv_exchange.set_property("cdv.document_id", &v.clone());
        }

        // Copier les infos de l'émetteur pour le routage retour
        if let Some(inv) = &exchange.invoice {
            if let Some(ref siret) = inv.seller_siret {
                cdv_exchange.set_property("return.destination_siret", siret);
            }
            if let Some(ref name) = inv.seller_name {
                cdv_exchange.set_property("return.destination_name", name);
            }
        }

        // Copier callback URL si disponible (AFNOR)
        if let Some(v) = exchange.get_property("callback.url") {
            cdv_exchange.set_property("callback.url", &v.clone());
        }

        // Nom de fichier CDV
        let cdv_doc_id = exchange
            .get_property("cdv.document_id")
            .cloned()
            .unwrap_or_else(|| cdv_exchange.id.to_string());
        cdv_exchange.source_filename = Some(format!("CDV_{}.xml", cdv_doc_id));

        // Envoyer le CDV
        match self.cdv_producer.send(cdv_exchange).await {
            Ok(_) => {
                tracing::info!(
                    flow_id = %exchange.flow_id,
                    cdv_status = exchange.get_property("cdv.status_code").unwrap_or(&"?".to_string()),
                    "CDV renvoyé à l'émetteur"
                );
            }
            Err(e) => {
                tracing::error!(
                    flow_id = %exchange.flow_id,
                    error = %e,
                    "Erreur lors du renvoi du CDV"
                );
                // On ne bloque pas le pipeline pour un échec de CDV
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

    /// Producer de test qui enregistre les exchanges envoyés
    struct MockProducer {
        sent: Mutex<Vec<Exchange>>,
        fail: bool,
    }

    impl MockProducer {
        fn new() -> Self {
            Self {
                sent: Mutex::new(Vec::new()),
                fail: false,
            }
        }

        fn failing() -> Self {
            Self {
                sent: Mutex::new(Vec::new()),
                fail: true,
            }
        }

        fn sent_count(&self) -> usize {
            self.sent.lock().unwrap().len()
        }
    }

    #[async_trait]
    impl Producer for MockProducer {
        fn name(&self) -> &str {
            "mock-cdv-producer"
        }

        async fn send(&self, exchange: Exchange) -> PdpResult<Exchange> {
            if self.fail {
                return Err(PdpError::RoutingError("Erreur mock".to_string()));
            }
            self.sent.lock().unwrap().push(exchange.clone());
            Ok(exchange)
        }
    }

    #[tokio::test]
    async fn test_cdv_return_sends_when_cdv_present() {
        let producer = Arc::new(MockProducer::new());
        let processor = CdvReturnProcessor::new(producer.clone());

        let mut exchange = Exchange::new(b"<Invoice/>".to_vec());
        exchange.set_property("cdv.xml", "<CdvResponse>OK</CdvResponse>");
        exchange.set_property("cdv.status_code", "200");
        exchange.set_property("cdv.document_id", "CDV-001");
        exchange.set_header("cdv.generated", "true");

        let result = processor.process(exchange).await.unwrap();

        // Le CDV a été envoyé
        assert_eq!(producer.sent_count(), 1);
        // L'exchange original est retourné inchangé
        assert_eq!(result.body, b"<Invoice/>");
    }

    #[tokio::test]
    async fn test_cdv_return_skips_when_no_cdv() {
        let producer = Arc::new(MockProducer::new());
        let processor = CdvReturnProcessor::new(producer.clone());

        let exchange = Exchange::new(b"<Invoice/>".to_vec());

        let result = processor.process(exchange).await.unwrap();

        // Rien n'a été envoyé
        assert_eq!(producer.sent_count(), 0);
        assert_eq!(result.body, b"<Invoice/>");
    }

    #[tokio::test]
    async fn test_cdv_return_skips_when_no_header() {
        let producer = Arc::new(MockProducer::new());
        let processor = CdvReturnProcessor::new(producer.clone());

        let mut exchange = Exchange::new(b"<Invoice/>".to_vec());
        exchange.set_property("cdv.xml", "<CdvResponse>OK</CdvResponse>");
        // Pas de header cdv.generated

        let result = processor.process(exchange).await.unwrap();

        assert_eq!(producer.sent_count(), 0);
        assert_eq!(result.body, b"<Invoice/>");
    }

    #[tokio::test]
    async fn test_cdv_return_does_not_block_on_error() {
        let producer = Arc::new(MockProducer::failing());
        let processor = CdvReturnProcessor::new(producer.clone());

        let mut exchange = Exchange::new(b"<Invoice/>".to_vec());
        exchange.set_property("cdv.xml", "<CdvResponse>OK</CdvResponse>");
        exchange.set_property("cdv.status_code", "200");
        exchange.set_header("cdv.generated", "true");

        // Le pipeline ne doit pas échouer même si le producer échoue
        let result = processor.process(exchange).await.unwrap();
        assert_eq!(result.body, b"<Invoice/>");
    }
}
