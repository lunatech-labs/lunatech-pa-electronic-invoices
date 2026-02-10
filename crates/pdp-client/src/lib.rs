//! # pdp-client — Clients PPF, AFNOR et Annuaire
//!
//! Clients pour communiquer avec la Plateforme Publique de Facturation (PPF)
//! et les autres PDP via les protocoles SFTP et HTTP (AFNOR XP Z12-013).
//!
//! # Modules
//!
//! - **[`ppf`]** — Construction de flux PPF : nommage tar.gz, codes interface, flux F1/F6/F10
//! - **[`afnor`]** — Client HTTP AFNOR Flow Service v1.1.0 (POST /v1/flows)
//! - **[`annuaire`]** — Client Annuaire PPF (PISTE) et AFNOR Directory Service
//! - **[`producer`]** — Producers : [`PpfSftpProducer`] (dépôt SFTP PPF), [`AfnorFlowProducer`] (HTTP AFNOR)
//! - **[`auth`]** — Authentification PISTE (OAuth2 JWT Bearer)
//! - **[`attachment`]** — Gestion des pièces jointes (BG-24)
//! - **[`model`]** — Modèles partagés (réponses API, statuts)
//! - **[`error`]** — Type d'erreur [`ClientError`]
//!
//! # Communication PPF (SFTP)
//!
//! ```text
//! PDP ──SFTP──▶ PPF
//!   │  tar.gz nommé : {CODE_INTERFACE}_{CODE_APP}_{ID_FLUX}.tar.gz
//!   │  Contenu : fichiers XML (F1, F6, F10)
//!   │  Auth : clé RSA X509v3
//! ```
//!
//! # Exemple : construire un flux F1
//!
//! ```no_run
//! use pdp_client::{CodeInterface, PpfFluxConfig, build_tar_gz, FluxFile, flux_envelope_name};
//!
//! let files = vec![FluxFile {
//!     filename: "Base_facture_001.xml".to_string(),
//!     content: b"<Invoice/>".to_vec(),
//! }];
//! let tar_gz = build_tar_gz(&files).unwrap();
//! let name = flux_envelope_name(CodeInterface::F1Ubl, "AAA123", "0111000000000000001").unwrap();
//! // → "FFE0111A_AAA123_AAA1230111000000000000001.tar.gz"
//! ```
//!
//! # Communication PDP↔PDP (AFNOR)
//!
//! ```ignore
//! use pdp_client::AfnorFlowClient;
//! use pdp_client::afnor::{AfnorFlowConfig, AfnorFlowInfo};
//! use pdp_client::auth::PisteAuth;
//!
//! async fn example() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = AfnorFlowConfig {
//!         base_url: "https://pdp.example.com/flow-service".to_string(),
//!         partner_id: "PARTNER01".to_string(),
//!     };
//!     let auth = PisteAuth::new("https://auth.example.com", "client_id", "client_secret");
//!     let client = AfnorFlowClient::new(config, auth);
//!
//!     let flow_info = AfnorFlowInfo::new("facture.xml", "CII", "CIUS", b"<Invoice/>");
//!     let ack = client.envoyer_flux(&flow_info, "facture.xml", b"<Invoice/>").await?;
//!     println!("Statut : {:?}", ack.status);
//!     Ok(())
//! }
//! ```
//!
//! # Codes interface PPF
//!
//! | Code | Flux | Format |
//! |------|------|--------|
//! | `FFE0111A` | F1 e-invoicing | UBL |
//! | `FFE0112A` | F1 e-invoicing | CII |
//! | `FFE0614A` | F6 CDV factures | CDAR |
//! | `FFE0654A` | F6 CDV statuts | CDAR |
//! | `FFE1025A` | F10 e-reporting | Spécifique |

pub mod auth;
pub mod error;
pub mod model;
pub mod ppf;
pub mod afnor;
pub mod annuaire;
pub mod producer;
pub mod attachment;

pub use auth::PisteAuth;
pub use error::ClientError;
pub use ppf::{CodeInterface, ProfilF1, PpfFluxConfig, FluxFile, build_tar_gz, flux_envelope_name, f1_inner_filename};
pub use afnor::AfnorFlowClient;
pub use annuaire::{AnnuaireClient, AfnorDirectoryClient};
pub use producer::{PpfSftpProducer, PpfSftpProducerConfig, AfnorFlowProducer, Destination};
