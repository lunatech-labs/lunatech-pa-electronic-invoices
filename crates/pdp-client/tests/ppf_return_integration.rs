//! Tests d'intégration : ingestion des flux retour PPF (PPF → PDP) via SAS SFTP.
//!
//! Ces tests simulent l'ingestion d'enveloppes `tar.gz` PPF retour sans dépendre
//! d'un serveur SFTP réel : on construit une enveloppe en mémoire, on la
//! décompresse et on l'annote avec `PpfReturnConsumer.annotate()`, puis on la
//! fait passer par les processors aval (DocumentTypeRouter, CdvReceptionProcessor).
//!
//! Couvrent :
//! - Round-trip enveloppe F6 CDV (200 Reçu PPF, 501 Irrecevable)
//! - Routage du code interface depuis le nom de l'enveloppe
//! - Mapping de fallback par chemin SAS
//! - Annotation `source.protocol = ppf-sftp-return` et `flow.syntax = CDAR`

use std::collections::HashMap;

use pdp_client::ppf::{build_tar_gz, flux_envelope_name, FluxFile};
use pdp_client::{CodeInterface, PpfReturnConsumer, PpfReturnConsumerConfig};
use pdp_core::archive::read_tar_gz;
use pdp_core::exchange::Exchange;
use pdp_core::model::FlowStatus;
use pdp_core::processor::Processor;
use pdp_sftp::SftpConfig;

/// Construit une enveloppe `tar.gz` PPF avec son nommage spec, contenant le XML donné.
/// Retourne (nom_enveloppe, contenu_tar_gz).
fn make_envelope(
    code_interface: CodeInterface,
    code_app: &str,
    sequence: &str,
    inner_filename: &str,
    inner_content: &[u8],
) -> (String, Vec<u8>) {
    let envelope_name =
        flux_envelope_name(code_interface, code_app, sequence).expect("envelope name");
    let files = vec![FluxFile {
        filename: inner_filename.to_string(),
        content: inner_content.to_vec(),
    }];
    let tar_gz = build_tar_gz(&files).expect("tar.gz build");
    (envelope_name, tar_gz)
}

/// Simule la chaîne ingestion SFTP retrait → SftpConsumer (décompression) → annotate().
/// Renvoie l'exchange annoté tel qu'il serait injecté dans le pipeline réception.
fn ingest_return_envelope(
    consumer: &PpfReturnConsumer,
    depot_path: &str,
    envelope_name: &str,
    tar_gz: &[u8],
) -> Vec<Exchange> {
    let extracted = read_tar_gz(tar_gz).expect("decompress tar.gz");
    extracted
        .into_iter()
        .map(|entry| {
            let mut exchange =
                Exchange::new(entry.content).with_filename(&entry.filename);
            // SftpConsumer.poll() pose cette propriété sur les fichiers extraits
            exchange.set_property("source_archive", envelope_name);
            consumer.annotate(exchange, depot_path)
        })
        .collect()
}

fn build_consumer(paths: Vec<&str>) -> PpfReturnConsumer {
    PpfReturnConsumer::new(
        "ppf-return-test",
        PpfReturnConsumerConfig {
            sftp: SftpConfig::default(),
            paths: paths.into_iter().map(String::from).collect(),
            code_interface_by_path: HashMap::new(),
            archive_path: None,
            delete_after_read: false,
        },
    )
}

// ============================================================
// 1. Roundtrip F6 CDV 200 Reçu PPF
// ============================================================

#[tokio::test]
async fn test_ppf_return_cdv_200_recue_full_pipeline() {
    // CDV 200 « Reçu PPF » : la fixture représente un accusé de réception type
    let cdv_xml = std::fs::read("../../tests/fixtures/cdar/cdv_200_deposee.xml")
        .expect("Fixture cdv_200_deposee.xml introuvable");

    let (envelope_name, tar_gz) = make_envelope(
        CodeInterface::F6Facture,
        "AAA123",
        "0614000000000000200",
        "cdv_recu_001.xml",
        &cdv_xml,
    );
    assert!(envelope_name.starts_with("FFE0614A_AAA123_"));
    assert!(envelope_name.ends_with(".tar.gz"));

    // Ingestion (décompression + annotate)
    let consumer = build_consumer(vec!["/sas/retrait/F6"]);
    let exchanges = ingest_return_envelope(&consumer, "/sas/retrait/F6", &envelope_name, &tar_gz);
    assert_eq!(exchanges.len(), 1, "Une enveloppe = un fichier interne");

    let exchange = exchanges.into_iter().next().unwrap();

    // Vérifier les annotations PPF retour
    assert_eq!(
        exchange.get_header("source.protocol").map(String::as_str),
        Some("ppf-sftp-return")
    );
    assert_eq!(
        exchange.get_property("ppf.envelope").map(String::as_str),
        Some(envelope_name.as_str())
    );
    assert_eq!(
        exchange.get_property("ppf.code_interface").map(String::as_str),
        Some("FFE0614A")
    );
    assert_eq!(
        exchange.get_property("ppf.depot_path").map(String::as_str),
        Some("/sas/retrait/F6")
    );
    assert_eq!(
        exchange.get_property("flow.syntax").map(String::as_str),
        Some("CDAR")
    );

    // DocumentTypeRouter doit reconnaître le CDAR (et le parser au passage)
    let router = pdp_cdar::DocumentTypeRouter::new();
    let result = router.process(exchange).await.unwrap();
    assert_eq!(
        result.get_header("document.type").map(String::as_str),
        Some("CDAR"),
        "DocumentTypeRouter doit identifier le document comme CDAR"
    );
    // La source est détectée comme PPF grâce au header source.protocol = ppf-sftp-return
    assert_eq!(
        result.get_property("cdv.source").map(String::as_str),
        Some("ppf"),
        "La source du CDAR doit être 'ppf' (depuis source.protocol = ppf-sftp-return)"
    );

    // Le CDV est parsé par DocumentTypeRouter lui-même
    assert_eq!(
        result.get_property("cdv.received").map(String::as_str),
        Some("true"),
        "Le CDV doit être marqué comme reçu"
    );
    assert_eq!(
        result.get_property("cdv.status_code").map(String::as_str),
        Some("200"),
        "CDV 200 attendu"
    );
    assert_eq!(result.status, FlowStatus::Distributed);
    assert!(!result.has_errors(), "Aucune erreur attendue pour un CDV 200");
}

// ============================================================
// 2. Roundtrip F6 CDV 501 Irrecevable PPF
// ============================================================

#[tokio::test]
async fn test_ppf_return_cdv_501_irrecevable_full_pipeline() {
    let cdv_xml = std::fs::read("../../tests/fixtures/cdar/cdv_501_irrecevable.xml")
        .expect("Fixture cdv_501_irrecevable.xml introuvable");

    let (envelope_name, tar_gz) = make_envelope(
        CodeInterface::F6Facture,
        "AAA123",
        "0614000000000000501",
        "cdv_irr_001.xml",
        &cdv_xml,
    );

    let consumer = build_consumer(vec!["/sas/retrait/F6"]);
    let exchanges = ingest_return_envelope(&consumer, "/sas/retrait/F6", &envelope_name, &tar_gz);
    assert_eq!(exchanges.len(), 1);

    let exchange = exchanges.into_iter().next().unwrap();

    assert_eq!(
        exchange.get_property("ppf.code_interface").map(String::as_str),
        Some("FFE0614A")
    );
    assert_eq!(
        exchange.get_property("flow.syntax").map(String::as_str),
        Some("CDAR")
    );

    // Pipeline aval : DocumentTypeRouter (qui parse aussi le CDV)
    let router = pdp_cdar::DocumentTypeRouter::new();
    let result = router.process(exchange).await.unwrap();
    assert_eq!(
        result.get_header("document.type").map(String::as_str),
        Some("CDAR")
    );
    assert_eq!(
        result.get_property("cdv.source").map(String::as_str),
        Some("ppf")
    );

    assert_eq!(
        result.get_property("cdv.status_code").map(String::as_str),
        Some("501"),
        "CDV 501 Irrecevable attendu"
    );
    assert_eq!(result.status, FlowStatus::Rejected);
    assert!(result.has_errors(), "Le CDV 501 doit produire une erreur");
}

// ============================================================
// 3. Mapping par chemin (fallback quand le nom d'enveloppe ne suffit pas)
// ============================================================

#[tokio::test]
async fn test_ppf_return_path_mapping_fallback() {
    // Enveloppe sans préfixe FFExxxxA reconnu : on doit retomber sur le mapping path
    let cdv_xml = std::fs::read("../../tests/fixtures/cdar/cdv_200_deposee.xml").unwrap();

    let mut by_path = HashMap::new();
    by_path.insert("/sas/retrait/custom".to_string(), "FFE0614A".to_string());

    let consumer = PpfReturnConsumer::new(
        "ppf-return-test",
        PpfReturnConsumerConfig {
            sftp: SftpConfig::default(),
            paths: vec!["/sas/retrait/custom".to_string()],
            code_interface_by_path: by_path,
            archive_path: None,
            delete_after_read: false,
        },
    );

    // On simule un fichier extrait d'une archive au nom non standard
    let mut exchange = Exchange::new(cdv_xml).with_filename("cdv.xml");
    exchange.set_property("source_archive", "ARCHIVE_NON_STANDARD.tar.gz");

    let annotated = consumer.annotate(exchange, "/sas/retrait/custom");

    // code_interface dérivé du mapping par path, pas du nom d'enveloppe
    assert_eq!(
        annotated.get_property("ppf.code_interface").map(String::as_str),
        Some("FFE0614A"),
        "Le mapping par path doit prendre le relais quand l'enveloppe est non standard"
    );
    assert_eq!(
        annotated.get_property("flow.syntax").map(String::as_str),
        Some("CDAR")
    );
}

// ============================================================
// 4. Annotation isolée — propriétés et headers attendus
// ============================================================

#[tokio::test]
async fn test_ppf_return_annotation_sets_all_metadata() {
    let consumer = build_consumer(vec!["/sas/retrait/F6"]);

    let mut exchange = Exchange::new(b"<CrossIndustryDocumentApplicationResponse/>".to_vec())
        .with_filename("cdv.xml");
    exchange.set_property(
        "source_archive",
        "FFE0614A_AAA123_AAA1230614000000000000123.tar.gz",
    );

    let annotated = consumer.annotate(exchange, "/sas/retrait/F6");

    // Header source
    assert_eq!(
        annotated.get_header("source.protocol").map(String::as_str),
        Some("ppf-sftp-return")
    );

    // Propriétés PPF
    assert_eq!(
        annotated.get_property("ppf.depot_path").map(String::as_str),
        Some("/sas/retrait/F6")
    );
    assert_eq!(
        annotated.get_property("ppf.envelope").map(String::as_str),
        Some("FFE0614A_AAA123_AAA1230614000000000000123.tar.gz")
    );
    assert_eq!(
        annotated.get_property("ppf.code_interface").map(String::as_str),
        Some("FFE0614A")
    );

    // flow.syntax doit être positionné pour permettre au pipeline de router le doc
    assert_eq!(
        annotated.get_property("flow.syntax").map(String::as_str),
        Some("CDAR")
    );
}

// ============================================================
// 5. Vérifie que le nom d'enveloppe respecte le format spec
// ============================================================

#[test]
fn test_ppf_envelope_naming_spec_format() {
    // {CODE_INTERFACE}_{CODE_APP}_{IDENTIFIANT_FLUX}.tar.gz
    // L'identifiant flux fait 25 caractères : 6 (code app) + 19 (séquence)
    let name = flux_envelope_name(
        CodeInterface::F6Facture,
        "AAA123",
        "0614000000000000001",
    )
    .unwrap();

    assert_eq!(
        name, "FFE0614A_AAA123_AAA1230614000000000000001.tar.gz",
        "Le nommage doit suivre le format AIFE Externes v3.1 chap. 3.4.6"
    );

    // Le code interface peut être retrouvé depuis le nom
    let ci = PpfReturnConsumer::code_interface_from_envelope(&name);
    assert_eq!(ci, Some(CodeInterface::F6Facture));
}
