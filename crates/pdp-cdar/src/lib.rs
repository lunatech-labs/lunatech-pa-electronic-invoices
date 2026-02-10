//! # pdp-cdar — Comptes-rendus De Vie (CDV) au format CDAR D23B
//!
//! Génération, parsing et traitement des statuts de cycle de vie des factures,
//! conforme au format UN/CEFACT CrossDomainAcknowledgementAndResponse (CDAR) D23B.
//!
//! # Modules
//!
//! - **[`model`]** — Modèle de données CDAR : statuts, rôles, codes action, parties
//! - **[`generator`]** — Génération de XML CDAR à partir du modèle
//! - **[`parser`]** — Parsing de XML CDAR vers le modèle
//! - **[`processor`]** — Processors pipeline : [`CdarProcessor`] (émission), [`CdvReceptionProcessor`] (réception)
//!
//! # Statuts de cycle de vie
//!
//! | Code | Statut | Phase |
//! |------|--------|-------|
//! | 200 | Déposée | Transmission |
//! | 201 | Émise | Transmission |
//! | 202 | Reçue | Transmission |
//! | 204 | Prise en charge | Traitement |
//! | 205 | Approuvée | Traitement |
//! | 207 | En litige | Traitement |
//! | 210 | Refusée | Traitement |
//! | 212 | Encaissée | Traitement |
//! | 213 | Rejetée | Transmission |
//!
//! # Exemple : générer un CDV « Déposée »
//!
//! ```no_run
//! use pdp_cdar::CdarGenerator;
//! use pdp_core::model::{InvoiceData, InvoiceFormat};
//!
//! let generator = CdarGenerator::new("123456789", "Ma PDP");
//! let invoice = InvoiceData::new("FA-2025-001".to_string(), InvoiceFormat::CII);
//! let cdv = generator.generate_deposee(&invoice, "380");
//! println!("CDV : statut {:?}, type {:?}", cdv.status_code(), cdv.type_code);
//! ```
//!
//! # Exemple : parser un CDV reçu
//!
//! ```no_run
//! use pdp_cdar::CdarParser;
//!
//! let parser = CdarParser::new();
//! let xml = std::fs::read_to_string("cdv_200.xml").unwrap();
//! let cdv = parser.parse(&xml).unwrap();
//! println!("CDV {} : statut {:?}", cdv.document_id, cdv.status_code());
//! ```
//!
//! # Conformité
//!
//! - Format : UN/CEFACT CDAR D23B
//! - Règles : BR-FR-CDV-01 à BR-FR-CDV-14
//! - XSD : `specs/xsd/cdar-d23b/`

pub mod model;
pub mod generator;
pub mod parser;
pub mod processor;

pub use model::{
    CdvResponse, CdarResponse, CdvTypeCode, InvoiceStatusCode, TransmissionStatusCode,
    RoleCode, StatusReasonCode, ActionCode, BusinessProcessCode, TradeParty,
    ReferencedDocument, DocumentStatus, DocumentCharacteristic, CdarValidationError,
};
pub use generator::CdarGenerator;
pub use parser::CdarParser;
pub use processor::{CdarProcessor, CdvReceptionProcessor, DocumentTypeRouter, IrrecevabiliteProcessor};
