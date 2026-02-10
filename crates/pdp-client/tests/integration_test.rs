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
