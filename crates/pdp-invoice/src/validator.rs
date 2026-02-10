use std::path::{Path, PathBuf};
use pdp_core::model::{InvoiceData, InvoiceFormat};
use pdp_validate::{SchematronValidator, ValidationLevel, ValidationReport};
use serde::{Deserialize, Serialize};

/// Résultat de validation d'une facture
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<ValidationIssue>,
    pub warnings: Vec<ValidationIssue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationIssue {
    pub rule_id: String,
    pub severity: Severity,
    pub field: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Severity {
    Fatal,
    Error,
    Warning,
    Info,
}

/// Validateur de factures.
/// Les règles métier (EN16931, BR-FR) sont déléguées au Schematron
/// via les XSLT compilés. Le validateur applique les Schematrons
/// EN16931 + BR-FR sur le XML brut de la facture.
pub struct InvoiceValidator {
    specs_dir: PathBuf,
}

impl InvoiceValidator {
    /// Crée un validateur avec le répertoire specs par défaut (../../specs relatif au crate)
    pub fn new() -> Self {
        let default_specs = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../specs");
        Self {
            specs_dir: default_specs,
        }
    }

    /// Crée un validateur avec un répertoire specs personnalisé
    pub fn with_specs_dir(specs_dir: &Path) -> Self {
        Self {
            specs_dir: specs_dir.to_path_buf(),
        }
    }

    /// Valide une facture via Schematron (EN16931 + BR-FR) sur le XML brut.
    /// Si le XML brut n'est pas disponible, effectue une validation structurelle minimale.
    pub fn validate(&self, invoice: &InvoiceData) -> ValidationResult {
        // Si le XML brut est disponible, valider via Schematron
        if let Some(ref raw_xml) = invoice.raw_xml {
            return self.validate_schematron(raw_xml, &invoice.source_format, invoice);
        }

        // Fallback : validation structurelle minimale (pas de XML brut)
        tracing::warn!(
            invoice = %invoice.invoice_number,
            "XML brut non disponible, validation structurelle minimale uniquement"
        );
        self.validate_structural(invoice)
    }

    /// Validation Schematron complète : EN16931 + BR-FR sur le XML brut
    fn validate_schematron(&self, xml: &str, format: &InvoiceFormat, invoice: &InvoiceData) -> ValidationResult {
        let sch = SchematronValidator::new(&self.specs_dir);

        let report = match format {
            InvoiceFormat::UBL => sch.validate_ubl_invoice(xml),
            InvoiceFormat::CII | InvoiceFormat::FacturX => sch.validate_cii_invoice(xml),
        };

        self.report_to_result(report, invoice)
    }

    /// Convertit un ValidationReport (pdp-validate) en ValidationResult (pdp-invoice)
    fn report_to_result(&self, report: ValidationReport, invoice: &InvoiceData) -> ValidationResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        for issue in &report.issues {
            let vi = ValidationIssue {
                rule_id: issue.rule_id.clone(),
                severity: match issue.level {
                    ValidationLevel::Fatal => Severity::Fatal,
                    ValidationLevel::Error => Severity::Error,
                    ValidationLevel::Warning => Severity::Warning,
                    ValidationLevel::Info => Severity::Info,
                },
                field: issue.location.clone().unwrap_or_default(),
                message: issue.message.clone(),
            };
            match issue.level {
                ValidationLevel::Fatal | ValidationLevel::Error => errors.push(vi),
                _ => warnings.push(vi),
            }
        }

        // Vérifications BR-FR complémentaires (non couvertes par tous les Schematrons)
        // BR-FR-01 : ID facture ≤ 35 caractères
        if invoice.invoice_number.len() > 35 {
            errors.push(ValidationIssue {
                rule_id: "BR-FR-01".to_string(),
                severity: Severity::Fatal,
                field: "invoice_number".to_string(),
                message: format!(
                    "BR-FR-01 : Le numéro de facture (BT-1) ne doit pas dépasser 35 caractères (actuel : {})",
                    invoice.invoice_number.len()
                ),
            });
        }

        // BR-FR-04 : types de facture autorisés
        const VALID_TYPE_CODES: &[&str] = &[
            "380", "389", "393", "501", "386", "500", "384", "471", "472", "473",
            "261", "262", "381", "396", "502", "503",
        ];
        if let Some(ref tc) = invoice.invoice_type_code {
            if !VALID_TYPE_CODES.contains(&tc.as_str()) {
                errors.push(ValidationIssue {
                    rule_id: "BR-FR-04".to_string(),
                    severity: Severity::Fatal,
                    field: "invoice_type_code".to_string(),
                    message: format!(
                        "BR-FR-04 : Le type de facture '{}' n'est pas autorisé. Types valides : {:?}",
                        tc, VALID_TYPE_CODES
                    ),
                });
            }
        }

        let has_fatal = errors.iter().any(|e| e.severity == Severity::Fatal || e.severity == Severity::Error);
        let is_valid = report.is_valid() && !has_fatal;

        tracing::info!(
            invoice = %invoice.invoice_number,
            valid = is_valid,
            errors = errors.len(),
            warnings = warnings.len(),
            "Validation Schematron terminée"
        );

        ValidationResult {
            is_valid,
            errors,
            warnings,
        }
    }

    /// Validation structurelle minimale (quand le XML brut n'est pas disponible).
    /// Vérifie uniquement que les champs obligatoires du modèle parsé sont présents.
    fn validate_structural(&self, invoice: &InvoiceData) -> ValidationResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        if invoice.invoice_number.is_empty() || invoice.invoice_number == "INCONNU" {
            errors.push(ValidationIssue {
                rule_id: "BR-01".to_string(),
                severity: Severity::Fatal,
                field: "invoice_number".to_string(),
                message: "Le numéro de facture (BT-1) est obligatoire".to_string(),
            });
        }

        // BR-FR-01 : ID facture ≤ 35 caractères
        if invoice.invoice_number.len() > 35 {
            errors.push(ValidationIssue {
                rule_id: "BR-FR-01".to_string(),
                severity: Severity::Fatal,
                field: "invoice_number".to_string(),
                message: format!(
                    "BR-FR-01 : Le numéro de facture (BT-1) ne doit pas dépasser 35 caractères (actuel : {})",
                    invoice.invoice_number.len()
                ),
            });
        }

        // BR-FR-04 : types de facture autorisés
        const VALID_TYPE_CODES: &[&str] = &[
            "380", "389", "393", "501", "386", "500", "384", "471", "472", "473",
            "261", "262", "381", "396", "502", "503",
        ];
        if let Some(ref tc) = invoice.invoice_type_code {
            if !VALID_TYPE_CODES.contains(&tc.as_str()) {
                errors.push(ValidationIssue {
                    rule_id: "BR-FR-04".to_string(),
                    severity: Severity::Fatal,
                    field: "invoice_type_code".to_string(),
                    message: format!(
                        "BR-FR-04 : Le type de facture '{}' n'est pas autorisé. Types valides : {:?}",
                        tc, VALID_TYPE_CODES
                    ),
                });
            }
        }

        if invoice.issue_date.is_none() {
            errors.push(ValidationIssue {
                rule_id: "BR-02".to_string(),
                severity: Severity::Fatal,
                field: "issue_date".to_string(),
                message: "La date d'émission (BT-2) est obligatoire".to_string(),
            });
        }

        if invoice.seller_name.is_none() {
            errors.push(ValidationIssue {
                rule_id: "BR-04".to_string(),
                severity: Severity::Error,
                field: "seller_name".to_string(),
                message: "Le nom du vendeur (BT-27) est obligatoire".to_string(),
            });
        }

        if invoice.buyer_name.is_none() {
            errors.push(ValidationIssue {
                rule_id: "BR-07".to_string(),
                severity: Severity::Error,
                field: "buyer_name".to_string(),
                message: "Le nom de l'acheteur (BT-44) est obligatoire".to_string(),
            });
        }

        if invoice.currency.is_none() {
            errors.push(ValidationIssue {
                rule_id: "BR-05".to_string(),
                severity: Severity::Error,
                field: "currency".to_string(),
                message: "La devise du document (BT-5) est obligatoire".to_string(),
            });
        }

        if invoice.seller_siret.is_none() {
            warnings.push(ValidationIssue {
                rule_id: "FR-SIRET-01".to_string(),
                severity: Severity::Warning,
                field: "seller_siret".to_string(),
                message: "Le SIRET du vendeur est recommandé pour la facturation électronique française".to_string(),
            });
        }

        let is_valid = !errors.iter().any(|e| e.severity == Severity::Fatal || e.severity == Severity::Error);

        tracing::info!(
            invoice = %invoice.invoice_number,
            valid = is_valid,
            errors = errors.len(),
            warnings = warnings.len(),
            "Validation structurelle terminée (pas de Schematron)"
        );

        ValidationResult {
            is_valid,
            errors,
            warnings,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_valid_invoice() -> InvoiceData {
        let mut inv = InvoiceData::new("FA-TEST-001".to_string(), InvoiceFormat::UBL);
        inv.issue_date = Some("2025-11-15".to_string());
        inv.seller_name = Some("Vendeur Test".to_string());
        inv.seller_siret = Some("12345678901234".to_string());
        inv.buyer_name = Some("Acheteur Test".to_string());
        inv.buyer_siret = Some("98765432109876".to_string());
        inv.currency = Some("EUR".to_string());
        inv.total_ht = Some(1000.00);
        inv.total_tax = Some(200.00);
        inv.total_ttc = Some(1200.00);
        // Pas de raw_xml → validation structurelle uniquement
        inv
    }

    // ===== Tests validation structurelle (sans raw_xml) =====

    #[test]
    fn test_structural_valid_invoice() {
        let validator = InvoiceValidator::new();
        let result = validator.validate(&make_valid_invoice());
        assert!(result.is_valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_structural_missing_invoice_number() {
        let validator = InvoiceValidator::new();
        let mut inv = make_valid_invoice();
        inv.invoice_number = String::new();
        let result = validator.validate(&inv);
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.rule_id == "BR-01"));
    }

    #[test]
    fn test_structural_missing_issue_date() {
        let validator = InvoiceValidator::new();
        let mut inv = make_valid_invoice();
        inv.issue_date = None;
        let result = validator.validate(&inv);
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.rule_id == "BR-02"));
    }

    #[test]
    fn test_structural_missing_siret_warning() {
        let validator = InvoiceValidator::new();
        let mut inv = make_valid_invoice();
        inv.seller_siret = None;
        let result = validator.validate(&inv);
        assert!(result.is_valid);
        assert!(result.warnings.iter().any(|e| e.rule_id == "FR-SIRET-01"));
    }

    // ===== Tests Schematron sur fixtures réelles =====

    #[test]
    fn test_schematron_ubl_fixture() {
        let xml = std::fs::read_to_string("../../tests/fixtures/ubl/facture_ubl_001.xml")
            .expect("Fixture UBL introuvable");
        let invoice = crate::ubl::UblParser::new().parse(&xml).unwrap();

        let validator = InvoiceValidator::new();
        let result = validator.validate(&invoice);
        assert!(result.is_valid, "La facture UBL fixture devrait être valide via Schematron: {:?}", result.errors);
    }

    #[test]
    fn test_schematron_cii_fixture() {
        let xml = std::fs::read_to_string("../../tests/fixtures/cii/facture_cii_001.xml")
            .expect("Fixture CII introuvable");
        let invoice = crate::cii::CiiParser::new().parse(&xml).unwrap();

        let validator = InvoiceValidator::new();
        let result = validator.validate(&invoice);
        assert!(result.is_valid, "La facture CII fixture devrait être valide via Schematron: {:?}", result.errors);
    }

    #[test]
    fn test_schematron_rectificative_cii_fixture() {
        let xml = std::fs::read_to_string("../../tests/fixtures/cii/facture_rectificative_cii_384.xml")
            .expect("Fixture rectificative CII introuvable");
        let invoice = crate::cii::CiiParser::new().parse(&xml).unwrap();

        let validator = InvoiceValidator::new();
        let result = validator.validate(&invoice);
        assert!(result.is_valid, "La facture rectificative CII devrait être valide: {:?}", result.errors);
    }

    #[test]
    fn test_schematron_rectificative_ubl_fixture() {
        let xml = std::fs::read_to_string("../../tests/fixtures/ubl/facture_rectificative_ubl_384.xml")
            .expect("Fixture rectificative UBL introuvable");
        let invoice = crate::ubl::UblParser::new().parse(&xml).unwrap();

        let validator = InvoiceValidator::new();
        let result = validator.validate(&invoice);
        assert!(result.is_valid, "La facture rectificative UBL devrait être valide: {:?}", result.errors);
    }

    // ===== Tests Schematron sur fixtures métier =====

    #[test]
    fn test_schematron_soustraitance_cii() {
        let xml = std::fs::read_to_string("../../tests/fixtures/cii/facture_cii_soustraitance_a4.xml")
            .expect("Fixture sous-traitance CII introuvable");
        let invoice = crate::cii::CiiParser::new().parse(&xml).unwrap();

        let validator = InvoiceValidator::new();
        let result = validator.validate(&invoice);
        assert!(result.is_valid, "Sous-traitance CII devrait être valide: {:?}", result.errors);
    }

    #[test]
    fn test_schematron_soustraitance_ubl() {
        let xml = std::fs::read_to_string("../../tests/fixtures/ubl/facture_ubl_soustraitance_a4.xml")
            .expect("Fixture sous-traitance UBL introuvable");
        let invoice = crate::ubl::UblParser::new().parse(&xml).unwrap();

        let validator = InvoiceValidator::new();
        let result = validator.validate(&invoice);
        assert!(result.is_valid, "Sous-traitance UBL devrait être valide: {:?}", result.errors);
    }

    #[test]
    fn test_schematron_marketplace_cii() {
        let xml = std::fs::read_to_string("../../tests/fixtures/cii/facture_cii_marketplace_a8.xml")
            .expect("Fixture marketplace CII introuvable");
        let invoice = crate::cii::CiiParser::new().parse(&xml).unwrap();

        let validator = InvoiceValidator::new();
        let result = validator.validate(&invoice);
        assert!(result.is_valid, "Marketplace CII devrait être valide: {:?}", result.errors);
    }

    #[test]
    fn test_schematron_marketplace_ubl() {
        let xml = std::fs::read_to_string("../../tests/fixtures/ubl/facture_ubl_marketplace_a8.xml")
            .expect("Fixture marketplace UBL introuvable");
        let invoice = crate::ubl::UblParser::new().parse(&xml).unwrap();

        let validator = InvoiceValidator::new();
        let result = validator.validate(&invoice);
        assert!(result.is_valid, "Marketplace UBL devrait être valide: {:?}", result.errors);
    }

    #[test]
    fn test_schematron_acompte_cii() {
        let xml = std::fs::read_to_string("../../tests/fixtures/cii/facture_cii_acompte.xml")
            .expect("Fixture acompte CII introuvable");
        let invoice = crate::cii::CiiParser::new().parse(&xml).unwrap();

        let validator = InvoiceValidator::new();
        let result = validator.validate(&invoice);
        assert!(result.is_valid, "Acompte CII devrait être valide: {:?}", result.errors);
        assert_eq!(invoice.invoice_type_code.as_deref(), Some("386"));
    }

    #[test]
    fn test_schematron_acompte_ubl() {
        let xml = std::fs::read_to_string("../../tests/fixtures/ubl/facture_ubl_acompte_386.xml")
            .expect("Fixture acompte UBL introuvable");
        let invoice = crate::ubl::UblParser::new().parse(&xml).unwrap();

        let validator = InvoiceValidator::new();
        let result = validator.validate(&invoice);
        assert!(result.is_valid, "Acompte UBL devrait être valide: {:?}", result.errors);
        assert_eq!(invoice.invoice_type_code.as_deref(), Some("386"));
    }

    #[test]
    fn test_schematron_definitive_apres_acompte_cii() {
        let xml = std::fs::read_to_string("../../tests/fixtures/cii/facture_cii_definitive_apres_acompte.xml")
            .expect("Fixture définitive CII introuvable");
        let invoice = crate::cii::CiiParser::new().parse(&xml).unwrap();

        let validator = InvoiceValidator::new();
        let result = validator.validate(&invoice);
        assert!(result.is_valid, "Définitive après acompte CII devrait être valide: {:?}", result.errors);
    }

    #[test]
    fn test_schematron_definitive_apres_acompte_ubl() {
        let xml = std::fs::read_to_string("../../tests/fixtures/ubl/facture_ubl_definitive_apres_acompte.xml")
            .expect("Fixture définitive UBL introuvable");
        let invoice = crate::ubl::UblParser::new().parse(&xml).unwrap();

        let validator = InvoiceValidator::new();
        let result = validator.validate(&invoice);
        assert!(result.is_valid, "Définitive après acompte UBL devrait être valide: {:?}", result.errors);
    }

    #[test]
    fn test_schematron_multitva_cii() {
        let xml = std::fs::read_to_string("../../tests/fixtures/cii/facture_cii_remises_multitva.xml")
            .expect("Fixture multi-TVA CII introuvable");
        let invoice = crate::cii::CiiParser::new().parse(&xml).unwrap();

        let validator = InvoiceValidator::new();
        let result = validator.validate(&invoice);
        assert!(result.is_valid, "Multi-TVA CII devrait être valide: {:?}", result.errors);
    }

    #[test]
    fn test_schematron_multitva_ubl() {
        let xml = std::fs::read_to_string("../../tests/fixtures/ubl/facture_ubl_remises_multitva.xml")
            .expect("Fixture multi-TVA UBL introuvable");
        let invoice = crate::ubl::UblParser::new().parse(&xml).unwrap();

        let validator = InvoiceValidator::new();
        let result = validator.validate(&invoice);
        assert!(result.is_valid, "Multi-TVA UBL devrait être valide: {:?}", result.errors);
    }

    #[test]
    fn test_schematron_autofacture_cii() {
        let xml = std::fs::read_to_string("../../tests/fixtures/cii/autofacture_cii_389.xml")
            .expect("Fixture autofacture CII introuvable");
        let invoice = crate::cii::CiiParser::new().parse(&xml).unwrap();

        let validator = InvoiceValidator::new();
        let result = validator.validate(&invoice);
        assert!(result.is_valid, "Autofacture CII devrait être valide: {:?}", result.errors);
        assert_eq!(invoice.invoice_type_code.as_deref(), Some("389"));
    }

    #[test]
    fn test_schematron_autofacture_ubl() {
        let xml = std::fs::read_to_string("../../tests/fixtures/ubl/autofacture_ubl_389.xml")
            .expect("Fixture autofacture UBL introuvable");
        let invoice = crate::ubl::UblParser::new().parse(&xml).unwrap();

        let validator = InvoiceValidator::new();
        let result = validator.validate(&invoice);
        assert!(result.is_valid, "Autofacture UBL devrait être valide: {:?}", result.errors);
        assert_eq!(invoice.invoice_type_code.as_deref(), Some("389"));
    }

    // ===== Tests Schematron sur Factur-X =====

    #[test]
    fn test_schematron_facturx_standard() {
        let pdf = std::fs::read("../../tests/fixtures/facturx/facture_facturx_001.pdf")
            .expect("Fixture Factur-X introuvable");
        let invoice = crate::facturx::FacturXParser::new().parse(&pdf).unwrap();

        let validator = InvoiceValidator::new();
        let result = validator.validate(&invoice);
        assert!(result.is_valid, "La facture Factur-X devrait être valide via Schematron: {:?}", result.errors);
    }

    #[test]
    fn test_schematron_facturx_avoir() {
        let pdf = std::fs::read("../../tests/fixtures/facturx/avoir_facturx_381.pdf")
            .expect("Fixture Factur-X avoir introuvable");
        let invoice = crate::facturx::FacturXParser::new().parse(&pdf).unwrap();

        let validator = InvoiceValidator::new();
        let result = validator.validate(&invoice);
        assert!(result.is_valid, "L'avoir Factur-X devrait être valide via Schematron: {:?}", result.errors);
    }

    #[test]
    fn test_schematron_facturx_rectificative() {
        let pdf = std::fs::read("../../tests/fixtures/facturx/facture_rectificative_facturx_384.pdf")
            .expect("Fixture Factur-X rectificative introuvable");
        let invoice = crate::facturx::FacturXParser::new().parse(&pdf).unwrap();

        let validator = InvoiceValidator::new();
        let result = validator.validate(&invoice);
        assert!(result.is_valid, "La facture rectificative Factur-X devrait être valide via Schematron: {:?}", result.errors);
    }

    // ===== Tests Schematron sur fixtures d'erreurs =====

    #[test]
    fn test_schematron_invalid_cii_fixture() {
        let xml = std::fs::read_to_string("../../tests/fixtures/errors/facture_invalide_001.xml")
            .expect("Fixture invalide CII introuvable");
        let invoice = crate::cii::CiiParser::new().parse(&xml).unwrap();

        let validator = InvoiceValidator::new();
        let result = validator.validate(&invoice);
        assert!(!result.is_valid, "La facture invalide CII ne devrait pas être valide");
        // Le Schematron doit détecter les erreurs (BR-01, BR-02, etc.)
        assert!(!result.errors.is_empty(), "Le Schematron doit détecter des erreurs");
    }

    #[test]
    fn test_schematron_invalid_ubl_fixture() {
        let xml = std::fs::read_to_string("../../tests/fixtures/errors/facture_invalide_ubl_001.xml")
            .expect("Fixture invalide UBL introuvable");
        let invoice = crate::ubl::UblParser::new().parse(&xml).unwrap();

        let validator = InvoiceValidator::new();
        let result = validator.validate(&invoice);
        assert!(!result.is_valid, "La facture invalide UBL ne devrait pas être valide");
        assert!(!result.errors.is_empty(), "Le Schematron doit détecter des erreurs");
    }

    // ===== Tests Schematron délégation et multi-vendeurs =====

    #[test]
    fn test_schematron_delegation_s8_cii() {
        let xml = std::fs::read_to_string("../../tests/fixtures/cii/facture_cii_delegation_s8.xml")
            .expect("Fixture délégation CII introuvable");
        let invoice = crate::cii::CiiParser::new().parse(&xml).unwrap();

        let validator = InvoiceValidator::new();
        let result = validator.validate(&invoice);
        assert!(result.is_valid, "Délégation S8 CII devrait être valide: {:?}", result.errors);
        assert_eq!(invoice.business_process.as_deref(), Some("S8"));
        assert!(invoice.invoicer_name.is_some(), "Le facturant (II) doit être renseigné");
    }

    #[test]
    fn test_schematron_delegation_s8_ubl() {
        let xml = std::fs::read_to_string("../../tests/fixtures/ubl/facture_ubl_delegation_s8.xml")
            .expect("Fixture délégation UBL introuvable");
        let invoice = crate::ubl::UblParser::new().parse(&xml).unwrap();

        let validator = InvoiceValidator::new();
        let result = validator.validate(&invoice);
        assert!(result.is_valid, "Délégation S8 UBL devrait être valide: {:?}", result.errors);
        assert_eq!(invoice.business_process.as_deref(), Some("S8"));
        assert!(invoice.invoicer_name.is_some(), "Le facturant (II) doit être renseigné");
    }

    #[test]
    fn test_schematron_multivendeurs_b8_cii() {
        let xml = std::fs::read_to_string("../../tests/fixtures/cii/facture_cii_multivendeurs_b8.xml")
            .expect("Fixture multi-vendeurs CII introuvable");
        let invoice = crate::cii::CiiParser::new().parse(&xml).unwrap();

        let validator = InvoiceValidator::new();
        let result = validator.validate(&invoice);
        assert!(result.is_valid, "Multi-vendeurs B8 CII devrait être valide: {:?}", result.errors);
        assert_eq!(invoice.business_process.as_deref(), Some("B8"));
        assert!(invoice.invoicer_name.is_some(), "Le facturant (II) doit être renseigné");
    }

    #[test]
    fn test_schematron_multivendeurs_b8_ubl() {
        let xml = std::fs::read_to_string("../../tests/fixtures/ubl/facture_ubl_multivendeurs_b8.xml")
            .expect("Fixture multi-vendeurs UBL introuvable");
        let invoice = crate::ubl::UblParser::new().parse(&xml).unwrap();

        let validator = InvoiceValidator::new();
        let result = validator.validate(&invoice);
        assert!(result.is_valid, "Multi-vendeurs B8 UBL devrait être valide: {:?}", result.errors);
        assert_eq!(invoice.business_process.as_deref(), Some("B8"));
        assert!(invoice.invoicer_name.is_some(), "Le facturant (II) doit être renseigné");
    }

    // ===== Tests de rejet : fixtures invalides =====

    #[test]
    fn test_reject_cii_sans_endpoint() {
        let xml = std::fs::read_to_string("../../tests/fixtures/errors/cii_sans_endpoint.xml")
            .expect("Fixture CII sans endpoint introuvable");
        let invoice = crate::cii::CiiParser::new().parse(&xml).unwrap();
        assert!(invoice.seller_endpoint_id.is_none(), "seller_endpoint_id doit être absent");
        assert!(invoice.buyer_endpoint_id.is_none(), "buyer_endpoint_id doit être absent");
    }

    #[test]
    fn test_reject_ubl_sans_endpoint() {
        let xml = std::fs::read_to_string("../../tests/fixtures/errors/ubl_sans_endpoint.xml")
            .expect("Fixture UBL sans endpoint introuvable");
        let invoice = crate::ubl::UblParser::new().parse(&xml).unwrap();
        assert!(invoice.seller_endpoint_id.is_none(), "seller_endpoint_id doit être absent");
        assert!(invoice.buyer_endpoint_id.is_none(), "buyer_endpoint_id doit être absent");
    }

    #[test]
    fn test_reject_cii_id_trop_long() {
        let xml = std::fs::read_to_string("../../tests/fixtures/errors/cii_id_trop_long.xml")
            .expect("Fixture CII ID trop long introuvable");
        let invoice = crate::cii::CiiParser::new().parse(&xml).unwrap();
        assert!(invoice.invoice_number.len() > 35, "L'ID doit dépasser 35 caractères");

        let validator = InvoiceValidator::new();
        let result = validator.validate(&invoice);
        assert!(!result.is_valid, "BR-FR-01 : ID > 35 chars doit être rejeté");
        let has_br_fr_01 = result.errors.iter().any(|e| {
            e.rule_id.contains("BR-FR-01") || e.message.contains("35")
        });
        assert!(has_br_fr_01, "Doit contenir une erreur BR-FR-01 : {:?}", result.errors);
    }

    #[test]
    fn test_reject_ubl_id_trop_long() {
        let xml = std::fs::read_to_string("../../tests/fixtures/errors/ubl_id_trop_long.xml")
            .expect("Fixture UBL ID trop long introuvable");
        let invoice = crate::ubl::UblParser::new().parse(&xml).unwrap();
        assert!(invoice.invoice_number.len() > 35, "L'ID doit dépasser 35 caractères");

        let validator = InvoiceValidator::new();
        let result = validator.validate(&invoice);
        assert!(!result.is_valid, "BR-FR-01 : ID > 35 chars doit être rejeté");
        let has_br_fr_01 = result.errors.iter().any(|e| {
            e.rule_id.contains("BR-FR-01") || e.message.contains("35")
        });
        assert!(has_br_fr_01, "Doit contenir une erreur BR-FR-01 : {:?}", result.errors);
    }

    #[test]
    fn test_reject_cii_sans_acheteur() {
        let xml = std::fs::read_to_string("../../tests/fixtures/errors/cii_sans_acheteur.xml")
            .expect("Fixture CII sans acheteur introuvable");
        let invoice = crate::cii::CiiParser::new().parse(&xml).unwrap();
        assert!(invoice.buyer_name.is_none(), "buyer_name doit être absent");

        let validator = InvoiceValidator::new();
        let result = validator.validate(&invoice);
        assert!(!result.is_valid, "Facture sans acheteur doit être rejetée");
    }

    #[test]
    fn test_reject_ubl_sans_acheteur() {
        let xml = std::fs::read_to_string("../../tests/fixtures/errors/ubl_sans_acheteur.xml")
            .expect("Fixture UBL sans acheteur introuvable");
        let invoice = crate::ubl::UblParser::new().parse(&xml).unwrap();
        assert!(invoice.buyer_name.is_none(), "buyer_name doit être absent");

        let validator = InvoiceValidator::new();
        let result = validator.validate(&invoice);
        assert!(!result.is_valid, "Facture sans acheteur doit être rejetée");
    }

    #[test]
    fn test_reject_cii_type_invalide() {
        let xml = std::fs::read_to_string("../../tests/fixtures/errors/cii_type_invalide.xml")
            .expect("Fixture CII type invalide introuvable");
        let invoice = crate::cii::CiiParser::new().parse(&xml).unwrap();
        assert_eq!(invoice.invoice_type_code.as_deref(), Some("999"));

        let validator = InvoiceValidator::new();
        let result = validator.validate(&invoice);
        assert!(!result.is_valid, "TypeCode 999 doit être rejeté (BR-FR-04)");
        let has_type_error = result.errors.iter().any(|e| {
            e.rule_id.contains("BR-FR-04") || e.message.contains("type") || e.message.contains("999")
        });
        assert!(has_type_error, "Doit contenir une erreur de type : {:?}", result.errors);
    }
}
