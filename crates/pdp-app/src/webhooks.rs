//! Webhooks AFNOR Flow Service (XP Z12-013 §5.4).
//!
//! Implémente les 5 endpoints de gestion des abonnements webhooks :
//! - POST /v1/webhooks → créer un abonnement (201 Created)
//! - GET /v1/webhooks → lister les abonnements (200 OK)
//! - GET /v1/webhooks/{webhookId} → consulter un webhook (200 OK)
//! - PATCH /v1/webhooks/{webhookId} → mettre à jour (204 No Content)
//! - DELETE /v1/webhooks/{webhookId} → supprimer (204 No Content)
//!
//! Les webhooks sont déclenchés sur les événements :
//! - `flow.received` : un flux entrant a été reçu
//! - `flow.ack.updated` : le statut d'acquittement d'un flux a changé

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================================
// Modèles conformes XP Z12-013 V1.2.0
// ============================================================

/// En-tête HTTP personnalisé à injecter lors du callback
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CallbackHeader {
    pub header_name: String,
    pub header_value: String,
}

/// Authentification du callback (BASIC ou OAUTH2)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CallbackAuthentication {
    /// "BASIC" ou "OAUTH2"
    pub auth_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_secret: Option<String>,
}

/// Signature des payloads webhook (HMAC, RSA, ECDSA, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallbackSignature {
    /// "HS256", "RS256", "ECDSA", "EDDSA_25519", "RSA_PSS", "EDDSA_448"
    pub algo: String,
    /// Clé encodée en base64
    pub key: String,
}

/// Configuration du callback (URL + auth + signature)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CallbackParameters {
    /// URL HTTPS du callback
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<Vec<CallbackHeader>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authentication: Option<CallbackAuthentication>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<CallbackSignature>,
}

/// Métadonnées de filtrage du webhook
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebhookMetadata {
    /// Type de flux (ex: "CustomerInvoice", "SupplierInvoice", "Cdar", "EReporting")
    pub flow_type: String,
    /// Direction du flux : "In" ou "Out"
    pub flow_direction: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub processing_rule: Option<String>,
    /// "Pending", "Ok", ou "Error"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ack_status: Option<String>,
}

/// Requête de création de webhook (POST /v1/webhooks)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookCreateRequest {
    pub callback: CallbackParameters,
    pub metadata: WebhookMetadata,
}

/// Requête de mise à jour (PATCH /v1/webhooks/{uid}) — partielle
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebhookUpdateRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<Vec<CallbackHeader>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authentication: Option<CallbackAuthentication>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<CallbackSignature>,
}

/// Webhook stocké
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Webhook {
    pub webhook_id: Uuid,
    pub callback: CallbackParameters,
    pub metadata: WebhookMetadata,
    /// Token propriétaire (multi-tenant)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
}

/// Réponse à POST /v1/webhooks
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WebhookCreateResponse {
    pub webhook_id: Uuid,
}

/// Réponse à GET /v1/webhooks (liste des UIDs)
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WebhookListResponse {
    pub webhook_ids: Vec<Uuid>,
}

// ============================================================
// Store en mémoire
// ============================================================

/// Store thread-safe pour les webhooks (in-memory).
///
/// À terme, à remplacer par PostgreSQL pour persistence.
#[derive(Default)]
pub struct WebhookStore {
    inner: RwLock<HashMap<Uuid, Webhook>>,
}

impl WebhookStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Crée un nouveau webhook et retourne son ID
    pub fn create(&self, req: WebhookCreateRequest, owner: Option<String>) -> Webhook {
        let webhook_id = Uuid::new_v4();
        let webhook = Webhook {
            webhook_id,
            callback: req.callback,
            metadata: req.metadata,
            owner,
        };
        self.inner
            .write()
            .unwrap()
            .insert(webhook_id, webhook.clone());
        webhook
    }

    /// Récupère un webhook par son ID
    pub fn get(&self, webhook_id: &Uuid) -> Option<Webhook> {
        self.inner.read().unwrap().get(webhook_id).cloned()
    }

    /// Liste tous les webhooks (filtrés par owner si fourni)
    pub fn list(&self, owner: Option<&str>) -> Vec<Webhook> {
        self.inner
            .read()
            .unwrap()
            .values()
            .filter(|w| match (owner, &w.owner) {
                (None, _) => true,
                (Some(o), Some(wo)) => o == wo,
                (Some(_), None) => true, // webhooks sans owner sont visibles par tous
            })
            .cloned()
            .collect()
    }

    /// Met à jour un webhook (PATCH partiel)
    pub fn update(&self, webhook_id: &Uuid, patch: WebhookUpdateRequest) -> Option<Webhook> {
        let mut store = self.inner.write().unwrap();
        let webhook = store.get_mut(webhook_id)?;
        if let Some(headers) = patch.headers {
            webhook.callback.headers = Some(headers);
        }
        if let Some(auth) = patch.authentication {
            webhook.callback.authentication = Some(auth);
        }
        if let Some(sig) = patch.signature {
            webhook.callback.signature = Some(sig);
        }
        Some(webhook.clone())
    }

    /// Supprime un webhook. Retourne `true` si le webhook existait.
    pub fn delete(&self, webhook_id: &Uuid) -> bool {
        self.inner.write().unwrap().remove(webhook_id).is_some()
    }

    /// Filtre les webhooks correspondant à un événement.
    /// Utilisé par le `WebhookDispatcher` pour trouver les abonnements à notifier.
    pub fn matching(
        &self,
        flow_type: &str,
        flow_direction: &str,
        ack_status: Option<&str>,
    ) -> Vec<Webhook> {
        self.inner
            .read()
            .unwrap()
            .values()
            .filter(|w| {
                w.metadata.flow_type == flow_type
                    && w.metadata.flow_direction == flow_direction
                    && match (&w.metadata.ack_status, ack_status) {
                        (None, _) => true, // pas de filtre sur ackStatus → matche tout
                        (Some(_), None) => false,
                        (Some(a), Some(b)) => a == b,
                    }
            })
            .cloned()
            .collect()
    }

    /// Nombre de webhooks enregistrés
    pub fn len(&self) -> usize {
        self.inner.read().unwrap().len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.read().unwrap().is_empty()
    }
}

// ============================================================
// Handlers HTTP
// ============================================================

use crate::server::AppState;

/// POST /v1/webhooks — créer un nouvel abonnement
pub async fn handle_create_webhook(
    State(state): State<Arc<AppState>>,
    Json(req): Json<WebhookCreateRequest>,
) -> impl IntoResponse {
    // Validation de base
    if !req.callback.url.starts_with("https://") && !req.callback.url.starts_with("http://") {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "callback.url must be a valid HTTP(S) URL",
            })),
        )
            .into_response();
    }
    if req.metadata.flow_type.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "metadata.flowType is required",
            })),
        )
            .into_response();
    }
    if req.metadata.flow_direction != "In" && req.metadata.flow_direction != "Out" {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "metadata.flowDirection must be 'In' or 'Out'",
            })),
        )
            .into_response();
    }

    let webhook = state.webhook_store.create(req, None);

    tracing::info!(
        webhook_id = %webhook.webhook_id,
        flow_type = %webhook.metadata.flow_type,
        flow_direction = %webhook.metadata.flow_direction,
        callback_url = %webhook.callback.url,
        "Webhook créé"
    );

    (
        StatusCode::CREATED,
        Json(WebhookCreateResponse {
            webhook_id: webhook.webhook_id,
        }),
    )
        .into_response()
}

/// GET /v1/webhooks — lister les abonnements
pub async fn handle_list_webhooks(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let webhooks = state.webhook_store.list(None);
    let response = WebhookListResponse {
        webhook_ids: webhooks.iter().map(|w| w.webhook_id).collect(),
    };
    (StatusCode::OK, Json(response)).into_response()
}

/// GET /v1/webhooks/{webhookId} — détails d'un webhook
pub async fn handle_get_webhook(
    State(state): State<Arc<AppState>>,
    Path(webhook_id): Path<Uuid>,
) -> impl IntoResponse {
    match state.webhook_store.get(&webhook_id) {
        Some(webhook) => (StatusCode::OK, Json(webhook)).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": format!("Webhook {} not found", webhook_id),
            })),
        )
            .into_response(),
    }
}

/// PATCH /v1/webhooks/{webhookId} — mise à jour partielle
pub async fn handle_update_webhook(
    State(state): State<Arc<AppState>>,
    Path(webhook_id): Path<Uuid>,
    Json(patch): Json<WebhookUpdateRequest>,
) -> impl IntoResponse {
    match state.webhook_store.update(&webhook_id, patch) {
        Some(_) => StatusCode::NO_CONTENT.into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": format!("Webhook {} not found", webhook_id),
            })),
        )
            .into_response(),
    }
}

/// DELETE /v1/webhooks/{webhookId} — désabonnement
pub async fn handle_delete_webhook(
    State(state): State<Arc<AppState>>,
    Path(webhook_id): Path<Uuid>,
) -> impl IntoResponse {
    if state.webhook_store.delete(&webhook_id) {
        tracing::info!(webhook_id = %webhook_id, "Webhook supprimé");
        StatusCode::NO_CONTENT.into_response()
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": format!("Webhook {} not found", webhook_id),
            })),
        )
            .into_response()
    }
}

// ============================================================
// Dispatcher
// ============================================================

/// Type d'événement webhook
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebhookEventType {
    /// Un nouveau flux entrant a été reçu
    FlowReceived,
    /// Le statut d'acquittement d'un flux a changé
    FlowAckUpdated,
}

impl WebhookEventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::FlowReceived => "flow.received",
            Self::FlowAckUpdated => "flow.ack.updated",
        }
    }
}

/// Payload envoyé au callback du webhook
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WebhookPayload {
    pub event: String,
    pub webhook_id: Uuid,
    pub flow_id: String,
    pub flow_type: String,
    pub flow_direction: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ack_status: Option<String>,
    pub timestamp: String,
}

/// Dispatcher de webhooks : envoie les événements aux abonnés.
pub struct WebhookDispatcher {
    store: Arc<WebhookStore>,
    client: reqwest::Client,
}

impl WebhookDispatcher {
    pub fn new(store: Arc<WebhookStore>) -> Self {
        Self {
            store,
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .unwrap_or_default(),
        }
    }

    /// Envoie un événement à tous les webhooks correspondants (non-bloquant).
    pub async fn dispatch(
        &self,
        event: WebhookEventType,
        flow_id: &str,
        flow_type: &str,
        flow_direction: &str,
        ack_status: Option<&str>,
    ) {
        let webhooks = self
            .store
            .matching(flow_type, flow_direction, ack_status);

        if webhooks.is_empty() {
            tracing::debug!(
                event = event.as_str(),
                flow_id = flow_id,
                "Aucun webhook abonné à cet événement"
            );
            return;
        }

        for webhook in webhooks {
            let payload = WebhookPayload {
                event: event.as_str().to_string(),
                webhook_id: webhook.webhook_id,
                flow_id: flow_id.to_string(),
                flow_type: flow_type.to_string(),
                flow_direction: flow_direction.to_string(),
                ack_status: ack_status.map(String::from),
                timestamp: chrono::Utc::now().to_rfc3339(),
            };

            if let Err(e) = self.send_one(&webhook, &payload).await {
                tracing::warn!(
                    webhook_id = %webhook.webhook_id,
                    callback_url = %webhook.callback.url,
                    error = %e,
                    "Échec d'envoi de webhook (non bloquant)"
                );
            }
        }
    }

    async fn send_one(
        &self,
        webhook: &Webhook,
        payload: &WebhookPayload,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let body = serde_json::to_vec(payload)?;
        let mut req = self.client.post(&webhook.callback.url).body(body.clone());

        // Headers personnalisés
        if let Some(ref headers) = webhook.callback.headers {
            for h in headers {
                req = req.header(&h.header_name, &h.header_value);
            }
        }

        // Authentification
        if let Some(ref auth) = webhook.callback.authentication {
            if auth.auth_type == "BASIC" {
                if let (Some(u), Some(p)) = (&auth.user_id, &auth.user_password) {
                    req = req.basic_auth(u, Some(p));
                }
            }
            // OAUTH2 non implémenté pour l'instant
        }

        // Signature HMAC SHA-256 (algo HS256)
        if let Some(ref sig) = webhook.callback.signature {
            if sig.algo == "HS256" {
                use hmac::{Hmac, Mac};
                use sha2::Sha256;
                if let Ok(key) = base64::Engine::decode(
                    &base64::engine::general_purpose::STANDARD,
                    &sig.key,
                ) {
                    if let Ok(mut mac) = Hmac::<Sha256>::new_from_slice(&key) {
                        mac.update(&body);
                        let result = mac.finalize().into_bytes();
                        let hex_sig = result
                            .iter()
                            .map(|b| format!("{:02x}", b))
                            .collect::<String>();
                        req = req.header("X-Webhook-Signature", format!("sha256={}", hex_sig));
                    }
                }
            }
        }

        req = req.header("X-Webhook-Event", payload.event.clone());
        req = req.header("X-Webhook-Id", webhook.webhook_id.to_string());

        let resp = req.send().await?;
        if !resp.status().is_success() {
            return Err(format!("HTTP {}", resp.status()).into());
        }
        tracing::debug!(
            webhook_id = %webhook.webhook_id,
            status = %resp.status(),
            "Webhook envoyé avec succès"
        );
        Ok(())
    }
}

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_request(flow_type: &str, direction: &str) -> WebhookCreateRequest {
        WebhookCreateRequest {
            callback: CallbackParameters {
                url: "https://example.com/webhook".to_string(),
                headers: None,
                authentication: None,
                signature: None,
            },
            metadata: WebhookMetadata {
                flow_type: flow_type.to_string(),
                flow_direction: direction.to_string(),
                processing_rule: None,
                ack_status: None,
            },
        }
    }

    #[test]
    fn test_store_create_and_get() {
        let store = WebhookStore::new();
        let webhook = store.create(sample_request("CustomerInvoice", "In"), None);
        assert_eq!(store.len(), 1);

        let fetched = store.get(&webhook.webhook_id).unwrap();
        assert_eq!(fetched.webhook_id, webhook.webhook_id);
        assert_eq!(fetched.metadata.flow_type, "CustomerInvoice");
    }

    #[test]
    fn test_store_list() {
        let store = WebhookStore::new();
        store.create(sample_request("CustomerInvoice", "In"), None);
        store.create(sample_request("SupplierInvoice", "Out"), None);
        assert_eq!(store.list(None).len(), 2);
    }

    #[test]
    fn test_store_update_partial() {
        let store = WebhookStore::new();
        let w = store.create(sample_request("CustomerInvoice", "In"), None);
        assert!(w.callback.headers.is_none());

        let patch = WebhookUpdateRequest {
            headers: Some(vec![CallbackHeader {
                header_name: "X-Custom".to_string(),
                header_value: "value".to_string(),
            }]),
            authentication: None,
            signature: None,
        };
        let updated = store.update(&w.webhook_id, patch).unwrap();
        assert_eq!(updated.callback.headers.unwrap().len(), 1);
    }

    #[test]
    fn test_store_delete() {
        let store = WebhookStore::new();
        let w = store.create(sample_request("CustomerInvoice", "In"), None);
        assert!(store.delete(&w.webhook_id));
        assert!(!store.delete(&w.webhook_id)); // déjà supprimé
        assert_eq!(store.len(), 0);
    }

    #[test]
    fn test_store_get_not_found() {
        let store = WebhookStore::new();
        assert!(store.get(&Uuid::new_v4()).is_none());
    }

    #[test]
    fn test_matching_filters() {
        let store = WebhookStore::new();
        store.create(sample_request("CustomerInvoice", "In"), None);
        store.create(sample_request("CustomerInvoice", "Out"), None);
        store.create(sample_request("SupplierInvoice", "In"), None);

        let matches = store.matching("CustomerInvoice", "In", None);
        assert_eq!(matches.len(), 1);

        let matches = store.matching("Cdar", "In", None);
        assert_eq!(matches.len(), 0);
    }

    #[test]
    fn test_matching_with_ack_status() {
        let store = WebhookStore::new();
        let mut req = sample_request("CustomerInvoice", "In");
        req.metadata.ack_status = Some("Ok".to_string());
        store.create(req, None);

        // Webhook filtré sur ackStatus=Ok ne matche que les events Ok
        assert_eq!(store.matching("CustomerInvoice", "In", Some("Ok")).len(), 1);
        assert_eq!(store.matching("CustomerInvoice", "In", Some("Error")).len(), 0);
        assert_eq!(store.matching("CustomerInvoice", "In", None).len(), 0);
    }

    #[test]
    fn test_event_type_str() {
        assert_eq!(WebhookEventType::FlowReceived.as_str(), "flow.received");
        assert_eq!(WebhookEventType::FlowAckUpdated.as_str(), "flow.ack.updated");
    }
}
