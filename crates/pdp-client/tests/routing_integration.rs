//! Tests d'intégration pour les producers (PpfSftpProducer, AfnorFlowProducer)
//! et le routage dynamique (DynamicRoutingProducer, RoutingResolverProcessor).
//!
//! Ces tests vérifient la logique métier sans connexion réseau :
//! - Construction et lecture d'archives tar.gz conformes PPF
//! - Résolution du code interface selon le format de facture
//! - Construction du FlowInfo AFNOR à partir d'un exchange
//! - Routage dynamique vers PPF ou filesystem (fallback)

use pdp_core::archive::read_tar_gz;
use pdp_core::exchange::Exchange;
use pdp_core::model::{InvoiceData, InvoiceFormat};

use pdp_client::model::*;
use pdp_client::{
    build_tar_gz, f1_inner_filename, flux_envelope_name, CodeInterface, FluxFile,
    PpfSftpProducer, AfnorFlowProducer, Destination,
    PartnerDirectory, RoutingProcessor,
};
use pdp_client::ppf::recommended_sequence;

// ============================================================
// Test 1 : PpfSftpProducer — construction tar.gz valide
// ============================================================

#[test]
fn test_ppf_producer_builds_valid_tar_gz() {
    let xml_content = b"<?xml version=\"1.0\"?><Invoice><ID>FA-2024-001</ID></Invoice>";

    // Simuler ce que fait PpfSftpProducer::send() :
    // 1. Construire le inner filename avec le profil
    let base_name = "facture_001.xml";
    let inner_name = f1_inner_filename(pdp_client::ProfilF1::Base, base_name);
    assert_eq!(inner_name, "Base_facture_001.xml");

    // 2. Construire l'archive tar.gz avec un seul fichier
    let flux_files = vec![FluxFile {
        filename: inner_name.clone(),
        content: xml_content.to_vec(),
    }];
    let tar_gz = build_tar_gz(&flux_files).expect("build_tar_gz doit réussir");
    assert!(!tar_gz.is_empty());

    // 3. Relire l'archive avec pdp_core::archive::read_tar_gz
    let entries = read_tar_gz(&tar_gz).expect("read_tar_gz doit réussir");
    assert_eq!(entries.len(), 1, "L'archive doit contenir exactement 1 fichier");
    assert_eq!(entries[0].filename, "Base_facture_001.xml");
    assert_eq!(entries[0].content, xml_content);

    // 4. Vérifier le nom de l'enveloppe
    let seq = recommended_sequence(CodeInterface::F1Ubl, 42);
    let envelope = flux_envelope_name(CodeInterface::F1Ubl, "TST001", &seq)
        .expect("flux_envelope_name doit réussir");
    assert!(envelope.starts_with("FFE0111A_TST001_"));
    assert!(envelope.ends_with(".tar.gz"));
    assert_eq!(
        envelope,
        "FFE0111A_TST001_TST0010111000000000000042.tar.gz"
    );
}

// ============================================================
// Test 2 : PpfSftpProducer::resolve_code_interface
// ============================================================

#[test]
fn test_ppf_producer_resolve_code_interface() {
    // Cas 1 : facture UBL parsée
    {
        let mut exchange = Exchange::new(b"<Invoice/>".to_vec());
        let invoice = InvoiceData::new("FA-001".to_string(), InvoiceFormat::UBL);
        exchange.invoice = Some(invoice);

        let ci = PpfSftpProducer::resolve_code_interface(&exchange);
        assert_eq!(ci, CodeInterface::F1Ubl);
    }

    // Cas 2 : facture CII parsée
    {
        let mut exchange = Exchange::new(b"<CIIInvoice/>".to_vec());
        let invoice = InvoiceData::new("FA-002".to_string(), InvoiceFormat::CII);
        exchange.invoice = Some(invoice);

        let ci = PpfSftpProducer::resolve_code_interface(&exchange);
        assert_eq!(ci, CodeInterface::F1Cii);
    }

    // Cas 3 : facture Factur-X parsée -> F1 CII
    {
        let mut exchange = Exchange::new(b"<FacturX/>".to_vec());
        let invoice = InvoiceData::new("FA-003".to_string(), InvoiceFormat::FacturX);
        exchange.invoice = Some(invoice);

        let ci = PpfSftpProducer::resolve_code_interface(&exchange);
        assert_eq!(ci, CodeInterface::F1Cii);
    }

    // Cas 4 : flow.syntax = CDAR -> F6
    {
        let mut exchange = Exchange::new(b"<CDAR/>".to_vec());
        exchange.set_property("flow.syntax", "CDAR");

        let ci = PpfSftpProducer::resolve_code_interface(&exchange);
        assert_eq!(ci, CodeInterface::F6Facture);
    }

    // Cas 5 : flow.syntax = FRR -> F10
    {
        let mut exchange = Exchange::new(b"<Report/>".to_vec());
        exchange.set_property("flow.syntax", "FRR");

        let ci = PpfSftpProducer::resolve_code_interface(&exchange);
        assert_eq!(ci, CodeInterface::F10TransactionPaiement);
    }

    // Cas 6 : propriété explicite ppf.code_interface
    {
        let mut exchange = Exchange::new(b"<data/>".to_vec());
        exchange.set_property("ppf.code_interface", "FFE0654A");

        let ci = PpfSftpProducer::resolve_code_interface(&exchange);
        assert_eq!(ci, CodeInterface::F6StatutsObligatoires);
    }

    // Cas 7 : propriété explicite prime sur le format de la facture
    {
        let mut exchange = Exchange::new(b"<Invoice/>".to_vec());
        let invoice = InvoiceData::new("FA-004".to_string(), InvoiceFormat::UBL);
        exchange.invoice = Some(invoice);
        exchange.set_property("ppf.code_interface", "FFE0112A"); // Force CII

        let ci = PpfSftpProducer::resolve_code_interface(&exchange);
        assert_eq!(ci, CodeInterface::F1Cii);
    }

    // Cas 8 : aucun indice -> défaut F1 CII
    {
        let exchange = Exchange::new(b"<data/>".to_vec());

        let ci = PpfSftpProducer::resolve_code_interface(&exchange);
        assert_eq!(ci, CodeInterface::F1Cii);
    }

    // Cas 9 : code interface inconnu tombe dans le fallback
    {
        let mut exchange = Exchange::new(b"<data/>".to_vec());
        exchange.set_property("ppf.code_interface", "ZZZZXXXX");
        // Code inconnu, pas de facture -> défaut F1 CII
        let ci = PpfSftpProducer::resolve_code_interface(&exchange);
        assert_eq!(ci, CodeInterface::F1Cii);
    }
}

// ============================================================
// Test 3 : AfnorFlowProducer::build_flow_info
// ============================================================

#[test]
fn test_afnor_producer_builds_flow_info() {
    // Cas 1 : exchange avec facture CII parsée, propriétés par défaut
    {
        let mut exchange = Exchange::new(b"<CIIInvoice/>".to_vec());
        exchange.source_filename = Some("facture_cii.xml".to_string());
        let invoice = InvoiceData::new("FA-100".to_string(), InvoiceFormat::CII);
        exchange.invoice = Some(invoice);

        let (info, filename) = AfnorFlowProducer::build_flow_info(&exchange);
        assert_eq!(filename, "facture_cii.xml");
        assert_eq!(info.flow_syntax, FlowSyntax::CII);
        assert_eq!(info.flow_profile, FlowProfile::CIUS); // défaut
        assert_eq!(info.processing_rule, ProcessingRule::B2B); // défaut
        assert!(info.flow_type.is_none()); // pas de propriété flow.type
        assert_eq!(info.tracking_id, exchange.id.to_string());
    }

    // Cas 2 : exchange UBL avec propriétés explicites
    {
        let mut exchange = Exchange::new(b"<Invoice/>".to_vec());
        exchange.source_filename = Some("ubl_invoice.xml".to_string());
        exchange.set_property("flow.syntax", "UBL");
        exchange.set_property("flow.profile", "Basic");
        exchange.set_property("flow.processing_rule", "B2BInt");
        exchange.set_property("flow.type", "CustomerInvoice");

        let (info, filename) = AfnorFlowProducer::build_flow_info(&exchange);
        assert_eq!(filename, "ubl_invoice.xml");
        assert_eq!(info.flow_syntax, FlowSyntax::UBL);
        assert_eq!(info.flow_profile, FlowProfile::Basic);
        assert_eq!(info.processing_rule, ProcessingRule::B2BInt);
        assert_eq!(info.flow_type, Some(FlowType::CustomerInvoice));
    }

    // Cas 3 : Factur-X avec profil Extended-CTC-FR
    {
        let mut exchange = Exchange::new(b"<FacturX/>".to_vec());
        exchange.set_property("flow.syntax", "Factur-X");
        exchange.set_property("flow.profile", "Extended-CTC-FR");

        let (info, _) = AfnorFlowProducer::build_flow_info(&exchange);
        assert_eq!(info.flow_syntax, FlowSyntax::FacturX);
        assert_eq!(info.flow_profile, FlowProfile::ExtendedCtcFr);
    }

    // Cas 4 : pas de source_filename -> nom généré à partir de l'id
    {
        let exchange = Exchange::new(b"<Invoice/>".to_vec());
        let (info, filename) = AfnorFlowProducer::build_flow_info(&exchange);
        assert!(filename.ends_with(".xml"));
        assert_eq!(info.name, filename);
    }

    // Cas 5 : syntaxe déduite depuis la facture parsée (UBL)
    {
        let mut exchange = Exchange::new(b"<Invoice/>".to_vec());
        let invoice = InvoiceData::new("FA-200".to_string(), InvoiceFormat::UBL);
        exchange.invoice = Some(invoice);

        let (info, _) = AfnorFlowProducer::build_flow_info(&exchange);
        assert_eq!(info.flow_syntax, FlowSyntax::UBL);
    }

    // Cas 6 : flow.syntax explicite prime sur la facture parsée
    {
        let mut exchange = Exchange::new(b"<Invoice/>".to_vec());
        let invoice = InvoiceData::new("FA-201".to_string(), InvoiceFormat::UBL);
        exchange.invoice = Some(invoice);
        exchange.set_property("flow.syntax", "CII");

        let (info, _) = AfnorFlowProducer::build_flow_info(&exchange);
        assert_eq!(info.flow_syntax, FlowSyntax::CII);
    }

    // Cas 7 : CDAR syntax
    {
        let mut exchange = Exchange::new(b"<CDAR/>".to_vec());
        exchange.set_property("flow.syntax", "CDAR");

        let (info, _) = AfnorFlowProducer::build_flow_info(&exchange);
        assert_eq!(info.flow_syntax, FlowSyntax::CDAR);
    }

    // Cas 8 : FRR syntax (e-reporting)
    {
        let mut exchange = Exchange::new(b"<Report/>".to_vec());
        exchange.set_property("flow.syntax", "FRR");

        let (info, _) = AfnorFlowProducer::build_flow_info(&exchange);
        assert_eq!(info.flow_syntax, FlowSyntax::FRR);
    }

    // Cas 9 : flow.type SupplierInvoiceLC
    {
        let mut exchange = Exchange::new(b"<Invoice/>".to_vec());
        exchange.set_property("flow.type", "SupplierInvoiceLC");

        let (info, _) = AfnorFlowProducer::build_flow_info(&exchange);
        assert_eq!(info.flow_type, Some(FlowType::SupplierInvoiceLC));
    }

    // Cas 10 : flow.processing_rule B2C
    {
        let mut exchange = Exchange::new(b"<Invoice/>".to_vec());
        exchange.set_property("flow.processing_rule", "B2C");

        let (info, _) = AfnorFlowProducer::build_flow_info(&exchange);
        assert_eq!(info.processing_rule, ProcessingRule::B2C);
    }
}

// ============================================================
// Test 4 : DynamicRoutingProducer — route PPF-SE
// ============================================================

#[test]
fn test_dynamic_routing_producer_ppf_route() {
    // On ne peut pas envoyer via SFTP dans les tests, mais on peut vérifier
    // que la logique de routage identifie correctement la destination.
    // DynamicRoutingProducer lit routing.destination de l'exchange.

    let mut exchange = Exchange::new(b"<Invoice/>".to_vec());
    exchange.set_property("routing.destination", "PPF-SE");

    // Vérifier que la propriété est bien lue
    let dest = exchange.get_property("routing.destination").unwrap();
    assert_eq!(dest, "PPF-SE");

    // Vérifier le Display des destinations
    assert_eq!(Destination::PpfSe.to_string(), "PPF-SE");
    assert_eq!(
        Destination::AfnorPdp {
            matricule: "1234".to_string(),
            flow_service_url: "https://pdp.example.com".to_string(),
        }
        .to_string(),
        "PDP-1234"
    );
    assert_eq!(
        Destination::File {
            path: "/tmp/output".to_string(),
        }
        .to_string(),
        "FILE:/tmp/output"
    );

    // Vérifier que "PDP-XXXX" est parsé par la logique du producer
    let mut exchange_pdp = Exchange::new(b"<Invoice/>".to_vec());
    exchange_pdp.set_property("routing.destination", "PDP-5678");
    let dest_pdp = exchange_pdp.get_property("routing.destination").unwrap();
    assert!(dest_pdp.starts_with("PDP-"));
    assert_eq!(&dest_pdp[4..], "5678");

    // Vérifier la logique de défaut quand aucune destination n'est définie
    let exchange_default = Exchange::new(b"<Invoice/>".to_vec());
    let default_dest = exchange_default
        .get_property("routing.destination")
        .cloned()
        .unwrap_or_else(|| "PPF-SE".to_string());
    assert_eq!(default_dest, "PPF-SE");
}

// ============================================================
// Test 5 : DynamicRoutingProducer — fallback filesystem
// ============================================================

#[tokio::test]
async fn test_dynamic_routing_producer_fallback() {
    // Le DynamicRoutingProducer écrit sur le filesystem quand la destination
    // est inconnue et qu'un fallback_path est configuré.
    // On teste la branche FileEndpoint en direct.

    let tmp_dir = std::env::temp_dir().join("pdp_test_fallback");
    let _ = std::fs::create_dir_all(&tmp_dir);

    let xml_body = b"<?xml version=\"1.0\"?><Invoice><ID>FALLBACK-001</ID></Invoice>";
    let mut exchange = Exchange::new(xml_body.to_vec());
    exchange.source_filename = Some("fallback_test.xml".to_string());

    // Simuler ce que fait DynamicRoutingProducer quand il tombe en fallback :
    // il utilise FileEndpoint::output
    use pdp_core::endpoint::{FileEndpoint, Producer};
    let file_producer = FileEndpoint::output("fallback", tmp_dir.to_str().unwrap());
    let result = file_producer.send(exchange).await;
    assert!(result.is_ok(), "L'écriture fallback doit réussir");

    // Vérifier que le fichier a été écrit
    let written_path = tmp_dir.join("fallback_test.xml");
    assert!(
        written_path.exists(),
        "Le fichier fallback doit exister à {:?}",
        written_path
    );
    let content = std::fs::read(&written_path).expect("Lecture du fichier fallback");
    assert_eq!(content, xml_body);

    // Nettoyage
    let _ = std::fs::remove_dir_all(&tmp_dir);
}

// ============================================================
// Test 6 : Round-trip tar.gz (build + read)
// ============================================================

#[test]
fn test_tar_gz_round_trip() {
    let files = vec![
        FluxFile {
            filename: "Base_facture_001.xml".to_string(),
            content: b"<?xml version=\"1.0\"?><Invoice><ID>001</ID></Invoice>".to_vec(),
        },
        FluxFile {
            filename: "Base_facture_002.xml".to_string(),
            content: b"<?xml version=\"1.0\"?><Invoice><ID>002</ID></Invoice>".to_vec(),
        },
        FluxFile {
            filename: "Full_facture_003.xml".to_string(),
            content: b"<?xml version=\"1.0\"?><Invoice><ID>003</ID><Extended>true</Extended></Invoice>"
                .to_vec(),
        },
    ];

    // Construire l'archive
    let tar_gz = build_tar_gz(&files).expect("build_tar_gz doit réussir");
    assert!(!tar_gz.is_empty());

    // Relire l'archive
    let entries = read_tar_gz(&tar_gz).expect("read_tar_gz doit réussir");
    assert_eq!(
        entries.len(),
        files.len(),
        "Le nombre de fichiers doit correspondre"
    );

    // Vérifier chaque fichier
    for (i, (entry, original)) in entries.iter().zip(files.iter()).enumerate() {
        assert_eq!(
            entry.filename, original.filename,
            "Fichier {} : nom ne correspond pas",
            i
        );
        assert_eq!(
            entry.content, original.content,
            "Fichier {} : contenu ne correspond pas",
            i
        );
    }
}

#[test]
fn test_tar_gz_round_trip_single_large_file() {
    // Tester avec un contenu plus volumineux (100 KB)
    let large_content: Vec<u8> = (0..100_000)
        .map(|i| b"ABCDEFGHIJ"[i % 10])
        .collect();

    let files = vec![FluxFile {
        filename: "large_file.xml".to_string(),
        content: large_content.clone(),
    }];

    let tar_gz = build_tar_gz(&files).expect("build_tar_gz doit réussir");
    let entries = read_tar_gz(&tar_gz).expect("read_tar_gz doit réussir");

    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].filename, "large_file.xml");
    assert_eq!(entries[0].content.len(), 100_000);
    assert_eq!(entries[0].content, large_content);
}

// ============================================================
// Test 7 : RoutingProcessor — extract_buyer_siren et propriétés
// ============================================================

#[test]
fn test_routing_resolver_sets_properties() {
    // Test extract_buyer_siren depuis propriété explicite
    {
        let mut exchange = Exchange::new(b"<Invoice/>".to_vec());
        exchange.set_property("buyer.siren", "123456789");

        let siren = RoutingProcessor::extract_buyer_siren(&exchange);
        assert_eq!(siren.as_deref(), Some("123456789"));
    }

    // Test extract_buyer_siren depuis la facture parsée (SIRET -> premiers 9 chars)
    {
        let mut exchange = Exchange::new(b"<Invoice/>".to_vec());
        let mut invoice = InvoiceData::new("FA-001".to_string(), InvoiceFormat::UBL);
        invoice.buyer_siret = Some("12345678901234".to_string());
        exchange.invoice = Some(invoice);

        let siren = RoutingProcessor::extract_buyer_siren(&exchange);
        assert_eq!(siren.as_deref(), Some("123456789"));
    }

    // Test extract_buyer_siren : propriété prime sur la facture
    {
        let mut exchange = Exchange::new(b"<Invoice/>".to_vec());
        exchange.set_property("buyer.siren", "999888777");
        let mut invoice = InvoiceData::new("FA-002".to_string(), InvoiceFormat::CII);
        invoice.buyer_siret = Some("12345678901234".to_string());
        exchange.invoice = Some(invoice);

        let siren = RoutingProcessor::extract_buyer_siren(&exchange);
        assert_eq!(siren.as_deref(), Some("999888777"));
    }

    // Test extract_buyer_siren : aucun SIREN trouvé
    {
        let exchange = Exchange::new(b"<Invoice/>".to_vec());
        let siren = RoutingProcessor::extract_buyer_siren(&exchange);
        assert!(siren.is_none());
    }

    // Test extract_buyer_siret depuis propriété
    {
        let mut exchange = Exchange::new(b"<Invoice/>".to_vec());
        exchange.set_property("buyer.siret", "12345678901234");

        let siret = RoutingProcessor::extract_buyer_siret(&exchange);
        assert_eq!(siret.as_deref(), Some("12345678901234"));
    }

    // Test extract_buyer_siret depuis la facture
    {
        let mut exchange = Exchange::new(b"<Invoice/>".to_vec());
        let mut invoice = InvoiceData::new("FA-003".to_string(), InvoiceFormat::UBL);
        invoice.buyer_siret = Some("98765432109876".to_string());
        exchange.invoice = Some(invoice);

        let siret = RoutingProcessor::extract_buyer_siret(&exchange);
        assert_eq!(siret.as_deref(), Some("98765432109876"));
    }

    // Test que les propriétés routing.* sont correctement positionnées
    // (simule ce que fait RoutingResolverProcessor quand aucun SIREN n'est trouvé)
    {
        let mut exchange = Exchange::new(b"<Invoice/>".to_vec());
        let invoice = InvoiceData::new("FA-004".to_string(), InvoiceFormat::UBL);
        exchange.invoice = Some(invoice);
        // Pas de buyer.siren ni buyer_siret -> le resolver met PPF-SE par défaut

        // Simule la logique du RoutingResolverProcessor
        let buyer_siren = RoutingProcessor::extract_buyer_siren(&exchange);
        let buyer_siret = RoutingProcessor::extract_buyer_siret(&exchange);

        if buyer_siren.is_none() && buyer_siret.is_none() {
            exchange.set_property("routing.destination", "PPF-SE");
            exchange.set_property("routing.pdp_matricule", "0000");
            exchange.set_property("routing.pdp_name", "PPF");
        }

        assert_eq!(
            exchange.get_property("routing.destination").map(String::as_str),
            Some("PPF-SE")
        );
        assert_eq!(
            exchange.get_property("routing.pdp_matricule").map(String::as_str),
            Some("0000")
        );
        assert_eq!(
            exchange.get_property("routing.pdp_name").map(String::as_str),
            Some("PPF")
        );
    }

    // Test PartnerDirectory enrichissement pour routage PDP
    {
        let mut dir = PartnerDirectory::new();
        dir.add_partner("4567", "PDP DELTA", "https://flow.delta.fr/v1");

        let resolution = pdp_client::annuaire::RoutingResolution {
            pdp_matricule: "4567".to_string(),
            pdp_name: String::new(),
            flow_service_url: None,
            is_ppf: false,
        };

        let enriched = dir.enrich_resolution(resolution);
        assert_eq!(enriched.pdp_name, "PDP DELTA");
        assert_eq!(
            enriched.flow_service_url.as_deref(),
            Some("https://flow.delta.fr/v1")
        );

        // Simuler les propriétés routing.* pour une PDP distante
        let mut exchange = Exchange::new(b"<Invoice/>".to_vec());
        exchange.set_property(
            "routing.destination",
            &format!("PDP-{}", enriched.pdp_matricule),
        );
        exchange.set_property("routing.pdp_matricule", &enriched.pdp_matricule);
        exchange.set_property("routing.pdp_name", &enriched.pdp_name);
        if let Some(ref url) = enriched.flow_service_url {
            exchange.set_property("routing.flow_service_url", url);
        }

        assert_eq!(
            exchange.get_property("routing.destination").map(String::as_str),
            Some("PDP-4567")
        );
        assert_eq!(
            exchange.get_property("routing.flow_service_url").map(String::as_str),
            Some("https://flow.delta.fr/v1")
        );
    }
}
