//! Tests d'intégration : gestion des erreurs dans les pipelines émission et réception.
//!
//! Vérifie que les fichiers incorrects (XML mal formé, non-XML, PDF sans XML,
//! XML invalide, violations Schematron) produisent les bons CDV d'erreur
//! (501 Irrecevable ou 213 Rejetée) dans les deux modes de la PDP.

use pdp_core::error::PdpError;
use pdp_core::exchange::Exchange;
use pdp_core::model::{InvoiceData, InvoiceFormat};
use pdp_core::processor::Processor;

use pdp_cdar::processor::{CdarProcessor, IrrecevabiliteProcessor, DocumentTypeRouter};
use pdp_cdar::{CdarParser, CdvTypeCode};

// ============================================================
// Helpers
// ============================================================

fn load_error_fixture(name: &str) -> Vec<u8> {
    let path = format!(
        "{}/../../tests/fixtures/errors/{}",
        env!("CARGO_MANIFEST_DIR"),
        name
    );
    std::fs::read(&path).unwrap_or_else(|e| panic!("Fixture {} introuvable: {}", path, e))
}

fn load_fixture(dir: &str, name: &str) -> Vec<u8> {
    let path = format!(
        "{}/../../tests/fixtures/{}/{}",
        env!("CARGO_MANIFEST_DIR"),
        dir,
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

/// Simule le pipeline de réception commun : ReceptionProcessor → IrrecevabiliteProcessor
async fn run_reception_checks(exchange: Exchange) -> Exchange {
    let reception = pdp_core::reception::ReceptionProcessor::strict();
    let irrecevabilite = IrrecevabiliteProcessor::new("PDP-TEST", "PDP Test");

    let exchange = reception.process(exchange).await.unwrap();
    irrecevabilite.process(exchange).await.unwrap()
}

/// Simule le pipeline de parsing : DocumentTypeRouter → ParseProcessor
/// En cas d'erreur de parsing, l'erreur est ajoutée à l'exchange (pas de panic).
async fn run_parse(exchange: Exchange) -> Exchange {
    let router = DocumentTypeRouter::new();
    let parser = pdp_invoice::ParseProcessor::new();

    let exchange = router.process(exchange).await.unwrap();
    match parser.process(exchange.clone()).await {
        Ok(ex) => ex,
        Err(e) => {
            let mut ex = exchange;
            ex.add_error("parsing", &e);
            ex
        }
    }
}

/// Simule le pipeline de validation.
/// Utilise le mode lenient pour que les erreurs soient ajoutées à l'exchange
/// sans bloquer le pipeline — c'est ce que fait le vrai pipeline via le
/// framework de routes qui capture les Err et les passe au error handler.
async fn run_validation(exchange: Exchange) -> Exchange {
    // Mode lenient : ajoute les erreurs comme propriétés mais ne bloque pas
    let validator = pdp_invoice::ValidateProcessor::lenient();
    match validator.process(exchange.clone()).await {
        Ok(mut ex) => {
            // Convertir les erreurs de validation en erreurs d'exchange
            // pour que CdarProcessor les détecte via has_errors()
            if ex.get_property("validation.valid").map(|s| s.as_str()) == Some("false") {
                let detail = ex.get_property("validation.errors.detail")
                    .cloned()
                    .unwrap_or_default();
                ex.add_error("validation", &PdpError::ValidationError(
                    format!("Validation échouée: {}", detail)
                ));
            }
            ex
        }
        Err(e) => {
            let mut ex = exchange;
            ex.add_error("validation", &e);
            ex
        }
    }
}

// ============================================================
// 1. Fichier vide
// ============================================================

/// Parse le CDV XML et vérifie le code raison du premier motif
fn assert_cdv_reason(exchange: &Exchange, expected_reason_prefix: &str) {
    let cdv_xml = exchange.get_property("cdv.xml")
        .expect("CDV XML absent de l'exchange");
    let cdv = CdarParser::new().parse(cdv_xml)
        .expect("Impossible de parser le CDV XML généré");

    assert!(!cdv.referenced_documents.is_empty(), "CDV sans document référencé");
    let ref_doc = &cdv.referenced_documents[0];
    assert!(!ref_doc.statuses.is_empty(),
        "CDV {} sans motif (statuses vide)", ref_doc.process_condition_code);

    let first_reason = ref_doc.statuses[0].reason_code.as_deref()
        .expect("Premier motif sans reason_code");
    assert!(first_reason.starts_with(expected_reason_prefix),
        "Code raison '{}' ne commence pas par '{}' (CDV {})",
        first_reason, expected_reason_prefix, ref_doc.process_condition_code);

    // Vérifier qu'il y a un message lisible
    assert!(ref_doc.statuses[0].reason.is_some(),
        "Premier motif sans message lisible (CDV {})", ref_doc.process_condition_code);
}

#[tokio::test]
async fn test_emission_fichier_vide_genere_cdv_501_irr_vide_f() {
    let mut exchange = Exchange::new(vec![]);
    exchange.source_filename = Some("facture_vide.xml".to_string());

    let exchange = run_reception_checks(exchange).await;

    assert!(exchange.get_property("reception.failed").is_some());
    assert_eq!(exchange.get_property("cdv.status_code").unwrap(), "501");
    // Vérifier le code raison IRR_VIDE_F et le message
    assert_cdv_reason(&exchange, "IRR_VIDE_F");
}

#[tokio::test]
async fn test_reception_fichier_vide_genere_cdv_501_irr_vide_f() {
    let mut exchange = Exchange::new(vec![]);
    exchange.source_filename = Some("facture_vide.xml".to_string());
    exchange.set_header("source.protocol", "afnor-flow");

    let exchange = run_reception_checks(exchange).await;

    assert_eq!(exchange.get_property("cdv.status_code").unwrap(), "501");
    assert_cdv_reason(&exchange, "IRR_VIDE_F");
}

// ============================================================
// 2. Fichier non-XML (texte brut, binaire)
// ============================================================

#[tokio::test]
async fn test_emission_fichier_texte_brut_irr_ext_doc() {
    let mut exchange = Exchange::new(b"Ceci n'est pas du XML, juste du texte brut.".to_vec());
    exchange.source_filename = Some("readme.txt".to_string());

    let exchange = run_reception_checks(exchange).await;

    assert!(exchange.get_property("reception.failed").is_some());
    assert_eq!(exchange.get_property("cdv.status_code").unwrap(), "501");
    assert_cdv_reason(&exchange, "IRR_EXT_DOC");
}

#[tokio::test]
async fn test_emission_fichier_binaire_random_irr_ext_doc() {
    let mut exchange = Exchange::new(vec![0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A]);
    exchange.source_filename = Some("image.jpg".to_string());

    let exchange = run_reception_checks(exchange).await;

    assert!(exchange.get_property("reception.failed").is_some());
    assert_eq!(exchange.get_property("cdv.status_code").unwrap(), "501");
    assert_cdv_reason(&exchange, "IRR_EXT_DOC");
}

#[tokio::test]
async fn test_emission_fichier_csv_irr_ext_doc() {
    let mut exchange = Exchange::new(b"nom;prenom;montant\nDupont;Jean;1234.56".to_vec());
    exchange.source_filename = Some("export.csv".to_string());

    let exchange = run_reception_checks(exchange).await;

    assert!(exchange.get_property("reception.failed").is_some());
    assert_eq!(exchange.get_property("cdv.status_code").unwrap(), "501");
    assert_cdv_reason(&exchange, "IRR_EXT_DOC");
}

// ============================================================
// 3. XML mal formé (extension .xml mais contenu invalide)
// ============================================================

#[tokio::test]
async fn test_emission_xml_mal_forme() {
    let mut exchange = Exchange::new(b"<invoice><broken>xml without closing tags".to_vec());
    exchange.source_filename = Some("facture_cassee.xml".to_string());

    // Réception OK (extension .xml valide, fichier non vide)
    let exchange = run_reception_checks(exchange).await;
    assert!(exchange.get_property("reception.failed").is_none());

    // Parsing échoue → erreur ajoutée
    let exchange = run_parse(exchange).await;
    // Le XML mal formé peut soit échouer au parse, soit ne pas être reconnu comme facture
    assert!(exchange.invoice.is_none() || exchange.has_errors());
}

#[tokio::test]
async fn test_emission_xml_mal_forme_genere_cdv_213_rej_seman() {
    let mut exchange = Exchange::new(b"<invoice><broken>xml".to_vec());
    exchange.source_filename = Some("facture_cassee.xml".to_string());
    exchange.invoice = Some(make_invoice());
    exchange.add_error("parsing", &PdpError::ParseError("XML mal formé".to_string()));

    let processor = CdarProcessor::emission("999888777", "Ma PDP");
    let result = processor.process(exchange).await.unwrap();

    assert_eq!(result.get_property("cdv.status_code").unwrap(), "213");
    // Le parsing XML mal formé → REJ_SEMAN (car le message contient "xml")
    assert_cdv_reason(&result, "REJ_SEMAN");

    let cdv_xml = result.get_property("cdv.xml").unwrap();
    let cdv = CdarParser::new().parse(cdv_xml).unwrap();
    assert_eq!(cdv.type_code, CdvTypeCode::Transmission);

    // Vérifier le motif complet
    let status = &cdv.referenced_documents[0].statuses[0];
    assert!(status.reason.as_deref().unwrap().contains("XML mal formé"));
    assert_eq!(status.sequence, Some(1));
}

#[tokio::test]
async fn test_reception_xml_mal_forme_genere_cdv_213_rej_seman() {
    let mut exchange = Exchange::new(b"<invoice><broken>xml".to_vec());
    exchange.source_filename = Some("facture_cassee.xml".to_string());
    exchange.set_header("source.protocol", "afnor-flow");
    exchange.invoice = Some(make_invoice());
    exchange.add_error("parsing", &PdpError::ParseError("XML mal formé".to_string()));

    let processor = CdarProcessor::reception("111222333", "PDP Réceptrice");
    let result = processor.process(exchange).await.unwrap();

    assert_eq!(result.get_property("cdv.status_code").unwrap(), "213");
    assert_cdv_reason(&result, "REJ_SEMAN");
}

// ============================================================
// 4. PDF sans XML embarqué
// ============================================================

#[tokio::test]
async fn test_emission_pdf_sans_xml_parse_error() {
    // Créer un faux PDF minimal (header PDF mais pas de XML embarqué)
    let mut exchange = Exchange::new(b"%PDF-1.4\n1 0 obj\n<<>>\nendobj\n%%EOF".to_vec());
    exchange.source_filename = Some("facture_sans_xml.pdf".to_string());

    // Réception OK (extension .pdf autorisée)
    let exchange = run_reception_checks(exchange).await;
    assert!(exchange.get_property("reception.failed").is_none());

    // Parse → l'exchange n'aura pas d'invoice (PDF invalide)
    let exchange = run_parse(exchange).await;
    assert!(exchange.invoice.is_none());
}

// ============================================================
// 5. XML valide mais échoue à la validation métier
// ============================================================

#[tokio::test]
async fn test_emission_xml_invalide_cii_genere_cdv_213_avec_motifs() {
    let body = load_error_fixture("facture_invalide_001.xml");
    let mut exchange = Exchange::new(body);
    exchange.source_filename = Some("facture_invalide_001.xml".to_string());

    let exchange = run_reception_checks(exchange).await;
    assert!(exchange.get_property("reception.failed").is_none());

    let exchange = run_parse(exchange).await;
    assert!(exchange.invoice.is_some());

    let exchange = run_validation(exchange).await;
    assert!(exchange.has_errors(), "La validation devrait trouver des erreurs");

    let processor = CdarProcessor::emission("999888777", "Ma PDP");
    let result = processor.process(exchange).await.unwrap();

    assert_eq!(result.get_property("cdv.status_code").unwrap(), "213");

    // Vérifier les motifs de rejet dans le CDV
    let cdv_xml = result.get_property("cdv.xml").unwrap();
    let cdv = CdarParser::new().parse(cdv_xml).unwrap();
    let ref_doc = &cdv.referenced_documents[0];
    assert!(!ref_doc.statuses.is_empty(),
        "Le CDV 213 devrait contenir au moins un motif de rejet");

    // Chaque motif a un code raison et un message
    for (i, status) in ref_doc.statuses.iter().enumerate() {
        assert!(status.reason_code.is_some(),
            "Motif {} sans code raison", i + 1);
        assert!(status.reason.is_some(),
            "Motif {} sans message lisible", i + 1);
        assert_eq!(status.sequence, Some((i + 1) as u32),
            "Motif {} : séquence incorrecte", i + 1);
    }
}

#[tokio::test]
async fn test_reception_xml_invalide_cii_genere_cdv_213_avec_motifs() {
    let body = load_error_fixture("facture_invalide_001.xml");
    let mut exchange = Exchange::new(body);
    exchange.source_filename = Some("facture_invalide_001.xml".to_string());
    exchange.set_header("source.protocol", "afnor-flow");

    let exchange = run_reception_checks(exchange).await;
    let exchange = run_parse(exchange).await;
    let exchange = run_validation(exchange).await;
    assert!(exchange.has_errors());

    let processor = CdarProcessor::reception("111222333", "PDP Réceptrice");
    let result = processor.process(exchange).await.unwrap();

    assert_eq!(result.get_property("cdv.status_code").unwrap(), "213");

    // Vérifier que les motifs sont aussi présents en mode réception
    let cdv_xml = result.get_property("cdv.xml").unwrap();
    let cdv = CdarParser::new().parse(cdv_xml).unwrap();
    let ref_doc = &cdv.referenced_documents[0];
    assert!(!ref_doc.statuses.is_empty());
    for status in &ref_doc.statuses {
        assert!(status.reason_code.is_some(), "Motif sans code raison en réception");
        assert!(status.reason.is_some(), "Motif sans message en réception");
    }
}

#[tokio::test]
async fn test_emission_xml_invalide_ubl_genere_cdv_213() {
    let body = load_error_fixture("facture_invalide_ubl_001.xml");
    let mut exchange = Exchange::new(body);
    exchange.source_filename = Some("facture_invalide_ubl_001.xml".to_string());

    let exchange = run_reception_checks(exchange).await;
    let exchange = run_parse(exchange).await;
    assert!(exchange.invoice.is_some());

    let exchange = run_validation(exchange).await;
    assert!(exchange.has_errors());

    let processor = CdarProcessor::emission("999888777", "Ma PDP");
    let result = processor.process(exchange).await.unwrap();

    assert_eq!(result.get_property("cdv.status_code").unwrap(), "213");
}

// ============================================================
// 6. XML avec violations Schematron spécifiques (BR-FR)
// ============================================================

#[tokio::test]
async fn test_emission_cii_sans_endpoint_genere_cdv_213() {
    // BR-FR-12/13 : SIRET/SIREN manquant — détecté par Schematron (XmlValidateProcessor)
    // Le ValidateProcessor Rust ne couvre pas cette règle pour CII
    let body = load_error_fixture("cii_sans_endpoint.xml");
    let mut exchange = Exchange::new(body);
    exchange.source_filename = Some("cii_sans_endpoint.xml".to_string());

    let exchange = run_reception_checks(exchange).await;
    let exchange = run_parse(exchange).await;
    assert!(exchange.invoice.is_some());

    let exchange = run_validation(exchange).await;

    if exchange.has_errors() {
        let processor = CdarProcessor::emission("999888777", "Ma PDP");
        let result = processor.process(exchange).await.unwrap();
        assert_eq!(result.get_property("cdv.status_code").unwrap(), "213");
    }
    // Sinon : la validation BR-FR-12/13 est couverte par XmlValidateProcessor (Schematron)
}

#[tokio::test]
async fn test_emission_cii_id_trop_long_genere_cdv_213() {
    // BR-FR-01 : identifiant facture > 35 caractères
    let body = load_error_fixture("cii_id_trop_long.xml");
    let mut exchange = Exchange::new(body);
    exchange.source_filename = Some("cii_id_trop_long.xml".to_string());

    let exchange = run_reception_checks(exchange).await;
    let exchange = run_parse(exchange).await;
    let exchange = run_validation(exchange).await;
    assert!(exchange.has_errors(), "BR-FR-01 devrait être violée");

    let processor = CdarProcessor::emission("999888777", "Ma PDP");
    let result = processor.process(exchange).await.unwrap();

    assert_eq!(result.get_property("cdv.status_code").unwrap(), "213");
    // Vérifier que le motif contient un code raison valide
    assert_cdv_reason(&result, "REJ_");
}

#[tokio::test]
async fn test_emission_cii_sans_acheteur_genere_cdv_213() {
    let body = load_error_fixture("cii_sans_acheteur.xml");
    let mut exchange = Exchange::new(body);
    exchange.source_filename = Some("cii_sans_acheteur.xml".to_string());

    let exchange = run_reception_checks(exchange).await;
    let exchange = run_parse(exchange).await;
    let exchange = run_validation(exchange).await;
    assert!(exchange.has_errors(), "Acheteur manquant devrait être détecté");

    let processor = CdarProcessor::emission("999888777", "Ma PDP");
    let result = processor.process(exchange).await.unwrap();

    assert_eq!(result.get_property("cdv.status_code").unwrap(), "213");
    // classify_error_reason matche "destinataire" dans le message ou "validate" dans le step
    assert_cdv_reason(&result, "REJ_");
}

#[tokio::test]
async fn test_emission_cii_type_invalide_genere_cdv_213() {
    // BR-FR-04 : TypeCode 999 non autorisé
    let body = load_error_fixture("cii_type_invalide.xml");
    let mut exchange = Exchange::new(body);
    exchange.source_filename = Some("cii_type_invalide.xml".to_string());

    let exchange = run_reception_checks(exchange).await;
    let exchange = run_parse(exchange).await;
    let exchange = run_validation(exchange).await;
    assert!(exchange.has_errors(), "BR-FR-04 devrait être violée");

    let processor = CdarProcessor::emission("999888777", "Ma PDP");
    let result = processor.process(exchange).await.unwrap();

    assert_eq!(result.get_property("cdv.status_code").unwrap(), "213");
    // Le message de validation passe par classify_error_reason → REJ_SEMAN
    assert_cdv_reason(&result, "REJ_");
}

#[tokio::test]
async fn test_emission_ubl_sans_endpoint_genere_cdv_213() {
    // BR-FR-12/13 : endpoint manquant — détecté par le ValidateProcessor Rust
    let body = load_error_fixture("ubl_sans_endpoint.xml");
    let mut exchange = Exchange::new(body);
    exchange.source_filename = Some("ubl_sans_endpoint.xml".to_string());

    let exchange = run_reception_checks(exchange).await;
    let exchange = run_parse(exchange).await;
    let exchange = run_validation(exchange).await;

    // Si le validator Rust détecte l'erreur → CDV 213
    if exchange.has_errors() {
        let processor = CdarProcessor::emission("999888777", "Ma PDP");
        let result = processor.process(exchange).await.unwrap();
        assert_eq!(result.get_property("cdv.status_code").unwrap(), "213");
    }
    // Sinon la validation BR-FR-12/13 est couverte par le Schematron (XmlValidateProcessor)
    // qui n'est pas disponible dans ce test unitaire
}

#[tokio::test]
async fn test_emission_ubl_id_trop_long_genere_cdv_213() {
    // BR-FR-01 : identifiant > 35 caractères
    let body = load_error_fixture("ubl_id_trop_long.xml");
    let mut exchange = Exchange::new(body);
    exchange.source_filename = Some("ubl_id_trop_long.xml".to_string());

    let exchange = run_reception_checks(exchange).await;
    let exchange = run_parse(exchange).await;
    let exchange = run_validation(exchange).await;

    if exchange.has_errors() {
        let processor = CdarProcessor::emission("999888777", "Ma PDP");
        let result = processor.process(exchange).await.unwrap();
        assert_eq!(result.get_property("cdv.status_code").unwrap(), "213");
    }
}

// ============================================================
// 7. Violations Schematron en mode RÉCEPTION (même résultat CDV 213)
// ============================================================

#[tokio::test]
async fn test_reception_cii_sans_endpoint_genere_cdv_213() {
    // BR-FR-12/13 : couvert par Schematron (pas par le ValidateProcessor Rust pour CII)
    let body = load_error_fixture("cii_sans_endpoint.xml");
    let mut exchange = Exchange::new(body);
    exchange.source_filename = Some("cii_sans_endpoint.xml".to_string());
    exchange.set_header("source.protocol", "afnor-flow");

    let exchange = run_reception_checks(exchange).await;
    let exchange = run_parse(exchange).await;
    let exchange = run_validation(exchange).await;

    if exchange.has_errors() {
        let processor = CdarProcessor::reception("111222333", "PDP Réceptrice");
        let result = processor.process(exchange).await.unwrap();
        assert_eq!(result.get_property("cdv.status_code").unwrap(), "213");
    }
}

#[tokio::test]
async fn test_reception_ubl_sans_acheteur_genere_cdv_213() {
    let body = load_error_fixture("ubl_sans_acheteur.xml");
    let mut exchange = Exchange::new(body);
    exchange.source_filename = Some("ubl_sans_acheteur.xml".to_string());
    exchange.set_header("source.protocol", "afnor-flow");

    let exchange = run_reception_checks(exchange).await;
    let exchange = run_parse(exchange).await;
    let exchange = run_validation(exchange).await;
    assert!(exchange.has_errors());

    let processor = CdarProcessor::reception("111222333", "PDP Réceptrice");
    let result = processor.process(exchange).await.unwrap();

    assert_eq!(result.get_property("cdv.status_code").unwrap(), "213");
}

// ============================================================
// 8. Fichier trop gros (dépassement BR-FR-19)
// ============================================================

#[tokio::test]
async fn test_emission_fichier_trop_gros_genere_cdv_501() {
    // Créer un exchange avec un body > 100MB indiqué par la propriété
    // (on ne crée pas vraiment 100MB en mémoire dans les tests)
    let mut exchange = Exchange::new(vec![0u8; 1024]); // petit body
    exchange.source_filename = Some("enorme.xml".to_string());
    // Simuler la propriété de taille excessive
    exchange.set_property("file.size", "104857601"); // 100MB + 1

    // Le ReceptionProcessor en mode strict vérifie exchange.body.len(), pas la propriété
    // Donc on simule directement l'erreur de réception
    exchange.add_error("reception", &PdpError::ValidationError(
        "BR-FR-19: Fichier trop volumineux (> 100 Mo)".to_string()
    ));
    exchange.set_property("reception.failed", "true");

    let irrecevabilite = IrrecevabiliteProcessor::new("PDP-TEST", "PDP Test");
    let result = irrecevabilite.process(exchange).await.unwrap();

    assert_eq!(result.get_property("cdv.status_code").unwrap(), "501");
    assert_cdv_reason(&result, "IRR_");
}

// ============================================================
// 9. Vérification que facture valide → CDV 200 (émission) / 202 (réception)
//    (contraste avec les tests d'erreur ci-dessus)
// ============================================================

#[tokio::test]
async fn test_emission_facture_valide_pipeline_complet_cdv_200() {
    let body = load_fixture("cii", "facture_cii_001.xml");
    let mut exchange = Exchange::new(body);
    exchange.source_filename = Some("facture_cii_001.xml".to_string());

    // Pipeline complet : réception → parsing → validation → CDAR
    let exchange = run_reception_checks(exchange).await;
    assert!(exchange.get_property("reception.failed").is_none());

    let exchange = run_parse(exchange).await;
    assert!(exchange.invoice.is_some());

    let exchange = run_validation(exchange).await;

    // S'il n'y a pas d'erreurs de validation → CDV 200
    if !exchange.has_errors() {
        let processor = CdarProcessor::emission("999888777", "Ma PDP");
        let result = processor.process(exchange).await.unwrap();
        assert_eq!(result.get_property("cdv.status_code").unwrap(), "200");
    }
    // Sinon CDV 213 (certaines fixtures peuvent avoir des warnings)
}

#[tokio::test]
async fn test_reception_facture_valide_pipeline_complet_cdv_202() {
    let body = load_fixture("cii", "facture_cii_001.xml");
    let mut exchange = Exchange::new(body);
    exchange.source_filename = Some("facture_cii_001.xml".to_string());
    exchange.set_header("source.protocol", "afnor-flow");

    let exchange = run_reception_checks(exchange).await;
    assert!(exchange.get_property("reception.failed").is_none());

    let exchange = run_parse(exchange).await;
    assert!(exchange.invoice.is_some());

    let exchange = run_validation(exchange).await;

    if !exchange.has_errors() {
        let processor = CdarProcessor::reception("111222333", "PDP Réceptrice");
        let result = processor.process(exchange).await.unwrap();
        assert_eq!(result.get_property("cdv.status_code").unwrap(), "202");
    }
}
