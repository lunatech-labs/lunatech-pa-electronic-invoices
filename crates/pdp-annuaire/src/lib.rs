//! Annuaire PPF — copie locale et synchronisation
//!
//! Ce crate implémente :
//! - Le parsing streaming XML des flux F14 (export annuaire PPF)
//! - Le stockage en PostgreSQL (5 tables + métadonnées de synchro)
//! - La résolution de routage locale (4 mailles de spécificité)
//!
//! # Architecture
//!
//! ```text
//! SFTP (tar.gz F14)
//!   → parser::parse_f14()    // streaming XML, 10+ Go OK
//!   → ingest::ingest_f14()   // batch insert PostgreSQL
//!   → db::AnnuaireStore      // résolution de routage locale
//! ```

pub mod db;
pub mod ingest;
pub mod model;
pub mod parser;

pub use db::{AnnuaireStore, EtablissementRow, PlateformeRow};
pub use ingest::ingest_f14;
pub use model::*;
pub use parser::parse_f14;
