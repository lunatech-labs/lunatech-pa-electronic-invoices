use reqwest::multipart;
use tracing;

use crate::auth::PisteAuth;
use crate::error::{ClientError, ClientResult};
use crate::model::*;
use crate::ppf::sha256_hex;

/// Configuration du client AFNOR Flow Service (PDP↔PDP)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AfnorFlowConfig {
    /// URL de base du Flow Service de la PDP distante
    /// Ex: "https://api.directory.pdp-partenaire.fr/flow-service"
    pub base_url: String,
}

/// Client HTTP pour l'API AFNOR XP Z12-013 Flow Service
pub struct AfnorFlowClient {
    config: AfnorFlowConfig,
    auth: PisteAuth,
    http: reqwest::Client,
}

impl AfnorFlowClient {
    pub fn new(config: AfnorFlowConfig, auth: PisteAuth) -> Self {
        Self {
            config,
            auth,
            http: reqwest::Client::new(),
        }
    }

    async fn auth_header(&self) -> ClientResult<String> {
        let token = self.auth.get_token().await?;
        Ok(format!("Bearer {}", token))
    }

    async fn handle_response<T: serde::de::DeserializeOwned>(
        &self,
        response: reqwest::Response,
        operation: &str,
    ) -> ClientResult<T> {
        let status = response.status();

        if status.is_success() {
            return Ok(response.json::<T>().await?);
        }

        if status.as_u16() == 401 {
            self.auth.invalidate().await;
            return Err(ClientError::TokenExpired);
        }

        let body = response.text().await.unwrap_or_default();
        tracing::error!(
            operation = %operation,
            status = status.as_u16(),
            body = %body,
            "Erreur AFNOR Flow Service"
        );

        Err(ClientError::HttpError {
            status: status.as_u16(),
            message: format!("{}: {}", operation, body),
        })
    }

    /// Envoie un flux (facture, CDV, e-reporting) à une autre PDP via le Flow Service AFNOR.
    ///
    /// Correspond à `POST /v1/flows` avec multipart/form-data :
    /// - `flowInfo` : JSON avec trackingId, name, processingRule, flowSyntax, flowProfile, sha256
    /// - `file` : contenu binaire du fichier (XML, PDF)
    pub async fn envoyer_flux(
        &self,
        flow_info: &AfnorFlowInfo,
        filename: &str,
        file_content: &[u8],
    ) -> ClientResult<AfnorFlowCreateResponse> {
        let url = format!("{}/v1/flows", self.config.base_url);

        let flow_info_json = serde_json::to_string(flow_info)?;

        let content_type = match flow_info.flow_syntax {
            FlowSyntax::CII => "application/cii+xml",
            FlowSyntax::UBL => "application/ubl+xml",
            FlowSyntax::FacturX => "application/facturx+pdf",
            FlowSyntax::CDAR => "application/xml",
            FlowSyntax::FRR => "application/xml",
        };

        let form = multipart::Form::new()
            .text("flowInfo", flow_info_json)
            .part(
                "file",
                multipart::Part::bytes(file_content.to_vec())
                    .file_name(filename.to_string())
                    .mime_str(content_type)
                    .map_err(|e| ClientError::AfnorError(e.to_string()))?,
            );

        let auth = self.auth_header().await?;

        tracing::info!(
            url = %url,
            tracking_id = %flow_info.tracking_id,
            flow_type = ?flow_info.flow_type,
            flow_syntax = %flow_info.flow_syntax,
            filename = %filename,
            size = file_content.len(),
            "Envoi flux AFNOR Flow Service"
        );

        let response = self
            .http
            .post(&url)
            .header("Authorization", auth)
            .multipart(form)
            .send()
            .await?;

        self.handle_response(response, "envoyer_flux").await
    }

    /// Recherche de flux dans le Flow Service AFNOR.
    /// Correspond à `POST /v1/flows/search`
    pub async fn rechercher_flux(
        &self,
        criteria: &serde_json::Value,
    ) -> ClientResult<AfnorFlowSearchResponse> {
        let url = format!("{}/v1/flows/search", self.config.base_url);
        let auth = self.auth_header().await?;

        let response = self
            .http
            .post(&url)
            .header("Authorization", auth)
            .json(criteria)
            .send()
            .await?;

        self.handle_response(response, "rechercher_flux").await
    }

    /// Récupère un flux par son ID.
    /// Correspond à `GET /v1/flows/{flowId}`
    pub async fn consulter_flux(&self, flow_id: &str) -> ClientResult<AfnorFlowItem> {
        let url = format!("{}/v1/flows/{}", self.config.base_url, flow_id);
        let auth = self.auth_header().await?;

        let response = self
            .http
            .get(&url)
            .header("Authorization", auth)
            .send()
            .await?;

        self.handle_response(response, "consulter_flux").await
    }

    /// Télécharge le contenu d'un flux par son ID.
    /// Correspond à `GET /v1/flows/{flowId}` avec Accept: application/octet-stream
    pub async fn telecharger_flux(&self, flow_id: &str) -> ClientResult<Vec<u8>> {
        let url = format!("{}/v1/flows/{}", self.config.base_url, flow_id);
        let auth = self.auth_header().await?;

        let response = self
            .http
            .get(&url)
            .header("Authorization", auth)
            .header("Accept", "application/octet-stream")
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(ClientError::HttpError {
                status: status.as_u16(),
                message: format!("telecharger_flux: {}", body),
            });
        }

        Ok(response.bytes().await?.to_vec())
    }
}

/// Helper : construit un AfnorFlowInfo pour une facture client (CustomerInvoice)
pub fn build_invoice_flow_info(
    tracking_id: &str,
    filename: &str,
    syntax: FlowSyntax,
    profile: FlowProfile,
    processing_rule: ProcessingRule,
    file_content: &[u8],
) -> AfnorFlowInfo {
    AfnorFlowInfo {
        tracking_id: tracking_id.to_string(),
        name: filename.to_string(),
        processing_rule,
        flow_syntax: syntax,
        flow_profile: profile,
        flow_type: FlowType::CustomerInvoice,
        sha256: sha256_hex(file_content),
        callback_url: None,
    }
}

/// Helper : construit un AfnorFlowInfo pour un CDV (cycle de vie)
pub fn build_cdv_flow_info(
    tracking_id: &str,
    filename: &str,
    file_content: &[u8],
    is_supplier: bool,
) -> AfnorFlowInfo {
    let flow_type = if is_supplier {
        FlowType::SupplierInvoiceLC
    } else {
        FlowType::CustomerInvoiceLC
    };

    AfnorFlowInfo {
        tracking_id: tracking_id.to_string(),
        name: filename.to_string(),
        processing_rule: ProcessingRule::B2B,
        flow_syntax: FlowSyntax::CDAR,
        flow_profile: FlowProfile::Basic,
        flow_type,
        sha256: sha256_hex(file_content),
        callback_url: None,
    }
}

/// Helper : construit un AfnorFlowInfo pour un e-reporting (FRR)
pub fn build_ereporting_flow_info(
    tracking_id: &str,
    filename: &str,
    file_content: &[u8],
    is_aggregated: bool,
    is_payment: bool,
) -> AfnorFlowInfo {
    let flow_type = match (is_aggregated, is_payment) {
        (false, false) => FlowType::IndividualCustomerTransactionReport,
        (false, true) => FlowType::UnitaryCustomerPaymentReport,
        (true, false) => FlowType::AggregatedCustomerTransactionReport,
        (true, true) => FlowType::AggregatedCustomerPaymentReport,
    };

    AfnorFlowInfo {
        tracking_id: tracking_id.to_string(),
        name: filename.to_string(),
        processing_rule: ProcessingRule::NotApplicable,
        flow_syntax: FlowSyntax::FRR,
        flow_profile: FlowProfile::Basic,
        flow_type,
        sha256: sha256_hex(file_content),
        callback_url: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_invoice_flow_info() {
        let content = b"<Invoice>test</Invoice>";
        let info = build_invoice_flow_info(
            "track-001",
            "facture.xml",
            FlowSyntax::UBL,
            FlowProfile::CIUS,
            ProcessingRule::B2B,
            content,
        );
        assert_eq!(info.tracking_id, "track-001");
        assert_eq!(info.flow_type, FlowType::CustomerInvoice);
        assert_eq!(info.flow_syntax, FlowSyntax::UBL);
        assert!(!info.sha256.is_empty());
    }

    #[test]
    fn test_build_cdv_flow_info() {
        let content = b"<CDV>test</CDV>";
        let info = build_cdv_flow_info("track-002", "cdv.xml", content, false);
        assert_eq!(info.flow_type, FlowType::CustomerInvoiceLC);
        assert_eq!(info.flow_syntax, FlowSyntax::CDAR);
    }

    #[test]
    fn test_build_ereporting_flow_info() {
        let content = b"<Report>test</Report>";
        let info = build_ereporting_flow_info("track-003", "report.xml", content, false, false);
        assert_eq!(info.flow_type, FlowType::IndividualCustomerTransactionReport);
        assert_eq!(info.flow_syntax, FlowSyntax::FRR);

        let info2 = build_ereporting_flow_info("track-004", "report.xml", content, true, true);
        assert_eq!(info2.flow_type, FlowType::AggregatedCustomerPaymentReport);
    }
}
