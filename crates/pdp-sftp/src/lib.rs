//! # pdp-sftp — Connecteur SFTP (ingestion et distribution)
//!
//! Consumer et producer SFTP pour l'ingestion de factures et la distribution
//! de flux vers le PPF ou d'autres partenaires, via `russh` / `russh-sftp`.
//!
//! # Modules
//!
//! - **[`consumer`]** — [`SftpConsumer`] : polling d'un répertoire SFTP distant, téléchargement des fichiers
//! - **[`producer`]** — [`SftpProducer`] : dépôt de fichiers sur un serveur SFTP distant
//! - **[`config`]** — [`SftpConfig`] : configuration de connexion (hôte, port, clé, répertoires)
//!
//! # Exemple : configuration
//!
//! ```rust
//! use pdp_sftp::SftpConfig;
//!
//! let config = SftpConfig {
//!     host: "sftp.ppf.gouv.fr".to_string(),
//!     port: 22,
//!     username: "pdp_aaa123".to_string(),
//!     private_key_path: Some("/app/keys/id_rsa".to_string()),
//!     remote_path: "/sas/depot".to_string(),
//!     ..Default::default()
//! };
//! ```
//!
//! # Protocole
//!
//! - **SFTP** (SSH File Transfer Protocol) — pas d'API REST
//! - Authentification par clé publique RSA (certificat X509v3)
//! - Chiffrement AES128_CBC / AES256_CBC

pub mod config;
pub mod consumer;
pub mod hostkey;
pub mod producer;

pub use consumer::SftpConsumer;
pub use producer::SftpProducer;
pub use config::SftpConfig;
