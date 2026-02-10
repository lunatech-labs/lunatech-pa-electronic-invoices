use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use pdp_invoice::{UblParser, CiiParser};
use pdp_transform::{XsltEngine, convert, convert_to, OutputFormat};
use pdp_core::model::InvoiceFormat;


fn load_fixture(path: &str) -> String {
    std::fs::read_to_string(format!("../../tests/fixtures/{}", path))
        .unwrap_or_else(|_| panic!("Fixture {} not found", path))
}

fn load_fixture_bytes(path: &str) -> Vec<u8> {
    std::fs::read(format!("../../tests/fixtures/{}", path))
        .unwrap_or_else(|_| panic!("Fixture {} not found", path))
}

fn xslt_engine() -> XsltEngine {
    let specs = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../specs");
    XsltEngine::new(&specs)
}

// ============================================================
// XSLT : UBL → CII
// ============================================================

fn bench_ubl_to_cii(c: &mut Criterion) {
    let engine = xslt_engine();
    let fixtures = vec![
        ("simple_380", load_fixture("ubl/facture_ubl_001.xml")),
        ("remises_multitva", load_fixture("ubl/facture_ubl_remises_multitva.xml")),
    ];

    let mut group = c.benchmark_group("transform_ubl_to_cii");
    for (name, xml) in &fixtures {
        group.bench_with_input(BenchmarkId::new("xslt", name), xml, |b, xml| {
            b.iter(|| engine.ubl_to_cii(black_box(xml)).unwrap())
        });
    }
    group.finish();
}

// ============================================================
// XSLT : CII → UBL
// ============================================================

fn bench_cii_to_ubl(c: &mut Criterion) {
    let engine = xslt_engine();
    let fixtures = vec![
        ("simple_380", load_fixture("cii/facture_cii_001.xml")),
        ("remises_multitva", load_fixture("cii/facture_cii_remises_multitva.xml")),
    ];

    let mut group = c.benchmark_group("transform_cii_to_ubl");
    for (name, xml) in &fixtures {
        group.bench_with_input(BenchmarkId::new("xslt", name), xml, |b, xml| {
            b.iter(|| engine.cii_to_ubl(black_box(xml)).unwrap())
        });
    }
    group.finish();
}

// ============================================================
// Pipeline XSLT complet + roundtrip
// ============================================================

fn bench_full_pipeline(c: &mut Criterion) {
    let engine = xslt_engine();
    let ubl_xml = load_fixture("ubl/facture_ubl_001.xml");
    let cii_xml = load_fixture("cii/facture_cii_001.xml");

    let mut group = c.benchmark_group("transform_full_pipeline");

    group.bench_function("xslt_ubl_to_cii", |b| {
        b.iter(|| engine.ubl_to_cii(black_box(&ubl_xml)).unwrap())
    });

    group.bench_function("xslt_cii_to_ubl", |b| {
        b.iter(|| engine.cii_to_ubl(black_box(&cii_xml)).unwrap())
    });

    group.bench_function("xslt_roundtrip_ubl_cii_ubl", |b| {
        b.iter(|| {
            let cii = engine.ubl_to_cii(black_box(&ubl_xml)).unwrap();
            engine.cii_to_ubl(&cii).unwrap()
        })
    });

    group.bench_function("xslt_roundtrip_cii_ubl_cii", |b| {
        b.iter(|| {
            let ubl = engine.cii_to_ubl(black_box(&cii_xml)).unwrap();
            engine.ubl_to_cii(&ubl).unwrap()
        })
    });

    group.finish();
}

// ============================================================
// Convert API : UBL/CII → CII/UBL
// ============================================================

fn bench_convert_api(c: &mut Criterion) {
    let ubl_parser = UblParser::new();
    let cii_parser = CiiParser::new();
    let ubl_xml = load_fixture("ubl/facture_ubl_001.xml");
    let cii_xml = load_fixture("cii/facture_cii_001.xml");
    let ubl_inv = ubl_parser.parse(&ubl_xml).unwrap();
    let cii_inv = cii_parser.parse(&cii_xml).unwrap();

    let mut group = c.benchmark_group("transform_convert_api");

    group.bench_function("convert_ubl_to_cii", |b| {
        b.iter(|| convert(black_box(&ubl_inv), InvoiceFormat::CII).unwrap())
    });
    group.bench_function("convert_cii_to_ubl", |b| {
        b.iter(|| convert(black_box(&cii_inv), InvoiceFormat::UBL).unwrap())
    });

    group.finish();
}

// ============================================================
// UBL/CII → PDF (FOP pipeline)
// ============================================================

fn bench_to_pdf(c: &mut Criterion) {
    let ubl_parser = UblParser::new();
    let cii_parser = CiiParser::new();
    let ubl_xml = load_fixture("ubl/facture_ubl_001.xml");
    let cii_xml = load_fixture("cii/facture_cii_001.xml");
    let ubl_inv = ubl_parser.parse(&ubl_xml).unwrap();
    let cii_inv = cii_parser.parse(&cii_xml).unwrap();

    let mut group = c.benchmark_group("transform_to_pdf");
    group.sample_size(10);

    group.bench_function("ubl_to_pdf", |b| {
        b.iter(|| convert_to(black_box(&ubl_inv), OutputFormat::PDF).unwrap())
    });

    group.bench_function("cii_to_pdf", |b| {
        b.iter(|| convert_to(black_box(&cii_inv), OutputFormat::PDF).unwrap())
    });

    group.finish();
}

// ============================================================
// UBL/CII → Factur-X (XSLT + FOP + lopdf)
// ============================================================

fn bench_to_facturx(c: &mut Criterion) {
    let ubl_parser = UblParser::new();
    let cii_parser = CiiParser::new();
    let ubl_xml = load_fixture("ubl/facture_ubl_001.xml");
    let cii_xml = load_fixture("cii/facture_cii_001.xml");
    let ubl_inv = ubl_parser.parse(&ubl_xml).unwrap();
    let cii_inv = cii_parser.parse(&cii_xml).unwrap();

    let mut group = c.benchmark_group("transform_to_facturx");
    group.sample_size(10);

    group.bench_function("ubl_to_facturx", |b| {
        b.iter(|| convert_to(black_box(&ubl_inv), OutputFormat::FacturX).unwrap())
    });

    group.bench_function("cii_to_facturx", |b| {
        b.iter(|| convert_to(black_box(&cii_inv), OutputFormat::FacturX).unwrap())
    });

    group.finish();
}

// ============================================================
// Factur-X → CII / UBL / PDF
// ============================================================

fn bench_facturx_conversions(c: &mut Criterion) {
    let facturx_path = "../../tests/fixtures/facturx/facture_facturx_001.pdf";
    if !std::path::Path::new(facturx_path).exists() {
        eprintln!("Fixture Factur-X non disponible, benchmarks Factur-X ignorés");
        return;
    }

    let pdf_bytes = load_fixture_bytes("facturx/facture_facturx_001.pdf");
    let facturx_parser = pdp_invoice::facturx::FacturXParser::new();
    let facturx_inv = facturx_parser.parse(&pdf_bytes).unwrap();

    let mut group = c.benchmark_group("transform_facturx");

    group.bench_function("facturx_to_cii", |b| {
        b.iter(|| convert_to(black_box(&facturx_inv), OutputFormat::CII).unwrap())
    });

    group.bench_function("facturx_to_ubl", |b| {
        b.iter(|| convert_to(black_box(&facturx_inv), OutputFormat::UBL).unwrap())
    });

    group.bench_function("facturx_to_pdf", |b| {
        b.iter(|| convert_to(black_box(&facturx_inv), OutputFormat::PDF).unwrap())
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_ubl_to_cii,
    bench_cii_to_ubl,
    bench_full_pipeline,
    bench_convert_api,
    bench_to_pdf,
    bench_to_facturx,
    bench_facturx_conversions,
);
criterion_main!(benches);
