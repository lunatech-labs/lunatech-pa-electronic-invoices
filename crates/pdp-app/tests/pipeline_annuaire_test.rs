//! Test pipeline end-to-end : AnnuaireValidationProcessor → CdarProcessor.
//!
//! Démarre un Postgres jetable, insère quelques unités légales, soumet une
//! facture au mini-pipeline et vérifie le CDV produit (code statut + code raison).
//!
//! Prérequis : Docker / Podman démarré.

use std::sync::Arc;

use pdp_annuaire::model::{Diffusible, MotifPresence, Statut, TypeEntite, UniteLegale};
use pdp_annuaire::{
    AnnuaireService, AnnuaireStore, AnnuaireValidationProcessor, ValidationMode,
};
use pdp_cdar::{CdarParser, CdarProcessor};
use pdp_core::exchange::Exchange;
use pdp_core::model::{InvoiceData, InvoiceFormat};
use pdp_core::processor::Processor;
use sqlx::postgres::PgPoolOptions;
use testcontainers::runners::AsyncRunner;
use testcontainers::ContainerAsync;
use testcontainers_modules::postgres::Postgres;

const SIREN_VENDEUR_OK: &str = "111111111";
const SIREN_INCONNU: &str = "999999999";

async fn setup() -> (ContainerAsync<Postgres>, Arc<AnnuaireService>) {
    let container = Postgres::default().start().await.expect("démarrage Postgres");
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

    let uls = vec![UniteLegale {
        id_instance: 1,
        motif_presence: MotifPresence::Creation,
        statut: Statut::Actif,
        siren: SIREN_VENDEUR_OK.to_string(),
        nom: "Vendeur SARL".to_string(),
        type_entite: TypeEntite::Assujetti,
        diffusible: Diffusible::Oui,
    }];
    store.insert_unites_legales(&uls).await.expect("insert UL");

    (container, Arc::new(AnnuaireService::new(store)))
}

fn make_invoice(seller_siret: &str, buyer_siret: &str) -> InvoiceData {
    let mut inv = InvoiceData::new("FA-PIPELINE-001".to_string(), InvoiceFormat::CII);
    inv.invoice_type_code = Some("380".to_string());
    inv.issue_date = Some("2026-04-26".to_string());
    inv.seller_siret = Some(seller_siret.to_string());
    inv.seller_name = Some("Vendeur SARL".to_string());
    inv.buyer_siret = Some(buyer_siret.to_string());
    inv.buyer_name = Some("Acheteur SAS".to_string());
    inv.currency = Some("EUR".to_string());
    inv
}

fn invoice_exchange(seller_siret: &str, buyer_siret: &str) -> Exchange {
    let mut exchange = Exchange::new(b"<Invoice/>".to_vec());
    exchange.invoice = Some(make_invoice(seller_siret, buyer_siret));
    exchange
}

fn siret_for(siren: &str) -> String {
    format!("{}00001", siren)
}

/// Traverse AnnuaireValidationProcessor puis CdarProcessor, retourne l'exchange final.
async fn run_pipeline(
    service: Arc<AnnuaireService>,
    mode: ValidationMode,
    exchange: Exchange,
) -> Exchange {
    let validator = AnnuaireValidationProcessor::new(Some(service), mode);
    let cdar = match mode {
        ValidationMode::Emission => CdarProcessor::emission("100000009", "PDP Test"),
        ValidationMode::Reception => CdarProcessor::reception("100000009", "PDP Test"),
    };

    let exchange = validator.process(exchange).await.expect("validator process");
    cdar.process(exchange).await.expect("cdar process")
}

fn parse_cdv(exchange: &Exchange) -> pdp_cdar::CdvResponse {
    let xml = exchange
        .get_property("cdv.xml")
        .expect("cdv.xml absent de l'exchange");
    CdarParser::new().parse(xml).expect("CDV non parsable")
}

// ============================================================
// Pipeline émission
// ============================================================

#[tokio::test]
async fn emission_seller_known_buyer_known_genere_cdv_200() {
    let (_pg, service) = setup().await;
    let exchange = invoice_exchange(&siret_for(SIREN_VENDEUR_OK), &siret_for(SIREN_VENDEUR_OK));

    let out = run_pipeline(service, ValidationMode::Emission, exchange).await;

    assert!(!out.has_errors(), "erreurs inattendues : {:?}", out.errors);
    assert_eq!(
        out.get_property("cdv.status_code").map(|s| s.as_str()),
        Some("200"),
        "CDV de dépôt attendu (200)"
    );
}

#[tokio::test]
async fn emission_seller_unknown_genere_cdv_213_rej_coh() {
    let (_pg, service) = setup().await;
    let exchange = invoice_exchange(&siret_for(SIREN_INCONNU), &siret_for(SIREN_VENDEUR_OK));

    let out = run_pipeline(service, ValidationMode::Emission, exchange).await;

    assert!(out.has_errors(), "vendeur inconnu doit générer une erreur");
    assert_eq!(
        out.get_property("cdv.status_code").map(|s| s.as_str()),
        Some("213"),
        "CDV de rejet attendu (213)"
    );

    let cdv = parse_cdv(&out);
    let ref_doc = cdv.referenced_documents.first().expect("aucun document référencé");
    let reason = ref_doc.statuses[0]
        .reason_code
        .as_deref()
        .expect("aucun reason_code");
    assert_eq!(reason, "REJ_COH", "vendeur inconnu attendu → REJ_COH");
}

#[tokio::test]
async fn emission_buyer_unknown_genere_cdv_213_dest_inc() {
    let (_pg, service) = setup().await;
    let exchange = invoice_exchange(&siret_for(SIREN_VENDEUR_OK), &siret_for(SIREN_INCONNU));

    let out = run_pipeline(service, ValidationMode::Emission, exchange).await;

    assert!(out.has_errors());
    assert_eq!(
        out.get_property("cdv.status_code").map(|s| s.as_str()),
        Some("213")
    );

    let cdv = parse_cdv(&out);
    let reason = cdv.referenced_documents[0].statuses[0]
        .reason_code
        .as_deref()
        .expect("aucun reason_code");
    assert_eq!(reason, "DEST_INC", "destinataire inconnu attendu → DEST_INC");
}

// ============================================================
// Pipeline réception
// ============================================================

#[tokio::test]
async fn reception_seller_known_genere_cdv_202() {
    let (_pg, service) = setup().await;
    // Acheteur inconnu : ignoré en réception
    let exchange = invoice_exchange(&siret_for(SIREN_VENDEUR_OK), &siret_for(SIREN_INCONNU));

    let out = run_pipeline(service, ValidationMode::Reception, exchange).await;

    assert!(!out.has_errors(), "erreurs inattendues : {:?}", out.errors);
    assert_eq!(
        out.get_property("cdv.status_code").map(|s| s.as_str()),
        Some("202"),
        "CDV de réception attendu (202)"
    );
}

#[tokio::test]
async fn reception_seller_unknown_genere_cdv_213_rej_coh() {
    let (_pg, service) = setup().await;
    let exchange = invoice_exchange(&siret_for(SIREN_INCONNU), &siret_for(SIREN_VENDEUR_OK));

    let out = run_pipeline(service, ValidationMode::Reception, exchange).await;

    assert!(out.has_errors());
    assert_eq!(
        out.get_property("cdv.status_code").map(|s| s.as_str()),
        Some("213")
    );

    let cdv = parse_cdv(&out);
    let reason = cdv.referenced_documents[0].statuses[0]
        .reason_code
        .as_deref()
        .expect("aucun reason_code");
    assert_eq!(reason, "REJ_COH");
}
