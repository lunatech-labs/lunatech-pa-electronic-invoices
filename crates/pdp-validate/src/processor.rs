use async_trait::async_trait;
use std::path::PathBuf;

use pdp_core::error::{PdpError, PdpResult};
use pdp_core::exchange::Exchange;
use pdp_core::model::{FlowStatus, InvoiceFormat};
use pdp_core::processor::Processor;

use crate::error::ValidationLevel;
use crate::schematron::{SchematronValidator, SchematronType};
use crate::xsd::{XsdValidator, XsdDocumentType};

/// Processor de validation XML complète : XSD + Schematron (EN16931 + BR-FR)
/// Ce processor remplace/complète le ValidateProcessor de pdp-invoice
/// en ajoutant la validation structurelle (XSD) et les règles métier (Schematron).
pub struct XmlValidateProcessor {
    /// Répertoire racine des specs
    specs_dir: PathBuf,
    /// Activer la validation XSD
    xsd_enabled: bool,
    /// Activer la validation Schematron EN16931
    en16931_enabled: bool,
    /// Activer la validation Schematron BR-FR
    br_fr_enabled: bool,
    /// Mode strict : les erreurs bloquent le flux
    strict: bool,
}

impl XmlValidateProcessor {
    /// Crée un processor avec toutes les validations activées en mode strict
    pub fn new(specs_dir: &str) -> Self {
        Self {
            specs_dir: PathBuf::from(specs_dir),
            xsd_enabled: true,
            en16931_enabled: true,
            br_fr_enabled: true,
            strict: true,
        }
    }

    /// Crée un processor avec configuration personnalisée
    pub fn with_options(
        specs_dir: &str,
        xsd: bool,
        en16931: bool,
        br_fr: bool,
        strict: bool,
    ) -> Self {
        Self {
            specs_dir: PathBuf::from(specs_dir),
            xsd_enabled: xsd,
            en16931_enabled: en16931,
            br_fr_enabled: br_fr,
            strict,
        }
    }

    /// Détermine le format de la facture à partir de l'exchange
    fn detect_format(exchange: &Exchange) -> Option<InvoiceFormat> {
        // D'abord vérifier le header posé par ParseProcessor
        if let Some(fmt) = exchange.get_header("invoice.format") {
            return match fmt.as_str() {
                "UBL" => Some(InvoiceFormat::UBL),
                "CII" => Some(InvoiceFormat::CII),
                "FacturX" => Some(InvoiceFormat::FacturX),
                _ => None,
            };
        }
        // Sinon détecter depuis le body
        if let Ok(xml) = exchange.body_as_str() {
            if xml.contains("CrossDomainAcknowledgementAndResponse") {
                return None; // CDAR, pas une facture
            }
            if xml.contains("urn:oasis:names:specification:ubl:schema:xsd:Invoice-2")
                || xml.contains("urn:oasis:names:specification:ubl:schema:xsd:CreditNote-2")
            {
                return Some(InvoiceFormat::UBL);
            }
            if xml.contains("CrossIndustryInvoice") {
                return Some(InvoiceFormat::CII);
            }
        }
        None
    }

    /// Détermine si c'est un CDAR
    fn is_cdar(exchange: &Exchange) -> bool {
        if let Ok(xml) = exchange.body_as_str() {
            xml.contains("CrossDomainAcknowledgementAndResponse")
        } else {
            false
        }
    }
}

#[async_trait]
impl Processor for XmlValidateProcessor {
    fn name(&self) -> &str {
        "XmlValidateProcessor"
    }

    async fn process(&self, mut exchange: Exchange) -> PdpResult<Exchange> {
        let xml = exchange.body_as_str().map_err(|e| {
            PdpError::ValidationError(format!("Le body n'est pas du XML valide UTF-8: {}", e))
        })?.to_string();

        let is_cdar = Self::is_cdar(&exchange);
        let format = Self::detect_format(&exchange);

        exchange.set_status(FlowStatus::Validating);

        let mut total_errors = 0usize;
        let mut total_warnings = 0usize;
        let mut all_issues = Vec::new();

        // === Validation XSD ===
        if self.xsd_enabled {
            let xsd_validator = XsdValidator::new(&self.specs_dir);

            let xsd_report = if is_cdar {
                xsd_validator.validate(&xml, &XsdDocumentType::Cdar)
            } else {
                match &format {
                    Some(InvoiceFormat::UBL) => {
                        // Détecter Invoice vs CreditNote
                        if xml.contains("CreditNote") {
                            xsd_validator.validate(&xml, &XsdDocumentType::UblCreditNote)
                        } else {
                            xsd_validator.validate(&xml, &XsdDocumentType::UblInvoice)
                        }
                    }
                    Some(InvoiceFormat::CII) | Some(InvoiceFormat::FacturX) => {
                        xsd_validator.validate(&xml, &XsdDocumentType::CiiInvoice)
                    }
                    None => {
                        xsd_validator.validate_auto(&xml)
                    }
                }
            };

            exchange.set_property("validation.xsd.valid", &xsd_report.is_valid().to_string());
            exchange.set_property("validation.xsd.errors", &xsd_report.error_count().to_string());
            total_errors += xsd_report.error_count() + xsd_report.fatal_count();
            total_warnings += xsd_report.warning_count();
            all_issues.extend(xsd_report.issues);
        }

        // === Validation Schematron EN16931 ===
        if self.en16931_enabled && !is_cdar {
            let sch_validator = SchematronValidator::new(&self.specs_dir);

            let en_report = match &format {
                Some(InvoiceFormat::UBL) => {
                    sch_validator.validate(&xml, &SchematronType::En16931Ubl)
                }
                Some(InvoiceFormat::CII) | Some(InvoiceFormat::FacturX) => {
                    sch_validator.validate(&xml, &SchematronType::En16931Cii)
                }
                None => {
                    let mut r = crate::error::ValidationReport::new("EN16931");
                    r.add_issue(crate::error::ValidationIssue {
                        level: ValidationLevel::Warning,
                        rule_id: "SCH-SKIP".to_string(),
                        message: "Format non détecté, validation EN16931 ignorée".to_string(),
                        location: None,
                        source: "EN16931".to_string(),
                    });
                    r
                }
            };

            exchange.set_property("validation.en16931.valid", &en_report.is_valid().to_string());
            exchange.set_property("validation.en16931.errors", &en_report.error_count().to_string());
            total_errors += en_report.error_count() + en_report.fatal_count();
            total_warnings += en_report.warning_count();
            all_issues.extend(en_report.issues);
        }

        // === Validation Schematron BR-FR ===
        if self.br_fr_enabled {
            let sch_validator = SchematronValidator::new(&self.specs_dir);

            let fr_report = if is_cdar {
                sch_validator.validate_cdar(&xml)
            } else {
                match &format {
                    Some(InvoiceFormat::UBL) => {
                        sch_validator.validate(&xml, &SchematronType::BrFrUbl)
                    }
                    Some(InvoiceFormat::CII) | Some(InvoiceFormat::FacturX) => {
                        sch_validator.validate(&xml, &SchematronType::BrFrCii)
                    }
                    None => {
                        let mut r = crate::error::ValidationReport::new("BR-FR");
                        r.add_issue(crate::error::ValidationIssue {
                            level: ValidationLevel::Warning,
                            rule_id: "SCH-SKIP".to_string(),
                            message: "Format non détecté, validation BR-FR ignorée".to_string(),
                            location: None,
                            source: "BR-FR".to_string(),
                        });
                        r
                    }
                }
            };

            exchange.set_property("validation.br_fr.valid", &fr_report.is_valid().to_string());
            exchange.set_property("validation.br_fr.errors", &fr_report.error_count().to_string());
            total_errors += fr_report.error_count() + fr_report.fatal_count();
            total_warnings += fr_report.warning_count();
            all_issues.extend(fr_report.issues);
        }

        // === Résultat global ===
        let is_valid = total_errors == 0;
        exchange.set_property("validation.xml.valid", &is_valid.to_string());
        exchange.set_property("validation.xml.total_errors", &total_errors.to_string());
        exchange.set_property("validation.xml.total_warnings", &total_warnings.to_string());

        // Stocker le détail en JSON
        if let Ok(json) = serde_json::to_string(&all_issues) {
            exchange.set_property("validation.xml.issues", &json);
        }

        if is_valid {
            tracing::info!(
                exchange_id = %exchange.id,
                total_warnings = total_warnings,
                "Validation XML complète réussie (XSD + Schematron)"
            );
            exchange.set_status(FlowStatus::Validated);
            Ok(exchange)
        } else if self.strict {
            // Collecter les messages d'erreur
            let error_msgs: Vec<String> = all_issues.iter()
                .filter(|i| matches!(i.level, ValidationLevel::Fatal | ValidationLevel::Error))
                .take(10) // Limiter pour ne pas surcharger le message
                .map(|i| format!("[{}][{}] {}", i.source, i.rule_id, i.message))
                .collect();
            let combined = error_msgs.join("; ");

            tracing::error!(
                exchange_id = %exchange.id,
                total_errors = total_errors,
                "Validation XML échouée (mode strict)"
            );

            Err(PdpError::ValidationError(format!(
                "Document invalide ({} erreurs): {}{}",
                total_errors,
                combined,
                if total_errors > 10 { "..." } else { "" }
            )))
        } else {
            tracing::warn!(
                exchange_id = %exchange.id,
                total_errors = total_errors,
                total_warnings = total_warnings,
                "Validation XML échouée (mode lenient, on continue)"
            );
            exchange.set_status(FlowStatus::Validated);
            Ok(exchange)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pdp_core::exchange::Exchange;

    fn specs_dir() -> String {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../specs")
            .to_string_lossy()
            .to_string()
    }

    #[tokio::test]
    async fn test_processor_xsd_only_ubl() {
        let specs = specs_dir();
        let processor = XmlValidateProcessor::with_options(&specs, true, false, false, false);

        let example_path = PathBuf::from(&specs).join("examples/en16931/ubl/ubl-tc434-example1.xml");
        if !example_path.exists() {
            eprintln!("Exemple UBL non disponible, test ignoré");
            return;
        }

        let xml = std::fs::read(example_path).unwrap();
        let mut exchange = Exchange::new(xml);
        exchange.set_header("invoice.format", "UBL");

        let result = processor.process(exchange).await;
        assert!(result.is_ok(), "XSD validation should pass: {:?}", result.err());
        let ex = result.unwrap();
        println!("XSD valid: {:?}", ex.get_property("validation.xsd.valid"));
    }

    #[tokio::test]
    async fn test_processor_full_validation_cdar() {
        let specs = specs_dir();
        let processor = XmlValidateProcessor::with_options(&specs, true, false, true, false);

        let example_path = PathBuf::from(&specs).join("examples/xp-z12/cdar/_Cas Nominal/UC1_F202500003_01-CDV-200_Deposee.xml");
        if !example_path.exists() {
            eprintln!("Exemple CDAR non disponible, test ignoré");
            return;
        }

        let xml = std::fs::read(example_path).unwrap();
        let exchange = Exchange::new(xml);

        let result = processor.process(exchange).await;
        assert!(result.is_ok());
        let ex = result.unwrap();
        println!("CDAR validation: xsd={:?}, br_fr={:?}",
            ex.get_property("validation.xsd.valid"),
            ex.get_property("validation.br_fr.valid"));
    }

    #[tokio::test]
    async fn test_processor_invalid_xml_strict() {
        let specs = specs_dir();
        let processor = XmlValidateProcessor::new(&specs);

        let exchange = Exchange::new(b"<invalid>not a real invoice</invalid>".to_vec());
        let result = processor.process(exchange).await;
        // Should fail in strict mode
        assert!(result.is_err());
    }
}
