use async_trait::async_trait;
use pdp_core::error::{PdpError, PdpResult};
use pdp_core::exchange::Exchange;
use pdp_core::model::{DocumentType, FlowStatus};
use pdp_core::processor::Processor;

use crate::generator::CdarGenerator;
use crate::model::{CdarValidationError, InvoiceStatusCode, StatusReasonCode};
use crate::parser::CdarParser;

// ============================================================
// DocumentTypeRouter — détection et routage facture vs CDAR
// ============================================================

/// Processor qui détecte le type de document entrant (facture, CDAR, e-reporting)
/// et positionne le header `document.type` sur l'exchange.
///
/// Si le document est un CDAR, il est immédiatement parsé et les propriétés
/// `cdv.*` sont renseignées. Le header `document.type` = "CDAR" permet aux
/// processors suivants (ParseProcessor, ValidateProcessor, etc.) de sauter
/// le traitement facture.
///
/// # Sources de CDAR
///
/// Les CDAR peuvent arriver de 3 sources différentes :
/// - **Client émission** (`source.protocol` = `file` ou `sftp`, côté vendeur) :
///   le client de notre PDP nous envoie un statut sur une facture qu'il a émise
///   (ex: 204 Prise en charge, 205 Approuvée, 210 Refusée, 212 Encaissée)
/// - **Client réception** (`source.protocol` = `file` ou `sftp`, côté acheteur) :
///   le client de notre PDP nous envoie un statut sur une facture qu'il a reçue
/// - **Autre PDP** (`source.protocol` = `peppol-as4` ou `afnor-flow`) :
///   une PDP distante nous transmet un CDV via PEPPOL ou AFNOR Flow Service
///
/// La source est déterminée par le header `source.protocol` et la propriété
/// `peppol.received` ou `afnor.received`.
pub struct DocumentTypeRouter;

impl DocumentTypeRouter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DocumentTypeRouter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Processor for DocumentTypeRouter {
    fn name(&self) -> &str {
        "DocumentTypeRouter"
    }

    async fn process(&self, mut exchange: Exchange) -> PdpResult<Exchange> {
        // Si déjà marqué comme irrecevable, ne pas toucher
        if exchange.get_property("reception.failed").is_some() {
            return Ok(exchange);
        }

        let doc_type = pdp_invoice::detect_document_type(&exchange.body);
        exchange.set_header("document.type", &doc_type.to_string());

        match doc_type {
            DocumentType::Cdar => {
                // Déterminer la source du CDAR
                let source = determine_cdar_source(&exchange);
                exchange.set_property("cdv.source", &source);

                tracing::info!(
                    exchange_id = %exchange.id,
                    filename = exchange.source_filename.as_deref().unwrap_or("N/A"),
                    source = %source,
                    "Document CDAR détecté — routage vers traitement CDV"
                );

                // Parser le CDAR immédiatement
                let parser = CdarParser::new();
                let xml = std::str::from_utf8(&exchange.body)
                    .map_err(|e| PdpError::CdarError(format!("CDV non UTF-8: {}", e)))?;

                let cdv = parser.parse(xml)?;

                // Extraire les informations clés
                let status_code = cdv.status_code().unwrap_or(0);
                let invoice_id = cdv.referenced_documents.first()
                    .map(|d| d.invoice_id.clone())
                    .unwrap_or_default();
                let process_condition = cdv.referenced_documents.first()
                    .and_then(|d| d.process_condition.clone())
                    .unwrap_or_default();

                // Stocker les données parsées
                exchange.set_property("cdv.received", "true");
                exchange.set_property("cdv.document_id", &cdv.document_id);
                exchange.set_property("cdv.type_code", cdv.type_code.code());
                exchange.set_property("cdv.status_code", &status_code.to_string());
                exchange.set_property("cdv.invoice_id", &invoice_id);
                exchange.set_property("cdv.process_condition", &process_condition);
                exchange.set_property("cdv.guideline_id", &cdv.guideline_id);

                if let Some(ref name) = cdv.document_name {
                    exchange.set_property("cdv.document_name", name);
                }

                // Stocker les infos des parties
                if let Some(ref gid) = cdv.sender.global_id {
                    exchange.set_property("cdv.sender.id", gid);
                }
                for (i, recipient) in cdv.recipients.iter().enumerate() {
                    if let Some(ref gid) = recipient.global_id {
                        exchange.set_property(&format!("cdv.recipient.{}.id", i), gid);
                    }
                    if let Some(ref name) = recipient.name {
                        exchange.set_property(&format!("cdv.recipient.{}.name", i), name);
                    }
                }

                // Stocker les motifs de rejet s'il y en a
                for (i, doc) in cdv.referenced_documents.iter().enumerate() {
                    for (j, status) in doc.statuses.iter().enumerate() {
                        if let Some(ref reason_code) = status.reason_code {
                            exchange.set_property(
                                &format!("cdv.doc.{}.status.{}.reason_code", i, j),
                                reason_code,
                            );
                        }
                        if let Some(ref reason) = status.reason {
                            exchange.set_property(
                                &format!("cdv.doc.{}.status.{}.reason", i, j),
                                reason,
                            );
                        }
                    }
                }

                // Si le CDV indique un rejet, ajouter une erreur
                if cdv.is_rejected() {
                    let error = PdpError::CdarError(format!(
                        "Facture {} rejetée (code {}): {}",
                        invoice_id, status_code, process_condition
                    ));
                    exchange.add_error("CdvReception", &error);
                }

                // Mapper le statut CDV vers le FlowStatus du pipeline
                let flow_status = map_cdv_to_flow_status(status_code);
                exchange.status = flow_status.clone();

                tracing::info!(
                    cdv_id = %cdv.document_id,
                    invoice_id = %invoice_id,
                    status_code = status_code,
                    source = %source,
                    flow_status = %flow_status,
                    "CDV entrant traité"
                );
            }
            DocumentType::EReporting => {
                tracing::info!(
                    exchange_id = %exchange.id,
                    "Document e-reporting détecté"
                );
            }
            DocumentType::Invoice => {
                tracing::debug!(
                    exchange_id = %exchange.id,
                    "Document facture détecté — traitement standard"
                );
            }
            DocumentType::Unknown => {
                tracing::debug!(
                    exchange_id = %exchange.id,
                    "Type de document non déterminé — traitement standard"
                );
            }
        }

        Ok(exchange)
    }
}

/// Détermine la source d'un CDAR entrant.
///
/// Retourne une chaîne descriptive :
/// - `"peppol"` : reçu d'une autre PDP via PEPPOL AS4
/// - `"afnor"` : reçu d'une autre PDP via AFNOR Flow Service
/// - `"ppf"` : reçu du PPF (Portail Public de Facturation)
/// - `"client"` : reçu d'un client (émission ou réception)
fn determine_cdar_source(exchange: &Exchange) -> String {
    // PEPPOL
    if exchange.get_property("peppol.received").is_some()
        || exchange.get_header("source.protocol").map(|s| s.as_str()) == Some("peppol-as4")
    {
        return "peppol".to_string();
    }

    // AFNOR Flow Service
    if exchange.get_property("afnor.received").is_some()
        || exchange.get_header("source.protocol").map(|s| s.as_str()) == Some("afnor-flow")
    {
        return "afnor".to_string();
    }

    // PPF (code interface FFE06*)
    if let Some(ref filename) = exchange.source_filename {
        if filename.starts_with("FFE06") || filename.starts_with("CFE") {
            return "ppf".to_string();
        }
    }

    // Par défaut : client local
    "client".to_string()
}

/// Processor qui génère un CDV (Compte-rendu De Vie) après traitement d'une facture.
/// Il ajoute le XML CDV conforme D23B dans les propriétés de l'exchange.
pub struct CdarProcessor {
    generator: CdarGenerator,
}

impl CdarProcessor {
    pub fn new(pdp_siren: &str, pdp_name: &str) -> Self {
        Self {
            generator: CdarGenerator::new(pdp_siren, pdp_name),
        }
    }
}

#[async_trait]
impl Processor for CdarProcessor {
    fn name(&self) -> &str {
        "CdarProcessor"
    }

    async fn process(&self, mut exchange: Exchange) -> PdpResult<Exchange> {
        let invoice = exchange.invoice.as_ref().ok_or_else(|| {
            PdpError::CdarError(
                "Aucune facture parsée dans l'exchange pour générer le CDV".to_string(),
            )
        })?;

        let invoice_type_code = invoice.invoice_type_code.as_deref().unwrap_or("380");

        let cdv = if exchange.has_errors() {
            let errors: Vec<CdarValidationError> = exchange
                .errors
                .iter()
                .map(|e| {
                    let reason_code = classify_error_reason(&e.step, &e.message);
                    CdarValidationError {
                        rule_id: format!("PDP-{}", e.step),
                        severity: "ERROR".to_string(),
                        location: None,
                        message: e.message.clone(),
                        reason_code: Some(reason_code),
                    }
                })
                .collect();

            tracing::warn!(
                invoice = %invoice.invoice_number,
                error_count = errors.len(),
                "Génération CDV de rejet (213)"
            );

            self.generator.generate_rejetee(invoice, invoice_type_code, errors)
        } else {
            tracing::info!(
                invoice = %invoice.invoice_number,
                "Génération CDV de dépôt (200)"
            );

            self.generator.generate_deposee(invoice, invoice_type_code)
        };

        let cdv_xml = self.generator.to_xml(&cdv)?;
        exchange.set_property("cdv.xml", &cdv_xml);
        exchange.set_property("cdv.document_id", &cdv.document_id);
        exchange.set_property("cdv.status_code", &cdv.status_code().unwrap_or(0).to_string());
        exchange.set_property("cdv.type_code", cdv.type_code.code());
        exchange.set_header("cdv.generated", "true");

        Ok(exchange)
    }
}

// ============================================================
// CdvReceptionProcessor — traitement des CDV entrants
// ============================================================

/// Processor qui parse les CDV (Compte-rendu De Vie) reçus du PPF ou d'une PDP
/// et met à jour le statut de l'exchange en conséquence.
///
/// Entrée : exchange.body contient le XML CDV D23B
/// Sortie : propriétés cdv.* renseignées, status mis à jour
pub struct CdvReceptionProcessor {
    parser: CdarParser,
}

impl CdvReceptionProcessor {
    pub fn new() -> Self {
        Self {
            parser: CdarParser::new(),
        }
    }
}

#[async_trait]
impl Processor for CdvReceptionProcessor {
    fn name(&self) -> &str {
        "CdvReceptionProcessor"
    }

    async fn process(&self, mut exchange: Exchange) -> PdpResult<Exchange> {
        let xml = std::str::from_utf8(&exchange.body)
            .map_err(|e| PdpError::CdarError(format!("CDV non UTF-8: {}", e)))?;

        let cdv = self.parser.parse(xml)?;

        // Extraire les informations clés
        let status_code = cdv.status_code().unwrap_or(0);
        let invoice_id = cdv.referenced_documents.first()
            .map(|d| d.invoice_id.clone())
            .unwrap_or_default();
        let process_condition = cdv.referenced_documents.first()
            .and_then(|d| d.process_condition.clone())
            .unwrap_or_default();

        // Stocker les données parsées dans les propriétés
        exchange.set_property("cdv.received", "true");
        exchange.set_property("cdv.document_id", &cdv.document_id);
        exchange.set_property("cdv.type_code", cdv.type_code.code());
        exchange.set_property("cdv.status_code", &status_code.to_string());
        exchange.set_property("cdv.invoice_id", &invoice_id);
        exchange.set_property("cdv.process_condition", &process_condition);
        exchange.set_property("cdv.guideline_id", &cdv.guideline_id);

        if let Some(ref name) = cdv.document_name {
            exchange.set_property("cdv.document_name", name);
        }

        // Stocker les infos des parties
        if let Some(ref gid) = cdv.sender.global_id {
            exchange.set_property("cdv.sender.id", gid);
        }
        for (i, recipient) in cdv.recipients.iter().enumerate() {
            if let Some(ref gid) = recipient.global_id {
                exchange.set_property(&format!("cdv.recipient.{}.id", i), gid);
            }
            if let Some(ref name) = recipient.name {
                exchange.set_property(&format!("cdv.recipient.{}.name", i), name);
            }
        }

        // Stocker les motifs de rejet s'il y en a
        for (i, doc) in cdv.referenced_documents.iter().enumerate() {
            for (j, status) in doc.statuses.iter().enumerate() {
                if let Some(ref reason_code) = status.reason_code {
                    exchange.set_property(
                        &format!("cdv.doc.{}.status.{}.reason_code", i, j),
                        reason_code,
                    );
                }
                if let Some(ref reason) = status.reason {
                    exchange.set_property(
                        &format!("cdv.doc.{}.status.{}.reason", i, j),
                        reason,
                    );
                }
            }
        }

        // Si le CDV indique un rejet, ajouter une erreur à l'exchange
        if cdv.is_rejected() {
            let error = PdpError::CdarError(format!(
                "Facture {} rejetée (code {}): {}",
                invoice_id, status_code, process_condition
            ));
            exchange.add_error("CdvReception", &error);
        }

        // Mapper le statut CDV vers le FlowStatus du pipeline
        // (après add_error qui force Error, on rétablit le statut correct)
        let flow_status = map_cdv_to_flow_status(status_code);
        exchange.status = flow_status.clone();

        tracing::info!(
            cdv_id = %cdv.document_id,
            invoice_id = %invoice_id,
            status_code = status_code,
            condition = %process_condition,
            flow_status = %flow_status,
            success = cdv.is_success(),
            rejected = cdv.is_rejected(),
            "CDV entrant traité"
        );

        Ok(exchange)
    }
}

/// Mappe un code statut CDV (InvoiceStatusCode) vers un FlowStatus du pipeline
fn map_cdv_to_flow_status(status_code: u16) -> FlowStatus {
    match InvoiceStatusCode::from_code(status_code) {
        // Phase Transmission
        Some(InvoiceStatusCode::Deposee) => FlowStatus::Distributed,
        Some(InvoiceStatusCode::Emise) => FlowStatus::Distributed,
        Some(InvoiceStatusCode::Recue) => FlowStatus::Acknowledged,
        Some(InvoiceStatusCode::MiseADisposition) => FlowStatus::Distributed,
        // Phase Traitement — statuts positifs
        Some(InvoiceStatusCode::PriseEnCharge) => FlowStatus::Acknowledged,
        Some(InvoiceStatusCode::Approuvee) => FlowStatus::Acknowledged,
        Some(InvoiceStatusCode::ApprouveePartiellement) => FlowStatus::Acknowledged,
        Some(InvoiceStatusCode::Completee) => FlowStatus::Acknowledged,
        Some(InvoiceStatusCode::PaiementTransmis) => FlowStatus::Acknowledged,
        Some(InvoiceStatusCode::Encaissee) => FlowStatus::Acknowledged,
        Some(InvoiceStatusCode::Visee) => FlowStatus::Acknowledged,
        Some(InvoiceStatusCode::DemandePaiementDirect) => FlowStatus::Acknowledged,
        Some(InvoiceStatusCode::Affacturee) => FlowStatus::Acknowledged,
        Some(InvoiceStatusCode::AffactureeConfidentiel) => FlowStatus::Acknowledged,
        Some(InvoiceStatusCode::ChangementCompteAPayer) => FlowStatus::Acknowledged,
        Some(InvoiceStatusCode::NonAffacturee) => FlowStatus::Acknowledged,
        Some(InvoiceStatusCode::Annulee) => FlowStatus::Cancelled,
        // Phase Traitement — statuts d'attente
        Some(InvoiceStatusCode::EnLitige) => FlowStatus::WaitingAck,
        Some(InvoiceStatusCode::Suspendue) => FlowStatus::WaitingAck,
        // Rejets et erreurs
        Some(InvoiceStatusCode::Rejetee) => FlowStatus::Rejected,
        Some(InvoiceStatusCode::Refusee) => FlowStatus::Rejected,
        Some(InvoiceStatusCode::Irrecevable) => FlowStatus::Rejected,
        Some(InvoiceStatusCode::ErreurRoutage) => FlowStatus::Error,
        _ => FlowStatus::WaitingAck,
    }
}

// ============================================================
// IrrecevabiliteProcessor — génération CDAR 501 pour fichiers irrecevables
// ============================================================

/// Processor qui détecte les fichiers marqués comme irrecevables par le
/// `ReceptionProcessor` (propriété `reception.failed`) et génère un CDAR
/// d'irrecevabilité (statut 501) avec les codes motifs IRR_* appropriés.
///
/// Le XML CDAR est stocké dans la propriété `cdv.xml` de l'exchange.
/// L'exchange est ensuite routé vers la destination d'erreur.
pub struct IrrecevabiliteProcessor {
    generator: CdarGenerator,
}

impl IrrecevabiliteProcessor {
    pub fn new(pdp_siren: &str, pdp_name: &str) -> Self {
        Self {
            generator: CdarGenerator::new(pdp_siren, pdp_name),
        }
    }
}

#[async_trait]
impl Processor for IrrecevabiliteProcessor {
    fn name(&self) -> &str {
        "IrrecevabiliteProcessor"
    }

    async fn process(&self, mut exchange: Exchange) -> PdpResult<Exchange> {
        // Si pas d'erreur de réception, passer au processor suivant
        if exchange.get_property("reception.failed").is_none() {
            return Ok(exchange);
        }

        let filename = exchange.source_filename.clone().unwrap_or_default();
        let rule_ids_str = exchange.get_property("reception.rule_ids")
            .cloned()
            .unwrap_or_default();

        // Mapper les rule_ids vers le code IRR_* le plus pertinent
        let (reason_code, message) = map_reception_to_irrecevabilite(&rule_ids_str, &filename);

        // Créer un InvoiceData minimal pour le CDAR (pas de facture parsée à ce stade)
        let mut minimal_invoice = pdp_core::model::InvoiceData::new(
            filename.clone(),
            pdp_core::model::InvoiceFormat::CII,
        );
        // Utiliser le nom de fichier comme identifiant de référence
        minimal_invoice.invoice_number = filename.clone();

        let cdv = self.generator.generate_irrecevable(
            &minimal_invoice,
            reason_code.clone(),
            &message,
        );

        let cdv_xml = self.generator.to_xml(&cdv)?;

        exchange.set_property("cdv.xml", &cdv_xml);
        exchange.set_property("cdv.document_id", &cdv.document_id);
        exchange.set_property("cdv.status_code", "501");
        exchange.set_property("cdv.type_code", cdv.type_code.code());
        exchange.set_property("cdv.reason_code", reason_code.code());
        exchange.set_header("cdv.generated", "true");
        exchange.set_status(FlowStatus::Rejected);

        tracing::warn!(
            filename = %filename,
            exchange_id = %exchange.id,
            reason_code = %reason_code.code(),
            "CDAR 501 Irrecevable généré"
        );

        Ok(exchange)
    }
}

/// Mappe les rule_ids de réception vers un StatusReasonCode d'irrecevabilité
/// et un message descriptif.
///
/// Correspondance :
/// - REC-01 (fichier vide) → IRR_VIDE_F
/// - BR-FR-19 (taille > 100 Mo) → IRR_TAILLE_F
/// - REC-02 (extension non reconnue) → IRR_EXT_DOC
/// - REC-03 (caractères invalides dans le nom) → IRR_NOM_PJ
/// - REC-04 (nom de fichier absent) → IRR_NOM_PJ
/// - REC-05 (doublon) → IRR_TYPE_F
fn map_reception_to_irrecevabilite(rule_ids: &str, filename: &str) -> (StatusReasonCode, String) {
    let rules: Vec<&str> = rule_ids.split(',').collect();

    // Priorité : fichier vide > taille > extension > nom > doublon
    if rules.contains(&"REC-01") {
        return (
            StatusReasonCode::IrrVideF,
            format!("Fichier vide : '{}'", filename),
        );
    }
    if rules.contains(&"BR-FR-19") {
        return (
            StatusReasonCode::IrrTailleF,
            format!("Fichier trop volumineux (> 100 Mo) : '{}'", filename),
        );
    }
    if rules.contains(&"REC-02") {
        return (
            StatusReasonCode::IrrExtDoc,
            format!("Extension de fichier non reconnue : '{}' (attendu : xml, pdf)", filename),
        );
    }
    if rules.contains(&"REC-03") || rules.contains(&"REC-04") {
        return (
            StatusReasonCode::IrrNomPj,
            format!("Nom de fichier invalide : '{}'", filename),
        );
    }
    if rules.contains(&"REC-05") {
        return (
            StatusReasonCode::IrrTypeF,
            format!("Fichier en doublon : '{}'", filename),
        );
    }

    // Fallback
    (
        StatusReasonCode::IrrSyntax,
        format!("Fichier irrecevable : '{}' ({})", filename, rule_ids),
    )
}

/// Classifie une erreur de pipeline en code motif CDV officiel
fn classify_error_reason(step: &str, message: &str) -> StatusReasonCode {
    let msg_lower = message.to_lowercase();

    if msg_lower.contains("syntax") || msg_lower.contains("xml") || msg_lower.contains("parse") {
        return StatusReasonCode::RejSeman;
    }
    if msg_lower.contains("schematron") || msg_lower.contains("br-") || msg_lower.contains("rule") {
        return StatusReasonCode::RejSeman;
    }
    if msg_lower.contains("xsd") || msg_lower.contains("schema") {
        return StatusReasonCode::RejUni;
    }
    if msg_lower.contains("doublon") || msg_lower.contains("duplicate") {
        return StatusReasonCode::Doublon;
    }
    if msg_lower.contains("siret") || msg_lower.contains("siren") {
        return StatusReasonCode::SiretErr;
    }
    if msg_lower.contains("tva") || msg_lower.contains("vat") {
        return StatusReasonCode::TxTvaErr;
    }
    if msg_lower.contains("montant") || msg_lower.contains("total") || msg_lower.contains("amount") {
        return StatusReasonCode::MontantTotalErr;
    }
    if msg_lower.contains("calcul") || msg_lower.contains("calculation") {
        return StatusReasonCode::CalculErr;
    }
    if msg_lower.contains("adresse") || msg_lower.contains("address") {
        return StatusReasonCode::AdrErr;
    }
    if msg_lower.contains("destinataire") || msg_lower.contains("recipient") {
        return StatusReasonCode::DestErr;
    }
    if step.contains("validate") || step.contains("Validate") {
        return StatusReasonCode::RejSeman;
    }

    StatusReasonCode::NonConforme
}

#[cfg(test)]
mod tests {
    use super::*;
    use pdp_core::model::FlowStatus;

    #[tokio::test]
    async fn test_cdv_reception_deposee() {
        let xml = std::fs::read_to_string("../../tests/fixtures/cdar/cdv_200_deposee.xml")
            .expect("Fixture CDV introuvable");

        let processor = CdvReceptionProcessor::new();
        let exchange = Exchange::new(xml.into_bytes());
        let result = processor.process(exchange).await.unwrap();

        assert_eq!(result.get_property("cdv.received").map(|s| s.as_str()), Some("true"));
        assert_eq!(result.get_property("cdv.status_code").map(|s| s.as_str()), Some("200"));
        assert_eq!(result.get_property("cdv.invoice_id").map(|s| s.as_str()), Some("F202500001"));
        assert_eq!(result.get_property("cdv.type_code").map(|s| s.as_str()), Some("305"));
        assert_eq!(result.status, FlowStatus::Distributed);
        assert!(!result.has_errors());
    }

    #[tokio::test]
    async fn test_cdv_reception_rejetee() {
        let xml = std::fs::read_to_string("../../tests/fixtures/cdar/cdv_213_rejetee.xml")
            .expect("Fixture CDV rejet introuvable");

        let processor = CdvReceptionProcessor::new();
        let exchange = Exchange::new(xml.into_bytes());
        let result = processor.process(exchange).await.unwrap();

        assert_eq!(result.get_property("cdv.status_code").map(|s| s.as_str()), Some("213"));
        assert_eq!(result.status, FlowStatus::Rejected);
        assert!(result.has_errors());
    }

    #[tokio::test]
    async fn test_cdv_reception_litige() {
        let xml = std::fs::read_to_string("../../tests/fixtures/cdar/cdv_207_litige.xml")
            .expect("Fixture CDV litige introuvable");

        let processor = CdvReceptionProcessor::new();
        let exchange = Exchange::new(xml.into_bytes());
        let result = processor.process(exchange).await.unwrap();

        assert_eq!(result.get_property("cdv.status_code").map(|s| s.as_str()), Some("207"));
        assert_eq!(result.status, FlowStatus::WaitingAck);
        assert!(!result.has_errors());
    }

    // ===== Tests IrrecevabiliteProcessor =====

    #[tokio::test]
    async fn test_irrecevabilite_skips_valid_exchange() {
        let processor = IrrecevabiliteProcessor::new("123456789", "Test PDP");
        let exchange = Exchange::new(b"<Invoice/>".to_vec()).with_filename("facture.xml");
        let result = processor.process(exchange).await.unwrap();
        // Pas de reception.failed → pas de CDAR généré
        assert!(result.get_property("cdv.xml").is_none());
        assert!(result.get_property("cdv.status_code").is_none());
    }

    #[tokio::test]
    async fn test_irrecevabilite_generates_cdar_501_for_empty_file() {
        let processor = IrrecevabiliteProcessor::new("123456789", "Test PDP");
        let mut exchange = Exchange::new(Vec::new()).with_filename("vide.xml");
        exchange.set_property("reception.failed", "true");
        exchange.set_property("reception.rule_ids", "REC-01");

        let result = processor.process(exchange).await.unwrap();

        assert_eq!(result.get_property("cdv.status_code").map(|s| s.as_str()), Some("501"));
        assert_eq!(result.get_property("cdv.reason_code").map(|s| s.as_str()), Some("IRR_VIDE_F"));
        assert!(result.get_property("cdv.xml").is_some());
        assert!(result.get_property("cdv.xml").unwrap().contains("Irrecevable"));
        assert_eq!(result.status, FlowStatus::Rejected);
    }

    #[tokio::test]
    async fn test_irrecevabilite_generates_cdar_501_for_oversized() {
        let processor = IrrecevabiliteProcessor::new("123456789", "Test PDP");
        let mut exchange = Exchange::new(b"data".to_vec()).with_filename("gros.xml");
        exchange.set_property("reception.failed", "true");
        exchange.set_property("reception.rule_ids", "BR-FR-19");

        let result = processor.process(exchange).await.unwrap();

        assert_eq!(result.get_property("cdv.status_code").map(|s| s.as_str()), Some("501"));
        assert_eq!(result.get_property("cdv.reason_code").map(|s| s.as_str()), Some("IRR_TAILLE_F"));
    }

    #[tokio::test]
    async fn test_irrecevabilite_generates_cdar_501_for_bad_extension() {
        let processor = IrrecevabiliteProcessor::new("123456789", "Test PDP");
        let mut exchange = Exchange::new(b"data".to_vec()).with_filename("facture.csv");
        exchange.set_property("reception.failed", "true");
        exchange.set_property("reception.rule_ids", "REC-02");

        let result = processor.process(exchange).await.unwrap();

        assert_eq!(result.get_property("cdv.reason_code").map(|s| s.as_str()), Some("IRR_EXT_DOC"));
    }

    #[tokio::test]
    async fn test_irrecevabilite_generates_cdar_501_for_bad_filename() {
        let processor = IrrecevabiliteProcessor::new("123456789", "Test PDP");
        let mut exchange = Exchange::new(b"<Invoice/>".to_vec()).with_filename("bad file!.xml");
        exchange.set_property("reception.failed", "true");
        exchange.set_property("reception.rule_ids", "REC-03");

        let result = processor.process(exchange).await.unwrap();

        assert_eq!(result.get_property("cdv.reason_code").map(|s| s.as_str()), Some("IRR_NOM_PJ"));
    }

    #[tokio::test]
    async fn test_irrecevabilite_generates_cdar_501_for_duplicate() {
        let processor = IrrecevabiliteProcessor::new("123456789", "Test PDP");
        let mut exchange = Exchange::new(b"<Invoice/>".to_vec()).with_filename("facture.xml");
        exchange.set_property("reception.failed", "true");
        exchange.set_property("reception.rule_ids", "REC-05");

        let result = processor.process(exchange).await.unwrap();

        assert_eq!(result.get_property("cdv.reason_code").map(|s| s.as_str()), Some("IRR_TYPE_F"));
    }

    #[tokio::test]
    async fn test_irrecevabilite_cdar_xml_is_valid() {
        let processor = IrrecevabiliteProcessor::new("123456789", "Test PDP");
        let mut exchange = Exchange::new(Vec::new()).with_filename("vide.xml");
        exchange.set_property("reception.failed", "true");
        exchange.set_property("reception.rule_ids", "REC-01");

        let result = processor.process(exchange).await.unwrap();
        let xml = result.get_property("cdv.xml").unwrap();

        // Vérifier que le XML est parseable par le CdarParser
        let parser = crate::parser::CdarParser::new();
        let cdv = parser.parse(xml).unwrap();
        assert_eq!(cdv.status_code(), Some(501));
        assert!(cdv.is_irrecevable());
        assert_eq!(cdv.type_code, crate::model::CdvTypeCode::Transmission);
    }

    #[tokio::test]
    async fn test_irrecevabilite_multiple_errors_uses_priority() {
        let processor = IrrecevabiliteProcessor::new("123456789", "Test PDP");
        let mut exchange = Exchange::new(Vec::new()).with_filename("bad file!.csv");
        exchange.set_property("reception.failed", "true");
        // REC-01 (vide) + REC-02 (extension) + REC-03 (nom) → priorité = REC-01 → IRR_VIDE_F
        exchange.set_property("reception.rule_ids", "REC-01,REC-02,REC-03");

        let result = processor.process(exchange).await.unwrap();

        assert_eq!(result.get_property("cdv.reason_code").map(|s| s.as_str()), Some("IRR_VIDE_F"));
    }

    #[test]
    fn test_map_reception_to_irrecevabilite() {
        assert_eq!(map_reception_to_irrecevabilite("REC-01", "f.xml").0, StatusReasonCode::IrrVideF);
        assert_eq!(map_reception_to_irrecevabilite("BR-FR-19", "f.xml").0, StatusReasonCode::IrrTailleF);
        assert_eq!(map_reception_to_irrecevabilite("REC-02", "f.csv").0, StatusReasonCode::IrrExtDoc);
        assert_eq!(map_reception_to_irrecevabilite("REC-03", "f.xml").0, StatusReasonCode::IrrNomPj);
        assert_eq!(map_reception_to_irrecevabilite("REC-04", "").0, StatusReasonCode::IrrNomPj);
        assert_eq!(map_reception_to_irrecevabilite("REC-05", "f.xml").0, StatusReasonCode::IrrTypeF);
        assert_eq!(map_reception_to_irrecevabilite("UNKNOWN", "f.xml").0, StatusReasonCode::IrrSyntax);
    }

    #[test]
    fn test_map_cdv_to_flow_status() {
        assert_eq!(map_cdv_to_flow_status(200), FlowStatus::Distributed);
        assert_eq!(map_cdv_to_flow_status(202), FlowStatus::Acknowledged);
        assert_eq!(map_cdv_to_flow_status(213), FlowStatus::Rejected);
        assert_eq!(map_cdv_to_flow_status(501), FlowStatus::Rejected);
        assert_eq!(map_cdv_to_flow_status(221), FlowStatus::Error);
        assert_eq!(map_cdv_to_flow_status(220), FlowStatus::Cancelled);
        assert_eq!(map_cdv_to_flow_status(207), FlowStatus::WaitingAck);
        assert_eq!(map_cdv_to_flow_status(9999), FlowStatus::WaitingAck);
    }

    // ===== Tests exhaustifs : FlowStatus pour toutes les fixtures =====

    async fn assert_processor_status(filename: &str, expected_status: FlowStatus, expected_has_errors: bool) {
        let xml = std::fs::read_to_string(format!("../../tests/fixtures/cdar/{}", filename))
            .unwrap_or_else(|_| panic!("Fixture {} introuvable", filename));
        let processor = CdvReceptionProcessor::new();
        let exchange = Exchange::new(xml.into_bytes());
        let result = processor.process(exchange).await.unwrap();
        assert_eq!(result.status, expected_status,
            "{}: FlowStatus attendu {:?}, obtenu {:?}", filename, expected_status, result.status);
        assert_eq!(result.has_errors(), expected_has_errors,
            "{}: has_errors attendu {}", filename, expected_has_errors);
    }

    #[tokio::test]
    async fn test_cdv_reception_all_transmission_statuses() {
        // Phase Transmission — statuts positifs → Distributed
        assert_processor_status("cdv_201_emise.xml", FlowStatus::Distributed, false).await;
        assert_processor_status("cdv_203_mise_a_disposition.xml", FlowStatus::Distributed, false).await;
        // 202 Reçue → Acknowledged
        assert_processor_status("cdv_202_recue.xml", FlowStatus::Acknowledged, false).await;
    }

    #[tokio::test]
    async fn test_cdv_reception_all_traitement_statuses() {
        // Phase Traitement — statuts positifs → Acknowledged
        assert_processor_status("cdv_204_prise_en_charge.xml", FlowStatus::Acknowledged, false).await;
        assert_processor_status("cdv_205_approuvee.xml", FlowStatus::Acknowledged, false).await;
        assert_processor_status("cdv_206_approuvee_partiellement.xml", FlowStatus::Acknowledged, false).await;
        assert_processor_status("cdv_209_completee.xml", FlowStatus::Acknowledged, false).await;
        assert_processor_status("cdv_211_paiement_transmis.xml", FlowStatus::Acknowledged, false).await;
        assert_processor_status("cdv_212_encaissee.xml", FlowStatus::Acknowledged, false).await;
        assert_processor_status("cdv_214_visee.xml", FlowStatus::Acknowledged, false).await;
        // Statuts d'attente → WaitingAck
        assert_processor_status("cdv_208_suspendue.xml", FlowStatus::WaitingAck, false).await;
    }

    #[tokio::test]
    async fn test_cdv_reception_all_error_statuses() {
        // Rejets → Rejected
        assert_processor_status("cdv_210_refusee.xml", FlowStatus::Rejected, false).await;
        assert_processor_status("cdv_213_rejetee.xml", FlowStatus::Rejected, true).await;
        assert_processor_status("cdv_501_irrecevable.xml", FlowStatus::Rejected, false).await;
        // Erreurs techniques → Error
        assert_processor_status("cdv_221_erreur_routage.xml", FlowStatus::Error, false).await;
    }

    #[tokio::test]
    async fn test_cdv_reception_extended_statuses() {
        // Annulée → Cancelled
        assert_processor_status("cdv_220_annulee.xml", FlowStatus::Cancelled, false).await;
        // Statuts 224-228 → Acknowledged
        assert_processor_status("cdv_224_demande_paiement_direct.xml", FlowStatus::Acknowledged, false).await;
        assert_processor_status("cdv_225_affacturee.xml", FlowStatus::Acknowledged, false).await;
        assert_processor_status("cdv_226_affacturee_confidentiel.xml", FlowStatus::Acknowledged, false).await;
        assert_processor_status("cdv_227_changement_compte_a_payer.xml", FlowStatus::Acknowledged, false).await;
        assert_processor_status("cdv_228_non_affacturee.xml", FlowStatus::Acknowledged, false).await;
    }

    // ===== Tests DocumentTypeRouter =====

    #[tokio::test]
    async fn test_router_detects_cdar() {
        let xml = std::fs::read("../../tests/fixtures/cdar/cdv_200_deposee.xml").unwrap();
        let router = DocumentTypeRouter::new();
        let exchange = Exchange::new(xml);
        let result = router.process(exchange).await.unwrap();

        assert_eq!(result.get_header("document.type").map(|s| s.as_str()), Some("CDAR"));
        assert_eq!(result.get_property("cdv.received").map(|s| s.as_str()), Some("true"));
        assert_eq!(result.get_property("cdv.status_code").map(|s| s.as_str()), Some("200"));
        assert_eq!(result.get_property("cdv.source").map(|s| s.as_str()), Some("client"));
        assert_eq!(result.status, FlowStatus::Distributed);
    }

    #[tokio::test]
    async fn test_router_detects_invoice_ubl() {
        let xml = b"<Invoice xmlns=\"urn:oasis:names:specification:ubl:schema:xsd:Invoice-2\"><cbc:ID>TEST</cbc:ID></Invoice>";
        let router = DocumentTypeRouter::new();
        let exchange = Exchange::new(xml.to_vec());
        let result = router.process(exchange).await.unwrap();

        assert_eq!(result.get_header("document.type").map(|s| s.as_str()), Some("Invoice"));
        assert!(result.get_property("cdv.received").is_none());
    }

    #[tokio::test]
    async fn test_router_detects_invoice_cii() {
        let xml = b"<rsm:CrossIndustryInvoice xmlns:rsm=\"urn:un:unece:uncefact:data:standard:CrossIndustryInvoice:100\"/>";
        let router = DocumentTypeRouter::new();
        let exchange = Exchange::new(xml.to_vec());
        let result = router.process(exchange).await.unwrap();

        assert_eq!(result.get_header("document.type").map(|s| s.as_str()), Some("Invoice"));
    }

    #[tokio::test]
    async fn test_router_skips_irrecevable() {
        let xml = std::fs::read("../../tests/fixtures/cdar/cdv_200_deposee.xml").unwrap();
        let router = DocumentTypeRouter::new();
        let mut exchange = Exchange::new(xml);
        exchange.set_property("reception.failed", "true");
        let result = router.process(exchange).await.unwrap();

        // Ne doit pas détecter le type si irrecevable
        assert!(result.get_header("document.type").is_none());
    }

    #[tokio::test]
    async fn test_router_cdar_source_peppol() {
        let xml = std::fs::read("../../tests/fixtures/cdar/cdv_200_deposee.xml").unwrap();
        let router = DocumentTypeRouter::new();
        let mut exchange = Exchange::new(xml);
        exchange.set_header("source.protocol", "peppol-as4");
        let result = router.process(exchange).await.unwrap();

        assert_eq!(result.get_property("cdv.source").map(|s| s.as_str()), Some("peppol"));
    }

    #[tokio::test]
    async fn test_router_cdar_source_afnor() {
        let xml = std::fs::read("../../tests/fixtures/cdar/cdv_200_deposee.xml").unwrap();
        let router = DocumentTypeRouter::new();
        let mut exchange = Exchange::new(xml);
        exchange.set_header("source.protocol", "afnor-flow");
        let result = router.process(exchange).await.unwrap();

        assert_eq!(result.get_property("cdv.source").map(|s| s.as_str()), Some("afnor"));
    }

    #[tokio::test]
    async fn test_router_cdar_source_ppf() {
        let xml = std::fs::read("../../tests/fixtures/cdar/cdv_200_deposee.xml").unwrap();
        let router = DocumentTypeRouter::new();
        let exchange = Exchange::new(xml).with_filename("FFE0614A_AAA123_flux001.xml");
        let result = router.process(exchange).await.unwrap();

        assert_eq!(result.get_property("cdv.source").map(|s| s.as_str()), Some("ppf"));
    }

    #[tokio::test]
    async fn test_router_cdar_rejetee_adds_error() {
        let xml = std::fs::read("../../tests/fixtures/cdar/cdv_213_rejetee.xml").unwrap();
        let router = DocumentTypeRouter::new();
        let exchange = Exchange::new(xml);
        let result = router.process(exchange).await.unwrap();

        assert_eq!(result.get_header("document.type").map(|s| s.as_str()), Some("CDAR"));
        assert_eq!(result.get_property("cdv.status_code").map(|s| s.as_str()), Some("213"));
        assert_eq!(result.status, FlowStatus::Rejected);
        assert!(result.has_errors());
    }

    #[test]
    fn test_determine_cdar_source_default() {
        let exchange = Exchange::new(Vec::new());
        assert_eq!(determine_cdar_source(&exchange), "client");
    }

    #[test]
    fn test_determine_cdar_source_peppol_header() {
        let mut exchange = Exchange::new(Vec::new());
        exchange.set_header("source.protocol", "peppol-as4");
        assert_eq!(determine_cdar_source(&exchange), "peppol");
    }

    #[test]
    fn test_determine_cdar_source_peppol_property() {
        let mut exchange = Exchange::new(Vec::new());
        exchange.set_property("peppol.received", "true");
        assert_eq!(determine_cdar_source(&exchange), "peppol");
    }

    #[test]
    fn test_determine_cdar_source_afnor() {
        let mut exchange = Exchange::new(Vec::new());
        exchange.set_header("source.protocol", "afnor-flow");
        assert_eq!(determine_cdar_source(&exchange), "afnor");
    }

    #[test]
    fn test_determine_cdar_source_ppf_filename() {
        let exchange = Exchange::new(Vec::new()).with_filename("FFE0654A_AAA123_flux.xml");
        assert_eq!(determine_cdar_source(&exchange), "ppf");
    }
}
