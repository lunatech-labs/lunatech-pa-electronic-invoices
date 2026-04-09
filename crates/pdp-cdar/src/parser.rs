use pdp_core::error::{PdpError, PdpResult};
use roxmltree::Document;

use crate::model::*;

/// Parser de CDV (Compte-rendu De Vie) au format
/// CrossDomainAcknowledgementAndResponse D22B
pub struct CdarParser;

impl CdarParser {
    pub fn new() -> Self {
        Self
    }

    /// Parse un XML CDV conforme D22B en structure CdvResponse
    pub fn parse(&self, xml: &str) -> PdpResult<CdvResponse> {
        let doc = Document::parse(xml)
            .map_err(|e| PdpError::CdarError(format!("XML CDV invalide: {}", e)))?;

        let root = doc.root_element();

        // ExchangedDocumentContext
        let business_process = self.find_in_parent(&root, "BusinessProcessSpecifiedDocumentContextParameter", "ID")
            .unwrap_or_else(|| "REGULATED".to_string());
        let guideline_id = self.find_in_parent(&root, "GuidelineSpecifiedDocumentContextParameter", "ID")
            .unwrap_or_default();

        // ExchangedDocument
        let exchanged_doc = root.descendants()
            .find(|n| n.tag_name().name() == "ExchangedDocument")
            .ok_or_else(|| PdpError::CdarError("ExchangedDocument manquant".to_string()))?;

        let document_id = self.find_child_text(&exchanged_doc, "ID")
            .unwrap_or_default();
        let document_name = self.find_child_text(&exchanged_doc, "Name");
        let issue_datetime = self.find_datetime_string(&exchanged_doc, "IssueDateTime")
            .unwrap_or_default();

        // SenderTradeParty
        let sender = self.parse_trade_party_in(&exchanged_doc, "SenderTradeParty")
            .unwrap_or_else(|| TradeParty {
                global_id: None, global_id_scheme: None, name: None,
                role_code: RoleCode::WK, endpoint_id: None, endpoint_scheme: None,
            });

        // IssuerTradeParty
        let issuer = self.parse_trade_party_in(&exchanged_doc, "IssuerTradeParty");

        // RecipientTradeParty(s)
        let recipients: Vec<TradeParty> = exchanged_doc.children()
            .filter(|n| n.tag_name().name() == "RecipientTradeParty")
            .filter_map(|n| self.parse_trade_party_node(&n))
            .collect();

        // AcknowledgementDocument
        let ack_doc = root.descendants()
            .find(|n| n.tag_name().name() == "AcknowledgementDocument")
            .ok_or_else(|| PdpError::CdarError("AcknowledgementDocument manquant".to_string()))?;

        let multiple_references = self.find_in_parent(&ack_doc, "MultipleReferencesIndicator", "Indicator")
            .map(|s| s == "true")
            .unwrap_or(false);

        let type_code_str = self.find_child_text(&ack_doc, "TypeCode")
            .unwrap_or_else(|| "305".to_string());
        let type_code = CdvTypeCode::from_code(&type_code_str)
            .unwrap_or(CdvTypeCode::Transmission);

        let status_datetime = self.find_datetime_string(&ack_doc, "IssueDateTime")
            .unwrap_or_default();

        // ReferenceReferencedDocument(s)
        let referenced_documents: Vec<ReferencedDocument> = ack_doc.children()
            .filter(|n| n.tag_name().name() == "ReferenceReferencedDocument")
            .map(|n| self.parse_referenced_document(&n))
            .collect();

        let cdv = CdvResponse {
            business_process,
            guideline_id,
            document_id,
            document_name,
            issue_datetime,
            sender,
            issuer,
            recipients,
            multiple_references,
            type_code,
            status_datetime,
            referenced_documents,
        };

        let status_code = cdv.status_code().unwrap_or(0);
        let invoice_id = cdv.referenced_documents.first()
            .map(|d| d.invoice_id.as_str())
            .unwrap_or("?");

        tracing::info!(
            cdv_id = %cdv.document_id,
            invoice = %invoice_id,
            status_code = status_code,
            type_code = %cdv.type_code,
            success = cdv.is_success(),
            "CDV parsé"
        );

        Ok(cdv)
    }

    fn find_child_text(&self, node: &roxmltree::Node<'_, '_>, name: &str) -> Option<String> {
        node.children()
            .find(|n| n.tag_name().name() == name)
            .and_then(|n| {
                n.text().map(|t| t.trim().to_string())
                    .filter(|s| !s.is_empty())
            })
    }

    fn find_in_parent(&self, root: &roxmltree::Node<'_, '_>, parent_name: &str, child_name: &str) -> Option<String> {
        let parent = root.descendants().find(|n| n.tag_name().name() == parent_name)?;
        parent.descendants()
            .find(|n| n.tag_name().name() == child_name)
            .and_then(|n| n.text())
            .map(|t| t.trim().to_string())
            .filter(|s| !s.is_empty())
    }

    fn find_datetime_string(&self, parent: &roxmltree::Node<'_, '_>, datetime_tag: &str) -> Option<String> {
        let dt_node = parent.children()
            .find(|n| n.tag_name().name() == datetime_tag)?;
        dt_node.descendants()
            .find(|n| n.tag_name().name() == "DateTimeString")
            .and_then(|n| n.text())
            .map(|t| t.trim().to_string())
            .filter(|s| !s.is_empty())
    }

    fn parse_trade_party_in(&self, parent: &roxmltree::Node<'_, '_>, tag: &str) -> Option<TradeParty> {
        let node = parent.children().find(|n| n.tag_name().name() == tag)?;
        self.parse_trade_party_node(&node)
    }

    fn parse_trade_party_node(&self, node: &roxmltree::Node<'_, '_>) -> Option<TradeParty> {
        let role_code_str = self.find_child_text(node, "RoleCode")?;
        let role_code = RoleCode::from_code(&role_code_str)?;

        let global_id_node = node.children().find(|n| n.tag_name().name() == "GlobalID");
        let global_id = global_id_node.as_ref()
            .and_then(|n| n.text())
            .map(|t| t.trim().to_string())
            .filter(|s| !s.is_empty());
        let global_id_scheme = global_id_node.as_ref()
            .and_then(|n| n.attribute("schemeID"))
            .map(|s| s.to_string());

        let name = self.find_child_text(node, "Name");

        let uri_comm = node.children()
            .find(|n| n.tag_name().name() == "URIUniversalCommunication");
        let (endpoint_id, endpoint_scheme) = if let Some(uri_node) = uri_comm {
            let uriid_node = uri_node.children()
                .find(|n| n.tag_name().name() == "URIID");
            let eid = uriid_node.as_ref()
                .and_then(|n| n.text())
                .map(|t| t.trim().to_string())
                .filter(|s| !s.is_empty());
            let es = uriid_node.as_ref()
                .and_then(|n| n.attribute("schemeID"))
                .map(|s| s.to_string());
            (eid, es)
        } else {
            (None, None)
        };

        Some(TradeParty {
            global_id,
            global_id_scheme,
            name,
            role_code,
            endpoint_id,
            endpoint_scheme,
        })
    }

    fn parse_referenced_document(&self, node: &roxmltree::Node<'_, '_>) -> ReferencedDocument {
        let invoice_id = self.find_child_text(node, "IssuerAssignedID")
            .unwrap_or_default();
        let status_code = self.find_child_text(node, "StatusCode")
            .and_then(|s| s.parse::<u16>().ok());
        let type_code = self.find_child_text(node, "TypeCode");
        let receipt_datetime = self.find_datetime_string(node, "ReceiptDateTime");
        let issue_date = self.find_datetime_string(node, "FormattedIssueDateTime");
        let process_condition_code = self.find_child_text(node, "ProcessConditionCode")
            .and_then(|s| s.parse::<u16>().ok())
            .unwrap_or(0);
        let process_condition = self.find_child_text(node, "ProcessCondition");

        let issuer = self.parse_trade_party_in(node, "IssuerTradeParty");
        let recipient = self.parse_trade_party_in(node, "RecipientTradeParty");

        let statuses: Vec<DocumentStatus> = node.children()
            .filter(|n| n.tag_name().name() == "SpecifiedDocumentStatus")
            .map(|n| self.parse_document_status(&n))
            .collect();

        ReferencedDocument {
            invoice_id,
            status_code,
            type_code,
            receipt_datetime,
            issue_date,
            process_condition_code,
            process_condition,
            issuer,
            recipient,
            statuses,
        }
    }

    fn parse_document_status(&self, node: &roxmltree::Node<'_, '_>) -> DocumentStatus {
        let status_code = self.find_child_text(node, "ProcessConditionCode")
            .and_then(|s| s.parse::<u16>().ok());
        let reason_code = self.find_child_text(node, "ReasonCode");
        let reason = self.find_child_text(node, "Reason");
        let action_code = self.find_child_text(node, "RequestedActionCode");
        let action = self.find_child_text(node, "RequestedAction");
        let sequence = self.find_child_text(node, "SequenceNumeric")
            .and_then(|s| s.parse::<u32>().ok());

        let characteristics: Vec<DocumentCharacteristic> = node.children()
            .filter(|n| n.tag_name().name() == "SpecifiedDocumentCharacteristic")
            .map(|n| self.parse_characteristic(&n))
            .collect();

        DocumentStatus {
            status_code,
            reason_code,
            reason,
            action_code,
            action,
            sequence,
            characteristics,
        }
    }

    fn parse_characteristic(&self, node: &roxmltree::Node<'_, '_>) -> DocumentCharacteristic {
        let id = self.find_child_text(node, "ID");
        let type_code = self.find_child_text(node, "TypeCode").unwrap_or_default();
        let value_changed = self.find_in_parent(node, "ValueChangedIndicator", "IndicatorString")
            .map(|s| s == "true");
        let name = self.find_child_text(node, "Name");
        let location = self.find_child_text(node, "Location");
        let value_percent = self.find_child_text(node, "ValuePercent");
        let value_amount = self.find_child_text(node, "ValueAmount");

        DocumentCharacteristic {
            id,
            type_code,
            value_changed,
            name,
            location,
            value_percent,
            value_amount,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_parse_cdv_deposee() {
        let xml = fs::read_to_string("../../tests/fixtures/cdar/cdv_200_deposee.xml")
            .expect("Fixture CDV introuvable");

        let parser = CdarParser::new();
        let cdv = parser.parse(&xml).expect("Parsing CDV échoué");

        assert_eq!(cdv.guideline_id, "urn.cpro.gouv.fr:1p0:CDV:invoice");
        assert_eq!(cdv.business_process, "REGULATED");
        assert_eq!(cdv.type_code, CdvTypeCode::Transmission);
        assert!(cdv.is_success());
        assert!(!cdv.is_rejected());

        let doc = &cdv.referenced_documents[0];
        assert_eq!(doc.invoice_id, "F202500001");
        assert_eq!(doc.process_condition_code, 200);
        assert_eq!(doc.status_code, Some(10));
        assert_eq!(doc.type_code, Some("380".to_string()));
    }

    #[test]
    fn test_parse_cdv_rejetee() {
        let xml = fs::read_to_string("../../tests/fixtures/cdar/cdv_213_rejetee.xml")
            .expect("Fixture CDV rejet introuvable");

        let parser = CdarParser::new();
        let cdv = parser.parse(&xml).expect("Parsing CDV rejet échoué");

        assert!(cdv.is_rejected());
        assert!(!cdv.is_success());

        let doc = &cdv.referenced_documents[0];
        assert_eq!(doc.process_condition_code, 213);
        assert!(!doc.statuses.is_empty());
        assert_eq!(doc.statuses[0].reason_code, Some("REJ_SEMAN".to_string()));
    }

    #[test]
    fn test_parse_cdv_litige() {
        let xml = fs::read_to_string("../../tests/fixtures/cdar/cdv_207_litige.xml")
            .expect("Fixture CDV litige introuvable");

        let parser = CdarParser::new();
        let cdv = parser.parse(&xml).expect("Parsing CDV litige échoué");

        assert_eq!(cdv.type_code, CdvTypeCode::Traitement);
        let doc = &cdv.referenced_documents[0];
        assert_eq!(doc.process_condition_code, 207);
        assert_eq!(doc.statuses[0].reason_code, Some("TX_TVA_ERR".to_string()));
        assert_eq!(doc.statuses[0].action_code, Some("NIN".to_string()));
        assert!(!doc.statuses[0].characteristics.is_empty());
    }

    #[test]
    fn test_parse_cdv_recue_202() {
        let xml = fs::read_to_string("../../tests/fixtures/cdar/cdv_202_recue.xml")
            .expect("Fixture CDV 202 introuvable");

        let parser = CdarParser::new();
        let cdv = parser.parse(&xml).expect("Parsing CDV 202 échoué");

        assert!(cdv.is_success());
        let doc = &cdv.referenced_documents[0];
        assert_eq!(doc.invoice_id, "F202500003");
        assert_eq!(doc.process_condition_code, 202);
        assert_eq!(doc.process_condition.as_deref(), Some("Reçue"));
        // Destinataire = acheteur
        assert_eq!(cdv.recipients[0].role_code, RoleCode::BY);
        assert_eq!(cdv.recipients[0].name.as_deref(), Some("ACHETEUR"));
    }

    #[test]
    fn test_parse_cdv_approuvee_205() {
        let xml = fs::read_to_string("../../tests/fixtures/cdar/cdv_205_approuvee.xml")
            .expect("Fixture CDV 205 introuvable");

        let parser = CdarParser::new();
        let cdv = parser.parse(&xml).expect("Parsing CDV 205 échoué");

        assert!(cdv.is_success());
        let doc = &cdv.referenced_documents[0];
        assert_eq!(doc.invoice_id, "F202500003");
        assert_eq!(doc.process_condition_code, 205);
        assert_eq!(doc.process_condition.as_deref(), Some("Approuvée"));
        // Émetteur = acheteur (c'est lui qui approuve)
        assert_eq!(cdv.sender.role_code, RoleCode::BY);
        // Destinataire = vendeur
        assert_eq!(cdv.recipients[0].role_code, RoleCode::SE);
    }

    #[test]
    fn test_parse_cdar_response_001() {
        let xml = fs::read_to_string("../../tests/fixtures/cdar/cdar_response_001.xml")
            .expect("Fixture CDAR response 001 introuvable");

        let parser = CdarParser::new();
        let cdv = parser.parse(&xml).expect("Parsing CDAR response 001 échoué");

        assert!(cdv.is_success());
        let doc = &cdv.referenced_documents[0];
        assert_eq!(doc.invoice_id, "FA-2025-00142");
        assert_eq!(doc.process_condition_code, 200);
        assert_eq!(doc.type_code, Some("380".to_string()));
    }

    #[test]
    fn test_parse_cdar_response_002_rejet() {
        let xml = fs::read_to_string("../../tests/fixtures/cdar/cdar_response_002_rejet.xml")
            .expect("Fixture CDAR response 002 introuvable");

        let parser = CdarParser::new();
        let cdv = parser.parse(&xml).expect("Parsing CDAR response 002 échoué");

        assert!(cdv.is_rejected());
        assert!(!cdv.is_success());
        let doc = &cdv.referenced_documents[0];
        assert_eq!(doc.invoice_id, "FA-2025-00256");
        assert_eq!(doc.process_condition_code, 213);
        // 2 SpecifiedDocumentStatus
        assert_eq!(doc.statuses.len(), 2);
        assert_eq!(doc.statuses[0].reason_code.as_deref(), Some("REJ_SEMAN"));
        assert_eq!(doc.statuses[0].sequence, Some(1));
        assert_eq!(doc.statuses[1].sequence, Some(2));
    }

    // ===== Tests sur exemples officiels AFNOR XP Z12-014 v1.2 =====

    #[test]
    fn test_parse_official_uc1_cdv_200_deposee() {
        let xml = fs::read_to_string("../../specs/examples/xp-z12-014/UC1/UC1_F202500003_01-CDV-200_Deposee.xml")
            .expect("Fixture officielle UC1 CDV 200 introuvable");

        let parser = CdarParser::new();
        let cdv = parser.parse(&xml).expect("Parsing UC1 CDV 200 échoué");

        assert_eq!(cdv.document_id, "F202500003_200_20250701151000#380_20250701");
        assert_eq!(cdv.document_name.as_deref(), Some("UC1_F202500003_01-CDV-200_Deposee"));
        assert_eq!(cdv.business_process, "REGULATED");
        assert_eq!(cdv.guideline_id, "urn.cpro.gouv.fr:1p0:CDV:invoice");
        // Sender = WK (PDP)
        assert_eq!(cdv.sender.role_code, RoleCode::WK);
        // Issuer = WK
        assert_eq!(cdv.issuer.as_ref().unwrap().role_code, RoleCode::WK);
        // Recipients: SE (vendeur) + DFH (PPF)
        assert!(cdv.recipients.len() >= 2);
        assert!(cdv.recipients.iter().any(|r| r.role_code == RoleCode::SE));
        assert!(cdv.recipients.iter().any(|r| r.role_code == RoleCode::DFH));
        // AcknowledgementDocument
        assert_eq!(cdv.type_code, CdvTypeCode::Transmission);
        // Referenced document
        assert_eq!(cdv.referenced_documents.len(), 1);
        let doc = &cdv.referenced_documents[0];
        assert_eq!(doc.invoice_id, "F202500003");
        assert_eq!(doc.process_condition_code, 200);
        assert_eq!(doc.process_condition.as_deref(), Some("Déposée"));
        assert_eq!(doc.type_code, Some("380".to_string()));
    }

    #[test]
    fn test_parse_official_uc2_cdv_213_rejetee() {
        let xml = fs::read_to_string("../../specs/examples/xp-z12-014/UC2/UC2_F202500004_02-CDV-213_Rejetee.xml")
            .expect("Fixture officielle UC2 CDV 213 introuvable");

        let parser = CdarParser::new();
        let cdv = parser.parse(&xml).expect("Parsing UC2 CDV 213 échoué");

        assert!(cdv.is_rejected());
        assert!(!cdv.is_success());
        let doc = &cdv.referenced_documents[0];
        assert_eq!(doc.invoice_id, "F202500004");
        assert_eq!(doc.process_condition_code, 213);
        assert_eq!(doc.process_condition.as_deref(), Some("Rejetée"));
        // Motif de rejet
        assert_eq!(doc.statuses.len(), 1);
        assert_eq!(doc.statuses[0].reason_code.as_deref(), Some("DOUBLON"));
        assert_eq!(doc.statuses[0].reason.as_deref(), Some("Facture en doublon (déjà émise / réçue)"));
        // 3 destinataires: SE, BY, DFH
        assert_eq!(cdv.recipients.len(), 3);
    }

    #[test]
    fn test_parse_official_uc3_cdv_210_refusee() {
        let xml = fs::read_to_string("../../specs/examples/xp-z12-014/UC3/UC3_F202500005_04-CDV-210_Refusee.xml")
            .expect("Fixture officielle UC3 CDV 210 introuvable");

        let parser = CdarParser::new();
        let cdv = parser.parse(&xml).expect("Parsing UC3 CDV 210 échoué");

        let doc = &cdv.referenced_documents[0];
        assert_eq!(doc.invoice_id, "F202500005");
        assert_eq!(doc.process_condition_code, 210);
        assert_eq!(doc.process_condition.as_deref(), Some("Refusée"));
        // TypeCode 23 = Traitement (émis par l'acheteur)
        assert_eq!(cdv.type_code, CdvTypeCode::Traitement);
        // Issuer = BY (acheteur qui refuse)
        let issuer = cdv.issuer.as_ref().unwrap();
        assert_eq!(issuer.role_code, RoleCode::BY);
        assert_eq!(issuer.name.as_deref(), Some("ACHETEUR"));
        // Motif
        assert_eq!(doc.statuses[0].reason_code.as_deref(), Some("TX_TVA_ERR"));
    }

    #[test]
    fn test_parse_official_uc4_cdv_207_en_litige() {
        let xml = fs::read_to_string("../../specs/examples/xp-z12-014/UC4/UC4_F202500006_04-CDV-207_En_litige.xml")
            .expect("Fixture officielle UC4 CDV 207 introuvable");

        let parser = CdarParser::new();
        let cdv = parser.parse(&xml).expect("Parsing UC4 CDV 207 échoué");

        let doc = &cdv.referenced_documents[0];
        assert_eq!(doc.invoice_id, "F202500006");
        assert_eq!(doc.process_condition_code, 207);
        assert_eq!(doc.process_condition.as_deref(), Some("En_litige"));
        // TypeCode 23 = Traitement
        assert_eq!(cdv.type_code, CdvTypeCode::Traitement);
        // Motif avec action attendue
        assert_eq!(doc.statuses[0].reason_code.as_deref(), Some("TX_TVA_ERR"));
        assert_eq!(doc.statuses[0].action_code.as_deref(), Some("NIN"));
        // DocumentCharacteristic (BT-152 taux TVA)
        assert!(doc.statuses[0].characteristics.len() >= 2);
        let char1 = &doc.statuses[0].characteristics[0];
        assert_eq!(char1.id.as_deref(), Some("BT-152"));
        assert_eq!(char1.type_code, "DIV");
    }

    #[test]
    fn test_parse_cdv_invalid_xml() {
        let parser = CdarParser::new();
        let result = parser.parse("not xml");
        assert!(result.is_err());
    }

    // ===== Tests exhaustifs : toutes les fixtures CDAR par statut =====

    /// Helper : parse une fixture et vérifie le code statut + label + phase
    fn assert_fixture_status(filename: &str, expected_code: u16, expected_label: &str, expected_type_code: CdvTypeCode) {
        let path = format!("../../tests/fixtures/cdar/{}", filename);
        let xml = fs::read_to_string(&path)
            .unwrap_or_else(|_| panic!("Fixture {} introuvable", path));
        let parser = CdarParser::new();
        let cdv = parser.parse(&xml)
            .unwrap_or_else(|e| panic!("Parsing {} échoué: {}", filename, e));

        assert_eq!(cdv.type_code, expected_type_code,
            "{}: TypeCode attendu {:?}", filename, expected_type_code);
        assert_eq!(cdv.referenced_documents.len(), 1,
            "{}: doit avoir 1 document référencé", filename);
        let doc = &cdv.referenced_documents[0];
        assert_eq!(doc.process_condition_code, expected_code,
            "{}: ProcessConditionCode attendu {}", filename, expected_code);
        assert_eq!(doc.process_condition.as_deref(), Some(expected_label),
            "{}: ProcessCondition attendu '{}'", filename, expected_label);
    }

    // --- Phase Transmission (305) ---

    #[test]
    fn test_parse_fixture_201_emise() {
        assert_fixture_status("cdv_201_emise.xml", 201, "Émise", CdvTypeCode::Transmission);
    }

    #[test]
    fn test_parse_fixture_202_recue() {
        assert_fixture_status("cdv_202_recue.xml", 202, "Reçue", CdvTypeCode::Transmission);
    }

    #[test]
    fn test_parse_fixture_203_mise_a_disposition() {
        assert_fixture_status("cdv_203_mise_a_disposition.xml", 203, "Mise_à_disposition", CdvTypeCode::Transmission);
    }

    // --- Phase Traitement (23) ---

    #[test]
    fn test_parse_fixture_204_prise_en_charge() {
        assert_fixture_status("cdv_204_prise_en_charge.xml", 204, "Prise_en_charge", CdvTypeCode::Traitement);
    }

    #[test]
    fn test_parse_fixture_205_approuvee() {
        assert_fixture_status("cdv_205_approuvee.xml", 205, "Approuvée", CdvTypeCode::Traitement);
    }

    #[test]
    fn test_parse_fixture_206_approuvee_partiellement() {
        assert_fixture_status("cdv_206_approuvee_partiellement.xml", 206, "Approuvée_partiellement", CdvTypeCode::Traitement);
    }

    #[test]
    fn test_parse_fixture_208_suspendue() {
        assert_fixture_status("cdv_208_suspendue.xml", 208, "Suspendue", CdvTypeCode::Traitement);
    }

    #[test]
    fn test_parse_fixture_209_completee() {
        assert_fixture_status("cdv_209_completee.xml", 209, "Complétée", CdvTypeCode::Traitement);
    }

    #[test]
    fn test_parse_fixture_211_paiement_transmis() {
        assert_fixture_status("cdv_211_paiement_transmis.xml", 211, "Paiement_transmis", CdvTypeCode::Traitement);
    }

    #[test]
    fn test_parse_fixture_214_mise_en_paiement() {
        assert_fixture_status("cdv_214_visee.xml", 214, "Visée", CdvTypeCode::Traitement);
    }

    // --- Statuts avec motifs ---

    #[test]
    fn test_parse_fixture_210_refusee_avec_motif() {
        let xml = fs::read_to_string("../../tests/fixtures/cdar/cdv_210_refusee.xml")
            .expect("Fixture CDV 210 introuvable");
        let parser = CdarParser::new();
        let cdv = parser.parse(&xml).expect("Parsing CDV 210 échoué");

        assert_eq!(cdv.type_code, CdvTypeCode::Traitement);
        let doc = &cdv.referenced_documents[0];
        assert_eq!(doc.process_condition_code, 210);
        assert_eq!(doc.status_code, Some(8));
        assert_eq!(doc.statuses.len(), 1);
        assert_eq!(doc.statuses[0].reason_code.as_deref(), Some("NON_CONFORME"));
        assert_eq!(doc.statuses[0].action_code.as_deref(), Some("NIN"));
    }

    #[test]
    fn test_parse_fixture_501_irrecevable_avec_motif() {
        let xml = fs::read_to_string("../../tests/fixtures/cdar/cdv_501_irrecevable.xml")
            .expect("Fixture CDV 501 introuvable");
        let parser = CdarParser::new();
        let cdv = parser.parse(&xml).expect("Parsing CDV 501 échoué");

        assert_eq!(cdv.type_code, CdvTypeCode::Transmission);
        assert!(cdv.is_irrecevable());
        assert!(!cdv.is_success());
        let doc = &cdv.referenced_documents[0];
        assert_eq!(doc.process_condition_code, 501);
        assert_eq!(doc.status_code, Some(8));
        assert_eq!(doc.statuses.len(), 1);
        assert_eq!(doc.statuses[0].reason_code.as_deref(), Some("IRR_SYNTAX"));
    }

    #[test]
    fn test_parse_fixture_221_erreur_routage_avec_motif() {
        let xml = fs::read_to_string("../../tests/fixtures/cdar/cdv_221_erreur_routage.xml")
            .expect("Fixture CDV 221 introuvable");
        let parser = CdarParser::new();
        let cdv = parser.parse(&xml).expect("Parsing CDV 221 échoué");

        assert_eq!(cdv.type_code, CdvTypeCode::Transmission);
        let doc = &cdv.referenced_documents[0];
        assert_eq!(doc.process_condition_code, 221);
        assert_eq!(doc.statuses[0].reason_code.as_deref(), Some("ROUTAGE_ERR"));
    }

    // --- Statuts avec caractéristiques (MEN) ---

    #[test]
    fn test_parse_fixture_212_encaissee_avec_men() {
        let xml = fs::read_to_string("../../tests/fixtures/cdar/cdv_212_encaissee.xml")
            .expect("Fixture CDV 212 introuvable");
        let parser = CdarParser::new();
        let cdv = parser.parse(&xml).expect("Parsing CDV 212 échoué");

        assert_eq!(cdv.type_code, CdvTypeCode::Traitement);
        // Issuer = SE (vendeur déclare encaissement)
        let issuer = cdv.issuer.as_ref().unwrap();
        assert_eq!(issuer.role_code, RoleCode::SE);
        // 2 destinataires: BY + DFH (PPF)
        assert_eq!(cdv.recipients.len(), 2);
        assert!(cdv.recipients.iter().any(|r| r.role_code == RoleCode::BY));
        assert!(cdv.recipients.iter().any(|r| r.role_code == RoleCode::DFH));

        let doc = &cdv.referenced_documents[0];
        assert_eq!(doc.process_condition_code, 212);
        // MEN characteristic
        assert_eq!(doc.statuses.len(), 1);
        assert_eq!(doc.statuses[0].characteristics.len(), 1);
        let men = &doc.statuses[0].characteristics[0];
        assert_eq!(men.type_code, "MEN");
        assert_eq!(men.value_amount.as_deref(), Some("12000.00"));
        assert_eq!(men.value_percent.as_deref(), Some("20.00"));
    }

    // --- Multi-documents ---

    #[test]
    fn test_parse_fixture_multi_documents() {
        let xml = fs::read_to_string("../../tests/fixtures/cdar/cdv_multi_documents.xml")
            .expect("Fixture CDV multi introuvable");
        let parser = CdarParser::new();
        let cdv = parser.parse(&xml).expect("Parsing CDV multi échoué");

        assert!(cdv.multiple_references);
        assert_eq!(cdv.referenced_documents.len(), 2);
        assert_eq!(cdv.referenced_documents[0].invoice_id, "F202500010");
        assert_eq!(cdv.referenced_documents[0].type_code, Some("380".to_string()));
        assert_eq!(cdv.referenced_documents[0].process_condition_code, 200);
        assert_eq!(cdv.referenced_documents[1].invoice_id, "F202500011");
        assert_eq!(cdv.referenced_documents[1].type_code, Some("381".to_string()));
        assert_eq!(cdv.referenced_documents[1].process_condition_code, 200);
    }

    // --- Statuts avec motifs obligatoires (206, 208) ---

    #[test]
    fn test_parse_fixture_206_approuvee_partiellement_avec_motif() {
        let xml = fs::read_to_string("../../tests/fixtures/cdar/cdv_206_approuvee_partiellement.xml")
            .expect("Fixture CDV 206 introuvable");
        let parser = CdarParser::new();
        let cdv = parser.parse(&xml).expect("Parsing CDV 206 échoué");

        assert_eq!(cdv.type_code, CdvTypeCode::Traitement);
        let doc = &cdv.referenced_documents[0];
        assert_eq!(doc.process_condition_code, 206);
        assert_eq!(doc.status_code, Some(4));
        // Motif conforme BR-FR-CDV-CL-09 (206 autorise QTE_ERR)
        assert_eq!(doc.statuses.len(), 1);
        let status = &doc.statuses[0];
        assert_eq!(status.reason_code.as_deref(), Some("QTE_ERR"));
        assert!(status.reason.is_some());
        assert_eq!(status.action_code.as_deref(), Some("NIN"));
        assert!(status.action.is_some());
    }

    #[test]
    fn test_parse_fixture_208_suspendue_avec_motif() {
        let xml = fs::read_to_string("../../tests/fixtures/cdar/cdv_208_suspendue.xml")
            .expect("Fixture CDV 208 introuvable");
        let parser = CdarParser::new();
        let cdv = parser.parse(&xml).expect("Parsing CDV 208 échoué");

        assert_eq!(cdv.type_code, CdvTypeCode::Traitement);
        let doc = &cdv.referenced_documents[0];
        assert_eq!(doc.process_condition_code, 208);
        assert_eq!(doc.status_code, Some(4));
        // Motif conforme BR-FR-CDV-CL-09 (208 autorise JUSTIF_ABS)
        assert_eq!(doc.statuses.len(), 1);
        let status = &doc.statuses[0];
        assert_eq!(status.reason_code.as_deref(), Some("JUSTIF_ABS"));
        assert!(status.reason.is_some());
        assert_eq!(status.action_code.as_deref(), Some("PIN"));
        assert!(status.action.is_some());
    }

    // --- Statuts étendus (220-228) ---

    #[test]
    fn test_parse_fixture_220_annulee() {
        assert_fixture_status("cdv_220_annulee.xml", 220, "Annulée", CdvTypeCode::Traitement);
    }

    #[test]
    fn test_parse_fixture_224_demande_paiement_direct() {
        assert_fixture_status("cdv_224_demande_paiement_direct.xml", 224, "Demande_de_Paiement_Direct", CdvTypeCode::Traitement);
    }

    #[test]
    fn test_parse_fixture_225_affacturee() {
        assert_fixture_status("cdv_225_affacturee.xml", 225, "Affacturée", CdvTypeCode::Traitement);
    }

    #[test]
    fn test_parse_fixture_226_affacturee_confidentiel() {
        assert_fixture_status("cdv_226_affacturee_confidentiel.xml", 226, "Affacturée_Confidentiel", CdvTypeCode::Traitement);
    }

    #[test]
    fn test_parse_fixture_227_changement_compte_a_payer() {
        let xml = fs::read_to_string("../../tests/fixtures/cdar/cdv_227_changement_compte_a_payer.xml")
            .expect("Fixture CDV 227 introuvable");
        let parser = CdarParser::new();
        let cdv = parser.parse(&xml).expect("Parsing CDV 227 échoué");

        assert_eq!(cdv.type_code, CdvTypeCode::Traitement);
        let doc = &cdv.referenced_documents[0];
        assert_eq!(doc.process_condition_code, 227);
        assert_eq!(doc.status_code, Some(8));
        assert_eq!(doc.statuses.len(), 1);
        let status = &doc.statuses[0];
        assert_eq!(status.reason_code.as_deref(), Some("AUTRE"));
        assert_eq!(status.action_code.as_deref(), Some("NIN"));
    }

    #[test]
    fn test_parse_fixture_228_non_affacturee() {
        let xml = fs::read_to_string("../../tests/fixtures/cdar/cdv_228_non_affacturee.xml")
            .expect("Fixture CDV 228 introuvable");
        let parser = CdarParser::new();
        let cdv = parser.parse(&xml).expect("Parsing CDV 228 échoué");

        assert_eq!(cdv.type_code, CdvTypeCode::Traitement);
        let doc = &cdv.referenced_documents[0];
        assert_eq!(doc.process_condition_code, 228);
        assert_eq!(doc.status_code, Some(8));
        assert_eq!(doc.statuses.len(), 1);
        let status = &doc.statuses[0];
        assert_eq!(status.reason_code.as_deref(), Some("AUTRE"));
        assert_eq!(status.action_code.as_deref(), Some("NIN"));
    }

    // --- Validation is_success / is_rejected / is_irrecevable ---

    #[test]
    fn test_fixture_status_classification() {
        let parser = CdarParser::new();

        // Statuts "success" (200-228 sauf 210, 213, 220, 221)
        for (file, code) in &[
            ("cdv_200_deposee.xml", 200u16),
            ("cdv_201_emise.xml", 201),
            ("cdv_202_recue.xml", 202),
            ("cdv_203_mise_a_disposition.xml", 203),
            ("cdv_204_prise_en_charge.xml", 204),
            ("cdv_205_approuvee.xml", 205),
            ("cdv_206_approuvee_partiellement.xml", 206),
            ("cdv_208_suspendue.xml", 208),
            ("cdv_209_completee.xml", 209),
            ("cdv_211_paiement_transmis.xml", 211),
            ("cdv_212_encaissee.xml", 212),
            ("cdv_214_visee.xml", 214),
            ("cdv_224_demande_paiement_direct.xml", 224),
            ("cdv_225_affacturee.xml", 225),
            ("cdv_226_affacturee_confidentiel.xml", 226),
            ("cdv_227_changement_compte_a_payer.xml", 227),
            ("cdv_228_non_affacturee.xml", 228),
        ] {
            let xml = fs::read_to_string(format!("../../tests/fixtures/cdar/{}", file))
                .unwrap_or_else(|_| panic!("Fixture {} introuvable", file));
            let cdv = parser.parse(&xml).unwrap();
            assert!(cdv.is_success(), "CDV {} (code {}) devrait être success", file, code);
            assert!(!cdv.is_rejected(), "CDV {} ne devrait pas être rejected", file);
        }

        // Statuts "rejected"
        let xml = fs::read_to_string("../../tests/fixtures/cdar/cdv_213_rejetee.xml").unwrap();
        let cdv = parser.parse(&xml).unwrap();
        assert!(cdv.is_rejected());
        assert!(!cdv.is_success());

        // Statuts "refusée" (210) = not success
        let xml = fs::read_to_string("../../tests/fixtures/cdar/cdv_210_refusee.xml").unwrap();
        let cdv = parser.parse(&xml).unwrap();
        assert!(!cdv.is_success());

        // Statuts "irrecevable"
        let xml = fs::read_to_string("../../tests/fixtures/cdar/cdv_501_irrecevable.xml").unwrap();
        let cdv = parser.parse(&xml).unwrap();
        assert!(cdv.is_irrecevable());
        assert!(!cdv.is_success());

        // Statut "annulée" (220) = not success, not rejected, is_annulee
        let xml = fs::read_to_string("../../tests/fixtures/cdar/cdv_220_annulee.xml").unwrap();
        let cdv = parser.parse(&xml).unwrap();
        assert!(!cdv.is_success(), "CDV 220 Annulée ne devrait pas être success");
        assert!(!cdv.is_rejected(), "CDV 220 Annulée ne devrait pas être rejected");
        assert!(cdv.is_annulee(), "CDV 220 devrait être annulée");
    }
}
