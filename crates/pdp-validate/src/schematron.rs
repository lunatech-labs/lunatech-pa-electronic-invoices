use std::path::{Path, PathBuf};
use std::process::Command;
use libxml::parser::Parser as XmlParser;
use libxml::xpath::Context as XpathContext;
use tracing;

use crate::error::{ValidationIssue, ValidationLevel, ValidationReport};

/// Règles Schematron dont les warnings sont promus en erreurs fatales.
/// Ces règles sont marquées `flag="warning"` dans les .sch officiels mais
/// sont bloquantes pour l'e-invoicing français.
const RULES_PROMOTED_TO_FATAL: &[&str] = &[
    "BR-FR-12_BT-49", // BT-49 : adresse électronique acheteur obligatoire
    "BR-FR-13_BT-34", // BT-34 : adresse électronique vendeur obligatoire
];

/// Type de validation Schematron à appliquer
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SchematronType {
    /// EN16931 pour UBL
    En16931Ubl,
    /// EN16931 pour CII
    En16931Cii,
    /// Factur-X 1.08 EN16931 (CII embarqué, inclut PEPPOL)
    FacturXEN16931,
    /// Factur-X 1.08 EXTENDED (CII embarqué, inclut PEPPOL)
    FacturXExtended,
    /// Règles françaises BR-FR pour UBL (Flux 2)
    BrFrUbl,
    /// Règles françaises BR-FR pour CII (Flux 2)
    BrFrCii,
    /// Règles françaises BR-FR pour CDAR (CDV)
    BrFrCdar,
}

impl std::fmt::Display for SchematronType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SchematronType::En16931Ubl => write!(f, "EN16931-UBL"),
            SchematronType::En16931Cii => write!(f, "EN16931-CII"),
            SchematronType::FacturXEN16931 => write!(f, "Factur-X-EN16931"),
            SchematronType::FacturXExtended => write!(f, "Factur-X-EXTENDED"),
            SchematronType::BrFrUbl => write!(f, "BR-FR-UBL"),
            SchematronType::BrFrCii => write!(f, "BR-FR-CII"),
            SchematronType::BrFrCdar => write!(f, "BR-FR-CDAR"),
        }
    }
}

/// Validateur Schematron utilisant xsltproc pour appliquer les XSLT compilés
/// et produire un SVRL (Schematron Validation Report Language).
pub struct SchematronValidator {
    /// Répertoire racine des specs
    specs_dir: PathBuf,
}

impl SchematronValidator {
    pub fn new(specs_dir: &Path) -> Self {
        Self {
            specs_dir: specs_dir.to_path_buf(),
        }
    }

    /// Retourne le chemin du XSLT compilé pour un type de schematron
    fn xslt_path(&self, schematron_type: &SchematronType) -> PathBuf {
        match schematron_type {
            SchematronType::En16931Ubl => {
                self.specs_dir.join("xslt/en16931/EN16931-UBL-validation.xslt")
            }
            SchematronType::En16931Cii => {
                self.specs_dir.join("xslt/en16931/EN16931-CII-validation.xslt")
            }
            SchematronType::FacturXEN16931 => {
                self.specs_dir.join("xslt/facturx-1.08/EN16931/FACTUR-X_EN16931.xslt")
            }
            SchematronType::FacturXExtended => {
                self.specs_dir.join("xslt/facturx-1.08/EXTENDED/FACTUR-X_EXTENDED.xslt")
            }
            SchematronType::BrFrUbl => {
                self.specs_dir.join("xslt/fr/20251114_BR-FR-Flux2-Schematron-UBL_V1.2.0-compiled.xsl")
            }
            SchematronType::BrFrCii => {
                self.specs_dir.join("xslt/fr/20251114_BR-FR-Flux2-Schematron-CII_V1.2.0-compiled.xsl")
            }
            SchematronType::BrFrCdar => {
                self.specs_dir.join("xslt/fr/20251114_BR-FR-CDV-Schematron-CDAR_V1.2.0-compiled.xsl")
            }
        }
    }

    /// Valide un document XML avec un schematron spécifique via Saxon/xsltproc.
    /// 1. Écrit le XML dans un fichier temporaire
    /// 2. Exécute Saxon (ou xsltproc) avec le XSLT compilé → produit du SVRL
    /// 3. Parse le SVRL pour extraire les failed-assert et successful-report
    pub fn validate(&self, xml: &str, schematron_type: &SchematronType) -> ValidationReport {
        let source_name = schematron_type.to_string();

        let xslt_path = self.xslt_path(schematron_type);
        if !xslt_path.exists() {
            let mut report = ValidationReport::new(&format!("Schematron {}", source_name));
            report.add_issue(ValidationIssue {
                level: ValidationLevel::Fatal,
                rule_id: "SCH-000".to_string(),
                message: format!("XSLT Schematron compilé introuvable: {}", xslt_path.display()),
                location: None,
                source: source_name.clone(),
            });
            return report;
        }

        // Écrire le XML dans un fichier temporaire
        let tmp_dir = std::env::temp_dir();
        let tmp_xml = tmp_dir.join(format!("pdp-validate-{}.xml", uuid::Uuid::new_v4()));
        if let Err(e) = std::fs::write(&tmp_xml, xml) {
            let mut report = ValidationReport::new(&format!("Schematron {}", source_name));
            report.add_issue(ValidationIssue {
                level: ValidationLevel::Fatal,
                rule_id: "SCH-001".to_string(),
                message: format!("Impossible d'écrire le fichier temporaire: {}", e),
                location: None,
                source: source_name,
            });
            return report;
        }

        let report = self.validate_with_file(&tmp_xml, schematron_type);

        // Nettoyer le fichier temporaire
        let _ = std::fs::remove_file(&tmp_xml);

        report
    }

    /// Valide un fichier XML déjà écrit sur disque avec un schematron spécifique.
    /// Évite de réécrire le fichier temporaire quand on enchaîne plusieurs validations.
    fn validate_with_file(&self, xml_path: &Path, schematron_type: &SchematronType) -> ValidationReport {
        let source_name = schematron_type.to_string();
        let mut report = ValidationReport::new(&format!("Schematron {}", source_name));

        let xslt_path = self.xslt_path(schematron_type);
        if !xslt_path.exists() {
            report.add_issue(ValidationIssue {
                level: ValidationLevel::Fatal,
                rule_id: "SCH-000".to_string(),
                message: format!("XSLT Schematron compilé introuvable: {}", xslt_path.display()),
                location: None,
                source: source_name.clone(),
            });
            return report;
        }

        tracing::debug!(
            schematron = %source_name,
            xslt = %xslt_path.display(),
            "Validation Schematron en cours"
        );

        // Exécuter SaxonC natif (transform) en priorité — ~4× plus rapide que SaxonJ
        // Fallback sur SaxonJ (saxon) puis xsltproc (XSLT 1.0)
        let xml_str = xml_path.to_str().unwrap_or("");
        let xslt_str = xslt_path.to_str().unwrap_or("");
        let result = Command::new("transform")
            .arg(format!("-s:{}", xml_str))
            .arg(format!("-xsl:{}", xslt_str))
            .output()
            .or_else(|_| {
                tracing::debug!("SaxonC (transform) non disponible, essai SaxonJ (saxon)");
                Command::new("saxon")
                    .arg(format!("-s:{}", xml_str))
                    .arg(format!("-xsl:{}", xslt_str))
                    .output()
            })
            .or_else(|_| {
                tracing::warn!("Saxon non disponible, fallback sur xsltproc (XSLT 1.0)");
                Command::new("xsltproc")
                    .arg("--nonet")
                    .arg(xslt_str)
                    .arg(xml_str)
                    .output()
            });

        let output = match result {
            Ok(output) => output,
            Err(e) => {
                report.add_issue(ValidationIssue {
                    level: ValidationLevel::Fatal,
                    rule_id: "SCH-002".to_string(),
                    message: format!("Impossible d'exécuter saxon/xsltproc: {}. Vérifiez que saxon ou xsltproc est installé.", e),
                    location: None,
                    source: source_name,
                });
                return report;
            }
        };

        if !output.status.success() && output.stdout.is_empty() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            report.add_issue(ValidationIssue {
                level: ValidationLevel::Fatal,
                rule_id: "SCH-003".to_string(),
                message: format!("saxon/xsltproc a échoué: {}", stderr.trim()),
                location: None,
                source: source_name,
            });
            return report;
        }

        let svrl_xml = String::from_utf8_lossy(&output.stdout).to_string();
        if svrl_xml.trim().is_empty() {
            report.add_issue(ValidationIssue {
                level: ValidationLevel::Fatal,
                rule_id: "SCH-004".to_string(),
                message: "Le résultat SVRL est vide".to_string(),
                location: None,
                source: source_name,
            });
            return report;
        }

        // Parser le SVRL pour extracter les erreurs
        self.parse_svrl(&svrl_xml, &source_name, &mut report);

        // Promouvoir certains warnings en fatal (BR-FR-12/13)
        Self::promote_warnings(&mut report);

        if report.is_valid() {
            tracing::info!(schematron = %source_name, "Validation Schematron réussie");
        } else {
            tracing::warn!(
                schematron = %source_name,
                errors = report.error_count(),
                warnings = report.warning_count(),
                "Validation Schematron: problèmes détectés"
            );
        }

        report
    }

    /// Parse le SVRL XML pour extraire les failed-assert et successful-report
    fn parse_svrl(&self, svrl_xml: &str, source: &str, report: &mut ValidationReport) {
        let parser = XmlParser::default();
        let doc = match parser.parse_string(svrl_xml) {
            Ok(doc) => doc,
            Err(_) => {
                // Fallback : parser le SVRL comme texte brut
                self.parse_svrl_text(svrl_xml, source, report);
                return;
            }
        };

        let mut ctx = match XpathContext::new(&doc) {
            Ok(ctx) => ctx,
            Err(_) => {
                self.parse_svrl_text(svrl_xml, source, report);
                return;
            }
        };

        // Enregistrer le namespace SVRL
        let _ = ctx.register_namespace("svrl", "http://purl.oclc.org/dsdl/svrl");

        // Extraire les failed-assert (erreurs)
        if let Ok(nodes) = ctx.findnodes("//svrl:failed-assert", None) {
            for node in &nodes {
                let flag = node.get_attribute("flag").unwrap_or_default();
                let id = node.get_attribute("id").unwrap_or_else(|| "UNKNOWN".to_string());
                let location = node.get_attribute("location");
                let text = node.get_content();

                let level = match flag.to_lowercase().as_str() {
                    "fatal" => ValidationLevel::Fatal,
                    "warning" => ValidationLevel::Warning,
                    _ => ValidationLevel::Error,
                };

                report.add_issue(ValidationIssue {
                    level,
                    rule_id: id,
                    message: text.trim().to_string(),
                    location,
                    source: source.to_string(),
                });
            }
        }

        // Extraire les successful-report (informations/warnings)
        if let Ok(nodes) = ctx.findnodes("//svrl:successful-report", None) {
            for node in &nodes {
                let flag = node.get_attribute("flag").unwrap_or_default();
                let id = node.get_attribute("id").unwrap_or_else(|| "UNKNOWN".to_string());
                let location = node.get_attribute("location");
                let text = node.get_content();

                let level = match flag.to_lowercase().as_str() {
                    "fatal" => ValidationLevel::Fatal,
                    "error" => ValidationLevel::Error,
                    "warning" => ValidationLevel::Warning,
                    _ => ValidationLevel::Info,
                };

                report.add_issue(ValidationIssue {
                    level,
                    rule_id: id,
                    message: text.trim().to_string(),
                    location,
                    source: source.to_string(),
                });
            }
        }
    }

    /// Fallback : parse le SVRL comme texte brut si le parsing XML échoue
    fn parse_svrl_text(&self, svrl: &str, source: &str, report: &mut ValidationReport) {
        for line in svrl.lines() {
            let trimmed = line.trim();
            if trimmed.contains("failed-assert") || trimmed.contains("svrl:failed-assert") {
                let id = extract_attr(trimmed, "id").unwrap_or_else(|| "UNKNOWN".to_string());
                let flag = extract_attr(trimmed, "flag").unwrap_or_default();
                let location = extract_attr(trimmed, "location");

                let level = match flag.to_lowercase().as_str() {
                    "fatal" => ValidationLevel::Fatal,
                    "warning" => ValidationLevel::Warning,
                    _ => ValidationLevel::Error,
                };

                report.add_issue(ValidationIssue {
                    level,
                    rule_id: id,
                    message: "Schematron assertion failed".to_string(),
                    location,
                    source: source.to_string(),
                });
            }
        }
    }

    /// Promouvoir certains warnings en fatal selon RULES_PROMOTED_TO_FATAL.
    /// Les schematrons officiels marquent BR-FR-12/13 comme `flag="warning"`,
    /// mais ces règles sont bloquantes pour l'e-invoicing français.
    fn promote_warnings(report: &mut ValidationReport) {
        for issue in &mut report.issues {
            if issue.level == ValidationLevel::Warning
                && RULES_PROMOTED_TO_FATAL.iter().any(|r| issue.rule_id == *r)
            {
                tracing::debug!(
                    rule = %issue.rule_id,
                    "Promotion warning → fatal (règle bloquante e-invoicing)"
                );
                issue.level = ValidationLevel::Fatal;
            }
        }
    }

    /// Valide une facture UBL avec EN16931 + BR-FR **en parallèle**.
    /// Les deux validations Schematron s'exécutent simultanément sur le même
    /// fichier temporaire, divisant le temps par ~2.
    pub fn validate_ubl_invoice(&self, xml: &str) -> ValidationReport {
        self.validate_parallel(xml, &SchematronType::En16931Ubl, &SchematronType::BrFrUbl, "Schematron UBL")
    }

    /// Valide une facture CII avec EN16931 + BR-FR **en parallèle**.
    pub fn validate_cii_invoice(&self, xml: &str) -> ValidationReport {
        self.validate_parallel(xml, &SchematronType::En16931Cii, &SchematronType::BrFrCii, "Schematron CII")
    }

    /// Exécute deux validations Schematron en parallèle sur le même fichier temporaire.
    fn validate_parallel(
        &self,
        xml: &str,
        type_a: &SchematronType,
        type_b: &SchematronType,
        report_name: &str,
    ) -> ValidationReport {
        // Écrire le XML une seule fois
        let tmp_dir = std::env::temp_dir();
        let tmp_xml = tmp_dir.join(format!("pdp-validate-{}.xml", uuid::Uuid::new_v4()));
        if let Err(e) = std::fs::write(&tmp_xml, xml) {
            let mut report = ValidationReport::new(report_name);
            report.add_issue(ValidationIssue {
                level: ValidationLevel::Fatal,
                rule_id: "SCH-001".to_string(),
                message: format!("Impossible d'écrire le fichier temporaire: {}", e),
                location: None,
                source: report_name.to_string(),
            });
            return report;
        }

        // Lancer les deux validations en parallèle (threads OS)
        let (report_a, report_b) = std::thread::scope(|s| {
            let handle_a = s.spawn(|| self.validate_with_file(&tmp_xml, type_a));
            let handle_b = s.spawn(|| self.validate_with_file(&tmp_xml, type_b));
            (
                handle_a.join().expect("thread Schematron A panic"),
                handle_b.join().expect("thread Schematron B panic"),
            )
        });

        // Nettoyer le fichier temporaire
        let _ = std::fs::remove_file(&tmp_xml);

        // Fusionner les résultats
        let mut report = ValidationReport::new(report_name);
        report.merge(report_a);
        report.merge(report_b);
        report
    }

    /// Valide un CDAR avec BR-FR-CDV
    pub fn validate_cdar(&self, xml: &str) -> ValidationReport {
        self.validate(xml, &SchematronType::BrFrCdar)
    }
}

/// Extrait un attribut XML d'une chaîne de texte
fn extract_attr(text: &str, attr_name: &str) -> Option<String> {
    let pattern = format!("{}=\"", attr_name);
    if let Some(start) = text.find(&pattern) {
        let after = &text[start + pattern.len()..];
        if let Some(end) = after.find('"') {
            return Some(after[..end].to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn specs_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../specs")
    }

    #[test]
    fn test_validate_en16931_ubl_example() {
        let specs = specs_dir();
        let xslt = specs.join("xslt/en16931/EN16931-UBL-validation.xslt");
        if !xslt.exists() {
            eprintln!("XSLT EN16931 UBL non disponible, test ignoré");
            return;
        }

        let validator = SchematronValidator::new(&specs);
        let example = specs.join("examples/en16931/ubl/ubl-tc434-example1.xml");
        if !example.exists() {
            eprintln!("Exemple UBL non disponible, test ignoré");
            return;
        }

        let xml = std::fs::read_to_string(&example).unwrap();
        let report = validator.validate(&xml, &SchematronType::En16931Ubl);
        println!("EN16931 UBL: {}", report.summary());
        for issue in &report.issues {
            println!("  {}", issue);
        }
        // Les exemples officiels EN16931 doivent être valides
        assert!(report.is_valid(), "L'exemple officiel EN16931 UBL devrait être valide: {:?}",
            report.issues.iter().filter(|i| matches!(i.level, ValidationLevel::Fatal | ValidationLevel::Error)).collect::<Vec<_>>());
    }

    #[test]
    fn test_validate_en16931_cii_example() {
        let specs = specs_dir();
        let xslt = specs.join("xslt/en16931/EN16931-CII-validation.xslt");
        if !xslt.exists() {
            eprintln!("XSLT EN16931 CII non disponible, test ignoré");
            return;
        }

        let validator = SchematronValidator::new(&specs);
        let example = specs.join("examples/en16931/cii/CII_example1.xml");
        if !example.exists() {
            eprintln!("Exemple CII non disponible, test ignoré");
            return;
        }

        let xml = std::fs::read_to_string(&example).unwrap();
        let report = validator.validate(&xml, &SchematronType::En16931Cii);
        println!("EN16931 CII: {}", report.summary());
        for issue in &report.issues {
            println!("  {}", issue);
        }
        assert!(report.is_valid(), "L'exemple officiel EN16931 CII devrait être valide: {:?}",
            report.issues.iter().filter(|i| matches!(i.level, ValidationLevel::Fatal | ValidationLevel::Error)).collect::<Vec<_>>());
    }

    #[test]
    fn test_validate_br_fr_ubl() {
        let specs = specs_dir();
        let xslt = specs.join("xslt/fr/20251114_BR-FR-Flux2-Schematron-UBL_V1.2.0-compiled.xsl");
        if !xslt.exists() {
            eprintln!("XSLT BR-FR UBL non disponible, test ignoré");
            return;
        }

        let validator = SchematronValidator::new(&specs);
        let example = specs.join("examples/xp-z12/factures/F202500001/F202500001_INV_20250201_UBL.xml");
        if !example.exists() {
            eprintln!("Exemple XP Z12 UBL non disponible, test ignoré");
            return;
        }

        let xml = std::fs::read_to_string(&example).unwrap();
        let report = validator.validate(&xml, &SchematronType::BrFrUbl);
        println!("BR-FR UBL: {}", report.summary());
        for issue in &report.issues {
            println!("  {}", issue);
        }
    }

    #[test]
    fn test_validate_cdar_example() {
        let specs = specs_dir();
        let xslt = specs.join("xslt/fr/20251114_BR-FR-CDV-Schematron-CDAR_V1.2.0-compiled.xsl");
        if !xslt.exists() {
            eprintln!("XSLT BR-FR CDAR non disponible, test ignoré");
            return;
        }

        let validator = SchematronValidator::new(&specs);
        let example = specs.join("examples/xp-z12/cdar/_Cas Nominal/UC1_F202500003_01-CDV-200_Deposee.xml");
        if !example.exists() {
            eprintln!("Exemple CDAR non disponible, test ignoré");
            return;
        }

        let xml = std::fs::read_to_string(&example).unwrap();
        let report = validator.validate_cdar(&xml);
        println!("BR-FR CDAR: {}", report.summary());
        for issue in &report.issues {
            println!("  {}", issue);
        }
    }

    #[test]
    fn test_validate_full_ubl() {
        let specs = specs_dir();
        let validator = SchematronValidator::new(&specs);
        let example = specs.join("examples/xp-z12/factures/F202500001/F202500001_INV_20250201_UBL.xml");
        if !example.exists() {
            eprintln!("Exemple XP Z12 UBL non disponible, test ignoré");
            return;
        }

        let xml = std::fs::read_to_string(&example).unwrap();
        let report = validator.validate_ubl_invoice(&xml);
        println!("Full UBL validation: {}", report.summary());
        for issue in &report.issues {
            println!("  {}", issue);
        }
    }

    #[test]
    fn test_extract_attr() {
        assert_eq!(extract_attr(r#"id="BR-01" flag="fatal""#, "id"), Some("BR-01".to_string()));
        assert_eq!(extract_attr(r#"id="BR-01" flag="fatal""#, "flag"), Some("fatal".to_string()));
        assert_eq!(extract_attr(r#"id="BR-01""#, "location"), None);
    }
}
