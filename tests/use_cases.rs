//! Tests d'intégration pour les cas d'usage XP Z12-014
//!
//! Ces tests vérifient le flux complet pour chaque cas d'usage :
//! - Parsing de la facture exemple
//! - Génération des CDV successifs
//! - Comparaison avec les exemples attendus
//! - Traitement des CDV entrants

use pdp_core::model::InvoiceFormat;
use pdp_cdar::CdarGenerator;
use pdp_ereporting::generator::EReportingGenerator;
use pdp_invoice::cii::CiiParser;
use pdp_invoice::ubl::UblParser;

/// Test pour le Cas d'Usage 1 : Facture simple
#[test]
fn test_uc1_invoice_flow() {
    // Charger la facture exemple (CII)
    let cii_xml = std::fs::read_to_string("tests/fixtures/xp-z12-014/UC1/UC1_F202500003_00-INV_20250701_CII.xml")
        .expect("Fixture CII UC1 introuvable");

    // Parser la facture
    let parser = CiiParser::new();
    let invoice = parser.parse(&cii_xml).expect("Parsing CII échoué");

    assert_eq!(invoice.invoice_number, "F202500003");
    assert_eq!(invoice.source_format, InvoiceFormat::CII);

    // Générer le CDV "Déposée" (200)
    let cdv_gen = CdarGenerator::new("100000009", "PDP Test");
    let invoice_type = invoice.invoice_type_code.as_deref().unwrap_or("380");
    let cdv_deposee = cdv_gen.generate_deposee(&invoice, invoice_type);

    assert_eq!(cdv_deposee.status_code(), Some(200));
    assert!(cdv_deposee.is_success());

    // Sérialiser et comparer avec l'exemple attendu
    let cdv_xml = cdv_gen.to_xml(&cdv_deposee).expect("Sérialisation CDV échouée");
    let expected_xml = std::fs::read_to_string("tests/fixtures/xp-z12-014/UC1/UC1_F202500003_01-CDV-200_Deposee.xml")
        .expect("Fixture CDV attendu introuvable");

    // TODO: Comparer les XML (ignorer les timestamps, IDs générés, etc.)
    // Pour l'instant, vérifier que les éléments clés sont présents
    assert!(cdv_xml.contains("CrossDomainAcknowledgementAndResponse"));
    assert!(cdv_xml.contains("200"));
    assert!(cdv_xml.contains(&invoice.invoice_number));

    // TODO: Générer les CDV suivants : Reçue (202), Mise à disposition (203), etc.
    // et comparer avec les exemples
}