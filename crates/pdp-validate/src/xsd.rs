use std::path::{Path, PathBuf};
use libxml::parser::Parser as XmlParser;
use libxml::schemas::{SchemaParserContext, SchemaValidationContext};
use tracing;

use crate::error::{ValidationIssue, ValidationLevel, ValidationReport};

/// Type de document à valider par XSD
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum XsdDocumentType {
    /// Facture UBL 2.1
    UblInvoice,
    /// Avoir UBL 2.1
    UblCreditNote,
    /// Facture CII D22B (profil F1BASE restreint)
    CiiInvoice,
    /// Facture CII D22B complet (non restreint)
    CiiD22B,
    /// Factur-X 1.08 EN16931 (CII embarqué)
    FacturXEN16931,
    /// Factur-X 1.08 EXTENDED (CII embarqué)
    FacturXExtended,
    /// CDAR D22B
    Cdar,
    /// Flux 1 Full CII D22B
    F1FullCii,
    /// Flux 1 Full UBL Invoice
    F1FullUblInvoice,
    /// Flux 1 Full UBL CreditNote
    F1FullUblCreditNote,
}

impl std::fmt::Display for XsdDocumentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            XsdDocumentType::UblInvoice => write!(f, "UBL Invoice"),
            XsdDocumentType::UblCreditNote => write!(f, "UBL CreditNote"),
            XsdDocumentType::CiiInvoice => write!(f, "CII Invoice (F1BASE)"),
            XsdDocumentType::CiiD22B => write!(f, "CII D22B"),
            XsdDocumentType::FacturXEN16931 => write!(f, "Factur-X EN16931"),
            XsdDocumentType::FacturXExtended => write!(f, "Factur-X EXTENDED"),
            XsdDocumentType::Cdar => write!(f, "CDAR"),
            XsdDocumentType::F1FullCii => write!(f, "F1Full CII"),
            XsdDocumentType::F1FullUblInvoice => write!(f, "F1Full UBL Invoice"),
            XsdDocumentType::F1FullUblCreditNote => write!(f, "F1Full UBL CreditNote"),
        }
    }
}

/// Validateur XSD utilisant libxml2
pub struct XsdValidator {
    /// Répertoire racine des specs
    specs_dir: PathBuf,
}

impl XsdValidator {
    /// Crée un validateur XSD avec le répertoire des specs
    pub fn new(specs_dir: &Path) -> Self {
        Self {
            specs_dir: specs_dir.to_path_buf(),
        }
    }

    /// Détecte automatiquement le type de document XML
    pub fn detect_document_type(xml: &str) -> Option<XsdDocumentType> {
        if xml.contains("CrossDomainAcknowledgementAndResponse") {
            Some(XsdDocumentType::Cdar)
        } else if xml.contains("urn:oasis:names:specification:ubl:schema:xsd:Invoice-2")
            || xml.contains("<Invoice ")
            || xml.contains("<Invoice>")
        {
            Some(XsdDocumentType::UblInvoice)
        } else if xml.contains("urn:oasis:names:specification:ubl:schema:xsd:CreditNote-2")
            || xml.contains("<CreditNote ")
            || xml.contains("<CreditNote>")
        {
            Some(XsdDocumentType::UblCreditNote)
        } else if xml.contains("CrossIndustryInvoice") {
            Some(XsdDocumentType::CiiInvoice)
        } else {
            None
        }
    }

    /// Retourne le chemin du XSD racine pour un type de document
    fn xsd_path(&self, doc_type: &XsdDocumentType) -> PathBuf {
        match doc_type {
            XsdDocumentType::UblInvoice => {
                self.specs_dir.join("xsd/ubl/base/F1BASE_UBL-invoice-2.1.xsd")
            }
            XsdDocumentType::UblCreditNote => {
                self.specs_dir.join("xsd/ubl/base/F1BASE_UBL-CreditNote-2.1.xsd")
            }
            XsdDocumentType::CiiInvoice => {
                self.specs_dir.join("xsd/cii/base/uncefact/data/standard/F1BASE_CrossIndustryInvoice_100pD22B.xsd")
            }
            XsdDocumentType::CiiD22B => {
                self.specs_dir.join("xsd/cii-d22b/CrossIndustryInvoice_100pD22B.xsd")
            }
            XsdDocumentType::FacturXEN16931 => {
                self.specs_dir.join("xsd/facturx-1.08/EN16931/Factur-X_1.08_EN16931.xsd")
            }
            XsdDocumentType::FacturXExtended => {
                self.specs_dir.join("xsd/facturx-1.08/EXTENDED/Factur-X_1.08_EXTENDED.xsd")
            }
            XsdDocumentType::Cdar => {
                self.specs_dir.join("xsd/cdar/CrossDomainAcknowledgementAndResponse_100pD22B.xsd")
            }
            XsdDocumentType::F1FullCii => {
                self.specs_dir.join("xsd/cii/full/uncefact/data/standard/F1FULL_CrossIndustryInvoice_100pD22B.xsd")
            }
            XsdDocumentType::F1FullUblInvoice => {
                self.specs_dir.join("xsd/ubl/full/F1FULL_UBL_invoice-2.1.xsd")
            }
            XsdDocumentType::F1FullUblCreditNote => {
                self.specs_dir.join("xsd/ubl/full/F1FULL_UBL_CreditNote-2.1.xsd")
            }
        }
    }

    /// Valide un document XML contre son XSD
    pub fn validate(&self, xml: &str, doc_type: &XsdDocumentType) -> ValidationReport {
        let mut report = ValidationReport::new(&format!("XSD {}", doc_type));

        let xsd_path = self.xsd_path(doc_type);
        if !xsd_path.exists() {
            report.add_issue(ValidationIssue {
                level: ValidationLevel::Fatal,
                rule_id: "XSD-000".to_string(),
                message: format!("Fichier XSD introuvable: {}", xsd_path.display()),
                location: None,
                source: "XSD".to_string(),
            });
            return report;
        }

        tracing::debug!(
            doc_type = %doc_type,
            xsd = %xsd_path.display(),
            "Validation XSD en cours"
        );

        // Parser le XSD
        let xsd_path_str = xsd_path.to_string_lossy().to_string();
        let mut schema_parser = SchemaParserContext::from_file(&xsd_path_str);

        // Créer le contexte de validation à partir du parser
        let mut validation_ctx = match SchemaValidationContext::from_parser(&mut schema_parser) {
            Ok(ctx) => ctx,
            Err(errors) => {
                for error in &errors {
                    report.add_issue(ValidationIssue {
                        level: ValidationLevel::Fatal,
                        rule_id: "XSD-001".to_string(),
                        message: error.message.clone().unwrap_or_else(|| "Erreur de parsing XSD".to_string()),
                        location: None,
                        source: "XSD".to_string(),
                    });
                }
                if errors.is_empty() {
                    report.add_issue(ValidationIssue {
                        level: ValidationLevel::Fatal,
                        rule_id: "XSD-001".to_string(),
                        message: format!("Impossible de parser le schéma XSD: {}", xsd_path.display()),
                        location: None,
                        source: "XSD".to_string(),
                    });
                }
                return report;
            }
        };

        // Parser le document XML
        let xml_parser = XmlParser::default();
        let doc = match xml_parser.parse_string(xml) {
            Ok(doc) => doc,
            Err(e) => {
                report.add_issue(ValidationIssue {
                    level: ValidationLevel::Fatal,
                    rule_id: "XSD-002".to_string(),
                    message: format!("XML mal formé: {}", e),
                    location: None,
                    source: "XSD".to_string(),
                });
                return report;
            }
        };

        // Valider le document contre le schéma
        if let Err(errors) = validation_ctx.validate_document(&doc) {
            for error in errors {
                report.add_issue(ValidationIssue {
                    level: ValidationLevel::Error,
                    rule_id: "XSD-100".to_string(),
                    message: error.message.clone().unwrap_or_else(|| "Erreur XSD inconnue".to_string()),
                    location: None,
                    source: "XSD".to_string(),
                });
            }
        }

        if report.is_valid() {
            tracing::info!(doc_type = %doc_type, "Validation XSD réussie");
        } else {
            tracing::warn!(
                doc_type = %doc_type,
                errors = report.error_count(),
                fatals = report.fatal_count(),
                "Validation XSD échouée"
            );
        }

        report
    }

    /// Valide un document XML avec détection automatique du type
    pub fn validate_auto(&self, xml: &str) -> ValidationReport {
        match Self::detect_document_type(xml) {
            Some(doc_type) => self.validate(xml, &doc_type),
            None => {
                let mut report = ValidationReport::new("XSD");
                report.add_issue(ValidationIssue {
                    level: ValidationLevel::Fatal,
                    rule_id: "XSD-010".to_string(),
                    message: "Type de document XML non reconnu pour la validation XSD".to_string(),
                    location: None,
                    source: "XSD".to_string(),
                });
                report
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn specs_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../specs")
    }

    #[test]
    fn test_detect_ubl_invoice() {
        let xml = r#"<?xml version="1.0"?><Invoice xmlns="urn:oasis:names:specification:ubl:schema:xsd:Invoice-2"></Invoice>"#;
        assert_eq!(XsdValidator::detect_document_type(xml), Some(XsdDocumentType::UblInvoice));
    }

    #[test]
    fn test_detect_cii() {
        let xml = r#"<?xml version="1.0"?><rsm:CrossIndustryInvoice xmlns:rsm="urn:un:unece:uncefact:data:standard:CrossIndustryInvoice:100"></rsm:CrossIndustryInvoice>"#;
        assert_eq!(XsdValidator::detect_document_type(xml), Some(XsdDocumentType::CiiInvoice));
    }

    #[test]
    fn test_detect_cdar() {
        let xml = r#"<?xml version="1.0"?><rsm:CrossDomainAcknowledgementAndResponse></rsm:CrossDomainAcknowledgementAndResponse>"#;
        assert_eq!(XsdValidator::detect_document_type(xml), Some(XsdDocumentType::Cdar));
    }

    #[test]
    fn test_detect_unknown() {
        assert_eq!(XsdValidator::detect_document_type("<html></html>"), None);
    }

    #[test]
    fn test_validate_malformed_xml() {
        let validator = XsdValidator::new(&specs_dir());
        let report = validator.validate("not xml at all", &XsdDocumentType::UblInvoice);
        assert!(!report.is_valid());
    }

    #[test]
    fn test_validate_ubl_fixture() {
        let specs = specs_dir();
        if !specs.join("xsd/ubl/base/F1BASE_UBL-invoice-2.1.xsd").exists() {
            eprintln!("XSD UBL non disponible, test ignoré");
            return;
        }
        let validator = XsdValidator::new(&specs);
        // Utiliser un exemple officiel EN16931
        let example_path = specs.join("examples/en16931/ubl/ubl-tc434-example1.xml");
        if example_path.exists() {
            let xml = std::fs::read_to_string(&example_path).unwrap();
            let report = validator.validate(&xml, &XsdDocumentType::UblInvoice);
            println!("UBL XSD validation: {}", report.summary());
            for issue in &report.issues {
                println!("  {}", issue);
            }
        }
    }

    #[test]
    fn test_validate_cii_fixture() {
        let specs = specs_dir();
        if !specs.join("xsd/cii/base/uncefact/data/standard/F1BASE_CrossIndustryInvoice_100pD22B.xsd").exists() {
            eprintln!("XSD CII non disponible, test ignoré");
            return;
        }
        let validator = XsdValidator::new(&specs);
        let example_path = specs.join("examples/en16931/cii/CII_example1.xml");
        if example_path.exists() {
            let xml = std::fs::read_to_string(&example_path).unwrap();
            let report = validator.validate(&xml, &XsdDocumentType::CiiInvoice);
            println!("CII XSD validation: {}", report.summary());
            for issue in &report.issues {
                println!("  {}", issue);
            }
        }
    }

    #[test]
    fn test_validate_cdar_fixture() {
        let specs = specs_dir();
        if !specs.join("xsd/cdar/CrossDomainAcknowledgementAndResponse_100pD22B.xsd").exists() {
            eprintln!("XSD CDAR non disponible, test ignoré");
            return;
        }
        let validator = XsdValidator::new(&specs);
        let example_path = specs.join("examples/xp-z12/cdar/_Cas Nominal/UC1_F202500003_01-CDV-200_Deposee.xml");
        if example_path.exists() {
            let xml = std::fs::read_to_string(&example_path).unwrap();
            let report = validator.validate_auto(&xml);
            println!("CDAR XSD validation: {}", report.summary());
            for issue in &report.issues {
                println!("  {}", issue);
            }
        }
    }
}
