use pdp_core::error::{PdpError, PdpResult};
use pdp_core::model::{InvoiceAttachment, InvoiceData, InvoiceFormat};

/// Parser pour les factures Factur-X (PDF avec XML embarqué)
/// Factur-X est un PDF/A-3 contenant un fichier XML CII en pièce jointe.
pub struct FacturXParser;

impl FacturXParser {
    pub fn new() -> Self {
        Self
    }

    /// Parse un PDF Factur-X et en extrait les données de facture
    pub fn parse(&self, pdf_data: &[u8]) -> PdpResult<InvoiceData> {
        // Vérifier que c'est bien un PDF
        if pdf_data.len() < 5 || &pdf_data[0..5] != b"%PDF-" {
            return Err(PdpError::ParseError(
                "Le fichier n'est pas un PDF valide".to_string(),
            ));
        }

        // Extraire le XML embarqué du PDF
        let xml = self.extract_xml_from_pdf(pdf_data)?;

        // Parser le XML CII extrait
        let cii_parser = super::cii::CiiParser::new();
        let mut invoice = cii_parser.parse(&xml)?;

        // Mettre à jour le format source
        invoice.source_format = InvoiceFormat::FacturX;
        invoice.raw_pdf = Some(pdf_data.to_vec());

        // Extraire les pièces jointes embarquées du PDF (autres que factur-x.xml)
        // On vide d'abord les PJ parsées depuis le XML CII (elles n'ont que le base64),
        // car les Filespec du PDF contiennent le contenu binaire réel.
        invoice.attachments.clear();
        let doc = lopdf::Document::load_mem(pdf_data)
            .map_err(|e| PdpError::ParseError(format!("PDF invalide: {}", e)))?;
        self.extract_attachments(&doc, &mut invoice);

        tracing::info!(
            invoice_number = %invoice.invoice_number,
            attachments = invoice.attachments.len(),
            "Facture Factur-X parsée (XML CII extrait du PDF)"
        );

        Ok(invoice)
    }

    /// Extrait le fichier XML embarqué dans le PDF
    /// Les fichiers Factur-X contiennent un XML nommé "factur-x.xml" ou "ZUGFeRD-invoice.xml"
    fn extract_xml_from_pdf(&self, pdf_data: &[u8]) -> PdpResult<String> {
        let doc = lopdf::Document::load_mem(pdf_data)
            .map_err(|e| PdpError::ParseError(format!("PDF invalide: {}", e)))?;

        // Chercher les fichiers embarqués (EmbeddedFiles)
        // Dans un PDF/A-3, les pièces jointes sont dans le catalogue -> Names -> EmbeddedFiles
        let xml_content = self.find_embedded_xml(&doc)?;

        Ok(xml_content)
    }

    /// Cherche le XML embarqué dans les objets du PDF
    fn find_embedded_xml(&self, doc: &lopdf::Document) -> PdpResult<String> {
        // Parcourir tous les objets du PDF pour trouver les streams
        // qui contiennent du XML CII ou Factur-X
        for (_id, object) in doc.objects.iter() {
            if let Ok(stream) = object.as_stream() {
                // Essayer le contenu décompressé, sinon le contenu brut
                let content = stream
                    .decompressed_content()
                    .unwrap_or_else(|_| stream.content.clone());
                if let Ok(text) = std::str::from_utf8(&content) {
                    if text.contains("CrossIndustryInvoice") || text.contains("uncefact") {
                        tracing::debug!("XML CII trouvé dans le PDF");
                        return Ok(text.to_string());
                    }
                }
            }
        }

        // Méthode alternative : chercher dans les noms de fichiers embarqués
        // via le dictionnaire Names -> EmbeddedFiles
        Err(PdpError::ParseError(
            "Aucun XML Factur-X/CII trouvé dans le PDF. \
             Vérifiez que le PDF contient bien un fichier factur-x.xml embarqué."
                .to_string(),
        ))
    }

    /// Extrait les pièces jointes embarquées du PDF (hors factur-x.xml / ZUGFeRD-invoice.xml)
    fn extract_attachments(&self, doc: &lopdf::Document, invoice: &mut InvoiceData) {
        use lopdf::Object;

        // Parcourir tous les objets pour trouver les Filespec
        for (_id, object) in doc.objects.iter() {
            let dict = match object {
                Object::Dictionary(d) => d,
                _ => continue,
            };

            // Vérifier que c'est un Filespec
            let is_filespec = dict.get(b"Type").ok()
                .and_then(|o| o.as_name().ok())
                .map(|n| n == b"Filespec")
                .unwrap_or(false);
            if !is_filespec {
                continue;
            }

            // Extraire le nom de fichier
            let filename = dict.get(b"F")
                .or_else(|_| dict.get(b"UF"))
                .ok()
                .and_then(|o| match o {
                    Object::String(s, _) => String::from_utf8(s.clone()).ok(),
                    _ => None,
                })
                .unwrap_or_default();

            // Ignorer factur-x.xml et ZUGFeRD-invoice.xml (c'est le XML CII, pas une PJ)
            let fname_lower = filename.to_lowercase();
            if fname_lower == "factur-x.xml" || fname_lower == "zugferd-invoice.xml" {
                continue;
            }
            if filename.is_empty() {
                continue;
            }

            // Extraire la description
            let description = dict.get(b"Desc")
                .ok()
                .and_then(|o| match o {
                    Object::String(s, _) => String::from_utf8(s.clone()).ok(),
                    _ => None,
                });

            // Extraire le contenu du stream embarqué via EF -> F
            let mut embedded_content = None;
            let mut mime_code = None;

            if let Ok(ef_dict) = dict.get(b"EF").and_then(|o| o.as_dict()) {
                if let Ok(stream_ref) = ef_dict.get(b"F").and_then(|o| o.as_reference()) {
                    if let Ok(stream_obj) = doc.get_object(stream_ref) {
                        if let Ok(stream) = stream_obj.as_stream() {
                            let content = stream
                                .decompressed_content()
                                .unwrap_or_else(|_| stream.content.clone());
                            embedded_content = Some(content);

                            // Extraire le type MIME du stream
                            mime_code = stream.dict.get(b"Subtype").ok()
                                .and_then(|o| o.as_name().ok())
                                .map(|n| String::from_utf8_lossy(n).to_string());
                        }
                    }
                }
            }

            invoice.attachments.push(InvoiceAttachment {
                id: None,
                description,
                external_uri: None,
                embedded_content,
                mime_code,
                filename: Some(filename),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reject_non_pdf() {
        let parser = FacturXParser::new();
        let result = parser.parse(b"not a pdf");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("PDF valide"));
    }

    #[test]
    fn test_reject_empty_pdf() {
        let parser = FacturXParser::new();
        let result = parser.parse(b"%PDF-1.4 minimal but no embedded xml");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_facturx_standard() {
        let pdf = std::fs::read("../../tests/fixtures/facturx/facture_facturx_001.pdf")
            .expect("Fixture Factur-X introuvable — lancer: cargo test -p pdp-invoice --test generate_facturx_fixtures -- --ignored");

        let parser = FacturXParser::new();
        let invoice = parser.parse(&pdf).expect("Parsing Factur-X échoué");

        assert_eq!(invoice.source_format, InvoiceFormat::FacturX);
        assert_eq!(invoice.invoice_number, "FA-2025-00256");
        assert_eq!(invoice.invoice_type_code.as_deref(), Some("380"));
        assert_eq!(invoice.seller_name.as_deref(), Some("InfoTech Solutions SARL"));
        assert_eq!(invoice.buyer_name.as_deref(), Some("Manufacture Lyonnaise SAS"));
        assert_eq!(invoice.currency.as_deref(), Some("EUR"));
        assert_eq!(invoice.total_ht, Some(32000.00));
        assert_eq!(invoice.total_ttc, Some(38400.00));
        assert_eq!(invoice.total_tax, Some(6400.00));
        assert_eq!(invoice.lines.len(), 3);
        assert!(invoice.raw_pdf.is_some());
    }

    #[test]
    fn test_parse_facturx_avoir() {
        let pdf = std::fs::read("../../tests/fixtures/facturx/avoir_facturx_381.pdf")
            .expect("Fixture Factur-X avoir introuvable");

        let parser = FacturXParser::new();
        let invoice = parser.parse(&pdf).expect("Parsing Factur-X avoir échoué");

        assert_eq!(invoice.source_format, InvoiceFormat::FacturX);
        assert_eq!(invoice.invoice_type_code.as_deref(), Some("381"));
    }

    #[test]
    fn test_parse_facturx_rectificative() {
        let pdf = std::fs::read("../../tests/fixtures/facturx/facture_rectificative_facturx_384.pdf")
            .expect("Fixture Factur-X rectificative introuvable");

        let parser = FacturXParser::new();
        let invoice = parser.parse(&pdf).expect("Parsing Factur-X rectificative échoué");

        assert_eq!(invoice.source_format, InvoiceFormat::FacturX);
        assert_eq!(invoice.invoice_type_code.as_deref(), Some("384"));
    }
}
