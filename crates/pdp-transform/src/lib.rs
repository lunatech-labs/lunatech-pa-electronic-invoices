//! # pdp-transform — Conversion entre formats de facture
//!
//! Transformations entre UBL 2.1, CII D22B, Factur-X (PDF/A-3a) et PDF visuel.
//!
//! # Pipeline de transformation
//!
//! ```text
//! UBL ──XSLT──▶ CII ──FOP──▶ PDF
//!  │              │              │
//!  ├──XSLT+FOP+lopdf──▶ Factur-X (PDF/A-3a + XML embarqué)
//!  │              │
//! CII ──XSLT──▶ UBL
//! ```
//!
//! # Modules
//!
//! - **[`converter`]** — API unifiée [`convert_to`] pour les 9 chemins de conversion
//! - **[`xslt_engine`]** — Moteur XSLT 2.0 (UBL↔CII) via Saxon-HE
//! - **[`fop_engine`]** — Pipeline PDF : CII/UBL → XR → XSL-FO → PDF via Apache FOP
//! - **[`facturx_generator`]** — Génération Factur-X PDF/A-3a (XML embarqué + pièces jointes)
//! - **[`processor`]** — Processor pipeline [`TransformProcessor`]
//!
//! # Conversions supportées
//!
//! | Source | → CII | → UBL | → Factur-X | → PDF |
//! |--------|-------|-------|------------|-------|
//! | **UBL** | ✅ XSLT | — | ✅ XSLT+FOP+lopdf | ✅ FOP |
//! | **CII** | — | ✅ XSLT | ✅ FOP+lopdf | ✅ FOP |
//! | **Factur-X** | ✅ extraction | ✅ extraction+XSLT | — | ✅ identité |
//!
//! # Exemple : conversion UBL → CII
//!
//! ```no_run
//! use pdp_invoice::UblParser;
//! use pdp_transform::{convert_to, OutputFormat};
//!
//! let xml = std::fs::read_to_string("facture_ubl.xml").unwrap();
//! let invoice = UblParser::new().parse(&xml).unwrap();
//! let result = convert_to(&invoice, OutputFormat::CII).unwrap();
//! std::fs::write(&result.suggested_filename, result.as_string().unwrap()).unwrap();
//! ```
//!
//! # Exemple : génération PDF (sans Factur-X)
//!
//! ```no_run
//! use pdp_invoice::CiiParser;
//! use pdp_transform::{convert_to, OutputFormat};
//!
//! let xml = std::fs::read_to_string("facture_cii.xml").unwrap();
//! let invoice = CiiParser::new().parse(&xml).unwrap();
//! let result = convert_to(&invoice, OutputFormat::PDF).unwrap();
//! assert!(result.is_pdf());
//! std::fs::write("facture.pdf", &result.content).unwrap();
//! ```
//!
//! # Exemple : génération Factur-X
//!
//! ```no_run
//! use pdp_invoice::UblParser;
//! use pdp_transform::{convert_to, OutputFormat};
//!
//! let xml = std::fs::read_to_string("facture_ubl.xml").unwrap();
//! let invoice = UblParser::new().parse(&xml).unwrap();
//! let result = convert_to(&invoice, OutputFormat::FacturX).unwrap();
//! std::fs::write("facturx.pdf", &result.content).unwrap();
//! ```
//!
//! # Exemple : Factur-X avec pièces jointes
//!
//! ```no_run
//! use pdp_invoice::UblParser;
//! use pdp_transform::{convert_to, OutputFormat};
//! use pdp_core::model::InvoiceAttachment;
//!
//! let xml = std::fs::read_to_string("facture_ubl.xml").unwrap();
//! let mut invoice = UblParser::new().parse(&xml).unwrap();
//!
//! // Ajouter un bon de commande PDF
//! invoice.attachments.push(InvoiceAttachment {
//!     id: Some("BC-001".to_string()),
//!     description: Some("Bon de commande".to_string()),
//!     external_uri: None,
//!     embedded_content: Some(std::fs::read("bon_commande.pdf").unwrap()),
//!     mime_code: Some("application/pdf".to_string()),
//!     filename: Some("bon_commande.pdf".to_string()),
//! });
//!
//! // Ajouter un bordereau de livraison PNG
//! invoice.attachments.push(InvoiceAttachment {
//!     id: Some("BL-001".to_string()),
//!     description: Some("Bordereau de livraison".to_string()),
//!     external_uri: None,
//!     embedded_content: Some(std::fs::read("bordereau.png").unwrap()),
//!     mime_code: Some("image/png".to_string()),
//!     filename: Some("bordereau.png".to_string()),
//! });
//!
//! // Factur-X : PJ embarquées dans le PDF avec AFRelationship=Supplement
//! let result = convert_to(&invoice, OutputFormat::FacturX).unwrap();
//! std::fs::write("facturx_avec_pj.pdf", &result.content).unwrap();
//!
//! // CII : PJ encodées en base64 dans <ram:AdditionalReferencedDocument>
//! let result = convert_to(&invoice, OutputFormat::CII).unwrap();
//! std::fs::write("facture_avec_pj_cii.xml", result.as_string().unwrap()).unwrap();
//! ```
//!
//! # Dépendances externes
//!
//! - **Saxon-HE** (SaxonC natif ou SaxonJ Java) — transformations XSLT 2.0
//! - **Apache FOP** — rendu XSL-FO → PDF
//! - **qpdf** — correction header binaire PDF/A-3

pub mod converter;
pub mod processor;
pub mod ppf_flux1;
pub mod xslt_engine;
pub mod fop_engine;
pub mod facturx_generator;
#[cfg(feature = "saxonc_ffi")]
pub mod saxonc_ffi;

pub use converter::{convert, convert_to, xslt_transform, ConversionResult, OutputFormat, supported_conversions, supported_output_formats};
pub use processor::TransformProcessor;
pub use xslt_engine::{XsltEngine, XsltTransform};
pub use fop_engine::{FopEngine, SourceSyntax, XsltBackend, detect_xslt_backend};
pub use facturx_generator::{FacturXGenerator, FacturXLevel, FacturXResult};
pub use ppf_flux1::{PpfFlux1Processor, Flux1ProfileStrategy};
