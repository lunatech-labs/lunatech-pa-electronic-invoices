use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use pdp_invoice::{UblParser, CiiParser, FacturXParser, detect_format};

fn load_fixture(path: &str) -> String {
    std::fs::read_to_string(format!("../../tests/fixtures/{}", path))
        .unwrap_or_else(|_| panic!("Fixture {} not found", path))
}

fn load_fixture_bytes(path: &str) -> Vec<u8> {
    std::fs::read(format!("../../tests/fixtures/{}", path))
        .unwrap_or_else(|_| panic!("Fixture {} not found", path))
}

fn bench_detect_format(c: &mut Criterion) {
    let ubl_bytes = load_fixture("ubl/facture_ubl_001.xml").into_bytes();
    let cii_bytes = load_fixture("cii/facture_cii_001.xml").into_bytes();

    let mut group = c.benchmark_group("invoice_detect_format");
    group.bench_function("detect_ubl", |b| {
        b.iter(|| detect_format(black_box(&ubl_bytes)))
    });
    group.bench_function("detect_cii", |b| {
        b.iter(|| detect_format(black_box(&cii_bytes)))
    });
    group.finish();
}

fn bench_parse_ubl(c: &mut Criterion) {
    let fixtures = vec![
        ("simple_380", load_fixture("ubl/facture_ubl_001.xml")),
        ("avoir_381", load_fixture("ubl/facture_ubl_002_avoir.xml")),
        ("remises_multitva", load_fixture("ubl/facture_ubl_remises_multitva.xml")),
        ("marketplace_a8", load_fixture("ubl/facture_ubl_marketplace_a8.xml")),
    ];

    let parser = UblParser::new();

    let mut group = c.benchmark_group("ubl_parse");
    for (name, xml) in &fixtures {
        group.bench_with_input(BenchmarkId::new("parse", name), xml, |b, xml| {
            b.iter(|| parser.parse(black_box(xml)).unwrap())
        });
    }
    group.finish();
}

fn bench_parse_cii(c: &mut Criterion) {
    let fixtures = vec![
        ("simple_380", load_fixture("cii/facture_cii_001.xml")),
        ("avoir_381", load_fixture("cii/avoir_cii_381.xml")),
        ("remises_multitva", load_fixture("cii/facture_cii_remises_multitva.xml")),
        ("marketplace_a8", load_fixture("cii/facture_cii_marketplace_a8.xml")),
    ];

    let parser = CiiParser::new();

    let mut group = c.benchmark_group("cii_parse");
    for (name, xml) in &fixtures {
        group.bench_with_input(BenchmarkId::new("parse", name), xml, |b, xml| {
            b.iter(|| parser.parse(black_box(xml)).unwrap())
        });
    }
    group.finish();
}

// ============================================================
// Factur-X (PDF) parsing
// ============================================================

fn bench_parse_facturx(c: &mut Criterion) {
    let facturx_path = "../../tests/fixtures/facturx/facture_facturx_001.pdf";
    if !std::path::Path::new(facturx_path).exists() {
        eprintln!("Fixture Factur-X non disponible, benchmarks ignorés");
        return;
    }

    let fixtures = vec![
        ("facture_001", load_fixture_bytes("facturx/facture_facturx_001.pdf")),
    ];

    // Ajouter les autres fixtures si elles existent
    let avoir_path = "../../tests/fixtures/facturx/avoir_facturx_381.pdf";
    let rect_path = "../../tests/fixtures/facturx/facture_rectificative_facturx_384.pdf";

    let mut all_fixtures = fixtures;
    if std::path::Path::new(avoir_path).exists() {
        all_fixtures.push(("avoir_381", load_fixture_bytes("facturx/avoir_facturx_381.pdf")));
    }
    if std::path::Path::new(rect_path).exists() {
        all_fixtures.push(("rectificative_384", load_fixture_bytes("facturx/facture_rectificative_facturx_384.pdf")));
    }

    let parser = FacturXParser::new();

    let mut group = c.benchmark_group("facturx_parse");
    for (name, pdf) in &all_fixtures {
        group.bench_with_input(BenchmarkId::new("parse", name), pdf, |b, pdf| {
            b.iter(|| parser.parse(black_box(pdf)).unwrap())
        });
    }
    group.finish();
}

// ============================================================
// Detect format (UBL, CII, Factur-X PDF)
// ============================================================

fn bench_detect_format_all(c: &mut Criterion) {
    let facturx_path = "../../tests/fixtures/facturx/facture_facturx_001.pdf";
    if !std::path::Path::new(facturx_path).exists() {
        return;
    }

    let facturx_bytes = load_fixture_bytes("facturx/facture_facturx_001.pdf");

    let mut group = c.benchmark_group("invoice_detect_format");
    group.bench_function("detect_facturx_pdf", |b| {
        b.iter(|| detect_format(black_box(&facturx_bytes)))
    });
    group.finish();
}

// ============================================================
// Throughput (UBL, CII, Factur-X)
// ============================================================

fn bench_parse_throughput(c: &mut Criterion) {
    let ubl_xml = load_fixture("ubl/facture_ubl_001.xml");
    let cii_xml = load_fixture("cii/facture_cii_001.xml");
    let ubl_parser = UblParser::new();
    let cii_parser = CiiParser::new();

    let mut group = c.benchmark_group("invoice_throughput");

    group.throughput(criterion::Throughput::Bytes(ubl_xml.len() as u64));
    group.bench_function("ubl_simple", |b| {
        b.iter(|| ubl_parser.parse(black_box(&ubl_xml)).unwrap())
    });

    group.throughput(criterion::Throughput::Bytes(cii_xml.len() as u64));
    group.bench_function("cii_simple", |b| {
        b.iter(|| cii_parser.parse(black_box(&cii_xml)).unwrap())
    });

    let facturx_path = "../../tests/fixtures/facturx/facture_facturx_001.pdf";
    if std::path::Path::new(facturx_path).exists() {
        let facturx_pdf = load_fixture_bytes("facturx/facture_facturx_001.pdf");
        let facturx_parser = FacturXParser::new();
        group.throughput(criterion::Throughput::Bytes(facturx_pdf.len() as u64));
        group.bench_function("facturx_simple", |b| {
            b.iter(|| facturx_parser.parse(black_box(&facturx_pdf)).unwrap())
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_detect_format,
    bench_detect_format_all,
    bench_parse_ubl,
    bench_parse_cii,
    bench_parse_facturx,
    bench_parse_throughput,
);
criterion_main!(benches);
