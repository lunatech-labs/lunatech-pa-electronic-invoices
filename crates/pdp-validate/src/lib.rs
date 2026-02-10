//! # pdp-validate — Validation XSD et Schematron de factures électroniques
//!
//! Validation structurelle (XSD) et métier (Schematron) des factures UBL et CII,
//! conforme aux normes EN16931, BR-FR et Factur-X.
//!
//! # Modules
//!
//! - **[`xsd`]** — Validation XSD via libxml2 : CII D22B, UBL 2.1, Factur-X EN16931/EXTENDED
//! - **[`schematron`]** — Validation Schematron via Saxon-HE : EN16931, BR-FR, Factur-X
//! - **[`processor`]** — Processor pipeline [`XmlValidateProcessor`]
//! - **[`error`]** — Rapport de validation : [`ValidationReport`], [`ValidationIssue`], [`ValidationLevel`]
//!
//! # 3 niveaux de validation
//!
//! 1. **XSD** — Validation structurelle (éléments, types, cardinalités)
//! 2. **Schematron EN16931 + BR-FR** — Règles métier européennes et françaises
//! 3. **Schematron Factur-X** — Règles PEPPOL, éléments vides, listes de codes
//!
//! # Exemple : validation XSD
//!
//! ```no_run
//! use std::path::Path;
//! use pdp_validate::XsdValidator;
//! use pdp_validate::xsd::XsdDocumentType;
//!
//! let validator = XsdValidator::new(Path::new("specs/"));
//! let xml = std::fs::read_to_string("facture_cii.xml").unwrap();
//! let report = validator.validate(&xml, &XsdDocumentType::CiiD22B);
//! println!("Valide : {}", report.is_valid());
//! ```
//!
//! # Exemple : validation Schematron
//!
//! ```no_run
//! use std::path::Path;
//! use pdp_validate::SchematronValidator;
//! use pdp_validate::schematron::SchematronType;
//!
//! let validator = SchematronValidator::new(Path::new("specs/"));
//! let xml = std::fs::read_to_string("facture_cii.xml").unwrap();
//! let report = validator.validate(&xml, &SchematronType::En16931Cii);
//! for issue in &report.issues {
//!     println!("[{}] {}: {}", issue.level, issue.rule_id, issue.message);
//! }
//! ```
//!
//! # Dépendances externes
//!
//! - **libxml2** — Validation XSD (binding C via `libxml`)
//! - **Saxon-HE** — Exécution des XSLT Schematron compilées

pub mod xsd;
pub mod schematron;
pub mod processor;
pub mod error;

pub use xsd::XsdValidator;
pub use schematron::SchematronValidator;
pub use processor::XmlValidateProcessor;
pub use error::{ValidationLevel, ValidationIssue, ValidationReport};
