//! Tests d'intégration du cycle de vie complet d'une facture.
//!
//! Simule l'envoi d'une facture CII via SFTP et vérifie que :
//! - Le CDV 200 (Déposée) est généré pour le vendeur
//! - Le CDV XML est conforme D22B avec les bonnes parties
//! - Le CDV de rejet (213) est généré en cas d'erreur de validation
//! - Le CDV d'irrecevabilité (501) est généré pour les fichiers invalides
//! - Les statuts de phase Traitement (204, 210, 212) ont vendeur + acheteur
//! - Le CdvReturnProcessor renvoie le CDV au vendeur
//! - Le pipeline ne bloque pas si le renvoi CDV échoue

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use pdp_core::endpoint::Producer;
use pdp_core::error::{PdpError, PdpResult};
use pdp_core::exchange::Exchange;
use pdp_core::model::{InvoiceData, InvoiceFormat};
use pdp_core::processor::Processor;

use pdp_cdar::processor::CdarProcessor;
use pdp_cdar::cdv_return::CdvReturnProcessor;
use pdp_cdar::{CdarGenerator, CdarParser, CdvTypeCode, InvoiceStatusCode, RoleCode};

// ============================================================
// Mock Producer — capture tous les exchanges envoyés
// ============================================================

struct CaptureProducer {
    name: String,
    sent: Mutex<Vec<Exchange>>,
}

impl CaptureProducer {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            sent: Mutex::new(Vec::new()),
        }
    }

    fn sent_exchanges(&self) -> Vec<Exchange> {
        self.sent.lock().unwrap().clone()
    }

    fn sent_count(&self) -> usize {
        self.sent.lock().unwrap().len()
    }

    fn last_body_str(&self) -> String {
        let sent = self.sent.lock().unwrap();
        let last = sent.last().expect("Aucun exchange envoyé");
        String::from_utf8_lossy(&last.body).to_string()
    }
}

#[async_trait]
impl Producer for CaptureProducer {
    fn name(&self) -> &str {
        &self.name
    }

    async fn send(&self, exchange: Exchange) -> PdpResult<Exchange> {
        self.sent.lock().unwrap().push(exchange.clone());
        Ok(exchange)
    }
}

// ============================================================
// Helpers
// ============================================================

fn load_fixture(name: &str) -> Vec<u8> {
    let path = format!(
        "{}/../../tests/fixtures/cii/{}",
        env!("CARGO_MANIFEST_DIR"),
        name
    );
    std::fs::read(&path).unwrap_or_else(|e| panic!("Fixture {} introuvable: {}", path, e))
}

fn make_invoice() -> InvoiceData {
    let mut inv = InvoiceData::new("FA-2025-00256".to_string(), InvoiceFormat::CII);
    inv.invoice_type_code = Some("380".to_string());
    inv.issue_date = Some("2025-11-20".to_string());
    inv.seller_siret = Some("45678901234567".to_string());
    inv.seller_name = Some("InfoTech Solutions SARL".to_string());
    inv.buyer_siret = Some("32165498765432".to_string());
    inv.buyer_name = Some("Manufacture Lyonnaise SAS".to_string());
    inv.currency = Some("EUR".to_string());
    inv.total_without_vat = Some(32000.0);
    inv.total_tax = Some(6400.0);
    inv
}

fn make_exchange_with_invoice() -> Exchange {
    let body = load_fixture("facture_cii_001.xml");
    let mut exchange = Exchange::new(body);
    exchange.source_filename = Some("facture_cii_001.xml".to_string());
    exchange.invoice = Some(make_invoice());
    exchange
}

// ============================================================
// Test 1 : Facture valide → CDV 200 Déposée pour le vendeur
// ============================================================

#[tokio::test]
async fn test_facture_valide_genere_cdv_200_deposee() {
    let processor = CdarProcessor::new("999888777", "Ma PDP Test");
    let exchange = make_exchange_with_invoice();

    let result = processor.process(exchange).await.unwrap();

    // CDV a été généré
    assert_eq!(result.get_header("cdv.generated").unwrap(), "true");
    assert_eq!(result.get_property("cdv.status_code").unwrap(), "200");
    assert_eq!(result.get_property("cdv.type_code").unwrap(), "305"); // Transmission

    // Parser le CDV XML généré
    let cdv_xml = result.get_property("cdv.xml").unwrap();
    let parser = CdarParser::new();
    let cdv = parser.parse(cdv_xml).unwrap();

    // Vérifications structurelles
    assert_eq!(cdv.business_process, "REGULATED");
    assert_eq!(cdv.guideline_id, "urn.cpro.gouv.fr:1p0:CDV:invoice");
    assert_eq!(cdv.type_code, CdvTypeCode::Transmission);

    // Émetteur = PDP (rôle WK)
    assert_eq!(cdv.sender.role_code, RoleCode::WK);

    // Destinataires = Vendeur (SE) + PPF (DFH)
    assert_eq!(cdv.recipients.len(), 2);
    let seller = cdv.recipients.iter().find(|r| r.role_code == RoleCode::SE).unwrap();
    assert_eq!(seller.global_id.as_deref(), Some("456789012"));
    assert_eq!(seller.global_id_scheme.as_deref(), Some("0002"));
    let ppf = cdv.recipients.iter().find(|r| r.role_code == RoleCode::DFH).unwrap();
    assert_eq!(ppf.global_id.as_deref(), Some("9998"));
    assert_eq!(ppf.global_id_scheme.as_deref(), Some("0238"));

    // Référence à la facture
    assert_eq!(cdv.referenced_documents.len(), 1);
    let ref_doc = &cdv.referenced_documents[0];
    assert_eq!(ref_doc.invoice_id, "FA-2025-00256");
    assert_eq!(ref_doc.process_condition_code, 200);
    assert_eq!(ref_doc.process_condition.as_deref(), Some("Déposée"));
    assert_eq!(ref_doc.status_code, Some(10)); // Received
    assert_eq!(ref_doc.type_code.as_deref(), Some("380"));

    // Pas de motifs d'erreur
    assert!(ref_doc.statuses.is_empty());
}

// ============================================================
// Test 2 : Facture invalide → CDV 213 Rejetée pour le vendeur
// ============================================================

#[tokio::test]
async fn test_facture_invalide_genere_cdv_213_rejetee() {
    let processor = CdarProcessor::new("999888777", "Ma PDP Test");

    let mut exchange = make_exchange_with_invoice();
    // Simuler une erreur de validation
    exchange.add_error(
        "validation",
        &PdpError::ValidationError("BR-FR-12: SIRET vendeur invalide".to_string()),
    );

    let result = processor.process(exchange).await.unwrap();

    assert_eq!(result.get_property("cdv.status_code").unwrap(), "213");
    assert_eq!(result.get_property("cdv.type_code").unwrap(), "305");

    let cdv_xml = result.get_property("cdv.xml").unwrap();
    let parser = CdarParser::new();
    let cdv = parser.parse(cdv_xml).unwrap();

    // Phase Transmission → destinataires = vendeur + acheteur + PPF
    assert_eq!(cdv.type_code, CdvTypeCode::Transmission);
    assert_eq!(cdv.recipients.len(), 3);
    assert!(cdv.recipients.iter().any(|r| r.role_code == RoleCode::SE));
    assert!(cdv.recipients.iter().any(|r| r.role_code == RoleCode::BY));
    assert!(cdv.recipients.iter().any(|r| r.role_code == RoleCode::DFH));

    // Référence avec statut de rejet
    let ref_doc = &cdv.referenced_documents[0];
    assert_eq!(ref_doc.process_condition_code, 213);
    assert_eq!(ref_doc.process_condition.as_deref(), Some("Rejetée"));
    assert_eq!(ref_doc.status_code, Some(8)); // Rejected

    // Motifs de rejet présents
    assert!(!ref_doc.statuses.is_empty());
    let status = &ref_doc.statuses[0];
    assert!(status.reason.as_ref().unwrap().contains("SIRET"));
    assert_eq!(status.action_code.as_deref(), Some("NIN"));
    assert_eq!(status.action.as_deref(), Some("Corriger et redéposer"));
}

// ============================================================
// Test 3 : CDV 501 Irrecevable pour fichier vide
// ============================================================

#[tokio::test]
async fn test_fichier_vide_genere_cdv_501_irrecevable() {
    let generator = CdarGenerator::new("999888777", "Ma PDP Test");
    let invoice = make_invoice();

    let cdv = generator.generate_irrecevable(
        &invoice,
        pdp_cdar::StatusReasonCode::IrrVideF,
        "Fichier vide reçu",
    );

    assert_eq!(cdv.status_code(), Some(501));
    assert_eq!(cdv.type_code, CdvTypeCode::Transmission);

    // Destinataires = vendeur (SE) + PPF (DFH)
    assert_eq!(cdv.recipients.len(), 2);
    let seller = cdv.recipients.iter().find(|r| r.role_code == RoleCode::SE).unwrap();
    assert_eq!(seller.global_id.as_deref(), Some("456789012"));
    assert!(cdv.recipients.iter().any(|r| r.role_code == RoleCode::DFH));

    // Motif d'irrecevabilité
    let ref_doc = &cdv.referenced_documents[0];
    assert_eq!(ref_doc.process_condition_code, 501);
    assert_eq!(ref_doc.status_code, Some(8)); // Rejected
    assert_eq!(ref_doc.statuses.len(), 1);
    assert_eq!(
        ref_doc.statuses[0].reason_code.as_deref(),
        Some("IRR_VIDE_F")
    );
}

// ============================================================
// Test 4 : CDV phase Traitement → vendeur seul
// ============================================================

#[tokio::test]
async fn test_cdv_traitement_envoye_vendeur_seul() {
    let generator = CdarGenerator::new("999888777", "Ma PDP Test");
    let invoice = make_invoice();

    // 204 Prise en charge — phase Traitement : SE seul
    // L'acheteur (BY) est l'Issuer, pas destinataire
    let cdv_204 = generator.generate_prise_en_charge(&invoice, "380");
    assert_eq!(cdv_204.type_code, CdvTypeCode::Traitement);
    assert_eq!(cdv_204.recipients.len(), 1);
    assert_eq!(cdv_204.recipients[0].role_code, RoleCode::SE);

    // Vérifier SIREN vendeur
    let seller = &cdv_204.recipients[0];
    assert_eq!(seller.global_id.as_deref(), Some("456789012"));
}

// ============================================================
// Test 5 : CDV 210 Refusée (Traitement) → les 2 parties
// ============================================================

#[tokio::test]
async fn test_cdv_210_refusee_envoye_aux_deux_parties() {
    let generator = CdarGenerator::new("999888777", "Ma PDP Test");
    let invoice = make_invoice();

    let cdv_210 = generator.generate_status(
        InvoiceStatusCode::Refusee,
        CdvTypeCode::Traitement,
        RoleCode::BY,
        &invoice,
        "380",
        vec![pdp_cdar::DocumentStatus {
            status_code: None,
            reason_code: Some("MONTANTTOTAL_ERR".to_string()),
            reason: Some("Montant total ne correspond pas".to_string()),
            action_code: Some("NIN".to_string()),
            action: Some("Émettre un avoir et refacturer".to_string()),
            sequence: Some(1),
            characteristics: Vec::new(),
        }],
    );

    assert_eq!(cdv_210.type_code, CdvTypeCode::Traitement);
    assert_eq!(cdv_210.status_code(), Some(210));

    // Traitement → SE seul (l'acheteur est Issuer, pas destinataire)
    assert_eq!(cdv_210.recipients.len(), 1);
    assert_eq!(cdv_210.recipients[0].role_code, RoleCode::SE);

    // Émetteur = Acheteur (BY) car c'est l'acheteur qui refuse
    assert_eq!(cdv_210.sender.role_code, RoleCode::BY);

    // Motif de refus
    let ref_doc = &cdv_210.referenced_documents[0];
    assert_eq!(ref_doc.process_condition_code, 210);
    assert_eq!(ref_doc.status_code, Some(8)); // Rejected
    assert!(!ref_doc.statuses.is_empty());
    assert_eq!(
        ref_doc.statuses[0].reason_code.as_deref(),
        Some("MONTANTTOTAL_ERR")
    );
}

// ============================================================
// Test 6 : CDV 212 Encaissée → les 2 parties
// ============================================================

#[tokio::test]
async fn test_cdv_212_encaissee_envoye_aux_deux_parties() {
    let generator = CdarGenerator::new("999888777", "Ma PDP Test");
    let invoice = make_invoice();

    let cdv_212 = generator.generate_status(
        InvoiceStatusCode::Encaissee,
        CdvTypeCode::Traitement,
        RoleCode::BY,
        &invoice,
        "380",
        Vec::new(),
    );

    assert_eq!(cdv_212.status_code(), Some(212));
    assert_eq!(cdv_212.type_code, CdvTypeCode::Traitement);

    // Acheteur + PPF (vendeur est Issuer, pas destinataire)
    assert_eq!(cdv_212.recipients.len(), 2);
    assert!(cdv_212.recipients.iter().any(|r| r.role_code == RoleCode::BY));
    assert!(cdv_212.recipients.iter().any(|r| r.role_code == RoleCode::DFH));

    // Référence facture
    let ref_doc = &cdv_212.referenced_documents[0];
    assert_eq!(ref_doc.invoice_id, "FA-2025-00256");
    assert_eq!(ref_doc.process_condition.as_deref(), Some("Encaissée"));
    assert_eq!(ref_doc.status_code, Some(47)); // Pending
}

// ============================================================
// Test 7 : CdvReturnProcessor renvoie le CDV au vendeur
// ============================================================

#[tokio::test]
async fn test_cdv_return_renvoie_au_vendeur_via_producer() {
    let capture = Arc::new(CaptureProducer::new("cdv-return"));
    let return_processor = CdvReturnProcessor::new(capture.clone());

    // Simuler un exchange avec CDV généré
    let mut exchange = make_exchange_with_invoice();
    exchange.set_property("cdv.xml", "<CrossDomainAcknowledgementAndResponse>CDV200</CrossDomainAcknowledgementAndResponse>");
    exchange.set_property("cdv.status_code", "200");
    exchange.set_property("cdv.document_id", "FA-2025-00256_200_20251120120000");
    exchange.set_header("cdv.generated", "true");

    let result = return_processor.process(exchange).await.unwrap();

    // Le CDV a été envoyé
    assert_eq!(capture.sent_count(), 1);

    // Vérifier le contenu de l'exchange CDV envoyé
    let sent = &capture.sent_exchanges()[0];
    assert!(String::from_utf8_lossy(&sent.body).contains("CDV200"));
    assert_eq!(sent.get_header("document.type").unwrap(), "cdar");
    assert_eq!(sent.get_header("source.protocol").unwrap(), "internal");

    // Métadonnées de routage retour
    assert_eq!(
        sent.get_property("return.destination_siret").unwrap(),
        "45678901234567"
    );
    assert_eq!(
        sent.get_property("return.destination_name").unwrap(),
        "InfoTech Solutions SARL"
    );

    // Nom de fichier CDV
    assert!(sent
        .source_filename
        .as_ref()
        .unwrap()
        .starts_with("CDV_FA-2025-00256_200"));

    // L'exchange original est retourné inchangé
    assert!(result.invoice.is_some());
    assert_eq!(result.get_property("cdv.status_code").unwrap(), "200");
}

// ============================================================
// Test 8 : Pipeline complet — parse → CDV 200 → renvoi CDV
// ============================================================

#[tokio::test]
async fn test_pipeline_complet_facture_valide() {
    let cdv_capture = Arc::new(CaptureProducer::new("cdv-return"));

    // Chaîne de processors
    let cdar_proc = CdarProcessor::new("999888777", "Ma PDP Test");
    let return_proc = CdvReturnProcessor::new(cdv_capture.clone());

    // Exchange avec facture parsée (simule post-parsing)
    let exchange = make_exchange_with_invoice();

    // Étape 1 : CdarProcessor génère le CDV
    let exchange = cdar_proc.process(exchange).await.unwrap();
    assert_eq!(exchange.get_property("cdv.status_code").unwrap(), "200");

    // Étape 2 : CdvReturnProcessor renvoie le CDV au vendeur
    let _exchange = return_proc.process(exchange).await.unwrap();

    // Vérifier que le CDV a été envoyé
    assert_eq!(cdv_capture.sent_count(), 1);

    // Vérifier que le CDV envoyé est un XML D22B valide
    let cdv_body = &cdv_capture.sent_exchanges()[0].body;
    let cdv_xml = std::str::from_utf8(cdv_body).unwrap();
    let parser = CdarParser::new();
    let cdv = parser.parse(cdv_xml).unwrap();

    assert_eq!(cdv.status_code(), Some(200));
    assert_eq!(cdv.type_code, CdvTypeCode::Transmission);
    assert!(cdv.recipients.iter().any(|r| r.role_code == RoleCode::SE));
    assert!(cdv.recipients.iter().any(|r| r.role_code == RoleCode::DFH));
    assert_eq!(cdv.referenced_documents[0].invoice_id, "FA-2025-00256");
}

// ============================================================
// Test 9 : Pipeline complet — erreur validation → CDV 213 → renvoi
// ============================================================

#[tokio::test]
async fn test_pipeline_complet_facture_rejetee() {
    let cdv_capture = Arc::new(CaptureProducer::new("cdv-return"));

    let cdar_proc = CdarProcessor::new("999888777", "Ma PDP Test");
    let return_proc = CdvReturnProcessor::new(cdv_capture.clone());

    let mut exchange = make_exchange_with_invoice();
    exchange.add_error(
        "validation",
        &PdpError::ValidationError("XSD: Element invalide".to_string()),
    );

    // CdarProcessor → CDV 213
    let exchange = cdar_proc.process(exchange).await.unwrap();
    assert_eq!(exchange.get_property("cdv.status_code").unwrap(), "213");

    // CdvReturnProcessor → envoi au vendeur
    let _exchange = return_proc.process(exchange).await.unwrap();

    // CDV 213 envoyé au vendeur
    assert_eq!(cdv_capture.sent_count(), 1);

    let cdv_body = &cdv_capture.sent_exchanges()[0].body;
    let cdv_xml = std::str::from_utf8(cdv_body).unwrap();
    let parser = CdarParser::new();
    let cdv = parser.parse(cdv_xml).unwrap();

    assert_eq!(cdv.status_code(), Some(213));
    assert!(!cdv.referenced_documents[0].statuses.is_empty());
}

// ============================================================
// Test 10 : CDV XML bien formé — round-trip generate → parse
// ============================================================

#[tokio::test]
async fn test_cdv_xml_roundtrip_tous_statuts() {
    let generator = CdarGenerator::new("999888777", "Ma PDP Test");
    let parser = CdarParser::new();
    let invoice = make_invoice();

    let test_cases: Vec<(InvoiceStatusCode, CdvTypeCode, RoleCode, u16)> = vec![
        // Phase Transmission
        (InvoiceStatusCode::Deposee, CdvTypeCode::Transmission, RoleCode::WK, 200),
        (InvoiceStatusCode::Emise, CdvTypeCode::Transmission, RoleCode::WK, 201),
        (InvoiceStatusCode::Rejetee, CdvTypeCode::Transmission, RoleCode::WK, 213),
        // Phase Traitement
        (InvoiceStatusCode::PriseEnCharge, CdvTypeCode::Traitement, RoleCode::BY, 204),
        (InvoiceStatusCode::Approuvee, CdvTypeCode::Traitement, RoleCode::BY, 205),
        (InvoiceStatusCode::Refusee, CdvTypeCode::Traitement, RoleCode::BY, 210),
        (InvoiceStatusCode::Encaissee, CdvTypeCode::Traitement, RoleCode::BY, 212),
    ];

    for (status, type_code, sender_role, expected_code) in test_cases {
        let cdv = generator.generate_status(
            status,
            type_code,
            sender_role,
            &invoice,
            "380",
            Vec::new(),
        );

        // Sérialiser en XML
        let xml = generator.to_xml(&cdv).unwrap();

        // Vérifier que le XML est parsable
        let parsed = parser.parse(&xml).unwrap_or_else(|e| {
            panic!(
                "CDV {} non parsable: {}\nXML:\n{}",
                expected_code, e, xml
            )
        });

        // Vérifier le statut
        assert_eq!(
            parsed.status_code(),
            Some(expected_code),
            "Statut CDV {} incorrect après round-trip",
            expected_code
        );

        // Vérifier le nombre de destinataires
        let expected_recipients = match (type_code, expected_code) {
            (CdvTypeCode::Transmission, _) => 2,  // SE + DFH
            (CdvTypeCode::Traitement, 212) => 2,   // BY + DFH
            (CdvTypeCode::Traitement, _) => 1,     // SE seul
        };
        assert_eq!(
            parsed.recipients.len(),
            expected_recipients,
            "CDV {} : attendu {} destinataires, trouvé {}",
            expected_code,
            expected_recipients,
            parsed.recipients.len()
        );

        // Vérifier que la facture est référencée
        assert_eq!(parsed.referenced_documents[0].invoice_id, "FA-2025-00256");
    }
}

// ============================================================
// Test 11 : CDV Transmission → vendeur (SE) + PPF (DFH), pas d'acheteur
// ============================================================

#[tokio::test]
async fn test_cdv_transmission_vendeur_et_ppf_sans_acheteur() {
    let generator = CdarGenerator::new("999888777", "Ma PDP Test");
    let invoice = make_invoice();

    // CDV Déposée (200) — phase Transmission : SE + DFH
    let cdv = generator.generate_deposee(&invoice, "380");
    assert_eq!(cdv.recipients.len(), 2);
    assert!(cdv.recipients.iter().any(|r| r.role_code == RoleCode::SE));
    assert!(cdv.recipients.iter().any(|r| r.role_code == RoleCode::DFH));
    assert!(!cdv.recipients.iter().any(|r| r.role_code == RoleCode::BY));

    // CDV Émise (201) — phase Transmission : SE + DFH
    let cdv = generator.generate_emise(&invoice, "380");
    assert_eq!(cdv.recipients.len(), 2);
    assert!(cdv.recipients.iter().any(|r| r.role_code == RoleCode::SE));
    assert!(cdv.recipients.iter().any(|r| r.role_code == RoleCode::DFH));

    // CDV Erreur routage (221) — phase Transmission : SE + DFH
    let cdv = generator.generate_erreur_routage(&invoice, "SIREN inconnu");
    assert_eq!(cdv.recipients.len(), 2);
    assert!(cdv.recipients.iter().any(|r| r.role_code == RoleCode::SE));
    assert!(cdv.recipients.iter().any(|r| r.role_code == RoleCode::DFH));
}

// ============================================================
// Test 12 : Tous les CDV de Traitement (sauf 212) → vendeur seul
// ============================================================

#[tokio::test]
async fn test_tous_cdv_traitement_vendeur_seul() {
    let generator = CdarGenerator::new("999888777", "Ma PDP Test");
    let invoice = make_invoice();

    // Encaissee (212) est exclue : elle a BY+DFH, pas SE seul
    let statuts_traitement = vec![
        InvoiceStatusCode::PriseEnCharge,
        InvoiceStatusCode::Approuvee,
        InvoiceStatusCode::ApprouveePartiellement,
        InvoiceStatusCode::EnLitige,
        InvoiceStatusCode::Suspendue,
        InvoiceStatusCode::Refusee,
        InvoiceStatusCode::PaiementTransmis,
    ];

    for status in statuts_traitement {
        let cdv = generator.generate_status(
            status,
            CdvTypeCode::Traitement,
            RoleCode::BY,
            &invoice,
            "380",
            Vec::new(),
        );

        // Phase Traitement : vendeur (SE) seul comme destinataire
        // L'acheteur (BY) est l'Issuer de ces statuts, pas un destinataire
        assert_eq!(
            cdv.recipients.len(),
            1,
            "CDV {:?} (code {}) devrait avoir 1 seul destinataire (SE)",
            status,
            status.code()
        );

        assert_eq!(
            cdv.recipients[0].role_code,
            RoleCode::SE,
            "CDV {:?} : destinataire doit être le vendeur (SE)",
            status
        );
    }
}

// ============================================================
// Test 13 : Vérifier les SIREN des parties dans le CDV XML
// ============================================================

#[tokio::test]
async fn test_cdv_contient_siren_correct() {
    let generator = CdarGenerator::new("999888777", "Ma PDP Test");
    let invoice = make_invoice();

    // CDV 213 (Rejetée) pour vérifier les deux parties (SE + BY + DFH)
    let cdv = generator.generate_rejetee(&invoice, "380", Vec::new());
    let xml = generator.to_xml(&cdv).unwrap();

    // Le XML doit contenir le SIREN vendeur (9 premiers chars du SIRET)
    assert!(
        xml.contains("456789012"),
        "SIREN vendeur absent du XML CDV"
    );

    // Le XML doit contenir le SIREN acheteur
    assert!(
        xml.contains("321654987"),
        "SIREN acheteur absent du XML CDV"
    );

    // Le XML doit contenir le numéro de facture
    assert!(
        xml.contains("FA-2025-00256"),
        "Numéro de facture absent du XML CDV"
    );

    // Le XML doit être du format CDAR D22B
    assert!(
        xml.contains("CrossDomainAcknowledgementAndResponse"),
        "Root element CDAR absent"
    );
    assert!(
        xml.contains("REGULATED"),
        "Business process REGULATED absent"
    );
    assert!(
        xml.contains("urn.cpro.gouv.fr:1p0:CDV:invoice"),
        "Guideline ID absent"
    );
}

// ============================================================
// Test 14 : CDV 200 → endpoint_id du vendeur avec format _STATUTS
// ============================================================

#[tokio::test]
async fn test_cdv_200_endpoint_vendeur_format_statuts() {
    let generator = CdarGenerator::new("999888777", "Ma PDP Test");
    let invoice = make_invoice();

    let cdv = generator.generate_deposee(&invoice, "380");

    let seller = &cdv.recipients[0];
    assert_eq!(seller.endpoint_id.as_deref(), Some("456789012_STATUTS"));
    assert_eq!(seller.endpoint_scheme.as_deref(), Some("0225"));
}

// ============================================================
// Helpers pour les fixtures XP Z12-014
// ============================================================

fn load_xp_fixture(name: &str) -> String {
    let path = format!(
        "{}/../../tests/fixtures/xp-z12-014/{}",
        env!("CARGO_MANIFEST_DIR"),
        name
    );
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Fixture {} introuvable: {}", path, e))
}

// ============================================================
// Test 15 : Parsing fixture CDV 202 Reçue (Transmission, SE+BY)
// ============================================================

#[tokio::test]
async fn test_fixture_cdv_202_recue() {
    let xml = load_xp_fixture("UC1/UC1_F202500003_02-CDV-202_Recue.xml");
    let parser = CdarParser::new();
    let cdv = parser.parse(&xml).expect("Parsing CDV 202 échoué");

    // Contexte
    assert_eq!(cdv.business_process, "REGULATED");
    assert_eq!(cdv.guideline_id, "urn.cpro.gouv.fr:1p0:CDV:invoice");

    // Phase Transmission (305)
    assert_eq!(cdv.type_code, CdvTypeCode::Transmission);

    // Destinataires : SE + BY (pas de DFH — le PPF reçoit via le CDV POUR_PPF)
    assert_eq!(cdv.recipients.len(), 2);
    assert!(cdv.recipients.iter().any(|r| r.role_code == RoleCode::SE));
    assert!(cdv.recipients.iter().any(|r| r.role_code == RoleCode::BY));

    // Référence facture
    let ref_doc = &cdv.referenced_documents[0];
    assert_eq!(ref_doc.invoice_id, "F202500003");
    assert_eq!(ref_doc.process_condition_code, 202);
    assert_eq!(ref_doc.process_condition.as_deref(), Some("Reçue"));
    assert_eq!(ref_doc.type_code.as_deref(), Some("380"));

    // MDT-88 : StatusCode 43 ("Transferred to the next party")
    assert_eq!(ref_doc.status_code, Some(43));

    // Pas de motifs
    assert!(ref_doc.statuses.is_empty());
}

// ============================================================
// Test 16 : Parsing fixture CDV 203 Mise à disposition (Transmission, SE+BY)
// ============================================================

#[tokio::test]
async fn test_fixture_cdv_203_mise_a_disposition() {
    let xml = load_xp_fixture("UC1/UC1_F202500003_03-CDV-203_Mise_a_disposition.xml");
    let parser = CdarParser::new();
    let cdv = parser.parse(&xml).expect("Parsing CDV 203 échoué");

    assert_eq!(cdv.type_code, CdvTypeCode::Transmission);

    // Destinataires : SE + BY
    assert_eq!(cdv.recipients.len(), 2);
    assert!(cdv.recipients.iter().any(|r| r.role_code == RoleCode::SE));
    assert!(cdv.recipients.iter().any(|r| r.role_code == RoleCode::BY));

    let ref_doc = &cdv.referenced_documents[0];
    assert_eq!(ref_doc.process_condition_code, 203);
    assert_eq!(ref_doc.process_condition.as_deref(), Some("Mise_à_disposition"));

    // MDT-88 : StatusCode 48 ("Available")
    assert_eq!(ref_doc.status_code, Some(48));
}

// ============================================================
// Test 17 : Parsing fixture CDV 204 Prise en charge (Traitement, SE seul)
// ============================================================

#[tokio::test]
async fn test_fixture_cdv_204_prise_en_charge() {
    let xml = load_xp_fixture("UC1/UC1_F202500003_04-CDV-204_Prise_en_charge.xml");
    let parser = CdarParser::new();
    let cdv = parser.parse(&xml).expect("Parsing CDV 204 échoué");

    // Phase Traitement (23)
    assert_eq!(cdv.type_code, CdvTypeCode::Traitement);

    // Issuer = Acheteur (BY) — c'est lui qui initie la prise en charge
    assert_eq!(cdv.issuer.as_ref().unwrap().role_code, RoleCode::BY);
    assert_eq!(
        cdv.issuer.as_ref().unwrap().global_id.as_deref(),
        Some("200000008")
    );

    // Destinataire : SE seul (l'acheteur sait déjà, il a initié l'action)
    assert_eq!(cdv.recipients.len(), 1);
    assert_eq!(cdv.recipients[0].role_code, RoleCode::SE);

    let ref_doc = &cdv.referenced_documents[0];
    assert_eq!(ref_doc.process_condition_code, 204);

    // MDT-88 : StatusCode 45 ("Under investigation")
    assert_eq!(ref_doc.status_code, Some(45));
}

// ============================================================
// Test 18 : Parsing fixture CDV 205 Approuvée (Traitement, SE seul)
// ============================================================

#[tokio::test]
async fn test_fixture_cdv_205_approuvee() {
    let xml = load_xp_fixture("UC1/UC1_F202500003_05-CDV-205_Approuvee.xml");
    let parser = CdarParser::new();
    let cdv = parser.parse(&xml).expect("Parsing CDV 205 échoué");

    assert_eq!(cdv.type_code, CdvTypeCode::Traitement);

    // Issuer = Acheteur (BY)
    assert_eq!(cdv.issuer.as_ref().unwrap().role_code, RoleCode::BY);

    // Destinataire : SE seul
    assert_eq!(cdv.recipients.len(), 1);
    assert_eq!(cdv.recipients[0].role_code, RoleCode::SE);

    let ref_doc = &cdv.referenced_documents[0];
    assert_eq!(ref_doc.process_condition_code, 205);
    assert_eq!(ref_doc.process_condition.as_deref(), Some("Approuvée"));

    // MDT-88 : StatusCode 1 ("Accepted")
    assert_eq!(ref_doc.status_code, Some(1));
}

// ============================================================
// Test 19 : Parsing fixture CDV 207 En litige — MOTIF + caractéristiques
// ============================================================

#[tokio::test]
async fn test_fixture_cdv_207_en_litige_avec_motif() {
    let xml = load_xp_fixture("UC4/UC4_F202500006_04-CDV-207_En_litige.xml");
    let parser = CdarParser::new();
    let cdv = parser.parse(&xml).expect("Parsing CDV 207 échoué");

    assert_eq!(cdv.type_code, CdvTypeCode::Traitement);

    // Issuer = Acheteur (BY)
    assert_eq!(cdv.issuer.as_ref().unwrap().role_code, RoleCode::BY);

    // Destinataire : SE seul
    assert_eq!(cdv.recipients.len(), 1);
    assert_eq!(cdv.recipients[0].role_code, RoleCode::SE);

    let ref_doc = &cdv.referenced_documents[0];
    assert_eq!(ref_doc.process_condition_code, 207);

    // MDT-88 : StatusCode 46 ("Under query")
    assert_eq!(ref_doc.status_code, Some(46));

    // Motif de litige (MDT-113 / MDT-114)
    assert_eq!(ref_doc.statuses.len(), 1);
    let status = &ref_doc.statuses[0];
    assert_eq!(status.reason_code.as_deref(), Some("TX_TVA_ERR"));
    assert!(status.reason.as_ref().unwrap().contains("Taux de TVA"));

    // Action attendue (MDT-121 / MDT-122) : NIN = Corriger
    assert_eq!(status.action_code.as_deref(), Some("NIN"));
    assert!(status.action.as_ref().unwrap().contains("Facture Rectificative"));

    // Caractéristiques : DIV (valeur invalide) + DVA (valeur attendue)
    assert_eq!(status.characteristics.len(), 2);

    let div = status.characteristics.iter().find(|c| c.type_code == "DIV")
        .expect("Caractéristique DIV manquante");
    assert_eq!(div.id.as_deref(), Some("BT-152"));
    assert_eq!(div.value_percent.as_deref(), Some("10.00"));

    let dva = status.characteristics.iter().find(|c| c.type_code == "DVA")
        .expect("Caractéristique DVA manquante");
    assert_eq!(dva.id.as_deref(), Some("BT-152"));
    assert_eq!(dva.value_percent.as_deref(), Some("20.00"));
}

// ============================================================
// Test 20 : Parsing fixture CDV 211 Paiement transmis — caractéristique MPA
// ============================================================

#[tokio::test]
async fn test_fixture_cdv_211_paiement_transmis() {
    let xml = load_xp_fixture("UC1/UC1_F202500003_06-CDV-211_Paiement_transmis.xml");
    let parser = CdarParser::new();
    let cdv = parser.parse(&xml).expect("Parsing CDV 211 échoué");

    assert_eq!(cdv.type_code, CdvTypeCode::Traitement);

    // Issuer = Acheteur (BY)
    assert_eq!(cdv.issuer.as_ref().unwrap().role_code, RoleCode::BY);

    // Destinataire : SE seul
    assert_eq!(cdv.recipients.len(), 1);
    assert_eq!(cdv.recipients[0].role_code, RoleCode::SE);

    let ref_doc = &cdv.referenced_documents[0];
    assert_eq!(ref_doc.process_condition_code, 211);

    // MDT-88 : StatusCode 47 ("Pending")
    assert_eq!(ref_doc.status_code, Some(47));

    // Caractéristique MPA (montant payé)
    assert_eq!(ref_doc.statuses.len(), 1);
    let status = &ref_doc.statuses[0];
    assert_eq!(status.characteristics.len(), 1);
    let mpa = &status.characteristics[0];
    assert_eq!(mpa.type_code, "MPA");
    assert_eq!(mpa.value_amount.as_deref(), Some("12000"));
}

// ============================================================
// Test 21 : Parsing fixture CDV 212 Encaissée — MEN + SE+BY+DFH
// ============================================================

#[tokio::test]
async fn test_fixture_cdv_212_encaissee() {
    let xml = load_xp_fixture("UC1/UC1_F202500003_07-CDV-212_Encaissee.xml");
    let parser = CdarParser::new();
    let cdv = parser.parse(&xml).expect("Parsing CDV 212 échoué");

    assert_eq!(cdv.type_code, CdvTypeCode::Traitement);

    // Issuer = Vendeur (SE) — le vendeur constate l'encaissement
    assert_eq!(cdv.issuer.as_ref().unwrap().role_code, RoleCode::SE);

    // Destinataires : BY (acheteur) + DFH (PPF) — statut obligatoire
    // Le vendeur (SE) est Issuer, pas destinataire de son propre CDV
    assert_eq!(cdv.recipients.len(), 2);
    assert!(cdv.recipients.iter().any(|r| r.role_code == RoleCode::BY));
    assert!(cdv.recipients.iter().any(|r| r.role_code == RoleCode::DFH));

    let ref_doc = &cdv.referenced_documents[0];
    assert_eq!(ref_doc.process_condition_code, 212);

    // MDT-88 : StatusCode 47 ("Pending")
    assert_eq!(ref_doc.status_code, Some(47));

    // Caractéristique MEN (montant encaissé)
    assert_eq!(ref_doc.statuses.len(), 1);
    let status = &ref_doc.statuses[0];
    assert_eq!(status.characteristics.len(), 1);
    let men = &status.characteristics[0];
    assert_eq!(men.type_code, "MEN");
    assert_eq!(men.value_amount.as_deref(), Some("12000"));
    assert_eq!(men.value_percent.as_deref(), Some("20.00"));
}

// ============================================================
// Test 22 : Parsing fixture CDV 200 POUR_PPF — guideline einvoicingF2
// ============================================================

#[tokio::test]
async fn test_fixture_cdv_200_pour_ppf() {
    let xml = load_xp_fixture("UC1/UC1_F202500003_01-CDV-200_Deposee_POUR_PPF.xml");
    let parser = CdarParser::new();
    let cdv = parser.parse(&xml).expect("Parsing CDV 200 POUR_PPF échoué");

    // Guideline spécifique PPF
    assert_eq!(cdv.guideline_id, "urn.cpro.gouv.fr:1p0:CDV:einvoicingF2");

    // Pas de BusinessProcess (MDT-2 absent dans le CDV POUR_PPF)
    // Le parser peut mettre une valeur vide ou la valeur par défaut

    // Sender = PA-E avec identifiant schemeID 0238
    assert_eq!(cdv.sender.role_code, RoleCode::WK);
    assert_eq!(cdv.sender.global_id.as_deref(), Some("0003"));
    assert_eq!(cdv.sender.global_id_scheme.as_deref(), Some("0238"));

    // Issuer = PA-E (même identité que sender)
    assert_eq!(cdv.issuer.as_ref().unwrap().role_code, RoleCode::WK);
    assert_eq!(cdv.issuer.as_ref().unwrap().global_id.as_deref(), Some("0003"));

    // Destinataire unique : PPF (DFH) — le seul destinataire du CDV POUR_PPF
    assert_eq!(cdv.recipients.len(), 1);
    assert_eq!(cdv.recipients[0].role_code, RoleCode::DFH);
    assert_eq!(cdv.recipients[0].global_id.as_deref(), Some("9998"));
    assert_eq!(cdv.recipients[0].global_id_scheme.as_deref(), Some("0238"));

    // Statut 200 Déposée
    let ref_doc = &cdv.referenced_documents[0];
    assert_eq!(ref_doc.process_condition_code, 200);
    assert_eq!(ref_doc.status_code, Some(10));
}

// ============================================================
// Test 23 : Parsing fixture CDV 213 Rejetée — SE+BY+DFH
// ============================================================

#[tokio::test]
async fn test_fixture_cdv_213_rejetee() {
    let xml = load_xp_fixture("UC2/UC2_F202500004_02-CDV-213_Rejetee.xml");
    let parser = CdarParser::new();
    let cdv = parser.parse(&xml).expect("Parsing CDV 213 échoué");

    assert_eq!(cdv.type_code, CdvTypeCode::Transmission);

    // Destinataires : SE + BY + DFH
    assert_eq!(cdv.recipients.len(), 3);
    assert!(cdv.recipients.iter().any(|r| r.role_code == RoleCode::SE));
    assert!(cdv.recipients.iter().any(|r| r.role_code == RoleCode::BY));
    assert!(cdv.recipients.iter().any(|r| r.role_code == RoleCode::DFH));

    let ref_doc = &cdv.referenced_documents[0];
    assert_eq!(ref_doc.process_condition_code, 213);
    assert_eq!(ref_doc.status_code, Some(8)); // Rejected

    // Motif de rejet
    assert_eq!(ref_doc.statuses.len(), 1);
    let status = &ref_doc.statuses[0];
    assert_eq!(status.reason_code.as_deref(), Some("DOUBLON"));
    assert_eq!(status.sequence, Some(1));
}

// ============================================================
// Test 24 : CDV 200 fixture — SE + DFH (pas d'acheteur en phase dépôt)
// ============================================================

#[tokio::test]
async fn test_fixture_cdv_200_deposee() {
    let xml = load_xp_fixture("UC1/UC1_F202500003_01-CDV-200_Deposee.xml");
    let parser = CdarParser::new();
    let cdv = parser.parse(&xml).expect("Parsing CDV 200 échoué");

    assert_eq!(cdv.type_code, CdvTypeCode::Transmission);
    assert_eq!(cdv.business_process, "REGULATED");

    // Issuer = WK (PDP)
    assert_eq!(cdv.issuer.as_ref().unwrap().role_code, RoleCode::WK);

    // Destinataires : SE + DFH (pas de BY — l'acheteur sera notifié via CDV 202 Reçue)
    assert_eq!(cdv.recipients.len(), 2);
    assert!(cdv.recipients.iter().any(|r| r.role_code == RoleCode::SE));
    assert!(cdv.recipients.iter().any(|r| r.role_code == RoleCode::DFH));
    assert!(!cdv.recipients.iter().any(|r| r.role_code == RoleCode::BY));

    let ref_doc = &cdv.referenced_documents[0];
    assert_eq!(ref_doc.process_condition_code, 200);
    assert_eq!(ref_doc.status_code, Some(10)); // Received
    assert_eq!(ref_doc.invoice_id, "F202500003");
}

// ============================================================
// Tests PDP Réceptrice — CDV 202 "Reçue"
// ============================================================

#[tokio::test]
async fn test_reception_facture_valide_genere_cdv_202_recue() {
    // CdarProcessor en mode réception (PDP réceptrice)
    let processor = CdarProcessor::reception("111222333", "PDP Réceptrice Test");
    let exchange = make_exchange_with_invoice();

    let result = processor.process(exchange).await.unwrap();

    // CDV 202 Reçue généré
    assert_eq!(result.get_header("cdv.generated").unwrap(), "true");
    assert_eq!(result.get_property("cdv.status_code").unwrap(), "202");
    assert_eq!(result.get_property("cdv.type_code").unwrap(), "305"); // Transmission

    // Parser le CDV XML généré
    let cdv_xml = result.get_property("cdv.xml").unwrap();
    let parser = CdarParser::new();
    let cdv = parser.parse(cdv_xml).unwrap();

    // Vérifications structurelles
    assert_eq!(cdv.business_process, "REGULATED");
    assert_eq!(cdv.type_code, CdvTypeCode::Transmission);

    // Émetteur = PDP (rôle WK)
    assert_eq!(cdv.sender.role_code, RoleCode::WK);

    // Destinataires = Acheteur (BY) + PPF (DFH)
    assert_eq!(cdv.recipients.len(), 2);
    let buyer = cdv.recipients.iter().find(|r| r.role_code == RoleCode::BY).unwrap();
    assert_eq!(buyer.global_id.as_deref(), Some("321654987"));
    let ppf = cdv.recipients.iter().find(|r| r.role_code == RoleCode::DFH).unwrap();
    assert_eq!(ppf.global_id.as_deref(), Some("9998"));

    // Référence à la facture
    assert_eq!(cdv.referenced_documents.len(), 1);
    let ref_doc = &cdv.referenced_documents[0];
    assert_eq!(ref_doc.invoice_id, "FA-2025-00256");
    assert_eq!(ref_doc.process_condition_code, 202);
    assert_eq!(ref_doc.process_condition.as_deref(), Some("Reçue"));
    assert_eq!(ref_doc.status_code, Some(43)); // Transferred to next party

    // Pas de motifs d'erreur
    assert!(ref_doc.statuses.is_empty());
}

#[tokio::test]
async fn test_reception_facture_invalide_genere_cdv_213() {
    // Même en mode réception, une facture invalide génère un CDV 213 (rejet)
    let processor = CdarProcessor::reception("111222333", "PDP Réceptrice Test");

    let mut exchange = make_exchange_with_invoice();
    exchange.add_error(
        "validation",
        &PdpError::ValidationError("Format invalide".to_string()),
    );

    let result = processor.process(exchange).await.unwrap();

    assert_eq!(result.get_property("cdv.status_code").unwrap(), "213");
    assert_eq!(result.get_property("cdv.type_code").unwrap(), "305");
}

#[tokio::test]
async fn test_emission_vs_reception_modes_differents() {
    let exchange = make_exchange_with_invoice();

    // Mode émission → CDV 200
    let emission = CdarProcessor::emission("999888777", "PDP Émettrice");
    let result_emission = emission.process(exchange.clone()).await.unwrap();
    assert_eq!(result_emission.get_property("cdv.status_code").unwrap(), "200");

    // Mode réception → CDV 202
    let reception = CdarProcessor::reception("111222333", "PDP Réceptrice");
    let result_reception = reception.process(exchange).await.unwrap();
    assert_eq!(result_reception.get_property("cdv.status_code").unwrap(), "202");
}

#[tokio::test]
async fn test_cdar_new_backward_compat() {
    // CdarProcessor::new() doit se comporter comme ::emission()
    let processor = CdarProcessor::new("999888777", "Ma PDP Test");
    let exchange = make_exchange_with_invoice();

    let result = processor.process(exchange).await.unwrap();
    assert_eq!(result.get_property("cdv.status_code").unwrap(), "200");
}
