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
use sqlx::postgres::PgPool;
use sqlx::Row;
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
// Store : in-memory ou PostgreSQL
// ============================================================

/// Store thread-safe pour les webhooks.
///
/// Deux backends supportés :
/// - `Memory` : `HashMap` en RAM (utile pour tests et dev sans Postgres)
/// - `Postgres` : table `webhooks` (persistance, multi-instance)
pub enum WebhookStore {
    Memory(RwLock<HashMap<Uuid, Webhook>>),
    Postgres(PgPool),
}

impl Default for WebhookStore {
    fn default() -> Self {
        Self::Memory(RwLock::new(HashMap::new()))
    }
}

impl WebhookStore {
    /// Crée un store en mémoire (par défaut, pour les tests).
    pub fn new() -> Self {
        Self::default()
    }

    /// Crée un store PostgreSQL.
    pub fn new_postgres(pool: PgPool) -> Self {
        Self::Postgres(pool)
    }

    /// Crée la table `webhooks` si elle n'existe pas (no-op pour `Memory`).
    pub async fn migrate(&self) -> Result<(), sqlx::Error> {
        if let Self::Postgres(pool) = self {
            for stmt in WEBHOOK_SCHEMA_SQL.split(';') {
                let s = stmt.trim();
                if s.is_empty() {
                    continue;
                }
                sqlx::query(s).execute(pool).await?;
            }
            tracing::info!("Schéma webhooks initialisé");
        }
        Ok(())
    }

    /// Crée un nouveau webhook et retourne son ID
    pub async fn create(&self, req: WebhookCreateRequest, owner: Option<String>) -> Webhook {
        let webhook_id = Uuid::new_v4();
        let webhook = Webhook {
            webhook_id,
            callback: req.callback,
            metadata: req.metadata,
            owner,
        };
        match self {
            Self::Memory(inner) => {
                inner.write().unwrap().insert(webhook_id, webhook.clone());
            }
            Self::Postgres(pool) => {
                let headers_json = webhook.callback.headers.as_ref().map(|h| {
                    serde_json::to_value(h).unwrap_or(serde_json::Value::Null)
                });
                let auth_json = webhook.callback.authentication.as_ref().map(|a| {
                    serde_json::to_value(a).unwrap_or(serde_json::Value::Null)
                });
                let sig_json = webhook.callback.signature.as_ref().map(|s| {
                    serde_json::to_value(s).unwrap_or(serde_json::Value::Null)
                });
                if let Err(e) = sqlx::query(
                    "INSERT INTO webhooks (webhook_id, callback_url, callback_headers, \
                     callback_auth, callback_signature, flow_type, flow_direction, \
                     processing_rule, ack_status, owner) \
                     VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
                )
                .bind(webhook.webhook_id)
                .bind(&webhook.callback.url)
                .bind(headers_json)
                .bind(auth_json)
                .bind(sig_json)
                .bind(&webhook.metadata.flow_type)
                .bind(&webhook.metadata.flow_direction)
                .bind(webhook.metadata.processing_rule.as_deref())
                .bind(webhook.metadata.ack_status.as_deref())
                .bind(webhook.owner.as_deref())
                .execute(pool)
                .await
                {
                    tracing::error!(error = %e, "Insertion webhook PostgreSQL échouée");
                }
            }
        }
        webhook
    }

    /// Récupère un webhook par son ID
    pub async fn get(&self, webhook_id: &Uuid) -> Option<Webhook> {
        match self {
            Self::Memory(inner) => inner.read().unwrap().get(webhook_id).cloned(),
            Self::Postgres(pool) => {
                let row = sqlx::query(
                    "SELECT webhook_id, callback_url, callback_headers, callback_auth, \
                     callback_signature, flow_type, flow_direction, processing_rule, \
                     ack_status, owner FROM webhooks WHERE webhook_id = $1",
                )
                .bind(webhook_id)
                .fetch_optional(pool)
                .await
                .ok()
                .flatten()?;
                Some(row_to_webhook(&row))
            }
        }
    }

    /// Liste tous les webhooks (filtrés par owner si fourni)
    pub async fn list(&self, owner: Option<&str>) -> Vec<Webhook> {
        match self {
            Self::Memory(inner) => inner
                .read()
                .unwrap()
                .values()
                .filter(|w| match (owner, &w.owner) {
                    (None, _) => true,
                    (Some(o), Some(wo)) => o == wo,
                    (Some(_), None) => true, // webhooks sans owner sont visibles par tous
                })
                .cloned()
                .collect(),
            Self::Postgres(pool) => {
                let q = "SELECT webhook_id, callback_url, callback_headers, callback_auth, \
                         callback_signature, flow_type, flow_direction, processing_rule, \
                         ack_status, owner FROM webhooks \
                         WHERE $1::TEXT IS NULL OR owner IS NULL OR owner = $1";
                match sqlx::query(q)
                    .bind(owner)
                    .fetch_all(pool)
                    .await
                {
                    Ok(rows) => rows.iter().map(row_to_webhook).collect(),
                    Err(e) => {
                        tracing::error!(error = %e, "Listing webhooks PostgreSQL échoué");
                        Vec::new()
                    }
                }
            }
        }
    }

    /// Met à jour un webhook (PATCH partiel)
    pub async fn update(&self, webhook_id: &Uuid, patch: WebhookUpdateRequest) -> Option<Webhook> {
        match self {
            Self::Memory(inner) => {
                let mut store = inner.write().unwrap();
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
            Self::Postgres(pool) => {
                let mut current = self.get(webhook_id).await?;
                if let Some(headers) = patch.headers {
                    current.callback.headers = Some(headers);
                }
                if let Some(auth) = patch.authentication {
                    current.callback.authentication = Some(auth);
                }
                if let Some(sig) = patch.signature {
                    current.callback.signature = Some(sig);
                }
                let headers_json = current.callback.headers.as_ref().map(|h| {
                    serde_json::to_value(h).unwrap_or(serde_json::Value::Null)
                });
                let auth_json = current.callback.authentication.as_ref().map(|a| {
                    serde_json::to_value(a).unwrap_or(serde_json::Value::Null)
                });
                let sig_json = current.callback.signature.as_ref().map(|s| {
                    serde_json::to_value(s).unwrap_or(serde_json::Value::Null)
                });
                if let Err(e) = sqlx::query(
                    "UPDATE webhooks SET callback_headers = $2, callback_auth = $3, \
                     callback_signature = $4, updated_at = NOW() WHERE webhook_id = $1",
                )
                .bind(webhook_id)
                .bind(headers_json)
                .bind(auth_json)
                .bind(sig_json)
                .execute(pool)
                .await
                {
                    tracing::error!(error = %e, "Update webhook PostgreSQL échoué");
                    return None;
                }
                Some(current)
            }
        }
    }

    /// Supprime un webhook. Retourne `true` si le webhook existait.
    pub async fn delete(&self, webhook_id: &Uuid) -> bool {
        match self {
            Self::Memory(inner) => inner.write().unwrap().remove(webhook_id).is_some(),
            Self::Postgres(pool) => {
                match sqlx::query("DELETE FROM webhooks WHERE webhook_id = $1")
                    .bind(webhook_id)
                    .execute(pool)
                    .await
                {
                    Ok(r) => r.rows_affected() > 0,
                    Err(e) => {
                        tracing::error!(error = %e, "Suppression webhook PostgreSQL échouée");
                        false
                    }
                }
            }
        }
    }

    /// Filtre les webhooks correspondant à un événement.
    /// Utilisé par le `WebhookDispatcher` pour trouver les abonnements à notifier.
    pub async fn matching(
        &self,
        flow_type: &str,
        flow_direction: &str,
        ack_status: Option<&str>,
    ) -> Vec<Webhook> {
        match self {
            Self::Memory(inner) => inner
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
                .collect(),
            Self::Postgres(pool) => {
                // ack_status filter sémantique :
                //   - webhook.ack_status IS NULL  → matche tout
                //   - webhook.ack_status = X      → matche si event ack_status = X
                let q = "SELECT webhook_id, callback_url, callback_headers, callback_auth, \
                         callback_signature, flow_type, flow_direction, processing_rule, \
                         ack_status, owner FROM webhooks \
                         WHERE flow_type = $1 AND flow_direction = $2 \
                         AND (ack_status IS NULL OR ack_status = $3)";
                match sqlx::query(q)
                    .bind(flow_type)
                    .bind(flow_direction)
                    .bind(ack_status)
                    .fetch_all(pool)
                    .await
                {
                    Ok(rows) => rows.iter().map(row_to_webhook).collect(),
                    Err(e) => {
                        tracing::error!(error = %e, "Matching webhooks PostgreSQL échoué");
                        Vec::new()
                    }
                }
            }
        }
    }

    /// Nombre de webhooks enregistrés
    pub async fn len(&self) -> usize {
        match self {
            Self::Memory(inner) => inner.read().unwrap().len(),
            Self::Postgres(pool) => sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM webhooks")
                .fetch_one(pool)
                .await
                .map(|n| n as usize)
                .unwrap_or(0),
        }
    }

    pub async fn is_empty(&self) -> bool {
        self.len().await == 0
    }
}

/// Convertit une ligne PostgreSQL en `Webhook`.
fn row_to_webhook(row: &sqlx::postgres::PgRow) -> Webhook {
    let headers: Option<serde_json::Value> = row.try_get("callback_headers").ok();
    let auth: Option<serde_json::Value> = row.try_get("callback_auth").ok();
    let sig: Option<serde_json::Value> = row.try_get("callback_signature").ok();
    Webhook {
        webhook_id: row.try_get("webhook_id").unwrap_or_else(|_| Uuid::nil()),
        callback: CallbackParameters {
            url: row.try_get("callback_url").unwrap_or_default(),
            headers: headers.and_then(|v| serde_json::from_value(v).ok()),
            authentication: auth.and_then(|v| serde_json::from_value(v).ok()),
            signature: sig.and_then(|v| serde_json::from_value(v).ok()),
        },
        metadata: WebhookMetadata {
            flow_type: row.try_get("flow_type").unwrap_or_default(),
            flow_direction: row.try_get("flow_direction").unwrap_or_default(),
            processing_rule: row.try_get::<Option<String>, _>("processing_rule").ok().flatten(),
            ack_status: row.try_get::<Option<String>, _>("ack_status").ok().flatten(),
        },
        owner: row.try_get::<Option<String>, _>("owner").ok().flatten(),
    }
}

const WEBHOOK_SCHEMA_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS webhooks (
    webhook_id          UUID PRIMARY KEY,
    callback_url        TEXT NOT NULL,
    callback_headers    JSONB,
    callback_auth       JSONB,
    callback_signature  JSONB,
    flow_type           TEXT NOT NULL,
    flow_direction      TEXT NOT NULL,
    processing_rule     TEXT,
    ack_status          TEXT,
    owner               TEXT,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_webhooks_match ON webhooks(flow_type, flow_direction);
CREATE INDEX IF NOT EXISTS idx_webhooks_owner ON webhooks(owner);
"#;

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

    let webhook = state.webhook_store.create(req, None).await;

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
    let webhooks = state.webhook_store.list(None).await;
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
    match state.webhook_store.get(&webhook_id).await {
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
    match state.webhook_store.update(&webhook_id, patch).await {
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
    if state.webhook_store.delete(&webhook_id).await {
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

/// Configuration du retry du dispatcher webhook.
#[derive(Debug, Clone, Copy)]
pub struct RetryConfig {
    /// Nombre maximum de tentatives (la 1ère + retries)
    pub max_attempts: u32,
    /// Délai initial en millisecondes
    pub initial_delay_ms: u64,
    /// Multiplicateur exponentiel
    pub multiplier: u64,
    /// Délai maximum entre tentatives
    pub max_delay_ms: u64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay_ms: 500,
            multiplier: 2,
            max_delay_ms: 30_000,
        }
    }
}

/// Dispatcher de webhooks : envoie les événements aux abonnés.
pub struct WebhookDispatcher {
    store: Arc<WebhookStore>,
    client: reqwest::Client,
    retry: RetryConfig,
}

impl WebhookDispatcher {
    pub fn new(store: Arc<WebhookStore>) -> Self {
        Self {
            store,
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .unwrap_or_default(),
            retry: RetryConfig::default(),
        }
    }

    /// Configure le retry exponentiel.
    pub fn with_retry(mut self, retry: RetryConfig) -> Self {
        self.retry = retry;
        self
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
            .matching(flow_type, flow_direction, ack_status)
            .await;

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

            // Retry exponentiel
            let mut attempt = 0u32;
            let mut delay_ms = self.retry.initial_delay_ms;
            loop {
                attempt += 1;
                match self.send_one(&webhook, &payload).await {
                    Ok(()) => break,
                    Err(e) => {
                        if attempt >= self.retry.max_attempts {
                            tracing::warn!(
                                webhook_id = %webhook.webhook_id,
                                callback_url = %webhook.callback.url,
                                attempts = attempt,
                                error = %e,
                                "Échec d'envoi de webhook après retries (non bloquant)"
                            );
                            break;
                        }
                        tracing::debug!(
                            webhook_id = %webhook.webhook_id,
                            attempt = attempt,
                            next_delay_ms = delay_ms,
                            error = %e,
                            "Webhook en erreur, retry après backoff"
                        );
                        tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
                        delay_ms = (delay_ms * self.retry.multiplier).min(self.retry.max_delay_ms);
                    }
                }
            }
        }
    }

    /// Récupère un access token OAUTH2 via client_credentials grant.
    async fn fetch_oauth2_token(&self, auth: &CallbackAuthentication) -> Option<String> {
        let token_url = auth.token_url.as_deref()?;
        let client_id = auth.client_id.as_deref()?;
        let client_secret = auth.client_secret.as_deref()?;

        #[derive(serde::Deserialize)]
        struct TokenResponse {
            access_token: String,
        }

        let resp = self
            .client
            .post(token_url)
            .form(&[
                ("grant_type", "client_credentials"),
                ("client_id", client_id),
                ("client_secret", client_secret),
            ])
            .send()
            .await
            .ok()?;

        if !resp.status().is_success() {
            tracing::warn!(
                status = %resp.status(),
                token_url = %token_url,
                "OAUTH2 token endpoint a échoué"
            );
            return None;
        }

        resp.json::<TokenResponse>()
            .await
            .ok()
            .map(|tr| tr.access_token)
    }

    async fn send_one(
        &self,
        webhook: &Webhook,
        payload: &WebhookPayload,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
            match auth.auth_type.as_str() {
                "BASIC" => {
                    if let (Some(u), Some(p)) = (&auth.user_id, &auth.user_password) {
                        req = req.basic_auth(u, Some(p));
                    }
                }
                "OAUTH2" => {
                    if let Some(token) = self.fetch_oauth2_token(auth).await {
                        req = req.bearer_auth(token);
                    }
                }
                _ => {}
            }
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
// WebhookAckProcessor — déclenche flow.ack.updated dans le pipeline
// ============================================================

/// Processor qui déclenche un événement webhook `flow.ack.updated`
/// quand le statut de l'exchange atteint un point notable :
/// - `Validated` → ackStatus = "Ok"
/// - `Distributed` → ackStatus = "Ok"
/// - `Acknowledged` → ackStatus = "Ok"
/// - `Rejected` → ackStatus = "Error"
/// - `Error` → ackStatus = "Error"
///
/// Le dispatch est non-bloquant (tokio::spawn).
/// Utilise la propriété `webhook.last_ack_status` pour éviter les doublons.
pub struct WebhookAckProcessor {
    store: Arc<WebhookStore>,
    flow_direction: String,
}

impl WebhookAckProcessor {
    pub fn new(store: Arc<WebhookStore>, flow_direction: &str) -> Self {
        Self {
            store,
            flow_direction: flow_direction.to_string(),
        }
    }

    /// Mappe un FlowStatus vers un ackStatus AFNOR.
    fn map_ack_status(status: &pdp_core::model::FlowStatus) -> Option<&'static str> {
        use pdp_core::model::FlowStatus;
        match status {
            FlowStatus::Validated | FlowStatus::Distributed | FlowStatus::Acknowledged => {
                Some("Ok")
            }
            FlowStatus::Rejected | FlowStatus::Error => Some("Error"),
            _ => None,
        }
    }
}

#[async_trait::async_trait]
impl pdp_core::processor::Processor for WebhookAckProcessor {
    fn name(&self) -> &str {
        "WebhookAckProcessor"
    }

    async fn process(&self, exchange: pdp_core::exchange::Exchange) -> pdp_core::error::PdpResult<pdp_core::exchange::Exchange> {
        // Skip si pas d'invoice ou pas de statut notable
        let ack_status = match Self::map_ack_status(&exchange.status) {
            Some(s) => s,
            None => return Ok(exchange),
        };

        // Éviter les doublons : ne déclencher qu'une fois par valeur de ackStatus
        let last = exchange.get_property("webhook.last_ack_status");
        if last.map(|s| s.as_str()) == Some(ack_status) {
            return Ok(exchange);
        }

        let flow_id = exchange.flow_id.to_string();
        let flow_type = exchange
            .invoice
            .as_ref()
            .and_then(|i| i.invoice_type_code.clone())
            .unwrap_or_else(|| "Unknown".to_string());
        let direction = self.flow_direction.clone();
        let ack = ack_status.to_string();
        let store = self.store.clone();

        // Dispatch non-bloquant
        tokio::spawn(async move {
            let dispatcher = WebhookDispatcher::new(store);
            dispatcher
                .dispatch(
                    WebhookEventType::FlowAckUpdated,
                    &flow_id,
                    &flow_type,
                    &direction,
                    Some(&ack),
                )
                .await;
        });

        let mut exchange = exchange;
        exchange.set_property("webhook.last_ack_status", ack_status);
        Ok(exchange)
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

    #[tokio::test]
    async fn test_store_create_and_get() {
        let store = WebhookStore::new();
        let webhook = store.create(sample_request("CustomerInvoice", "In"), None).await;
        assert_eq!(store.len().await, 1);

        let fetched = store.get(&webhook.webhook_id).await.unwrap();
        assert_eq!(fetched.webhook_id, webhook.webhook_id);
        assert_eq!(fetched.metadata.flow_type, "CustomerInvoice");
    }

    #[tokio::test]
    async fn test_store_list() {
        let store = WebhookStore::new();
        store.create(sample_request("CustomerInvoice", "In"), None).await;
        store.create(sample_request("SupplierInvoice", "Out"), None).await;
        assert_eq!(store.list(None).await.len(), 2);
    }

    #[tokio::test]
    async fn test_store_update_partial() {
        let store = WebhookStore::new();
        let w = store.create(sample_request("CustomerInvoice", "In"), None).await;
        assert!(w.callback.headers.is_none());

        let patch = WebhookUpdateRequest {
            headers: Some(vec![CallbackHeader {
                header_name: "X-Custom".to_string(),
                header_value: "value".to_string(),
            }]),
            authentication: None,
            signature: None,
        };
        let updated = store.update(&w.webhook_id, patch).await.unwrap();
        assert_eq!(updated.callback.headers.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_store_delete() {
        let store = WebhookStore::new();
        let w = store.create(sample_request("CustomerInvoice", "In"), None).await;
        assert!(store.delete(&w.webhook_id).await);
        assert!(!store.delete(&w.webhook_id).await); // déjà supprimé
        assert_eq!(store.len().await, 0);
    }

    #[tokio::test]
    async fn test_store_get_not_found() {
        let store = WebhookStore::new();
        assert!(store.get(&Uuid::new_v4()).await.is_none());
    }

    #[tokio::test]
    async fn test_matching_filters() {
        let store = WebhookStore::new();
        store.create(sample_request("CustomerInvoice", "In"), None).await;
        store.create(sample_request("CustomerInvoice", "Out"), None).await;
        store.create(sample_request("SupplierInvoice", "In"), None).await;

        let matches = store.matching("CustomerInvoice", "In", None).await;
        assert_eq!(matches.len(), 1);

        let matches = store.matching("Cdar", "In", None).await;
        assert_eq!(matches.len(), 0);
    }

    #[tokio::test]
    async fn test_matching_with_ack_status() {
        let store = WebhookStore::new();
        let mut req = sample_request("CustomerInvoice", "In");
        req.metadata.ack_status = Some("Ok".to_string());
        store.create(req, None).await;

        // Webhook filtré sur ackStatus=Ok ne matche que les events Ok
        assert_eq!(store.matching("CustomerInvoice", "In", Some("Ok")).await.len(), 1);
        assert_eq!(store.matching("CustomerInvoice", "In", Some("Error")).await.len(), 0);
        assert_eq!(store.matching("CustomerInvoice", "In", None).await.len(), 0);
    }

    #[test]
    fn test_event_type_str() {
        assert_eq!(WebhookEventType::FlowReceived.as_str(), "flow.received");
        assert_eq!(WebhookEventType::FlowAckUpdated.as_str(), "flow.ack.updated");
    }
}

// ============================================================
// Tests d'intégration PostgreSQL (testcontainers, requiert Docker)
// ============================================================

#[cfg(test)]
mod pg_tests {
    use super::*;
    use sqlx::postgres::PgPoolOptions;
    use testcontainers::runners::AsyncRunner;
    use testcontainers::ContainerAsync;
    use testcontainers_modules::postgres::Postgres;

    async fn setup() -> (ContainerAsync<Postgres>, WebhookStore) {
        let container = Postgres::default().start().await.expect("démarrage Postgres");
        let host = container.get_host().await.unwrap();
        let port = container.get_host_port_ipv4(5432).await.unwrap();
        let url = format!("postgres://postgres:postgres@{}:{}/postgres", host, port);
        let pool = PgPoolOptions::new()
            .max_connections(2)
            .connect(&url)
            .await
            .unwrap();
        let store = WebhookStore::new_postgres(pool);
        store.migrate().await.unwrap();
        (container, store)
    }

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

    #[tokio::test]
    async fn test_pg_create_and_get() {
        let (_c, store) = setup().await;
        let webhook = store.create(sample_request("CustomerInvoice", "In"), None).await;
        let fetched = store.get(&webhook.webhook_id).await.expect("get");
        assert_eq!(fetched.webhook_id, webhook.webhook_id);
        assert_eq!(fetched.metadata.flow_type, "CustomerInvoice");
        assert_eq!(fetched.callback.url, "https://example.com/webhook");
        assert_eq!(store.len().await, 1);
    }

    #[tokio::test]
    async fn test_pg_get_not_found() {
        let (_c, store) = setup().await;
        assert!(store.get(&Uuid::new_v4()).await.is_none());
    }

    #[tokio::test]
    async fn test_pg_list_filters_by_owner() {
        let (_c, store) = setup().await;
        store
            .create(sample_request("CustomerInvoice", "In"), Some("tenant-a".to_string()))
            .await;
        store
            .create(sample_request("SupplierInvoice", "Out"), Some("tenant-b".to_string()))
            .await;
        store.create(sample_request("Cdar", "In"), None).await;

        assert_eq!(store.list(None).await.len(), 3);
        // tenant-a voit son webhook + ceux sans owner (visibles par tous)
        assert_eq!(store.list(Some("tenant-a")).await.len(), 2);
        assert_eq!(store.list(Some("tenant-b")).await.len(), 2);
        assert_eq!(store.list(Some("tenant-c")).await.len(), 1);
    }

    #[tokio::test]
    async fn test_pg_update_partial_persists() {
        let (_c, store) = setup().await;
        let w = store.create(sample_request("CustomerInvoice", "In"), None).await;

        let patch = WebhookUpdateRequest {
            headers: Some(vec![CallbackHeader {
                header_name: "X-Custom".to_string(),
                header_value: "value".to_string(),
            }]),
            authentication: None,
            signature: None,
        };
        store.update(&w.webhook_id, patch).await.expect("update");

        // Re-lecture : la modification est bien persistée
        let reread = store.get(&w.webhook_id).await.expect("reread");
        let h = reread.callback.headers.expect("headers");
        assert_eq!(h.len(), 1);
        assert_eq!(h[0].header_name, "X-Custom");
        assert_eq!(h[0].header_value, "value");
    }

    #[tokio::test]
    async fn test_pg_delete_removes_persisted() {
        let (_c, store) = setup().await;
        let w = store.create(sample_request("CustomerInvoice", "In"), None).await;
        assert!(store.delete(&w.webhook_id).await);
        assert!(!store.delete(&w.webhook_id).await);
        assert!(store.get(&w.webhook_id).await.is_none());
        assert_eq!(store.len().await, 0);
    }

    #[tokio::test]
    async fn test_pg_matching_filters() {
        let (_c, store) = setup().await;
        store.create(sample_request("CustomerInvoice", "In"), None).await;
        store.create(sample_request("CustomerInvoice", "Out"), None).await;
        store.create(sample_request("SupplierInvoice", "In"), None).await;

        let m = store.matching("CustomerInvoice", "In", None).await;
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].metadata.flow_direction, "In");

        let m = store.matching("Cdar", "In", None).await;
        assert_eq!(m.len(), 0);
    }

    #[tokio::test]
    async fn test_pg_matching_ack_status() {
        let (_c, store) = setup().await;
        let mut req = sample_request("CustomerInvoice", "In");
        req.metadata.ack_status = Some("Ok".to_string());
        store.create(req, None).await;

        assert_eq!(store.matching("CustomerInvoice", "In", Some("Ok")).await.len(), 1);
        assert_eq!(store.matching("CustomerInvoice", "In", Some("Error")).await.len(), 0);
    }

    #[tokio::test]
    async fn test_pg_roundtrip_complex_payload() {
        // Webhook avec auth BASIC + headers + processing_rule + ack_status + owner
        let (_c, store) = setup().await;

        let req = WebhookCreateRequest {
            callback: CallbackParameters {
                url: "https://hook.example.com/x".to_string(),
                headers: Some(vec![CallbackHeader {
                    header_name: "Authorization".to_string(),
                    header_value: "Bearer xyz".to_string(),
                }]),
                authentication: Some(CallbackAuthentication {
                    auth_type: "BASIC".to_string(),
                    user_id: Some("alice".to_string()),
                    user_password: Some("secret".to_string()),
                    token_url: None,
                    client_id: None,
                    client_secret: None,
                }),
                signature: None,
            },
            metadata: WebhookMetadata {
                flow_type: "Cdar".to_string(),
                flow_direction: "In".to_string(),
                processing_rule: Some("default".to_string()),
                ack_status: Some("Error".to_string()),
            },
        };

        let created = store.create(req, Some("tenant-x".to_string())).await;
        let fetched = store.get(&created.webhook_id).await.expect("get");

        assert_eq!(fetched.callback.url, "https://hook.example.com/x");
        assert_eq!(fetched.callback.headers.as_ref().unwrap().len(), 1);
        let auth = fetched.callback.authentication.as_ref().expect("auth");
        assert_eq!(auth.auth_type, "BASIC");
        assert_eq!(auth.user_id.as_deref(), Some("alice"));
        assert_eq!(fetched.metadata.processing_rule.as_deref(), Some("default"));
        assert_eq!(fetched.metadata.ack_status.as_deref(), Some("Error"));
        assert_eq!(fetched.owner.as_deref(), Some("tenant-x"));
    }
}
