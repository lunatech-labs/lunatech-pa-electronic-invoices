//! Tests d'intégration pour `AnnuaireStore::lookup_code_routage()`.
//!
//! Utilisé par l'endpoint AFNOR `GET /v1/routing-code/siret:{siret}/code:{routing-id}`
//! (XP Z12-013 V1.2.0 Annexe B).
//!
//! Démarre un Postgres jetable via testcontainers pour chaque test.
//! Prérequis : Docker / Podman démarré.

use pdp_annuaire::model::{CodeRoutage, MotifPresence, Statut};
use pdp_annuaire::AnnuaireStore;
use sqlx::postgres::PgPoolOptions;
use testcontainers::runners::AsyncRunner;
use testcontainers::ContainerAsync;
use testcontainers_modules::postgres::Postgres;

async fn setup() -> (ContainerAsync<Postgres>, AnnuaireStore) {
    let container = Postgres::default().start().await.unwrap();
    let host = container.get_host().await.unwrap();
    let port = container.get_host_port_ipv4(5432).await.unwrap();
    let url = format!("postgres://postgres:postgres@{}:{}/postgres", host, port);

    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(&url)
        .await
        .unwrap();
    let store = AnnuaireStore::new(pool);
    store.migrate().await.unwrap();

    // Insert un code routage de test
    let codes = vec![CodeRoutage {
        id_instance: 1,
        motif_presence: MotifPresence::Creation,
        statut: Statut::Actif,
        siret: "12345678901234".to_string(),
        id_routage: "0224ABC".to_string(),
        qualifiant_routage: "0224".to_string(),
        nom: "Service Comptabilité".to_string(),
        adresse_1: Some("10 rue de la Paix".to_string()),
        adresse_2: None,
        adresse_3: None,
        localite: Some("Paris".to_string()),
        code_postal: Some("75001".to_string()),
        subdivision_pays: None,
        code_pays: Some("FR".to_string()),
        engagement_juridique: Some(true),
    }];
    store.insert_codes_routage(&codes).await.unwrap();
    (container, store)
}

#[tokio::test]
async fn test_lookup_code_routage_existing() {
    let (_container, store) = setup().await;

    let result = store
        .lookup_code_routage("12345678901234", "0224ABC")
        .await
        .expect("lookup ok");

    let row = result.expect("code routage trouvé");
    assert_eq!(row.siret, "12345678901234");
    assert_eq!(row.id_routage, "0224ABC");
    assert_eq!(row.nom, "Service Comptabilité");
    assert_eq!(row.statut, "A");
    assert_eq!(row.adresse_1.as_deref(), Some("10 rue de la Paix"));
    assert_eq!(row.localite.as_deref(), Some("Paris"));
    assert_eq!(row.code_postal.as_deref(), Some("75001"));
}

#[tokio::test]
async fn test_lookup_code_routage_unknown_siret() {
    let (_container, store) = setup().await;

    let result = store
        .lookup_code_routage("99999999999999", "0224ABC")
        .await
        .expect("lookup ok");
    assert!(result.is_none(), "SIRET inconnu doit retourner None");
}

#[tokio::test]
async fn test_lookup_code_routage_unknown_id() {
    let (_container, store) = setup().await;

    let result = store
        .lookup_code_routage("12345678901234", "0224ZZZ")
        .await
        .expect("lookup ok");
    assert!(result.is_none(), "id_routage inconnu doit retourner None");
}
