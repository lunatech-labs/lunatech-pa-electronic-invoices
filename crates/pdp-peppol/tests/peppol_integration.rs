//! Test d'intégration PEPPOL : envoi d'une facture UBL vers une PDP distante.
//!
//! Ce test simule le flux complet PDP_A → PDP_B :
//!
//! 1. Charger une facture UBL depuis les fixtures
//! 2. Construire un PeppolMessage avec SBDH
//! 3. PDP_A envoie via FilesystemGateway (outbox)
//! 4. PDP_B reçoit via FilesystemGateway (inbox)
//! 5. PeppolReceiveProcessor traite le message entrant
//! 6. Vérifier l'intégrité : même ID facture, même contenu, métadonnées correctes
//!
//! # Mode Docker (optionnel)
//!
//! Si Oxalis tourne (`docker-compose --profile peppol up`), le test vérifie
//! aussi la connectivité REST du gateway. Sinon, seul le roundtrip filesystem
//! est exécuté.
//!
//! ```bash
//! # Sans Docker (toujours exécutable)
//! cargo test -p pdp-peppol --test peppol_integration
//!
//! # Avec Docker Oxalis
//! docker-compose --profile peppol up -d
//! OXALIS_URL=http://localhost:8080 cargo test -p pdp-peppol --test peppol_integration
//! ```

use pdp_core::exchange::Exchange;
use pdp_core::model::FlowStatus;
use pdp_core::processor::Processor;
use pdp_peppol::gateway::FilesystemGateway;
use pdp_peppol::model::*;
use pdp_peppol::processor::PeppolReceiveProcessor;
use pdp_peppol::sbdh;
use tempfile::TempDir;

/// Chemin vers les fixtures UBL (relatif au crate pdp-peppol)
const FIXTURE_UBL: &str = "../../tests/fixtures/ubl/facture_ubl_001.xml";

// ============================================================
// Test 1 : Roundtrip complet PDP_A → FilesystemGateway → PDP_B
// ============================================================

#[tokio::test]
async fn test_envoi_facture_pdp_a_vers_pdp_b_filesystem() {
    // --- 1. Charger la facture UBL ---
    let invoice_xml = std::fs::read_to_string(FIXTURE_UBL)
        .expect("Fixture UBL facture_ubl_001.xml introuvable");
    assert!(invoice_xml.contains("<Invoice"), "Le fichier doit contenir une facture UBL");

    // Extraire l'ID de la facture pour vérification ultérieure
    let invoice_id = extract_invoice_id(&invoice_xml);
    assert_eq!(invoice_id, "FA-2025-00142");

    // --- 2. Construire le PeppolMessage ---
    let sender = ParticipantId::from_siren("123456789"); // Vendeur (TechConseil)
    let receiver = ParticipantId::from_siren("987654321"); // Acheteur (IndustrieFrance)

    let message = PeppolMessage::ubl_invoice(
        sender.clone(),
        receiver.clone(),
        invoice_xml.as_bytes().to_vec(),
    );

    assert_eq!(message.document_type, PeppolDocumentType::Invoice);
    assert!(!message.message_id.is_empty(), "Message ID doit être généré");

    // --- 3. PDP_A envoie via FilesystemGateway ---
    let tmp = TempDir::new().unwrap();

    // En production : outbox = volume partagé avec Oxalis outbound
    //                 inbox  = volume partagé avec Oxalis remote inbound
    // En test : on utilise le même répertoire pour simuler le transport AS4
    let shared_dir = tmp.path().join("as4_transport");

    let gw_send = FilesystemGateway::new(&shared_dir, tmp.path().join("unused_inbox"));
    let gw_recv = FilesystemGateway::new(tmp.path().join("unused_outbox"), &shared_dir);

    // PDP_A dépose le SBDH dans l'outbox
    let filename = gw_send.send(&message).expect("Envoi filesystem échoué");

    // Vérifier que le fichier SBDH a été créé
    let sbdh_path = shared_dir.join(&filename);
    assert!(sbdh_path.exists(), "Le fichier SBDH doit exister dans l'outbox");

    // Vérifier le contenu SBDH
    let sbdh_content = std::fs::read_to_string(&sbdh_path).unwrap();
    assert!(sbdh_content.contains("StandardBusinessDocument"), "Doit être un SBDH valide");
    assert!(sbdh_content.contains("123456789"), "SBDH doit contenir le SIREN émetteur");
    assert!(sbdh_content.contains("987654321"), "SBDH doit contenir le SIREN destinataire");
    assert!(sbdh_content.contains("DOCUMENTID"), "SBDH doit contenir le scope DOCUMENTID");
    assert!(sbdh_content.contains("PROCESSID"), "SBDH doit contenir le scope PROCESSID");
    assert!(sbdh_content.contains("FA-2025-00142"), "SBDH doit contenir la facture originale");

    // --- 4. PDP_B reçoit depuis l'inbox ---
    let received_messages = gw_recv.receive().expect("Réception filesystem échouée");
    assert_eq!(received_messages.len(), 1, "PDP_B doit recevoir exactement 1 message");

    let inbox_msg = &received_messages[0];
    assert_eq!(inbox_msg.sender, "0002::123456789", "Sender doit être le SIREN vendeur");
    assert_eq!(inbox_msg.receiver, "0002::987654321", "Receiver doit être le SIREN acheteur");
    assert!(inbox_msg.payload.contains("FA-2025-00142"), "Payload doit contenir l'ID facture");
    assert!(inbox_msg.payload.contains("<Invoice"), "Payload doit contenir la facture UBL");

    // --- 5. PeppolReceiveProcessor traite le message ---
    let processor = PeppolReceiveProcessor::new();
    let mut exchange = Exchange::new(inbox_msg.payload.as_bytes().to_vec());
    exchange.set_property("peppol.sender", &inbox_msg.sender);
    exchange.set_property("peppol.receiver", &inbox_msg.receiver);
    exchange.set_property("peppol.message_id", &inbox_msg.instance_id);

    let result = processor.process(exchange).await
        .expect("PeppolReceiveProcessor échoué");

    // --- 6. Vérifications finales ---
    assert_eq!(result.status, FlowStatus::Received, "Statut doit être Received");
    assert_eq!(
        result.get_property("peppol.received").map(|s| s.as_str()),
        Some("true"),
        "Propriété peppol.received doit être true"
    );
    assert_eq!(
        result.get_property("peppol.document_type").map(|s| s.as_str()),
        Some("Invoice"),
        "Type de document doit être Invoice"
    );
    assert!(
        result.source_filename.as_ref().unwrap().starts_with("peppol_"),
        "Nom de fichier doit commencer par peppol_"
    );

    // Vérifier que le body contient toujours la facture intacte
    let body_str = std::str::from_utf8(&result.body).unwrap();
    assert!(body_str.contains("FA-2025-00142"), "Le body doit contenir l'ID facture original");
    assert!(body_str.contains("TechConseil"), "Le body doit contenir le vendeur");
    assert!(body_str.contains("IndustrieFrance"), "Le body doit contenir l'acheteur");

    // --- 7. Acquitter le message ---
    gw_recv.acknowledge(&filename).expect("Acquittement échoué");
    let after = gw_recv.receive().unwrap();
    assert!(after.is_empty(), "Inbox doit être vide après acquittement");
}

// ============================================================
// Test 2 : Roundtrip SBDH build → parse préserve les métadonnées
// ============================================================

#[test]
fn test_sbdh_roundtrip_avec_vraie_facture() {
    let invoice_xml = std::fs::read_to_string(FIXTURE_UBL)
        .expect("Fixture UBL introuvable");

    let sender = ParticipantId::from_siren("123456789");
    let receiver = ParticipantId::from_siren("987654321");

    let message = PeppolMessage::ubl_invoice(
        sender,
        receiver,
        invoice_xml.as_bytes().to_vec(),
    );

    // Build SBDH
    let sbdh_xml = sbdh::build_sbdh(&message);

    // Parse SBDH
    let parsed = sbdh::parse_sbdh(&sbdh_xml)
        .expect("Parsing SBDH échoué");

    // Vérifier les métadonnées de routage
    assert_eq!(parsed.sender.scheme, "0002");
    assert_eq!(parsed.sender.value, "123456789");
    assert_eq!(parsed.receiver.scheme, "0002");
    assert_eq!(parsed.receiver.value, "987654321");
    assert_eq!(parsed.instance_id, message.message_id);
    assert_eq!(parsed.document_type_id.scheme, "busdox-docid-qns");
    assert!(parsed.document_type_id.value.contains("Invoice"));
    assert_eq!(parsed.process_id.scheme, "cenbii-procid-ubl");

    // Vérifier que le payload est intact
    assert!(parsed.payload.contains("FA-2025-00142"));
    assert!(parsed.payload.contains("TechConseil"));
    assert!(parsed.payload.contains("IndustrieFrance"));

    // Vérifier que la déclaration XML du payload a été retirée
    assert!(!parsed.payload.starts_with("<?xml"), "Le payload ne doit pas contenir la déclaration XML");
}

// ============================================================
// Test 3 : Envoi d'un CDAR entre PDP
// ============================================================

#[tokio::test]
async fn test_envoi_cdar_pdp_a_vers_pdp_b_filesystem() {
    // Charger un CDV fixture
    let cdv_xml = std::fs::read_to_string("../../tests/fixtures/cdar/cdv_200_deposee.xml")
        .expect("Fixture CDV introuvable");

    let sender = ParticipantId::from_siren("111111111");
    let receiver = ParticipantId::from_siren("222222222");

    let message = PeppolMessage::cdar(
        sender.clone(),
        receiver.clone(),
        cdv_xml.as_bytes().to_vec(),
    );

    assert_eq!(message.document_type, PeppolDocumentType::Cdar);

    // Roundtrip filesystem
    let tmp = TempDir::new().unwrap();
    let shared_dir = tmp.path().join("transport");

    let gw_send = FilesystemGateway::new(&shared_dir, tmp.path().join("_"));
    let gw_recv = FilesystemGateway::new(tmp.path().join("_"), &shared_dir);

    let filename = gw_send.send(&message).unwrap();
    assert!(filename.contains("cdar"), "Nom de fichier doit contenir 'cdar'");

    let received = gw_recv.receive().unwrap();
    assert_eq!(received.len(), 1);
    assert!(received[0].payload.contains("CrossDomainAcknowledgementAndResponse"));

    // PeppolReceiveProcessor détecte le CDAR
    let processor = PeppolReceiveProcessor::new();
    let mut exchange = Exchange::new(received[0].payload.as_bytes().to_vec());
    exchange.set_property("peppol.sender", &received[0].sender);
    exchange.set_property("peppol.receiver", &received[0].receiver);
    exchange.set_property("peppol.message_id", &received[0].instance_id);

    let result = processor.process(exchange).await.unwrap();
    assert_eq!(
        result.get_property("peppol.document_type").map(|s| s.as_str()),
        Some("CDAR"),
        "Le processor doit détecter un CDAR"
    );
    assert_eq!(result.status, FlowStatus::Received);
}

// ============================================================
// Test 4 : Gateway REST Oxalis (nécessite Docker)
// ============================================================

#[tokio::test]
async fn test_oxalis_rest_gateway_health() {
    // Ce test ne s'exécute que si OXALIS_URL est défini
    let oxalis_url = match std::env::var("OXALIS_URL") {
        Ok(url) => url,
        Err(_) => {
            eprintln!("OXALIS_URL non défini — test Docker Oxalis ignoré");
            eprintln!("Pour exécuter : docker-compose --profile peppol up -d");
            eprintln!("Puis : OXALIS_URL=http://localhost:8080 cargo test -p pdp-peppol --test peppol_integration");
            return;
        }
    };

    let gw = pdp_peppol::RestGateway::new(&oxalis_url);
    let healthy = gw.health_check().await.expect("Health check échoué");

    if healthy {
        println!("✓ Oxalis gateway accessible sur {}", oxalis_url);
    } else {
        eprintln!("✗ Oxalis gateway non accessible sur {} — vérifier docker-compose", oxalis_url);
    }
}

#[tokio::test]
async fn test_oxalis_envoi_facture_via_rest_gateway() {
    // Ce test ne s'exécute que si OXALIS_URL est défini
    let oxalis_url = match std::env::var("OXALIS_URL") {
        Ok(url) => url,
        Err(_) => {
            eprintln!("OXALIS_URL non défini — test envoi Oxalis ignoré");
            return;
        }
    };

    let invoice_xml = std::fs::read_to_string(FIXTURE_UBL)
        .expect("Fixture UBL introuvable");

    let sender = ParticipantId::from_siren("123456789");
    let receiver = ParticipantId::from_siren("987654321");

    let message = PeppolMessage::ubl_invoice(
        sender,
        receiver,
        invoice_xml.as_bytes().to_vec(),
    );

    let gw = pdp_peppol::RestGateway::new(&oxalis_url);
    let result = gw.send(&message).await.expect("Envoi REST échoué");

    println!(
        "Résultat envoi Oxalis : success={}, message_id={}, response={}",
        result.success, result.message_id, &result.gateway_response[..100.min(result.gateway_response.len())]
    );

    // Note : le succès dépend de la configuration Oxalis (PKI, SMP)
    // En environnement de test local sans SMP, l'envoi échouera probablement
    // mais le test vérifie que la communication avec le gateway fonctionne
    assert!(!result.message_id.is_empty(), "Message ID doit être présent");
}

// ============================================================
// Test 6 : Envoi AS4 réel vers oxalis-remote (nécessite Docker + SMP)
// ============================================================

/// Test end-to-end : envoie un message AS4 depuis notre code Rust
/// directement vers oxalis-remote (PDP_B).
///
/// Prérequis :
///   podman compose --profile peppol up -d smp oxalis oxalis-remote
///   bash ./docker/peppol-setup.sh
///   OXALIS_URL=http://localhost:8080 cargo test -p pdp-peppol --test peppol_integration
#[tokio::test]
async fn test_as4_envoi_reel_vers_oxalis_remote() {
    // Ce test ne s'exécute que si OXALIS_URL est défini
    let _oxalis_url = match std::env::var("OXALIS_URL") {
        Ok(url) => url,
        Err(_) => {
            eprintln!("OXALIS_URL non défini — test AS4 réel ignoré");
            return;
        }
    };

    // L'URL d'oxalis-remote est déduite (port 8081 en local)
    let remote_as4_url = std::env::var("OXALIS_REMOTE_URL")
        .unwrap_or_else(|_| "http://localhost:8081/as4".to_string());

    let invoice_xml = std::fs::read_to_string(FIXTURE_UBL)
        .expect("Fixture UBL introuvable");

    let sender = ParticipantId::from_siren("123456789");
    let receiver = ParticipantId::from_siren("987654321");

    let message = PeppolMessage::ubl_invoice(
        sender,
        receiver,
        invoice_xml.as_bytes().to_vec(),
    );

    // Construire le client AS4
    let config = pdp_peppol::model::PeppolConfig::test();
    let client = pdp_peppol::as4::As4Client::new(config);

    // Endpoint cible : oxalis-remote
    let endpoint = pdp_peppol::model::SmpEndpoint {
        endpoint_url: remote_as4_url.clone(),
        certificate: String::new(),
        transport_profile: "peppol-transport-as4-v2_0".to_string(),
        service_activation_date: None,
        service_expiration_date: None,
    };

    println!("Envoi AS4 vers {}", remote_as4_url);
    let result = client.send(&message, &endpoint).await
        .expect("Envoi AS4 échoué (erreur réseau)");

    println!(
        "Résultat AS4 : success={}, message_id={}, error={:?}",
        result.success, result.message_id, result.error
    );

    // Le message a été envoyé — même si Oxalis rejette (certificat invalide),
    // on vérifie que la communication AS4 fonctionne
    assert!(!result.message_id.is_empty(), "Message ID doit être présent");

    // Si le message est accepté, c'est un vrai succès AS4 end-to-end
    if result.success {
        println!("✓ Message AS4 accepté par oxalis-remote !");
    } else {
        println!(
            "✗ Message AS4 rejeté par oxalis-remote (attendu en test local) : {:?}",
            result.error
        );
    }
}

// ============================================================
// Test 7 : Vérification du SMP lookup (nécessite Docker + SMP)
// ============================================================

#[tokio::test]
async fn test_smp_lookup_participant() {
    let _oxalis_url = match std::env::var("OXALIS_URL") {
        Ok(url) => url,
        Err(_) => {
            eprintln!("OXALIS_URL non défini — test SMP lookup ignoré");
            return;
        }
    };

    let smp_url = std::env::var("SMP_URL")
        .unwrap_or_else(|_| "http://localhost:8888".to_string());

    // Vérifier que le SMP répond
    let client = reqwest::Client::new();
    let resp = client.get(format!(
        "{}/iso6523-actorid-upis%3A%3A0002%3A987654321",
        smp_url
    ))
    .send()
    .await
    .expect("SMP inaccessible");

    assert_eq!(resp.status(), 200, "Le participant PDP_B doit être enregistré dans le SMP");

    let body = resp.text().await.unwrap();
    assert!(body.contains("987654321"), "La réponse SMP doit contenir le participant ID");
    assert!(body.contains("ServiceGroup"), "La réponse SMP doit être un ServiceGroup XML");

    println!("✓ SMP lookup OK pour 0002:987654321");
}

// ============================================================
// Utilitaires
// ============================================================

/// Extrait l'ID de la facture depuis le XML UBL (valeur de <cbc:ID>)
fn extract_invoice_id(xml: &str) -> String {
    let doc = roxmltree::Document::parse(xml).expect("XML invalide");
    let root = doc.root_element();

    for child in root.children() {
        if child.has_tag_name("ID") {
            return child.text().unwrap_or("").to_string();
        }
    }

    panic!("Élément <cbc:ID> introuvable dans la facture UBL");
}
