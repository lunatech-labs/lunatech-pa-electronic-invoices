//! Moteur de génération PDF via la pipeline Mustang : CII/UBL → XR → XSL-FO → PDF.
//!
//! Pipeline en 3 étapes :
//! 1. CII/UBL XML → XR intermediate XML (via XSLT 2.0 + cii-xr.xsl ou ubl-invoice-xr.xsl)
//! 2. XR XML → XSL-FO (via XSLT 2.0 + xr-pdf.xsl, param foengine=fop, lang=fr)
//! 3. XSL-FO → PDF (via Apache FOP + fop-config.xconf)
//!
//! Le moteur XSLT supporte deux backends :
//! - **SaxonC-HE** (natif C++, pas de JVM) : binaire `transform` — préféré
//! - **SaxonJ-HE** (Java) : binaire `saxon` via Homebrew — fallback
//!
//! La détection est automatique : SaxonC est préféré s'il est disponible.
//!
//! Inspiré du projet Mustang (ZUGFeRD/mustangproject).

use std::path::{Path, PathBuf};
use std::process::Command;

use pdp_core::error::{PdpError, PdpResult};

/// Format source pour la transformation vers PDF
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceSyntax {
    /// UN/CEFACT CII D22B
    CII,
    /// OASIS UBL 2.1 Invoice
    UBL,
    /// OASIS UBL 2.1 CreditNote
    UBLCreditNote,
}

impl std::fmt::Display for SourceSyntax {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SourceSyntax::CII => write!(f, "CII"),
            SourceSyntax::UBL => write!(f, "UBL"),
            SourceSyntax::UBLCreditNote => write!(f, "UBL CreditNote"),
        }
    }
}

/// Backend XSLT utilisé pour les transformations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XsltBackend {
    /// SaxonC-HE natif (C++/GraalVM, pas de JVM) — binaire `transform`
    SaxonC,
    /// SaxonJ-HE via Java — binaire `saxon` (Homebrew)
    SaxonJ,
}

impl XsltBackend {
    /// Nom du binaire CLI
    fn command(&self) -> &'static str {
        match self {
            XsltBackend::SaxonC => "transform",
            XsltBackend::SaxonJ => "saxon",
        }
    }
}

impl std::fmt::Display for XsltBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            XsltBackend::SaxonC => write!(f, "SaxonC-HE (natif)"),
            XsltBackend::SaxonJ => write!(f, "SaxonJ-HE (Java)"),
        }
    }
}

/// Détecte le backend XSLT disponible.
/// Préfère SaxonC (natif, pas de JVM) puis SaxonJ (Java).
pub fn detect_xslt_backend() -> Option<XsltBackend> {
    // Préférer SaxonC (natif)
    if Command::new("transform")
        .arg("-?")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok()
    {
        return Some(XsltBackend::SaxonC);
    }
    // Fallback SaxonJ (Java)
    if Command::new("saxon")
        .arg("-?")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok()
    {
        return Some(XsltBackend::SaxonJ);
    }
    None
}

/// Moteur FOP pour la génération de PDF à partir de factures CII/UBL.
/// Utilise un processeur XSLT 2.0 (SaxonC natif ou SaxonJ Java) et fop-rs (Rust natif) pour le rendu PDF.
pub struct FopEngine {
    /// Répertoire racine des specs (contient xslt/mustang/)
    specs_dir: PathBuf,
    /// Langue pour le rendu PDF (défaut: "fr")
    lang: String,
    /// Backend XSLT détecté
    xslt_backend: XsltBackend,
    /// Chemin vers le fop-config résolu (pré-calculé au new()) — utilisé uniquement avec le fallback Java
    #[cfg(not(feature = "fop_native"))]
    resolved_fop_config: Option<PathBuf>,
    /// Processeur SaxonC in-process (FFI, pas de fork/exec) — optionnel
    #[cfg(feature = "saxonc_ffi")]
    saxonc: Option<crate::saxonc_ffi::SaxonCProcessor>,
}

impl FopEngine {
    pub fn new(specs_dir: &Path) -> Self {
        let backend = detect_xslt_backend().unwrap_or(XsltBackend::SaxonJ);
        tracing::info!(backend = %backend, "Moteur XSLT détecté");

        #[cfg(not(feature = "fop_native"))]
        let resolved = Self::resolve_fop_config_static(specs_dir);

        #[cfg(feature = "fop_native")]
        tracing::info!("FOP natif Rust activé (pas de subprocess Java)");

        #[cfg(feature = "saxonc_ffi")]
        let saxonc = {
            let proc = crate::saxonc_ffi::SaxonCProcessor::new();
            if proc.is_some() {
                tracing::info!("SaxonC FFI: in-process XSLT activé (pas de fork/exec)");
            }
            proc
        };

        Self {
            specs_dir: specs_dir.to_path_buf(),
            lang: "fr".to_string(),
            xslt_backend: backend,
            #[cfg(not(feature = "fop_native"))]
            resolved_fop_config: resolved,
            #[cfg(feature = "saxonc_ffi")]
            saxonc,
        }
    }

    /// Construit le moteur avec un backend XSLT explicite.
    pub fn with_backend(specs_dir: &Path, backend: XsltBackend) -> Self {
        #[cfg(not(feature = "fop_native"))]
        let resolved = Self::resolve_fop_config_static(specs_dir);

        #[cfg(feature = "saxonc_ffi")]
        let saxonc = crate::saxonc_ffi::SaxonCProcessor::new();

        Self {
            specs_dir: specs_dir.to_path_buf(),
            lang: "fr".to_string(),
            xslt_backend: backend,
            #[cfg(not(feature = "fop_native"))]
            resolved_fop_config: resolved,
            #[cfg(feature = "saxonc_ffi")]
            saxonc,
        }
    }

    /// Construit le moteur en déduisant le chemin specs depuis CARGO_MANIFEST_DIR
    pub fn from_manifest_dir() -> Self {
        let specs = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../specs");
        Self::new(&specs)
    }

    pub fn with_lang(mut self, lang: &str) -> Self {
        self.lang = lang.to_string();
        self
    }

    /// Retourne le backend XSLT utilisé.
    pub fn xslt_backend(&self) -> XsltBackend {
        self.xslt_backend
    }

    // --- Chemins des ressources ---

    fn mustang_dir(&self) -> PathBuf {
        self.specs_dir.join("xslt/mustang/stylesheets")
    }

    fn cii_xr_xsl(&self) -> PathBuf {
        self.mustang_dir().join("cii-xr.xsl")
    }

    fn ubl_xr_xsl(&self) -> PathBuf {
        self.mustang_dir().join("ubl-invoice-xr.xsl")
    }

    fn ubl_creditnote_xr_xsl(&self) -> PathBuf {
        self.mustang_dir().join("ubl-creditnote-xr.xsl")
    }

    fn xr_pdf_xsl(&self) -> PathBuf {
        self.mustang_dir().join("xr-pdf.xsl")
    }

    // --- Pipeline publique ---

    /// Génère un PDF à partir d'un XML CII ou UBL.
    /// Pipeline optimisé : XML→XR→FO avec fichiers temporaires réutilisés, puis FO→PDF via FOP.
    pub fn generate_pdf(&self, xml: &str, syntax: SourceSyntax) -> PdpResult<Vec<u8>> {
        // Pipeline optimisé : 1 écriture XML, 2 appels Saxon chaînés via fichiers, 1 appel FOP
        let fo_xml = self.transform_to_fo(xml, syntax)?;

        // Étape 3 : XSL-FO → PDF via FOP
        let pdf = self.render_fo_to_pdf(&fo_xml)?;

        tracing::info!(
            syntax = %syntax,
            pdf_size = pdf.len(),
            xslt = %self.xslt_backend,
            "PDF généré via pipeline Mustang (XSLT+FOP)"
        );

        Ok(pdf)
    }

    /// Pipeline XSLT via SaxonC FFI in-process (pas de fork/exec).
    #[cfg(feature = "saxonc_ffi")]
    fn transform_to_fo_ffi(
        &self,
        saxonc: &crate::saxonc_ffi::SaxonCProcessor,
        xml: &str,
        syntax: SourceSyntax,
    ) -> PdpResult<String> {
        let xr_xsl = match syntax {
            SourceSyntax::CII => self.cii_xr_xsl(),
            SourceSyntax::UBL => self.ubl_xr_xsl(),
            SourceSyntax::UBLCreditNote => self.ubl_creditnote_xr_xsl(),
        };
        let fo_xsl = self.xr_pdf_xsl();

        if !xr_xsl.exists() {
            return Err(PdpError::TransformError {
                source_format: syntax.to_string(),
                target_format: "XR".to_string(),
                message: format!("XSLT {} introuvable: {}", syntax, xr_xsl.display()),
            });
        }
        if !fo_xsl.exists() {
            return Err(PdpError::TransformError {
                source_format: "XR".to_string(),
                target_format: "FO".to_string(),
                message: format!("XSLT xr-pdf.xsl introuvable: {}", fo_xsl.display()),
            });
        }

        // Écrire le XML source dans un fichier temporaire (SaxonC FFI lit depuis un fichier)
        let tmp_dir = std::env::temp_dir();
        let id = uuid::Uuid::new_v4();
        let tmp_xml = tmp_dir.join(format!("pdp-ffi-in-{}.xml", id));
        let tmp_xr = tmp_dir.join(format!("pdp-ffi-xr-{}.xml", id));

        std::fs::write(&tmp_xml, xml).map_err(|e| PdpError::TransformError {
            source_format: syntax.to_string(),
            target_format: "FO".to_string(),
            message: format!("Impossible d'écrire le fichier temporaire: {}", e),
        })?;

        // Étape 1 : XML → XR (SaxonC FFI in-process)
        let xr_result = saxonc.transform_file_to_string(&tmp_xml, &xr_xsl, &[]);
        let _ = std::fs::remove_file(&tmp_xml);

        let xr_xml = xr_result.map_err(|e| PdpError::TransformError {
            source_format: syntax.to_string(),
            target_format: "XR".to_string(),
            message: format!("SaxonC FFI {}→XR échoué: {}", syntax, e),
        })?;

        // Écrire le XR pour l'étape 2
        std::fs::write(&tmp_xr, &xr_xml).map_err(|e| PdpError::TransformError {
            source_format: "XR".to_string(),
            target_format: "FO".to_string(),
            message: format!("Impossible d'écrire le XR temporaire: {}", e),
        })?;

        // Étape 2 : XR → FO (SaxonC FFI in-process)
        let fo_result = saxonc.transform_file_to_string(
            &tmp_xr, &fo_xsl,
            &[("foengine", "fop"), ("lang", &self.lang)],
        );
        let _ = std::fs::remove_file(&tmp_xr);

        let fo_xml = fo_result.map_err(|e| PdpError::TransformError {
            source_format: "XR".to_string(),
            target_format: "FO".to_string(),
            message: format!("SaxonC FFI XR→FO échoué: {}", e),
        })?;

        if fo_xml.trim().is_empty() {
            return Err(PdpError::TransformError {
                source_format: syntax.to_string(),
                target_format: "FO".to_string(),
                message: "Le résultat FO est vide (SaxonC FFI)".to_string(),
            });
        }

        tracing::debug!(
            syntax = %syntax,
            fo_len = fo_xml.len(),
            "Pipeline XSLT {}→XR→FO terminé (SaxonC FFI)", syntax
        );

        Ok(fo_xml)
    }

    /// Pipeline XSLT optimisé : XML → XR → FO en 2 appels Saxon chaînés via fichiers.
    /// Quand SaxonC FFI est disponible, utilise l'API in-process (pas de fork/exec).
    fn transform_to_fo(&self, xml: &str, syntax: SourceSyntax) -> PdpResult<String> {
        // Chemin rapide : SaxonC FFI in-process
        #[cfg(feature = "saxonc_ffi")]
        if let Some(ref saxonc) = self.saxonc {
            return self.transform_to_fo_ffi(saxonc, xml, syntax);
        }
        let xr_xsl = match syntax {
            SourceSyntax::CII => self.cii_xr_xsl(),
            SourceSyntax::UBL => self.ubl_xr_xsl(),
            SourceSyntax::UBLCreditNote => self.ubl_creditnote_xr_xsl(),
        };
        let fo_xsl = self.xr_pdf_xsl();

        if !xr_xsl.exists() {
            return Err(PdpError::TransformError {
                source_format: syntax.to_string(),
                target_format: "XR".to_string(),
                message: format!("XSLT {} introuvable: {}", syntax, xr_xsl.display()),
            });
        }
        if !fo_xsl.exists() {
            return Err(PdpError::TransformError {
                source_format: "XR".to_string(),
                target_format: "FO".to_string(),
                message: format!("XSLT xr-pdf.xsl introuvable: {}", fo_xsl.display()),
            });
        }

        let tmp_dir = std::env::temp_dir();
        let id = uuid::Uuid::new_v4();
        let tmp_xml = tmp_dir.join(format!("pdp-pipe-in-{}.xml", id));
        let tmp_xr = tmp_dir.join(format!("pdp-pipe-xr-{}.xml", id));
        let tmp_fo = tmp_dir.join(format!("pdp-pipe-fo-{}.xml", id));

        // Écrire le XML source une seule fois
        std::fs::write(&tmp_xml, xml).map_err(|e| PdpError::TransformError {
            source_format: syntax.to_string(),
            target_format: "FO".to_string(),
            message: format!("Impossible d'écrire le fichier temporaire: {}", e),
        })?;

        // Étape 1 : XML → XR (Saxon, fichier → fichier)
        let xr_status = Command::new(self.xslt_backend.command())
            .arg(format!("-s:{}", tmp_xml.to_str().unwrap_or("")))
            .arg(format!("-xsl:{}", xr_xsl.to_str().unwrap_or("")))
            .arg(format!("-o:{}", tmp_xr.to_str().unwrap_or("")))
            .output();

        let _ = std::fs::remove_file(&tmp_xml);

        let xr_output = xr_status.map_err(|e| PdpError::TransformError {
            source_format: syntax.to_string(),
            target_format: "XR".to_string(),
            message: format!("Impossible d'exécuter {} pour {}→XR: {}", self.xslt_backend, syntax, e),
        })?;

        if !tmp_xr.exists() {
            let stderr = String::from_utf8_lossy(&xr_output.stderr);
            return Err(PdpError::TransformError {
                source_format: syntax.to_string(),
                target_format: "XR".to_string(),
                message: format!("Saxon {}→XR n'a pas produit de sortie: {}", syntax, stderr.trim()),
            });
        }

        // Étape 2 : XR → FO (Saxon, fichier → fichier, pas de lecture en mémoire Rust)
        let fo_status = Command::new(self.xslt_backend.command())
            .arg(format!("-s:{}", tmp_xr.to_str().unwrap_or("")))
            .arg(format!("-xsl:{}", fo_xsl.to_str().unwrap_or("")))
            .arg(format!("-o:{}", tmp_fo.to_str().unwrap_or("")))
            .arg("foengine=fop")
            .arg(format!("lang={}", self.lang))
            .output();

        let _ = std::fs::remove_file(&tmp_xr);

        let fo_output = fo_status.map_err(|e| PdpError::TransformError {
            source_format: "XR".to_string(),
            target_format: "FO".to_string(),
            message: format!("Impossible d'exécuter {} pour XR→FO: {}", self.xslt_backend, e),
        })?;

        // Lire le FO résultat
        let fo_xml = if tmp_fo.exists() {
            let content = std::fs::read_to_string(&tmp_fo).map_err(|e| PdpError::TransformError {
                source_format: "XR".to_string(),
                target_format: "FO".to_string(),
                message: format!("Impossible de lire le résultat FO: {}", e),
            })?;
            let _ = std::fs::remove_file(&tmp_fo);
            content
        } else {
            let stderr = String::from_utf8_lossy(&fo_output.stderr);
            return Err(PdpError::TransformError {
                source_format: "XR".to_string(),
                target_format: "FO".to_string(),
                message: format!("Saxon XR→FO n'a pas produit de sortie: {}", stderr.trim()),
            });
        };

        if fo_xml.trim().is_empty() {
            return Err(PdpError::TransformError {
                source_format: syntax.to_string(),
                target_format: "FO".to_string(),
                message: "Le résultat FO est vide".to_string(),
            });
        }

        tracing::debug!(
            syntax = %syntax,
            fo_len = fo_xml.len(),
            "Pipeline XSLT {}→XR→FO terminé", syntax
        );

        Ok(fo_xml)
    }

    /// Génère un PDF à partir d'un XML CII.
    pub fn cii_to_pdf(&self, cii_xml: &str) -> PdpResult<Vec<u8>> {
        self.generate_pdf(cii_xml, SourceSyntax::CII)
    }

    /// Génère un PDF à partir d'un XML UBL.
    pub fn ubl_to_pdf(&self, ubl_xml: &str) -> PdpResult<Vec<u8>> {
        self.generate_pdf(ubl_xml, SourceSyntax::UBL)
    }

    /// Étape 1 : CII/UBL XML → XR intermediate XML via Saxon.
    pub fn transform_to_xr(&self, xml: &str, syntax: SourceSyntax) -> PdpResult<String> {
        let xsl_path = match syntax {
            SourceSyntax::CII => self.cii_xr_xsl(),
            SourceSyntax::UBL => self.ubl_xr_xsl(),
            SourceSyntax::UBLCreditNote => self.ubl_creditnote_xr_xsl(),
        };

        if !xsl_path.exists() {
            return Err(PdpError::TransformError {
                source_format: syntax.to_string(),
                target_format: "XR".to_string(),
                message: format!("XSLT {} introuvable: {}", syntax, xsl_path.display()),
            });
        }

        self.run_saxon(xml, &xsl_path, &[], &format!("{}→XR", syntax))
    }

    /// Étape 2 : XR XML → XSL-FO via Saxon + xr-pdf.xsl.
    pub fn transform_xr_to_fo(&self, xr_xml: &str) -> PdpResult<String> {
        let xsl_path = self.xr_pdf_xsl();
        if !xsl_path.exists() {
            return Err(PdpError::TransformError {
                source_format: "XR".to_string(),
                target_format: "FO".to_string(),
                message: format!("XSLT xr-pdf.xsl introuvable: {}", xsl_path.display()),
            });
        }

        let params = vec![
            ("foengine", "fop"),
            ("lang", &self.lang),
        ];

        self.run_saxon(xr_xml, &xsl_path, &params, "XR→FO")
    }

    /// Résout le fichier fop-config en remplaçant {SPECS_DIR} par le chemin absolu.
    /// Utilisé uniquement avec le fallback Java (sans feature fop_native).
    #[cfg(not(feature = "fop_native"))]
    fn resolve_fop_config_static(specs_dir: &Path) -> Option<PathBuf> {
        let fop_config = specs_dir.join("fop/fop-facturx.xconf");
        if !fop_config.exists() {
            return None;
        }

        let content = match std::fs::read_to_string(&fop_config) {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!("Impossible de lire fop-config: {}", e);
                return None;
            }
        };

        let specs_abs = specs_dir.canonicalize().unwrap_or_else(|_| specs_dir.to_path_buf());
        let resolved = content.replace("{SPECS_DIR}", specs_abs.to_str().unwrap_or(""));

        // Fichier stable (pas UUID) pour réutilisation
        let tmp_config = std::env::temp_dir().join("pdp-fop-resolved.xconf");
        if let Err(e) = std::fs::write(&tmp_config, &resolved) {
            tracing::warn!("Impossible d'écrire fop-config résolu: {}", e);
            return None;
        }

        Some(tmp_config)
    }

    /// Construit la FontConfig avec les polices SourceSansPro et SourceSerifPro embarquées.
    #[cfg(feature = "fop_native")]
    fn build_font_config(&self) -> fop_render::FontConfig {
        let fonts_dir = self.specs_dir.join("xslt/mustang/fonts");
        let mut font_config = fop_render::FontConfig::new();

        // SourceSansPro (sans-serif)
        let sans_mappings = [
            ("SourceSansPro", "SourceSansPro-Regular.ttf"),
            ("SourceSansPro-Bold", "SourceSansPro-Bold.ttf"),
            ("SourceSansPro-Italic", "SourceSansPro-It.ttf"),
            ("SourceSansPro-BoldItalic", "SourceSansPro-BoldIt.ttf"),
        ];

        // SourceSerifPro (serif)
        let serif_mappings = [
            ("SourceSerifPro", "SourceSerifPro-Regular.ttf"),
            ("SourceSerifPro-Bold", "SourceSerifPro-Bold.ttf"),
            ("SourceSerifPro-Italic", "SourceSerifPro-It.ttf"),
            ("SourceSerifPro-BoldItalic", "SourceSerifPro-BoldIt.ttf"),
        ];

        for (name, file) in sans_mappings.iter().chain(serif_mappings.iter()) {
            let path = fonts_dir.join(file);
            if path.exists() {
                font_config.add_mapping(name, path);
            } else {
                tracing::warn!(font = %file, "Police introuvable: {}", path.display());
            }
        }

        font_config
    }

    /// Corrige les propriétés XSL-FO incompatibles avec fop-rs.
    /// Les stylesheets Mustang produisent `line-height="0pt"` que Java FOP tolère
    /// mais que fop-rs rejette (validation stricte).
    #[cfg(feature = "fop_native")]
    fn sanitize_fo(fo_xml: &str) -> String {
        fo_xml
            .replace(r#"line-height="0pt""#, r#"line-height="normal""#)
            .replace(r#"line-height="0mm""#, r#"line-height="normal""#)
            .replace(r#"line-height="0""#, r#"line-height="normal""#)
    }

    /// Étape 3 : XSL-FO → PDF via fop-rs (Rust natif, in-process).
    #[cfg(feature = "fop_native")]
    pub fn render_fo_to_pdf(&self, fo_xml: &str) -> PdpResult<Vec<u8>> {
        use std::io::Cursor;

        tracing::debug!(fo_len = fo_xml.len(), "Rendu FO→PDF via fop-rs natif");

        // Sanitiser le FO pour compatibilité fop-rs (line-height="0pt" → "normal")
        let fo_xml = Self::sanitize_fo(fo_xml);

        // Étape 1 : Parser le XSL-FO en arbre FO
        let fo_tree = fop_core::FoTreeBuilder::new()
            .parse(Cursor::new(fo_xml.as_str()))
            .map_err(|e| PdpError::TransformError {
                source_format: "FO".to_string(),
                target_format: "PDF".to_string(),
                message: format!("fop-rs: parsing XSL-FO échoué: {}", e),
            })?;

        // Étape 2 : Layout (arbre FO → arbre de zones/pages)
        let area_tree = fop_layout::LayoutEngine::new()
            .layout(&fo_tree)
            .map_err(|e| PdpError::TransformError {
                source_format: "FO".to_string(),
                target_format: "PDF".to_string(),
                message: format!("fop-rs: layout échoué: {}", e),
            })?;

        // Étape 3 : Rendu PDF avec polices configurées (render_with_fo pour bookmarks)
        let font_config = self.build_font_config();
        let pdf_doc = fop_render::PdfRenderer::with_system_fonts()
            .with_font_config(font_config)
            .render_with_fo(&area_tree, &fo_tree)
            .map_err(|e| PdpError::TransformError {
                source_format: "FO".to_string(),
                target_format: "PDF".to_string(),
                message: format!("fop-rs: rendu PDF échoué: {}", e),
            })?;

        let pdf = pdf_doc.to_bytes().map_err(|e| PdpError::TransformError {
            source_format: "FO".to_string(),
            target_format: "PDF".to_string(),
            message: format!("fop-rs: sérialisation PDF échouée: {}", e),
        })?;

        if pdf.len() < 5 || &pdf[0..5] != b"%PDF-" {
            return Err(PdpError::TransformError {
                source_format: "FO".to_string(),
                target_format: "PDF".to_string(),
                message: "Le fichier généré par fop-rs n'est pas un PDF valide".to_string(),
            });
        }

        tracing::debug!(pdf_size = pdf.len(), "PDF généré via fop-rs natif");
        Ok(pdf)
    }

    /// Étape 3 : XSL-FO → PDF via Apache FOP (subprocess Java) — fallback.
    #[cfg(not(feature = "fop_native"))]
    pub fn render_fo_to_pdf(&self, fo_xml: &str) -> PdpResult<Vec<u8>> {
        let tmp_dir = std::env::temp_dir();
        let id = uuid::Uuid::new_v4();
        let tmp_fo = tmp_dir.join(format!("pdp-fo-{}.fo", id));
        let tmp_pdf = tmp_dir.join(format!("pdp-pdf-{}.pdf", id));

        // Écrire le FO dans un fichier temporaire
        std::fs::write(&tmp_fo, fo_xml).map_err(|e| PdpError::TransformError {
            source_format: "FO".to_string(),
            target_format: "PDF".to_string(),
            message: format!("Impossible d'écrire le fichier FO temporaire: {}", e),
        })?;

        // Construire la commande FOP (config pré-résolue au new())
        let mut cmd = Command::new("fop");
        if let Some(ref config_path) = self.resolved_fop_config {
            cmd.arg("-c").arg(config_path.to_str().unwrap_or(""));
        }
        cmd.arg("-fo").arg(tmp_fo.to_str().unwrap_or(""))
           .arg("-pdf").arg(tmp_pdf.to_str().unwrap_or(""));

        tracing::debug!(
            fo = %tmp_fo.display(),
            pdf = %tmp_pdf.display(),
            "Exécution de FOP Java"
        );

        let output = cmd.output().map_err(|e| PdpError::TransformError {
            source_format: "FO".to_string(),
            target_format: "PDF".to_string(),
            message: format!(
                "Impossible d'exécuter Apache FOP: {}. \
                 Vérifiez que FOP est installé (brew install fop ou https://xmlgraphics.apache.org/fop/).",
                e
            ),
        })?;

        // Nettoyer le fichier FO temporaire (config est pré-résolue, on ne la supprime pas)
        let _ = std::fs::remove_file(&tmp_fo);

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // FOP émet souvent des warnings sur stderr même en cas de succès
            // On vérifie si le PDF a été généré
            if !tmp_pdf.exists() {
                let _ = std::fs::remove_file(&tmp_pdf);
                return Err(PdpError::TransformError {
                    source_format: "FO".to_string(),
                    target_format: "PDF".to_string(),
                    message: format!("FOP a échoué (exit code {:?}): {}", output.status.code(), stderr.trim()),
                });
            }
        }

        // Lire le PDF généré
        let pdf = std::fs::read(&tmp_pdf).map_err(|e| PdpError::TransformError {
            source_format: "FO".to_string(),
            target_format: "PDF".to_string(),
            message: format!("Impossible de lire le PDF généré: {}", e),
        })?;

        // Nettoyer le PDF temporaire
        let _ = std::fs::remove_file(&tmp_pdf);

        if pdf.len() < 5 || &pdf[0..5] != b"%PDF-" {
            return Err(PdpError::TransformError {
                source_format: "FO".to_string(),
                target_format: "PDF".to_string(),
                message: "Le fichier généré par FOP n'est pas un PDF valide".to_string(),
            });
        }

        Ok(pdf)
    }

    // --- Helpers internes ---

    /// Exécute Saxon avec un XML source et un XSLT, retourne le résultat XML.
    fn run_saxon(
        &self,
        xml: &str,
        xsl_path: &Path,
        params: &[(&str, &str)],
        label: &str,
    ) -> PdpResult<String> {
        let tmp_dir = std::env::temp_dir();
        let id = uuid::Uuid::new_v4();
        let tmp_xml = tmp_dir.join(format!("pdp-saxon-in-{}.xml", id));
        let tmp_out = tmp_dir.join(format!("pdp-saxon-out-{}.xml", id));

        std::fs::write(&tmp_xml, xml).map_err(|e| PdpError::TransformError {
            source_format: label.to_string(),
            target_format: label.to_string(),
            message: format!("Impossible d'écrire le fichier temporaire: {}", e),
        })?;

        tracing::debug!(
            transform = %label,
            xslt = %xsl_path.display(),
            "Transformation XSLT via {}", self.xslt_backend
        );

        let mut cmd = Command::new(self.xslt_backend.command());
        cmd.arg(format!("-s:{}", tmp_xml.to_str().unwrap_or("")))
           .arg(format!("-xsl:{}", xsl_path.to_str().unwrap_or("")))
           .arg(format!("-o:{}", tmp_out.to_str().unwrap_or("")));

        for (key, value) in params {
            cmd.arg(format!("{}={}", key, value));
        }

        let output = cmd.output();

        // Nettoyer l'entrée
        let _ = std::fs::remove_file(&tmp_xml);

        let output = output.map_err(|e| {
            let _ = std::fs::remove_file(&tmp_out);
            PdpError::TransformError {
                source_format: label.to_string(),
                target_format: label.to_string(),
                message: format!(
                    "Impossible d'exécuter {} pour {}: {}. Vérifiez que {} est installé \
                     (SaxonC: https://saxonica.com/download/c.html ou SaxonJ: brew install saxon).",
                    self.xslt_backend, label, e, self.xslt_backend
                ),
            }
        })?;

        // Lire le résultat depuis le fichier de sortie
        let result = if tmp_out.exists() {
            let content = std::fs::read_to_string(&tmp_out).map_err(|e| {
                PdpError::TransformError {
                    source_format: label.to_string(),
                    target_format: label.to_string(),
                    message: format!("Impossible de lire le résultat Saxon: {}", e),
                }
            })?;
            let _ = std::fs::remove_file(&tmp_out);
            content
        } else {
            // Fallback: lire depuis stdout
            let stdout = String::from_utf8(output.stdout).map_err(|e| {
                PdpError::TransformError {
                    source_format: label.to_string(),
                    target_format: label.to_string(),
                    message: format!("Le résultat Saxon n'est pas du UTF-8 valide: {}", e),
                }
            })?;
            stdout
        };

        if result.trim().is_empty() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(PdpError::TransformError {
                source_format: label.to_string(),
                target_format: label.to_string(),
                message: format!("Le résultat Saxon est vide pour {}. Stderr: {}", label, stderr.trim()),
            });
        }

        tracing::debug!(transform = %label, result_len = result.len(), "Transformation XSLT réussie");
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn engine() -> FopEngine {
        let specs = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../specs");
        FopEngine::new(&specs)
    }

    #[test]
    fn test_cii_to_xr() {
        let cii_xml = std::fs::read_to_string("../../tests/fixtures/cii/facture_cii_001.xml")
            .expect("Fixture CII introuvable");
        let xr = engine().transform_to_xr(&cii_xml, SourceSyntax::CII)
            .expect("CII→XR échoué");
        assert!(xr.contains("xr:invoice") || xr.contains("invoice"),
            "Le résultat XR doit contenir l'élément invoice");
        assert!(xr.contains("FA-2025-00256"),
            "Le résultat XR doit contenir le numéro de facture");
    }

    #[test]
    fn test_ubl_to_xr() {
        let ubl_xml = std::fs::read_to_string("../../tests/fixtures/ubl/facture_ubl_001.xml")
            .expect("Fixture UBL introuvable");
        let xr = engine().transform_to_xr(&ubl_xml, SourceSyntax::UBL)
            .expect("UBL→XR échoué");
        assert!(xr.contains("xr:invoice") || xr.contains("invoice"),
            "Le résultat XR doit contenir l'élément invoice");
        assert!(xr.contains("FA-2025-00142"),
            "Le résultat XR doit contenir le numéro de facture");
    }

    #[test]
    fn test_cii_to_fo() {
        let cii_xml = std::fs::read_to_string("../../tests/fixtures/cii/facture_cii_001.xml")
            .expect("Fixture CII introuvable");
        let xr = engine().transform_to_xr(&cii_xml, SourceSyntax::CII)
            .expect("CII→XR échoué");
        let fo = engine().transform_xr_to_fo(&xr)
            .expect("XR→FO échoué");
        assert!(fo.contains("fo:root"), "Le résultat FO doit contenir fo:root");
        assert!(fo.contains("fo:page-sequence"), "Le résultat FO doit contenir fo:page-sequence");
    }

    #[test]
    fn test_cii_to_pdf() {
        let cii_xml = std::fs::read_to_string("../../tests/fixtures/cii/facture_cii_001.xml")
            .expect("Fixture CII introuvable");
        let pdf = engine().cii_to_pdf(&cii_xml)
            .expect("CII→PDF échoué");
        assert!(pdf.len() > 100, "Le PDF doit avoir une taille raisonnable");
        assert_eq!(&pdf[0..5], b"%PDF-", "Le fichier doit commencer par %PDF-");
    }

    #[test]
    fn test_ubl_to_pdf() {
        let ubl_xml = std::fs::read_to_string("../../tests/fixtures/ubl/facture_ubl_001.xml")
            .expect("Fixture UBL introuvable");
        let pdf = engine().ubl_to_pdf(&ubl_xml)
            .expect("UBL→PDF échoué");
        assert!(pdf.len() > 100, "Le PDF doit avoir une taille raisonnable");
        assert_eq!(&pdf[0..5], b"%PDF-", "Le fichier doit commencer par %PDF-");
    }
}
