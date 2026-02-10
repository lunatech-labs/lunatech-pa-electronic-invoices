//! # pdp-invoice — Parsing et validation métier de factures électroniques
//!
//! Parsing de factures aux formats UBL 2.1, CII D22B et Factur-X (PDF/A-3),
//! avec détection automatique du format et validation métier EN16931.
//!
//! # Modules
//!
//! - **[`detector`]** — Détection automatique du format (UBL, CII, Factur-X) à partir du contenu
//! - **[`ubl`]** — Parser XML UBL 2.1 (Invoice et CreditNote) → [`InvoiceData`](pdp_core::model::InvoiceData)
//! - **[`cii`]** — Parser XML CII D22B (CrossIndustryInvoice) → [`InvoiceData`](pdp_core::model::InvoiceData)
//! - **[`facturx`]** — Parser PDF Factur-X : extraction du XML CII embarqué (factur-x.xml) via lopdf
//! - **[`validator`]** — Validation métier : champs obligatoires, cohérence montants, règles BR-FR
//! - **[`processor`]** — Processors pipeline : [`ParseProcessor`], [`ValidateProcessor`]
//!
//! # Exemple : parser une facture UBL
//!
//! ```no_run
//! use pdp_invoice::UblParser;
//!
//! let xml = std::fs::read_to_string("facture.xml").unwrap();
//! let invoice = UblParser::new().parse(&xml).unwrap();
//! println!("Facture {} du {}", invoice.invoice_number, invoice.issue_date.unwrap());
//! ```
//!
//! # Exemple : parser une facture CII
//!
//! ```no_run
//! use pdp_invoice::CiiParser;
//!
//! let xml = std::fs::read_to_string("facture_cii.xml").unwrap();
//! let invoice = CiiParser::new().parse(&xml).unwrap();
//! println!("Vendeur : {}", invoice.seller_name.unwrap_or_default());
//! ```
//!
//! # Exemple : parser un PDF Factur-X
//!
//! ```no_run
//! use pdp_invoice::FacturXParser;
//!
//! let pdf = std::fs::read("facturx.pdf").unwrap();
//! let invoice = FacturXParser::new().parse(&pdf).unwrap();
//! println!("Factur-X : {} ({})", invoice.invoice_number, invoice.source_format);
//! ```
//!
//! # Détection automatique
//!
//! ```no_run
//! use pdp_invoice::detect_format;
//!
//! let content = std::fs::read("document.xml").unwrap();
//! let format = detect_format(&content);
//! println!("Format détecté : {:?}", format);
//! ```

pub mod detector;
pub mod ubl;
pub mod cii;
pub mod facturx;
pub mod validator;
pub mod processor;

pub use detector::{detect_format, detect_document_type, FormatDetector};
pub use ubl::UblParser;
pub use cii::CiiParser;
pub use facturx::FacturXParser;
pub use validator::InvoiceValidator;
pub use processor::{ParseProcessor, ValidateProcessor};
