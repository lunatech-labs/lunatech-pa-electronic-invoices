use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use pdp_core::endpoint::{Consumer, FileEndpoint};
use std::io::Write;

/// Crée un tar.gz en mémoire contenant N fichiers XML identiques
fn create_tar_gz(count: usize, xml: &[u8]) -> Vec<u8> {
    use flate2::write::GzEncoder;
    use flate2::Compression;
    let buf = Vec::new();
    let enc = GzEncoder::new(buf, Compression::default());
    let mut tar = tar::Builder::new(enc);
    for i in 0..count {
        let name = format!("facture_{:04}.xml", i);
        let mut header = tar::Header::new_gnu();
        header.set_size(xml.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        tar.append_data(&mut header, &name, xml).unwrap();
    }
    let enc = tar.into_inner().unwrap();
    enc.finish().unwrap()
}

/// Crée un ZIP en mémoire contenant N fichiers XML identiques
fn create_zip(count: usize, xml: &[u8]) -> Vec<u8> {
    let buf = std::io::Cursor::new(Vec::new());
    let mut zip = zip::ZipWriter::new(buf);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    for i in 0..count {
        let name = format!("facture_{:04}.xml", i);
        zip.start_file(&name, options).unwrap();
        zip.write_all(xml).unwrap();
    }
    zip.finish().unwrap().into_inner()
}

/// XML CII minimal pour les benchmarks
const SAMPLE_XML: &[u8] = br#"<?xml version="1.0" encoding="UTF-8"?>
<rsm:CrossIndustryInvoice xmlns:rsm="urn:un:unece:uncefact:data:standard:CrossIndustryInvoice:100">
  <rsm:ExchangedDocumentContext>
    <ram:GuidelineSpecifiedDocumentContextParameter xmlns:ram="urn:un:unece:uncefact:data:standard:ReusableAggregateBusinessInformationEntity:100">
      <ram:ID>urn:cen.eu:en16931:2017</ram:ID>
    </ram:GuidelineSpecifiedDocumentContextParameter>
  </rsm:ExchangedDocumentContext>
  <rsm:ExchangedDocument xmlns:ram="urn:un:unece:uncefact:data:standard:ReusableAggregateBusinessInformationEntity:100">
    <ram:ID>FA-2025-BENCH</ram:ID>
    <ram:TypeCode>380</ram:TypeCode>
  </rsm:ExchangedDocument>
</rsm:CrossIndustryInvoice>"#;

fn bench_poll_tar_gz(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut group = c.benchmark_group("archive_poll_tar_gz");

    for count in [1, 10, 50, 100] {
        let dir = tempfile::tempdir().unwrap();
        let archive = create_tar_gz(count, SAMPLE_XML);
        std::fs::write(dir.path().join("lot.tar.gz"), &archive).unwrap();
        let dir_path = dir.path().to_str().unwrap().to_string();

        group.bench_with_input(
            BenchmarkId::new("files", count),
            &count,
            |b, _| {
                let endpoint = FileEndpoint::input("bench", &dir_path).with_stable_delay(0);
                b.iter(|| {
                    let exchanges = rt.block_on(endpoint.poll()).unwrap();
                    assert_eq!(exchanges.len(), count);
                    black_box(&exchanges);
                });
            },
        );
    }
    group.finish();
}

fn bench_poll_zip(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut group = c.benchmark_group("archive_poll_zip");

    for count in [1, 10, 50, 100] {
        let dir = tempfile::tempdir().unwrap();
        let archive = create_zip(count, SAMPLE_XML);
        std::fs::write(dir.path().join("lot.zip"), &archive).unwrap();
        let dir_path = dir.path().to_str().unwrap().to_string();

        group.bench_with_input(
            BenchmarkId::new("files", count),
            &count,
            |b, _| {
                let endpoint = FileEndpoint::input("bench", &dir_path).with_stable_delay(0);
                b.iter(|| {
                    let exchanges = rt.block_on(endpoint.poll()).unwrap();
                    assert_eq!(exchanges.len(), count);
                    black_box(&exchanges);
                });
            },
        );
    }
    group.finish();
}

fn bench_poll_mixed(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut group = c.benchmark_group("archive_poll_mixed");

    // 5 XML directs + 1 tar.gz(10) + 1 zip(10) = 25 exchanges
    let dir = tempfile::tempdir().unwrap();
    for i in 0..5 {
        std::fs::write(dir.path().join(format!("direct_{}.xml", i)), SAMPLE_XML).unwrap();
    }
    let tar_gz = create_tar_gz(10, SAMPLE_XML);
    std::fs::write(dir.path().join("batch.tar.gz"), &tar_gz).unwrap();
    let zip_data = create_zip(10, SAMPLE_XML);
    std::fs::write(dir.path().join("lot.zip"), &zip_data).unwrap();
    let dir_path = dir.path().to_str().unwrap().to_string();

    group.bench_function("5xml_10tar_10zip", |b| {
        let endpoint = FileEndpoint::input("bench", &dir_path).with_stable_delay(0);
        b.iter(|| {
            let exchanges = rt.block_on(endpoint.poll()).unwrap();
            assert_eq!(exchanges.len(), 25);
            black_box(&exchanges);
        });
    });
    group.finish();
}

fn bench_poll_plain_files(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut group = c.benchmark_group("archive_poll_plain");

    for count in [1, 10, 50, 100] {
        let dir = tempfile::tempdir().unwrap();
        for i in 0..count {
            std::fs::write(
                dir.path().join(format!("facture_{:04}.xml", i)),
                SAMPLE_XML,
            ).unwrap();
        }
        let dir_path = dir.path().to_str().unwrap().to_string();

        group.bench_with_input(
            BenchmarkId::new("files", count),
            &count,
            |b, _| {
                let endpoint = FileEndpoint::input("bench", &dir_path).with_stable_delay(0);
                b.iter(|| {
                    let exchanges = rt.block_on(endpoint.poll()).unwrap();
                    assert_eq!(exchanges.len(), count);
                    black_box(&exchanges);
                });
            },
        );
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_poll_plain_files,
    bench_poll_tar_gz,
    bench_poll_zip,
    bench_poll_mixed,
);
criterion_main!(benches);
