//! Moteur de transformation XSLT via Saxon (XSLT 2.0/3.0).
//!
//! Exécute les transformations UBL↔CII en appliquant les XSLT
//! sur le XML brut (pas sur InvoiceData parsé).

use std::path::{Path, PathBuf};
use std::process::Command;

use pdp_core::error::{PdpError, PdpResult};

/// Moteur XSLT utilisant Saxon pour les transformations XML↔XML
pub struct XsltEngine {
    /// Répertoire racine des specs (contient xslt/ubl-cii-convert-1.0/, xslt/ppf-flux1-v3.1/, etc.)
    specs_dir: PathBuf,
}

/// Direction de la transformation
#[derive(Debug, Clone, Copy)]
pub enum XsltTransform {
    /// UBL Invoice → CII CrossIndustryInvoice
    UblToCii,
    /// CII CrossIndustryInvoice → UBL Invoice
    CiiToUbl,
    /// CII → Flux 1 Base CII (PPF)
    CiiToF1BaseCii,
    /// UBL → Flux 1 Base UBL (PPF)
    UblToF1BaseUbl,
    /// CII → Flux 1 Full CII (PPF)
    CiiToF1FullCii,
    /// UBL → Flux 1 Full UBL (PPF)
    UblToF1FullUbl,
}

impl std::fmt::Display for XsltTransform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            XsltTransform::UblToCii => write!(f, "UBL→CII"),
            XsltTransform::CiiToUbl => write!(f, "CII→UBL"),
            XsltTransform::CiiToF1BaseCii => write!(f, "CII→F1Base-CII"),
            XsltTransform::UblToF1BaseUbl => write!(f, "UBL→F1Base-UBL"),
            XsltTransform::CiiToF1FullCii => write!(f, "CII→F1Full-CII"),
            XsltTransform::UblToF1FullUbl => write!(f, "UBL→F1Full-UBL"),
        }
    }
}

impl XsltEngine {
    pub fn new(specs_dir: &Path) -> Self {
        Self {
            specs_dir: specs_dir.to_path_buf(),
        }
    }

    /// Construit le moteur en déduisant le chemin specs depuis CARGO_MANIFEST_DIR
    pub fn from_manifest_dir() -> Self {
        let specs = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../specs");
        Self::new(&specs)
    }

    /// Chemin du XSLT de conversion
    fn xslt_path(&self, transform: XsltTransform) -> PathBuf {
        match transform {
            XsltTransform::UblToCii => self.specs_dir.join("xslt/ubl-cii-convert-1.0/UBL-to-CII.xslt"),
            XsltTransform::CiiToUbl => self.specs_dir.join("xslt/ubl-cii-convert-1.0/CII-to-UBL.xslt"),
            XsltTransform::CiiToF1BaseCii => self.specs_dir.join("xslt/ppf-flux1-v3.1/CII-to-F1Base-CII.xslt"),
            XsltTransform::UblToF1BaseUbl => self.specs_dir.join("xslt/ppf-flux1-v3.1/UBL-to-F1Base-UBL.xslt"),
            XsltTransform::CiiToF1FullCii => self.specs_dir.join("xslt/ppf-flux1-v3.1/CII-to-F1Full-CII.xslt"),
            XsltTransform::UblToF1FullUbl => self.specs_dir.join("xslt/ppf-flux1-v3.1/UBL-to-F1Full-UBL.xslt"),
        }
    }

    /// Transforme un XML brut via XSLT.
    /// Retourne le XML résultat en String.
    pub fn transform(&self, xml: &str, direction: XsltTransform) -> PdpResult<String> {
        let xslt_path = self.xslt_path(direction);
        if !xslt_path.exists() {
            return Err(PdpError::TransformError {
                source_format: direction.to_string(),
                target_format: direction.to_string(),
                message: format!("XSLT de conversion introuvable: {}", xslt_path.display()),
            });
        }

        // Écrire le XML source dans un fichier temporaire
        let tmp_dir = std::env::temp_dir();
        let tmp_xml = tmp_dir.join(format!("pdp-xslt-in-{}.xml", uuid::Uuid::new_v4()));
        std::fs::write(&tmp_xml, xml).map_err(|e| PdpError::TransformError {
            source_format: direction.to_string(),
            target_format: direction.to_string(),
            message: format!("Impossible d'écrire le fichier temporaire: {}", e),
        })?;

        tracing::debug!(
            transform = %direction,
            xslt = %xslt_path.display(),
            "Transformation XSLT en cours via Saxon"
        );

        // Exécuter SaxonC natif (transform) en priorité, fallback SaxonJ (saxon)
        let xml_str = tmp_xml.to_str().unwrap_or("");
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
            });

        // Nettoyer
        let _ = std::fs::remove_file(&tmp_xml);

        let output = result.map_err(|e| PdpError::TransformError {
            source_format: direction.to_string(),
            target_format: direction.to_string(),
            message: format!(
                "Impossible d'exécuter Saxon: {}. Vérifiez que Saxon est installé.",
                e
            ),
        })?;

        if !output.status.success() && output.stdout.is_empty() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(PdpError::TransformError {
                source_format: direction.to_string(),
                target_format: direction.to_string(),
                message: format!("Saxon a échoué: {}", stderr.trim()),
            });
        }

        let result_xml = String::from_utf8(output.stdout).map_err(|e| {
            PdpError::TransformError {
                source_format: direction.to_string(),
                target_format: direction.to_string(),
                message: format!("Le résultat XSLT n'est pas du UTF-8 valide: {}", e),
            }
        })?;

        if result_xml.trim().is_empty() {
            return Err(PdpError::TransformError {
                source_format: direction.to_string(),
                target_format: direction.to_string(),
                message: "Le résultat XSLT est vide".to_string(),
            });
        }

        tracing::info!(transform = %direction, "Transformation XSLT réussie");
        Ok(result_xml)
    }

    /// UBL XML → CII XML
    pub fn ubl_to_cii(&self, ubl_xml: &str) -> PdpResult<String> {
        self.transform(ubl_xml, XsltTransform::UblToCii)
    }

    /// CII XML → UBL XML
    pub fn cii_to_ubl(&self, cii_xml: &str) -> PdpResult<String> {
        self.transform(cii_xml, XsltTransform::CiiToUbl)
    }

    /// CII XML → Flux 1 Base CII (PPF)
    pub fn cii_to_f1base_cii(&self, cii_xml: &str) -> PdpResult<String> {
        self.transform(cii_xml, XsltTransform::CiiToF1BaseCii)
    }

    /// UBL XML → Flux 1 Base UBL (PPF)
    pub fn ubl_to_f1base_ubl(&self, ubl_xml: &str) -> PdpResult<String> {
        self.transform(ubl_xml, XsltTransform::UblToF1BaseUbl)
    }

    /// CII XML → Flux 1 Full CII (PPF)
    pub fn cii_to_f1full_cii(&self, cii_xml: &str) -> PdpResult<String> {
        self.transform(cii_xml, XsltTransform::CiiToF1FullCii)
    }

    /// UBL XML → Flux 1 Full UBL (PPF)
    pub fn ubl_to_f1full_ubl(&self, ubl_xml: &str) -> PdpResult<String> {
        self.transform(ubl_xml, XsltTransform::UblToF1FullUbl)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pdp_validate::{SchematronValidator, ValidationLevel};

    fn engine() -> XsltEngine {
        let specs = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../specs");
        XsltEngine::new(&specs)
    }

    fn specs_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../specs")
    }

    /// Valide un XML CII via Schematron EN16931 + BR-FR.
    /// Note: les XSD F1BASE/F1FULL sont trop restrictifs (éléments commentés),
    /// le Schematron EN16931 est le validateur de référence.
    fn validate_cii_schematron(cii_xml: &str, label: &str) -> bool {
        let specs = specs_dir();
        let validator = SchematronValidator::new(&specs);
        let report = validator.validate_cii_invoice(cii_xml);
        if !report.is_valid() {
            eprintln!("[{}] Schematron CII errors:", label);
            for issue in &report.issues {
                if matches!(issue.level, ValidationLevel::Fatal | ValidationLevel::Error) {
                    eprintln!("  [{}] {} - {}", issue.source, issue.rule_id, issue.message);
                }
            }
        }
        report.is_valid()
    }

    /// Valide un XML UBL via Schematron EN16931 + BR-FR.
    fn validate_ubl_schematron(ubl_xml: &str, label: &str) -> bool {
        let specs = specs_dir();
        let validator = SchematronValidator::new(&specs);
        let report = validator.validate_ubl_invoice(ubl_xml);
        if !report.is_valid() {
            eprintln!("[{}] Schematron UBL errors:", label);
            for issue in &report.issues {
                if matches!(issue.level, ValidationLevel::Fatal | ValidationLevel::Error) {
                    eprintln!("  [{}] {} - {}", issue.source, issue.rule_id, issue.message);
                }
            }
        }
        report.is_valid()
    }

    // ===== UBL → CII : contenu =====

    #[test]
    fn test_xslt_ubl_to_cii_content() {
        let ubl_xml = std::fs::read_to_string("../../tests/fixtures/ubl/facture_ubl_001.xml")
            .expect("Fixture UBL introuvable");
        let cii_xml = engine().ubl_to_cii(&ubl_xml).expect("XSLT UBL→CII échoué");

        assert!(cii_xml.contains("CrossIndustryInvoice"));
        assert!(cii_xml.contains("FA-2025-00142"));
        assert!(cii_xml.contains("TechConseil SAS"));
        assert!(cii_xml.contains("12000.00"));
    }

    // ===== CII → UBL : contenu =====

    #[test]
    fn test_xslt_cii_to_ubl_content() {
        let cii_xml = std::fs::read_to_string("../../tests/fixtures/cii/facture_cii_001.xml")
            .expect("Fixture CII introuvable");
        let ubl_xml = engine().cii_to_ubl(&cii_xml).expect("XSLT CII→UBL échoué");

        assert!(ubl_xml.contains("urn:oasis:names:specification:ubl:schema:xsd:Invoice-2"));
        assert!(ubl_xml.contains("FA-2025-00256"));
        assert!(ubl_xml.contains("InfoTech Solutions SARL"));
        assert!(ubl_xml.contains("38400.00"));
    }

    // ===== UBL → CII : Schematron validation sur toutes les fixtures =====

    #[test]
    fn test_xslt_ubl_to_cii_validate_001() {
        let ubl_xml = std::fs::read_to_string("../../tests/fixtures/ubl/facture_ubl_001.xml").unwrap();
        let cii_xml = engine().ubl_to_cii(&ubl_xml).unwrap();
        assert!(validate_cii_schematron(&cii_xml, "UBL→CII facture_001"),
            "facture_ubl_001 → CII doit être Schematron-valide");
    }

    #[test]
    fn test_xslt_ubl_to_cii_validate_marketplace() {
        let ubl_xml = std::fs::read_to_string("../../tests/fixtures/ubl/facture_ubl_marketplace_a8.xml").unwrap();
        let cii_xml = engine().ubl_to_cii(&ubl_xml).unwrap();
        assert!(validate_cii_schematron(&cii_xml, "UBL→CII marketplace"),
            "facture_ubl_marketplace → CII doit être Schematron-valide");
    }

    #[test]
    fn test_xslt_ubl_to_cii_validate_soustraitance() {
        let ubl_xml = std::fs::read_to_string("../../tests/fixtures/ubl/facture_ubl_soustraitance_a4.xml").unwrap();
        let cii_xml = engine().ubl_to_cii(&ubl_xml).unwrap();
        assert!(validate_cii_schematron(&cii_xml, "UBL→CII soustraitance"),
            "facture_ubl_soustraitance → CII doit être Schematron-valide");
    }

    // ===== CII → UBL (SFTI XSLT) : Schematron validation sur toutes les fixtures =====

    #[test]
    fn test_xslt_cii_to_ubl_validate_001() {
        let cii_xml = std::fs::read_to_string("../../tests/fixtures/cii/facture_cii_001.xml").unwrap();
        let ubl_xml = engine().cii_to_ubl(&cii_xml).unwrap();
        assert!(validate_ubl_schematron(&ubl_xml, "CII→UBL facture_001"),
            "facture_cii_001 → UBL doit être Schematron-valide");
    }

    #[test]
    fn test_xslt_cii_to_ubl_validate_avoir() {
        let cii_xml = std::fs::read_to_string("../../tests/fixtures/cii/avoir_cii_381.xml").unwrap();
        let ubl_xml = engine().cii_to_ubl(&cii_xml).unwrap();
        assert!(validate_ubl_schematron(&ubl_xml, "CII→UBL avoir_381"),
            "avoir_cii_381 → UBL doit être Schematron-valide");
    }

    #[test]
    fn test_xslt_cii_to_ubl_validate_rectificative() {
        let cii_xml = std::fs::read_to_string("../../tests/fixtures/cii/facture_rectificative_cii_384.xml").unwrap();
        let ubl_xml = engine().cii_to_ubl(&cii_xml).unwrap();
        assert!(validate_ubl_schematron(&ubl_xml, "CII→UBL rectificative_384"),
            "facture_rectificative_cii_384 → UBL doit être Schematron-valide");
    }

    // ===== Roundtrip =====

    #[test]
    fn test_xslt_roundtrip_ubl_cii_ubl() {
        let ubl_xml = std::fs::read_to_string("../../tests/fixtures/ubl/facture_ubl_001.xml").unwrap();
        let engine = engine();

        let cii_xml = engine.ubl_to_cii(&ubl_xml).unwrap();
        assert!(cii_xml.contains("FA-2025-00142"));

        let ubl_xml2 = engine.cii_to_ubl(&cii_xml).unwrap();
        assert!(ubl_xml2.contains("FA-2025-00142"));
        assert!(ubl_xml2.contains("TechConseil SAS"));

        // Le roundtrip UBL→CII→UBL doit aussi être Schematron-valide
        assert!(validate_ubl_schematron(&ubl_xml2, "roundtrip UBL→CII→UBL"),
            "Le roundtrip UBL→CII→UBL doit être Schematron-valide");
    }

    #[test]
    fn test_xslt_roundtrip_cii_ubl_cii() {
        let cii_xml = std::fs::read_to_string("../../tests/fixtures/cii/facture_cii_001.xml").unwrap();
        let engine = engine();

        let ubl_xml = engine.cii_to_ubl(&cii_xml).unwrap();
        assert!(ubl_xml.contains("FA-2025-00256"));

        let cii_xml2 = engine.ubl_to_cii(&ubl_xml).unwrap();
        assert!(cii_xml2.contains("FA-2025-00256"));
        assert!(cii_xml2.contains("InfoTech Solutions SARL"));

        // Le roundtrip CII→UBL→CII doit aussi être Schematron-valide
        assert!(validate_cii_schematron(&cii_xml2, "roundtrip CII→UBL→CII"),
            "Le roundtrip CII→UBL→CII doit être Schematron-valide");
    }
}
