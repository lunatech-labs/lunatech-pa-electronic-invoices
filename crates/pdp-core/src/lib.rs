//! # pdp-core — Moteur pipeline et modèle de données
//!
//! Crate fondamentale de la PDP Facture. Fournit :
//!
//! - **[`Exchange`]** — Enveloppe d'un message circulant dans le pipeline (facture + métadonnées)
//! - **[`Processor`]** — Trait async pour les étapes de traitement (parse, validate, transform…)
//! - **[`Endpoint`]** — Abstraction des sources et destinations (fichier, SFTP…)
//! - **[`Route`] / [`RouteBuilder`]** — Définition déclarative des routes de traitement
//! - **[`Router`]** — Orchestrateur qui exécute les routes avec des channels tokio
//! - **[`model::InvoiceData`]** — Modèle de facture conforme EN16931 / specs v3.0
//! - **[`archive`]** — Utilitaires de création/lecture d'archives ZIP et tar.gz
//! - **[`error::PdpError`]** — Type d'erreur unifié pour tout le workspace
//!
//! # Architecture pipeline
//!
//! ```text
//! Endpoint(source) → Exchange → [Processor₁ → Processor₂ → …] → Endpoint(destination)
//! ```
//!
//! # Exemple
//!
//! ```rust
//! use pdp_core::{RouteBuilder, Exchange};
//! use pdp_core::model::{InvoiceData, InvoiceFormat};
//!
//! // Créer une facture
//! let invoice = InvoiceData::new("FA-2025-001".to_string(), InvoiceFormat::CII);
//!
//! // Créer un exchange
//! let exchange = Exchange::new(b"<Invoice/>".to_vec());
//! ```
//!
//! # Archives ZIP / tar.gz
//!
//! ```rust
//! use pdp_core::archive::{TarGzBuilder, ZipBuilder};
//!
//! let tgz = TarGzBuilder::new()
//!     .add("facture.xml", b"<Invoice/>")
//!     .build()
//!     .unwrap();
//!
//! let zip = ZipBuilder::new()
//!     .add("facture.xml", b"<Invoice/>")
//!     .build()
//!     .unwrap();
//! ```

pub mod alert;
pub mod archive;
pub mod channel;
pub mod exchange;
pub mod processor;
pub mod reception;
pub mod endpoint;
pub mod retry;
pub mod route;
pub mod router;
pub mod error;
pub mod model;

pub use exchange::Exchange;
pub use processor::{Processor, TenantTagProcessor};
pub use endpoint::{Endpoint, EndpointType};
pub use channel::ChannelConsumer;
pub use route::{Route, RouteBuilder};
pub use router::Router;
pub use error::PdpError;
pub use retry::{RetryProducer, DeadLetterProducer};
pub use alert::{AlertErrorHandler, AlertLevel, AlertClassifier, Alert};
