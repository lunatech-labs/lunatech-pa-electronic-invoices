//! Script de génération de fixtures Factur-X (PDF/A-3a avec XML CII embarqué)
//!
//! Usage: cargo test -p pdp-invoice --test generate_facturx_fixtures -- --ignored --nocapture
//!
//! Ce script utilise le pipeline complet FacturXGenerator (FOP + lopdf + qpdf)
//! pour produire des PDF/A-3a conformes, validés par veraPDF.
//!
//! Prérequis: fop, saxon, qpdf installés (brew install fop qpdf)

/// Génère un PDF Factur-X conforme PDF/A-3a via le pipeline complet.
/// Retourne les bytes du PDF.
fn generate_facturx(cii_xml_path: &str) -> (Vec<u8>, String) {
    let specs = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../specs");
    let gen = pdp_transform::facturx_generator::FacturXGenerator::from_specs_dir(&specs);

    let xml = std::fs::read_to_string(cii_xml_path)
        .unwrap_or_else(|_| panic!("Fixture CII introuvable: {}", cii_xml_path));
    let parser = pdp_invoice::cii::CiiParser::new();
    let invoice = parser.parse(&xml).expect("Parsing CII échoué");

    let result = gen.generate(&invoice).expect("Génération Factur-X échouée");
    let number = invoice.invoice_number.clone();
    (result.pdf, number)
}

#[test]
#[ignore]
fn generate_facturx_standard() {
    let (pdf, number) = generate_facturx("../../tests/fixtures/cii/facture_cii_001.xml");
    let out_path = "../../tests/fixtures/facturx/facture_facturx_001.pdf";
    std::fs::create_dir_all("../../tests/fixtures/facturx").unwrap();
    std::fs::write(out_path, &pdf).unwrap();
    println!("Fixture Factur-X générée: {} ({} Ko)", out_path, pdf.len() / 1024);

    // Vérifier que le PDF est lisible par le parser Factur-X
    let parser = pdp_invoice::facturx::FacturXParser::new();
    let pdf_bytes = std::fs::read(out_path).unwrap();
    let invoice = parser.parse(&pdf_bytes).expect("Le parser Factur-X doit pouvoir lire le PDF généré");
    assert_eq!(invoice.invoice_number, number);
    assert_eq!(invoice.source_format, pdp_core::model::InvoiceFormat::FacturX);
}

#[test]
#[ignore]
fn generate_facturx_avoir() {
    let (pdf, _) = generate_facturx("../../tests/fixtures/cii/avoir_cii_381.xml");
    let out_path = "../../tests/fixtures/facturx/avoir_facturx_381.pdf";
    std::fs::create_dir_all("../../tests/fixtures/facturx").unwrap();
    std::fs::write(out_path, &pdf).unwrap();
    println!("Fixture Factur-X avoir générée: {} ({} Ko)", out_path, pdf.len() / 1024);
}

#[test]
#[ignore]
fn generate_facturx_rectificative() {
    let (pdf, _) = generate_facturx("../../tests/fixtures/cii/facture_rectificative_cii_384.xml");
    let out_path = "../../tests/fixtures/facturx/facture_rectificative_facturx_384.pdf";
    std::fs::create_dir_all("../../tests/fixtures/facturx").unwrap();
    std::fs::write(out_path, &pdf).unwrap();
    println!("Fixture Factur-X rectificative générée: {} ({} Ko)", out_path, pdf.len() / 1024);
}
