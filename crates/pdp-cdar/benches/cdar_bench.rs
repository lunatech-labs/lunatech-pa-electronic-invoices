use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use pdp_cdar::{CdarParser, CdarGenerator, CdarValidationError, StatusReasonCode};
use pdp_core::model::{InvoiceData, InvoiceFormat};

fn make_test_invoice() -> InvoiceData {
    let mut inv = InvoiceData::new("F202500003".to_string(), InvoiceFormat::CII);
    inv.issue_date = Some("2025-07-01".to_string());
    inv.seller_name = Some("VENDEUR SAS".to_string());
    inv.seller_siret = Some("10000000900014".to_string());
    inv.seller_vat_id = Some("FR32100000009".to_string());
    inv.seller_endpoint_id = Some("100000009".to_string());
    inv.buyer_name = Some("ACHETEUR SARL".to_string());
    inv.buyer_siret = Some("20000000800015".to_string());
    inv.buyer_vat_id = Some("FR45200000008".to_string());
    inv.buyer_endpoint_id = Some("200000008".to_string());
    inv
}

fn make_test_errors() -> Vec<CdarValidationError> {
    vec![CdarValidationError {
        rule_id: "BR-01".to_string(),
        severity: "error".to_string(),
        location: None,
        message: "Facture en doublon".to_string(),
        reason_code: Some(StatusReasonCode::Doublon),
    }]
}

fn load_fixture(name: &str) -> String {
    std::fs::read_to_string(format!("../../tests/fixtures/cdar/{}", name))
        .unwrap_or_else(|_| panic!("Fixture {} not found", name))
}

fn bench_parse(c: &mut Criterion) {
    let fixtures = vec![
        ("cdv_200_deposee", load_fixture("cdv_200_deposee.xml")),
        ("cdv_207_litige", load_fixture("cdv_207_litige.xml")),
        ("cdv_212_encaissee", load_fixture("cdv_212_encaissee.xml")),
        ("cdv_213_rejetee", load_fixture("cdv_213_rejetee.xml")),
        ("cdv_501_irrecevable", load_fixture("cdv_501_irrecevable.xml")),
    ];

    let parser = CdarParser::new();

    let mut group = c.benchmark_group("cdar_parse");
    for (name, xml) in &fixtures {
        group.bench_with_input(BenchmarkId::new("parse", name), xml, |b, xml| {
            b.iter(|| parser.parse(black_box(xml)).unwrap())
        });
    }
    group.finish();
}

fn bench_generate(c: &mut Criterion) {
    let gen = CdarGenerator::new("100000009", "PDP Test");
    let inv = make_test_invoice();

    let mut group = c.benchmark_group("cdar_generate");

    group.bench_function("deposee_200", |b| {
        b.iter(|| gen.generate_deposee(black_box(&inv), "380"))
    });
    let errors = make_test_errors();
    group.bench_function("rejetee_213", |b| {
        b.iter(|| gen.generate_rejetee(black_box(&inv), "380", errors.clone()))
    });
    group.bench_function("encaissee_212", |b| {
        b.iter(|| gen.generate_encaissee(black_box(&inv), "380", 1200.00))
    });
    group.bench_function("irrecevable_501", |b| {
        b.iter(|| gen.generate_irrecevable(black_box(&inv), StatusReasonCode::IrrSyntax, "Erreur de syntaxe XML"))
    });

    group.finish();
}

fn bench_generate_serialize(c: &mut Criterion) {
    let gen = CdarGenerator::new("100000009", "PDP Test");
    let inv = make_test_invoice();

    let mut group = c.benchmark_group("cdar_generate_serialize");

    group.bench_function("deposee_200_to_xml", |b| {
        b.iter(|| {
            let cdv = gen.generate_deposee(black_box(&inv), "380");
            gen.to_xml(&cdv).unwrap()
        })
    });
    let errors = make_test_errors();
    group.bench_function("rejetee_213_to_xml", |b| {
        b.iter(|| {
            let cdv = gen.generate_rejetee(black_box(&inv), "380", errors.clone());
            gen.to_xml(&cdv).unwrap()
        })
    });

    group.finish();
}

fn bench_roundtrip(c: &mut Criterion) {
    let gen = CdarGenerator::new("100000009", "PDP Test");
    let parser = CdarParser::new();
    let inv = make_test_invoice();

    let mut group = c.benchmark_group("cdar_roundtrip");

    group.bench_function("generate_serialize_parse_200", |b| {
        b.iter(|| {
            let cdv = gen.generate_deposee(black_box(&inv), "380");
            let xml = gen.to_xml(&cdv).unwrap();
            parser.parse(&xml).unwrap()
        })
    });
    let errors = make_test_errors();
    group.bench_function("generate_serialize_parse_213", |b| {
        b.iter(|| {
            let cdv = gen.generate_rejetee(black_box(&inv), "380", errors.clone());
            let xml = gen.to_xml(&cdv).unwrap();
            parser.parse(&xml).unwrap()
        })
    });

    group.finish();
}

fn bench_parse_throughput(c: &mut Criterion) {
    let xml = load_fixture("cdv_200_deposee.xml");
    let parser = CdarParser::new();
    let size = xml.len() as u64;

    let mut group = c.benchmark_group("cdar_throughput");
    group.throughput(criterion::Throughput::Bytes(size));
    group.bench_function("parse_deposee", |b| {
        b.iter(|| parser.parse(black_box(&xml)).unwrap())
    });

    let xml_big = load_fixture("cdv_212_encaissee.xml");
    let size_big = xml_big.len() as u64;
    group.throughput(criterion::Throughput::Bytes(size_big));
    group.bench_function("parse_encaissee_with_characteristics", |b| {
        b.iter(|| parser.parse(black_box(&xml_big)).unwrap())
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_parse,
    bench_generate,
    bench_generate_serialize,
    bench_roundtrip,
    bench_parse_throughput,
);
criterion_main!(benches);
