//! # pdp-trace — Traçabilité des flux et archivage Elasticsearch
//!
//! Persistance des factures, PDF et événements dans Elasticsearch.
//! Un index par SIREN (endpoint = client = numéro SIREN).
//!
//! # Modules
//!
//! - **[`store`]** — [`TraceStore`] : persistance Elasticsearch (exchanges, événements, XML, PDF)
//! - **[`processor`]** — [`TraceProcessor`] : processor pipeline qui enregistre chaque exchange
//! - **[`init`]** — [`init_tracing`] : initialisation du subscriber `tracing` (stdout + filtrage)
//!
//! # Architecture
//!
//! - **Un index par SIREN** : `pdp-{siren}` (ex: `pdp-123456789`)
//! - Chaque document contient : métadonnées facture + XML brut + PDF base64 + événements
//! - Recherche full-text dans les XML, filtrage par SIREN, statut, date
//! - Archivage naturel : tout est dans Elasticsearch
//!
//! # Exemple
//!
//! ```no_run
//! use pdp_trace::{TraceStore, init_tracing};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! init_tracing();
//! let store = TraceStore::new("http://localhost:9200").await?;
//! let stats = store.get_stats().await?;
//! println!("Total exchanges : {}", stats.total_exchanges);
//!
//! // Recherche full-text dans les XML d'un client
//! let results = store.search_xml("FA-2025-001", Some("123456789")).await?;
//!
//! // Liste des SIREN connus
//! let sirens = store.list_sirens().await?;
//! # Ok(())
//! # }
//! ```

pub mod store;
pub mod processor;
pub mod init;

pub use store::TraceStore;
pub use processor::TraceProcessor;
pub use init::init_tracing;
