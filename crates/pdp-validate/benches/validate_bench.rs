use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use pdp_invoice::{UblParser, CiiParser};
use pdp_validate::{XsdValidator, SchematronValidator};
use pdp_validate::xsd::XsdDocumentType;
use pdp_validate::schematron::SchematronType;
use std::path::PathBuf;

fn specs_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../specs")
}

fn load_fixture(path: &str) -> String {
    std::fs::read_to_string(format!("../../tests/fixtures/{}", path))
        .unwrap_or_else(|_| panic!("Fixture {} not found", path))
}

// ============================================================
// XSD Validation
// ============================================================

fn bench_xsd_validation(c: &mut Criterion) {
    let specs = specs_dir();
    let validator = XsdValidator::new(&specs);

    let cii_fixtures = vec![
        ("cii_simple_380", load_fixture("cii/facture_cii_001.xml"), XsdDocumentType::CiiD22B),
        ("cii_avoir_381", load_fixture("cii/avoir_cii_381.xml"), XsdDocumentType::CiiD22B),
        ("cii_remises_multitva", load_fixture("cii/facture_cii_remises_multitva.xml"), XsdDocumentType::CiiD22B),
    ];

    let ubl_fixtures = vec![
        ("ubl_simple_380", load_fixture("ubl/facture_ubl_001.xml"), XsdDocumentType::UblInvoice),
        ("ubl_avoir_381", load_fixture("ubl/facture_ubl_002_avoir.xml"), XsdDocumentType::UblCreditNote),
        ("ubl_remises_multitva", load_fixture("ubl/facture_ubl_remises_multitva.xml"), XsdDocumentType::UblInvoice),
    ];

    let mut group = c.benchmark_group("xsd_validation");

    for (name, xml, doc_type) in &cii_fixtures {
        group.bench_with_input(BenchmarkId::new("validate", name), xml, |b, xml| {
            b.iter(|| validator.validate(black_box(xml), black_box(doc_type)))
        });
    }

    for (name, xml, doc_type) in &ubl_fixtures {
        group.bench_with_input(BenchmarkId::new("validate", name), xml, |b, xml| {
            b.iter(|| validator.validate(black_box(xml), black_box(doc_type)))
        });
    }

    group.finish();
}

// ============================================================
// Schematron Validation
// ============================================================

fn bench_schematron_validation(c: &mut Criterion) {
    let specs = specs_dir();
    let validator = SchematronValidator::new(&specs);

    let cii_xml = load_fixture("cii/facture_cii_001.xml");
    let ubl_xml = load_fixture("ubl/facture_ubl_001.xml");

    let mut group = c.benchmark_group("schematron_validation");

    group.bench_function("en16931_cii", |b| {
        b.iter(|| validator.validate(black_box(&cii_xml), &SchematronType::En16931Cii))
    });

    group.bench_function("en16931_ubl", |b| {
        b.iter(|| validator.validate(black_box(&ubl_xml), &SchematronType::En16931Ubl))
    });

    group.bench_function("brfr_cii", |b| {
        b.iter(|| validator.validate(black_box(&cii_xml), &SchematronType::BrFrCii))
    });

    group.bench_function("brfr_ubl", |b| {
        b.iter(|| validator.validate(black_box(&ubl_xml), &SchematronType::BrFrUbl))
    });

    group.bench_function("full_cii_en16931_brfr", |b| {
        b.iter(|| validator.validate_cii_invoice(black_box(&cii_xml)))
    });

    group.bench_function("full_ubl_en16931_brfr", |b| {
        b.iter(|| validator.validate_ubl_invoice(black_box(&ubl_xml)))
    });

    group.finish();
}

// ============================================================
// Pipeline complet : Parse + XSD + Schematron
// ============================================================

fn bench_full_validation_pipeline(c: &mut Criterion) {
    let specs = specs_dir();
    let xsd = XsdValidator::new(&specs);
    let schematron = SchematronValidator::new(&specs);
    let ubl_parser = UblParser::new();
    let cii_parser = CiiParser::new();

    let ubl_xml = load_fixture("ubl/facture_ubl_001.xml");
    let cii_xml = load_fixture("cii/facture_cii_001.xml");

    let mut group = c.benchmark_group("full_pipeline");

    // Parse + XSD + Schematron (EN16931 + BR-FR) pour CII
    group.bench_function("cii_parse_xsd_schematron", |b| {
        b.iter(|| {
            let _invoice = cii_parser.parse(black_box(&cii_xml)).unwrap();
            let _xsd_report = xsd.validate(&cii_xml, &XsdDocumentType::CiiD22B);
            let _sch_report = schematron.validate_cii_invoice(&cii_xml);
        })
    });

    // Parse + XSD + Schematron (EN16931 + BR-FR) pour UBL
    group.bench_function("ubl_parse_xsd_schematron", |b| {
        b.iter(|| {
            let _invoice = ubl_parser.parse(black_box(&ubl_xml)).unwrap();
            let _xsd_report = xsd.validate(&ubl_xml, &XsdDocumentType::UblInvoice);
            let _sch_report = schematron.validate_ubl_invoice(&ubl_xml);
        })
    });

    // Parse seul (pour comparaison)
    group.bench_function("cii_parse_only", |b| {
        b.iter(|| cii_parser.parse(black_box(&cii_xml)).unwrap())
    });

    group.bench_function("ubl_parse_only", |b| {
        b.iter(|| ubl_parser.parse(black_box(&ubl_xml)).unwrap())
    });

    // XSD seul
    group.bench_function("cii_xsd_only", |b| {
        b.iter(|| xsd.validate(black_box(&cii_xml), &XsdDocumentType::CiiD22B))
    });

    group.bench_function("ubl_xsd_only", |b| {
        b.iter(|| xsd.validate(black_box(&ubl_xml), &XsdDocumentType::UblInvoice))
    });

    group.finish();
}

// ============================================================
// Factur-X Validation (XSD + Schematron on embedded CII XML)
// ============================================================

fn bench_facturx_validation(c: &mut Criterion) {
    let facturx_path = "../../tests/fixtures/facturx/facture_facturx_001.pdf";
    if !std::path::Path::new(facturx_path).exists() {
        eprintln!("Fixture Factur-X non disponible, benchmarks ignorés");
        return;
    }

    let specs = specs_dir();
    let xsd = XsdValidator::new(&specs);
    let schematron = SchematronValidator::new(&specs);

    // Parse Factur-X PDF to extract embedded CII XML
    let pdf_bytes = std::fs::read(facturx_path).unwrap();
    let facturx_parser = pdp_invoice::facturx::FacturXParser::new();
    let invoice = facturx_parser.parse(&pdf_bytes).unwrap();
    let cii_xml = invoice.raw_xml.as_ref().expect("Factur-X doit contenir raw_xml CII");

    let mut group = c.benchmark_group("facturx_validation");

    // XSD Factur-X EN16931
    group.bench_function("xsd_facturx_en16931", |b| {
        b.iter(|| xsd.validate(black_box(cii_xml), &XsdDocumentType::FacturXEN16931))
    });

    // Schematron EN16931-CII (same rules apply to Factur-X embedded CII)
    group.bench_function("schematron_en16931_cii", |b| {
        b.iter(|| schematron.validate(black_box(cii_xml), &SchematronType::En16931Cii))
    });

    // Schematron Factur-X EN16931 (PEPPOL + empty elements + code lists)
    group.bench_function("schematron_facturx_en16931", |b| {
        b.iter(|| schematron.validate(black_box(cii_xml), &SchematronType::FacturXEN16931))
    });

    // Schematron BR-FR CII
    group.bench_function("schematron_brfr_cii", |b| {
        b.iter(|| schematron.validate(black_box(cii_xml), &SchematronType::BrFrCii))
    });

    // Full Factur-X validation: XSD + EN16931 + Factur-X Schematron + BR-FR (parallel)
    group.bench_function("full_facturx_validation", |b| {
        b.iter(|| {
            let _xsd_report = xsd.validate(black_box(cii_xml), &XsdDocumentType::FacturXEN16931);
            let _sch_report = schematron.validate_cii_invoice(black_box(cii_xml));
            let _fx_report = schematron.validate(black_box(cii_xml), &SchematronType::FacturXEN16931);
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_xsd_validation,
    bench_schematron_validation,
    bench_full_validation_pipeline,
    bench_facturx_validation,
);
criterion_main!(benches);
