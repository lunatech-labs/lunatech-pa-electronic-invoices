//! Tests d'intégration : AnnuaireService + AnnuaireValidationProcessor (G1.63).
//!
//! Démarre un Postgres jetable via testcontainers pour chaque test.
//! Prérequis : Docker / Podman démarré.

use std::sync::Arc;

use pdp_annuaire::model::{Diffusible, MotifPresence, Statut, TypeEntite, UniteLegale};
use pdp_annuaire::{
    AnnuaireService, AnnuaireStore, AnnuaireValidationProcessor, ValidationMode,
};
use pdp_core::exchange::Exchange;
use pdp_core::model::{InvoiceData, InvoiceFormat};
use pdp_core::processor::Processor;
use sqlx::postgres::PgPoolOptions;
use testcontainers::runners::AsyncRunner;
use testcontainers::ContainerAsync;
use testcontainers_modules::postgres::Postgres;

const SIREN_VENDEUR_ACTIF: &str = "111111111";
const SIREN_INACTIF: &str = "222222222";
const SIREN_INCONNU: &str = "999999999";

/// Lance un Postgres éphémère, applique les migrations et insère deux UL :
/// - 111111111 actif "Vendeur SARL"
/// - 222222222 inactif "Buyer Old SAS"
async fn setup() -> (ContainerAsync<Postgres>, Arc<AnnuaireService>) {
    let container = Postgres::default()
        .start()
        .await
        .expect("démarrage container Postgres");
    let host = container.get_host().await.unwrap();
    let port = container.get_host_port_ipv4(5432).await.unwrap();
    let url = format!("postgres://postgres:postgres@{}:{}/postgres", host, port);

    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(&url)
        .await
        .expect("connect postgres");
    let store = AnnuaireStore::new(pool);
    store.migrate().await.expect("migrations");

    let uls = vec![
        UniteLegale {
            id_instance: 1,
            motif_presence: MotifPresence::Creation,
            statut: Statut::Actif,
            siren: SIREN_VENDEUR_ACTIF.to_string(),
            nom: "Vendeur SARL".to_string(),
            type_entite: TypeEntite::Assujetti,
            diffusible: Diffusible::Oui,
        },
        UniteLegale {
            id_instance: 2,
            motif_presence: MotifPresence::Creation,
            statut: Statut::Inactif,
            siren: SIREN_INACTIF.to_string(),
            nom: "Buyer Old SAS".to_string(),
            type_entite: TypeEntite::Assujetti,
            diffusible: Diffusible::Oui,
        },
    ];
    store.insert_unites_legales(&uls).await.expect("insert UL");

    let service = Arc::new(AnnuaireService::new(store));
    (container, service)
}

/// Construit un Exchange minimal avec une facture portant les SIRET donnés.
/// Le processor extrait le SIREN sur les 9 premiers caractères.
fn invoice_exchange(seller_siret: &str, buyer_siret: Option<&str>) -> Exchange {
    let mut invoice = InvoiceData::new("F-001".to_string(), InvoiceFormat::CII);
    invoice.seller_siret = Some(seller_siret.to_string());
    invoice.buyer_siret = buyer_siret.map(str::to_string);
    let mut exchange = Exchange::new(b"<Invoice/>".to_vec());
    exchange.invoice = Some(invoice);
    exchange
}

fn siret_for(siren: &str) -> String {
    format!("{}00001", siren)
}

// ---- AnnuaireService ----

#[tokio::test]
async fn service_seller_active_buyer_unknown() {
    let (_pg, service) = setup().await;
    let r = service
        .validate_parties(SIREN_VENDEUR_ACTIF, Some(SIREN_INCONNU))
        .await
        .expect("validate");
    assert!(r.seller_exists);
    assert!(r.seller_active);
    assert_eq!(r.seller_name.as_deref(), Some("Vendeur SARL"));
    assert!(!r.buyer_exists);
    assert!(!r.buyer_active);
}

#[tokio::test]
async fn service_seller_inactive() {
    let (_pg, service) = setup().await;
    let r = service
        .validate_parties(SIREN_INACTIF, None)
        .await
        .expect("validate");
    assert!(r.seller_exists);
    assert!(!r.seller_active);
    assert_eq!(r.seller_name.as_deref(), Some("Buyer Old SAS"));
}

#[tokio::test]
async fn service_no_buyer_check_when_none() {
    let (_pg, service) = setup().await;
    let r = service
        .validate_parties(SIREN_VENDEUR_ACTIF, None)
        .await
        .expect("validate");
    assert!(r.buyer_exists, "skip buyer check → buyer_exists=true par convention");
    assert!(r.buyer_active);
    assert!(r.buyer_name.is_none());
}

// ---- AnnuaireValidationProcessor mode Emission ----

#[tokio::test]
async fn processor_emission_seller_unknown_adds_error() {
    let (_pg, service) = setup().await;
    let processor =
        AnnuaireValidationProcessor::new(Some(service), ValidationMode::Emission);
    let exchange =
        invoice_exchange(&siret_for(SIREN_INCONNU), Some(&siret_for(SIREN_VENDEUR_ACTIF)));

    let out = processor.process(exchange).await.expect("process");
    assert!(out.has_errors(), "vendeur inconnu doit ajouter une erreur");
    let msgs: Vec<_> = out.errors.iter().map(|e| e.message.clone()).collect();
    assert!(
        msgs.iter()
            .any(|m| m.contains("Vendeur inconnu") && m.contains(SIREN_INCONNU)),
        "messages: {:?}",
        msgs
    );
}

#[tokio::test]
async fn processor_emission_buyer_unknown_adds_error() {
    let (_pg, service) = setup().await;
    let processor =
        AnnuaireValidationProcessor::new(Some(service), ValidationMode::Emission);
    let exchange =
        invoice_exchange(&siret_for(SIREN_VENDEUR_ACTIF), Some(&siret_for(SIREN_INCONNU)));

    let out = processor.process(exchange).await.expect("process");
    assert!(out.has_errors(), "acheteur inconnu doit ajouter une erreur");
    let msgs: Vec<_> = out.errors.iter().map(|e| e.message.clone()).collect();
    assert!(
        msgs.iter()
            .any(|m| m.contains("Destinataire inconnu") && m.contains(SIREN_INCONNU)),
        "messages: {:?}",
        msgs
    );
}

#[tokio::test]
async fn processor_emission_seller_inactive_adds_error() {
    let (_pg, service) = setup().await;
    let processor =
        AnnuaireValidationProcessor::new(Some(service), ValidationMode::Emission);
    let exchange =
        invoice_exchange(&siret_for(SIREN_INACTIF), Some(&siret_for(SIREN_VENDEUR_ACTIF)));

    let out = processor.process(exchange).await.expect("process");
    assert!(out.has_errors());
    let msgs: Vec<_> = out.errors.iter().map(|e| e.message.clone()).collect();
    assert!(
        msgs.iter().any(|m| m.contains("Vendeur inactif")),
        "messages: {:?}",
        msgs
    );
}

#[tokio::test]
async fn processor_emission_buyer_inactive_adds_error() {
    let (_pg, service) = setup().await;
    let processor =
        AnnuaireValidationProcessor::new(Some(service), ValidationMode::Emission);
    let exchange =
        invoice_exchange(&siret_for(SIREN_VENDEUR_ACTIF), Some(&siret_for(SIREN_INACTIF)));

    let out = processor.process(exchange).await.expect("process");
    assert!(out.has_errors());
    let msgs: Vec<_> = out.errors.iter().map(|e| e.message.clone()).collect();
    assert!(
        msgs.iter().any(|m| m.contains("Destinataire inactif")),
        "messages: {:?}",
        msgs
    );
}

#[tokio::test]
async fn processor_emission_both_known_active_passes() {
    let (_pg, service) = setup().await;
    let processor =
        AnnuaireValidationProcessor::new(Some(service), ValidationMode::Emission);
    let exchange = invoice_exchange(
        &siret_for(SIREN_VENDEUR_ACTIF),
        Some(&siret_for(SIREN_VENDEUR_ACTIF)),
    );

    let out = processor.process(exchange).await.expect("process");
    assert!(!out.has_errors(), "erreurs inattendues : {:?}", out.errors);
}

// ---- AnnuaireValidationProcessor mode Reception ----

#[tokio::test]
async fn processor_reception_skips_buyer_check() {
    let (_pg, service) = setup().await;
    let processor =
        AnnuaireValidationProcessor::new(Some(service), ValidationMode::Reception);
    // Vendeur connu actif, acheteur inconnu : en réception, l'acheteur n'est PAS vérifié
    let exchange =
        invoice_exchange(&siret_for(SIREN_VENDEUR_ACTIF), Some(&siret_for(SIREN_INCONNU)));

    let out = processor.process(exchange).await.expect("process");
    assert!(
        !out.has_errors(),
        "réception ne vérifie pas l'acheteur, erreurs: {:?}",
        out.errors
    );
}

#[tokio::test]
async fn processor_reception_seller_unknown_adds_error() {
    let (_pg, service) = setup().await;
    let processor =
        AnnuaireValidationProcessor::new(Some(service), ValidationMode::Reception);
    let exchange =
        invoice_exchange(&siret_for(SIREN_INCONNU), Some(&siret_for(SIREN_VENDEUR_ACTIF)));

    let out = processor.process(exchange).await.expect("process");
    assert!(out.has_errors());
    let msgs: Vec<_> = out.errors.iter().map(|e| e.message.clone()).collect();
    assert!(
        msgs.iter().any(|m| m.contains("Vendeur inconnu")),
        "messages: {:?}",
        msgs
    );
}
