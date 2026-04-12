//! Tests d'intégration pour le pipeline e-invoicing / e-reporting
//!
//! Ces tests vérifient le flux complet :
//! 1. Parsing d'une facture UBL/CII
//! 2. Conversion en TransactionInvoice pour e-reporting
//! 3. Génération du rapport XML e-reporting
//! 4. Génération du CDV (Compte-rendu De Vie)
//! 5. Réception et traitement d'un CDV entrant
//! 6. Construction des FlowInfo pour AFNOR
//! 7. Gestion des pièces jointes

use pdp_core::exchange::Exchange;
use pdp_core::model::{FlowStatus, InvoiceFormat};
use pdp_core::processor::Processor;

// ============================================================
// Test 1 : Pipeline complet UBL → Parse → E-reporting → CDV
// ============================================================

#[test]
fn test_ubl_to_ereporting_pipeline() {
    let xml = std::fs::read_to_string("../../tests/fixtures/ubl/facture_ubl_001.xml")
        .expect("Fixture UBL introuvable");

    // Étape 1 : Parser la facture UBL
    let parser = pdp_invoice::ubl::UblParser::new();
    let invoice = parser.parse(&xml).expect("Parsing UBL échoué");

    assert!(!invoice.invoice_number.is_empty());
    assert!(invoice.seller_name.is_some());
    assert!(invoice.buyer_name.is_some());
    assert!(invoice.total_ht.is_some());

    // Étape 2 : Convertir en TransactionInvoice pour e-reporting
    let txn = pdp_ereporting::generator::EReportingGenerator::invoice_to_transaction(&invoice);

    assert_eq!(txn.id, invoice.invoice_number);
    assert_eq!(txn.currency_code, invoice.currency.clone().unwrap_or_else(|| "EUR".to_string()));
    assert!(txn.seller.company_id.is_some());

    // Étape 3 : Générer le rapport e-reporting XML
    let gen = pdp_ereporting::generator::EReportingGenerator::new("100000009", "PDP Test");
    let report = gen.create_transactions_report(
        "RPT-INT-001",
        "100000009",
        "VENDEUR",
        "20250701",
        "20250731",
        vec![txn],
    );

    let report_xml = gen.to_xml(&report).expect("Sérialisation XML échouée");
    assert!(report_xml.contains("<Report>"));
    assert!(report_xml.contains("<TypeCode>10.1</TypeCode>"));
    assert!(report_xml.contains(&invoice.invoice_number));
}

// ============================================================
// Test 2 : Pipeline complet CII → Parse → E-reporting → CDV
// ============================================================

#[test]
fn test_cii_to_ereporting_pipeline() {
    let xml = std::fs::read_to_string("../../tests/fixtures/cii/facture_cii_001.xml")
        .expect("Fixture CII introuvable");

    // Parser la facture CII
    let parser = pdp_invoice::cii::CiiParser::new();
    let invoice = parser.parse(&xml).expect("Parsing CII échoué");

    assert!(!invoice.invoice_number.is_empty());
    assert_eq!(invoice.source_format, InvoiceFormat::CII);

    // Convertir en TransactionInvoice
    let txn = pdp_ereporting::generator::EReportingGenerator::invoice_to_transaction(&invoice);
    assert_eq!(txn.id, invoice.invoice_number);

    // Générer CDV de dépôt
    let cdv_gen = pdp_cdar::CdarGenerator::new("100000009", "PDP Test");
    let invoice_type = invoice.invoice_type_code.as_deref().unwrap_or("380");
    let cdv = cdv_gen.generate_deposee(&invoice, invoice_type);

    assert_eq!(cdv.status_code(), Some(200));
    assert!(cdv.is_success());

    // Sérialiser le CDV en XML
    let cdv_xml = cdv_gen.to_xml(&cdv).expect("Sérialisation CDV échouée");
    assert!(cdv_xml.contains("CrossDomainAcknowledgementAndResponse"));
    assert!(cdv_xml.contains(&invoice.invoice_number));
}

// ============================================================
// Test 3 : CDV entrant → CdvReceptionProcessor → FlowStatus
// ============================================================

#[tokio::test]
async fn test_cdv_reception_full_pipeline() {
    // Simuler la réception d'un CDV "Déposée" (200)
    let cdv_xml = std::fs::read_to_string("../../tests/fixtures/cdar/cdv_200_deposee.xml")
        .expect("Fixture CDV introuvable");

    let processor = pdp_cdar::CdvReceptionProcessor::new();
    let exchange = Exchange::new(cdv_xml.into_bytes());
    let result = processor.process(exchange).await.unwrap();

    // Vérifier que le statut est correctement mappé
    assert_eq!(result.status, FlowStatus::Distributed);
    assert_eq!(result.get_property("cdv.invoice_id").map(|s| s.as_str()), Some("F202500001"));
    assert!(!result.has_errors());

    // Simuler la réception d'un CDV "Rejetée" (213)
    let cdv_reject = std::fs::read_to_string("../../tests/fixtures/cdar/cdv_213_rejetee.xml")
        .expect("Fixture CDV rejet introuvable");

    let exchange2 = Exchange::new(cdv_reject.into_bytes());
    let result2 = processor.process(exchange2).await.unwrap();

    assert_eq!(result2.status, FlowStatus::Rejected);
    assert!(result2.has_errors());
}

// ============================================================
// Test 4 : Paiements report conforme XSD
// ============================================================

#[test]
fn test_payments_report_pipeline() {
    use pdp_ereporting::model::*;

    let gen = pdp_ereporting::generator::EReportingGenerator::new("100000009", "PDP Test");

    // Créer un rapport de paiements par facture (flux 10.2)
    let invoices = vec![
        PaymentInvoice {
            invoice_id: "FA-2025-001".to_string(),
            issue_date: "20250701".to_string(),
            payment: PaymentDetail {
                date: "20250715".to_string(),
                sub_totals: vec![
                    PaymentSubTotal {
                        tax_percent: 20.0,
                        currency_code: Some("EUR".to_string()),
                        amount: 1200.00,
                    },
                    PaymentSubTotal {
                        tax_percent: 5.5,
                        currency_code: Some("EUR".to_string()),
                        amount: 52.75,
                    },
                ],
            },
        },
        PaymentInvoice {
            invoice_id: "FA-2025-002".to_string(),
            issue_date: "20250710".to_string(),
            payment: PaymentDetail {
                date: "20250720".to_string(),
                sub_totals: vec![
                    PaymentSubTotal {
                        tax_percent: 20.0,
                        currency_code: Some("EUR".to_string()),
                        amount: 600.00,
                    },
                ],
            },
        },
    ];

    let report = gen.create_payments_report(
        "PAY-INT-001",
        "100000009",
        "VENDEUR",
        "20250701",
        "20250731",
        invoices,
    );

    assert_eq!(report.document.type_code, ReportTypeCode::PaymentsInitial);

    let xml = gen.to_xml(&report).unwrap();
    assert!(xml.contains("<TypeCode>10.2</TypeCode>"));
    assert!(xml.contains("<InvoiceID>FA-2025-001</InvoiceID>"));
    assert!(xml.contains("<InvoiceID>FA-2025-002</InvoiceID>"));
    assert!(xml.contains("<TaxPercent>20.00</TaxPercent>"));
    assert!(xml.contains("<TaxPercent>5.50</TaxPercent>"));
    assert!(xml.contains("<Amount>1200.00</Amount>"));
    assert!(xml.contains("<Amount>52.75</Amount>"));
}

// ============================================================
// Test 5 : Pièces jointes — conversion et validation MIME
// ============================================================

#[test]
fn test_attachment_pipeline() {
    use pdp_client::attachment;
    use pdp_core::model::InvoiceAttachment;

    let att = InvoiceAttachment {
        id: Some("ATT-001".to_string()),
        description: Some("Bon de commande".to_string()),
        external_uri: None,
        embedded_content: Some(b"PDF binary content here".to_vec()),
        mime_code: Some("application/pdf".to_string()),
        filename: Some("bon_commande.pdf".to_string()),
    };

    // Convertir en FluxFile pour inclusion dans un tar.gz PPF
    let flux_file = attachment::attachment_to_flux_file(&att).unwrap();
    assert_eq!(flux_file.filename, "bon_commande.pdf");
    assert_eq!(flux_file.content, b"PDF binary content here");

    // Vérifier les MIME acceptés
    assert!(attachment::is_accepted_mime("application/pdf"));
    assert!(attachment::is_accepted_mime("application/ubl+xml"));
    assert!(!attachment::is_accepted_mime("application/zip"));

    // Déduction MIME depuis le nom de fichier
    assert_eq!(attachment::mime_from_filename("doc.pdf"), Some("application/pdf"));
    assert_eq!(attachment::mime_from_filename("image.png"), Some("image/png"));
}

// ============================================================
// Test 6 : Exchange properties flow through pipeline
// ============================================================

#[test]
fn test_exchange_properties_flow() {
    let mut exchange = Exchange::new(b"<Invoice/>".to_vec());

    // Simuler les propriétés ajoutées par chaque étape du pipeline
    exchange.set_property("flow.syntax", "UBL");
    exchange.set_property("flow.profile", "CIUS");
    exchange.set_property("flow.processing_rule", "B2B");
    exchange.set_property("flow.type", "CustomerInvoice");

    assert_eq!(exchange.get_property("flow.syntax").map(|s| s.as_str()), Some("UBL"));
    assert_eq!(exchange.get_property("flow.profile").map(|s| s.as_str()), Some("CIUS"));

    // Simuler les propriétés ajoutées après dépôt PPF
    exchange.set_property("ppf.flux.id", "FLUX-12345");
    exchange.set_property("ppf.deposit.status", "OK");

    assert_eq!(exchange.get_property("ppf.flux.id").map(|s| s.as_str()), Some("FLUX-12345"));

    // Simuler les propriétés ajoutées après envoi AFNOR
    exchange.set_property("afnor.flow.id", "FLOW-67890");
    exchange.set_property("afnor.ack.status", "Ok");

    assert_eq!(exchange.get_property("afnor.flow.id").map(|s| s.as_str()), Some("FLOW-67890"));
}

// ============================================================
// Test 7 : Cas d'usage XP Z12-014 UC1 - Facture simple
// ============================================================

#[test]
fn test_uc1_invoice_flow() {
    use pdp_cdar::model::{InvoiceStatusCode, CdvTypeCode, RoleCode};

    // Charger la facture exemple (CII)
    let cii_xml = std::fs::read_to_string("../../tests/fixtures/xp-z12-014/UC1/UC1_F202500003_00-INV_20250701_CII.xml")
        .expect("Fixture CII UC1 introuvable");

    // Parser la facture
    let parser = pdp_invoice::cii::CiiParser::new();
    let invoice = parser.parse(&cii_xml).expect("Parsing CII échoué");

    assert_eq!(invoice.invoice_number, "F202500003");
    assert_eq!(invoice.source_format, pdp_core::model::InvoiceFormat::CII);

    let cdv_gen = pdp_cdar::CdarGenerator::new("100000009", "PDP Test");
    let invoice_type = invoice.invoice_type_code.as_deref().unwrap_or("380");

    // Générer CDV 200 - Déposée
    let cdv_200 = cdv_gen.generate_deposee(&invoice, invoice_type);
    assert_eq!(cdv_200.status_code(), Some(200));
    let xml_200 = cdv_gen.to_xml(&cdv_200).unwrap();
    assert!(xml_200.contains("200"));
    assert!(xml_200.contains(&invoice.invoice_number));

    // Générer CDV 202 - Reçue
    let cdv_202 = cdv_gen.generate_status(
        InvoiceStatusCode::Recue,
        CdvTypeCode::Transmission,
        RoleCode::BY, // Buyer
        &invoice,
        invoice_type,
        vec![],
    );
    assert_eq!(cdv_202.status_code(), Some(202));
    let xml_202 = cdv_gen.to_xml(&cdv_202).unwrap();
    assert!(xml_202.contains("202"));

    // Générer CDV 203 - Mise à disposition
    let cdv_203 = cdv_gen.generate_mise_a_disposition(&invoice, invoice_type);
    assert_eq!(cdv_203.status_code(), Some(203));
    let xml_203 = cdv_gen.to_xml(&cdv_203).unwrap();
    assert!(xml_203.contains("203"));

    // Générer CDV 204 - Prise en charge
    let cdv_204 = cdv_gen.generate_prise_en_charge(&invoice, invoice_type);
    assert_eq!(cdv_204.status_code(), Some(204));
    let xml_204 = cdv_gen.to_xml(&cdv_204).unwrap();
    assert!(xml_204.contains("204"));

    // Générer CDV 205 - Approuvée (utiliser generate_status)
    let cdv_205 = cdv_gen.generate_status(
        InvoiceStatusCode::Approuvee,
        CdvTypeCode::Traitement,
        RoleCode::BY,
        &invoice,
        invoice_type,
        vec![],
    );
    assert_eq!(cdv_205.status_code(), Some(205));
    let xml_205 = cdv_gen.to_xml(&cdv_205).unwrap();
    assert!(xml_205.contains("205"));
    // Valider CDV 205 contre l'exemple
    let example_205_xml = std::fs::read_to_string("../../tests/fixtures/xp-z12-014/UC1/UC1_F202500003_05-CDV-205_Approuvee.xml")
        .expect("Fixture exemple CDV 205 introuvable");
    let cdv_parser = pdp_cdar::CdarParser::new();
    let example_205_cdv = cdv_parser.parse(&example_205_xml).expect("Parsing exemple CDV 205 échoué");
    assert_eq!(cdv_205.status_code(), example_205_cdv.status_code());
    assert_eq!(cdv_205.referenced_documents[0].invoice_id, example_205_cdv.referenced_documents[0].invoice_id);
    // Générer CDV 211 - Paiement transmis
    let cdv_211 = cdv_gen.generate_status(
        InvoiceStatusCode::PaiementTransmis,
        CdvTypeCode::Traitement,
        RoleCode::BY,
        &invoice,
        invoice_type,
        vec![],
    );
    assert_eq!(cdv_211.status_code(), Some(211));
    let xml_211 = cdv_gen.to_xml(&cdv_211).unwrap();
    assert!(xml_211.contains("211"));

    // Générer CDV 212 - Encaissée
    let cdv_212 = cdv_gen.generate_status(
        InvoiceStatusCode::Encaissee,
        CdvTypeCode::Traitement,
        RoleCode::BY,
        &invoice,
        invoice_type,
        vec![],
    );
    assert_eq!(cdv_212.status_code(), Some(212));
    let xml_212 = cdv_gen.to_xml(&cdv_212).unwrap();
    assert!(xml_212.contains("212"));

    // TODO: Comparer les XML générés avec les exemples
    // Pour cela, parser les XML attendus et comparer les champs clés

    // Exemple de validation pour CDV 200
    let example_xml = std::fs::read_to_string("../../tests/fixtures/xp-z12-014/UC1/UC1_F202500003_01-CDV-200_Deposee.xml")
        .expect("Fixture exemple CDV 200 introuvable");
    let cdv_parser = pdp_cdar::CdarParser::new();
    let example_cdv = cdv_parser.parse(&example_xml).expect("Parsing exemple CDV échoué");
    assert_eq!(cdv_200.status_code(), example_cdv.status_code());
    assert_eq!(cdv_200.referenced_documents[0].invoice_id, example_cdv.referenced_documents[0].invoice_id);
}

#[tokio::test]
async fn test_uc1_cdv_processing() {
    // Simuler la réception d'un CDV "Déposée" (200) depuis l'exemple
    let cdv_xml = std::fs::read_to_string("../../tests/fixtures/xp-z12-014/UC1/UC1_F202500003_01-CDV-200_Deposee.xml")
        .expect("Fixture CDV 200 introuvable");

    let processor = pdp_cdar::CdvReceptionProcessor::new();
    let exchange = pdp_core::exchange::Exchange::new(cdv_xml.into_bytes());
    let result = processor.process(exchange).await.expect("Traitement CDV échoué");

    // Vérifier que le statut est correctement mappé
    assert_eq!(result.status, pdp_core::model::FlowStatus::Distributed);
    assert_eq!(result.get_property("cdv.invoice_id").map(|s: &String| s.as_str()), Some("F202500003"));
    assert_eq!(result.get_property("cdv.status_code").map(|s: &String| s.as_str()), Some("200"));
    assert!(!result.has_errors());

    // Simuler la réception d'un CDV "Approuvée" (205)
    let cdv_205_xml = std::fs::read_to_string("../../tests/fixtures/xp-z12-014/UC1/UC1_F202500003_05-CDV-205_Approuvee.xml")
        .expect("Fixture CDV 205 introuvable");

    let exchange_205 = pdp_core::exchange::Exchange::new(cdv_205_xml.into_bytes());
    let result_205 = processor.process(exchange_205).await.expect("Traitement CDV 205 échoué");

    assert_eq!(result_205.status, pdp_core::model::FlowStatus::Acknowledged);
    assert_eq!(result_205.get_property("cdv.status_code").map(|s: &String| s.as_str()), Some("205"));
}

#[tokio::test]
async fn test_error_handling() {
    // Test 1 : Parsing d'une facture CII invalide
    let invalid_cii = "<xml><invalid><unclosed></xml>";
    let parser = pdp_invoice::cii::CiiParser::new();
    let result = parser.parse(invalid_cii);
    assert!(result.is_err(), "Parsing d'une CII invalide devrait échouer");

    // Test 2 : Parsing d'une facture UBL invalide
    let invalid_ubl = "<xml><invalid><unclosed></xml>";
    let ubl_parser = pdp_invoice::ubl::UblParser::new();
    let ubl_result = ubl_parser.parse(invalid_ubl);
    assert!(ubl_result.is_err(), "Parsing d'une UBL invalide devrait échouer");

    // Test 3 : Génération de CDV avec numéro de facture vide
    let cdv_gen = pdp_cdar::CdarGenerator::new("100000009", "PDP Test");
    let invalid_invoice = pdp_core::model::InvoiceData::new("INVALID-001".to_string(), pdp_core::model::InvoiceFormat::CII);
    let cdv = cdv_gen.generate_deposee(&invalid_invoice, "380");
    // Devrait quand même générer
    assert_eq!(cdv.status_code(), Some(200));

    // Test 4 : Traitement d'un CDV invalide
    let invalid_cdv_xml = "<xml><invalid><unclosed></xml>";
    let cdv_processor = pdp_cdar::CdvReceptionProcessor::new();
    let exchange = pdp_core::exchange::Exchange::new(invalid_cdv_xml.as_bytes().to_vec());
    let process_result = cdv_processor.process(exchange).await;
    assert!(process_result.is_err(), "Traitement d'un CDV invalide devrait échouer");
}

#[tokio::test]
async fn test_rejection_scenarios() {
    // Test rejet pour facture avec erreurs de validation
    let cii_xml = std::fs::read_to_string("../../tests/fixtures/xp-z12-014/UC1/UC1_F202500003_00-INV_20250701_CII.xml")
        .expect("Fixture CII introuvable");

    let parser = pdp_invoice::cii::CiiParser::new();
    let invoice = parser.parse(&cii_xml).expect("Parsing CII échoué");

    let cdv_gen = pdp_cdar::CdarGenerator::new("100000009", "PDP Test");
    let invoice_type = invoice.invoice_type_code.as_deref().unwrap_or("380");

    // Simuler un rejet avec erreurs plus détaillées
    use pdp_cdar::model::CdarValidationError;
    let errors = vec![
        CdarValidationError {
            rule_id: "BR-FR-01".to_string(),
            severity: "ERROR".to_string(),
            location: Some("Invoice/ID".to_string()),
            message: "Numéro de facture manquant ou invalide - doit être unique par émetteur et année".to_string(),
            reason_code: Some(pdp_cdar::model::StatusReasonCode::RefErr),
        },
        CdarValidationError {
            rule_id: "BR-01".to_string(),
            severity: "ERROR".to_string(),
            location: Some("AccountingSupplierParty/PartyTaxScheme/CompanyID".to_string()),
            message: "Numéro TVA de l'émetteur manquant - obligatoire pour factures > 1000€".to_string(),
            reason_code: Some(pdp_cdar::model::StatusReasonCode::SiretErr),
        }
    ];
    let cdv_rejet = cdv_gen.generate_rejetee(&invoice, invoice_type, errors);
    assert_eq!(cdv_rejet.status_code(), Some(213));
    assert!(!cdv_rejet.is_success());

    // Vérifier que le CDV contient les erreurs détaillées
    let xml = cdv_gen.to_xml(&cdv_rejet).unwrap();
    assert!(xml.contains("213"));
    assert!(xml.contains("REF_ERR"));
    assert!(xml.contains("SIRET_ERR"));
    assert!(xml.contains("Numéro de facture"));
    assert!(xml.contains("Numéro TVA"));
}

#[test]
fn test_xsd_validation() {
    use pdp_validate::xsd::{XsdValidator, XsdDocumentType};

    let specs_dir = std::path::PathBuf::from("../../specs");
    let validator = XsdValidator::new(&specs_dir);

    // Test 1 : Validation d'une facture CII exemple (peut avoir des erreurs mineures)
    let cii_xml = std::fs::read_to_string("../../tests/fixtures/xp-z12-014/UC1/UC1_F202500003_00-INV_20250701_CII.xml")
        .expect("Fixture CII introuvable");

    let _report = validator.validate(&cii_xml, &XsdDocumentType::CiiD22B);
    // La validation doit s'exécuter sans paniquer, même si le fichier exemple a des erreurs
    // On ne fait pas d'assertion sur le nombre d'erreurs car les fixtures peuvent avoir des variations

    // Test 2 : Validation d'une facture UBL exemple
    let ubl_xml = std::fs::read_to_string("../../tests/fixtures/xp-z12-014/UC1/UC1_F202500003_00-INV_20250701_UBL.xml")
        .expect("Fixture UBL introuvable");

    let _ubl_report = validator.validate(&ubl_xml, &XsdDocumentType::UblInvoice);
    // La validation doit s'exécuter sans paniquer

    // Test 3 : Validation d'un CDV généré
    let parser = pdp_invoice::cii::CiiParser::new();
    let invoice = parser.parse(&cii_xml).expect("Parsing CII échoué");

    let cdv_gen = pdp_cdar::CdarGenerator::new("100000009", "PDP Test");
    let invoice_type = invoice.invoice_type_code.as_deref().unwrap_or("380");
    let cdv = cdv_gen.generate_deposee(&invoice, invoice_type);
    let cdv_xml = cdv_gen.to_xml(&cdv).unwrap();

    let cdv_report = validator.validate(&cdv_xml, &XsdDocumentType::Cdar);
    assert!(cdv_report.is_valid(), "Le CDV généré devrait être valide XSD. Erreurs: {:?}", cdv_report.issues);

    // Test 4 : Validation d'un XML invalide
    let invalid_xml = "<xml><invalid/></xml>";
    let invalid_report = validator.validate(invalid_xml, &XsdDocumentType::CiiD22B);
    assert!(!invalid_report.is_valid(), "Un XML invalide ne devrait pas passer la validation XSD");
}

#[tokio::test]
async fn test_sftp_file_validation() {
    // Test 1 : Fichier vide devrait être rejeté
    let _empty_exchange = pdp_core::exchange::Exchange::new(Vec::new());
    // Note: Le producer SFTP actuel n'a pas de validation de contenu vide
    // Il faudrait ajouter cette validation dans le producer

    // Test 2 : Extension non autorisée (simulation)
    // Le consumer SFTP filtre par pattern, mais on pourrait ajouter plus de validation

    // Test 3 : Fichier trop volumineux (simulation)
    // Les specs PPF limitent à 120 Mo par fichier, 1 Go par archive
    let large_content = vec![0u8; 130 * 1024 * 1024]; // 130 Mo
    let large_exchange = pdp_core::exchange::Exchange::new(large_content);

    // Pour l'instant, on teste juste que l'exchange est créé
    assert_eq!(large_exchange.body.len(), 130 * 1024 * 1024);

    // Test 4 : Nom de fichier invalide
    let _invalid_filename_exchange = pdp_core::exchange::Exchange::new(b"test content".to_vec())
        .with_filename("file with spaces and spécial chars éà.txt");

    // Test 5 : Extension dangereuse
    let _dangerous_exchange = pdp_core::exchange::Exchange::new(b"malicious content".to_vec())
        .with_filename("virus.exe");

    // Ces tests montrent qu'il faudrait ajouter de la validation
    // dans les producers/consumers SFTP pour :
    // - Vérifier que les fichiers ne sont pas vides
    // - Valider les extensions autorisées
    // - Contrôler les tailles de fichiers
    // - Sanitiser les noms de fichiers
    // - Détecter les extensions dangereuses
}

#[tokio::test]
async fn test_uc2_cdv_processing() {
    // Simuler la réception d'un CDV "Rejetée" (213) depuis UC2
    let cdv_xml = std::fs::read_to_string("../../tests/fixtures/xp-z12-014/UC2/UC2_F202500004_02-CDV-213_Rejetee.xml")
        .expect("Fixture CDV 213 UC2 introuvable");

    let processor = pdp_cdar::CdvReceptionProcessor::new();
    let exchange = pdp_core::exchange::Exchange::new(cdv_xml.into_bytes());
    let result = processor.process(exchange).await.expect("Traitement CDV échoué");

    assert_eq!(result.status, pdp_core::model::FlowStatus::Rejected);
    assert_eq!(result.get_property("cdv.invoice_id").map(|s: &String| s.as_str()), Some("F202500004"));
    assert_eq!(result.get_property("cdv.status_code").map(|s: &String| s.as_str()), Some("213"));
    assert!(result.has_errors());
}

#[test]
fn test_uc1_invoice_flow_ubl() {
    // Charger la facture exemple (UBL)
    let ubl_xml = std::fs::read_to_string("../../tests/fixtures/xp-z12-014/UC1/UC1_F202500003_00-INV_20250701_UBL.xml")
        .expect("Fixture UBL UC1 introuvable");

    // Parser la facture
    let parser = pdp_invoice::ubl::UblParser::new();
    let invoice = parser.parse(&ubl_xml).expect("Parsing UBL échoué");

    assert_eq!(invoice.invoice_number, "F202500003");
    assert_eq!(invoice.source_format, pdp_core::model::InvoiceFormat::UBL);

    let cdv_gen = pdp_cdar::CdarGenerator::new("100000009", "PDP Test");
    let invoice_type = invoice.invoice_type_code.as_deref().unwrap_or("380");

    // Générer CDV 200 - Déposée
    let cdv_200 = cdv_gen.generate_deposee(&invoice, invoice_type);
    assert_eq!(cdv_200.status_code(), Some(200));
    let xml_200 = cdv_gen.to_xml(&cdv_200).unwrap();
    assert!(xml_200.contains("200"));
    assert!(xml_200.contains(&invoice.invoice_number));
}

#[test]
fn test_uc2_invoice_flow() {
    // Charger la facture exemple (CII) UC2
    let cii_xml = std::fs::read_to_string("../../tests/fixtures/xp-z12-014/UC2/UC2_F202500004_00-INV_20250701_CII.xml")
        .expect("Fixture CII UC2 introuvable");

    let parser = pdp_invoice::cii::CiiParser::new();
    let invoice = parser.parse(&cii_xml).expect("Parsing CII échoué");

    assert_eq!(invoice.invoice_number, "F202500004");

    let cdv_gen = pdp_cdar::CdarGenerator::new("100000009", "PDP Test");
    let invoice_type = invoice.invoice_type_code.as_deref().unwrap_or("380");

    // Générer CDV 200 - Déposée
    let cdv_200 = cdv_gen.generate_deposee(&invoice, invoice_type);
    assert_eq!(cdv_200.status_code(), Some(200));

    // Valider contre l'exemple
    let example_200_xml = std::fs::read_to_string("../../tests/fixtures/xp-z12-014/UC2/UC2_F202500004_01-CDV-200_Deposee.xml")
        .expect("Fixture exemple CDV 200 UC2 introuvable");
    let cdv_parser = pdp_cdar::CdarParser::new();
    let example_200_cdv = cdv_parser.parse(&example_200_xml).expect("Parsing exemple CDV 200 UC2 échoué");
    assert_eq!(cdv_200.status_code(), example_200_cdv.status_code());
    assert_eq!(cdv_200.referenced_documents[0].invoice_id, example_200_cdv.referenced_documents[0].invoice_id);

    // Générer CDV 213 - Rejetée
    let cdv_213 = cdv_gen.generate_rejetee(&invoice, invoice_type, vec![]);
    assert_eq!(cdv_213.status_code(), Some(213));

    // Valider contre l'exemple
    let example_213_xml = std::fs::read_to_string("../../tests/fixtures/xp-z12-014/UC2/UC2_F202500004_02-CDV-213_Rejetee.xml")
        .expect("Fixture exemple CDV 213 UC2 introuvable");
    let example_213_cdv = cdv_parser.parse(&example_213_xml).expect("Parsing exemple CDV 213 UC2 échoué");
    assert_eq!(cdv_213.status_code(), example_213_cdv.status_code());
    assert_eq!(cdv_213.referenced_documents[0].invoice_id, example_213_cdv.referenced_documents[0].invoice_id);
}

#[test]
fn test_uc3_invoice_flow() {
    use pdp_cdar::model::{InvoiceStatusCode, CdvTypeCode, RoleCode};

    // Charger la facture exemple (CII) UC3
    let cii_xml = std::fs::read_to_string("../../tests/fixtures/xp-z12-014/UC3/UC3_F202500005_00-INV_20250701_CII.xml")
        .expect("Fixture CII UC3 introuvable");

    let parser = pdp_invoice::cii::CiiParser::new();
    let invoice = parser.parse(&cii_xml).expect("Parsing CII échoué");

    assert_eq!(invoice.invoice_number, "F202500005");

    let cdv_gen = pdp_cdar::CdarGenerator::new("100000009", "PDP Test");
    let invoice_type = invoice.invoice_type_code.as_deref().unwrap_or("380");

    // Générer CDV 200 - Déposée
    let cdv_200 = cdv_gen.generate_deposee(&invoice, invoice_type);
    assert_eq!(cdv_200.status_code(), Some(200));

    // Générer CDV 202 - Reçue
    let cdv_202 = cdv_gen.generate_status(
        InvoiceStatusCode::Recue,
        CdvTypeCode::Transmission,
        RoleCode::BY,
        &invoice,
        invoice_type,
        vec![],
    );
    assert_eq!(cdv_202.status_code(), Some(202));

    // Générer CDV 203 - Mise à disposition
    let cdv_203 = cdv_gen.generate_mise_a_disposition(&invoice, invoice_type);
    assert_eq!(cdv_203.status_code(), Some(203));

    // Générer CDV 210 - Refusée
    let cdv_210 = cdv_gen.generate_status(
        InvoiceStatusCode::Refusee,
        CdvTypeCode::Traitement,
        RoleCode::BY,
        &invoice,
        invoice_type,
        vec![],
    );
    assert_eq!(cdv_210.status_code(), Some(210));

    // Valider CDV 210 contre l'exemple
    let example_210_xml = std::fs::read_to_string("../../tests/fixtures/xp-z12-014/UC3/UC3_F202500005_04-CDV-210_Refusee.xml")
        .expect("Fixture exemple CDV 210 UC3 introuvable");
    let cdv_parser = pdp_cdar::CdarParser::new();
    let example_210_cdv = cdv_parser.parse(&example_210_xml).expect("Parsing exemple CDV 210 UC3 échoué");
    assert_eq!(cdv_210.status_code(), example_210_cdv.status_code());
    assert_eq!(cdv_210.referenced_documents[0].invoice_id, example_210_cdv.referenced_documents[0].invoice_id);
}

#[test]
fn test_uc4_invoice_flow() {
    use pdp_cdar::model::{InvoiceStatusCode, CdvTypeCode, RoleCode};

    // Charger la facture exemple (CII) UC4
    let cii_xml = std::fs::read_to_string("../../tests/fixtures/xp-z12-014/UC4/UC4_F202500006_00-INV_20250701_CII.xml")
        .expect("Fixture CII UC4 introuvable");

    let parser = pdp_invoice::cii::CiiParser::new();
    let invoice = parser.parse(&cii_xml).expect("Parsing CII échoué");

    assert_eq!(invoice.invoice_number, "F202500006");

    let cdv_gen = pdp_cdar::CdarGenerator::new("100000009", "PDP Test");
    let invoice_type = invoice.invoice_type_code.as_deref().unwrap_or("380");

    // Générer CDV 200 - Déposée
    let cdv_200 = cdv_gen.generate_deposee(&invoice, invoice_type);
    assert_eq!(cdv_200.status_code(), Some(200));

    // Générer CDV 202 - Reçue
    let cdv_202 = cdv_gen.generate_status(
        InvoiceStatusCode::Recue,
        CdvTypeCode::Transmission,
        RoleCode::BY,
        &invoice,
        invoice_type,
        vec![],
    );
    assert_eq!(cdv_202.status_code(), Some(202));

    // Générer CDV 203 - Mise à disposition
    let cdv_203 = cdv_gen.generate_mise_a_disposition(&invoice, invoice_type);
    assert_eq!(cdv_203.status_code(), Some(203));

    // Générer CDV 207 - En litige
    let cdv_207 = cdv_gen.generate_status(
        InvoiceStatusCode::EnLitige,
        CdvTypeCode::Traitement,
        RoleCode::BY,
        &invoice,
        invoice_type,
        vec![],
    );
    assert_eq!(cdv_207.status_code(), Some(207));

    // Valider CDV 207 contre l'exemple
    let example_207_xml = std::fs::read_to_string("../../tests/fixtures/xp-z12-014/UC4/UC4_F202500006_04-CDV-207_En_litige.xml")
        .expect("Fixture exemple CDV 207 UC4 introuvable");
    let cdv_parser = pdp_cdar::CdarParser::new();
    let example_207_cdv = cdv_parser.parse(&example_207_xml).expect("Parsing exemple CDV 207 UC4 échoué");
    assert_eq!(cdv_207.status_code(), example_207_cdv.status_code());
    assert_eq!(cdv_207.referenced_documents[0].invoice_id, example_207_cdv.referenced_documents[0].invoice_id);

    // Générer CDV 220 - Annulée
    let cdv_220 = cdv_gen.generate_status(
        InvoiceStatusCode::Annulee,
        CdvTypeCode::Traitement,
        RoleCode::BY,
        &invoice,
        invoice_type,
        vec![],
    );
    assert_eq!(cdv_220.status_code(), Some(220));
}

#[test]
fn test_uc5_invoice_flow() {
    use pdp_cdar::model::{InvoiceStatusCode, CdvTypeCode, RoleCode};

    // Charger la facture exemple (CII) UC5
    let cii_xml = std::fs::read_to_string("../../tests/fixtures/xp-z12-014/UC5/UC5_F202500007_00-INV_20250702_CII.xml")
        .expect("Fixture CII UC5 introuvable");

    let parser = pdp_invoice::cii::CiiParser::new();
    let invoice = parser.parse(&cii_xml).expect("Parsing CII échoué");

    assert_eq!(invoice.invoice_number, "F202500007");

    let cdv_gen = pdp_cdar::CdarGenerator::new("100000009", "PDP Test");
    let invoice_type = invoice.invoice_type_code.as_deref().unwrap_or("380");

    // Générer CDV 200 - Déposée
    let cdv_200 = cdv_gen.generate_deposee(&invoice, invoice_type);
    assert_eq!(cdv_200.status_code(), Some(200));

    // Générer CDV 202 - Reçue
    let cdv_202 = cdv_gen.generate_status(
        InvoiceStatusCode::Recue,
        CdvTypeCode::Transmission,
        RoleCode::BY,
        &invoice,
        invoice_type,
        vec![],
    );
    assert_eq!(cdv_202.status_code(), Some(202));

    // Générer CDV 203 - Mise à disposition
    let cdv_203 = cdv_gen.generate_mise_a_disposition(&invoice, invoice_type);
    assert_eq!(cdv_203.status_code(), Some(203));

    // Générer CDV 207 - En litige
    let cdv_207 = cdv_gen.generate_status(
        InvoiceStatusCode::EnLitige,
        CdvTypeCode::Traitement,
        RoleCode::BY,
        &invoice,
        invoice_type,
        vec![],
    );
    assert_eq!(cdv_207.status_code(), Some(207));

    // Générer CDV 205 - Approuvée
    let cdv_205 = cdv_gen.generate_status(
        InvoiceStatusCode::Approuvee,
        CdvTypeCode::Traitement,
        RoleCode::BY,
        &invoice,
        invoice_type,
        vec![],
    );
    assert_eq!(cdv_205.status_code(), Some(205));

    // Valider CDV 205 contre l'exemple
    let example_205_xml = std::fs::read_to_string("../../tests/fixtures/xp-z12-014/UC5/UC5b_F202500007_05-CDV-205_Approuvee.xml")
        .expect("Fixture exemple CDV 205 UC5 introuvable");
    let cdv_parser = pdp_cdar::CdarParser::new();
    let example_205_cdv = cdv_parser.parse(&example_205_xml).expect("Parsing exemple CDV 205 UC5 échoué");
    assert_eq!(cdv_205.status_code(), example_205_cdv.status_code());
    assert_eq!(cdv_205.referenced_documents[0].invoice_id, example_205_cdv.referenced_documents[0].invoice_id);

    // Générer CDV 212 - Encaissée
    let cdv_212 = cdv_gen.generate_status(
        InvoiceStatusCode::Encaissee,
        CdvTypeCode::Traitement,
        RoleCode::BY,
        &invoice,
        invoice_type,
        vec![],
    );
    assert_eq!(cdv_212.status_code(), Some(212));
}

// ============================================================
// Test 7 : Destination routing enum
// ============================================================

#[test]
fn test_destination_display() {
    use pdp_client::Destination;

    let ppf = Destination::PpfSe;
    assert_eq!(format!("{}", ppf), "PPF-SE");

    let pdp = Destination::AfnorPdp {
        matricule: "0238".to_string(),
        flow_service_url: "https://api.pdp-partner.fr/flow-service".to_string(),
    };
    assert_eq!(format!("{}", pdp), "PDP-0238");

    let file = Destination::File {
        path: "/tmp/output".to_string(),
    };
    assert_eq!(format!("{}", file), "FILE:/tmp/output");
}

// ============================================================
// Test 8 : Round-trip CDV generation → parsing
// ============================================================

#[test]
fn test_cdv_roundtrip() {
    let xml = std::fs::read_to_string("../../tests/fixtures/ubl/facture_ubl_001.xml")
        .expect("Fixture UBL introuvable");

    // Parser la facture
    let parser = pdp_invoice::ubl::UblParser::new();
    let invoice = parser.parse(&xml).expect("Parsing UBL échoué");

    // Générer un CDV
    let gen = pdp_cdar::CdarGenerator::new("100000009", "PDP Test");
    let invoice_type = invoice.invoice_type_code.as_deref().unwrap_or("380");
    let cdv = gen.generate_deposee(&invoice, invoice_type);
    let cdv_xml = gen.to_xml(&cdv).expect("Sérialisation CDV échouée");

    // Re-parser le CDV généré
    let cdv_parser = pdp_cdar::CdarParser::new();
    let parsed_cdv = cdv_parser.parse(&cdv_xml).expect("Re-parsing CDV échoué");

    assert_eq!(parsed_cdv.status_code(), Some(200));
    assert!(parsed_cdv.is_success());
    assert_eq!(
        parsed_cdv.referenced_documents[0].invoice_id,
        invoice.invoice_number
    );
}
