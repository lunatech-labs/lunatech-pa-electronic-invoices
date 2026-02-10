//! # pdp-ereporting — E-reporting flux 10 (transactions et paiements)
//!
//! Génération des déclarations e-reporting conformes au XSD PPF V1.0,
//! pour les flux 10.1 (transactions ventes), 10.2 (paiements ventes),
//! 10.3 (transactions acquisitions) et 10.4 (paiements acquisitions).
//!
//! # Modules
//!
//! - **[`model`]** — Modèle de données : transactions, paiements, périodes, catégories
//! - **[`generator`]** — Génération XML conforme au XSD PPF (`ereporting.xsd`)
//!
//! # Types de flux
//!
//! | Flux | Description | Contenu |
//! |------|-------------|---------|
//! | 10.1 | Transactions ventes | Factures détaillées ou agrégées (TLB1/TPS1/TNT1/TMA1) |
//! | 10.2 | Paiements ventes | Encaissements par facture ou agrégés |
//! | 10.3 | Transactions acquisitions | Idem côté acheteur |
//! | 10.4 | Paiements acquisitions | Idem côté acheteur |
//!
//! # Exemple
//!
//! ```ignore
//! use pdp_ereporting::EReportingGenerator;
//!
//! let generator = EReportingGenerator::new("123456789", "Ma PDP");
//!
//! // Convertir des factures en transactions
//! let tx = EReportingGenerator::invoice_to_transaction(&invoice);
//!
//! // Créer un rapport de transactions (flux 10.1)
//! let report = generator.create_transactions_report(
//!     "RPT-001", "123456789", "Ma Société", "2025-01-01", "2025-01-31",
//!     vec![tx],
//! );
//! let xml = generator.to_xml(&report).unwrap();
//! ```
//!
//! # Conformité
//!
//! - XSD : `specs/xsd/ereporting/` (v3.1)
//! - Code interface PPF : `FFE1025A`

pub mod model;
pub mod generator;

pub use model::*;
pub use generator::EReportingGenerator;
