use async_trait::async_trait;
use pdp_core::error::{PdpError, PdpResult};
use pdp_core::exchange::Exchange;
use pdp_core::model::{FlowStatus, InvoiceFormat};
use pdp_core::processor::Processor;

/// Processor de transformation de format de facture.
/// Supporte les 6 chemins de conversion :
/// UBL↔CII, Factur-X↔CII, Factur-X↔UBL, UBL→Factur-X, CII→Factur-X
pub struct TransformProcessor {
    target_format: InvoiceFormat,
}

impl TransformProcessor {
    pub fn new(target_format: InvoiceFormat) -> Self {
        Self { target_format }
    }

    pub fn to_cii() -> Self {
        Self::new(InvoiceFormat::CII)
    }

    pub fn to_ubl() -> Self {
        Self::new(InvoiceFormat::UBL)
    }

    pub fn to_facturx() -> Self {
        Self::new(InvoiceFormat::FacturX)
    }
}

#[async_trait]
impl Processor for TransformProcessor {
    fn name(&self) -> &str {
        "TransformProcessor"
    }

    async fn process(&self, mut exchange: Exchange) -> PdpResult<Exchange> {
        // Skip si le document n'est pas une facture (CDAR, e-reporting, etc.)
        if exchange.get_header("document.type").map(|s| s.as_str()) == Some("CDAR") {
            tracing::debug!(exchange_id = %exchange.id, "TransformProcessor: skip (document CDAR)");
            return Ok(exchange);
        }

        let invoice = exchange.invoice.clone().ok_or_else(|| {
            PdpError::TransformError {
                source_format: "inconnu".to_string(),
                target_format: self.target_format.to_string(),
                message: "Aucune facture parsée dans l'exchange. Le ParseProcessor doit être exécuté avant.".to_string(),
            }
        })?;

        // Vérifier si une transformation est nécessaire
        if invoice.source_format == self.target_format {
            tracing::info!(
                invoice = %invoice.invoice_number,
                format = %self.target_format,
                "Pas de transformation nécessaire, format identique"
            );
            return Ok(exchange);
        }

        exchange.set_status(FlowStatus::Transforming);

        let result = crate::converter::convert(&invoice, self.target_format.clone())?;

        exchange.set_body(result.content);
        exchange.set_header("transform.source", &invoice.source_format.to_string());
        exchange.set_header("transform.target", &self.target_format.to_string());
        exchange.set_header("transform.filename", &result.suggested_filename);
        exchange.set_status(FlowStatus::Transformed);

        tracing::info!(
            invoice = %invoice.invoice_number,
            from = %invoice.source_format,
            to = %self.target_format,
            filename = %result.suggested_filename,
            "Transformation terminée"
        );

        Ok(exchange)
    }
}
