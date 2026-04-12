//! Benchmarks gros volumes — pipeline complet, batch processing, scalabilité.
//!
//! Mesure les performances end-to-end sur des volumes réalistes :
//! - Parsing batch de N factures (10, 100, 1000)
//! - Pipeline complet : parse → CDV 200 → sérialisation XML
//! - Réception HTTP simulée : multipart → channel
//! - Throughput en factures/seconde

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use pdp_cdar::{CdarGenerator, CdarParser};
use pdp_core::model::{InvoiceData, InvoiceFormat};
use pdp_invoice::cii::CiiParser;
use pdp_invoice::ubl::UblParser;

fn load_fixture(name: &str) -> Vec<u8> {
    let path = format!("{}/../../tests/fixtures/{}", env!("CARGO_MANIFEST_DIR"), name);
    std::fs::read(&path).unwrap_or_else(|_| panic!("Fixture {} introuvable", path))
}

fn make_invoice(num: usize) -> InvoiceData {
    let mut inv = InvoiceData::new(format!("FA-BENCH-{:06}", num), InvoiceFormat::CII);
    inv.invoice_type_code = Some("380".to_string());
    inv.issue_date = Some("2025-07-01".to_string());
    inv.seller_siret = Some("10000000900014".to_string());
    inv.seller_name = Some("VENDEUR SAS".to_string());
    inv.buyer_siret = Some("20000000800015".to_string());
    inv.buyer_name = Some("ACHETEUR SARL".to_string());
    inv.currency = Some("EUR".to_string());
    inv.total_without_vat = Some(10000.0);
    inv.total_tax = Some(2000.0);
    inv
}

// ============================================================
// 1. Parsing batch — N factures CII et UBL
// ============================================================

fn bench_batch_parse_cii(c: &mut Criterion) {
    let xml = String::from_utf8(load_fixture("cii/facture_cii_001.xml")).unwrap();
    let parser = CiiParser::new();

    let mut group = c.benchmark_group("batch_parse_cii");
    for count in [10, 100, 1000] {
        group.throughput(Throughput::Elements(count));
        group.bench_with_input(BenchmarkId::new("parse", count), &count, |b, &n| {
            b.iter(|| {
                for _ in 0..n {
                    let _ = parser.parse(black_box(&xml)).unwrap();
                }
            })
        });
    }
    group.finish();
}

fn bench_batch_parse_ubl(c: &mut Criterion) {
    let xml = String::from_utf8(load_fixture("ubl/facture_ubl_001.xml")).unwrap();
    let parser = UblParser::new();

    let mut group = c.benchmark_group("batch_parse_ubl");
    for count in [10, 100, 1000] {
        group.throughput(Throughput::Elements(count));
        group.bench_with_input(BenchmarkId::new("parse", count), &count, |b, &n| {
            b.iter(|| {
                for _ in 0..n {
                    let _ = parser.parse(black_box(&xml)).unwrap();
                }
            })
        });
    }
    group.finish();
}

// ============================================================
// 2. Pipeline complet : parse CII → CDV 200 → XML
// ============================================================

fn bench_pipeline_parse_to_cdv(c: &mut Criterion) {
    let xml = String::from_utf8(load_fixture("cii/facture_cii_001.xml")).unwrap();
    let cii_parser = CiiParser::new();
    let cdv_gen = CdarGenerator::new("100000009", "PDP Bench");

    let mut group = c.benchmark_group("pipeline_parse_to_cdv");

    // Pipeline unitaire
    group.bench_function("single_cii_to_cdv200", |b| {
        b.iter(|| {
            let invoice = cii_parser.parse(black_box(&xml)).unwrap();
            let type_code = invoice.invoice_type_code.as_deref().unwrap_or("380");
            let cdv = cdv_gen.generate_deposee(&invoice, type_code);
            cdv_gen.to_xml(&cdv).unwrap()
        })
    });

    // Pipeline batch
    for count in [10, 100] {
        group.throughput(Throughput::Elements(count));
        group.bench_with_input(BenchmarkId::new("batch_cii_to_cdv200", count), &count, |b, &n| {
            b.iter(|| {
                for _ in 0..n {
                    let invoice = cii_parser.parse(black_box(&xml)).unwrap();
                    let type_code = invoice.invoice_type_code.as_deref().unwrap_or("380");
                    let cdv = cdv_gen.generate_deposee(&invoice, type_code);
                    let _ = cdv_gen.to_xml(&cdv).unwrap();
                }
            })
        });
    }

    group.finish();
}

// ============================================================
// 3. CDV batch generation + sérialisation
// ============================================================

fn bench_batch_cdv_generation(c: &mut Criterion) {
    let cdv_gen = CdarGenerator::new("100000009", "PDP Bench");
    let cdv_parser = CdarParser::new();

    let mut group = c.benchmark_group("batch_cdv_generation");

    for count in [10, 100, 1000] {
        group.throughput(Throughput::Elements(count));

        // Génération + sérialisation
        group.bench_with_input(BenchmarkId::new("generate_serialize", count), &count, |b, &n| {
            b.iter(|| {
                for i in 0..n {
                    let inv = make_invoice(i as usize);
                    let cdv = cdv_gen.generate_deposee(&inv, "380");
                    let _ = cdv_gen.to_xml(black_box(&cdv)).unwrap();
                }
            })
        });

        // Roundtrip complet : generate → serialize → parse
        group.bench_with_input(BenchmarkId::new("roundtrip", count), &count, |b, &n| {
            b.iter(|| {
                for i in 0..n {
                    let inv = make_invoice(i as usize);
                    let cdv = cdv_gen.generate_deposee(&inv, "380");
                    let xml = cdv_gen.to_xml(&cdv).unwrap();
                    let _ = cdv_parser.parse(black_box(&xml)).unwrap();
                }
            })
        });
    }

    group.finish();
}

// ============================================================
// 4. Scalabilité mémoire — factures de tailles variées
// ============================================================

fn bench_scaling_by_size(c: &mut Criterion) {
    let cii_parser = CiiParser::new();
    let small = String::from_utf8(load_fixture("cii/facture_cii_001.xml")).unwrap();

    let mut group = c.benchmark_group("scaling_by_invoice_size");

    // Facture standard
    let size_small = small.len() as u64;
    group.throughput(Throughput::Bytes(size_small));
    group.bench_function("standard_cii", |b| {
        b.iter(|| cii_parser.parse(black_box(&small)).unwrap())
    });

    // Facture avec contenu répété (simule facture volumineuse)
    // On insère des lignes supplémentaires dans le XML
    let large_xml = small.replace(
        "</ram:IncludedSupplyChainTradeLineItem>",
        &format!("{}</ram:IncludedSupplyChainTradeLineItem>",
            (0..50).map(|i| format!(
                r#"<ram:IncludedSupplyChainTradeLineItem>
                    <ram:AssociatedDocumentLineDocument><ram:LineID>{}</ram:LineID></ram:AssociatedDocumentLineDocument>
                    <ram:SpecifiedTradeProduct><ram:Name>Article bench {}</ram:Name></ram:SpecifiedTradeProduct>
                    <ram:SpecifiedLineTradeAgreement><ram:NetPriceProductTradePrice><ram:ChargeAmount>10.00</ram:ChargeAmount></ram:NetPriceProductTradePrice></ram:SpecifiedLineTradeAgreement>
                    <ram:SpecifiedLineTradeDelivery><ram:BilledQuantity unitCode="C62">1</ram:BilledQuantity></ram:SpecifiedLineTradeDelivery>
                    <ram:SpecifiedLineTradeSettlement><ram:ApplicableTradeTax><ram:TypeCode>VAT</ram:TypeCode><ram:CategoryCode>S</ram:CategoryCode><ram:RateApplicablePercent>20</ram:RateApplicablePercent></ram:ApplicableTradeTax><ram:SpecifiedTradeSettlementLineMonetarySummation><ram:LineTotalAmount>10.00</ram:LineTotalAmount></ram:SpecifiedTradeSettlementLineMonetarySummation></ram:SpecifiedLineTradeSettlement>
                </ram:IncludedSupplyChainTradeLineItem>"#, i + 100, i
            )).collect::<String>()
        ),
    );

    // Seulement si le remplacement a fonctionné (la facture contient le pattern)
    if large_xml.len() > small.len() {
        let size_large = large_xml.len() as u64;
        group.throughput(Throughput::Bytes(size_large));
        group.bench_function("large_50_lines_cii", |b| {
            b.iter(|| cii_parser.parse(black_box(&large_xml)).unwrap())
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_batch_parse_cii,
    bench_batch_parse_ubl,
    bench_pipeline_parse_to_cdv,
    bench_batch_cdv_generation,
    bench_scaling_by_size,
);
criterion_main!(benches);
