//! Générateur Factur-X : PDF/A-3 avec XML CII embarqué et pièces jointes.
//!
//! Pipeline complète :
//! 1. Génère le XML CII (si source UBL, via ubl_to_cii)
//! 2. Génère le PDF visuel via FOP (CII/UBL → XR → FO → PDF)
//! 3. Embarque le XML CII comme `factur-x.xml` (AF relationship) dans le PDF
//! 4. Embarque les pièces jointes additionnelles (BG-24)
//! 5. Ajoute les métadonnées XMP Factur-X (PDF/A-3a)
//!
//! Utilise lopdf pour la manipulation PDF post-FOP.

use pdp_core::error::{PdpError, PdpResult};
use pdp_core::model::{InvoiceAttachment, InvoiceData, InvoiceFormat};

use crate::fop_engine::{FopEngine, SourceSyntax};

/// Niveau de conformité Factur-X
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FacturXLevel {
    Minimum,
    BasicWL,
    Basic,
    EN16931,
    Extended,
}

impl FacturXLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            FacturXLevel::Minimum => "MINIMUM",
            FacturXLevel::BasicWL => "BASIC WL",
            FacturXLevel::Basic => "BASIC",
            FacturXLevel::EN16931 => "EN 16931",
            FacturXLevel::Extended => "EXTENDED",
        }
    }
}

impl std::fmt::Display for FacturXLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Résultat de la génération Factur-X
pub struct FacturXResult {
    /// PDF/A-3 avec XML embarqué
    pub pdf: Vec<u8>,
    /// XML CII embarqué (factur-x.xml)
    pub cii_xml: String,
    /// Nom de fichier suggéré
    pub filename: String,
    /// Niveau Factur-X
    pub level: FacturXLevel,
}

/// Générateur Factur-X complet.
pub struct FacturXGenerator {
    fop_engine: FopEngine,
    typst_engine: crate::typst_engine::TypstPdfEngine,
    level: FacturXLevel,
}

impl FacturXGenerator {
    pub fn new(fop_engine: FopEngine) -> Self {
        let specs_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../specs");
        Self {
            fop_engine,
            typst_engine: crate::typst_engine::TypstPdfEngine::new(specs_dir),
            level: FacturXLevel::EN16931,
        }
    }

    pub fn from_specs_dir(specs_dir: &std::path::Path) -> Self {
        Self {
            fop_engine: FopEngine::new(specs_dir),
            typst_engine: crate::typst_engine::TypstPdfEngine::new(specs_dir.to_path_buf()),
            level: FacturXLevel::EN16931,
        }
    }

    pub fn from_manifest_dir() -> Self {
        Self::new(FopEngine::from_manifest_dir())
    }

    pub fn with_level(mut self, level: FacturXLevel) -> Self {
        self.level = level;
        self
    }

    /// Génère un Factur-X (PDF/A-3) à partir d'une InvoiceData.
    ///
    /// - Si source = CII : utilise le raw_xml directement comme factur-x.xml
    /// - Si source = UBL : convertit d'abord en CII via ubl_to_cii
    /// - Si source = FacturX : retourne le raw_pdf existant
    ///
    /// Le PDF visuel est généré via la pipeline FOP (CII/UBL → XR → FO → PDF),
    /// puis le XML CII et les pièces jointes sont embarqués via lopdf.
    pub fn generate(&self, invoice: &InvoiceData) -> PdpResult<FacturXResult> {
        // Si on a déjà un PDF Factur-X, le retourner tel quel
        if invoice.source_format == InvoiceFormat::FacturX {
            if let Some(ref pdf) = invoice.raw_pdf {
                let cii_xml = invoice.raw_xml.clone().unwrap_or_default();
                return Ok(FacturXResult {
                    pdf: pdf.clone(),
                    cii_xml,
                    filename: self.make_filename(&invoice.invoice_number),
                    level: self.level,
                });
            }
        }

        // Obtenir le XML source brut
        let raw_xml = invoice.raw_xml.as_deref().ok_or_else(|| {
            PdpError::TransformError {
                source_format: invoice.source_format.to_string(),
                target_format: "Factur-X".to_string(),
                message: "Pas de XML brut disponible pour la génération Factur-X".to_string(),
            }
        })?;

        // Déterminer la syntaxe source et obtenir le XML CII pour l'embarquement
        let (source_syntax, cii_xml) = match invoice.source_format {
            InvoiceFormat::CII => (SourceSyntax::CII, raw_xml.to_string()),
            InvoiceFormat::UBL => {
                let engine = crate::xslt_engine::XsltEngine::from_manifest_dir();
                let cii = engine.ubl_to_cii(raw_xml)?;
                // Détecter UBL CreditNote pour le pipeline FOP
                let syntax = if raw_xml.contains("<CreditNote") {
                    SourceSyntax::UBLCreditNote
                } else {
                    SourceSyntax::UBL
                };
                (syntax, cii)
            }
            InvoiceFormat::FacturX => {
                // Pas de raw_pdf (déjà géré ci-dessus), on a juste le XML
                (SourceSyntax::CII, raw_xml.to_string())
            }
        };

        // Étape 1 : Générer le PDF visuel (Typst prioritaire, FOP Java fallback)
        let base_pdf = match self.typst_engine.generate_pdf(invoice) {
            Ok(pdf) => {
                tracing::info!(
                    invoice = %invoice.invoice_number,
                    "PDF visuel généré via Typst"
                );
                pdf
            }
            Err(e) => {
                tracing::warn!(
                    invoice = %invoice.invoice_number,
                    error = %e,
                    "Typst échoué, fallback FOP Java"
                );
                self.fop_engine.generate_pdf(raw_xml, source_syntax)?
            }
        };

        // Détecter le niveau Factur-X depuis le profile_id du XML CII
        let effective_level = Self::detect_level_from_xml(&cii_xml).unwrap_or(self.level);

        // Étape 2 : Embarquer le XML CII + pièces jointes dans le PDF
        let final_pdf = self.embed_in_pdf(
            &base_pdf,
            &cii_xml,
            &invoice.attachments,
            &invoice.invoice_number,
            effective_level,
        )?;

        Ok(FacturXResult {
            pdf: final_pdf,
            cii_xml,
            filename: self.make_filename(&invoice.invoice_number),
            level: effective_level,
        })
    }

    /// Détecte le niveau Factur-X depuis le profile_id dans le XML CII.
    fn detect_level_from_xml(cii_xml: &str) -> Option<FacturXLevel> {
        // Chercher le GuidelineSpecifiedDocumentContextParameter > ID
        // Ordre important : les profils spécifiques avant EN16931 (qui est un préfixe commun)
        if cii_xml.contains("urn:factur-x.eu:1p0:extended") {
            Some(FacturXLevel::Extended)
        } else if cii_xml.contains("urn:factur-x.eu:1p0:basicwl") {
            Some(FacturXLevel::BasicWL)
        } else if cii_xml.contains("urn:factur-x.eu:1p0:basic") {
            Some(FacturXLevel::Basic)
        } else if cii_xml.contains("urn:factur-x.eu:1p0:minimum") {
            Some(FacturXLevel::Minimum)
        } else if cii_xml.contains("urn:cen.eu:en16931:2017") {
            Some(FacturXLevel::EN16931)
        } else {
            None
        }
    }

    /// Embarque le XML CII et les pièces jointes dans un PDF existant.
    /// Ajoute les métadonnées XMP Factur-X pour la conformité PDF/A-3.
    fn embed_in_pdf(
        &self,
        pdf_bytes: &[u8],
        cii_xml: &str,
        attachments: &[InvoiceAttachment],
        invoice_number: &str,
        level: FacturXLevel,
    ) -> PdpResult<Vec<u8>> {
        use lopdf::{Document, Object, Stream, StringFormat};
        use lopdf::dictionary;

        let mut doc = Document::load_mem(pdf_bytes).map_err(|e| {
            PdpError::TransformError {
                source_format: "PDF".to_string(),
                target_format: "Factur-X".to_string(),
                message: format!("Impossible de charger le PDF FOP: {}", e),
            }
        })?;

        // --- 1. Embarquer factur-x.xml ---
        let xml_bytes = cii_xml.as_bytes().to_vec();
        let xml_stream = Stream::new(
            dictionary! {
                "Type" => "EmbeddedFile",
                "Subtype" => Object::Name("text/xml".into()),
                "Params" => dictionary! {
                    "Size" => Object::Integer(xml_bytes.len() as i64),
                },
            },
            xml_bytes,
        );
        let xml_stream_id = doc.add_object(xml_stream);

        let xml_filespec = dictionary! {
            "Type" => "Filespec",
            "F" => Object::String("factur-x.xml".into(), StringFormat::Literal),
            "UF" => Object::String("factur-x.xml".into(), StringFormat::Hexadecimal),
            "Desc" => Object::String("Factur-X XML invoice".into(), StringFormat::Literal),
            "AFRelationship" => Object::Name("Data".into()),
            "EF" => dictionary! {
                "F" => Object::Reference(xml_stream_id),
                "UF" => Object::Reference(xml_stream_id),
            },
        };
        let xml_filespec_id = doc.add_object(xml_filespec);

        // Collecter tous les filespec IDs pour le Names/EmbeddedFiles et AF
        let mut filespec_ids = vec![xml_filespec_id];

        // --- 2. Embarquer les pièces jointes additionnelles (BG-24) ---
        for attachment in attachments {
            if let Some(ref content) = attachment.embedded_content {
                let filename = attachment.filename.as_deref()
                    .or(attachment.id.as_deref())
                    .unwrap_or("attachment.bin");
                let mime = attachment.mime_code.as_deref().unwrap_or("application/octet-stream");
                let desc = attachment.description.as_deref().unwrap_or("");

                let att_stream = Stream::new(
                    dictionary! {
                        "Type" => "EmbeddedFile",
                        "Subtype" => Object::Name(mime.into()),
                        "Params" => dictionary! {
                            "Size" => Object::Integer(content.len() as i64),
                        },
                    },
                    content.clone(),
                );
                let att_stream_id = doc.add_object(att_stream);

                let att_filespec = dictionary! {
                    "Type" => "Filespec",
                    "F" => Object::String(filename.into(), StringFormat::Literal),
                    "UF" => Object::String(filename.into(), StringFormat::Hexadecimal),
                    "Desc" => Object::String(desc.into(), StringFormat::Literal),
                    "AFRelationship" => Object::Name("Supplement".into()),
                    "EF" => dictionary! {
                        "F" => Object::Reference(att_stream_id),
                        "UF" => Object::Reference(att_stream_id),
                    },
                };
                let att_filespec_id = doc.add_object(att_filespec);
                filespec_ids.push(att_filespec_id);
            }
        }

        // --- 3. Ajouter Names/EmbeddedFiles au catalogue ---
        let names_array: Vec<Object> = {
            let mut arr = Vec::new();
            // factur-x.xml en premier
            arr.push(Object::String("factur-x.xml".into(), StringFormat::Literal));
            arr.push(Object::Reference(xml_filespec_id));
            // Pièces jointes (seules celles avec embedded_content ont un filespec)
            let mut filespec_idx = 1; // 0 = factur-x.xml
            for att in attachments.iter() {
                if att.embedded_content.is_some() {
                    let filename = att.filename.as_deref()
                        .or(att.id.as_deref())
                        .unwrap_or("attachment.bin");
                    arr.push(Object::String(filename.into(), StringFormat::Literal));
                    arr.push(Object::Reference(filespec_ids[filespec_idx]));
                    filespec_idx += 1;
                }
            }
            arr
        };

        let ef_name_tree = dictionary! {
            "Names" => Object::Array(names_array),
        };
        let ef_name_tree_id = doc.add_object(ef_name_tree);

        let names_dict = dictionary! {
            "EmbeddedFiles" => Object::Reference(ef_name_tree_id),
        };
        let names_dict_id = doc.add_object(names_dict);

        // AF array (Associated Files)
        let af_array: Vec<Object> = filespec_ids.iter()
            .map(|id| Object::Reference(*id))
            .collect();

        // --- 4. Ajouter les métadonnées XMP Factur-X ---
        let xmp = self.build_xmp_metadata(invoice_number, level);
        let xmp_stream = Stream::new(
            dictionary! {
                "Type" => "Metadata",
                "Subtype" => Object::Name("XML".into()),
                "Length" => Object::Integer(xmp.len() as i64),
            },
            xmp.into_bytes(),
        );
        let xmp_id = doc.add_object(xmp_stream);

        // Mettre à jour le catalogue (Root)
        let catalog_id = doc.trailer.get(b"Root")
            .and_then(|r| r.as_reference())
            .map_err(|e| PdpError::TransformError {
                source_format: "PDF".to_string(),
                target_format: "Factur-X".to_string(),
                message: format!("Catalogue PDF introuvable: {}", e),
            })?;

        if let Ok(catalog) = doc.get_object_mut(catalog_id) {
            if let Object::Dictionary(ref mut dict) = catalog {
                dict.set("Names", Object::Reference(names_dict_id));
                dict.set("AF", Object::Array(af_array));
                // MarkInfo pour PDF/A
                let mark_info = dictionary! { "Marked" => Object::Boolean(true) };
                dict.set("MarkInfo", mark_info);
                dict.set("Metadata", Object::Reference(xmp_id));
            }
        }

        // --- 5. Sérialiser le PDF ---
        let mut output = Vec::new();
        doc.save_to(&mut output).map_err(|e| {
            PdpError::TransformError {
                source_format: "PDF".to_string(),
                target_format: "Factur-X".to_string(),
                message: format!("Impossible de sérialiser le PDF Factur-X: {}", e),
            }
        })?;

        // --- 6. Corriger le header binaire pour PDF/A-3 ---
        // lopdf réécrit le header sans le commentaire binaire requis par PDF/A.
        // On insère le commentaire binaire après %PDF-1.x\n, puis on re-charge
        // et re-sérialise avec lopdf pour que les offsets xref soient recalculés.
        output = Self::inject_binary_header(output);

        tracing::info!(
            invoice = %invoice_number,
            level = %self.level,
            attachments = attachments.len(),
            pdf_size = output.len(),
            "PDF Factur-X généré avec XML embarqué et {} pièce(s) jointe(s)",
            attachments.iter().filter(|a| a.embedded_content.is_some()).count()
        );

        Ok(output)
    }

    /// Construit les métadonnées XMP pour PDF/A-3a + Factur-X.
    fn build_xmp_metadata(&self, invoice_number: &str, level: FacturXLevel) -> String {
        let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S+00:00");
        let level = level.as_str();

        format!(
r#"<?xpacket begin="" id="W5M0MpCehiHzreSzNTczkc9d"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/">
  <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
    <rdf:Description rdf:about=""
        xmlns:dc="http://purl.org/dc/elements/1.1/"
        xmlns:xmp="http://ns.adobe.com/xap/1.0/"
        xmlns:pdf="http://ns.adobe.com/pdf/1.3/"
        xmlns:pdfaid="http://www.aiim.org/pdfa/ns/id/"
        xmlns:pdfaExtension="http://www.aiim.org/pdfa/ns/extension/"
        xmlns:pdfaSchema="http://www.aiim.org/pdfa/ns/schema#"
        xmlns:pdfaProperty="http://www.aiim.org/pdfa/ns/property#"
        xmlns:fx="urn:factur-x:pdfa:CrossIndustryDocument:invoice:1p0#">
      <dc:title>
        <rdf:Alt>
          <rdf:li xml:lang="x-default">{invoice_number}</rdf:li>
        </rdf:Alt>
      </dc:title>
      <dc:creator>
        <rdf:Seq>
          <rdf:li>pdp-facture</rdf:li>
        </rdf:Seq>
      </dc:creator>
      <dc:description>
        <rdf:Alt>
          <rdf:li xml:lang="x-default">Factur-X {level}</rdf:li>
        </rdf:Alt>
      </dc:description>
      <xmp:CreateDate>{now}</xmp:CreateDate>
      <xmp:ModifyDate>{now}</xmp:ModifyDate>
      <xmp:CreatorTool>pdp-facture (Mustang/FOP pipeline)</xmp:CreatorTool>
      <pdf:Producer>Apache FOP + pdp-facture</pdf:Producer>
      <pdfaid:part>3</pdfaid:part>
      <pdfaid:conformance>A</pdfaid:conformance>
      <fx:DocumentFileName>factur-x.xml</fx:DocumentFileName>
      <fx:DocumentType>INVOICE</fx:DocumentType>
      <fx:ConformanceLevel>{level}</fx:ConformanceLevel>
      <fx:Version>1.0</fx:Version>
      <pdfaExtension:schemas>
        <rdf:Bag>
          <rdf:li rdf:parseType="Resource">
            <pdfaSchema:schema>Factur-X PDFA Extension Schema</pdfaSchema:schema>
            <pdfaSchema:namespaceURI>urn:factur-x:pdfa:CrossIndustryDocument:invoice:1p0#</pdfaSchema:namespaceURI>
            <pdfaSchema:prefix>fx</pdfaSchema:prefix>
            <pdfaSchema:property>
              <rdf:Seq>
                <rdf:li rdf:parseType="Resource">
                  <pdfaProperty:name>DocumentFileName</pdfaProperty:name>
                  <pdfaProperty:valueType>Text</pdfaProperty:valueType>
                  <pdfaProperty:category>external</pdfaProperty:category>
                  <pdfaProperty:description>Name of the embedded XML invoice file</pdfaProperty:description>
                </rdf:li>
                <rdf:li rdf:parseType="Resource">
                  <pdfaProperty:name>DocumentType</pdfaProperty:name>
                  <pdfaProperty:valueType>Text</pdfaProperty:valueType>
                  <pdfaProperty:category>external</pdfaProperty:category>
                  <pdfaProperty:description>INVOICE</pdfaProperty:description>
                </rdf:li>
                <rdf:li rdf:parseType="Resource">
                  <pdfaProperty:name>ConformanceLevel</pdfaProperty:name>
                  <pdfaProperty:valueType>Text</pdfaProperty:valueType>
                  <pdfaProperty:category>external</pdfaProperty:category>
                  <pdfaProperty:description>Factur-X conformance level</pdfaProperty:description>
                </rdf:li>
                <rdf:li rdf:parseType="Resource">
                  <pdfaProperty:name>Version</pdfaProperty:name>
                  <pdfaProperty:valueType>Text</pdfaProperty:valueType>
                  <pdfaProperty:category>external</pdfaProperty:category>
                  <pdfaProperty:description>Factur-X version</pdfaProperty:description>
                </rdf:li>
              </rdf:Seq>
            </pdfaSchema:property>
          </rdf:li>
        </rdf:Bag>
      </pdfaExtension:schemas>
    </rdf:Description>
  </rdf:RDF>
</x:xmpmeta>
<?xpacket end="w"?>"#,
            invoice_number = invoice_number,
            level = level,
            now = now,
        )
    }

    fn make_filename(&self, invoice_number: &str) -> String {
        let safe = invoice_number
            .replace('/', "-")
            .replace('\\', "-")
            .replace(' ', "_");
        format!("{}_facturx.pdf", safe)
    }

    /// Corrige le PDF via qpdf pour ajouter le commentaire binaire PDF/A
    /// et garantir une structure xref valide.
    /// qpdf --force-version=1.7 --normalize-content=n recalcule les offsets
    /// et écrit le header binaire requis par PDF/A.
    fn inject_binary_header(pdf: Vec<u8>) -> Vec<u8> {
        // Vérifier si le commentaire binaire est déjà présent
        if let Some(first_nl) = pdf.iter().position(|&b| b == b'\n') {
            if first_nl + 5 < pdf.len()
                && pdf[first_nl + 1] == b'%'
                && pdf[first_nl + 2] > 127
                && pdf[first_nl + 3] > 127
                && pdf[first_nl + 4] > 127
                && pdf[first_nl + 5] > 127
            {
                return pdf; // Déjà conforme
            }
        }

        let tmp_dir = std::env::temp_dir();
        let id = uuid::Uuid::new_v4();
        let tmp_in = tmp_dir.join(format!("pdp-fix-in-{}.pdf", id));
        let tmp_out = tmp_dir.join(format!("pdp-fix-out-{}.pdf", id));

        if std::fs::write(&tmp_in, &pdf).is_err() {
            return pdf;
        }

        // qpdf recalcule les offsets xref et écrit un header binaire valide
        // --stream-data=preserve : ne pas toucher au contenu des streams
        // --newline-before-endstream : ajouter \n avant endstream (requis PDF/A)
        // --object-streams=disable : utiliser xref table classique (pas xref stream)
        let result = std::process::Command::new("qpdf")
            .arg("--force-version=1.7")
            .arg("--normalize-content=n")
            .arg("--stream-data=preserve")
            .arg("--newline-before-endstream")
            .arg("--object-streams=disable")
            .arg(tmp_in.to_str().unwrap_or(""))
            .arg(tmp_out.to_str().unwrap_or(""))
            .output();

        let _ = std::fs::remove_file(&tmp_in);

        match result {
            Ok(output) if tmp_out.exists() => {
                let fixed = std::fs::read(&tmp_out).unwrap_or(pdf);
                let _ = std::fs::remove_file(&tmp_out);
                if output.status.success() || tmp_out.exists() {
                    fixed
                } else {
                    tracing::warn!("qpdf a échoué, header binaire non corrigé");
                    fixed
                }
            }
            _ => {
                let _ = std::fs::remove_file(&tmp_out);
                tracing::warn!("qpdf non disponible, header binaire PDF/A non corrigé");
                pdf
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn generator() -> FacturXGenerator {
        let specs = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../specs");
        FacturXGenerator::from_specs_dir(&specs)
    }

    #[test]
    fn test_generate_facturx_from_cii() {
        let cii_xml = std::fs::read_to_string("../../tests/fixtures/cii/facture_cii_001.xml")
            .expect("Fixture CII introuvable");
        let parser = pdp_invoice::cii::CiiParser::new();
        let invoice = parser.parse(&cii_xml).expect("Parsing CII échoué");

        let result = generator().generate(&invoice).expect("Génération Factur-X échouée");

        assert!(result.pdf.len() > 100, "Le PDF doit avoir une taille raisonnable");
        assert_eq!(&result.pdf[0..5], b"%PDF-");
        assert!(result.cii_xml.contains("CrossIndustryInvoice"));
        assert!(result.filename.contains("facturx.pdf"));
        // Le fixture CII déclare le profil Extended
        assert_eq!(result.level, FacturXLevel::Extended);
    }

    #[test]
    fn test_generate_facturx_from_ubl() {
        let ubl_xml = std::fs::read_to_string("../../tests/fixtures/ubl/facture_ubl_001.xml")
            .expect("Fixture UBL introuvable");
        let parser = pdp_invoice::ubl::UblParser::new();
        let invoice = parser.parse(&ubl_xml).expect("Parsing UBL échoué");

        let result = generator().generate(&invoice).expect("Génération Factur-X échouée");

        assert!(result.pdf.len() > 100);
        assert_eq!(&result.pdf[0..5], b"%PDF-");
        // Le CII XML embarqué doit contenir les données de la facture UBL convertie
        assert!(result.cii_xml.contains("CrossIndustryInvoice"));
        assert!(result.cii_xml.contains("FA-2025-00142"));
    }

    #[test]
    fn test_generate_facturx_with_attachments() {
        let cii_xml = std::fs::read_to_string("../../tests/fixtures/cii/facture_cii_001.xml")
            .expect("Fixture CII introuvable");
        let parser = pdp_invoice::cii::CiiParser::new();
        let mut invoice = parser.parse(&cii_xml).expect("Parsing CII échoué");

        // Ajouter une pièce jointe simulée
        invoice.attachments.push(InvoiceAttachment {
            id: Some("ATT-001".to_string()),
            description: Some("Bon de commande".to_string()),
            external_uri: None,
            embedded_content: Some(b"Contenu du bon de commande".to_vec()),
            mime_code: Some("text/plain".to_string()),
            filename: Some("bon_commande.txt".to_string()),
        });

        let result = generator().generate(&invoice).expect("Génération Factur-X avec PJ échouée");

        assert!(result.pdf.len() > 100);
        assert_eq!(&result.pdf[0..5], b"%PDF-");

        // Vérifier que le PDF contient les fichiers embarqués
        let doc = lopdf::Document::load_mem(&result.pdf).expect("Relecture PDF échouée");
        // Chercher les EmbeddedFiles dans le catalogue
        let mut found_facturx = false;
        let mut found_attachment = false;
        for (_id, obj) in doc.objects.iter() {
            if let Ok(dict) = obj.as_dict() {
                if let Ok(f) = dict.get(b"F") {
                    if let Ok(s) = f.as_str() {
                        if s == b"factur-x.xml" { found_facturx = true; }
                        if s == b"bon_commande.txt" { found_attachment = true; }
                    }
                }
            }
        }
        assert!(found_facturx, "Le PDF doit contenir factur-x.xml");
        assert!(found_attachment, "Le PDF doit contenir bon_commande.txt");
    }

    #[test]
    fn test_facturx_xmp_metadata() {
        let gen = generator();
        let xmp = gen.build_xmp_metadata("FA-2025-TEST", FacturXLevel::EN16931);
        assert!(xmp.contains("pdfaid:part>3</pdfaid:part"));
        assert!(xmp.contains("pdfaid:conformance>A</pdfaid:conformance>"));
        assert!(xmp.contains("fx:DocumentFileName>factur-x.xml</fx:DocumentFileName"));
        assert!(xmp.contains("fx:ConformanceLevel>EN 16931</fx:ConformanceLevel"));
        assert!(xmp.contains("FA-2025-TEST"));
    }

    // ===== Tests de conformité PDF/A-3 et Factur-X =====

    fn generate_facturx_pdf() -> (Vec<u8>, String) {
        let cii_xml = std::fs::read_to_string("../../tests/fixtures/cii/facture_cii_001.xml")
            .expect("Fixture CII introuvable");
        let parser = pdp_invoice::cii::CiiParser::new();
        let invoice = parser.parse(&cii_xml).expect("Parsing CII échoué");
        let result = generator().generate(&invoice).expect("Génération Factur-X échouée");
        (result.pdf, result.cii_xml)
    }

    #[test]
    fn test_pdfa3_catalog_metadata_stream() {
        let (pdf, _) = generate_facturx_pdf();
        let doc = lopdf::Document::load_mem(&pdf).expect("Relecture PDF échouée");

        let catalog_id = doc.trailer.get(b"Root")
            .expect("Trailer doit avoir Root")
            .as_reference().expect("Root doit être une référence");
        let catalog = doc.get_object(catalog_id)
            .expect("Catalogue introuvable")
            .as_dict().expect("Catalogue doit être un dictionnaire");

        // PDF/A-3 : le catalogue doit avoir un Metadata stream XMP
        let metadata_ref = catalog.get(b"Metadata")
            .expect("Catalogue doit avoir Metadata (PDF/A-3)");
        let metadata_id = metadata_ref.as_reference()
            .expect("Metadata doit être une référence");
        let metadata_obj = doc.get_object(metadata_id)
            .expect("Objet Metadata introuvable");

        // Extraire le contenu XMP du stream
        let xmp_bytes = match metadata_obj {
            lopdf::Object::Stream(ref stream) => {
                let mut s = stream.clone();
                let _ = s.decompress();
                s.content
            }
            _ => panic!("Metadata doit être un Stream"),
        };
        let xmp = String::from_utf8_lossy(&xmp_bytes);

        // Vérifier les champs PDF/A-3a obligatoires
        assert!(xmp.contains("pdfaid:part>3</pdfaid:part"),
            "XMP doit déclarer pdfaid:part=3");
        assert!(xmp.contains("pdfaid:conformance>A</pdfaid:conformance>"),
            "XMP doit déclarer pdfaid:conformance=A");

        // Vérifier les champs Factur-X obligatoires
        assert!(xmp.contains("fx:DocumentFileName>factur-x.xml</fx:DocumentFileName"),
            "XMP doit déclarer fx:DocumentFileName=factur-x.xml");
        assert!(xmp.contains("fx:DocumentType>INVOICE</fx:DocumentType"),
            "XMP doit déclarer fx:DocumentType=INVOICE");
        // Le fixture CII déclare le profil Extended
        assert!(xmp.contains("fx:ConformanceLevel>EXTENDED</fx:ConformanceLevel"),
            "XMP doit déclarer fx:ConformanceLevel cohérent avec le XML (EXTENDED)");
        assert!(xmp.contains("fx:Version>1.0</fx:Version"),
            "XMP doit déclarer fx:Version=1.0");

        // Vérifier le schéma d'extension PDF/A Factur-X
        assert!(xmp.contains("pdfaExtension:schemas"),
            "XMP doit contenir le schéma d'extension PDF/A Factur-X");
        assert!(xmp.contains("urn:factur-x:pdfa:CrossIndustryDocument:invoice:1p0#"),
            "XMP doit contenir le namespace Factur-X");
    }

    #[test]
    fn test_pdfa3_catalog_markinfo() {
        let (pdf, _) = generate_facturx_pdf();
        let doc = lopdf::Document::load_mem(&pdf).expect("Relecture PDF échouée");

        let catalog_id = doc.trailer.get(b"Root")
            .unwrap().as_reference().unwrap();
        let catalog = doc.get_object(catalog_id).unwrap().as_dict().unwrap();

        // PDF/A : MarkInfo avec Marked=true
        let mark_info = catalog.get(b"MarkInfo")
            .expect("Catalogue doit avoir MarkInfo (PDF/A)");
        let mark_dict = mark_info.as_dict()
            .expect("MarkInfo doit être un dictionnaire");
        let marked = mark_dict.get(b"Marked")
            .expect("MarkInfo doit avoir Marked");
        assert_eq!(marked.as_bool().unwrap(), true,
            "MarkInfo.Marked doit être true (PDF/A)");
    }

    #[test]
    fn test_pdfa3_catalog_af_array() {
        let (pdf, _) = generate_facturx_pdf();
        let doc = lopdf::Document::load_mem(&pdf).expect("Relecture PDF échouée");

        let catalog_id = doc.trailer.get(b"Root")
            .unwrap().as_reference().unwrap();
        let catalog = doc.get_object(catalog_id).unwrap().as_dict().unwrap();

        // PDF/A-3 : AF (Associated Files) array dans le catalogue
        let af = catalog.get(b"AF")
            .expect("Catalogue doit avoir AF (Associated Files, PDF/A-3)");
        let af_array = af.as_array()
            .expect("AF doit être un tableau");
        assert!(!af_array.is_empty(),
            "AF doit contenir au moins factur-x.xml");

        // Vérifier que le premier AF pointe vers un Filespec avec AFRelationship=Data
        let first_ref = af_array[0].as_reference()
            .expect("AF[0] doit être une référence");
        let filespec = doc.get_object(first_ref).unwrap().as_dict().unwrap();
        let af_rel = filespec.get(b"AFRelationship")
            .expect("Filespec doit avoir AFRelationship");
        assert_eq!(af_rel.as_name_str().unwrap(), "Data",
            "factur-x.xml AFRelationship doit être 'Data'");
    }

    #[test]
    fn test_pdfa3_embedded_files_name_tree() {
        let (pdf, _) = generate_facturx_pdf();
        let doc = lopdf::Document::load_mem(&pdf).expect("Relecture PDF échouée");

        let catalog_id = doc.trailer.get(b"Root")
            .unwrap().as_reference().unwrap();
        let catalog = doc.get_object(catalog_id).unwrap().as_dict().unwrap();

        // Names/EmbeddedFiles name tree
        let names_ref = catalog.get(b"Names")
            .expect("Catalogue doit avoir Names");
        let names_id = names_ref.as_reference()
            .expect("Names doit être une référence");
        let names_dict = doc.get_object(names_id).unwrap().as_dict().unwrap();

        let ef_ref = names_dict.get(b"EmbeddedFiles")
            .expect("Names doit avoir EmbeddedFiles");
        let ef_id = ef_ref.as_reference()
            .expect("EmbeddedFiles doit être une référence");
        let ef_dict = doc.get_object(ef_id).unwrap().as_dict().unwrap();

        let names_array = ef_dict.get(b"Names")
            .expect("EmbeddedFiles doit avoir Names")
            .as_array()
            .expect("Names doit être un tableau");

        // Le name tree doit contenir au moins "factur-x.xml" + sa référence
        assert!(names_array.len() >= 2,
            "Names doit contenir au moins 1 paire (nom, ref)");

        // Premier nom doit être "factur-x.xml"
        let first_name = names_array[0].as_str()
            .expect("Premier élément doit être une chaîne");
        assert_eq!(first_name, b"factur-x.xml",
            "Premier fichier embarqué doit être factur-x.xml");
    }

    #[test]
    fn test_facturx_embedded_xml_is_valid_cii() {
        let (pdf, expected_cii) = generate_facturx_pdf();
        let doc = lopdf::Document::load_mem(&pdf).expect("Relecture PDF échouée");

        // Extraire le contenu de factur-x.xml depuis le PDF
        let mut found_xml = None;
        for (_id, obj) in doc.objects.iter() {
            if let Ok(dict) = obj.as_dict() {
                if let Ok(f) = dict.get(b"F") {
                    if let Ok(s) = f.as_str() {
                        if s == b"factur-x.xml" {
                            // Trouver le stream EF/F
                            if let Ok(ef) = dict.get(b"EF") {
                                if let Ok(ef_dict) = ef.as_dict() {
                                    if let Ok(f_ref) = ef_dict.get(b"F") {
                                        if let Ok(stream_id) = f_ref.as_reference() {
                                            if let Ok(stream_obj) = doc.get_object(stream_id) {
                                                if let lopdf::Object::Stream(ref stream) = stream_obj {
                                                    let mut s = stream.clone();
                                                    let _ = s.decompress();
                                                    found_xml = Some(String::from_utf8_lossy(&s.content).to_string());
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        let embedded_xml = found_xml.expect("factur-x.xml doit être extractible du PDF");

        // Le XML embarqué doit être du CII valide
        assert!(embedded_xml.contains("CrossIndustryInvoice"),
            "XML embarqué doit être un CrossIndustryInvoice CII");
        assert!(embedded_xml.contains("ExchangedDocumentContext"),
            "XML embarqué doit contenir ExchangedDocumentContext");
        assert!(embedded_xml.contains("ExchangedDocument"),
            "XML embarqué doit contenir ExchangedDocument");
        assert!(embedded_xml.contains("SupplyChainTradeTransaction"),
            "XML embarqué doit contenir SupplyChainTradeTransaction");

        // Le XML embarqué doit correspondre au CII généré
        assert_eq!(embedded_xml.trim(), expected_cii.trim(),
            "Le XML embarqué doit correspondre au CII généré");
    }

    #[test]
    fn test_facturx_embedded_xml_parseable_as_invoice() {
        let (pdf, _) = generate_facturx_pdf();
        let doc = lopdf::Document::load_mem(&pdf).expect("Relecture PDF échouée");

        // Extraire factur-x.xml
        let mut found_xml = None;
        for (_id, obj) in doc.objects.iter() {
            if let Ok(dict) = obj.as_dict() {
                if let Ok(f) = dict.get(b"F") {
                    if let Ok(s) = f.as_str() {
                        if s == b"factur-x.xml" {
                            if let Ok(ef) = dict.get(b"EF") {
                                if let Ok(ef_dict) = ef.as_dict() {
                                    if let Ok(f_ref) = ef_dict.get(b"F") {
                                        if let Ok(stream_id) = f_ref.as_reference() {
                                            if let Ok(stream_obj) = doc.get_object(stream_id) {
                                                if let lopdf::Object::Stream(ref stream) = stream_obj {
                                                    let mut s = stream.clone();
                                                    let _ = s.decompress();
                                                    found_xml = Some(String::from_utf8_lossy(&s.content).to_string());
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        let embedded_xml = found_xml.expect("factur-x.xml doit être extractible");

        // Parser le XML embarqué comme une facture CII
        let cii_parser = pdp_invoice::cii::CiiParser::new();
        let invoice = cii_parser.parse(&embedded_xml)
            .expect("Le XML embarqué doit être parseable comme facture CII");

        // Vérifier les données de la facture
        assert!(!invoice.invoice_number.is_empty(),
            "La facture extraite doit avoir un numéro");
        assert!(invoice.seller_name.is_some(),
            "La facture extraite doit avoir un vendeur");
        assert!(invoice.buyer_name.is_some(),
            "La facture extraite doit avoir un acheteur");
        assert!(invoice.total_ttc.is_some(),
            "La facture extraite doit avoir un total TTC");
    }

    #[test]
    fn test_facturx_embedded_xml_schematron_valid() {
        let (pdf, _) = generate_facturx_pdf();
        let doc = lopdf::Document::load_mem(&pdf).expect("Relecture PDF échouée");

        // Extraire factur-x.xml
        let mut found_xml = None;
        for (_id, obj) in doc.objects.iter() {
            if let Ok(dict) = obj.as_dict() {
                if let Ok(f) = dict.get(b"F") {
                    if let Ok(s) = f.as_str() {
                        if s == b"factur-x.xml" {
                            if let Ok(ef) = dict.get(b"EF") {
                                if let Ok(ef_dict) = ef.as_dict() {
                                    if let Ok(f_ref) = ef_dict.get(b"F") {
                                        if let Ok(stream_id) = f_ref.as_reference() {
                                            if let Ok(stream_obj) = doc.get_object(stream_id) {
                                                if let lopdf::Object::Stream(ref stream) = stream_obj {
                                                    let mut s = stream.clone();
                                                    let _ = s.decompress();
                                                    found_xml = Some(String::from_utf8_lossy(&s.content).to_string());
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        let embedded_xml = found_xml.expect("factur-x.xml doit être extractible");

        // Valider le XML embarqué avec le Schematron EN16931-CII
        let specs_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../specs");
        let validator = pdp_validate::SchematronValidator::new(&specs_dir);
        let report = validator.validate_cii_invoice(&embedded_xml);
        let fatal: Vec<_> = report.issues.iter()
            .filter(|i| matches!(i.level, pdp_validate::ValidationLevel::Fatal | pdp_validate::ValidationLevel::Error))
            .collect();
        assert!(fatal.is_empty(),
            "Le XML Factur-X embarqué ne doit pas avoir d'erreurs Schematron: {:?}", fatal);
    }

    #[test]
    fn test_facturx_filespec_structure() {
        let (pdf, _) = generate_facturx_pdf();
        let doc = lopdf::Document::load_mem(&pdf).expect("Relecture PDF échouée");

        // Trouver le Filespec de factur-x.xml et vérifier sa structure complète
        let mut found = false;
        for (_id, obj) in doc.objects.iter() {
            if let Ok(dict) = obj.as_dict() {
                if let Ok(f) = dict.get(b"F") {
                    if let Ok(s) = f.as_str() {
                        if s == b"factur-x.xml" {
                            found = true;

                            // Type doit être Filespec
                            let type_val = dict.get(b"Type")
                                .expect("Filespec doit avoir Type");
                            assert_eq!(type_val.as_name_str().unwrap(), "Filespec",
                                "Type doit être Filespec");

                            // UF (Unicode filename) doit être présent
                            assert!(dict.get(b"UF").is_ok(),
                                "Filespec doit avoir UF (Unicode filename)");

                            // AFRelationship doit être Data
                            let af_rel = dict.get(b"AFRelationship")
                                .expect("Filespec doit avoir AFRelationship");
                            assert_eq!(af_rel.as_name_str().unwrap(), "Data",
                                "AFRelationship doit être 'Data' pour factur-x.xml");

                            // EF (Embedded File) doit être présent avec F et UF
                            let ef = dict.get(b"EF")
                                .expect("Filespec doit avoir EF")
                                .as_dict()
                                .expect("EF doit être un dictionnaire");
                            assert!(ef.get(b"F").is_ok(),
                                "EF doit avoir F (embedded file stream)");

                            // Le stream embarqué doit avoir le bon Subtype
                            let stream_ref = ef.get(b"F").unwrap().as_reference().unwrap();
                            let stream_obj = doc.get_object(stream_ref).unwrap();
                            if let lopdf::Object::Stream(ref stream) = stream_obj {
                                let subtype = stream.dict.get(b"Subtype");
                                if let Ok(st) = subtype {
                                    assert_eq!(st.as_name_str().unwrap(), "text/xml",
                                        "Subtype du stream doit être text/xml");
                                }
                            }
                        }
                    }
                }
            }
        }
        assert!(found, "Filespec factur-x.xml introuvable dans le PDF");
    }

    #[test]
    fn test_facturx_from_ubl_conformance() {
        let ubl_xml = std::fs::read_to_string("../../tests/fixtures/ubl/facture_ubl_001.xml")
            .expect("Fixture UBL introuvable");
        let parser = pdp_invoice::ubl::UblParser::new();
        let invoice = parser.parse(&ubl_xml).expect("Parsing UBL échoué");

        let result = generator().generate(&invoice).expect("Génération Factur-X depuis UBL échouée");
        let doc = lopdf::Document::load_mem(&result.pdf).expect("Relecture PDF échouée");

        // Vérifier que le XML embarqué est du CII (pas UBL)
        let mut found_xml = None;
        for (_id, obj) in doc.objects.iter() {
            if let Ok(dict) = obj.as_dict() {
                if let Ok(f) = dict.get(b"F") {
                    if let Ok(s) = f.as_str() {
                        if s == b"factur-x.xml" {
                            if let Ok(ef) = dict.get(b"EF") {
                                if let Ok(ef_dict) = ef.as_dict() {
                                    if let Ok(f_ref) = ef_dict.get(b"F") {
                                        if let Ok(stream_id) = f_ref.as_reference() {
                                            if let Ok(stream_obj) = doc.get_object(stream_id) {
                                                if let lopdf::Object::Stream(ref stream) = stream_obj {
                                                    let mut s = stream.clone();
                                                    let _ = s.decompress();
                                                    found_xml = Some(String::from_utf8_lossy(&s.content).to_string());
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        let embedded_xml = found_xml.expect("factur-x.xml doit être extractible");

        // Factur-X = CII, pas UBL
        assert!(embedded_xml.contains("CrossIndustryInvoice"),
            "Factur-X depuis UBL doit embarquer du CII, pas du UBL");
        assert!(!embedded_xml.contains("<Invoice"),
            "Factur-X ne doit pas contenir de balise UBL <Invoice>");

        // Le CII embarqué doit contenir les données de la facture UBL originale
        assert!(embedded_xml.contains("FA-2025-00142"),
            "Le CII embarqué doit contenir le numéro de facture UBL");

        // Parser et vérifier
        let cii_parser = pdp_invoice::cii::CiiParser::new();
        let extracted = cii_parser.parse(&embedded_xml)
            .expect("Le CII embarqué depuis UBL doit être parseable");
        assert_eq!(extracted.invoice_number, invoice.invoice_number,
            "Le numéro de facture doit être préservé après conversion UBL→CII→Factur-X");
    }

    #[test]
    fn test_facturx_verapdf_pdfa3_validation() {
        // Vérifier que veraPDF est installé
        let check = std::process::Command::new("verapdf")
            .arg("--version")
            .output();
        if check.is_err() {
            eprintln!("SKIP: veraPDF non installé, validation PDF/A-3 ignorée");
            return;
        }

        // Générer un Factur-X depuis CII
        let (pdf_cii, _) = generate_facturx_pdf();
        let tmp_dir = std::env::temp_dir();
        let pdf_path_cii = tmp_dir.join("pdp_test_facturx_cii.pdf");
        std::fs::write(&pdf_path_cii, &pdf_cii).expect("Écriture PDF temporaire échouée");

        // Valider avec veraPDF profil PDF/A-3A
        let output = std::process::Command::new("verapdf")
            .arg("--flavour").arg("3a")
            .arg("--format").arg("text")
            .arg(pdf_path_cii.to_str().unwrap())
            .output()
            .expect("Exécution veraPDF échouée");

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("=== veraPDF CII→Factur-X ===");
        println!("stdout: {}", stdout);
        if !stderr.is_empty() {
            println!("stderr: {}", stderr);
        }

        // Générer un Factur-X depuis UBL
        let ubl_xml = std::fs::read_to_string("../../tests/fixtures/ubl/facture_ubl_001.xml")
            .expect("Fixture UBL introuvable");
        let parser = pdp_invoice::ubl::UblParser::new();
        let invoice = parser.parse(&ubl_xml).expect("Parsing UBL échoué");
        let result = generator().generate(&invoice).expect("Génération Factur-X depuis UBL échouée");
        let pdf_path_ubl = tmp_dir.join("pdp_test_facturx_ubl.pdf");
        std::fs::write(&pdf_path_ubl, &result.pdf).expect("Écriture PDF temporaire échouée");

        let output_ubl = std::process::Command::new("verapdf")
            .arg("--flavour").arg("3a")
            .arg("--format").arg("text")
            .arg(pdf_path_ubl.to_str().unwrap())
            .output()
            .expect("Exécution veraPDF échouée");

        let stdout_ubl = String::from_utf8_lossy(&output_ubl.stdout);
        let stderr_ubl = String::from_utf8_lossy(&output_ubl.stderr);
        println!("=== veraPDF UBL→Factur-X ===");
        println!("stdout: {}", stdout_ubl);
        if !stderr_ubl.is_empty() {
            println!("stderr: {}", stderr_ubl);
        }

        // Détails des violations CII (format XML pour diagnostic)
        let detail_cii = std::process::Command::new("verapdf")
            .arg("--flavour").arg("3a")
            .arg(pdf_path_cii.to_str().unwrap())
            .output()
            .expect("Exécution veraPDF échouée");
        let detail_cii_out = String::from_utf8_lossy(&detail_cii.stdout);
        println!("=== veraPDF DETAIL CII ===\n{}", detail_cii_out);

        // Nettoyer
        let _ = std::fs::remove_file(&pdf_path_cii);
        let _ = std::fs::remove_file(&pdf_path_ubl);

        // Vérifier la conformité
        assert!(stdout.contains("PASS") || stdout.contains("isCompliant=\"true\""),
            "Le PDF Factur-X (CII) doit être conforme PDF/A-3A selon veraPDF.\nDétails:\n{}", detail_cii_out);
        assert!(stdout_ubl.contains("PASS") || stdout_ubl.contains("isCompliant=\"true\""),
            "Le PDF Factur-X (UBL) doit être conforme PDF/A-3A selon veraPDF.\nSortie: {}", stdout_ubl);
    }

    /// Extrait le contenu de factur-x.xml depuis un PDF Factur-X.
    fn extract_facturx_xml(pdf: &[u8]) -> String {
        let doc = lopdf::Document::load_mem(pdf).expect("Relecture PDF échouée");
        for (_id, obj) in doc.objects.iter() {
            if let Ok(dict) = obj.as_dict() {
                if let Ok(f) = dict.get(b"F") {
                    if let Ok(s) = f.as_str() {
                        if s == b"factur-x.xml" {
                            if let Ok(ef) = dict.get(b"EF") {
                                if let Ok(ef_dict) = ef.as_dict() {
                                    if let Ok(f_ref) = ef_dict.get(b"F") {
                                        if let Ok(stream_id) = f_ref.as_reference() {
                                            if let Ok(stream_obj) = doc.get_object(stream_id) {
                                                if let lopdf::Object::Stream(ref stream) = stream_obj {
                                                    let mut s = stream.clone();
                                                    let _ = s.decompress();
                                                    return String::from_utf8_lossy(&s.content).to_string();
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        panic!("factur-x.xml introuvable dans le PDF");
    }

    /// Génère un Factur-X depuis une fixture et valide le XML embarqué via XSD + Schematron.
    fn assert_facturx_valid(fixture_path: &str, source_format: &str) {
        let xml = std::fs::read_to_string(fixture_path)
            .unwrap_or_else(|_| panic!("Fixture introuvable: {}", fixture_path));

        let invoice = match source_format {
            "CII" => pdp_invoice::cii::CiiParser::new().parse(&xml)
                .unwrap_or_else(|e| panic!("Parsing CII échoué pour {}: {}", fixture_path, e)),
            "UBL" => pdp_invoice::ubl::UblParser::new().parse(&xml)
                .unwrap_or_else(|e| panic!("Parsing UBL échoué pour {}: {}", fixture_path, e)),
            _ => panic!("Format source inconnu: {}", source_format),
        };

        let result = generator().generate(&invoice)
            .unwrap_or_else(|e| panic!("Génération Factur-X échouée pour {} ({}): {}", fixture_path, source_format, e));

        // Vérifier que c'est un PDF
        assert_eq!(&result.pdf[0..5], b"%PDF-",
            "Le résultat doit être un PDF pour {}", fixture_path);

        // Extraire et valider le XML embarqué
        let embedded_xml = extract_facturx_xml(&result.pdf);

        // Le XML embarqué doit être du CII
        assert!(embedded_xml.contains("CrossIndustryInvoice"),
            "Le XML embarqué doit être du CII pour {} ({})", fixture_path, source_format);

        // Parser le XML embarqué
        let cii_parser = pdp_invoice::cii::CiiParser::new();
        let extracted = cii_parser.parse(&embedded_xml)
            .unwrap_or_else(|e| panic!("Le XML embarqué n'est pas parseable pour {} ({}): {}", fixture_path, source_format, e));
        assert_eq!(extracted.invoice_number, invoice.invoice_number,
            "Le numéro de facture doit être préservé pour {} ({})", fixture_path, source_format);

        let specs_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../specs");

        // Détecter le profil Factur-X depuis le XML embarqué
        let is_extended = embedded_xml.contains("urn:factur-x.eu:1p0:extended")
            || embedded_xml.contains("urn:zugferd.de:2p0:extended");
        let (xsd_type, fx_schematron) = if is_extended {
            (pdp_validate::xsd::XsdDocumentType::FacturXExtended,
             pdp_validate::schematron::SchematronType::FacturXExtended)
        } else {
            (pdp_validate::xsd::XsdDocumentType::FacturXEN16931,
             pdp_validate::schematron::SchematronType::FacturXEN16931)
        };

        // Validation XSD Factur-X
        let xsd_validator = pdp_validate::XsdValidator::new(&specs_dir);
        let xsd_report = xsd_validator.validate(&embedded_xml, &xsd_type);
        let xsd_errors: Vec<_> = xsd_report.issues.iter()
            .filter(|i| matches!(i.level, pdp_validate::ValidationLevel::Fatal | pdp_validate::ValidationLevel::Error))
            .collect();
        if !xsd_errors.is_empty() {
            eprintln!("[Factur-X {} → CII] XSD {:?} errors for {}:", source_format, xsd_type, fixture_path);
            for e in &xsd_errors {
                eprintln!("  [{}] {} - {}", e.source, e.rule_id, e.message);
            }
        }
        assert!(xsd_errors.is_empty(),
            "Le XML Factur-X embarqué doit être XSD-valide pour {} ({}): {} erreur(s)",
            fixture_path, source_format, xsd_errors.len());

        // Validation Schematron EN16931 + BR-FR (erreurs + warnings)
        let schematron_validator = pdp_validate::SchematronValidator::new(&specs_dir);
        let sch_report = schematron_validator.validate_cii_invoice(&embedded_xml);
        let sch_issues: Vec<_> = sch_report.issues.iter()
            .filter(|i| matches!(i.level,
                pdp_validate::ValidationLevel::Fatal
                | pdp_validate::ValidationLevel::Error
                | pdp_validate::ValidationLevel::Warning))
            .collect();
        if !sch_issues.is_empty() {
            eprintln!("[Factur-X {} → CII] Schematron EN16931+BR-FR issues for {}:", source_format, fixture_path);
            for e in &sch_issues {
                eprintln!("  [{:?}][{}] {} - {}", e.level, e.source, e.rule_id, e.message);
            }
        }
        assert!(sch_issues.is_empty(),
            "Le XML Factur-X embarqué doit être Schematron EN16931+BR-FR valide pour {} ({}): {} issue(s)",
            fixture_path, source_format, sch_issues.len());

        // Validation Schematron Factur-X (PEPPOL rules, empty elements, etc.)
        let fx_report = schematron_validator.validate(&embedded_xml, &fx_schematron);
        let fx_issues: Vec<_> = fx_report.issues.iter()
            .filter(|i| matches!(i.level,
                pdp_validate::ValidationLevel::Fatal
                | pdp_validate::ValidationLevel::Error
                | pdp_validate::ValidationLevel::Warning))
            .collect();
        if !fx_issues.is_empty() {
            eprintln!("[Factur-X {} → CII] Factur-X {:?} Schematron issues for {}:", source_format, fx_schematron, fixture_path);
            for e in &fx_issues {
                eprintln!("  [{:?}][{}] {} - {}", e.level, e.source, e.rule_id, e.message);
            }
        }
        assert!(fx_issues.is_empty(),
            "Le XML Factur-X embarqué doit être Factur-X Schematron valide pour {} ({}): {} issue(s)",
            fixture_path, source_format, fx_issues.len());
    }

    // ===== Factur-X depuis CII : validation Schematron =====

    #[test]
    fn test_facturx_from_cii_001_schematron() {
        assert_facturx_valid("../../tests/fixtures/cii/facture_cii_001.xml", "CII");
    }

    #[test]
    fn test_facturx_from_cii_avoir_schematron() {
        assert_facturx_valid("../../tests/fixtures/cii/avoir_cii_381.xml", "CII");
    }

    #[test]
    fn test_facturx_from_cii_rectificative_schematron() {
        assert_facturx_valid("../../tests/fixtures/cii/facture_rectificative_cii_384.xml", "CII");
    }

    #[test]
    fn test_facturx_from_cii_remises_multitva_schematron() {
        assert_facturx_valid("../../tests/fixtures/cii/facture_cii_remises_multitva.xml", "CII");
    }

    // ===== Factur-X depuis UBL : validation Schematron =====

    #[test]
    fn test_facturx_from_ubl_001_schematron() {
        assert_facturx_valid("../../tests/fixtures/ubl/facture_ubl_001.xml", "UBL");
    }

    #[test]
    fn test_facturx_from_ubl_marketplace_schematron() {
        assert_facturx_valid("../../tests/fixtures/ubl/facture_ubl_marketplace_a8.xml", "UBL");
    }

    #[test]
    fn test_facturx_from_ubl_soustraitance_schematron() {
        assert_facturx_valid("../../tests/fixtures/ubl/facture_ubl_soustraitance_a4.xml", "UBL");
    }

    #[test]
    fn test_facturx_from_ubl_remises_multitva_schematron() {
        assert_facturx_valid("../../tests/fixtures/ubl/facture_ubl_remises_multitva.xml", "UBL");
    }

    // ===== Tests d'attachements (BG-24) =====

    /// Helper : extrait le contenu binaire d'un fichier embarqué par nom depuis un PDF.
    fn extract_embedded_file(pdf: &[u8], filename: &str) -> Option<Vec<u8>> {
        let doc = lopdf::Document::load_mem(pdf).expect("Relecture PDF échouée");
        for (_id, obj) in doc.objects.iter() {
            if let Ok(dict) = obj.as_dict() {
                if let Ok(f) = dict.get(b"F") {
                    if let Ok(s) = f.as_str() {
                        if s == filename.as_bytes() {
                            if let Ok(ef) = dict.get(b"EF") {
                                if let Ok(ef_dict) = ef.as_dict() {
                                    if let Ok(f_ref) = ef_dict.get(b"F") {
                                        if let Ok(stream_id) = f_ref.as_reference() {
                                            if let Ok(stream_obj) = doc.get_object(stream_id) {
                                                if let lopdf::Object::Stream(ref stream) = stream_obj {
                                                    let mut s = stream.clone();
                                                    let _ = s.decompress();
                                                    return Some(s.content.clone());
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        None
    }

    /// Helper : liste tous les noms de fichiers embarqués dans un PDF.
    fn list_embedded_filenames(pdf: &[u8]) -> Vec<String> {
        let doc = lopdf::Document::load_mem(pdf).expect("Relecture PDF échouée");
        let mut names = Vec::new();
        for (_id, obj) in doc.objects.iter() {
            if let Ok(dict) = obj.as_dict() {
                if let Ok(type_val) = dict.get(b"Type") {
                    if type_val.as_name_str().ok() == Some("Filespec") {
                        if let Ok(f) = dict.get(b"F") {
                            if let Ok(s) = f.as_str() {
                                names.push(String::from_utf8_lossy(s).to_string());
                            }
                        }
                    }
                }
            }
        }
        names
    }

    /// Helper : vérifie l'AFRelationship d'un filespec par nom de fichier.
    fn get_af_relationship(pdf: &[u8], filename: &str) -> Option<String> {
        let doc = lopdf::Document::load_mem(pdf).expect("Relecture PDF échouée");
        for (_id, obj) in doc.objects.iter() {
            if let Ok(dict) = obj.as_dict() {
                if let Ok(f) = dict.get(b"F") {
                    if let Ok(s) = f.as_str() {
                        if s == filename.as_bytes() {
                            if let Ok(af_rel) = dict.get(b"AFRelationship") {
                                return af_rel.as_name_str().ok().map(|s| s.to_string());
                            }
                        }
                    }
                }
            }
        }
        None
    }

    /// Helper : crée une facture CII de base avec attachements.
    fn invoice_with_attachments(attachments: Vec<InvoiceAttachment>) -> InvoiceData {
        let cii_xml = std::fs::read_to_string("../../tests/fixtures/cii/facture_cii_001.xml")
            .expect("Fixture CII introuvable");
        let parser = pdp_invoice::cii::CiiParser::new();
        let mut invoice = parser.parse(&cii_xml).expect("Parsing CII échoué");
        invoice.attachments = attachments;
        invoice
    }

    #[test]
    fn test_attachment_multiple_types_pdf_png_csv() {
        // Tester l'embarquement de 3 types de fichiers différents
        let pdf_content = b"%PDF-1.4 fake pdf content for testing purposes only";
        let png_content = b"\x89PNG\r\n\x1a\n fake png content";
        let csv_content = b"col1;col2;col3\nval1;val2;val3\n";

        let invoice = invoice_with_attachments(vec![
            InvoiceAttachment {
                id: Some("ATT-PDF".to_string()),
                description: Some("Bon de commande PDF".to_string()),
                external_uri: None,
                embedded_content: Some(pdf_content.to_vec()),
                mime_code: Some("application/pdf".to_string()),
                filename: Some("bon_commande.pdf".to_string()),
            },
            InvoiceAttachment {
                id: Some("ATT-PNG".to_string()),
                description: Some("Photo produit".to_string()),
                external_uri: None,
                embedded_content: Some(png_content.to_vec()),
                mime_code: Some("image/png".to_string()),
                filename: Some("photo.png".to_string()),
            },
            InvoiceAttachment {
                id: Some("ATT-CSV".to_string()),
                description: Some("Détail lignes".to_string()),
                external_uri: None,
                embedded_content: Some(csv_content.to_vec()),
                mime_code: Some("text/csv".to_string()),
                filename: Some("details.csv".to_string()),
            },
        ]);

        let result = generator().generate(&invoice)
            .expect("Génération Factur-X avec 3 PJ échouée");

        // Vérifier le PDF de base
        assert_eq!(&result.pdf[0..5], b"%PDF-");

        // Vérifier que tous les fichiers sont embarqués
        let filenames = list_embedded_filenames(&result.pdf);
        assert!(filenames.contains(&"factur-x.xml".to_string()),
            "Doit contenir factur-x.xml, trouvé: {:?}", filenames);
        assert!(filenames.contains(&"bon_commande.pdf".to_string()),
            "Doit contenir bon_commande.pdf, trouvé: {:?}", filenames);
        assert!(filenames.contains(&"photo.png".to_string()),
            "Doit contenir photo.png, trouvé: {:?}", filenames);
        assert!(filenames.contains(&"details.csv".to_string()),
            "Doit contenir details.csv, trouvé: {:?}", filenames);

        // Vérifier le contenu extrait de chaque attachement
        let extracted_pdf = extract_embedded_file(&result.pdf, "bon_commande.pdf")
            .expect("bon_commande.pdf doit être extractible");
        assert_eq!(extracted_pdf, pdf_content, "Contenu PDF doit être identique");

        let extracted_png = extract_embedded_file(&result.pdf, "photo.png")
            .expect("photo.png doit être extractible");
        assert_eq!(extracted_png, png_content, "Contenu PNG doit être identique");

        let extracted_csv = extract_embedded_file(&result.pdf, "details.csv")
            .expect("details.csv doit être extractible");
        assert_eq!(extracted_csv, csv_content, "Contenu CSV doit être identique");
    }

    #[test]
    fn test_attachment_af_relationship_supplement() {
        // Les pièces jointes BG-24 doivent avoir AFRelationship=Supplement
        let invoice = invoice_with_attachments(vec![
            InvoiceAttachment {
                id: Some("ATT-001".to_string()),
                description: Some("Document joint".to_string()),
                external_uri: None,
                embedded_content: Some(b"test content".to_vec()),
                mime_code: Some("text/plain".to_string()),
                filename: Some("document.txt".to_string()),
            },
        ]);

        let result = generator().generate(&invoice)
            .expect("Génération Factur-X échouée");

        // factur-x.xml → Data
        assert_eq!(
            get_af_relationship(&result.pdf, "factur-x.xml"),
            Some("Data".to_string()),
            "factur-x.xml doit avoir AFRelationship=Data"
        );

        // Pièce jointe → Supplement
        assert_eq!(
            get_af_relationship(&result.pdf, "document.txt"),
            Some("Supplement".to_string()),
            "Les pièces jointes BG-24 doivent avoir AFRelationship=Supplement"
        );
    }

    #[test]
    fn test_attachment_af_array_contains_all_files() {
        // Le AF array du catalogue doit référencer tous les fichiers (factur-x.xml + PJ)
        let invoice = invoice_with_attachments(vec![
            InvoiceAttachment {
                id: Some("ATT-001".to_string()),
                description: None,
                external_uri: None,
                embedded_content: Some(b"content1".to_vec()),
                mime_code: Some("text/plain".to_string()),
                filename: Some("file1.txt".to_string()),
            },
            InvoiceAttachment {
                id: Some("ATT-002".to_string()),
                description: None,
                external_uri: None,
                embedded_content: Some(b"content2".to_vec()),
                mime_code: Some("application/octet-stream".to_string()),
                filename: Some("file2.bin".to_string()),
            },
        ]);

        let result = generator().generate(&invoice)
            .expect("Génération Factur-X échouée");
        let doc = lopdf::Document::load_mem(&result.pdf).expect("Relecture PDF échouée");

        // Vérifier AF array dans le catalogue
        let catalog_id = doc.trailer.get(b"Root").unwrap().as_reference().unwrap();
        let catalog = doc.get_object(catalog_id).unwrap().as_dict().unwrap();
        let af_array = catalog.get(b"AF").expect("AF manquant").as_array().expect("AF doit être un tableau");

        // 3 entrées : factur-x.xml + file1.txt + file2.bin
        assert_eq!(af_array.len(), 3,
            "AF doit contenir 3 entrées (factur-x.xml + 2 PJ), trouvé: {}", af_array.len());

        // Vérifier le Names/EmbeddedFiles name tree
        let names_ref = catalog.get(b"Names").unwrap().as_reference().unwrap();
        let names_dict = doc.get_object(names_ref).unwrap().as_dict().unwrap();
        let ef_ref = names_dict.get(b"EmbeddedFiles").unwrap().as_reference().unwrap();
        let ef_dict = doc.get_object(ef_ref).unwrap().as_dict().unwrap();
        let names_array = ef_dict.get(b"Names").unwrap().as_array().unwrap();

        // 6 entrées : 3 paires (nom, ref)
        assert_eq!(names_array.len(), 6,
            "Names doit contenir 6 entrées (3 paires nom+ref), trouvé: {}", names_array.len());

        // Vérifier l'ordre : factur-x.xml en premier
        let first_name = names_array[0].as_str().unwrap();
        assert_eq!(first_name, b"factur-x.xml", "Premier fichier doit être factur-x.xml");
    }

    #[test]
    fn test_attachment_external_uri_only_not_embedded() {
        // Les pièces jointes avec URI externe uniquement (sans embedded_content)
        // ne doivent PAS être embarquées dans le PDF
        let invoice = invoice_with_attachments(vec![
            InvoiceAttachment {
                id: Some("ATT-EXT".to_string()),
                description: Some("Document externe".to_string()),
                external_uri: Some("https://example.com/doc.pdf".to_string()),
                embedded_content: None,
                mime_code: Some("application/pdf".to_string()),
                filename: Some("doc_externe.pdf".to_string()),
            },
        ]);

        let result = generator().generate(&invoice)
            .expect("Génération Factur-X échouée");

        let filenames = list_embedded_filenames(&result.pdf);
        assert!(filenames.contains(&"factur-x.xml".to_string()),
            "Doit contenir factur-x.xml");
        assert!(!filenames.contains(&"doc_externe.pdf".to_string()),
            "Les PJ avec URI externe uniquement ne doivent pas être embarquées dans le PDF");
    }

    #[test]
    fn test_attachment_empty_content_not_embedded() {
        // Les pièces jointes sans contenu embarqué ne doivent pas être dans le PDF
        let invoice = invoice_with_attachments(vec![
            InvoiceAttachment {
                id: Some("ATT-EMPTY".to_string()),
                description: Some("Référence sans contenu".to_string()),
                external_uri: None,
                embedded_content: None,
                mime_code: None,
                filename: Some("vide.txt".to_string()),
            },
        ]);

        let result = generator().generate(&invoice)
            .expect("Génération Factur-X échouée");

        let filenames = list_embedded_filenames(&result.pdf);
        assert_eq!(filenames.len(), 1, "Seul factur-x.xml doit être embarqué");
        assert!(filenames.contains(&"factur-x.xml".to_string()));
    }

    #[test]
    fn test_attachment_mime_type_preserved() {
        // Vérifier que le MIME type est bien enregistré dans le stream embarqué
        let invoice = invoice_with_attachments(vec![
            InvoiceAttachment {
                id: Some("ATT-XML".to_string()),
                description: Some("Bordereau XML".to_string()),
                external_uri: None,
                embedded_content: Some(b"<root>test</root>".to_vec()),
                mime_code: Some("application/xml".to_string()),
                filename: Some("bordereau.xml".to_string()),
            },
        ]);

        let result = generator().generate(&invoice)
            .expect("Génération Factur-X échouée");
        let doc = lopdf::Document::load_mem(&result.pdf).expect("Relecture PDF échouée");

        // Trouver le filespec de bordereau.xml et vérifier le Subtype du stream
        for (_id, obj) in doc.objects.iter() {
            if let Ok(dict) = obj.as_dict() {
                if let Ok(f) = dict.get(b"F") {
                    if let Ok(s) = f.as_str() {
                        if s == b"bordereau.xml" {
                            let ef = dict.get(b"EF").unwrap().as_dict().unwrap();
                            let stream_ref = ef.get(b"F").unwrap().as_reference().unwrap();
                            let stream_obj = doc.get_object(stream_ref).unwrap();
                            if let lopdf::Object::Stream(ref stream) = stream_obj {
                                let subtype = stream.dict.get(b"Subtype")
                                    .expect("Stream doit avoir Subtype");
                                assert_eq!(subtype.as_name_str().unwrap(), "application/xml",
                                    "Subtype doit correspondre au MIME type");
                                let params = stream.dict.get(b"Params")
                                    .expect("Stream doit avoir Params")
                                    .as_dict().unwrap();
                                let size = params.get(b"Size").unwrap()
                                    .as_i64().unwrap();
                                assert_eq!(size, 17, "Size doit correspondre à la taille du contenu");
                            }
                            return;
                        }
                    }
                }
            }
        }
        panic!("bordereau.xml introuvable dans le PDF");
    }

    #[test]
    fn test_attachment_filename_fallback_to_id() {
        // Sans filename, le nom doit fallback sur l'id
        let invoice = invoice_with_attachments(vec![
            InvoiceAttachment {
                id: Some("BON-CMD-2025".to_string()),
                description: Some("Bon de commande".to_string()),
                external_uri: None,
                embedded_content: Some(b"contenu test".to_vec()),
                mime_code: Some("text/plain".to_string()),
                filename: None, // pas de filename
            },
        ]);

        let result = generator().generate(&invoice)
            .expect("Génération Factur-X échouée");

        let filenames = list_embedded_filenames(&result.pdf);
        assert!(filenames.contains(&"BON-CMD-2025".to_string()),
            "Sans filename, doit utiliser l'id comme nom. Trouvé: {:?}", filenames);
    }

    #[test]
    fn test_attachment_filename_fallback_to_default() {
        // Sans filename ni id, doit utiliser "attachment.bin"
        let invoice = invoice_with_attachments(vec![
            InvoiceAttachment {
                id: None,
                description: None,
                external_uri: None,
                embedded_content: Some(b"contenu anonyme".to_vec()),
                mime_code: None,
                filename: None,
            },
        ]);

        let result = generator().generate(&invoice)
            .expect("Génération Factur-X échouée");

        let filenames = list_embedded_filenames(&result.pdf);
        assert!(filenames.contains(&"attachment.bin".to_string()),
            "Sans filename ni id, doit utiliser 'attachment.bin'. Trouvé: {:?}", filenames);
    }

    #[test]
    fn test_attachment_large_binary() {
        // Tester avec un fichier binaire plus volumineux (100 Ko)
        let large_content: Vec<u8> = (0..100_000).map(|i| (i % 256) as u8).collect();

        let invoice = invoice_with_attachments(vec![
            InvoiceAttachment {
                id: Some("ATT-LARGE".to_string()),
                description: Some("Fichier volumineux".to_string()),
                external_uri: None,
                embedded_content: Some(large_content.clone()),
                mime_code: Some("application/octet-stream".to_string()),
                filename: Some("large_file.bin".to_string()),
            },
        ]);

        let result = generator().generate(&invoice)
            .expect("Génération Factur-X avec PJ volumineuse échouée");

        // Extraire et vérifier l'intégrité du contenu
        let extracted = extract_embedded_file(&result.pdf, "large_file.bin")
            .expect("large_file.bin doit être extractible");
        assert_eq!(extracted.len(), 100_000,
            "Taille du fichier extrait doit être 100 Ko");
        assert_eq!(extracted, large_content,
            "Contenu du fichier volumineux doit être identique après extraction");
    }

    #[test]
    fn test_attachment_pdfa3_compliance_with_attachments() {
        // Vérifier que la structure PDF/A-3 reste valide avec des pièces jointes
        let invoice = invoice_with_attachments(vec![
            InvoiceAttachment {
                id: Some("ATT-001".to_string()),
                description: Some("PJ test".to_string()),
                external_uri: None,
                embedded_content: Some(b"test pdfa3 compliance".to_vec()),
                mime_code: Some("text/plain".to_string()),
                filename: Some("test_compliance.txt".to_string()),
            },
        ]);

        let result = generator().generate(&invoice)
            .expect("Génération Factur-X échouée");
        let doc = lopdf::Document::load_mem(&result.pdf).expect("Relecture PDF échouée");
        let catalog_id = doc.trailer.get(b"Root").unwrap().as_reference().unwrap();
        let catalog = doc.get_object(catalog_id).unwrap().as_dict().unwrap();

        // PDF/A-3 : MarkInfo.Marked=true
        let mark_info = catalog.get(b"MarkInfo")
            .expect("MarkInfo manquant (PDF/A)")
            .as_dict().unwrap();
        assert_eq!(mark_info.get(b"Marked").unwrap().as_bool().unwrap(), true);

        // PDF/A-3 : Metadata stream XMP
        let metadata_ref = catalog.get(b"Metadata")
            .expect("Metadata manquant (PDF/A-3)");
        let metadata_id = metadata_ref.as_reference().unwrap();
        let metadata_obj = doc.get_object(metadata_id).unwrap();
        let xmp_bytes = match metadata_obj {
            lopdf::Object::Stream(ref stream) => {
                let mut s = stream.clone();
                let _ = s.decompress();
                s.content
            }
            _ => panic!("Metadata doit être un Stream"),
        };
        let xmp = String::from_utf8_lossy(&xmp_bytes);
        assert!(xmp.contains("pdfaid:part>3</pdfaid:part"),
            "XMP doit déclarer PDF/A part=3");
        assert!(xmp.contains("pdfaid:conformance>A</pdfaid:conformance>"),
            "XMP doit déclarer conformance=A");

        // PDF/A-3 : AF array doit contenir factur-x.xml + PJ
        let af_array = catalog.get(b"AF").unwrap().as_array().unwrap();
        assert_eq!(af_array.len(), 2, "AF doit contenir factur-x.xml + 1 PJ");

        // PDF/A-3 : Names/EmbeddedFiles
        let names_ref = catalog.get(b"Names").unwrap().as_reference().unwrap();
        let names_dict = doc.get_object(names_ref).unwrap().as_dict().unwrap();
        assert!(names_dict.get(b"EmbeddedFiles").is_ok(),
            "Names doit avoir EmbeddedFiles");
    }

    #[test]
    fn test_attachment_mixed_embedded_and_external() {
        // Mélange de PJ avec contenu embarqué et PJ avec URI externe
        // Seules les PJ embarquées doivent être dans le PDF
        let invoice = invoice_with_attachments(vec![
            InvoiceAttachment {
                id: Some("ATT-EMB".to_string()),
                description: Some("Embarquée".to_string()),
                external_uri: None,
                embedded_content: Some(b"embedded content".to_vec()),
                mime_code: Some("text/plain".to_string()),
                filename: Some("embedded.txt".to_string()),
            },
            InvoiceAttachment {
                id: Some("ATT-EXT".to_string()),
                description: Some("Externe".to_string()),
                external_uri: Some("https://example.com/external.pdf".to_string()),
                embedded_content: None,
                mime_code: Some("application/pdf".to_string()),
                filename: Some("external.pdf".to_string()),
            },
            InvoiceAttachment {
                id: Some("ATT-BOTH".to_string()),
                description: Some("Les deux".to_string()),
                external_uri: Some("https://example.com/both.pdf".to_string()),
                embedded_content: Some(b"both content".to_vec()),
                mime_code: Some("application/pdf".to_string()),
                filename: Some("both.pdf".to_string()),
            },
        ]);

        let result = generator().generate(&invoice)
            .expect("Génération Factur-X échouée");

        let filenames = list_embedded_filenames(&result.pdf);
        // factur-x.xml + embedded.txt + both.pdf = 3
        assert_eq!(filenames.len(), 3,
            "Doit contenir 3 fichiers (factur-x.xml + 2 embarquées). Trouvé: {:?}", filenames);
        assert!(filenames.contains(&"embedded.txt".to_string()));
        assert!(filenames.contains(&"both.pdf".to_string()));
        assert!(!filenames.contains(&"external.pdf".to_string()),
            "PJ externe seule ne doit pas être embarquée");
    }

    #[test]
    #[ignore] // Lancer manuellement : cargo test -p pdp-transform -- export_facturx_examples --ignored --nocapture
    fn export_facturx_examples() {
        let out_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../output");
        std::fs::create_dir_all(&out_dir).expect("Impossible de créer output/");

        let cii_parser = pdp_invoice::cii::CiiParser::new();
        let ubl_parser = pdp_invoice::ubl::UblParser::new();

        // Profils à générer : (suffixe fichier, profile_id Factur-X, FacturXLevel)
        let profiles: Vec<(&str, &str, FacturXLevel)> = vec![
            ("extended", "urn:cen.eu:en16931:2017#conformant#urn:factur-x.eu:1p0:extended", FacturXLevel::Extended),
            ("en16931", "urn:cen.eu:en16931:2017", FacturXLevel::EN16931),
        ];

        for (suffix, profile_id, level) in &profiles {
            println!("\n--- Profil {} ({}) ---", suffix, level);

            // 1. Factur-X depuis CII
            let cii_xml = std::fs::read_to_string("../../tests/fixtures/cii/facture_cii_001.xml")
                .expect("Fixture CII introuvable");
            let mut invoice_cii = cii_parser.parse(&cii_xml).expect("Parsing CII échoué");
            invoice_cii.profile_id = Some(profile_id.to_string());
            // Remplacer le profile_id dans le raw_xml CII aussi
            if let Some(ref raw) = invoice_cii.raw_xml {
                invoice_cii.raw_xml = Some(raw.replace(
                    "urn:cen.eu:en16931:2017#conformant#urn:factur-x.eu:1p0:extended",
                    profile_id,
                ));
            }
            let gen = generator().with_level(*level);
            let result_cii = gen.generate(&invoice_cii).expect("Génération Factur-X CII échouée");
            let path_cii = out_dir.join(format!("facturx_cii_{}.pdf", suffix));
            std::fs::write(&path_cii, &result_cii.pdf).expect("Écriture PDF échouée");
            println!("  CII → Factur-X : {} ({} Ko)", path_cii.display(), result_cii.pdf.len() / 1024);

            // 2. Factur-X depuis UBL
            let ubl_xml = std::fs::read_to_string("../../tests/fixtures/ubl/facture_ubl_001.xml")
                .expect("Fixture UBL introuvable");
            let mut invoice_ubl = ubl_parser.parse(&ubl_xml).expect("Parsing UBL échoué");
            invoice_ubl.profile_id = Some(profile_id.to_string());
            // Remplacer le profile_id dans le raw_xml UBL aussi (le XSLT copie CustomizationID)
            if let Some(ref raw) = invoice_ubl.raw_xml {
                invoice_ubl.raw_xml = Some(raw.replace(
                    "urn:cen.eu:en16931:2017#conformant#urn:factur-x.eu:1p0:extended",
                    profile_id,
                ));
            }
            let result_ubl = gen.generate(&invoice_ubl).expect("Génération Factur-X UBL échouée");
            let path_ubl = out_dir.join(format!("facturx_ubl_{}.pdf", suffix));
            std::fs::write(&path_ubl, &result_ubl.pdf).expect("Écriture PDF échouée");
            println!("  UBL → Factur-X : {} ({} Ko)", path_ubl.display(), result_ubl.pdf.len() / 1024);

            // 3. Sauvegarder les XML CII embarqués (source CII et source UBL)
            let xml_path = out_dir.join(format!("facturx_cii_{}.xml", suffix));
            std::fs::write(&xml_path, &result_cii.cii_xml).expect("Écriture XML échouée");
            println!("  XML CII embarqué (CII) : {}", xml_path.display());

            let xml_ubl_path = out_dir.join(format!("facturx_ubl_{}.xml", suffix));
            std::fs::write(&xml_ubl_path, &result_ubl.cii_xml).expect("Écriture XML échouée");
            println!("  XML CII embarqué (UBL) : {}", xml_ubl_path.display());
        }

        println!("\n📂 Fichiers générés dans : {}", out_dir.display());
    }

    // =====================================================================
    // Tests de détection de profil et cohérence XMP
    // =====================================================================

    #[test]
    fn test_detect_level_extended() {
        let xml = r#"<ram:ID>urn:cen.eu:en16931:2017#conformant#urn:factur-x.eu:1p0:extended</ram:ID>"#;
        assert_eq!(FacturXGenerator::detect_level_from_xml(xml), Some(FacturXLevel::Extended));
    }

    #[test]
    fn test_detect_level_en16931() {
        let xml = r#"<ram:ID>urn:cen.eu:en16931:2017</ram:ID>"#;
        assert_eq!(FacturXGenerator::detect_level_from_xml(xml), Some(FacturXLevel::EN16931));
    }

    #[test]
    fn test_detect_level_basic() {
        let xml = r#"<ram:ID>urn:cen.eu:en16931:2017#compliant#urn:factur-x.eu:1p0:basic</ram:ID>"#;
        assert_eq!(FacturXGenerator::detect_level_from_xml(xml), Some(FacturXLevel::Basic));
    }

    #[test]
    fn test_detect_level_basicwl() {
        let xml = r#"<ram:ID>urn:cen.eu:en16931:2017#compliant#urn:factur-x.eu:1p0:basicwl</ram:ID>"#;
        assert_eq!(FacturXGenerator::detect_level_from_xml(xml), Some(FacturXLevel::BasicWL));
    }

    #[test]
    fn test_detect_level_minimum() {
        let xml = r#"<ram:ID>urn:cen.eu:en16931:2017#compliant#urn:factur-x.eu:1p0:minimum</ram:ID>"#;
        assert_eq!(FacturXGenerator::detect_level_from_xml(xml), Some(FacturXLevel::Minimum));
    }

    #[test]
    fn test_detect_level_unknown() {
        let xml = r#"<ram:ID>some:unknown:profile</ram:ID>"#;
        assert_eq!(FacturXGenerator::detect_level_from_xml(xml), None);
    }

    /// Vérifie la cohérence XMP ↔ XML pour chaque profil Factur-X.
    #[test]
    fn test_xmp_level_matches_xml_profile() {
        let profiles: &[(&str, FacturXLevel)] = &[
            ("MINIMUM", FacturXLevel::Minimum),
            ("BASIC WL", FacturXLevel::BasicWL),
            ("BASIC", FacturXLevel::Basic),
            ("EN 16931", FacturXLevel::EN16931),
            ("EXTENDED", FacturXLevel::Extended),
        ];
        let gen = generator();
        for (expected_str, level) in profiles {
            let xmp = gen.build_xmp_metadata("TEST-001", *level);
            let tag = format!("fx:ConformanceLevel>{}</fx:ConformanceLevel>", expected_str);
            assert!(
                xmp.contains(&tag),
                "XMP pour {:?} doit contenir '{}', XMP={}",
                level, tag, &xmp[..200]
            );
        }
    }

    // =====================================================================
    // Tests Factur-X avec différents profils (CII)
    // =====================================================================

    /// Helper : modifie le profile_id dans le XML CII et le InvoiceData parsé.
    fn set_cii_profile(xml: &str, invoice: &mut InvoiceData, profile_uri: &str) -> String {
        let new_xml = xml.replace(
            "urn:cen.eu:en16931:2017#conformant#urn:factur-x.eu:1p0:extended",
            profile_uri,
        );
        invoice.profile_id = Some(profile_uri.to_string());
        invoice.raw_xml = Some(new_xml.clone());
        new_xml
    }

    #[test]
    fn test_facturx_cii_profile_extended() {
        let cii_xml = std::fs::read_to_string("../../tests/fixtures/cii/facture_cii_001.xml")
            .expect("Fixture CII introuvable");
        let invoice = pdp_invoice::CiiParser::new().parse(&cii_xml).expect("Parse CII");

        let result = generator().generate(&invoice).expect("Génération échouée");
        assert_eq!(result.level, FacturXLevel::Extended);

        let doc = lopdf::Document::load_mem(&result.pdf).expect("Relecture PDF");
        let xmp = extract_xmp(&doc);
        assert!(xmp.contains("fx:ConformanceLevel>EXTENDED</fx:ConformanceLevel>"));
    }

    #[test]
    fn test_facturx_cii_profile_en16931() {
        let cii_xml = std::fs::read_to_string("../../tests/fixtures/cii/facture_cii_001.xml")
            .expect("Fixture CII introuvable");
        let mut invoice = pdp_invoice::CiiParser::new().parse(&cii_xml).expect("Parse CII");
        set_cii_profile(&cii_xml, &mut invoice, "urn:cen.eu:en16931:2017");

        let result = generator().generate(&invoice).expect("Génération échouée");
        assert_eq!(result.level, FacturXLevel::EN16931);

        let doc = lopdf::Document::load_mem(&result.pdf).expect("Relecture PDF");
        let xmp = extract_xmp(&doc);
        assert!(xmp.contains("fx:ConformanceLevel>EN 16931</fx:ConformanceLevel>"));
    }

    #[test]
    fn test_facturx_cii_profile_basic() {
        let cii_xml = std::fs::read_to_string("../../tests/fixtures/cii/facture_cii_001.xml")
            .expect("Fixture CII introuvable");
        let mut invoice = pdp_invoice::CiiParser::new().parse(&cii_xml).expect("Parse CII");
        set_cii_profile(&cii_xml, &mut invoice, "urn:cen.eu:en16931:2017#compliant#urn:factur-x.eu:1p0:basic");

        let result = generator().generate(&invoice).expect("Génération échouée");
        assert_eq!(result.level, FacturXLevel::Basic);

        let doc = lopdf::Document::load_mem(&result.pdf).expect("Relecture PDF");
        let xmp = extract_xmp(&doc);
        assert!(xmp.contains("fx:ConformanceLevel>BASIC</fx:ConformanceLevel>"));
    }

    // =====================================================================
    // Tests Factur-X depuis UBL avec vérification profil
    // =====================================================================

    #[test]
    fn test_facturx_ubl_profile_extended() {
        let ubl_xml = std::fs::read_to_string("../../tests/fixtures/ubl/facture_ubl_001.xml")
            .expect("Fixture UBL introuvable");
        let invoice = pdp_invoice::UblParser::new().parse(&ubl_xml).expect("Parse UBL");

        let result = generator().generate(&invoice).expect("Génération échouée");
        // Le XSLT copie le profil UBL → CII, vérifier la cohérence
        let detected = FacturXGenerator::detect_level_from_xml(&result.cii_xml);
        assert_eq!(result.level, detected.unwrap_or(FacturXLevel::EN16931));

        let doc = lopdf::Document::load_mem(&result.pdf).expect("Relecture PDF");
        let xmp = extract_xmp(&doc);
        let expected_tag = format!(
            "fx:ConformanceLevel>{}</fx:ConformanceLevel>",
            result.level.as_str()
        );
        assert!(xmp.contains(&expected_tag),
            "XMP doit contenir {} pour profil {:?}", expected_tag, result.level);
    }

    // =====================================================================
    // Tests Factur-X avec pièces jointes + vérification profil
    // =====================================================================

    #[test]
    fn test_facturx_with_pdf_attachment_valid() {
        let cii_xml = std::fs::read_to_string("../../tests/fixtures/cii/facture_cii_001.xml")
            .expect("Fixture CII introuvable");
        let mut invoice = pdp_invoice::CiiParser::new().parse(&cii_xml).expect("Parse CII");

        // Générer un vrai petit PDF via lopdf
        let attachment_pdf = make_valid_pdf("Bon de commande BC-001");

        invoice.attachments.push(InvoiceAttachment {
            id: Some("BC-001".to_string()),
            description: Some("Bon de commande".to_string()),
            external_uri: None,
            embedded_content: Some(attachment_pdf),
            mime_code: Some("application/pdf".to_string()),
            filename: Some("bon_commande.pdf".to_string()),
        });

        let result = generator().generate(&invoice).expect("Génération échouée");
        assert_eq!(result.level, FacturXLevel::Extended);

        // Vérifier que le PDF embarqué est trouvable
        let doc = lopdf::Document::load_mem(&result.pdf).expect("Relecture PDF");
        assert!(find_embedded_file(&doc, b"bon_commande.pdf"), "bon_commande.pdf doit être embarqué");
        assert!(find_embedded_file(&doc, b"factur-x.xml"), "factur-x.xml doit être embarqué");
    }

    #[test]
    fn test_facturx_with_csv_attachment() {
        let cii_xml = std::fs::read_to_string("../../tests/fixtures/cii/facture_cii_001.xml")
            .expect("Fixture CII introuvable");
        let mut invoice = pdp_invoice::CiiParser::new().parse(&cii_xml).expect("Parse CII");

        invoice.attachments.push(InvoiceAttachment {
            id: Some("DET-001".to_string()),
            description: Some("Détail prestations".to_string()),
            external_uri: None,
            embedded_content: Some(b"Article;Qte;PU\nConseil;10;100.00\n".to_vec()),
            mime_code: Some("text/csv".to_string()),
            filename: Some("detail.csv".to_string()),
        });

        let result = generator().generate(&invoice).expect("Génération échouée");

        let doc = lopdf::Document::load_mem(&result.pdf).expect("Relecture PDF");
        assert!(find_embedded_file(&doc, b"detail.csv"), "detail.csv doit être embarqué");
        assert!(find_embedded_file(&doc, b"factur-x.xml"), "factur-x.xml doit être embarqué");
    }

    #[test]
    fn test_facturx_with_external_uri_only() {
        let cii_xml = std::fs::read_to_string("../../tests/fixtures/cii/facture_cii_001.xml")
            .expect("Fixture CII introuvable");
        let mut invoice = pdp_invoice::CiiParser::new().parse(&cii_xml).expect("Parse CII");

        // PJ avec URI externe uniquement (pas de contenu embarqué)
        invoice.attachments.push(InvoiceAttachment {
            id: Some("DEVIS-001".to_string()),
            description: Some("Devis original".to_string()),
            external_uri: Some("https://example.com/devis/001.pdf".to_string()),
            embedded_content: None,
            mime_code: None,
            filename: None,
        });

        let result = generator().generate(&invoice).expect("Génération échouée");

        // Seul factur-x.xml doit être embarqué (pas de fichier pour l'URI externe)
        let doc = lopdf::Document::load_mem(&result.pdf).expect("Relecture PDF");
        assert!(find_embedded_file(&doc, b"factur-x.xml"), "factur-x.xml doit être embarqué");
    }

    #[test]
    fn test_facturx_with_multiple_attachments_and_profile() {
        let cii_xml = std::fs::read_to_string("../../tests/fixtures/cii/facture_cii_001.xml")
            .expect("Fixture CII introuvable");
        let mut invoice = pdp_invoice::CiiParser::new().parse(&cii_xml).expect("Parse CII");

        // 3 pièces jointes : PDF, CSV, URI externe
        invoice.attachments.push(InvoiceAttachment {
            id: Some("BC-001".to_string()),
            description: Some("Bon de commande".to_string()),
            external_uri: None,
            embedded_content: Some(make_valid_pdf("Bon de commande")),
            mime_code: Some("application/pdf".to_string()),
            filename: Some("bon_commande.pdf".to_string()),
        });
        invoice.attachments.push(InvoiceAttachment {
            id: Some("DET-001".to_string()),
            description: Some("Détail".to_string()),
            external_uri: None,
            embedded_content: Some(b"col1;col2\nval1;val2\n".to_vec()),
            mime_code: Some("text/csv".to_string()),
            filename: Some("detail.csv".to_string()),
        });
        invoice.attachments.push(InvoiceAttachment {
            id: Some("EXT-001".to_string()),
            description: Some("Externe".to_string()),
            external_uri: Some("https://example.com/doc.pdf".to_string()),
            embedded_content: None,
            mime_code: None,
            filename: None,
        });

        let result = generator().generate(&invoice).expect("Génération échouée");
        assert_eq!(result.level, FacturXLevel::Extended);

        let doc = lopdf::Document::load_mem(&result.pdf).expect("Relecture PDF");
        assert!(find_embedded_file(&doc, b"factur-x.xml"));
        assert!(find_embedded_file(&doc, b"bon_commande.pdf"));
        assert!(find_embedded_file(&doc, b"detail.csv"));

        // Vérifier la cohérence XMP
        let xmp = extract_xmp(&doc);
        assert!(xmp.contains("fx:ConformanceLevel>EXTENDED</fx:ConformanceLevel>"));
        assert!(xmp.contains("pdfaid:part>3</pdfaid:part>"));
    }

    #[test]
    fn test_facturx_ubl_with_attachments() {
        let ubl_xml = std::fs::read_to_string("../../tests/fixtures/ubl/facture_ubl_001.xml")
            .expect("Fixture UBL introuvable");
        let mut invoice = pdp_invoice::UblParser::new().parse(&ubl_xml).expect("Parse UBL");

        invoice.attachments.push(InvoiceAttachment {
            id: Some("PJ-001".to_string()),
            description: Some("Pièce jointe UBL".to_string()),
            external_uri: None,
            embedded_content: Some(make_valid_pdf("PJ depuis UBL")),
            mime_code: Some("application/pdf".to_string()),
            filename: Some("pj_ubl.pdf".to_string()),
        });

        let result = generator().generate(&invoice).expect("Génération échouée");

        let doc = lopdf::Document::load_mem(&result.pdf).expect("Relecture PDF");
        assert!(find_embedded_file(&doc, b"factur-x.xml"));
        assert!(find_embedded_file(&doc, b"pj_ubl.pdf"));

        // Vérifier cohérence profil XMP ↔ XML CII embarqué
        let xmp = extract_xmp(&doc);
        let detected = FacturXGenerator::detect_level_from_xml(&result.cii_xml);
        if let Some(level) = detected {
            let tag = format!("fx:ConformanceLevel>{}</fx:ConformanceLevel>", level.as_str());
            assert!(xmp.contains(&tag), "XMP profil incohérent avec XML CII embarqué");
        }
    }

    #[test]
    fn test_facturx_no_attachments_still_has_xml() {
        let cii_xml = std::fs::read_to_string("../../tests/fixtures/cii/facture_cii_001.xml")
            .expect("Fixture CII introuvable");
        let invoice = pdp_invoice::CiiParser::new().parse(&cii_xml).expect("Parse CII");
        assert!(invoice.attachments.is_empty());

        let result = generator().generate(&invoice).expect("Génération échouée");

        let doc = lopdf::Document::load_mem(&result.pdf).expect("Relecture PDF");
        assert!(find_embedded_file(&doc, b"factur-x.xml"),
            "Même sans PJ, factur-x.xml doit être embarqué");
    }

    // =====================================================================
    // Helpers pour les tests
    // =====================================================================

    /// Génère un vrai petit PDF valide via lopdf.
    fn make_valid_pdf(title: &str) -> Vec<u8> {
        use lopdf::{Document, Stream};
        use lopdf::dictionary;

        let mut doc = Document::with_version("1.4");
        let font_id = doc.add_object(dictionary! {
            "Type" => "Font",
            "Subtype" => "Type1",
            "BaseFont" => "Helvetica",
        });
        let content = format!("BT /F1 12 Tf 50 750 Td ({}) Tj ET", title);
        let content_id = doc.add_object(Stream::new(dictionary! {}, content.into_bytes()));
        let resources_id = doc.add_object(dictionary! {
            "Font" => dictionary! { "F1" => font_id },
        });
        let page_id = doc.add_object(dictionary! {
            "Type" => "Page",
            "MediaBox" => vec![0.into(), 0.into(), 595.into(), 842.into()],
            "Contents" => content_id,
            "Resources" => resources_id,
        });
        let pages_id = doc.add_object(dictionary! {
            "Type" => "Pages",
            "Kids" => vec![page_id.into()],
            "Count" => 1,
        });
        doc.get_object_mut(page_id).unwrap()
            .as_dict_mut().unwrap()
            .set("Parent", pages_id);
        let catalog_id = doc.add_object(dictionary! {
            "Type" => "Catalog",
            "Pages" => pages_id,
        });
        doc.trailer.set("Root", catalog_id);
        let mut buf = Vec::new();
        doc.save_to(&mut buf).expect("Génération PDF");
        buf
    }

    /// Extrait le XMP metadata d'un document PDF.
    fn extract_xmp(doc: &lopdf::Document) -> String {
        let catalog_id = doc.trailer.get(b"Root").unwrap()
            .as_reference().unwrap();
        let catalog = doc.get_object(catalog_id).unwrap()
            .as_dict().unwrap();
        let metadata_id = catalog.get(b"Metadata").unwrap()
            .as_reference().unwrap();
        let metadata_obj = doc.get_object(metadata_id).unwrap();
        match metadata_obj {
            lopdf::Object::Stream(ref stream) => {
                let mut s = stream.clone();
                let _ = s.decompress();
                String::from_utf8_lossy(&s.content).to_string()
            }
            _ => panic!("Metadata doit être un Stream"),
        }
    }

    /// Vérifie qu'un fichier embarqué existe dans le PDF.
    fn find_embedded_file(doc: &lopdf::Document, filename: &[u8]) -> bool {
        for (_id, obj) in doc.objects.iter() {
            if let Ok(dict) = obj.as_dict() {
                if let Ok(f) = dict.get(b"F") {
                    if let Ok(s) = f.as_str() {
                        if s == filename { return true; }
                    }
                }
            }
        }
        false
    }
}
