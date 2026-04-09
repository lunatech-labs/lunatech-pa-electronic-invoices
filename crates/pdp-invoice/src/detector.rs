use pdp_core::error::{PdpError, PdpResult};
use pdp_core::model::{DocumentType, InvoiceFormat};

/// Détecte le format d'une facture à partir de son contenu
pub struct FormatDetector;

impl FormatDetector {
    pub fn new() -> Self {
        Self
    }
}

/// Détecte le type de document (facture, CDAR, e-reporting) à partir du contenu brut.
///
/// Cette fonction doit être appelée **avant** `detect_format` pour router
/// correctement les CDAR entrants (qui ne sont pas des factures).
pub fn detect_document_type(data: &[u8]) -> DocumentType {
    // PDF → forcément une facture (Factur-X)
    if data.len() >= 5 && &data[0..5] == b"%PDF-" {
        return DocumentType::Invoice;
    }

    let text = match std::str::from_utf8(data) {
        Ok(t) => t,
        Err(_) => return DocumentType::Unknown,
    };

    detect_document_type_from_xml(text)
}

/// Détecte le type de document à partir du contenu XML
fn detect_document_type_from_xml(xml: &str) -> DocumentType {
    // CDAR : CrossDomainAcknowledgementAndResponse (D22B)
    if xml.contains("CrossDomainAcknowledgementAndResponse")
        || xml.contains("uncefact:data:standard:CrossDomainAcknowledgementAndResponse")
    {
        return DocumentType::Cdar;
    }

    // E-Reporting : Report / PaymentsReport / TransactionsReport
    if xml.contains("<Report") || xml.contains("<PaymentsReport") || xml.contains("<TransactionsReport") {
        return DocumentType::EReporting;
    }

    // Factures : CII, UBL, ou autre XML
    if xml.contains("CrossIndustryInvoice")
        || xml.contains("oasis:names:specification:ubl:schema:xsd:Invoice")
        || xml.contains("oasis:names:specification:ubl:schema:xsd:CreditNote")
        || xml.contains("<Invoice ")
        || xml.contains("<CreditNote ")
    {
        return DocumentType::Invoice;
    }

    DocumentType::Unknown
}

/// Détecte le format d'un fichier facture à partir de son contenu brut
pub fn detect_format(data: &[u8]) -> PdpResult<InvoiceFormat> {
    // Vérifier si c'est un PDF (Factur-X)
    if data.len() >= 5 && &data[0..5] == b"%PDF-" {
        return Ok(InvoiceFormat::FacturX);
    }

    // Sinon, essayer de lire comme du texte XML
    let text = std::str::from_utf8(data)
        .map_err(|e| PdpError::ParseError(format!("Contenu non UTF-8: {}", e)))?;

    detect_format_from_xml(text)
}

/// Détecte le format à partir du contenu XML
fn detect_format_from_xml(xml: &str) -> PdpResult<InvoiceFormat> {
    // CII : présence du namespace CrossIndustryInvoice
    if xml.contains("CrossIndustryInvoice") || xml.contains("uncefact:data:standard:CrossIndustryInvoice") {
        return Ok(InvoiceFormat::CII);
    }

    // UBL : présence du namespace UBL Invoice ou CreditNote
    if xml.contains("oasis:names:specification:ubl:schema:xsd:Invoice")
        || xml.contains("oasis:names:specification:ubl:schema:xsd:CreditNote")
        || xml.contains("<Invoice ")
        || xml.contains("<CreditNote ")
    {
        return Ok(InvoiceFormat::UBL);
    }

    Err(PdpError::UnsupportedFormat(
        "Impossible de détecter le format de la facture (ni UBL, ni CII, ni PDF)".to_string(),
    ))
}

/// Détecte le format à partir du nom de fichier (heuristique)
pub fn detect_format_from_filename(filename: &str) -> Option<InvoiceFormat> {
    let lower = filename.to_lowercase();
    if lower.ends_with(".pdf") {
        Some(InvoiceFormat::FacturX)
    } else if lower.contains("ubl") {
        Some(InvoiceFormat::UBL)
    } else if lower.contains("cii") || lower.contains("facturx") {
        Some(InvoiceFormat::CII)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_ubl() {
        let xml = r#"<?xml version="1.0"?>
        <Invoice xmlns="urn:oasis:names:specification:ubl:schema:xsd:Invoice-2">
            <cbc:ID>TEST</cbc:ID>
        </Invoice>"#;
        let result = detect_format(xml.as_bytes()).unwrap();
        assert_eq!(result, InvoiceFormat::UBL);
    }

    #[test]
    fn test_detect_cii() {
        let xml = r#"<?xml version="1.0"?>
        <rsm:CrossIndustryInvoice xmlns:rsm="urn:un:unece:uncefact:data:standard:CrossIndustryInvoice:100">
        </rsm:CrossIndustryInvoice>"#;
        let result = detect_format(xml.as_bytes()).unwrap();
        assert_eq!(result, InvoiceFormat::CII);
    }

    #[test]
    fn test_detect_pdf() {
        let data = b"%PDF-1.4 fake pdf content";
        let result = detect_format(data).unwrap();
        assert_eq!(result, InvoiceFormat::FacturX);
    }

    #[test]
    fn test_detect_unknown() {
        let data = b"<html><body>Not an invoice</body></html>";
        let result = detect_format(data);
        assert!(result.is_err());
    }

    #[test]
    fn test_detect_credit_note_ubl() {
        let xml = r#"<?xml version="1.0"?>
        <CreditNote xmlns="urn:oasis:names:specification:ubl:schema:xsd:CreditNote-2">
        </CreditNote>"#;
        let result = detect_format(xml.as_bytes()).unwrap();
        assert_eq!(result, InvoiceFormat::UBL);
    }

    // ===== Tests détection sur fixtures réelles =====

    #[test]
    fn test_detect_real_cii_fixture() {
        let data = std::fs::read("../../tests/fixtures/cii/facture_cii_001.xml").unwrap();
        assert_eq!(detect_format(&data).unwrap(), InvoiceFormat::CII);
    }

    #[test]
    fn test_detect_real_ubl_fixture() {
        let data = std::fs::read("../../tests/fixtures/ubl/facture_ubl_001.xml").unwrap();
        assert_eq!(detect_format(&data).unwrap(), InvoiceFormat::UBL);
    }

    #[test]
    fn test_detect_real_facturx_fixture() {
        let data = std::fs::read("../../tests/fixtures/facturx/facture_facturx_001.pdf").unwrap();
        assert_eq!(detect_format(&data).unwrap(), InvoiceFormat::FacturX);
    }

    #[test]
    fn test_detect_real_cii_avoir() {
        let data = std::fs::read("../../tests/fixtures/cii/avoir_cii_381.xml").unwrap();
        assert_eq!(detect_format(&data).unwrap(), InvoiceFormat::CII);
    }

    #[test]
    fn test_detect_real_ubl_avoir() {
        let data = std::fs::read("../../tests/fixtures/ubl/facture_ubl_002_avoir.xml").unwrap();
        assert_eq!(detect_format(&data).unwrap(), InvoiceFormat::UBL);
    }

    #[test]
    fn test_detect_real_error_fixture() {
        let data = std::fs::read("../../tests/fixtures/errors/facture_invalide_001.xml").unwrap();
        assert_eq!(detect_format(&data).unwrap(), InvoiceFormat::CII);
    }

    #[test]
    fn test_detect_format_from_filename_pdf() {
        assert_eq!(detect_format_from_filename("facture.pdf"), Some(InvoiceFormat::FacturX));
        assert_eq!(detect_format_from_filename("FACTURE.PDF"), Some(InvoiceFormat::FacturX));
    }

    #[test]
    fn test_detect_format_from_filename_ubl() {
        assert_eq!(detect_format_from_filename("facture_ubl_001.xml"), Some(InvoiceFormat::UBL));
    }

    #[test]
    fn test_detect_format_from_filename_cii() {
        assert_eq!(detect_format_from_filename("facture_cii_001.xml"), Some(InvoiceFormat::CII));
    }

    #[test]
    fn test_detect_format_from_filename_unknown() {
        assert_eq!(detect_format_from_filename("data.csv"), None);
        assert_eq!(detect_format_from_filename("readme.txt"), None);
    }

    // ===== Tests detect_document_type =====

    #[test]
    fn test_detect_document_type_cdar() {
        let xml = r#"<?xml version="1.0"?>
        <rsm:CrossDomainAcknowledgementAndResponse xmlns:rsm="urn:un:unece:uncefact:data:standard:CrossDomainAcknowledgementAndResponse:100">
        </rsm:CrossDomainAcknowledgementAndResponse>"#;
        assert_eq!(detect_document_type(xml.as_bytes()), DocumentType::Cdar);
    }

    #[test]
    fn test_detect_document_type_cii_invoice() {
        let xml = r#"<?xml version="1.0"?>
        <rsm:CrossIndustryInvoice xmlns:rsm="urn:un:unece:uncefact:data:standard:CrossIndustryInvoice:100">
        </rsm:CrossIndustryInvoice>"#;
        assert_eq!(detect_document_type(xml.as_bytes()), DocumentType::Invoice);
    }

    #[test]
    fn test_detect_document_type_ubl_invoice() {
        let xml = r#"<Invoice xmlns="urn:oasis:names:specification:ubl:schema:xsd:Invoice-2"/>"#;
        assert_eq!(detect_document_type(xml.as_bytes()), DocumentType::Invoice);
    }

    #[test]
    fn test_detect_document_type_pdf() {
        let data = b"%PDF-1.4 fake pdf";
        assert_eq!(detect_document_type(data), DocumentType::Invoice);
    }

    #[test]
    fn test_detect_document_type_unknown() {
        let data = b"<html>not a document</html>";
        assert_eq!(detect_document_type(data), DocumentType::Unknown);
    }

    #[test]
    fn test_detect_document_type_real_cdar_fixture() {
        let data = std::fs::read("../../tests/fixtures/cdar/cdv_200_deposee.xml").unwrap();
        assert_eq!(detect_document_type(&data), DocumentType::Cdar);
    }
}
