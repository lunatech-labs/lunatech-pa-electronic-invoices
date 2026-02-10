use async_trait::async_trait;
use pdp_core::error::{PdpError, PdpResult};
use pdp_core::exchange::Exchange;
use pdp_core::model::FlowStatus;
use pdp_core::processor::Processor;

use crate::detector::detect_format;
use crate::validator::InvoiceValidator;

/// Processor de parsing : détecte le format et parse la facture.
/// Remplit `exchange.invoice` avec les données parsées.
pub struct ParseProcessor;

impl ParseProcessor {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ParseProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Processor for ParseProcessor {
    fn name(&self) -> &str {
        "ParseProcessor"
    }

    async fn process(&self, mut exchange: Exchange) -> PdpResult<Exchange> {
        // Skip si le document n'est pas une facture (CDAR, e-reporting, etc.)
        if exchange.get_header("document.type").map(|s| s.as_str()) == Some("CDAR") {
            tracing::debug!(exchange_id = %exchange.id, "ParseProcessor: skip (document CDAR)");
            return Ok(exchange);
        }

        exchange.set_status(FlowStatus::Parsing);

        let format = detect_format(&exchange.body)?;
        exchange.set_header("invoice.format", &format.to_string());

        tracing::info!(
            exchange_id = %exchange.id,
            filename = exchange.source_filename.as_deref().unwrap_or("N/A"),
            format = %format,
            "Parsing de la facture"
        );

        let invoice = match format {
            pdp_core::model::InvoiceFormat::UBL => {
                let xml = exchange.body_as_str()?;
                crate::ubl::UblParser::new().parse(xml)?
            }
            pdp_core::model::InvoiceFormat::CII => {
                let xml = exchange.body_as_str()?;
                crate::cii::CiiParser::new().parse(xml)?
            }
            pdp_core::model::InvoiceFormat::FacturX => {
                crate::facturx::FacturXParser::new().parse(&exchange.body)?
            }
        };

        exchange.set_header("invoice.number", &invoice.invoice_number);
        exchange.set_header("invoice.key", &invoice.key_string());
        if let Some(ref seller) = invoice.seller_name {
            exchange.set_header("invoice.seller", seller);
        }
        if let Some(ref buyer) = invoice.buyer_name {
            exchange.set_header("invoice.buyer", buyer);
        }

        exchange.invoice = Some(invoice);
        exchange.set_status(FlowStatus::Parsed);

        Ok(exchange)
    }
}

/// Processor de validation : valide la facture parsée selon EN16931 et règles métier.
/// Ajoute les résultats de validation dans les propriétés de l'exchange.
/// En cas d'erreur fatale, marque l'exchange en erreur.
pub struct ValidateProcessor {
    /// Si true, les erreurs de validation bloquent le flux (exchange en erreur)
    strict: bool,
}

impl ValidateProcessor {
    pub fn new() -> Self {
        Self { strict: true }
    }

    pub fn lenient() -> Self {
        Self { strict: false }
    }

    pub fn strict(strict: bool) -> Self {
        Self { strict }
    }
}

impl Default for ValidateProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Processor for ValidateProcessor {
    fn name(&self) -> &str {
        "ValidateProcessor"
    }

    async fn process(&self, mut exchange: Exchange) -> PdpResult<Exchange> {
        // Skip si le document n'est pas une facture (CDAR, e-reporting, etc.)
        if exchange.get_header("document.type").map(|s| s.as_str()) == Some("CDAR") {
            tracing::debug!(exchange_id = %exchange.id, "ValidateProcessor: skip (document CDAR)");
            return Ok(exchange);
        }

        let invoice = exchange.invoice.clone().ok_or_else(|| {
            PdpError::ValidationError(
                "Aucune facture parsée dans l'exchange. Le ParseProcessor doit être exécuté avant.".to_string(),
            )
        })?;

        exchange.set_status(FlowStatus::Validating);

        let validator = InvoiceValidator::new();
        let result = validator.validate(&invoice);

        // Stocker le résultat de validation dans les propriétés
        exchange.set_property("validation.valid", &result.is_valid.to_string());
        exchange.set_property("validation.errors", &result.errors.len().to_string());
        exchange.set_property("validation.warnings", &result.warnings.len().to_string());

        // Stocker le détail des erreurs en JSON
        if let Ok(errors_json) = serde_json::to_string(&result.errors) {
            exchange.set_property("validation.errors.detail", &errors_json);
        }
        if let Ok(warnings_json) = serde_json::to_string(&result.warnings) {
            exchange.set_property("validation.warnings.detail", &warnings_json);
        }

        if result.is_valid {
            tracing::info!(
                exchange_id = %exchange.id,
                invoice = %invoice.invoice_number,
                warnings = result.warnings.len(),
                "Facture valide"
            );
            exchange.set_status(FlowStatus::Validated);
        } else if self.strict {
            let error_msgs: Vec<String> = result.errors.iter()
                .map(|e| format!("[{}] {}", e.rule_id, e.message))
                .collect();
            let combined = error_msgs.join("; ");

            tracing::error!(
                exchange_id = %exchange.id,
                invoice = %invoice.invoice_number,
                errors = result.errors.len(),
                "Facture invalide (mode strict)"
            );

            return Err(PdpError::ValidationError(format!(
                "Facture {} invalide: {}",
                invoice.invoice_number, combined
            )));
        } else {
            tracing::warn!(
                exchange_id = %exchange.id,
                invoice = %invoice.invoice_number,
                errors = result.errors.len(),
                "Facture invalide (mode lenient, on continue)"
            );
            exchange.set_status(FlowStatus::Validated);
        }

        Ok(exchange)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pdp_core::exchange::Exchange;

    #[tokio::test]
    async fn test_parse_processor_ubl() {
        let xml = std::fs::read("../../tests/fixtures/ubl/facture_ubl_001.xml")
            .expect("Fixture UBL introuvable");

        let exchange = Exchange::new(xml).with_filename("facture_ubl_001.xml");
        let processor = ParseProcessor::new();
        let result = processor.process(exchange).await.expect("ParseProcessor failed");

        assert!(result.invoice.is_some());
        assert_eq!(result.status, FlowStatus::Parsed);
        assert_eq!(result.get_header("invoice.format").map(|s| s.as_str()), Some("UBL"));
        let invoice = result.invoice.unwrap();
        assert_eq!(invoice.invoice_number, "FA-2025-00142");
        assert_eq!(invoice.seller_name.as_deref(), Some("TechConseil SAS"));
    }

    #[tokio::test]
    async fn test_parse_processor_cii() {
        let xml = std::fs::read("../../tests/fixtures/cii/facture_cii_001.xml")
            .expect("Fixture CII introuvable");

        let exchange = Exchange::new(xml).with_filename("facture_cii_001.xml");
        let processor = ParseProcessor::new();
        let result = processor.process(exchange).await.expect("ParseProcessor failed");

        assert!(result.invoice.is_some());
        assert_eq!(result.get_header("invoice.format").map(|s| s.as_str()), Some("CII"));
        let invoice = result.invoice.unwrap();
        assert_eq!(invoice.invoice_number, "FA-2025-00256");
    }

    #[tokio::test]
    async fn test_parse_processor_invalid() {
        let exchange = Exchange::new(b"ceci n'est pas une facture".to_vec());
        let processor = ParseProcessor::new();
        let result = processor.process(exchange).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_processor_valid_ubl() {
        let xml = std::fs::read("../../tests/fixtures/ubl/facture_ubl_001.xml")
            .expect("Fixture UBL introuvable");

        let exchange = Exchange::new(xml).with_filename("facture_ubl_001.xml");

        // D'abord parser
        let parse = ParseProcessor::new();
        let exchange = parse.process(exchange).await.expect("Parse failed");

        // Puis valider
        let validate = ValidateProcessor::new();
        let result = validate.process(exchange).await.expect("Validate failed");

        assert_eq!(result.status, FlowStatus::Validated);
        assert_eq!(result.get_property("validation.valid").map(|s| s.as_str()), Some("true"));
    }

    #[tokio::test]
    async fn test_validate_processor_strict_rejects_invalid() {
        // Créer un exchange avec une facture incomplète
        let mut exchange = Exchange::new(b"dummy".to_vec());
        let mut invoice = pdp_core::model::InvoiceData::new(String::new(), pdp_core::model::InvoiceFormat::UBL);
        invoice.invoice_number = String::new(); // Invalide: pas de numéro
        exchange.invoice = Some(invoice);

        let validate = ValidateProcessor::new();
        let result = validate.process(exchange).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_processor_lenient_passes_invalid() {
        let mut exchange = Exchange::new(b"dummy".to_vec());
        let mut invoice = pdp_core::model::InvoiceData::new(String::new(), pdp_core::model::InvoiceFormat::UBL);
        invoice.invoice_number = String::new();
        exchange.invoice = Some(invoice);

        let validate = ValidateProcessor::lenient();
        let result = validate.process(exchange).await;
        assert!(result.is_ok()); // Lenient mode ne bloque pas
        let ex = result.unwrap();
        assert_eq!(ex.get_property("validation.valid").map(|s| s.as_str()), Some("false"));
    }

    #[tokio::test]
    async fn test_validate_processor_no_invoice_errors() {
        let exchange = Exchange::new(b"dummy".to_vec());
        let validate = ValidateProcessor::new();
        let result = validate.process(exchange).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_full_pipeline_parse_then_validate_cii() {
        let xml = std::fs::read("../../tests/fixtures/cii/facture_cii_001.xml")
            .expect("Fixture CII introuvable");

        let exchange = Exchange::new(xml).with_filename("facture_cii_001.xml");

        let parse = ParseProcessor::new();
        let exchange = parse.process(exchange).await.expect("Parse failed");

        let validate = ValidateProcessor::new();
        let exchange = validate.process(exchange).await.expect("Validate failed");

        assert_eq!(exchange.status, FlowStatus::Validated);
        assert_eq!(exchange.get_property("validation.valid").map(|s| s.as_str()), Some("true"));
        assert_eq!(exchange.get_header("invoice.number").map(|s| s.as_str()), Some("FA-2025-00256"));
    }

    // ===== Pipeline complet Factur-X =====

    #[tokio::test]
    async fn test_parse_processor_facturx() {
        let pdf = std::fs::read("../../tests/fixtures/facturx/facture_facturx_001.pdf")
            .expect("Fixture Factur-X introuvable");

        let exchange = Exchange::new(pdf).with_filename("facture_facturx_001.pdf");
        let processor = ParseProcessor::new();
        let result = processor.process(exchange).await.expect("ParseProcessor Factur-X failed");

        assert!(result.invoice.is_some());
        assert_eq!(result.get_header("invoice.format").map(|s| s.as_str()), Some("Factur-X"));
        let invoice = result.invoice.unwrap();
        assert_eq!(invoice.invoice_number, "FA-2025-00256");
        assert!(invoice.raw_pdf.is_some());
    }

    #[tokio::test]
    async fn test_full_pipeline_facturx() {
        let pdf = std::fs::read("../../tests/fixtures/facturx/facture_facturx_001.pdf")
            .expect("Fixture Factur-X introuvable");

        let exchange = Exchange::new(pdf).with_filename("facture.pdf");

        let parse = ParseProcessor::new();
        let exchange = parse.process(exchange).await.expect("Parse failed");

        let validate = ValidateProcessor::new();
        let exchange = validate.process(exchange).await.expect("Validate failed");

        assert_eq!(exchange.status, FlowStatus::Validated);
        assert_eq!(exchange.get_property("validation.valid").map(|s| s.as_str()), Some("true"));
    }

    // ===== Pipeline complet sur fixtures métier =====

    #[tokio::test]
    async fn test_pipeline_soustraitance_cii() {
        let data = std::fs::read("../../tests/fixtures/cii/facture_cii_soustraitance_a4.xml").unwrap();
        let exchange = Exchange::new(data);
        let exchange = ParseProcessor::new().process(exchange).await.unwrap();
        let exchange = ValidateProcessor::new().process(exchange).await.unwrap();
        assert_eq!(exchange.status, FlowStatus::Validated);
        assert_eq!(exchange.get_header("invoice.number").map(|s| s.as_str()), Some("ST-2025-00042"));
    }

    #[tokio::test]
    async fn test_pipeline_marketplace_cii() {
        let data = std::fs::read("../../tests/fixtures/cii/facture_cii_marketplace_a8.xml").unwrap();
        let exchange = Exchange::new(data);
        let exchange = ParseProcessor::new().process(exchange).await.unwrap();
        let exchange = ValidateProcessor::new().process(exchange).await.unwrap();
        assert_eq!(exchange.status, FlowStatus::Validated);
        assert_eq!(exchange.get_header("invoice.number").map(|s| s.as_str()), Some("MKP-2025-00789"));
    }

    #[tokio::test]
    async fn test_pipeline_acompte_cii() {
        let data = std::fs::read("../../tests/fixtures/cii/facture_cii_acompte.xml").unwrap();
        let exchange = Exchange::new(data);
        let exchange = ParseProcessor::new().process(exchange).await.unwrap();
        let exchange = ValidateProcessor::new().process(exchange).await.unwrap();
        assert_eq!(exchange.status, FlowStatus::Validated);
    }

    #[tokio::test]
    async fn test_pipeline_multitva_cii() {
        let data = std::fs::read("../../tests/fixtures/cii/facture_cii_remises_multitva.xml").unwrap();
        let exchange = Exchange::new(data);
        let exchange = ParseProcessor::new().process(exchange).await.unwrap();
        let exchange = ValidateProcessor::new().process(exchange).await.unwrap();
        assert_eq!(exchange.status, FlowStatus::Validated);
    }

    #[tokio::test]
    async fn test_pipeline_autofacture_cii() {
        let data = std::fs::read("../../tests/fixtures/cii/autofacture_cii_389.xml").unwrap();
        let exchange = Exchange::new(data);
        let exchange = ParseProcessor::new().process(exchange).await.unwrap();
        let exchange = ValidateProcessor::new().process(exchange).await.unwrap();
        assert_eq!(exchange.status, FlowStatus::Validated);
    }

    #[tokio::test]
    async fn test_pipeline_avoir_ubl() {
        let data = std::fs::read("../../tests/fixtures/ubl/facture_ubl_002_avoir.xml").unwrap();
        let exchange = Exchange::new(data);
        let exchange = ParseProcessor::new().process(exchange).await.unwrap();
        let exchange = ValidateProcessor::new().process(exchange).await.unwrap();
        assert_eq!(exchange.status, FlowStatus::Validated);
    }

    #[tokio::test]
    async fn test_pipeline_rectificative_ubl() {
        let data = std::fs::read("../../tests/fixtures/ubl/facture_rectificative_ubl_384.xml").unwrap();
        let exchange = Exchange::new(data);
        let exchange = ParseProcessor::new().process(exchange).await.unwrap();
        let exchange = ValidateProcessor::new().process(exchange).await.unwrap();
        assert_eq!(exchange.status, FlowStatus::Validated);
    }

    // ===== Pipeline erreurs =====

    #[tokio::test]
    async fn test_pipeline_strict_rejects_invalid_cii_fixture() {
        let data = std::fs::read("../../tests/fixtures/errors/facture_invalide_001.xml").unwrap();
        let exchange = Exchange::new(data);
        let exchange = ParseProcessor::new().process(exchange).await.unwrap();
        let result = ValidateProcessor::new().process(exchange).await;
        assert!(result.is_err(), "Le mode strict doit rejeter une facture invalide");
    }

    #[tokio::test]
    async fn test_pipeline_lenient_passes_invalid_cii_fixture() {
        let data = std::fs::read("../../tests/fixtures/errors/facture_invalide_001.xml").unwrap();
        let exchange = Exchange::new(data);
        let exchange = ParseProcessor::new().process(exchange).await.unwrap();
        let exchange = ValidateProcessor::lenient().process(exchange).await.unwrap();
        assert_eq!(exchange.get_property("validation.valid").map(|s| s.as_str()), Some("false"));
        assert_eq!(exchange.status, FlowStatus::Validated);
    }

    #[tokio::test]
    async fn test_pipeline_strict_rejects_invalid_ubl_fixture() {
        let data = std::fs::read("../../tests/fixtures/errors/facture_invalide_ubl_001.xml").unwrap();
        let exchange = Exchange::new(data);
        let exchange = ParseProcessor::new().process(exchange).await.unwrap();
        let result = ValidateProcessor::new().process(exchange).await;
        assert!(result.is_err(), "Le mode strict doit rejeter une facture UBL invalide");
    }

    // ===== Pipeline délégation et multi-vendeurs =====

    #[tokio::test]
    async fn test_pipeline_delegation_s8_cii() {
        let data = std::fs::read("../../tests/fixtures/cii/facture_cii_delegation_s8.xml").unwrap();
        let exchange = Exchange::new(data);
        let exchange = ParseProcessor::new().process(exchange).await.unwrap();
        let exchange = ValidateProcessor::new().process(exchange).await.unwrap();
        assert_eq!(exchange.status, FlowStatus::Validated);
        let inv = exchange.invoice.as_ref().unwrap();
        assert_eq!(inv.invoicer_name.as_deref(), Some("Cabinet Comptable Gironde SARL"));
        assert_eq!(inv.addressed_to_name.as_deref(), Some("Groupe Hôtelier Atlantique SA"));
        assert_eq!(inv.buyer_agent_name.as_deref(), Some("Service Achats Groupe Atlantique"));
    }

    #[tokio::test]
    async fn test_pipeline_delegation_s8_ubl() {
        let data = std::fs::read("../../tests/fixtures/ubl/facture_ubl_delegation_s8.xml").unwrap();
        let exchange = Exchange::new(data);
        let exchange = ParseProcessor::new().process(exchange).await.unwrap();
        let exchange = ValidateProcessor::new().process(exchange).await.unwrap();
        assert_eq!(exchange.status, FlowStatus::Validated);
        let inv = exchange.invoice.as_ref().unwrap();
        assert_eq!(inv.invoicer_name.as_deref(), Some("Cabinet Comptable Gironde SARL"));
    }

    #[tokio::test]
    async fn test_pipeline_multivendeurs_b8_cii() {
        let data = std::fs::read("../../tests/fixtures/cii/facture_cii_multivendeurs_b8.xml").unwrap();
        let exchange = Exchange::new(data);
        let exchange = ParseProcessor::new().process(exchange).await.unwrap();
        let exchange = ValidateProcessor::new().process(exchange).await.unwrap();
        assert_eq!(exchange.status, FlowStatus::Validated);
        let inv = exchange.invoice.as_ref().unwrap();
        assert_eq!(inv.invoicer_name.as_deref(), Some("ArtisanConnect SAS"));
    }

    #[tokio::test]
    async fn test_pipeline_multivendeurs_b8_ubl() {
        let data = std::fs::read("../../tests/fixtures/ubl/facture_ubl_multivendeurs_b8.xml").unwrap();
        let exchange = Exchange::new(data);
        let exchange = ParseProcessor::new().process(exchange).await.unwrap();
        let exchange = ValidateProcessor::new().process(exchange).await.unwrap();
        assert_eq!(exchange.status, FlowStatus::Validated);
        let inv = exchange.invoice.as_ref().unwrap();
        assert_eq!(inv.invoicer_name.as_deref(), Some("ArtisanConnect SAS"));
    }
}
