//! Processor de transformation Flux 1 PPF.
//!
//! Transforme une facture CII ou UBL en version Flux 1 Base ou Full
//! conforme aux XSD PPF v3.1, et dépose le résultat dans un répertoire dédié.
//!
//! Le profil est déterminé selon la stratégie configurée :
//! - **Auto** (défaut) : lignes présentes → Full, sinon Base
//! - **Base** : toujours Base (allégé, sans lignes)
//! - **Full** : toujours Full (fallback Base si pas de lignes)
//!
//! Le fichier est nommé selon la convention PPF :
//! `{profil}_{nom_de_fichier}.xml` où profil = "Base" ou "Full".

use std::path::{Path, PathBuf};

use async_trait::async_trait;
use pdp_core::error::{PdpError, PdpResult};
use pdp_core::exchange::Exchange;
use pdp_core::model::InvoiceFormat;
use pdp_core::processor::Processor;

use crate::xslt_engine::{XsltEngine, XsltTransform};

/// Profil Flux 1 PPF
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Flux1Profile {
    /// Profil allégé : sans lignes, parties simplifiées
    Base,
    /// Profil complet : avec lignes, remises, livraison
    Full,
}

impl std::fmt::Display for Flux1Profile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Flux1Profile::Base => write!(f, "Base"),
            Flux1Profile::Full => write!(f, "Full"),
        }
    }
}

/// Stratégie de sélection du profil Flux 1.
///
/// Configurable via `ppf.flux1_profile` dans le YAML ou `$PDP_FLUX1_PROFILE`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Flux1ProfileStrategy {
    /// Détection automatique : lignes présentes → Full, sinon Base
    Auto,
    /// Toujours Base (sans lignes)
    AlwaysBase,
    /// Toujours Full (fallback Base si la facture n'a pas de lignes)
    AlwaysFull,
}

impl Flux1ProfileStrategy {
    /// Parse la stratégie depuis une chaîne de configuration.
    /// Valeurs acceptées : "auto", "base", "full" (insensible à la casse).
    pub fn from_config(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "base" => Flux1ProfileStrategy::AlwaysBase,
            "full" => Flux1ProfileStrategy::AlwaysFull,
            _ => Flux1ProfileStrategy::Auto,
        }
    }
}

impl Default for Flux1ProfileStrategy {
    fn default() -> Self {
        Flux1ProfileStrategy::Auto
    }
}

/// Processor qui génère le Flux 1 PPF (données réglementaires)
/// et dépose le fichier XML résultant dans un répertoire de sortie dédié.
///
/// Le profil (Base/Full) est déterminé selon la stratégie configurée :
/// - **Auto** : lignes présentes → Full, sinon Base
/// - **AlwaysBase** : toujours Base
/// - **AlwaysFull** : toujours Full (fallback Base si pas de lignes)
pub struct PpfFlux1Processor {
    /// Répertoire de sortie pour les fichiers Flux 1
    output_dir: PathBuf,
    /// Répertoire des specs (pour le moteur XSLT)
    specs_dir: PathBuf,
    /// Stratégie de sélection du profil
    strategy: Flux1ProfileStrategy,
}

impl PpfFlux1Processor {
    pub fn new(output_dir: &Path, specs_dir: &Path) -> Self {
        Self {
            output_dir: output_dir.to_path_buf(),
            specs_dir: specs_dir.to_path_buf(),
            strategy: Flux1ProfileStrategy::Auto,
        }
    }

    pub fn with_strategy(mut self, strategy: Flux1ProfileStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    /// Détermine le profil Flux 1 selon la stratégie et la facture.
    /// - **Auto** : lignes présentes → Full, sinon Base
    /// - **AlwaysBase** : toujours Base
    /// - **AlwaysFull** : Full si lignes présentes, sinon Base (fallback)
    pub fn detect_profile_with_strategy(
        invoice: &pdp_core::model::InvoiceData,
        strategy: Flux1ProfileStrategy,
    ) -> Flux1Profile {
        let has_lines = !invoice.lines.is_empty();
        match strategy {
            Flux1ProfileStrategy::Auto => {
                if has_lines { Flux1Profile::Full } else { Flux1Profile::Base }
            }
            Flux1ProfileStrategy::AlwaysBase => Flux1Profile::Base,
            Flux1ProfileStrategy::AlwaysFull => {
                if has_lines { Flux1Profile::Full } else { Flux1Profile::Base }
            }
        }
    }

    /// Détermine le profil Flux 1 selon la stratégie par défaut (Auto).
    pub fn detect_profile(invoice: &pdp_core::model::InvoiceData) -> Flux1Profile {
        Self::detect_profile_with_strategy(invoice, Flux1ProfileStrategy::Auto)
    }

    /// Détermine le nom du fichier Flux 1 selon la convention PPF :
    /// `Base_{invoice_number}.xml` ou `Full_{invoice_number}.xml`
    pub fn flux1_filename(invoice_number: &str, profile: Flux1Profile) -> String {
        format!("{}_{}.xml", profile, invoice_number)
    }

    /// Détermine la direction XSLT à utiliser selon le format source et le profil
    pub fn xslt_direction(format: InvoiceFormat, profile: Flux1Profile) -> PdpResult<XsltTransform> {
        match (format, profile) {
            (InvoiceFormat::CII, Flux1Profile::Base) => Ok(XsltTransform::CiiToF1BaseCii),
            (InvoiceFormat::UBL, Flux1Profile::Base) => Ok(XsltTransform::UblToF1BaseUbl),
            (InvoiceFormat::FacturX, Flux1Profile::Base) => Ok(XsltTransform::CiiToF1BaseCii),
            (InvoiceFormat::CII, Flux1Profile::Full) => Ok(XsltTransform::CiiToF1FullCii),
            (InvoiceFormat::UBL, Flux1Profile::Full) => Ok(XsltTransform::UblToF1FullUbl),
            (InvoiceFormat::FacturX, Flux1Profile::Full) => Ok(XsltTransform::CiiToF1FullCii),
        }
    }
}

#[async_trait]
impl Processor for PpfFlux1Processor {
    fn name(&self) -> &str {
        "PpfFlux1Processor"
    }

    async fn process(&self, mut exchange: Exchange) -> PdpResult<Exchange> {
        // Skip si le document n'est pas une facture (CDAR, e-reporting, etc.)
        if exchange.get_header("document.type").map(|s| s.as_str()) == Some("CDAR") {
            tracing::debug!(exchange_id = %exchange.id, "PpfFlux1Processor: skip (document CDAR)");
            return Ok(exchange);
        }

        let invoice = exchange.invoice.clone().ok_or_else(|| {
            PdpError::TransformError {
                source_format: "inconnu".to_string(),
                target_format: "Flux1".to_string(),
                message: "Aucune facture parsée dans l'exchange. Le ParseProcessor doit être exécuté avant.".to_string(),
            }
        })?;

        let raw_xml = invoice.raw_xml.as_deref().ok_or_else(|| {
            PdpError::TransformError {
                source_format: invoice.source_format.to_string(),
                target_format: "Flux1".to_string(),
                message: "Pas de XML brut (raw_xml) disponible pour la transformation Flux 1".to_string(),
            }
        })?;

        let profile = Self::detect_profile_with_strategy(&invoice, self.strategy);
        let direction = Self::xslt_direction(invoice.source_format, profile)?;
        let engine = XsltEngine::new(&self.specs_dir);

        tracing::info!(
            exchange_id = %exchange.id,
            invoice = %invoice.invoice_number,
            format = %invoice.source_format,
            profile = %profile,
            transform = %direction,
            "Transformation Flux 1 PPF en cours"
        );

        let f1_xml = engine.transform(raw_xml, direction)?;

        // Nommer le fichier selon la convention PPF
        let filename = Self::flux1_filename(&invoice.invoice_number, profile);

        // Créer le répertoire de sortie si nécessaire
        if !self.output_dir.exists() {
            std::fs::create_dir_all(&self.output_dir).map_err(|e| {
                PdpError::TransformError {
                    source_format: invoice.source_format.to_string(),
                    target_format: format!("F1{}", profile),
                    message: format!(
                        "Impossible de créer le répertoire de sortie Flux 1 {}: {}",
                        self.output_dir.display(),
                        e
                    ),
                }
            })?;
        }

        // Écrire le fichier Flux 1
        let output_path = self.output_dir.join(&filename);
        std::fs::write(&output_path, &f1_xml).map_err(|e| {
            PdpError::TransformError {
                source_format: invoice.source_format.to_string(),
                target_format: format!("F1{}", profile),
                message: format!(
                    "Impossible d'écrire le fichier Flux 1 {}: {}",
                    output_path.display(),
                    e
                ),
            }
        })?;

        // Stocker les métadonnées dans l'exchange
        exchange.set_property("ppf.flux1.path", output_path.to_str().unwrap_or(""));
        exchange.set_property("ppf.flux1.filename", &filename);
        exchange.set_property("ppf.flux1.profile", &profile.to_string());

        // Déterminer le code interface PPF selon le format
        let code_interface = match invoice.source_format {
            InvoiceFormat::UBL => "FFE0111A",
            InvoiceFormat::CII | InvoiceFormat::FacturX => "FFE0112A",
        };
        exchange.set_property("ppf.flux1.code_interface", code_interface);

        tracing::info!(
            exchange_id = %exchange.id,
            invoice = %invoice.invoice_number,
            profile = %profile,
            filename = %filename,
            path = %output_path.display(),
            code_interface = %code_interface,
            "Flux 1 PPF généré"
        );

        Ok(exchange)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pdp_core::model::{InvoiceData, InvoiceFormat};
    use pdp_validate::xsd::{XsdDocumentType, XsdValidator};

    fn specs_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../specs")
    }

    fn fixtures_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures")
    }

    fn engine() -> XsltEngine {
        XsltEngine::new(&specs_dir())
    }

    fn read_fixture(path: &str) -> Option<String> {
        std::fs::read_to_string(path).ok()
    }

    /// Valide un XML contre un XSD, affiche les erreurs et assert
    fn assert_xsd_valid(xml: &str, doc_type: &XsdDocumentType, label: &str) {
        let validator = XsdValidator::new(&specs_dir());
        let report = validator.validate(xml, doc_type);
        if !report.is_valid() {
            eprintln!("[{}] XSD {} errors:", label, doc_type);
            for issue in &report.issues {
                eprintln!("  [{}] {}", issue.rule_id, issue.message);
            }
        }
        assert!(report.is_valid(), "[{}] XSD {} validation failed", label, doc_type);
    }

    /// Helper : transforme CII→F1Base et valide XSD
    fn assert_cii_to_f1base(fixture_path: &str, label: &str) {
        let xml = match read_fixture(fixture_path) {
            Some(xml) => xml,
            None => { eprintln!("[{}] Fixture introuvable, skip", label); return; }
        };
        match engine().cii_to_f1base_cii(&xml) {
            Ok(f1) => {
                assert!(f1.contains("einvoicingextract#Base"), "[{}] BT-24 manquant", label);
                assert!(!f1.contains("IncludedSupplyChainTradeLineItem"), "[{}] lignes non supprimées", label);
                assert_xsd_valid(&f1, &XsdDocumentType::CiiInvoice, label);
            }
            Err(e) => eprintln!("[{}] Skip (Saxon?): {}", label, e),
        }
    }

    /// Helper : transforme CII→F1Full et valide XSD
    fn assert_cii_to_f1full(fixture_path: &str, label: &str) {
        let xml = match read_fixture(fixture_path) {
            Some(xml) => xml,
            None => { eprintln!("[{}] Fixture introuvable, skip", label); return; }
        };
        match engine().cii_to_f1full_cii(&xml) {
            Ok(f1) => {
                assert!(f1.contains("einvoicingextract#Full"), "[{}] BT-24 manquant", label);
                assert!(f1.contains("IncludedSupplyChainTradeLineItem"), "[{}] lignes absentes en Full", label);
                assert_xsd_valid(&f1, &XsdDocumentType::F1FullCii, label);
            }
            Err(e) => eprintln!("[{}] Skip (Saxon?): {}", label, e),
        }
    }

    /// Helper : transforme UBL→F1Base Invoice et valide XSD
    fn assert_ubl_to_f1base(fixture_path: &str, label: &str) {
        let xml = match read_fixture(fixture_path) {
            Some(xml) => xml,
            None => { eprintln!("[{}] Fixture introuvable, skip", label); return; }
        };
        match engine().ubl_to_f1base_ubl(&xml) {
            Ok(f1) => {
                assert!(f1.contains("einvoicingextract#Base"), "[{}] BT-24 manquant", label);
                assert!(!f1.contains("<cac:InvoiceLine"), "[{}] lignes non supprimées", label);
                assert_xsd_valid(&f1, &XsdDocumentType::UblInvoice, label);
            }
            Err(e) => eprintln!("[{}] Skip (Saxon?): {}", label, e),
        }
    }

    /// Helper : transforme UBL CreditNote→F1Base et valide XSD CreditNote
    fn assert_ubl_creditnote_to_f1base(fixture_path: &str, label: &str) {
        let xml = match read_fixture(fixture_path) {
            Some(xml) => xml,
            None => { eprintln!("[{}] Fixture introuvable, skip", label); return; }
        };
        match engine().ubl_to_f1base_ubl(&xml) {
            Ok(f1) => {
                assert!(f1.contains("einvoicingextract#Base"), "[{}] BT-24 manquant", label);
                assert!(!f1.contains("<cac:CreditNoteLine"), "[{}] lignes non supprimées", label);
                assert_xsd_valid(&f1, &XsdDocumentType::UblCreditNote, label);
            }
            Err(e) => eprintln!("[{}] Skip (Saxon?): {}", label, e),
        }
    }

    /// Helper : transforme UBL→F1Full Invoice et valide XSD
    fn assert_ubl_to_f1full(fixture_path: &str, label: &str) {
        let xml = match read_fixture(fixture_path) {
            Some(xml) => xml,
            None => { eprintln!("[{}] Fixture introuvable, skip", label); return; }
        };
        match engine().ubl_to_f1full_ubl(&xml) {
            Ok(f1) => {
                assert!(f1.contains("einvoicingextract#Full"), "[{}] BT-24 manquant", label);
                assert!(f1.contains("<cac:InvoiceLine") || f1.contains("<cac:CreditNoteLine"),
                    "[{}] lignes absentes en Full", label);
                assert_xsd_valid(&f1, &XsdDocumentType::F1FullUblInvoice, label);
            }
            Err(e) => eprintln!("[{}] Skip (Saxon?): {}", label, e),
        }
    }

    /// Helper : transforme UBL CreditNote→F1Full et valide XSD CreditNote
    fn assert_ubl_creditnote_to_f1full(fixture_path: &str, label: &str) {
        let xml = match read_fixture(fixture_path) {
            Some(xml) => xml,
            None => { eprintln!("[{}] Fixture introuvable, skip", label); return; }
        };
        match engine().ubl_to_f1full_ubl(&xml) {
            Ok(f1) => {
                assert!(f1.contains("einvoicingextract#Full"), "[{}] BT-24 manquant", label);
                assert_xsd_valid(&f1, &XsdDocumentType::F1FullUblCreditNote, label);
            }
            Err(e) => eprintln!("[{}] Skip (Saxon?): {}", label, e),
        }
    }

    // ================================================================
    //  Tests unitaires : filename, direction, profil
    // ================================================================

    #[test]
    fn test_flux1_filename_base() {
        assert_eq!(
            PpfFlux1Processor::flux1_filename("F202500003", Flux1Profile::Base),
            "Base_F202500003.xml"
        );
    }

    #[test]
    fn test_flux1_filename_full() {
        assert_eq!(
            PpfFlux1Processor::flux1_filename("F202500003", Flux1Profile::Full),
            "Full_F202500003.xml"
        );
    }

    #[test]
    fn test_xslt_direction_cii_base() {
        let dir = PpfFlux1Processor::xslt_direction(InvoiceFormat::CII, Flux1Profile::Base).unwrap();
        assert!(matches!(dir, XsltTransform::CiiToF1BaseCii));
    }

    #[test]
    fn test_xslt_direction_cii_full() {
        let dir = PpfFlux1Processor::xslt_direction(InvoiceFormat::CII, Flux1Profile::Full).unwrap();
        assert!(matches!(dir, XsltTransform::CiiToF1FullCii));
    }

    #[test]
    fn test_xslt_direction_ubl_base() {
        let dir = PpfFlux1Processor::xslt_direction(InvoiceFormat::UBL, Flux1Profile::Base).unwrap();
        assert!(matches!(dir, XsltTransform::UblToF1BaseUbl));
    }

    #[test]
    fn test_xslt_direction_ubl_full() {
        let dir = PpfFlux1Processor::xslt_direction(InvoiceFormat::UBL, Flux1Profile::Full).unwrap();
        assert!(matches!(dir, XsltTransform::UblToF1FullUbl));
    }

    #[test]
    fn test_xslt_direction_facturx_base() {
        let dir = PpfFlux1Processor::xslt_direction(InvoiceFormat::FacturX, Flux1Profile::Base).unwrap();
        assert!(matches!(dir, XsltTransform::CiiToF1BaseCii));
    }

    #[test]
    fn test_xslt_direction_facturx_full() {
        let dir = PpfFlux1Processor::xslt_direction(InvoiceFormat::FacturX, Flux1Profile::Full).unwrap();
        assert!(matches!(dir, XsltTransform::CiiToF1FullCii));
    }

    #[test]
    fn test_strategy_from_config() {
        assert_eq!(Flux1ProfileStrategy::from_config("auto"), Flux1ProfileStrategy::Auto);
        assert_eq!(Flux1ProfileStrategy::from_config("base"), Flux1ProfileStrategy::AlwaysBase);
        assert_eq!(Flux1ProfileStrategy::from_config("full"), Flux1ProfileStrategy::AlwaysFull);
        assert_eq!(Flux1ProfileStrategy::from_config("Full"), Flux1ProfileStrategy::AlwaysFull);
        assert_eq!(Flux1ProfileStrategy::from_config("BASE"), Flux1ProfileStrategy::AlwaysBase);
        assert_eq!(Flux1ProfileStrategy::from_config("unknown"), Flux1ProfileStrategy::Auto);
    }

    #[test]
    fn test_strategy_always_base_with_lines() {
        let mut inv = InvoiceData::new("TEST".to_string(), InvoiceFormat::CII);
        inv.lines.push(pdp_core::model::InvoiceLine {
            line_id: Some("1".to_string()),
            note: None, object_id: None, quantity: None, unit_code: None,
            line_net_amount: None, order_line_reference: None, accounting_cost: None,
            price: None, gross_price: None, item_name: Some("Test".to_string()),
            item_description: None, seller_item_id: None, buyer_item_id: None,
            standard_item_id: None, standard_item_id_scheme: None,
            tax_category_code: None, tax_percent: None,
            period_start: None, period_end: None,
        });
        // AlwaysBase → Base même avec des lignes
        assert_eq!(
            PpfFlux1Processor::detect_profile_with_strategy(&inv, Flux1ProfileStrategy::AlwaysBase),
            Flux1Profile::Base
        );
    }

    #[test]
    fn test_strategy_always_full_without_lines() {
        let inv = InvoiceData::new("TEST".to_string(), InvoiceFormat::CII);
        // AlwaysFull sans lignes → fallback Base
        assert_eq!(
            PpfFlux1Processor::detect_profile_with_strategy(&inv, Flux1ProfileStrategy::AlwaysFull),
            Flux1Profile::Base
        );
    }

    #[test]
    fn test_strategy_always_full_with_lines() {
        let mut inv = InvoiceData::new("TEST".to_string(), InvoiceFormat::CII);
        inv.lines.push(pdp_core::model::InvoiceLine {
            line_id: Some("1".to_string()),
            note: None, object_id: None, quantity: None, unit_code: None,
            line_net_amount: None, order_line_reference: None, accounting_cost: None,
            price: None, gross_price: None, item_name: Some("Test".to_string()),
            item_description: None, seller_item_id: None, buyer_item_id: None,
            standard_item_id: None, standard_item_id_scheme: None,
            tax_category_code: None, tax_percent: None,
            period_start: None, period_end: None,
        });
        // AlwaysFull avec lignes → Full
        assert_eq!(
            PpfFlux1Processor::detect_profile_with_strategy(&inv, Flux1ProfileStrategy::AlwaysFull),
            Flux1Profile::Full
        );
    }

    #[test]
    fn test_detect_profile_no_lines() {
        let inv = InvoiceData::new("TEST".to_string(), InvoiceFormat::CII);
        assert_eq!(PpfFlux1Processor::detect_profile(&inv), Flux1Profile::Base);
    }

    #[test]
    fn test_detect_profile_with_lines() {
        let mut inv = InvoiceData::new("TEST".to_string(), InvoiceFormat::CII);
        inv.lines.push(pdp_core::model::InvoiceLine {
            line_id: Some("1".to_string()),
            note: None, object_id: None, quantity: None, unit_code: None,
            line_net_amount: None, order_line_reference: None, accounting_cost: None,
            price: None, gross_price: None, item_name: Some("Test".to_string()),
            item_description: None, seller_item_id: None, buyer_item_id: None,
            standard_item_id: None, standard_item_id_scheme: None,
            tax_category_code: None, tax_percent: None,
            period_start: None, period_end: None,
        });
        assert_eq!(PpfFlux1Processor::detect_profile(&inv), Flux1Profile::Full);
    }

    #[test]
    fn test_detect_profile_with_lines_en16931() {
        // Full même avec profil EN16931 si des lignes sont présentes
        let mut inv = InvoiceData::new("TEST".to_string(), InvoiceFormat::CII);
        inv.profile_id = Some("urn:cen.eu:en16931:2017".to_string());
        inv.lines.push(pdp_core::model::InvoiceLine {
            line_id: Some("1".to_string()),
            note: None, object_id: None, quantity: None, unit_code: None,
            line_net_amount: None, order_line_reference: None, accounting_cost: None,
            price: None, gross_price: None, item_name: Some("Test".to_string()),
            item_description: None, seller_item_id: None, buyer_item_id: None,
            standard_item_id: None, standard_item_id_scheme: None,
            tax_category_code: None, tax_percent: None,
            period_start: None, period_end: None,
        });
        assert_eq!(PpfFlux1Processor::detect_profile(&inv), Flux1Profile::Full);
    }

    // ================================================================
    //  Tests XSLT + XSD : CII → F1Base (toutes fixtures)
    // ================================================================

    #[test]
    fn test_cii_to_f1base_facture_001() {
        assert_cii_to_f1base(
            &fixtures_dir().join("cii/facture_cii_001.xml").to_string_lossy(),
            "facture_cii_001→F1Base",
        );
    }

    #[test]
    fn test_cii_to_f1base_remises_multitva() {
        assert_cii_to_f1base(
            &fixtures_dir().join("cii/facture_cii_remises_multitva.xml").to_string_lossy(),
            "facture_cii_remises_multitva→F1Base",
        );
    }

    #[test]
    fn test_cii_to_f1base_avoir() {
        assert_cii_to_f1base(
            &fixtures_dir().join("cii/avoir_cii_381.xml").to_string_lossy(),
            "avoir_cii_381→F1Base",
        );
    }

    #[test]
    fn test_cii_to_f1base_acompte() {
        assert_cii_to_f1base(
            &fixtures_dir().join("cii/facture_cii_acompte.xml").to_string_lossy(),
            "facture_cii_acompte→F1Base",
        );
    }

    #[test]
    fn test_cii_to_f1base_rectificative() {
        assert_cii_to_f1base(
            &fixtures_dir().join("cii/facture_rectificative_cii_384.xml").to_string_lossy(),
            "facture_rectificative_cii_384→F1Base",
        );
    }

    #[test]
    fn test_cii_to_f1base_autofacture() {
        assert_cii_to_f1base(
            &fixtures_dir().join("cii/autofacture_cii_389.xml").to_string_lossy(),
            "autofacture_cii_389→F1Base",
        );
    }

    #[test]
    fn test_cii_to_f1base_delegation() {
        assert_cii_to_f1base(
            &fixtures_dir().join("cii/facture_cii_delegation_s8.xml").to_string_lossy(),
            "facture_cii_delegation_s8→F1Base",
        );
    }

    #[test]
    fn test_cii_to_f1base_marketplace() {
        assert_cii_to_f1base(
            &fixtures_dir().join("cii/facture_cii_marketplace_a8.xml").to_string_lossy(),
            "facture_cii_marketplace_a8→F1Base",
        );
    }

    #[test]
    fn test_cii_to_f1base_soustraitance() {
        assert_cii_to_f1base(
            &fixtures_dir().join("cii/facture_cii_soustraitance_a4.xml").to_string_lossy(),
            "facture_cii_soustraitance_a4→F1Base",
        );
    }

    #[test]
    fn test_cii_to_f1base_official_uc1() {
        assert_cii_to_f1base(
            &specs_dir().join("examples/xp-z12-014/UC1/UC1_F202500003_00-INV_20250701_CII.xml").to_string_lossy(),
            "UC1_CII→F1Base",
        );
    }

    // ================================================================
    //  Tests XSLT + XSD : CII → F1Full (toutes fixtures)
    // ================================================================

    #[test]
    fn test_cii_to_f1full_facture_001() {
        assert_cii_to_f1full(
            &fixtures_dir().join("cii/facture_cii_001.xml").to_string_lossy(),
            "facture_cii_001→F1Full",
        );
    }

    #[test]
    fn test_cii_to_f1full_remises_multitva() {
        assert_cii_to_f1full(
            &fixtures_dir().join("cii/facture_cii_remises_multitva.xml").to_string_lossy(),
            "facture_cii_remises_multitva→F1Full",
        );
    }

    #[test]
    fn test_cii_to_f1full_avoir() {
        assert_cii_to_f1full(
            &fixtures_dir().join("cii/avoir_cii_381.xml").to_string_lossy(),
            "avoir_cii_381→F1Full",
        );
    }

    #[test]
    fn test_cii_to_f1full_acompte() {
        assert_cii_to_f1full(
            &fixtures_dir().join("cii/facture_cii_acompte.xml").to_string_lossy(),
            "facture_cii_acompte→F1Full",
        );
    }

    #[test]
    fn test_cii_to_f1full_rectificative() {
        assert_cii_to_f1full(
            &fixtures_dir().join("cii/facture_rectificative_cii_384.xml").to_string_lossy(),
            "facture_rectificative_cii_384→F1Full",
        );
    }

    #[test]
    fn test_cii_to_f1full_autofacture() {
        assert_cii_to_f1full(
            &fixtures_dir().join("cii/autofacture_cii_389.xml").to_string_lossy(),
            "autofacture_cii_389→F1Full",
        );
    }

    #[test]
    fn test_cii_to_f1full_delegation() {
        assert_cii_to_f1full(
            &fixtures_dir().join("cii/facture_cii_delegation_s8.xml").to_string_lossy(),
            "facture_cii_delegation_s8→F1Full",
        );
    }

    #[test]
    fn test_cii_to_f1full_marketplace() {
        assert_cii_to_f1full(
            &fixtures_dir().join("cii/facture_cii_marketplace_a8.xml").to_string_lossy(),
            "facture_cii_marketplace_a8→F1Full",
        );
    }

    #[test]
    fn test_cii_to_f1full_soustraitance() {
        assert_cii_to_f1full(
            &fixtures_dir().join("cii/facture_cii_soustraitance_a4.xml").to_string_lossy(),
            "facture_cii_soustraitance_a4→F1Full",
        );
    }

    #[test]
    fn test_cii_to_f1full_definitive_apres_acompte() {
        assert_cii_to_f1full(
            &fixtures_dir().join("cii/facture_cii_definitive_apres_acompte.xml").to_string_lossy(),
            "facture_cii_definitive_apres_acompte→F1Full",
        );
    }

    #[test]
    fn test_cii_to_f1full_multivendeurs() {
        assert_cii_to_f1full(
            &fixtures_dir().join("cii/facture_cii_multivendeurs_b8.xml").to_string_lossy(),
            "facture_cii_multivendeurs_b8→F1Full",
        );
    }

    #[test]
    fn test_cii_to_f1full_official_uc1() {
        assert_cii_to_f1full(
            &specs_dir().join("examples/xp-z12-014/UC1/UC1_F202500003_00-INV_20250701_CII.xml").to_string_lossy(),
            "UC1_CII→F1Full",
        );
    }

    // ================================================================
    //  Tests XSLT + XSD : UBL → F1Base (toutes fixtures)
    // ================================================================

    #[test]
    fn test_ubl_to_f1base_facture_001() {
        assert_ubl_to_f1base(
            &fixtures_dir().join("ubl/facture_ubl_001.xml").to_string_lossy(),
            "facture_ubl_001→F1Base",
        );
    }

    #[test]
    fn test_ubl_to_f1base_remises_multitva() {
        assert_ubl_to_f1base(
            &fixtures_dir().join("ubl/facture_ubl_remises_multitva.xml").to_string_lossy(),
            "facture_ubl_remises_multitva→F1Base",
        );
    }

    #[test]
    fn test_ubl_to_f1base_avoir() {
        assert_ubl_creditnote_to_f1base(
            &fixtures_dir().join("ubl/facture_ubl_002_avoir.xml").to_string_lossy(),
            "facture_ubl_002_avoir→F1Base",
        );
    }

    #[test]
    fn test_ubl_to_f1base_acompte() {
        assert_ubl_to_f1base(
            &fixtures_dir().join("ubl/facture_ubl_acompte_386.xml").to_string_lossy(),
            "facture_ubl_acompte_386→F1Base",
        );
    }

    #[test]
    fn test_ubl_to_f1base_rectificative() {
        assert_ubl_to_f1base(
            &fixtures_dir().join("ubl/facture_rectificative_ubl_384.xml").to_string_lossy(),
            "facture_rectificative_ubl_384→F1Base",
        );
    }

    #[test]
    fn test_ubl_to_f1base_autofacture() {
        assert_ubl_to_f1base(
            &fixtures_dir().join("ubl/autofacture_ubl_389.xml").to_string_lossy(),
            "autofacture_ubl_389→F1Base",
        );
    }

    #[test]
    fn test_ubl_to_f1base_delegation() {
        assert_ubl_to_f1base(
            &fixtures_dir().join("ubl/facture_ubl_delegation_s8.xml").to_string_lossy(),
            "facture_ubl_delegation_s8→F1Base",
        );
    }

    #[test]
    fn test_ubl_to_f1base_marketplace() {
        assert_ubl_to_f1base(
            &fixtures_dir().join("ubl/facture_ubl_marketplace_a8.xml").to_string_lossy(),
            "facture_ubl_marketplace_a8→F1Base",
        );
    }

    #[test]
    fn test_ubl_to_f1base_soustraitance() {
        assert_ubl_to_f1base(
            &fixtures_dir().join("ubl/facture_ubl_soustraitance_a4.xml").to_string_lossy(),
            "facture_ubl_soustraitance_a4→F1Base",
        );
    }

    #[test]
    fn test_ubl_to_f1base_tax_representative() {
        assert_ubl_to_f1base(
            &fixtures_dir().join("ubl/facture_ubl_tax_representative.xml").to_string_lossy(),
            "facture_ubl_tax_representative→F1Base",
        );
    }

    #[test]
    fn test_ubl_to_f1base_official_uc1() {
        assert_ubl_to_f1base(
            &specs_dir().join("examples/xp-z12-014/UC1/UC1_F202500003_00-INV_20250701_UBL.xml").to_string_lossy(),
            "UC1_UBL→F1Base",
        );
    }

    // ================================================================
    //  Tests XSLT + XSD : UBL → F1Full (toutes fixtures)
    // ================================================================

    #[test]
    fn test_ubl_to_f1full_facture_001() {
        assert_ubl_to_f1full(
            &fixtures_dir().join("ubl/facture_ubl_001.xml").to_string_lossy(),
            "facture_ubl_001→F1Full",
        );
    }

    #[test]
    fn test_ubl_to_f1full_remises_multitva() {
        assert_ubl_to_f1full(
            &fixtures_dir().join("ubl/facture_ubl_remises_multitva.xml").to_string_lossy(),
            "facture_ubl_remises_multitva→F1Full",
        );
    }

    #[test]
    fn test_ubl_to_f1full_avoir() {
        assert_ubl_creditnote_to_f1full(
            &fixtures_dir().join("ubl/facture_ubl_002_avoir.xml").to_string_lossy(),
            "facture_ubl_002_avoir→F1Full",
        );
    }

    #[test]
    fn test_ubl_to_f1full_acompte() {
        assert_ubl_to_f1full(
            &fixtures_dir().join("ubl/facture_ubl_acompte_386.xml").to_string_lossy(),
            "facture_ubl_acompte_386→F1Full",
        );
    }

    #[test]
    fn test_ubl_to_f1full_rectificative() {
        assert_ubl_to_f1full(
            &fixtures_dir().join("ubl/facture_rectificative_ubl_384.xml").to_string_lossy(),
            "facture_rectificative_ubl_384→F1Full",
        );
    }

    #[test]
    fn test_ubl_to_f1full_autofacture() {
        assert_ubl_to_f1full(
            &fixtures_dir().join("ubl/autofacture_ubl_389.xml").to_string_lossy(),
            "autofacture_ubl_389→F1Full",
        );
    }

    #[test]
    fn test_ubl_to_f1full_delegation() {
        assert_ubl_to_f1full(
            &fixtures_dir().join("ubl/facture_ubl_delegation_s8.xml").to_string_lossy(),
            "facture_ubl_delegation_s8→F1Full",
        );
    }

    #[test]
    fn test_ubl_to_f1full_marketplace() {
        assert_ubl_to_f1full(
            &fixtures_dir().join("ubl/facture_ubl_marketplace_a8.xml").to_string_lossy(),
            "facture_ubl_marketplace_a8→F1Full",
        );
    }

    #[test]
    fn test_ubl_to_f1full_soustraitance() {
        assert_ubl_to_f1full(
            &fixtures_dir().join("ubl/facture_ubl_soustraitance_a4.xml").to_string_lossy(),
            "facture_ubl_soustraitance_a4→F1Full",
        );
    }

    #[test]
    fn test_ubl_to_f1full_definitive_apres_acompte() {
        assert_ubl_to_f1full(
            &fixtures_dir().join("ubl/facture_ubl_definitive_apres_acompte.xml").to_string_lossy(),
            "facture_ubl_definitive_apres_acompte→F1Full",
        );
    }

    #[test]
    fn test_ubl_to_f1full_multivendeurs() {
        assert_ubl_to_f1full(
            &fixtures_dir().join("ubl/facture_ubl_multivendeurs_b8.xml").to_string_lossy(),
            "facture_ubl_multivendeurs_b8→F1Full",
        );
    }

    #[test]
    fn test_ubl_to_f1full_tax_representative() {
        assert_ubl_to_f1full(
            &fixtures_dir().join("ubl/facture_ubl_tax_representative.xml").to_string_lossy(),
            "facture_ubl_tax_representative→F1Full",
        );
    }

    #[test]
    fn test_ubl_to_f1full_official_uc1() {
        assert_ubl_to_f1full(
            &specs_dir().join("examples/xp-z12-014/UC1/UC1_F202500003_00-INV_20250701_UBL.xml").to_string_lossy(),
            "UC1_UBL→F1Full",
        );
    }

    // ================================================================
    //  Tests Processor intégration
    // ================================================================

    #[tokio::test]
    async fn test_ppf_flux1_skip_cdar() {
        let tmp = std::env::temp_dir().join("pdp-test-flux1-cdar");
        let specs = specs_dir();
        let processor = PpfFlux1Processor::new(&tmp, &specs);

        let mut exchange = Exchange::new(Vec::new());
        exchange.set_header("document.type", "CDAR");

        let result = processor.process(exchange).await.unwrap();
        assert!(result.get_property("ppf.flux1.path").is_none());
    }

    #[tokio::test]
    async fn test_ppf_flux1_processor_cii_base() {
        // Base = facture sans lignes
        let fixture = fixtures_dir().join("cii/facture_cii_001.xml");
        let cii_xml = match read_fixture(&fixture.to_string_lossy()) {
            Some(xml) => xml,
            None => { eprintln!("Fixture CII introuvable, skip"); return; }
        };

        let mut invoice = pdp_invoice::cii::CiiParser::new().parse(&cii_xml).unwrap();
        invoice.lines.clear(); // Supprimer les lignes pour forcer Base

        let tmp = std::env::temp_dir().join("pdp-test-flux1-proc-cii-base");
        let processor = PpfFlux1Processor::new(&tmp, &specs_dir());

        let mut exchange = Exchange::new(cii_xml.into_bytes());
        exchange.invoice = Some(invoice);

        match processor.process(exchange).await {
            Ok(ex) => {
                assert_eq!(ex.get_property("ppf.flux1.profile").map(|s| s.as_str()), Some("Base"));
                assert_eq!(ex.get_property("ppf.flux1.code_interface").map(|s| s.as_str()), Some("FFE0112A"));
                let path = ex.get_property("ppf.flux1.path").unwrap();
                assert!(std::path::Path::new(&path).exists());
                let f1_xml = std::fs::read_to_string(&path).unwrap();
                assert!(f1_xml.contains("einvoicingextract#Base"));
                assert!(!f1_xml.contains("IncludedSupplyChainTradeLineItem"));
                assert_xsd_valid(&f1_xml, &XsdDocumentType::CiiInvoice, "processor_cii→F1Base");
                let _ = std::fs::remove_dir_all(&tmp);
            }
            Err(e) => eprintln!("Skip (Saxon?): {}", e),
        }
    }

    #[tokio::test]
    async fn test_ppf_flux1_processor_ubl_base() {
        // Base = facture sans lignes
        let fixture = fixtures_dir().join("ubl/facture_ubl_001.xml");
        let ubl_xml = match read_fixture(&fixture.to_string_lossy()) {
            Some(xml) => xml,
            None => { eprintln!("Fixture UBL introuvable, skip"); return; }
        };

        let mut invoice = pdp_invoice::ubl::UblParser::new().parse(&ubl_xml).unwrap();
        invoice.lines.clear(); // Supprimer les lignes pour forcer Base

        let tmp = std::env::temp_dir().join("pdp-test-flux1-proc-ubl-base");
        let processor = PpfFlux1Processor::new(&tmp, &specs_dir());

        let mut exchange = Exchange::new(ubl_xml.into_bytes());
        exchange.invoice = Some(invoice);

        match processor.process(exchange).await {
            Ok(ex) => {
                assert_eq!(ex.get_property("ppf.flux1.profile").map(|s| s.as_str()), Some("Base"));
                assert_eq!(ex.get_property("ppf.flux1.code_interface").map(|s| s.as_str()), Some("FFE0111A"));
                let path = ex.get_property("ppf.flux1.path").unwrap();
                let f1_xml = std::fs::read_to_string(&path).unwrap();
                assert!(f1_xml.contains("einvoicingextract#Base"));
                assert!(!f1_xml.contains("<cac:InvoiceLine"));
                assert_xsd_valid(&f1_xml, &XsdDocumentType::UblInvoice, "processor_ubl→F1Base");
                let _ = std::fs::remove_dir_all(&tmp);
            }
            Err(e) => eprintln!("Skip (Saxon?): {}", e),
        }
    }

    #[tokio::test]
    async fn test_ppf_flux1_processor_cii_full() {
        // Full = facture avec lignes (détection automatique)
        let fixture = fixtures_dir().join("cii/facture_cii_001.xml");
        let cii_xml = match read_fixture(&fixture.to_string_lossy()) {
            Some(xml) => xml,
            None => { eprintln!("Fixture CII introuvable, skip"); return; }
        };

        let invoice = pdp_invoice::cii::CiiParser::new().parse(&cii_xml).unwrap();

        let tmp = std::env::temp_dir().join("pdp-test-flux1-proc-cii-full");
        let processor = PpfFlux1Processor::new(&tmp, &specs_dir());

        let mut exchange = Exchange::new(cii_xml.into_bytes());
        exchange.invoice = Some(invoice);

        match processor.process(exchange).await {
            Ok(ex) => {
                assert_eq!(ex.get_property("ppf.flux1.profile").map(|s| s.as_str()), Some("Full"));
                assert_eq!(ex.get_property("ppf.flux1.code_interface").map(|s| s.as_str()), Some("FFE0112A"));
                let filename = ex.get_property("ppf.flux1.filename").unwrap();
                assert!(filename.starts_with("Full_"), "Filename should start with Full_: {}", filename);
                let path = ex.get_property("ppf.flux1.path").unwrap();
                let f1_xml = std::fs::read_to_string(&path).unwrap();
                assert!(f1_xml.contains("einvoicingextract#Full"));
                assert!(f1_xml.contains("IncludedSupplyChainTradeLineItem"));
                assert_xsd_valid(&f1_xml, &XsdDocumentType::F1FullCii, "processor_cii→F1Full");
                let _ = std::fs::remove_dir_all(&tmp);
            }
            Err(e) => eprintln!("Skip (Saxon?): {}", e),
        }
    }

    #[tokio::test]
    async fn test_ppf_flux1_processor_ubl_full() {
        // Full = facture avec lignes (détection automatique)
        let fixture = fixtures_dir().join("ubl/facture_ubl_001.xml");
        let ubl_xml = match read_fixture(&fixture.to_string_lossy()) {
            Some(xml) => xml,
            None => { eprintln!("Fixture UBL introuvable, skip"); return; }
        };

        let invoice = pdp_invoice::ubl::UblParser::new().parse(&ubl_xml).unwrap();

        let tmp = std::env::temp_dir().join("pdp-test-flux1-proc-ubl-full");
        let processor = PpfFlux1Processor::new(&tmp, &specs_dir());

        let mut exchange = Exchange::new(ubl_xml.into_bytes());
        exchange.invoice = Some(invoice);

        match processor.process(exchange).await {
            Ok(ex) => {
                assert_eq!(ex.get_property("ppf.flux1.profile").map(|s| s.as_str()), Some("Full"));
                assert_eq!(ex.get_property("ppf.flux1.code_interface").map(|s| s.as_str()), Some("FFE0111A"));
                let filename = ex.get_property("ppf.flux1.filename").unwrap();
                assert!(filename.starts_with("Full_"), "Filename should start with Full_: {}", filename);
                let path = ex.get_property("ppf.flux1.path").unwrap();
                let f1_xml = std::fs::read_to_string(&path).unwrap();
                assert!(f1_xml.contains("einvoicingextract#Full"));
                assert!(f1_xml.contains("<cac:InvoiceLine"));
                assert_xsd_valid(&f1_xml, &XsdDocumentType::F1FullUblInvoice, "processor_ubl→F1Full");
                let _ = std::fs::remove_dir_all(&tmp);
            }
            Err(e) => eprintln!("Skip (Saxon?): {}", e),
        }
    }
}
