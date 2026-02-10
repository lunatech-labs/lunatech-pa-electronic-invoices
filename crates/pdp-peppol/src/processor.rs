//! Processors pipeline pour l'envoi et la réception PEPPOL.
//!
//! - [`PeppolSendProcessor`] : envoie une facture ou un CDAR vers une autre PDP via AS4
//! - [`PeppolReceiveProcessor`] : traite un message AS4 entrant et injecte dans le pipeline

use async_trait::async_trait;
use pdp_core::error::{PdpError, PdpResult};
use pdp_core::exchange::Exchange;
use pdp_core::model::{FlowStatus, InvoiceFormat};
use pdp_core::processor::Processor;

use crate::as4::As4Client;
use crate::model::*;
use crate::smp::SmpClient;

// ============================================================
// PeppolSendProcessor — envoi vers une autre PDP
// ============================================================

/// Processor qui envoie une facture ou un CDAR vers une autre PDP via PEPPOL AS4.
///
/// Prérequis dans l'exchange :
/// - `invoice` parsée (pour extraire sender/receiver SIREN) OU
/// - propriétés `peppol.sender` et `peppol.receiver` déjà renseignées
/// - `body` contient le XML à envoyer (UBL, CII ou CDAR)
///
/// Résultat :
/// - Propriétés `peppol.message_id`, `peppol.endpoint_url`, `peppol.status` ajoutées
/// - Statut `Distributed` si succès, `Error` si échec
pub struct PeppolSendProcessor {
    config: PeppolConfig,
    smp: SmpClient,
}

impl PeppolSendProcessor {
    pub fn new(config: PeppolConfig) -> Self {
        let smp = if config.test_mode {
            SmpClient::test()
        } else {
            SmpClient::production()
        };
        Self { config, smp }
    }

    /// Détermine le ParticipantId du destinataire à partir de l'exchange.
    fn resolve_receiver(&self, exchange: &Exchange) -> Result<ParticipantId, PdpError> {
        // 1. Propriété explicite
        if let Some(receiver) = exchange.get_property("peppol.receiver") {
            if let Some(pid) = ParticipantId::parse(receiver) {
                return Ok(pid);
            }
        }

        // 2. Depuis la facture parsée (SIREN acheteur)
        if let Some(invoice) = &exchange.invoice {
            if let Some(siret) = &invoice.buyer_siret {
                let siren = if siret.len() >= 9 { &siret[..9] } else { siret };
                return Ok(ParticipantId::from_siren(siren));
            }
            if let Some(endpoint) = &invoice.buyer_endpoint_id {
                return Ok(ParticipantId::from_french_endpoint(endpoint));
            }
        }

        Err(PdpError::DistributionError(
            "Impossible de déterminer le destinataire PEPPOL (pas de SIREN acheteur ni peppol.receiver)".into()
        ))
    }

    /// Détermine le ParticipantId de l'émetteur.
    fn resolve_sender(&self, exchange: &Exchange) -> ParticipantId {
        if let Some(sender) = exchange.get_property("peppol.sender") {
            if let Some(pid) = ParticipantId::parse(sender) {
                return pid;
            }
        }

        if let Some(invoice) = &exchange.invoice {
            if let Some(siret) = &invoice.seller_siret {
                let siren = if siret.len() >= 9 { &siret[..9] } else { siret };
                return ParticipantId::from_siren(siren);
            }
        }

        self.config.participant_id.clone()
    }

    /// Détermine le DocumentTypeId et ProcessId à partir de l'exchange.
    fn resolve_document_type(&self, exchange: &Exchange) -> (PeppolDocumentType, DocumentTypeId, ProcessId) {
        // CDAR ?
        if exchange.get_property("cdv.xml").is_some()
            || exchange.get_header("cdv.generated").map(|s| s.as_str()) == Some("true")
        {
            return (PeppolDocumentType::Cdar, DocumentTypeId::cdar(), ProcessId::billing());
        }

        // Facture : déterminer UBL vs CII
        if let Some(invoice) = &exchange.invoice {
            match invoice.source_format {
                InvoiceFormat::UBL => {
                    // Avoir ou facture ?
                    let type_code = invoice.invoice_type_code.as_deref().unwrap_or("380");
                    if ["261", "262", "381", "396", "502", "503"].contains(&type_code) {
                        return (PeppolDocumentType::CreditNote, DocumentTypeId::ubl_credit_note(), ProcessId::billing());
                    }
                    return (PeppolDocumentType::Invoice, DocumentTypeId::ubl_invoice(), ProcessId::billing());
                }
                InvoiceFormat::CII | InvoiceFormat::FacturX => {
                    return (PeppolDocumentType::Invoice, DocumentTypeId::cii_invoice(), ProcessId::billing());
                }
            }
        }

        // Défaut : UBL Invoice
        (PeppolDocumentType::Invoice, DocumentTypeId::ubl_invoice(), ProcessId::billing())
    }
}

#[async_trait]
impl Processor for PeppolSendProcessor {
    fn name(&self) -> &str {
        "PeppolSendProcessor"
    }

    async fn process(&self, mut exchange: Exchange) -> PdpResult<Exchange> {
        let sender = self.resolve_sender(&exchange);
        let receiver = self.resolve_receiver(&exchange)?;
        let (doc_type, doc_type_id, process_id) = self.resolve_document_type(&exchange);

        // Déterminer le payload à envoyer
        let payload = if doc_type == PeppolDocumentType::Cdar {
            // Pour un CDAR, utiliser le XML CDV stocké dans les propriétés
            exchange.get_property("cdv.xml")
                .map(|s| s.as_bytes().to_vec())
                .unwrap_or_else(|| exchange.body.clone())
        } else {
            exchange.body.clone()
        };

        let message = PeppolMessage::new(
            sender.clone(),
            receiver.clone(),
            doc_type,
            doc_type_id.clone(),
            process_id.clone(),
            payload,
        );

        tracing::info!(
            sender = %sender,
            receiver = %receiver,
            document_type = %doc_type,
            exchange_id = %exchange.id,
            "Envoi PEPPOL AS4"
        );

        // 1. Lookup SMP pour trouver l'endpoint du destinataire
        let lookup_result = self.smp.lookup(&receiver, &doc_type_id, &process_id).await
            .map_err(|e| PdpError::DistributionError(format!("SMP lookup échoué : {}", e)))?;

        exchange.set_property("peppol.endpoint_url", &lookup_result.endpoint.endpoint_url);

        // 2. Envoyer via AS4
        let client = As4Client::new(self.config.clone());
        let result = client.send(&message, &lookup_result.endpoint).await
            .map_err(|e| PdpError::DistributionError(format!("Envoi AS4 échoué : {}", e)))?;

        exchange.set_property("peppol.message_id", &result.message_id);
        exchange.set_property("peppol.timestamp", &result.timestamp.to_rfc3339());

        if result.success {
            exchange.set_property("peppol.status", "sent");
            exchange.set_status(FlowStatus::Distributed);
            tracing::info!(
                message_id = %result.message_id,
                endpoint = %result.endpoint_url,
                "Message PEPPOL envoyé avec succès"
            );
        } else {
            let error_msg = result.error.unwrap_or_else(|| "Erreur inconnue".into());
            exchange.set_property("peppol.status", "error");
            exchange.set_property("peppol.error", &error_msg);
            exchange.add_error("PeppolSend", &PdpError::DistributionError(error_msg.clone()));
            tracing::warn!(
                message_id = %result.message_id,
                error = %error_msg,
                "Envoi PEPPOL échoué"
            );
        }

        Ok(exchange)
    }
}

// ============================================================
// PeppolReceiveProcessor — réception depuis une autre PDP
// ============================================================

/// Processor qui traite un message PEPPOL AS4 entrant.
///
/// Prérequis :
/// - `exchange.body` contient le payload XML extrait du SBDH
/// - Propriétés `peppol.sender`, `peppol.receiver`, `peppol.message_id` renseignées
///   (par le handler HTTP AS4 en amont)
///
/// Résultat :
/// - `exchange.body` contient le document métier (facture ou CDAR)
/// - Propriétés `peppol.*` enrichies
/// - `source_filename` renseigné pour le pipeline
pub struct PeppolReceiveProcessor;

impl PeppolReceiveProcessor {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Processor for PeppolReceiveProcessor {
    fn name(&self) -> &str {
        "PeppolReceiveProcessor"
    }

    async fn process(&self, mut exchange: Exchange) -> PdpResult<Exchange> {
        let sender = exchange.get_property("peppol.sender").cloned().unwrap_or_default();
        let receiver = exchange.get_property("peppol.receiver").cloned().unwrap_or_default();
        let message_id = exchange.get_property("peppol.message_id").cloned().unwrap_or_default();

        // Déterminer le type de document à partir du contenu
        let body_str = std::str::from_utf8(&exchange.body)
            .map_err(|e| PdpError::ParseError(format!("Payload PEPPOL non UTF-8 : {}", e)))?;

        let doc_type = detect_document_type(body_str);

        exchange.set_property("peppol.received", "true");
        exchange.set_property("peppol.document_type", &doc_type.to_string());

        // Générer un nom de fichier pour le pipeline
        if exchange.source_filename.is_none() {
            let ext = match doc_type {
                PeppolDocumentType::Invoice | PeppolDocumentType::CreditNote => "xml",
                PeppolDocumentType::Cdar => "xml",
            };
            let filename = format!("peppol_{}_{}.{}", sender.replace("::", "_"), message_id.replace('@', "_"), ext);
            exchange.source_filename = Some(filename.clone());
            exchange.set_header("source.filename", &filename);
        }

        exchange.set_header("source.protocol", "peppol-as4");
        exchange.set_status(FlowStatus::Received);

        tracing::info!(
            sender = %sender,
            receiver = %receiver,
            message_id = %message_id,
            document_type = %doc_type,
            exchange_id = %exchange.id,
            "Message PEPPOL reçu et injecté dans le pipeline"
        );

        Ok(exchange)
    }
}

/// Détecte le type de document à partir du contenu XML.
fn detect_document_type(xml: &str) -> PeppolDocumentType {
    if xml.contains("CrossDomainAcknowledgementAndResponse") {
        PeppolDocumentType::Cdar
    } else if xml.contains("CreditNote") && !xml.contains("CrossIndustryInvoice") {
        PeppolDocumentType::CreditNote
    } else {
        PeppolDocumentType::Invoice
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_document_type_invoice_ubl() {
        let xml = r#"<Invoice xmlns="urn:oasis:names:specification:ubl:schema:xsd:Invoice-2"><ID>FA-001</ID></Invoice>"#;
        assert_eq!(detect_document_type(xml), PeppolDocumentType::Invoice);
    }

    #[test]
    fn test_detect_document_type_credit_note() {
        let xml = r#"<CreditNote xmlns="urn:oasis:names:specification:ubl:schema:xsd:CreditNote-2"><ID>AV-001</ID></CreditNote>"#;
        assert_eq!(detect_document_type(xml), PeppolDocumentType::CreditNote);
    }

    #[test]
    fn test_detect_document_type_cii() {
        let xml = r#"<rsm:CrossIndustryInvoice xmlns:rsm="urn:un:unece:uncefact:data:standard:CrossIndustryInvoice:100"><ID>FA-001</ID></rsm:CrossIndustryInvoice>"#;
        assert_eq!(detect_document_type(xml), PeppolDocumentType::Invoice);
    }

    #[test]
    fn test_detect_document_type_cdar() {
        let xml = r#"<rsm:CrossDomainAcknowledgementAndResponse xmlns:rsm="urn:un:unece:uncefact:data:standard:CrossDomainAcknowledgementAndResponse:100"/>"#;
        assert_eq!(detect_document_type(xml), PeppolDocumentType::Cdar);
    }

    #[tokio::test]
    async fn test_receive_processor_sets_properties() {
        let processor = PeppolReceiveProcessor::new();
        let mut exchange = Exchange::new(b"<Invoice><ID>FA-001</ID></Invoice>".to_vec());
        exchange.set_property("peppol.sender", "0002::111111111");
        exchange.set_property("peppol.receiver", "0002::222222222");
        exchange.set_property("peppol.message_id", "msg-001@AP");

        let result = processor.process(exchange).await.unwrap();

        assert_eq!(result.get_property("peppol.received").map(|s| s.as_str()), Some("true"));
        assert_eq!(result.get_property("peppol.document_type").map(|s| s.as_str()), Some("Invoice"));
        assert!(result.source_filename.is_some());
        assert!(result.source_filename.as_ref().unwrap().starts_with("peppol_"));
        assert_eq!(result.status, FlowStatus::Received);
    }

    #[tokio::test]
    async fn test_receive_processor_cdar() {
        let processor = PeppolReceiveProcessor::new();
        let mut exchange = Exchange::new(
            b"<rsm:CrossDomainAcknowledgementAndResponse xmlns:rsm=\"urn:un:unece:uncefact:data:standard:CrossDomainAcknowledgementAndResponse:100\"/>".to_vec()
        );
        exchange.set_property("peppol.sender", "0002::111111111");
        exchange.set_property("peppol.receiver", "0002::222222222");
        exchange.set_property("peppol.message_id", "cdv-001@AP");

        let result = processor.process(exchange).await.unwrap();
        assert_eq!(result.get_property("peppol.document_type").map(|s| s.as_str()), Some("CDAR"));
    }

    #[test]
    fn test_resolve_document_type_from_exchange() {
        let config = PeppolConfig::test();
        let processor = PeppolSendProcessor::new(config);

        // Exchange sans facture parsée → défaut UBL Invoice
        let exchange = Exchange::new(b"<Invoice/>".to_vec());
        let (doc_type, _, _) = processor.resolve_document_type(&exchange);
        assert_eq!(doc_type, PeppolDocumentType::Invoice);

        // Exchange avec CDV
        let mut exchange2 = Exchange::new(b"<Invoice/>".to_vec());
        exchange2.set_property("cdv.xml", "<CDAR/>");
        exchange2.set_header("cdv.generated", "true");
        let (doc_type2, _, _) = processor.resolve_document_type(&exchange2);
        assert_eq!(doc_type2, PeppolDocumentType::Cdar);
    }
}
