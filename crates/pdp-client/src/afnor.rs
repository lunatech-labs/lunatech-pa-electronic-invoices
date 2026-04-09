use reqwest::multipart;
use tracing;

use crate::auth::PisteAuth;
use crate::error::{ClientError, ClientResult};
use crate::model::*;
use crate::ppf::sha256_hex;

/// Configuration du client AFNOR Flow Service (PDP<>PDP)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AfnorFlowConfig {
    /// URL de base du Flow Service de la PDP distante
    /// Ex: "https://api.directory.pdp-partenaire.fr/flow-service"
    pub base_url: String,
}

/// Client HTTP pour l'API AFNOR XP Z12-013 Flow Service v1.2.0
///
/// Implémente l'ensemble des endpoints définis dans l'Annexe A de la norme :
/// - Gestion des flux (POST /v1/flows, POST /v1/flows/search, GET /v1/flows/{flowId})
/// - Gestion des webhooks (CRUD complet sur /v1/webhooks)
/// - Health check (GET /v1/healthcheck)
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

    /// Applique les headers optionnels AFNOR (Request-Id, Organization-Id, Accept-Language)
    fn apply_headers(
        &self,
        mut builder: reqwest::RequestBuilder,
        headers: Option<&AfnorRequestHeaders>,
    ) -> reqwest::RequestBuilder {
        if let Some(h) = headers {
            if let Some(ref id) = h.request_id {
                builder = builder.header("Request-Id", id.as_str());
            }
            if let Some(ref org) = h.organization_id {
                builder = builder.header("Organization-Id", org.as_str());
            }
            if let Some(ref lang) = h.accept_language {
                builder = builder.header("Accept-Language", lang.as_str());
            }
        }
        builder
    }

    /// Gère la réponse HTTP avec gestion fine des codes d'erreur (XP Z12-013 §5.5)
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
        }

        let retry_after = response
            .headers()
            .get("Retry-After")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u64>().ok());

        let body = response.text().await.unwrap_or_default();

        tracing::error!(
            operation = %operation,
            status = status.as_u16(),
            body = %body,
            "Erreur AFNOR Flow Service"
        );

        Err(ClientError::from_http_response(
            status.as_u16(),
            &body,
            operation,
            retry_after,
        ))
    }

    /// Gère une réponse sans corps (204 No Content)
    async fn handle_empty_response(
        &self,
        response: reqwest::Response,
        operation: &str,
    ) -> ClientResult<()> {
        let status = response.status();

        if status.is_success() {
            return Ok(());
        }

        if status.as_u16() == 401 {
            self.auth.invalidate().await;
        }

        let retry_after = response
            .headers()
            .get("Retry-After")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u64>().ok());

        let body = response.text().await.unwrap_or_default();

        tracing::error!(
            operation = %operation,
            status = status.as_u16(),
            body = %body,
            "Erreur AFNOR Flow Service"
        );

        Err(ClientError::from_http_response(
            status.as_u16(),
            &body,
            operation,
            retry_after,
        ))
    }

    // ============================================================
    // Flux — POST /v1/flows, POST /v1/flows/search, GET /v1/flows/{flowId}
    // ============================================================

    /// Envoie un flux (facture, CDV, e-reporting) à une autre PDP via le Flow Service AFNOR.
    ///
    /// Correspond à `POST /v1/flows` avec multipart/form-data (retour 202 Accepted) :
    /// - `flowInfo` : JSON avec trackingId, name, processingRule, flowSyntax, flowProfile, sha256
    /// - `file` : contenu binaire du fichier (XML, PDF)
    pub async fn envoyer_flux(
        &self,
        flow_info: &AfnorFlowInfo,
        filename: &str,
        file_content: &[u8],
        headers: Option<&AfnorRequestHeaders>,
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

        let builder = self
            .http
            .post(&url)
            .header("Authorization", auth)
            .multipart(form);

        let builder = self.apply_headers(builder, headers);

        let response = builder.send().await?;

        self.handle_response(response, "envoyer_flux").await
    }

    /// Recherche de flux dans le Flow Service AFNOR avec filtres typés.
    /// Correspond à `POST /v1/flows/search`
    pub async fn rechercher_flux(
        &self,
        params: &SearchFlowParams,
        headers: Option<&AfnorRequestHeaders>,
    ) -> ClientResult<AfnorFlowSearchResponse> {
        let url = format!("{}/v1/flows/search", self.config.base_url);
        let auth = self.auth_header().await?;

        let builder = self
            .http
            .post(&url)
            .header("Authorization", auth)
            .json(params);

        let builder = self.apply_headers(builder, headers);

        let response = builder.send().await?;

        self.handle_response(response, "rechercher_flux").await
    }

    /// Recherche de flux avec critères JSON bruts (rétrocompatibilité).
    pub async fn rechercher_flux_raw(
        &self,
        criteria: &serde_json::Value,
        headers: Option<&AfnorRequestHeaders>,
    ) -> ClientResult<AfnorFlowSearchResponse> {
        let url = format!("{}/v1/flows/search", self.config.base_url);
        let auth = self.auth_header().await?;

        let builder = self
            .http
            .post(&url)
            .header("Authorization", auth)
            .json(criteria);

        let builder = self.apply_headers(builder, headers);

        let response = builder.send().await?;

        self.handle_response(response, "rechercher_flux").await
    }

    /// Récupère les métadonnées ou le contenu d'un flux par son ID.
    ///
    /// Correspond à `GET /v1/flows/{flowId}?docType={docType}`
    ///
    /// Le paramètre `doc_type` contrôle ce qui est retourné :
    /// - `Metadata` : métadonnées JSON du flux
    /// - `Original` : document original tel que soumis
    /// - `Converted` : document converti par la PDP réceptrice
    /// - `ReadableView` : vue lisible (PDF)
    pub async fn consulter_flux(
        &self,
        flow_id: &str,
        doc_type: Option<DocType>,
        headers: Option<&AfnorRequestHeaders>,
    ) -> ClientResult<AfnorFlowItem> {
        let mut url = format!("{}/v1/flows/{}", self.config.base_url, flow_id);
        if let Some(dt) = doc_type {
            url = format!("{}?docType={}", url, dt);
        }
        let auth = self.auth_header().await?;

        let builder = self
            .http
            .get(&url)
            .header("Authorization", auth);

        let builder = self.apply_headers(builder, headers);

        let response = builder.send().await?;

        self.handle_response(response, "consulter_flux").await
    }

    /// Télécharge le contenu binaire d'un flux par son ID.
    ///
    /// Correspond à `GET /v1/flows/{flowId}?docType={docType}` avec Accept: application/octet-stream
    pub async fn telecharger_flux(
        &self,
        flow_id: &str,
        doc_type: Option<DocType>,
        headers: Option<&AfnorRequestHeaders>,
    ) -> ClientResult<Vec<u8>> {
        let mut url = format!("{}/v1/flows/{}", self.config.base_url, flow_id);
        if let Some(dt) = doc_type {
            url = format!("{}?docType={}", url, dt);
        }
        let auth = self.auth_header().await?;

        let builder = self
            .http
            .get(&url)
            .header("Authorization", auth)
            .header("Accept", "application/octet-stream");

        let builder = self.apply_headers(builder, headers);

        let response = builder.send().await?;

        let status = response.status();
        if !status.is_success() {
            if status.as_u16() == 401 {
                self.auth.invalidate().await;
            }

            let retry_after = response
                .headers()
                .get("Retry-After")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<u64>().ok());

            let body = response.text().await.unwrap_or_default();
            return Err(ClientError::from_http_response(
                status.as_u16(),
                &body,
                "telecharger_flux",
                retry_after,
            ));
        }

        Ok(response.bytes().await?.to_vec())
    }

    // ============================================================
    // Webhooks — CRUD /v1/webhooks (XP Z12-013 §5.4)
    // ============================================================

    /// Crée un abonnement webhook.
    /// Correspond à `POST /v1/webhooks` (retour 201 Created)
    pub async fn create_webhook(
        &self,
        request: &WebhookCreateRequest,
        headers: Option<&AfnorRequestHeaders>,
    ) -> ClientResult<AfnorWebhook> {
        let url = format!("{}/v1/webhooks", self.config.base_url);
        let auth = self.auth_header().await?;

        tracing::info!(
            callback_url = %request.callback_url,
            events = ?request.events,
            "Création webhook AFNOR"
        );

        let builder = self
            .http
            .post(&url)
            .header("Authorization", auth)
            .json(request);

        let builder = self.apply_headers(builder, headers);

        let response = builder.send().await?;

        self.handle_response(response, "create_webhook").await
    }

    /// Liste tous les webhooks enregistrés.
    /// Correspond à `GET /v1/webhooks`
    pub async fn list_webhooks(
        &self,
        headers: Option<&AfnorRequestHeaders>,
    ) -> ClientResult<AfnorWebhookListResponse> {
        let url = format!("{}/v1/webhooks", self.config.base_url);
        let auth = self.auth_header().await?;

        let builder = self
            .http
            .get(&url)
            .header("Authorization", auth);

        let builder = self.apply_headers(builder, headers);

        let response = builder.send().await?;

        self.handle_response(response, "list_webhooks").await
    }

    /// Récupère un webhook par son UID.
    /// Correspond à `GET /v1/webhooks/{webhookUid}`
    pub async fn get_webhook(
        &self,
        webhook_uid: &str,
        headers: Option<&AfnorRequestHeaders>,
    ) -> ClientResult<AfnorWebhook> {
        let url = format!("{}/v1/webhooks/{}", self.config.base_url, webhook_uid);
        let auth = self.auth_header().await?;

        let builder = self
            .http
            .get(&url)
            .header("Authorization", auth);

        let builder = self.apply_headers(builder, headers);

        let response = builder.send().await?;

        self.handle_response(response, "get_webhook").await
    }

    /// Met à jour un webhook existant.
    /// Correspond à `PATCH /v1/webhooks/{webhookUid}` (retour 204 No Content)
    pub async fn update_webhook(
        &self,
        webhook_uid: &str,
        request: &WebhookUpdateRequest,
        headers: Option<&AfnorRequestHeaders>,
    ) -> ClientResult<()> {
        let url = format!("{}/v1/webhooks/{}", self.config.base_url, webhook_uid);
        let auth = self.auth_header().await?;

        tracing::info!(
            webhook_uid = %webhook_uid,
            "Mise à jour webhook AFNOR"
        );

        let builder = self
            .http
            .patch(&url)
            .header("Authorization", auth)
            .json(request);

        let builder = self.apply_headers(builder, headers);

        let response = builder.send().await?;

        self.handle_empty_response(response, "update_webhook").await
    }

    /// Supprime un webhook.
    /// Correspond à `DELETE /v1/webhooks/{webhookUid}` (retour 204 No Content)
    pub async fn delete_webhook(
        &self,
        webhook_uid: &str,
        headers: Option<&AfnorRequestHeaders>,
    ) -> ClientResult<()> {
        let url = format!("{}/v1/webhooks/{}", self.config.base_url, webhook_uid);
        let auth = self.auth_header().await?;

        tracing::info!(
            webhook_uid = %webhook_uid,
            "Suppression webhook AFNOR"
        );

        let builder = self
            .http
            .delete(&url)
            .header("Authorization", auth);

        let builder = self.apply_headers(builder, headers);

        let response = builder.send().await?;

        self.handle_empty_response(response, "delete_webhook").await
    }

    // ============================================================
    // Health check — GET /v1/healthcheck
    // ============================================================

    /// Vérifie l'état du Flow Service.
    /// Correspond à `GET /v1/healthcheck`
    pub async fn healthcheck(&self) -> ClientResult<HealthCheckResponse> {
        let url = format!("{}/v1/healthcheck", self.config.base_url);

        let response = self.http.get(&url).send().await?;

        self.handle_response(response, "healthcheck").await
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
        flow_type: Some(FlowType::CustomerInvoice),
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
        flow_type: Some(flow_type),
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
        (false, false) => FlowType::UnitaryCustomerTransactionReport,
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
        flow_type: Some(flow_type),
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
        assert_eq!(info.flow_type, Some(FlowType::CustomerInvoice));
        assert_eq!(info.flow_syntax, FlowSyntax::UBL);
        assert!(!info.sha256.is_empty());
    }

    #[test]
    fn test_build_cdv_flow_info() {
        let content = b"<CDV>test</CDV>";
        let info = build_cdv_flow_info("track-002", "cdv.xml", content, false);
        assert_eq!(info.flow_type, Some(FlowType::CustomerInvoiceLC));
        assert_eq!(info.flow_syntax, FlowSyntax::CDAR);
    }

    #[test]
    fn test_build_ereporting_flow_info() {
        let content = b"<Report>test</Report>";
        let info = build_ereporting_flow_info("track-003", "report.xml", content, false, false);
        assert_eq!(info.flow_type, Some(FlowType::UnitaryCustomerTransactionReport));
        assert_eq!(info.flow_syntax, FlowSyntax::FRR);

        let info2 = build_ereporting_flow_info("track-004", "report.xml", content, true, true);
        assert_eq!(info2.flow_type, Some(FlowType::AggregatedCustomerPaymentReport));
    }

    #[test]
    fn test_doc_type_display() {
        assert_eq!(DocType::Metadata.to_string(), "Metadata");
        assert_eq!(DocType::Original.to_string(), "Original");
        assert_eq!(DocType::Converted.to_string(), "Converted");
        assert_eq!(DocType::ReadableView.to_string(), "ReadableView");
    }

    #[test]
    fn test_search_flow_params_serialization() {
        let params = SearchFlowParams {
            limit: Some(50),
            filters: SearchFlowFilters {
                updated_after: Some("2026-01-01T00:00:00Z".to_string()),
                flow_type: Some(vec![FlowType::CustomerInvoice]),
                ..Default::default()
            },
        };
        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("\"limit\":50"));
        assert!(json.contains("\"where\""));
        assert!(json.contains("updatedAfter"));
    }

    #[test]
    fn test_webhook_create_request_serialization() {
        let request = WebhookCreateRequest {
            callback_url: "https://my-pdp.example.com/hooks/afnor".to_string(),
            events: vec![WebhookEvent::FlowReceived, WebhookEvent::FlowAckUpdated],
            secret: Some("my-hmac-secret".to_string()),
        };
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("flow.received"));
        assert!(json.contains("callbackUrl"));
    }
}
