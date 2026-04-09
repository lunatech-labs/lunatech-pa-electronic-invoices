use pdp_core::error::{PdpError, PdpResult};
use pdp_core::model::{InvoiceData, InvoiceFormat};

use crate::xslt_engine::XsltEngine;

/// Format de sortie pour les conversions.
///
/// Étend [`InvoiceFormat`] avec `PDF` (PDF visuel sans XML embarqué).
///
/// # Exemple
///
/// ```
/// use pdp_transform::OutputFormat;
/// use pdp_core::model::InvoiceFormat;
///
/// // Créer depuis un InvoiceFormat
/// let format = OutputFormat::from(InvoiceFormat::UBL);
/// assert_eq!(format, OutputFormat::UBL);
///
/// // PDF n'existe pas dans InvoiceFormat
/// let pdf = OutputFormat::PDF;
/// assert_eq!(format!("{}" , pdf), "PDF");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    /// XML UBL 2.1
    UBL,
    /// XML CII D22B
    CII,
    /// PDF/A-3 Factur-X (PDF + XML CII embarqué + pièces jointes)
    FacturX,
    /// PDF visuel seul (sans XML embarqué, sans métadonnées Factur-X)
    PDF,
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputFormat::UBL => write!(f, "UBL"),
            OutputFormat::CII => write!(f, "CII"),
            OutputFormat::FacturX => write!(f, "Factur-X"),
            OutputFormat::PDF => write!(f, "PDF"),
        }
    }
}

impl From<InvoiceFormat> for OutputFormat {
    fn from(f: InvoiceFormat) -> Self {
        match f {
            InvoiceFormat::UBL => OutputFormat::UBL,
            InvoiceFormat::CII => OutputFormat::CII,
            InvoiceFormat::FacturX => OutputFormat::FacturX,
        }
    }
}

/// Résultat d'une conversion de format.
///
/// Contient le contenu converti (XML ou PDF), le format de sortie,
/// et un nom de fichier suggéré.
///
/// # Exemple
///
/// ```no_run
/// use pdp_transform::{convert_to, OutputFormat};
/// use pdp_invoice::ubl::UblParser;
///
/// let xml = std::fs::read_to_string("facture.xml").unwrap();
/// let invoice = UblParser::new().parse(&xml).unwrap();
/// let result = convert_to(&invoice, OutputFormat::CII).unwrap();
///
/// // Accéder au contenu XML
/// if let Some(xml) = result.as_string() {
///     println!("CII XML ({} octets) :", xml.len());
///     println!("{}", &xml[..200]);
/// }
///
/// // Vérifier si c'est un PDF
/// assert!(!result.is_pdf());
///
/// // Nom de fichier suggéré
/// println!("Enregistrer sous : {}", result.suggested_filename);
/// ```
pub struct ConversionResult {
    /// Contenu converti (XML ou PDF)
    pub content: Vec<u8>,
    /// Format de sortie
    pub output_format: OutputFormat,
    /// Nom de fichier suggéré
    pub suggested_filename: String,
}

impl ConversionResult {
    /// Retourne le contenu comme `String` (pour les formats XML).
    ///
    /// Retourne `None` pour les formats binaires (Factur-X, PDF).
    ///
    /// # Exemple
    ///
    /// ```no_run
    /// # use pdp_transform::{convert_to, OutputFormat};
    /// # use pdp_invoice::cii::CiiParser;
    /// # let xml = std::fs::read_to_string("facture_cii.xml").unwrap();
    /// # let invoice = CiiParser::new().parse(&xml).unwrap();
    /// let result = convert_to(&invoice, OutputFormat::UBL).unwrap();
    /// let ubl_xml = result.as_string().expect("Le résultat UBL est du XML");
    /// assert!(ubl_xml.contains("Invoice"));
    ///
    /// let pdf_result = convert_to(&invoice, OutputFormat::PDF).unwrap();
    /// assert!(pdf_result.as_string().is_none()); // PDF → pas de String
    /// ```
    pub fn as_string(&self) -> Option<String> {
        match self.output_format {
            OutputFormat::UBL | OutputFormat::CII => String::from_utf8(self.content.clone()).ok(),
            _ => None,
        }
    }

    /// Retourne `true` si le contenu est un PDF.
    ///
    /// # Exemple
    ///
    /// ```no_run
    /// # use pdp_transform::{convert_to, OutputFormat};
    /// # use pdp_invoice::ubl::UblParser;
    /// # let xml = std::fs::read_to_string("facture.xml").unwrap();
    /// # let invoice = UblParser::new().parse(&xml).unwrap();
    /// let result = convert_to(&invoice, OutputFormat::PDF).unwrap();
    /// assert!(result.is_pdf());
    ///
    /// let xml_result = convert_to(&invoice, OutputFormat::CII).unwrap();
    /// assert!(!xml_result.is_pdf());
    /// ```
    pub fn is_pdf(&self) -> bool {
        self.content.starts_with(b"%PDF-")
    }
}

// Pour la compatibilité ascendante, ConversionResult expose target_format
impl ConversionResult {
    /// Format cible (compatibilité avec l'ancien API).
    /// Retourne InvoiceFormat::FacturX pour PDF (le plus proche).
    pub fn target_format(&self) -> InvoiceFormat {
        match self.output_format {
            OutputFormat::UBL => InvoiceFormat::UBL,
            OutputFormat::CII => InvoiceFormat::CII,
            OutputFormat::FacturX | OutputFormat::PDF => InvoiceFormat::FacturX,
        }
    }
}

/// Convertit une facture vers le format de sortie spécifié.
///
/// C'est le point d'entrée principal de l'API de conversion.
/// Supporte les 9 chemins de conversion :
///
/// | Source | → CII | → UBL | → Factur-X | → PDF |
/// |--------|-------|-------|------------|-------|
/// | UBL | XSLT | — | XSLT+Typst+lopdf | Typst |
/// | CII | — | XSLT | Typst+lopdf | Typst |
/// | Factur-X | extraction | extraction+XSLT | — | retourne PDF |
///
/// # Exemples
///
/// ## UBL → CII (transformation XSLT)
///
/// ```no_run
/// use pdp_transform::{convert_to, OutputFormat};
/// use pdp_invoice::ubl::UblParser;
///
/// let xml = std::fs::read_to_string("facture_ubl.xml").unwrap();
/// let invoice = UblParser::new().parse(&xml).unwrap();
///
/// let result = convert_to(&invoice, OutputFormat::CII).unwrap();
/// let cii_xml = result.as_string().unwrap();
/// assert!(cii_xml.contains("CrossIndustryInvoice"));
/// std::fs::write(&result.suggested_filename, &result.content).unwrap();
/// ```
///
/// ## CII → UBL
///
/// ```no_run
/// use pdp_transform::{convert_to, OutputFormat};
/// use pdp_invoice::cii::CiiParser;
///
/// let xml = std::fs::read_to_string("facture_cii.xml").unwrap();
/// let invoice = CiiParser::new().parse(&xml).unwrap();
///
/// let result = convert_to(&invoice, OutputFormat::UBL).unwrap();
/// let ubl_xml = result.as_string().unwrap();
/// assert!(ubl_xml.contains("Invoice"));
/// ```
///
/// ## UBL → Factur-X (PDF/A-3 avec XML CII embarqué)
///
/// ```no_run
/// use pdp_transform::{convert_to, OutputFormat};
/// use pdp_invoice::ubl::UblParser;
///
/// let xml = std::fs::read_to_string("facture_ubl.xml").unwrap();
/// let invoice = UblParser::new().parse(&xml).unwrap();
///
/// let result = convert_to(&invoice, OutputFormat::FacturX).unwrap();
/// assert!(result.is_pdf());
/// std::fs::write("facture.pdf", &result.content).unwrap();
/// // Le PDF contient factur-x.xml embarqué + métadonnées XMP Factur-X
/// ```
///
/// ## UBL → PDF (visuel seul, sans XML embarqué)
///
/// ```no_run
/// use pdp_transform::{convert_to, OutputFormat};
/// use pdp_invoice::ubl::UblParser;
///
/// let xml = std::fs::read_to_string("facture_ubl.xml").unwrap();
/// let invoice = UblParser::new().parse(&xml).unwrap();
///
/// let result = convert_to(&invoice, OutputFormat::PDF).unwrap();
/// assert!(result.is_pdf());
/// std::fs::write("facture_visuel.pdf", &result.content).unwrap();
/// // PDF visuel uniquement — pas de XML embarqué, pas de métadonnées Factur-X
/// ```
///
/// ## Factur-X → CII (extraction du XML embarqué)
///
/// ```no_run
/// use pdp_transform::{convert_to, OutputFormat};
/// use pdp_invoice::facturx::FacturXParser;
///
/// let pdf = std::fs::read("facturx.pdf").unwrap();
/// let invoice = FacturXParser::new().parse(&pdf).unwrap();
///
/// let result = convert_to(&invoice, OutputFormat::CII).unwrap();
/// let cii_xml = result.as_string().unwrap();
/// // Le XML CII embarqué dans le Factur-X est extrait directement
/// ```
///
/// ## Factur-X → UBL (extraction + XSLT)
///
/// ```no_run
/// use pdp_transform::{convert_to, OutputFormat};
/// use pdp_invoice::facturx::FacturXParser;
///
/// let pdf = std::fs::read("facturx.pdf").unwrap();
/// let invoice = FacturXParser::new().parse(&pdf).unwrap();
///
/// let result = convert_to(&invoice, OutputFormat::UBL).unwrap();
/// let ubl_xml = result.as_string().unwrap();
/// assert!(ubl_xml.contains("Invoice"));
/// ```
///
/// ## Avec pièces jointes (BG-24)
///
/// ```no_run
/// use pdp_transform::{convert_to, OutputFormat};
/// use pdp_invoice::ubl::UblParser;
/// use pdp_core::model::InvoiceAttachment;
///
/// let xml = std::fs::read_to_string("facture_ubl.xml").unwrap();
/// let mut invoice = UblParser::new().parse(&xml).unwrap();
///
/// // Ajouter une pièce jointe
/// invoice.attachments.push(InvoiceAttachment {
///     id: Some("ATT-001".to_string()),
///     description: Some("Bon de commande".to_string()),
///     external_uri: None,
///     embedded_content: Some(std::fs::read("bon_commande.pdf").unwrap()),
///     mime_code: Some("application/pdf".to_string()),
///     filename: Some("bon_commande.pdf".to_string()),
/// });
///
/// // Les pièces jointes sont embarquées dans le PDF Factur-X
/// let result = convert_to(&invoice, OutputFormat::FacturX).unwrap();
/// std::fs::write("facture_avec_pj.pdf", &result.content).unwrap();
/// ```
pub fn convert_to(invoice: &InvoiceData, target: OutputFormat) -> PdpResult<ConversionResult> {
    match target {
        OutputFormat::CII => convert_to_cii(invoice),
        OutputFormat::UBL => convert_to_ubl(invoice),
        OutputFormat::FacturX => convert_to_facturx(invoice),
        OutputFormat::PDF => convert_to_pdf(invoice),
    }
}

/// Convertit une facture vers un [`InvoiceFormat`] (compatibilité ascendante).
///
/// Équivalent à `convert_to(invoice, OutputFormat::from(target))`.
/// Pour les nouvelles utilisations, préférez [`convert_to`] qui supporte aussi `OutputFormat::PDF`.
///
/// # Exemple
///
/// ```no_run
/// use pdp_transform::convert;
/// use pdp_core::model::InvoiceFormat;
/// use pdp_invoice::ubl::UblParser;
///
/// let xml = std::fs::read_to_string("facture.xml").unwrap();
/// let invoice = UblParser::new().parse(&xml).unwrap();
///
/// let result = convert(&invoice, InvoiceFormat::CII).unwrap();
/// assert_eq!(result.target_format(), InvoiceFormat::CII);
/// ```
pub fn convert(invoice: &InvoiceData, target: InvoiceFormat) -> PdpResult<ConversionResult> {
    convert_to(invoice, OutputFormat::from(target))
}

/// Transforme le XML brut via XSLT (UBL→CII ou CII→UBL).
///
/// Fonction bas-niveau qui applique directement la feuille XSLT.
/// Requiert que `raw_xml` soit présent dans l'`InvoiceData`.
/// Pour la plupart des cas, utilisez [`convert_to`] à la place.
///
/// # Exemple
///
/// ```no_run
/// use pdp_transform::xslt_transform;
/// use pdp_core::model::InvoiceFormat;
/// use pdp_invoice::ubl::UblParser;
///
/// let xml = std::fs::read_to_string("facture_ubl.xml").unwrap();
/// let invoice = UblParser::new().parse(&xml).unwrap();
///
/// // Transformation XSLT directe UBL → CII
/// let cii_xml = xslt_transform(&invoice, InvoiceFormat::CII).unwrap();
/// assert!(cii_xml.contains("CrossIndustryInvoice"));
/// ```
pub fn xslt_transform(invoice: &InvoiceData, target: InvoiceFormat) -> PdpResult<String> {
    let raw_xml = invoice.raw_xml.as_deref().ok_or_else(|| {
        PdpError::TransformError {
            source_format: invoice.source_format.to_string(),
            target_format: target.to_string(),
            message: "Pas de XML brut (raw_xml) disponible pour la transformation XSLT".to_string(),
        }
    })?;

    let engine = XsltEngine::from_manifest_dir();
    match target {
        InvoiceFormat::CII => engine.ubl_to_cii(raw_xml),
        InvoiceFormat::UBL => engine.cii_to_ubl(raw_xml),
        InvoiceFormat::FacturX => Err(PdpError::TransformError {
            source_format: invoice.source_format.to_string(),
            target_format: "Factur-X".to_string(),
            message: "Utilisez convert() pour la conversion vers Factur-X".to_string(),
        }),
    }
}

/// Convertit vers CII XML via XSLT
fn convert_to_cii(invoice: &InvoiceData) -> PdpResult<ConversionResult> {
    let xml = match invoice.source_format {
        InvoiceFormat::CII | InvoiceFormat::FacturX => invoice.raw_xml.clone().unwrap_or_default(),
        InvoiceFormat::UBL => xslt_transform(invoice, InvoiceFormat::CII)?,
    };
    Ok(ConversionResult {
        content: xml.into_bytes(),
        output_format: OutputFormat::CII,
        suggested_filename: make_output_filename(&invoice.invoice_number, OutputFormat::CII),
    })
}

/// Convertit vers UBL XML via XSLT
fn convert_to_ubl(invoice: &InvoiceData) -> PdpResult<ConversionResult> {
    let xml = match invoice.source_format {
        InvoiceFormat::UBL => invoice.raw_xml.clone().unwrap_or_default(),
        InvoiceFormat::CII | InvoiceFormat::FacturX => xslt_transform(invoice, InvoiceFormat::UBL)?,
    };
    Ok(ConversionResult {
        content: xml.into_bytes(),
        output_format: OutputFormat::UBL,
        suggested_filename: make_output_filename(&invoice.invoice_number, OutputFormat::UBL),
    })
}

/// Convertit vers Factur-X (PDF/A-3 avec XML CII embarqué).
fn convert_to_facturx(invoice: &InvoiceData) -> PdpResult<ConversionResult> {
    // Si on a déjà un PDF source (Factur-X original), le retourner
    if invoice.source_format == InvoiceFormat::FacturX {
        if let Some(ref pdf) = invoice.raw_pdf {
            return Ok(ConversionResult {
                content: pdf.clone(),
                output_format: OutputFormat::FacturX,
                suggested_filename: make_output_filename(&invoice.invoice_number, OutputFormat::FacturX),
            });
        }
    }

    let generator = crate::facturx_generator::FacturXGenerator::from_manifest_dir();
    match generator.generate(invoice) {
        Ok(result) => {
            tracing::info!(
                invoice = %invoice.invoice_number,
                level = %result.level,
                pdf_size = result.pdf.len(),
                "Factur-X PDF/A-3 généré via Typst"
            );
            Ok(ConversionResult {
                content: result.pdf,
                output_format: OutputFormat::FacturX,
                suggested_filename: result.filename,
            })
        }
        Err(e) => {
            Err(PdpError::TransformError {
                source_format: invoice.source_format.to_string(),
                target_format: "Factur-X".to_string(),
                message: format!("Échec génération Factur-X: {}", e),
            })
        }
    }
}

/// Convertit vers PDF visuel seul (sans XML embarqué, sans métadonnées Factur-X).
///
/// Utilise le moteur Typst (in-process, ~100ms).
fn convert_to_pdf(invoice: &InvoiceData) -> PdpResult<ConversionResult> {
    // Si on a déjà un PDF (Factur-X source), le retourner
    if let Some(ref pdf) = invoice.raw_pdf {
        return Ok(ConversionResult {
            content: pdf.clone(),
            output_format: OutputFormat::PDF,
            suggested_filename: make_output_filename(&invoice.invoice_number, OutputFormat::PDF),
        });
    }

    let typst = crate::typst_engine::TypstPdfEngine::from_manifest_dir();
    let pdf = typst.generate_pdf(invoice)?;

    Ok(ConversionResult {
        content: pdf,
        output_format: OutputFormat::PDF,
        suggested_filename: make_output_filename(&invoice.invoice_number, OutputFormat::PDF),
    })
}

/// Génère un nom de fichier suggéré pour le format de sortie
fn make_output_filename(invoice_number: &str, format: OutputFormat) -> String {
    let safe_name = invoice_number
        .replace('/', "-")
        .replace('\\', "-")
        .replace(' ', "_");
    match format {
        OutputFormat::UBL => format!("{}_ubl.xml", safe_name),
        OutputFormat::CII => format!("{}_cii.xml", safe_name),
        OutputFormat::FacturX => format!("{}_facturx.pdf", safe_name),
        OutputFormat::PDF => format!("{}.pdf", safe_name),
    }
}

/// Retourne les formats de sortie possibles depuis un format source.
///
/// Chaque format source peut être converti vers 3 formats de sortie.
///
/// # Exemple
///
/// ```
/// use pdp_transform::{supported_output_formats, OutputFormat};
/// use pdp_core::model::InvoiceFormat;
///
/// let formats = supported_output_formats(&InvoiceFormat::UBL);
/// assert_eq!(formats.len(), 3);
/// assert!(formats.contains(&OutputFormat::CII));
/// assert!(formats.contains(&OutputFormat::FacturX));
/// assert!(formats.contains(&OutputFormat::PDF));
///
/// let cii_formats = supported_output_formats(&InvoiceFormat::CII);
/// assert!(cii_formats.contains(&OutputFormat::UBL));
///
/// let fx_formats = supported_output_formats(&InvoiceFormat::FacturX);
/// assert!(fx_formats.contains(&OutputFormat::CII));
/// assert!(fx_formats.contains(&OutputFormat::UBL));
/// ```
pub fn supported_output_formats(source: &InvoiceFormat) -> Vec<OutputFormat> {
    match source {
        InvoiceFormat::UBL => vec![OutputFormat::CII, OutputFormat::FacturX, OutputFormat::PDF],
        InvoiceFormat::CII => vec![OutputFormat::UBL, OutputFormat::FacturX, OutputFormat::PDF],
        InvoiceFormat::FacturX => vec![OutputFormat::CII, OutputFormat::UBL, OutputFormat::PDF],
    }
}

/// Retourne les conversions possibles (compatibilité ascendante)
pub fn supported_conversions(source: &InvoiceFormat) -> Vec<InvoiceFormat> {
    match source {
        InvoiceFormat::UBL => vec![InvoiceFormat::CII, InvoiceFormat::FacturX],
        InvoiceFormat::CII => vec![InvoiceFormat::UBL, InvoiceFormat::FacturX],
        InvoiceFormat::FacturX => vec![InvoiceFormat::CII, InvoiceFormat::UBL],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::Engine as _;

    // --- Helpers ---

    fn parse_ubl(path: &str) -> InvoiceData {
        let xml = std::fs::read_to_string(path).expect("Fixture UBL introuvable");
        pdp_invoice::ubl::UblParser::new().parse(&xml).expect("Parsing UBL échoué")
    }

    fn parse_cii(path: &str) -> InvoiceData {
        let xml = std::fs::read_to_string(path).expect("Fixture CII introuvable");
        pdp_invoice::cii::CiiParser::new().parse(&xml).expect("Parsing CII échoué")
    }

    fn parse_facturx(path: &str) -> InvoiceData {
        let pdf = std::fs::read(path).expect("Fixture Factur-X introuvable");
        pdp_invoice::facturx::FacturXParser::new().parse(&pdf).expect("Parsing Factur-X échoué")
    }

    const UBL_FIXTURE: &str = "../../tests/fixtures/ubl/facture_ubl_001.xml";
    const CII_FIXTURE: &str = "../../tests/fixtures/cii/facture_cii_001.xml";

    // --- 1. UBL → CII ---

    #[test]
    fn test_ubl_to_cii() {
        let invoice = parse_ubl(UBL_FIXTURE);
        let result = convert_to(&invoice, OutputFormat::CII).unwrap();
        assert_eq!(result.output_format, OutputFormat::CII);
        let xml = result.as_string().unwrap();
        assert!(xml.contains("CrossIndustryInvoice"), "Doit contenir CrossIndustryInvoice");
        assert!(xml.contains("FA-2025-00142"), "Doit préserver le numéro de facture");
        assert!(result.suggested_filename.ends_with("_cii.xml"));
    }

    // --- 2. UBL → Factur-X ---

    #[test]
    fn test_ubl_to_facturx() {
        let invoice = parse_ubl(UBL_FIXTURE);
        let result = convert_to(&invoice, OutputFormat::FacturX).unwrap();
        assert_eq!(result.output_format, OutputFormat::FacturX);
        assert!(!result.content.is_empty());
        if result.is_pdf() {
            assert!(result.suggested_filename.ends_with("_facturx.pdf"));
        } else {
            let xml = String::from_utf8(result.content).unwrap();
            assert!(xml.contains("CrossIndustryInvoice"));
        }
    }

    // --- 3. UBL → PDF ---

    #[test]
    fn test_ubl_to_pdf() {
        let invoice = parse_ubl(UBL_FIXTURE);
        let result = convert_to(&invoice, OutputFormat::PDF).unwrap();
        assert_eq!(result.output_format, OutputFormat::PDF);
        assert!(result.is_pdf(), "Le résultat doit être un PDF");
        assert!(result.suggested_filename.ends_with(".pdf"));
        assert!(!result.suggested_filename.contains("facturx"), "PDF seul, pas Factur-X");
    }

    // --- 4. CII → UBL ---

    #[test]
    fn test_cii_to_ubl() {
        let invoice = parse_cii(CII_FIXTURE);
        let result = convert_to(&invoice, OutputFormat::UBL).unwrap();
        assert_eq!(result.output_format, OutputFormat::UBL);
        let xml = result.as_string().unwrap();
        assert!(xml.contains("urn:oasis:names:specification:ubl:schema:xsd:Invoice-2"));
        assert!(xml.contains("FA-2025-00256"), "Doit préserver le numéro de facture");
        assert!(result.suggested_filename.ends_with("_ubl.xml"));
    }

    // --- 5. CII → Factur-X ---

    #[test]
    fn test_cii_to_facturx() {
        let invoice = parse_cii(CII_FIXTURE);
        let result = convert_to(&invoice, OutputFormat::FacturX).unwrap();
        assert_eq!(result.output_format, OutputFormat::FacturX);
        assert!(!result.content.is_empty());
        if result.is_pdf() {
            assert!(result.suggested_filename.ends_with("_facturx.pdf"));
        }
    }

    // --- 6. CII → PDF ---

    #[test]
    fn test_cii_to_pdf() {
        let invoice = parse_cii(CII_FIXTURE);
        let result = convert_to(&invoice, OutputFormat::PDF).unwrap();
        assert_eq!(result.output_format, OutputFormat::PDF);
        assert!(result.is_pdf(), "Le résultat doit être un PDF");
        assert!(result.suggested_filename.ends_with(".pdf"));
    }

    // --- 7. Factur-X → CII ---

    #[test]
    fn test_facturx_to_cii() {
        let facturx_path = "../../output/facturx_cii_extended.pdf";
        if !std::path::Path::new(facturx_path).exists() {
            eprintln!("Factur-X PDF non disponible, test ignoré (lancer export_facturx_examples d'abord)");
            return;
        }
        let invoice = parse_facturx(facturx_path);
        let result = convert_to(&invoice, OutputFormat::CII).unwrap();
        assert_eq!(result.output_format, OutputFormat::CII);
        let xml = result.as_string().unwrap();
        assert!(xml.contains("CrossIndustryInvoice"), "Doit extraire le XML CII embarqué");
    }

    // --- 8. Factur-X → UBL ---

    #[test]
    fn test_facturx_to_ubl() {
        let facturx_path = "../../output/facturx_cii_extended.pdf";
        if !std::path::Path::new(facturx_path).exists() {
            eprintln!("Factur-X PDF non disponible, test ignoré");
            return;
        }
        let invoice = parse_facturx(facturx_path);
        let result = convert_to(&invoice, OutputFormat::UBL).unwrap();
        assert_eq!(result.output_format, OutputFormat::UBL);
        let xml = result.as_string().unwrap();
        assert!(xml.contains("urn:oasis:names:specification:ubl:schema:xsd:Invoice-2"));
    }

    // --- 9. Factur-X → PDF ---

    #[test]
    fn test_facturx_to_pdf() {
        let facturx_path = "../../output/facturx_cii_extended.pdf";
        if !std::path::Path::new(facturx_path).exists() {
            eprintln!("Factur-X PDF non disponible, test ignoré");
            return;
        }
        let invoice = parse_facturx(facturx_path);
        let result = convert_to(&invoice, OutputFormat::PDF).unwrap();
        assert_eq!(result.output_format, OutputFormat::PDF);
        assert!(result.is_pdf(), "Doit retourner le PDF existant");
    }

    // --- Noop (même format) ---

    #[test]
    fn test_ubl_to_ubl_noop() {
        let invoice = parse_ubl(UBL_FIXTURE);
        let result = convert_to(&invoice, OutputFormat::UBL).unwrap();
        assert_eq!(result.output_format, OutputFormat::UBL);
        assert!(!result.content.is_empty());
        let xml = result.as_string().unwrap();
        assert!(xml.contains("FA-2025-00142"));
    }

    #[test]
    fn test_cii_to_cii_noop() {
        let invoice = parse_cii(CII_FIXTURE);
        let result = convert_to(&invoice, OutputFormat::CII).unwrap();
        assert_eq!(result.output_format, OutputFormat::CII);
        let xml = result.as_string().unwrap();
        assert!(xml.contains("FA-2025-00256"));
    }

    // --- Roundtrip ---

    #[test]
    fn test_roundtrip_ubl_cii_ubl() {
        let invoice = parse_ubl(UBL_FIXTURE);

        // UBL → CII
        let cii_result = convert_to(&invoice, OutputFormat::CII).unwrap();
        let cii_xml = String::from_utf8(cii_result.content).unwrap();

        // Re-parse CII → UBL
        let invoice2 = pdp_invoice::cii::CiiParser::new().parse(&cii_xml).expect("Re-parsing CII échoué");
        let ubl_result = convert_to(&invoice2, OutputFormat::UBL).unwrap();
        let ubl_xml2 = result_as_string(&ubl_result);

        assert!(ubl_xml2.contains("FA-2025-00142"), "Numéro de facture préservé");
        assert!(ubl_xml2.contains("TechConseil SAS"), "Nom vendeur préservé");
    }

    #[test]
    fn test_roundtrip_cii_ubl_cii() {
        let invoice = parse_cii(CII_FIXTURE);

        // CII → UBL
        let ubl_result = convert_to(&invoice, OutputFormat::UBL).unwrap();
        let ubl_xml = String::from_utf8(ubl_result.content).unwrap();

        // Re-parse UBL → CII
        let invoice2 = pdp_invoice::ubl::UblParser::new().parse(&ubl_xml).expect("Re-parsing UBL échoué");
        let cii_result = convert_to(&invoice2, OutputFormat::CII).unwrap();
        let cii_xml2 = result_as_string(&cii_result);

        assert!(cii_xml2.contains("FA-2025-00256"), "Numéro de facture préservé");
    }

    // --- Avec pièces jointes (BG-24) ---

    #[test]
    fn test_ubl_to_facturx_with_attachments() {
        let mut invoice = parse_ubl(UBL_FIXTURE);
        // Ajouter une pièce jointe fictive
        invoice.attachments.push(pdp_core::model::InvoiceAttachment {
            id: Some("ATT-001".to_string()),
            description: Some("Bon de commande".to_string()),
            external_uri: None,
            embedded_content: Some(b"Contenu du bon de commande en texte".to_vec()),
            mime_code: Some("text/plain".to_string()),
            filename: Some("bon_commande.txt".to_string()),
        });

        let result = convert_to(&invoice, OutputFormat::FacturX).unwrap();
        assert_eq!(result.output_format, OutputFormat::FacturX);
        assert!(!result.content.is_empty());
        if result.is_pdf() {
            // Vérifier que le PDF contient la pièce jointe embarquée
            let doc = lopdf::Document::load_mem(&result.content).expect("PDF invalide");
            // Chercher le nom de fichier dans les objets
            let has_attachment = doc.objects.values().any(|obj| {
                format!("{:?}", obj).contains("bon_commande.txt")
            });
            assert!(has_attachment, "Le PDF doit contenir la pièce jointe bon_commande.txt");
        }
    }

    #[test]
    fn test_cii_to_facturx_with_attachments() {
        let mut invoice = parse_cii(CII_FIXTURE);
        invoice.attachments.push(pdp_core::model::InvoiceAttachment {
            id: Some("ATT-002".to_string()),
            description: Some("Feuille de temps".to_string()),
            external_uri: None,
            embedded_content: Some(b"Contenu de la feuille de temps".to_vec()),
            mime_code: Some("text/plain".to_string()),
            filename: Some("feuille_temps.txt".to_string()),
        });

        let result = convert_to(&invoice, OutputFormat::FacturX).unwrap();
        assert_eq!(result.output_format, OutputFormat::FacturX);
        assert!(!result.content.is_empty());
        if result.is_pdf() {
            let doc = lopdf::Document::load_mem(&result.content).expect("PDF invalide");
            let has_attachment = doc.objects.values().any(|obj| {
                format!("{:?}", obj).contains("feuille_temps.txt")
            });
            assert!(has_attachment, "Le PDF doit contenir la pièce jointe feuille_temps.txt");
        }
    }

    #[test]
    fn test_ubl_to_facturx_with_multiple_attachments() {
        let mut invoice = parse_ubl(UBL_FIXTURE);
        invoice.attachments.push(pdp_core::model::InvoiceAttachment {
            id: Some("ATT-001".to_string()),
            description: Some("Bon de commande".to_string()),
            external_uri: None,
            embedded_content: Some(b"BDC contenu".to_vec()),
            mime_code: Some("text/plain".to_string()),
            filename: Some("bdc.txt".to_string()),
        });
        invoice.attachments.push(pdp_core::model::InvoiceAttachment {
            id: Some("ATT-002".to_string()),
            description: Some("Photo chantier".to_string()),
            external_uri: None,
            embedded_content: Some(vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]), // PNG header
            mime_code: Some("image/png".to_string()),
            filename: Some("chantier.png".to_string()),
        });

        let result = convert_to(&invoice, OutputFormat::FacturX).unwrap();
        assert!(!result.content.is_empty());
        if result.is_pdf() {
            let doc = lopdf::Document::load_mem(&result.content).expect("PDF invalide");
            let objects_str = format!("{:?}", doc.objects);
            assert!(objects_str.contains("bdc.txt"), "Doit contenir bdc.txt");
            assert!(objects_str.contains("chantier.png"), "Doit contenir chantier.png");
        }
    }

    // --- Pièces jointes multi-types dans conversions XML ---

    /// Helper : crée un jeu de pièces jointes variées (PDF, PNG, CSV, référence externe)
    fn sample_attachments() -> Vec<pdp_core::model::InvoiceAttachment> {
        vec![
            // PDF embarqué (header PDF minimal)
            pdp_core::model::InvoiceAttachment {
                id: Some("ATT-PDF-001".to_string()),
                description: Some("Bon de commande PDF".to_string()),
                external_uri: None,
                embedded_content: Some(b"%PDF-1.4 fake pdf content for testing".to_vec()),
                mime_code: Some("application/pdf".to_string()),
                filename: Some("bon_commande.pdf".to_string()),
            },
            // Image PNG (header PNG réel)
            pdp_core::model::InvoiceAttachment {
                id: Some("ATT-PNG-002".to_string()),
                description: Some("Photo chantier".to_string()),
                external_uri: None,
                embedded_content: Some(vec![
                    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
                    0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52, // IHDR chunk
                ]),
                mime_code: Some("image/png".to_string()),
                filename: Some("photo_chantier.png".to_string()),
            },
            // CSV
            pdp_core::model::InvoiceAttachment {
                id: Some("ATT-CSV-003".to_string()),
                description: Some("Détail des lignes".to_string()),
                external_uri: None,
                embedded_content: Some(b"ref;qte;pu\nA001;10;25.00\nA002;5;12.50\nA003;3;8.75".to_vec()),
                mime_code: Some("text/csv".to_string()),
                filename: Some("detail_lignes.csv".to_string()),
            },
            // Référence externe (pas de contenu embarqué)
            pdp_core::model::InvoiceAttachment {
                id: Some("ATT-EXT-004".to_string()),
                description: Some("Cahier des charges".to_string()),
                external_uri: Some("https://example.com/specs/cahier_charges.pdf".to_string()),
                embedded_content: None,
                mime_code: None,
                filename: None,
            },
        ]
    }

    #[test]
    fn test_ubl_to_cii_with_mixed_attachments() {
        let mut invoice = parse_ubl(UBL_FIXTURE);
        let attachments = sample_attachments();
        invoice.attachments = attachments.clone();

        // Sérialiser en UBL avec PJ, puis transformer via XSLT
        // Les PJ doivent être dans le raw_xml pour que le XSLT les transforme
        // Ici on injecte les PJ dans le XML UBL avant transformation
        let raw_xml = invoice.raw_xml.as_ref().unwrap();

        // Construire les blocs AdditionalDocumentReference pour chaque PJ
        let mut pj_xml = String::new();
        for att in &attachments {
            pj_xml.push_str("  <cac:AdditionalDocumentReference>\n");
            pj_xml.push_str(&format!("    <cbc:ID>{}</cbc:ID>\n", att.id.as_deref().unwrap_or("ATT")));
            if let Some(ref desc) = att.description {
                pj_xml.push_str(&format!("    <cbc:DocumentDescription>{}</cbc:DocumentDescription>\n", desc));
            }
            pj_xml.push_str("    <cac:Attachment>\n");
            if let Some(ref content) = att.embedded_content {
                let b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, content);
                let mime = att.mime_code.as_deref().unwrap_or("application/octet-stream");
                let fname = att.filename.as_deref().unwrap_or("attachment.bin");
                pj_xml.push_str(&format!(
                    "      <cbc:EmbeddedDocumentBinaryObject mimeCode=\"{}\" filename=\"{}\">{}</cbc:EmbeddedDocumentBinaryObject>\n",
                    mime, fname, b64
                ));
            }
            if let Some(ref uri) = att.external_uri {
                pj_xml.push_str(&format!(
                    "      <cac:ExternalReference><cbc:URI>{}</cbc:URI></cac:ExternalReference>\n",
                    uri
                ));
            }
            pj_xml.push_str("    </cac:Attachment>\n");
            pj_xml.push_str("  </cac:AdditionalDocumentReference>\n");
        }

        // Injecter avant </Invoice>
        let modified_xml = raw_xml.replace("</Invoice>", &format!("{}</Invoice>", pj_xml));
        invoice.raw_xml = Some(modified_xml);

        let result = convert_to(&invoice, OutputFormat::CII).unwrap();
        let cii_xml = result_as_string(&result);

        // Vérifier que les PJ embarquées sont dans le CII
        assert!(cii_xml.contains("AdditionalReferencedDocument"),
            "CII doit contenir AdditionalReferencedDocument");
        assert!(cii_xml.contains("ATT-PDF-001"), "PJ PDF doit être présente");
        assert!(cii_xml.contains("ATT-PNG-002"), "PJ PNG doit être présente");
        assert!(cii_xml.contains("ATT-CSV-003"), "PJ CSV doit être présente");
        assert!(cii_xml.contains("ATT-EXT-004"), "PJ externe doit être présente");

        // Vérifier le contenu base64 des PJ embarquées
        assert!(cii_xml.contains("AttachmentBinaryObject"),
            "CII doit contenir AttachmentBinaryObject pour les PJ embarquées");
        assert!(cii_xml.contains("application/pdf"), "mimeCode PDF doit être préservé");
        assert!(cii_xml.contains("image/png"), "mimeCode PNG doit être préservé");
        assert!(cii_xml.contains("text/csv"), "mimeCode CSV doit être préservé");
        assert!(cii_xml.contains("bon_commande.pdf"), "filename PDF doit être préservé");
        assert!(cii_xml.contains("photo_chantier.png"), "filename PNG doit être préservé");
        assert!(cii_xml.contains("detail_lignes.csv"), "filename CSV doit être préservé");

        // Vérifier la référence externe
        assert!(cii_xml.contains("https://example.com/specs/cahier_charges.pdf"),
            "URI externe doit être préservée");
    }

    #[test]
    fn test_cii_to_ubl_with_mixed_attachments() {
        let mut invoice = parse_cii(CII_FIXTURE);
        let attachments = sample_attachments();

        // Injecter les PJ dans le XML CII avant transformation
        let raw_xml = invoice.raw_xml.as_ref().unwrap();

        let mut pj_xml = String::new();
        for att in &attachments {
            // Ordre XSD : IssuerAssignedID, URIID, LineID, TypeCode, Name, AttachmentBinaryObject
            pj_xml.push_str("        <ram:AdditionalReferencedDocument>\n");
            pj_xml.push_str(&format!("          <ram:IssuerAssignedID>{}</ram:IssuerAssignedID>\n",
                att.id.as_deref().unwrap_or("ATT")));
            if let Some(ref uri) = att.external_uri {
                pj_xml.push_str(&format!("          <ram:URIID>{}</ram:URIID>\n", uri));
            }
            pj_xml.push_str("          <ram:TypeCode>916</ram:TypeCode>\n");
            if let Some(ref desc) = att.description {
                pj_xml.push_str(&format!("          <ram:Name>{}</ram:Name>\n", desc));
            }
            if let Some(ref content) = att.embedded_content {
                let b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, content);
                let mime = att.mime_code.as_deref().unwrap_or("application/octet-stream");
                let fname = att.filename.as_deref().unwrap_or("attachment.bin");
                pj_xml.push_str(&format!(
                    "          <ram:AttachmentBinaryObject mimeCode=\"{}\" filename=\"{}\">{}</ram:AttachmentBinaryObject>\n",
                    mime, fname, b64
                ));
            }
            pj_xml.push_str("        </ram:AdditionalReferencedDocument>\n");
        }

        // Injecter dans ApplicableHeaderTradeAgreement (avant </ram:ApplicableHeaderTradeAgreement>)
        let modified_xml = raw_xml.replace(
            "</ram:ApplicableHeaderTradeAgreement>",
            &format!("{}</ram:ApplicableHeaderTradeAgreement>", pj_xml)
        );
        invoice.raw_xml = Some(modified_xml);

        let result = convert_to(&invoice, OutputFormat::UBL).unwrap();
        let ubl_xml = result_as_string(&result);

        // Vérifier que les PJ sont dans le UBL
        assert!(ubl_xml.contains("AdditionalDocumentReference"),
            "UBL doit contenir AdditionalDocumentReference");
        assert!(ubl_xml.contains("ATT-PDF-001"), "PJ PDF doit être présente");
        assert!(ubl_xml.contains("ATT-PNG-002"), "PJ PNG doit être présente");
        assert!(ubl_xml.contains("ATT-CSV-003"), "PJ CSV doit être présente");
        assert!(ubl_xml.contains("ATT-EXT-004"), "PJ externe doit être présente");

        // Vérifier le contenu base64 des PJ embarquées
        assert!(ubl_xml.contains("EmbeddedDocumentBinaryObject"),
            "UBL doit contenir EmbeddedDocumentBinaryObject pour les PJ embarquées");
        assert!(ubl_xml.contains("application/pdf"), "mimeCode PDF doit être préservé");
        assert!(ubl_xml.contains("image/png"), "mimeCode PNG doit être préservé");
        assert!(ubl_xml.contains("text/csv"), "mimeCode CSV doit être préservé");
        assert!(ubl_xml.contains("bon_commande.pdf"), "filename PDF doit être préservé");
        assert!(ubl_xml.contains("photo_chantier.png"), "filename PNG doit être préservé");
        assert!(ubl_xml.contains("detail_lignes.csv"), "filename CSV doit être préservé");

        // Vérifier la référence externe
        assert!(ubl_xml.contains("https://example.com/specs/cahier_charges.pdf"),
            "URI externe doit être préservée");
    }

    #[test]
    fn test_roundtrip_ubl_cii_ubl_with_attachments() {
        let mut invoice = parse_ubl(UBL_FIXTURE);
        let raw_xml = invoice.raw_xml.as_ref().unwrap().clone();

        // Injecter une PJ PDF et une PJ PNG dans le XML UBL
        let pdf_b64 = base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            b"%PDF-1.4 test roundtrip content",
        );
        let png_b64 = base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A],
        );

        let pj_xml = format!(
            r#"  <cac:AdditionalDocumentReference>
    <cbc:ID>RT-PDF-001</cbc:ID>
    <cbc:DocumentDescription>Roundtrip PDF</cbc:DocumentDescription>
    <cac:Attachment>
      <cbc:EmbeddedDocumentBinaryObject mimeCode="application/pdf" filename="roundtrip.pdf">{}</cbc:EmbeddedDocumentBinaryObject>
    </cac:Attachment>
  </cac:AdditionalDocumentReference>
  <cac:AdditionalDocumentReference>
    <cbc:ID>RT-PNG-002</cbc:ID>
    <cbc:DocumentDescription>Roundtrip PNG</cbc:DocumentDescription>
    <cac:Attachment>
      <cbc:EmbeddedDocumentBinaryObject mimeCode="image/png" filename="roundtrip.png">{}</cbc:EmbeddedDocumentBinaryObject>
    </cac:Attachment>
  </cac:AdditionalDocumentReference>
"#, pdf_b64, png_b64);

        let modified_xml = raw_xml.replace("</Invoice>", &format!("{}</Invoice>", pj_xml));
        invoice.raw_xml = Some(modified_xml);

        // UBL → CII
        let cii_result = convert_to(&invoice, OutputFormat::CII).unwrap();
        let cii_xml = result_as_string(&cii_result);

        assert!(cii_xml.contains("RT-PDF-001"), "CII doit contenir PJ PDF après UBL→CII");
        assert!(cii_xml.contains("RT-PNG-002"), "CII doit contenir PJ PNG après UBL→CII");
        assert!(cii_xml.contains("application/pdf"), "mimeCode PDF préservé dans CII");
        assert!(cii_xml.contains("image/png"), "mimeCode PNG préservé dans CII");
        assert!(cii_xml.contains("roundtrip.pdf"), "filename PDF préservé dans CII");
        assert!(cii_xml.contains("roundtrip.png"), "filename PNG préservé dans CII");

        // CII → UBL (re-parse le CII puis reconvertir)
        let invoice2 = pdp_invoice::cii::CiiParser::new().parse(&cii_xml)
            .expect("Re-parsing CII avec PJ échoué");
        let ubl_result = convert_to(&invoice2, OutputFormat::UBL).unwrap();
        let ubl_xml2 = result_as_string(&ubl_result);

        assert!(ubl_xml2.contains("RT-PDF-001"), "UBL doit contenir PJ PDF après roundtrip");
        assert!(ubl_xml2.contains("RT-PNG-002"), "UBL doit contenir PJ PNG après roundtrip");
        assert!(ubl_xml2.contains("application/pdf"), "mimeCode PDF préservé après roundtrip");
        assert!(ubl_xml2.contains("image/png"), "mimeCode PNG préservé après roundtrip");
        assert!(ubl_xml2.contains("roundtrip.pdf"), "filename PDF préservé après roundtrip");
        assert!(ubl_xml2.contains("roundtrip.png"), "filename PNG préservé après roundtrip");

        // Vérifier que le contenu base64 est identique
        assert!(ubl_xml2.contains(&pdf_b64), "Contenu base64 PDF identique après roundtrip");
        assert!(ubl_xml2.contains(&png_b64), "Contenu base64 PNG identique après roundtrip");
    }

    #[test]
    fn test_ubl_to_facturx_with_mixed_attachment_types() {
        let mut invoice = parse_ubl(UBL_FIXTURE);
        invoice.attachments = sample_attachments();

        let result = convert_to(&invoice, OutputFormat::FacturX).unwrap();
        assert!(!result.content.is_empty());

        if result.is_pdf() {
            let doc = lopdf::Document::load_mem(&result.content).expect("PDF invalide");
            let objects_str = format!("{:?}", doc.objects);

            // Vérifier que les 3 PJ embarquées sont dans le PDF (pas la référence externe)
            assert!(objects_str.contains("bon_commande.pdf"),
                "PDF doit contenir bon_commande.pdf");
            assert!(objects_str.contains("photo_chantier.png"),
                "PDF doit contenir photo_chantier.png");
            assert!(objects_str.contains("detail_lignes.csv"),
                "PDF doit contenir detail_lignes.csv");

            // Vérifier les types MIME
            assert!(objects_str.contains("application/pdf"),
                "PDF doit contenir le type MIME application/pdf");
            assert!(objects_str.contains("image/png"),
                "PDF doit contenir le type MIME image/png");
            assert!(objects_str.contains("text/csv"),
                "PDF doit contenir le type MIME text/csv");

            // Vérifier que factur-x.xml est aussi présent
            assert!(objects_str.contains("factur-x.xml"),
                "PDF doit contenir factur-x.xml");
        }
    }

    #[test]
    fn test_cii_to_facturx_with_mixed_attachment_types() {
        let mut invoice = parse_cii(CII_FIXTURE);
        invoice.attachments = sample_attachments();

        let result = convert_to(&invoice, OutputFormat::FacturX).unwrap();
        assert!(!result.content.is_empty());

        if result.is_pdf() {
            let doc = lopdf::Document::load_mem(&result.content).expect("PDF invalide");
            let objects_str = format!("{:?}", doc.objects);

            assert!(objects_str.contains("bon_commande.pdf"),
                "PDF doit contenir bon_commande.pdf");
            assert!(objects_str.contains("photo_chantier.png"),
                "PDF doit contenir photo_chantier.png");
            assert!(objects_str.contains("detail_lignes.csv"),
                "PDF doit contenir detail_lignes.csv");
            assert!(objects_str.contains("factur-x.xml"),
                "PDF doit contenir factur-x.xml");
        }
    }

    // --- Pipeline PJ : XML source → Factur-X → extraction PJ ---

    #[test]
    fn test_cii_with_attachments_to_facturx_then_extract() {
        // 1. Parser CII et ajouter des PJ au modèle
        let mut invoice = parse_cii(CII_FIXTURE);
        invoice.attachments.push(pdp_core::model::InvoiceAttachment {
            id: Some("PJ-PDF-001".to_string()),
            description: Some("Bon de commande".to_string()),
            external_uri: None,
            embedded_content: Some(b"%PDF-1.4 fake bon de commande".to_vec()),
            mime_code: Some("application/pdf".to_string()),
            filename: Some("bon_commande.pdf".to_string()),
        });
        invoice.attachments.push(pdp_core::model::InvoiceAttachment {
            id: Some("PJ-PNG-002".to_string()),
            description: Some("Photo chantier".to_string()),
            external_uri: None,
            embedded_content: Some(vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]),
            mime_code: Some("image/png".to_string()),
            filename: Some("photo.png".to_string()),
        });

        // 2. Convertir en Factur-X
        let facturx_result = convert_to(&invoice, OutputFormat::FacturX).unwrap();
        assert!(facturx_result.is_pdf(), "Doit être un PDF");

        // 3. Re-parser le Factur-X et vérifier les PJ extraites
        let parsed = pdp_invoice::facturx::FacturXParser::new()
            .parse(&facturx_result.content)
            .expect("Re-parsing Factur-X échoué");

        // Vérifier que factur-x.xml n'est PAS dans les attachments
        assert!(
            !parsed.attachments.iter().any(|a| {
                a.filename.as_deref().map(|f| f.to_lowercase()) == Some("factur-x.xml".to_string())
            }),
            "factur-x.xml ne doit PAS apparaître dans les pièces jointes"
        );

        // Vérifier que les 2 PJ sont extraites
        let filenames: Vec<&str> = parsed.attachments.iter()
            .filter_map(|a| a.filename.as_deref())
            .collect();
        assert!(filenames.contains(&"bon_commande.pdf"),
            "Doit contenir bon_commande.pdf, trouvé: {:?}", filenames);
        assert!(filenames.contains(&"photo.png"),
            "Doit contenir photo.png, trouvé: {:?}", filenames);

        // Vérifier le contenu des PJ
        for att in &parsed.attachments {
            assert!(att.embedded_content.is_some(),
                "PJ {} doit avoir du contenu", att.filename.as_deref().unwrap_or("?"));
        }
    }

    #[test]
    fn test_ubl_with_attachments_to_facturx_then_extract() {
        let mut invoice = parse_ubl(UBL_FIXTURE);
        invoice.attachments.push(pdp_core::model::InvoiceAttachment {
            id: Some("PJ-CSV-001".to_string()),
            description: Some("Détail lignes".to_string()),
            external_uri: None,
            embedded_content: Some(b"ref;qte;pu\nA001;10;25.00".to_vec()),
            mime_code: Some("text/csv".to_string()),
            filename: Some("detail.csv".to_string()),
        });

        let facturx_result = convert_to(&invoice, OutputFormat::FacturX).unwrap();
        assert!(facturx_result.is_pdf(), "Doit être un PDF");

        let parsed = pdp_invoice::facturx::FacturXParser::new()
            .parse(&facturx_result.content)
            .expect("Re-parsing Factur-X échoué");

        assert!(
            !parsed.attachments.iter().any(|a| {
                a.filename.as_deref().map(|f| f.to_lowercase()) == Some("factur-x.xml".to_string())
            }),
            "factur-x.xml ne doit PAS apparaître dans les pièces jointes"
        );

        let filenames: Vec<&str> = parsed.attachments.iter()
            .filter_map(|a| a.filename.as_deref())
            .collect();
        assert!(filenames.contains(&"detail.csv"),
            "Doit contenir detail.csv, trouvé: {:?}", filenames);

        // Vérifier que le contenu CSV est préservé
        let csv_att = parsed.attachments.iter()
            .find(|a| a.filename.as_deref() == Some("detail.csv"))
            .expect("PJ detail.csv introuvable");
        let content = csv_att.embedded_content.as_ref().expect("Contenu CSV manquant");
        let text = String::from_utf8_lossy(content);
        assert!(text.contains("A001;10;25.00"), "Contenu CSV doit être préservé, trouvé: {}", text);
    }

    #[test]
    fn test_facturx_to_cii_preserves_attachments() {
        // 1. Créer un Factur-X avec PJ
        let mut invoice = parse_cii(CII_FIXTURE);
        invoice.attachments.push(pdp_core::model::InvoiceAttachment {
            id: Some("PJ-001".to_string()),
            description: Some("Bordereau".to_string()),
            external_uri: None,
            embedded_content: Some(b"bordereau content".to_vec()),
            mime_code: Some("application/octet-stream".to_string()),
            filename: Some("bordereau.bin".to_string()),
        });

        let facturx_result = convert_to(&invoice, OutputFormat::FacturX).unwrap();
        assert!(facturx_result.is_pdf());

        // 2. Parser le Factur-X
        let parsed = pdp_invoice::facturx::FacturXParser::new()
            .parse(&facturx_result.content)
            .expect("Re-parsing Factur-X échoué");

        // 3. Convertir en CII — les PJ doivent être dans le modèle
        assert!(!parsed.attachments.is_empty(),
            "Le Factur-X parsé doit contenir des PJ");
        assert!(
            parsed.attachments.iter().any(|a| a.filename.as_deref() == Some("bordereau.bin")),
            "Doit contenir bordereau.bin"
        );
        assert!(
            !parsed.attachments.iter().any(|a| {
                a.filename.as_deref().map(|f| f.to_lowercase()) == Some("factur-x.xml".to_string())
            }),
            "factur-x.xml ne doit PAS être dans les PJ"
        );
    }

    // --- PDF (visuel seul, sans Factur-X) ---

    /// Vérifie qu'un ConversionResult est un PDF valide sans XML Factur-X embarqué
    fn assert_valid_pdf_no_facturx(result: &ConversionResult, context: &str) {
        assert_eq!(result.output_format, OutputFormat::PDF, "{context}: format doit être PDF");
        assert!(result.is_pdf(), "{context}: le contenu doit commencer par %PDF-");
        assert!(result.content.len() > 1000, "{context}: PDF trop petit ({} octets)", result.content.len());
        assert!(result.suggested_filename.ends_with(".pdf"), "{context}: extension .pdf attendue");
        assert!(!result.suggested_filename.contains("facturx"), "{context}: ne doit pas contenir 'facturx'");

        // Vérifier que le PDF est parseable par lopdf
        let doc = lopdf::Document::load_mem(&result.content)
            .unwrap_or_else(|e| panic!("{context}: PDF invalide (lopdf): {e}"));

        // Vérifier qu'il n'y a PAS de factur-x.xml embarqué (c'est un PDF seul)
        let objects_debug = format!("{:?}", doc.objects);
        assert!(!objects_debug.contains("factur-x.xml"),
            "{context}: le PDF ne doit PAS contenir factur-x.xml (c'est un PDF seul, pas Factur-X)");

        // Vérifier qu'il y a au moins une page
        let pages = doc.get_pages();
        assert!(!pages.is_empty(), "{context}: le PDF doit contenir au moins une page");

        println!("[PDF OK] {context}: {} octets, {} page(s), fichier: {}",
            result.content.len(), pages.len(), result.suggested_filename);
    }

    // --- UBL → PDF (toutes les fixtures) ---

    #[test]
    fn test_ubl_to_pdf_facture_001() {
        let invoice = parse_ubl("../../tests/fixtures/ubl/facture_ubl_001.xml");
        let result = convert_to(&invoice, OutputFormat::PDF).unwrap();
        assert_valid_pdf_no_facturx(&result, "UBL facture_001");
    }

    #[test]
    fn test_ubl_to_pdf_avoir() {
        let invoice = parse_ubl("../../tests/fixtures/ubl/facture_ubl_002_avoir.xml");
        let result = convert_to(&invoice, OutputFormat::PDF).unwrap();
        assert_valid_pdf_no_facturx(&result, "UBL avoir");
    }

    #[test]
    fn test_ubl_to_pdf_rectificative() {
        let invoice = parse_ubl("../../tests/fixtures/ubl/facture_rectificative_ubl_384.xml");
        let result = convert_to(&invoice, OutputFormat::PDF).unwrap();
        assert_valid_pdf_no_facturx(&result, "UBL rectificative 384");
    }

    #[test]
    fn test_ubl_to_pdf_autofacture() {
        let invoice = parse_ubl("../../tests/fixtures/ubl/autofacture_ubl_389.xml");
        let result = convert_to(&invoice, OutputFormat::PDF).unwrap();
        assert_valid_pdf_no_facturx(&result, "UBL autofacture 389");
    }

    #[test]
    fn test_ubl_to_pdf_acompte() {
        let invoice = parse_ubl("../../tests/fixtures/ubl/facture_ubl_acompte_386.xml");
        let result = convert_to(&invoice, OutputFormat::PDF).unwrap();
        assert_valid_pdf_no_facturx(&result, "UBL acompte 386");
    }

    #[test]
    fn test_ubl_to_pdf_definitive_apres_acompte() {
        let invoice = parse_ubl("../../tests/fixtures/ubl/facture_ubl_definitive_apres_acompte.xml");
        let result = convert_to(&invoice, OutputFormat::PDF).unwrap();
        assert_valid_pdf_no_facturx(&result, "UBL définitive après acompte");
    }

    #[test]
    fn test_ubl_to_pdf_remises_multitva() {
        let invoice = parse_ubl("../../tests/fixtures/ubl/facture_ubl_remises_multitva.xml");
        let result = convert_to(&invoice, OutputFormat::PDF).unwrap();
        assert_valid_pdf_no_facturx(&result, "UBL remises multi-TVA");
    }

    #[test]
    fn test_ubl_to_pdf_marketplace() {
        let invoice = parse_ubl("../../tests/fixtures/ubl/facture_ubl_marketplace_a8.xml");
        let result = convert_to(&invoice, OutputFormat::PDF).unwrap();
        assert_valid_pdf_no_facturx(&result, "UBL marketplace A8");
    }

    #[test]
    fn test_ubl_to_pdf_soustraitance() {
        let invoice = parse_ubl("../../tests/fixtures/ubl/facture_ubl_soustraitance_a4.xml");
        let result = convert_to(&invoice, OutputFormat::PDF).unwrap();
        assert_valid_pdf_no_facturx(&result, "UBL sous-traitance A4");
    }

    #[test]
    fn test_ubl_to_pdf_delegation() {
        let invoice = parse_ubl("../../tests/fixtures/ubl/facture_ubl_delegation_s8.xml");
        let result = convert_to(&invoice, OutputFormat::PDF).unwrap();
        assert_valid_pdf_no_facturx(&result, "UBL délégation S8");
    }

    #[test]
    fn test_ubl_to_pdf_multivendeurs() {
        let invoice = parse_ubl("../../tests/fixtures/ubl/facture_ubl_multivendeurs_b8.xml");
        let result = convert_to(&invoice, OutputFormat::PDF).unwrap();
        assert_valid_pdf_no_facturx(&result, "UBL multi-vendeurs B8");
    }

    #[test]
    fn test_ubl_to_pdf_tax_representative() {
        let invoice = parse_ubl("../../tests/fixtures/ubl/facture_ubl_tax_representative.xml");
        let result = convert_to(&invoice, OutputFormat::PDF).unwrap();
        assert_valid_pdf_no_facturx(&result, "UBL représentant fiscal");
    }

    // --- CII → PDF (toutes les fixtures) ---

    #[test]
    fn test_cii_to_pdf_facture_001() {
        let invoice = parse_cii("../../tests/fixtures/cii/facture_cii_001.xml");
        let result = convert_to(&invoice, OutputFormat::PDF).unwrap();
        assert_valid_pdf_no_facturx(&result, "CII facture_001");
    }

    #[test]
    fn test_cii_to_pdf_avoir() {
        let invoice = parse_cii("../../tests/fixtures/cii/avoir_cii_381.xml");
        let result = convert_to(&invoice, OutputFormat::PDF).unwrap();
        assert_valid_pdf_no_facturx(&result, "CII avoir 381");
    }

    #[test]
    fn test_cii_to_pdf_rectificative() {
        let invoice = parse_cii("../../tests/fixtures/cii/facture_rectificative_cii_384.xml");
        let result = convert_to(&invoice, OutputFormat::PDF).unwrap();
        assert_valid_pdf_no_facturx(&result, "CII rectificative 384");
    }

    #[test]
    fn test_cii_to_pdf_autofacture() {
        let invoice = parse_cii("../../tests/fixtures/cii/autofacture_cii_389.xml");
        let result = convert_to(&invoice, OutputFormat::PDF).unwrap();
        assert_valid_pdf_no_facturx(&result, "CII autofacture 389");
    }

    #[test]
    fn test_cii_to_pdf_acompte() {
        let invoice = parse_cii("../../tests/fixtures/cii/facture_cii_acompte.xml");
        let result = convert_to(&invoice, OutputFormat::PDF).unwrap();
        assert_valid_pdf_no_facturx(&result, "CII acompte");
    }

    #[test]
    fn test_cii_to_pdf_definitive_apres_acompte() {
        let invoice = parse_cii("../../tests/fixtures/cii/facture_cii_definitive_apres_acompte.xml");
        let result = convert_to(&invoice, OutputFormat::PDF).unwrap();
        assert_valid_pdf_no_facturx(&result, "CII définitive après acompte");
    }

    #[test]
    fn test_cii_to_pdf_remises_multitva() {
        let invoice = parse_cii("../../tests/fixtures/cii/facture_cii_remises_multitva.xml");
        let result = convert_to(&invoice, OutputFormat::PDF).unwrap();
        assert_valid_pdf_no_facturx(&result, "CII remises multi-TVA");
    }

    #[test]
    fn test_cii_to_pdf_marketplace() {
        let invoice = parse_cii("../../tests/fixtures/cii/facture_cii_marketplace_a8.xml");
        let result = convert_to(&invoice, OutputFormat::PDF).unwrap();
        assert_valid_pdf_no_facturx(&result, "CII marketplace A8");
    }

    #[test]
    fn test_cii_to_pdf_soustraitance() {
        let invoice = parse_cii("../../tests/fixtures/cii/facture_cii_soustraitance_a4.xml");
        let result = convert_to(&invoice, OutputFormat::PDF).unwrap();
        assert_valid_pdf_no_facturx(&result, "CII sous-traitance A4");
    }

    #[test]
    fn test_cii_to_pdf_delegation() {
        let invoice = parse_cii("../../tests/fixtures/cii/facture_cii_delegation_s8.xml");
        let result = convert_to(&invoice, OutputFormat::PDF).unwrap();
        assert_valid_pdf_no_facturx(&result, "CII délégation S8");
    }

    #[test]
    fn test_cii_to_pdf_multivendeurs() {
        let invoice = parse_cii("../../tests/fixtures/cii/facture_cii_multivendeurs_b8.xml");
        let result = convert_to(&invoice, OutputFormat::PDF).unwrap();
        assert_valid_pdf_no_facturx(&result, "CII multi-vendeurs B8");
    }

    // --- Factur-X → PDF (retourne le PDF existant, sans re-génération) ---

    #[test]
    fn test_facturx_to_pdf_returns_existing() {
        let facturx_path = "../../output/facturx_cii_extended.pdf";
        if !std::path::Path::new(facturx_path).exists() {
            eprintln!("Factur-X PDF non disponible, test ignoré");
            return;
        }
        let invoice = parse_facturx(facturx_path);
        let original_pdf = invoice.raw_pdf.as_ref().expect("raw_pdf doit exister");

        let result = convert_to(&invoice, OutputFormat::PDF).unwrap();
        assert_eq!(result.output_format, OutputFormat::PDF);
        assert!(result.is_pdf());
        // Le PDF retourné doit être identique au PDF source
        assert_eq!(result.content.len(), original_pdf.len(),
            "Le PDF retourné doit être identique au PDF source");
        assert_eq!(&result.content, original_pdf,
            "Le contenu PDF doit être identique octet par octet");
    }

    // --- PDF : vérification as_string() retourne None ---

    #[test]
    fn test_pdf_as_string_returns_none() {
        let invoice = parse_ubl(UBL_FIXTURE);
        let result = convert_to(&invoice, OutputFormat::PDF).unwrap();
        assert!(result.as_string().is_none(),
            "as_string() doit retourner None pour un PDF");
    }

    // --- Formats supportés ---

    #[test]
    fn test_supported_output_formats() {
        let ubl_formats = supported_output_formats(&InvoiceFormat::UBL);
        assert_eq!(ubl_formats.len(), 3);
        assert!(ubl_formats.contains(&OutputFormat::CII));
        assert!(ubl_formats.contains(&OutputFormat::FacturX));
        assert!(ubl_formats.contains(&OutputFormat::PDF));

        let cii_formats = supported_output_formats(&InvoiceFormat::CII);
        assert_eq!(cii_formats.len(), 3);
        assert!(cii_formats.contains(&OutputFormat::UBL));

        let fx_formats = supported_output_formats(&InvoiceFormat::FacturX);
        assert_eq!(fx_formats.len(), 3);
        assert!(fx_formats.contains(&OutputFormat::CII));
        assert!(fx_formats.contains(&OutputFormat::UBL));
        assert!(fx_formats.contains(&OutputFormat::PDF));
    }

    // --- Compatibilité ascendante ---

    #[test]
    fn test_convert_backward_compat() {
        let invoice = parse_ubl(UBL_FIXTURE);
        let result = convert(&invoice, InvoiceFormat::CII).unwrap();
        assert_eq!(result.target_format(), InvoiceFormat::CII);
        assert!(result.as_string().unwrap().contains("CrossIndustryInvoice"));
    }

    // --- Export avec pièces jointes ---

    #[test]
    #[ignore] // cargo test -p pdp-transform -- export_conversions_with_attachments --ignored --nocapture
    fn export_conversions_with_attachments() {
        let out_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../output");
        std::fs::create_dir_all(&out_dir).expect("Impossible de créer output/");

        // Pièces jointes de test (PDF, PNG, CSV)
        let attachments = vec![
            pdp_core::model::InvoiceAttachment {
                id: Some("ATT-PDF-001".to_string()),
                description: Some("Bon de commande".to_string()),
                external_uri: None,
                embedded_content: Some(b"%PDF-1.4 fake bon de commande pour test".to_vec()),
                mime_code: Some("application/pdf".to_string()),
                filename: Some("bon_commande.pdf".to_string()),
            },
            pdp_core::model::InvoiceAttachment {
                id: Some("ATT-PNG-002".to_string()),
                description: Some("Photo chantier".to_string()),
                external_uri: None,
                embedded_content: Some(vec![
                    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A,
                    0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
                ]),
                mime_code: Some("image/png".to_string()),
                filename: Some("photo_chantier.png".to_string()),
            },
            pdp_core::model::InvoiceAttachment {
                id: Some("ATT-CSV-003".to_string()),
                description: Some("Détail des lignes".to_string()),
                external_uri: None,
                embedded_content: Some(b"ref;qte;pu;total\nA001;10;25.00;250.00\nA002;5;12.50;62.50\nA003;3;8.75;26.25".to_vec()),
                mime_code: Some("text/csv".to_string()),
                filename: Some("detail_lignes.csv".to_string()),
            },
        ];

        println!("\n=== Export conversions avec pièces jointes ({} PJ) ===\n", attachments.len());

        // --- 1. UBL + PJ → CII ---
        let mut ubl_invoice = parse_ubl(UBL_FIXTURE);
        ubl_invoice.attachments = attachments.clone();
        // Injecter les PJ dans le raw_xml UBL pour que le XSLT les transforme
        let raw_ubl = ubl_invoice.raw_xml.as_ref().unwrap().clone();
        let mut pj_ubl_xml = String::new();
        for att in &attachments {
            pj_ubl_xml.push_str("  <cac:AdditionalDocumentReference>\n");
            pj_ubl_xml.push_str(&format!("    <cbc:ID>{}</cbc:ID>\n", att.id.as_deref().unwrap_or("ATT")));
            if let Some(ref desc) = att.description {
                pj_ubl_xml.push_str(&format!("    <cbc:DocumentDescription>{}</cbc:DocumentDescription>\n", desc));
            }
            pj_ubl_xml.push_str("    <cac:Attachment>\n");
            if let Some(ref content) = att.embedded_content {
                let b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, content);
                let mime = att.mime_code.as_deref().unwrap_or("application/octet-stream");
                let fname = att.filename.as_deref().unwrap_or("attachment.bin");
                pj_ubl_xml.push_str(&format!(
                    "      <cbc:EmbeddedDocumentBinaryObject mimeCode=\"{}\" filename=\"{}\">{}</cbc:EmbeddedDocumentBinaryObject>\n",
                    mime, fname, b64
                ));
            }
            pj_ubl_xml.push_str("    </cac:Attachment>\n");
            pj_ubl_xml.push_str("  </cac:AdditionalDocumentReference>\n");
        }
        ubl_invoice.raw_xml = Some(raw_ubl.replace("</Invoice>", &format!("{}</Invoice>", pj_ubl_xml)));

        let cii_from_ubl = convert_to(&ubl_invoice, OutputFormat::CII).unwrap();
        let path = out_dir.join("ubl_with_pj_to_cii.xml");
        std::fs::write(&path, &cii_from_ubl.content).unwrap();
        println!("  UBL+PJ → CII : {} ({} Ko)", path.display(), cii_from_ubl.content.len() / 1024);

        // --- 2. UBL + PJ → Factur-X ---
        let facturx_from_ubl = convert_to(&ubl_invoice, OutputFormat::FacturX).unwrap();
        let path = out_dir.join("ubl_with_pj_to_facturx.pdf");
        std::fs::write(&path, &facturx_from_ubl.content).unwrap();
        println!("  UBL+PJ → Factur-X : {} ({} Ko)", path.display(), facturx_from_ubl.content.len() / 1024);

        // --- 3. UBL + PJ → PDF ---
        let pdf_from_ubl = convert_to(&ubl_invoice, OutputFormat::PDF).unwrap();
        let path = out_dir.join("ubl_with_pj_to_pdf.pdf");
        std::fs::write(&path, &pdf_from_ubl.content).unwrap();
        println!("  UBL+PJ → PDF : {} ({} Ko)", path.display(), pdf_from_ubl.content.len() / 1024);

        // --- 4. CII + PJ → UBL ---
        let mut cii_invoice = parse_cii(CII_FIXTURE);
        cii_invoice.attachments = attachments.clone();
        let raw_cii = cii_invoice.raw_xml.as_ref().unwrap().clone();
        let mut pj_cii_xml = String::new();
        for att in &attachments {
            // Ordre XSD : IssuerAssignedID, URIID, LineID, TypeCode, Name, AttachmentBinaryObject
            pj_cii_xml.push_str("        <ram:AdditionalReferencedDocument>\n");
            pj_cii_xml.push_str(&format!("          <ram:IssuerAssignedID>{}</ram:IssuerAssignedID>\n",
                att.id.as_deref().unwrap_or("ATT")));
            if let Some(ref uri) = att.external_uri {
                pj_cii_xml.push_str(&format!("          <ram:URIID>{}</ram:URIID>\n", uri));
            }
            pj_cii_xml.push_str("          <ram:TypeCode>916</ram:TypeCode>\n");
            if let Some(ref desc) = att.description {
                pj_cii_xml.push_str(&format!("          <ram:Name>{}</ram:Name>\n", desc));
            }
            if let Some(ref content) = att.embedded_content {
                let b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, content);
                let mime = att.mime_code.as_deref().unwrap_or("application/octet-stream");
                let fname = att.filename.as_deref().unwrap_or("attachment.bin");
                pj_cii_xml.push_str(&format!(
                    "          <ram:AttachmentBinaryObject mimeCode=\"{}\" filename=\"{}\">{}</ram:AttachmentBinaryObject>\n",
                    mime, fname, b64
                ));
            }
            pj_cii_xml.push_str("        </ram:AdditionalReferencedDocument>\n");
        }
        cii_invoice.raw_xml = Some(raw_cii.replace(
            "</ram:ApplicableHeaderTradeAgreement>",
            &format!("{}</ram:ApplicableHeaderTradeAgreement>", pj_cii_xml)
        ));

        let ubl_from_cii = convert_to(&cii_invoice, OutputFormat::UBL).unwrap();
        let path = out_dir.join("cii_with_pj_to_ubl.xml");
        std::fs::write(&path, &ubl_from_cii.content).unwrap();
        println!("  CII+PJ → UBL : {} ({} Ko)", path.display(), ubl_from_cii.content.len() / 1024);

        // --- 5. CII + PJ → Factur-X ---
        let facturx_from_cii = convert_to(&cii_invoice, OutputFormat::FacturX).unwrap();
        let path = out_dir.join("cii_with_pj_to_facturx.pdf");
        std::fs::write(&path, &facturx_from_cii.content).unwrap();
        println!("  CII+PJ → Factur-X : {} ({} Ko)", path.display(), facturx_from_cii.content.len() / 1024);

        // --- 6. CII + PJ → PDF ---
        let pdf_from_cii = convert_to(&cii_invoice, OutputFormat::PDF).unwrap();
        let path = out_dir.join("cii_with_pj_to_pdf.pdf");
        std::fs::write(&path, &pdf_from_cii.content).unwrap();
        println!("  CII+PJ → PDF : {} ({} Ko)", path.display(), pdf_from_cii.content.len() / 1024);

        // --- 7. Factur-X (avec PJ) → CII ---
        if facturx_from_cii.is_pdf() {
            let fx_invoice = pdp_invoice::facturx::FacturXParser::new()
                .parse(&facturx_from_cii.content)
                .expect("Re-parsing Factur-X échoué");
            println!("  Factur-X parsé : {} PJ extraites (hors factur-x.xml)", fx_invoice.attachments.len());

            let cii_from_fx = convert_to(&fx_invoice, OutputFormat::CII).unwrap();
            let path = out_dir.join("facturx_with_pj_to_cii.xml");
            std::fs::write(&path, &cii_from_fx.content).unwrap();
            println!("  Factur-X+PJ → CII : {} ({} Ko)", path.display(), cii_from_fx.content.len() / 1024);

            // --- 8. Factur-X (avec PJ) → UBL ---
            let ubl_from_fx = convert_to(&fx_invoice, OutputFormat::UBL).unwrap();
            let path = out_dir.join("facturx_with_pj_to_ubl.xml");
            std::fs::write(&path, &ubl_from_fx.content).unwrap();
            println!("  Factur-X+PJ → UBL : {} ({} Ko)", path.display(), ubl_from_fx.content.len() / 1024);
        }

        println!("\n=== {} fichiers générés dans {} ===", 8, out_dir.display());
    }

    // --- Helpers ---

    fn result_as_string(result: &ConversionResult) -> String {
        String::from_utf8(result.content.clone()).unwrap()
    }
}
