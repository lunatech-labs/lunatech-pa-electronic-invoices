//! Serveur HTTP API conforme AFNOR XP Z12-013
//!
//! Expose les endpoints suivants :
//! - POST /v1/flows — Réception de flux entrants (factures, CDV, e-reporting)
//! - POST /v1/flows/search — Recherche de flux
//! - GET /v1/flows/{flowId} — Consultation d'un flux
//! - GET /v1/flows?status=error — Liste des flux en erreur
//! - GET /v1/stats — Statistiques du pipeline
//! - POST /v1/webhooks/callback — Réception de notifications webhook
//! - GET /v1/healthcheck — Health check
//! - GET /metrics — Métriques Prometheus
//!
//! Ce serveur implémente le rôle de "PDP réceptrice" dans l'architecture AFNOR.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use axum::{
    Router,
    extract::{Multipart, Path, Query, State},
    http::StatusCode,
    middleware,
    response::IntoResponse,
    routing::{get, post},
    Json,
};
use serde::{Deserialize, Serialize};
use tracing;

// ============================================================
// État partagé du serveur
// ============================================================

/// Compteurs Prometheus pour le monitoring du pipeline
pub struct Metrics {
    /// Nombre total de flux reçus via HTTP
    pub flows_received: AtomicU64,
    /// Nombre total de flux acceptés par le pipeline
    pub flows_accepted: AtomicU64,
    /// Nombre total de flux rejetés (erreur de validation, SHA, etc.)
    pub flows_rejected: AtomicU64,
    /// Nombre total de webhooks reçus
    pub webhooks_received: AtomicU64,
}

impl Default for Metrics {
    fn default() -> Self {
        Self {
            flows_received: AtomicU64::new(0),
            flows_accepted: AtomicU64::new(0),
            flows_rejected: AtomicU64::new(0),
            webhooks_received: AtomicU64::new(0),
        }
    }
}

/// État partagé du serveur HTTP
pub struct AppState {
    /// Nom de la PDP
    pub pdp_name: String,
    /// Matricule de la PDP
    pub pdp_matricule: String,
    /// Sender pour injecter les flux reçus dans le pipeline
    pub flow_sender: tokio::sync::mpsc::Sender<InboundFlow>,
    /// Secret HMAC pour la vérification des signatures webhook
    pub webhook_secret: Option<String>,
    /// Store pour la traçabilité
    pub trace_store: Option<Arc<pdp_trace::TraceStore>>,
    /// Tokens Bearer autorisés pour l'authentification API (si None ou vide, pas d'auth)
    pub bearer_tokens: Option<Vec<String>>,
    /// Métriques Prometheus
    pub metrics: Metrics,
    /// Store annuaire PPF (optionnel — nécessite PostgreSQL)
    pub annuaire_store: Option<pdp_annuaire::AnnuaireStore>,
}

/// Flux entrant reçu via l'API HTTP
#[derive(Debug)]
pub struct InboundFlow {
    /// Métadonnées du flux
    pub flow_info: InboundFlowInfo,
    /// Nom du fichier
    pub filename: String,
    /// Contenu du fichier
    pub content: Vec<u8>,
}

/// Métadonnées d'un flux entrant (subset de AfnorFlowInfo)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InboundFlowInfo {
    pub tracking_id: String,
    pub name: String,
    #[serde(default)]
    pub processing_rule: Option<String>,
    #[serde(default)]
    pub flow_syntax: Option<String>,
    #[serde(default)]
    pub flow_profile: Option<String>,
    #[serde(default)]
    pub flow_type: Option<String>,
    #[serde(default)]
    pub sha256: Option<String>,
    #[serde(default)]
    pub callback_url: Option<String>,
}

// ============================================================
// Réponses API
// ============================================================

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FlowAcceptedResponse {
    pub flow_id: String,
    pub status: String,
    pub message: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthCheckResponse {
    pub status: String,
    pub pdp_name: String,
    pub version: String,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

// ============================================================
// Construction du routeur
// ============================================================

/// Construit le routeur Axum avec tous les endpoints AFNOR
pub fn build_api_router(state: Arc<AppState>) -> Router {
    // Endpoints protégés par Bearer token (flows, stats, webhooks)
    let protected_routes = Router::new()
        .route("/v1/flows", post(handle_receive_flow).get(handle_list_flows))
        .route("/v1/flows/{flow_id}", get(handle_get_flow))
        .route("/v1/stats", get(handle_stats))
        .route("/v1/webhooks/callback", post(handle_webhook_callback))
        // Annuaire PPF — Directory Service conforme AFNOR XP Z12-013 Annexe B
        .route("/v1/siren/code-insee:{siren}", get(handle_ds_get_siren))
        .route("/v1/siren/search", post(handle_ds_search_siren))
        .route("/v1/siret/code-insee:{siret}", get(handle_ds_get_siret))
        .route("/v1/siret/search", post(handle_ds_search_siret))
        .route("/v1/routing-code/search", post(handle_ds_search_routing))
        .route("/v1/directory-line/code:{addressing_id}", get(handle_ds_get_directory_line))
        .route("/v1/directory-line/search", post(handle_ds_search_directory_lines))
        // Endpoints internes (stats, plateformes)
        .route("/v1/annuaire/stats", get(handle_annuaire_stats))
        .route("/v1/annuaire/plateformes", get(handle_annuaire_plateformes))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ))
        .with_state(state.clone());

    // Endpoints publics (healthcheck, métriques Prometheus)
    let public_routes = Router::new()
        .route("/v1/healthcheck", get(handle_healthcheck))
        .route("/metrics", get(handle_metrics))
        .with_state(state);

    Router::new()
        .merge(protected_routes)
        .merge(public_routes)
}

/// Middleware d'authentification Bearer token pour les endpoints protégés.
///
/// Si aucun token n'est configuré (`bearer_tokens` absent ou vide), toutes les
/// requêtes sont acceptées (mode développement). Sinon, le header
/// `Authorization: Bearer <token>` est vérifié.
async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    req: axum::extract::Request,
    next: middleware::Next,
) -> axum::response::Response {
    // Si pas de tokens configurés, tout passe (mode développement)
    let tokens = match &state.bearer_tokens {
        Some(tokens) if !tokens.is_empty() => tokens,
        _ => return next.run(req).await,
    };

    // Vérifier le header Authorization
    let auth_header = req
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok());

    match auth_header {
        Some(header) if header.starts_with("Bearer ") => {
            let token = &header[7..];
            if tokens.iter().any(|t| t == token) {
                next.run(req).await
            } else {
                (
                    StatusCode::UNAUTHORIZED,
                    Json(ErrorResponse {
                        error: "INVALID_TOKEN".to_string(),
                        message: "Token invalide".to_string(),
                    }),
                )
                    .into_response()
            }
        }
        _ => (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "MISSING_TOKEN".to_string(),
                message: "Header Authorization Bearer requis".to_string(),
            }),
        )
            .into_response(),
    }
}

// ============================================================
// Handlers
// ============================================================

/// POST /v1/flows — Réception d'un flux entrant (multipart/form-data)
///
/// Conforme AFNOR XP Z12-013 §5.3.1 :
/// - Part "flowInfo" : JSON avec les métadonnées du flux
/// - Part "file" : contenu binaire du document
///
/// Retourne 202 Accepted avec l'ID du flux créé
async fn handle_receive_flow(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let mut flow_info: Option<InboundFlowInfo> = None;
    let mut file_content: Option<Vec<u8>> = None;
    let mut filename: Option<String> = None;

    // Parser les parts multipart
    while let Ok(Some(field)) = multipart.next_field().await {
        let field_name = field.name().unwrap_or("").to_string();

        match field_name.as_str() {
            "flowInfo" => {
                match field.text().await {
                    Ok(text) => {
                        match serde_json::from_str::<InboundFlowInfo>(&text) {
                            Ok(info) => flow_info = Some(info),
                            Err(e) => {
                                tracing::warn!(error = %e, "flowInfo JSON invalide");
                                return (
                                    StatusCode::BAD_REQUEST,
                                    Json(ErrorResponse {
                                        error: "INVALID_FLOW_INFO".to_string(),
                                        message: format!("flowInfo invalide: {}", e),
                                    }),
                                ).into_response();
                            }
                        }
                    }
                    Err(e) => {
                        return (
                            StatusCode::BAD_REQUEST,
                            Json(ErrorResponse {
                                error: "READ_ERROR".to_string(),
                                message: format!("Impossible de lire flowInfo: {}", e),
                            }),
                        ).into_response();
                    }
                }
            }
            "file" => {
                filename = field.file_name().map(|s| s.to_string());
                match field.bytes().await {
                    Ok(bytes) => file_content = Some(bytes.to_vec()),
                    Err(e) => {
                        return (
                            StatusCode::BAD_REQUEST,
                            Json(ErrorResponse {
                                error: "READ_ERROR".to_string(),
                                message: format!("Impossible de lire le fichier: {}", e),
                            }),
                        ).into_response();
                    }
                }
            }
            _ => {
                tracing::debug!(field = %field_name, "Part multipart ignorée");
            }
        }
    }

    // Vérifier que les deux parts obligatoires sont présentes
    let flow_info = match flow_info {
        Some(info) => info,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: "MISSING_FLOW_INFO".to_string(),
                    message: "La part 'flowInfo' est obligatoire".to_string(),
                }),
            ).into_response();
        }
    };

    let content = match file_content {
        Some(c) => c,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: "MISSING_FILE".to_string(),
                    message: "La part 'file' est obligatoire".to_string(),
                }),
            ).into_response();
        }
    };

    let filename = filename.unwrap_or_else(|| flow_info.name.clone());

    // Vérifier le SHA-256 si fourni
    if let Some(ref expected_sha) = flow_info.sha256 {
        use sha2::{Digest, Sha256};
        let actual_sha = format!("{:x}", Sha256::digest(&content));
        if actual_sha != *expected_sha {
            state.metrics.flows_rejected.fetch_add(1, Ordering::Relaxed);
            tracing::warn!(
                tracking_id = %flow_info.tracking_id,
                expected = %expected_sha,
                actual = %actual_sha,
                "SHA-256 ne correspond pas"
            );
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: "SHA256_MISMATCH".to_string(),
                    message: format!(
                        "SHA-256 attendu: {}, reçu: {}",
                        expected_sha, actual_sha
                    ),
                }),
            ).into_response();
        }
    }

    // Générer un flow_id
    let flow_id = uuid::Uuid::new_v4().to_string();

    // Incrémenter le compteur de flux reçus
    state.metrics.flows_received.fetch_add(1, Ordering::Relaxed);

    // API AFNOR XP Z12-013 : le flux est toujours un fichier unique (XML, PDF)
    // PAS de tar.gz ici — le tar.gz est réservé au protocole SFTP PDP↔PPF
    tracing::info!(
        flow_id = %flow_id,
        tracking_id = %flow_info.tracking_id,
        filename = %filename,
        flow_syntax = flow_info.flow_syntax.as_deref().unwrap_or("N/A"),
        flow_type = flow_info.flow_type.as_deref().unwrap_or("N/A"),
        size = content.len(),
        "Flux entrant reçu via API HTTP"
    );

    let inbound = InboundFlow {
        flow_info: flow_info.clone(),
        filename,
        content,
    };

    match state.flow_sender.send(inbound).await {
        Ok(_) => {
            state.metrics.flows_accepted.fetch_add(1, Ordering::Relaxed);
            (
                StatusCode::ACCEPTED,
                Json(FlowAcceptedResponse {
                    flow_id,
                    status: "RECEIVED".to_string(),
                    message: "Flux accepté pour traitement".to_string(),
                }),
            ).into_response()
        }
        Err(e) => {
            state.metrics.flows_rejected.fetch_add(1, Ordering::Relaxed);
            tracing::error!(
                tracking_id = %flow_info.tracking_id,
                error = %e,
                "Impossible d'envoyer le flux dans le pipeline"
            );
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ErrorResponse {
                    error: "PIPELINE_UNAVAILABLE".to_string(),
                    message: "Le pipeline de traitement n'est pas disponible".to_string(),
                }),
            ).into_response()
        }
    }
}

/// Paramètres de requête pour GET /v1/flows
#[derive(Debug, Deserialize)]
pub struct FlowsQueryParams {
    /// Filtrer par statut (ex: "error")
    pub status: Option<String>,
}

/// Réponse détaillée pour la consultation d'un flux
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FlowDetailResponse {
    pub exchange_id: String,
    pub flow_id: String,
    pub status: String,
    pub invoice_number: Option<String>,
    pub seller_siret: Option<String>,
    pub buyer_siret: Option<String>,
    pub created_at: String,
    pub errors: Vec<FlowErrorEntry>,
}

/// Entrée d'erreur dans la réponse de consultation
#[derive(Debug, Serialize)]
pub struct FlowErrorEntry {
    pub step: String,
    pub message: String,
    pub detail: Option<String>,
    pub timestamp: String,
}

/// Réponse des statistiques pipeline
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatsResponse {
    pub total_exchanges: i64,
    pub total_errors: i64,
    pub total_distributed: i64,
}

/// GET /v1/flows/{flowId} — Consultation d'un flux via le TraceStore
async fn handle_get_flow(
    State(state): State<Arc<AppState>>,
    Path(flow_id): Path<String>,
) -> impl IntoResponse {
    let trace_store = match &state.trace_store {
        Some(store) => store,
        None => {
            return (
                StatusCode::NOT_IMPLEMENTED,
                Json(serde_json::json!({
                    "error": "NOT_IMPLEMENTED",
                    "message": "TraceStore non configuré"
                })),
            ).into_response();
        }
    };

    tracing::debug!(flow_id = %flow_id, "Consultation flux");

    match trace_store.get_exchange(&flow_id, None).await {
        Ok(Some(doc)) => {
            let errors: Vec<FlowErrorEntry> = doc.errors.iter().map(|e| FlowErrorEntry {
                step: e.step.clone(),
                message: e.message.clone(),
                detail: e.detail.clone(),
                timestamp: e.timestamp.clone(),
            }).collect();

            (
                StatusCode::OK,
                Json(serde_json::to_value(FlowDetailResponse {
                    exchange_id: doc.exchange_id,
                    flow_id: doc.flow_id,
                    status: doc.status,
                    invoice_number: doc.invoice_number,
                    seller_siret: doc.seller_siret,
                    buyer_siret: doc.buyer_siret,
                    created_at: doc.created_at,
                    errors,
                }).unwrap_or_default()),
            ).into_response()
        }
        Ok(None) => {
            (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({
                    "error": "NOT_FOUND",
                    "message": format!("Flux '{}' introuvable", flow_id)
                })),
            ).into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, flow_id = %flow_id, "Erreur consultation flux");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "INTERNAL_ERROR",
                    "message": format!("Erreur interne: {}", e)
                })),
            ).into_response()
        }
    }
}

/// GET /v1/flows?status=error — Liste des flux filtrés (pour l'instant, seul status=error est supporté)
async fn handle_list_flows(
    State(state): State<Arc<AppState>>,
    Query(params): Query<FlowsQueryParams>,
) -> impl IntoResponse {
    // Valider les paramètres AVANT de vérifier le TraceStore
    match params.status.as_deref() {
        Some("error") => {}
        Some(other) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "INVALID_FILTER",
                    "message": format!("Filtre status='{}' non supporté (valeurs acceptées: error)", other)
                })),
            ).into_response();
        }
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "MISSING_FILTER",
                    "message": "Le paramètre 'status' est requis (ex: ?status=error)"
                })),
            ).into_response();
        }
    }

    let trace_store = match &state.trace_store {
        Some(store) => store,
        None => {
            return (
                StatusCode::NOT_IMPLEMENTED,
                Json(serde_json::json!({
                    "error": "NOT_IMPLEMENTED",
                    "message": "TraceStore non configuré"
                })),
            ).into_response();
        }
    };

    // status=error validé ci-dessus
    match trace_store.get_error_flows().await {
        Ok(flows) => {
            (StatusCode::OK, Json(serde_json::to_value(flows).unwrap_or_default())).into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, "Erreur liste flux en erreur");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "INTERNAL_ERROR",
                    "message": format!("Erreur interne: {}", e)
                })),
            ).into_response()
        }
    }
}

/// GET /v1/stats — Statistiques globales du pipeline
async fn handle_stats(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let trace_store = match &state.trace_store {
        Some(store) => store,
        None => {
            return (
                StatusCode::NOT_IMPLEMENTED,
                Json(serde_json::json!({
                    "error": "NOT_IMPLEMENTED",
                    "message": "TraceStore non configuré"
                })),
            ).into_response();
        }
    };

    match trace_store.get_stats().await {
        Ok(stats) => {
            (
                StatusCode::OK,
                Json(serde_json::to_value(StatsResponse {
                    total_exchanges: stats.total_exchanges,
                    total_errors: stats.total_errors,
                    total_distributed: stats.total_distributed,
                }).unwrap_or_default()),
            ).into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, "Erreur récupération statistiques");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "INTERNAL_ERROR",
                    "message": format!("Erreur interne: {}", e)
                })),
            ).into_response()
        }
    }
}

/// POST /v1/webhooks/callback — Réception de notifications webhook
///
/// Vérifie la signature HMAC-SHA256 si un secret est configuré.
/// Les événements supportés :
/// - flow.received : un flux a été reçu par la PDP distante
/// - flow.ack.updated : le statut d'acquittement d'un flux a changé
async fn handle_webhook_callback(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    body: axum::body::Bytes,
) -> impl IntoResponse {
    // Vérifier la signature HMAC si un secret est configuré
    if let Some(ref secret) = state.webhook_secret {
        let signature = headers
            .get("X-Webhook-Signature")
            .or_else(|| headers.get("x-webhook-signature"))
            .and_then(|v| v.to_str().ok());

        match signature {
            Some(sig) => {
                if !verify_hmac_signature(secret.as_bytes(), &body, sig) {
                    tracing::warn!("Signature webhook HMAC invalide");
                    return (
                        StatusCode::UNAUTHORIZED,
                        Json(ErrorResponse {
                            error: "INVALID_SIGNATURE".to_string(),
                            message: "Signature HMAC-SHA256 invalide".to_string(),
                        }),
                    ).into_response();
                }
            }
            None => {
                tracing::warn!("Aucune signature webhook fournie");
                return (
                    StatusCode::UNAUTHORIZED,
                    Json(ErrorResponse {
                        error: "MISSING_SIGNATURE".to_string(),
                        message: "L'en-tête X-Webhook-Signature est requis".to_string(),
                    }),
                ).into_response();
                }
        }
    }

    // Parser le corps du webhook
    let event: WebhookCallbackEvent = match serde_json::from_slice(&body) {
        Ok(e) => e,
        Err(e) => {
            tracing::warn!(error = %e, "Webhook JSON invalide");
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: "INVALID_WEBHOOK".to_string(),
                    message: format!("JSON invalide: {}", e),
                }),
            ).into_response();
        }
    };

    state.metrics.webhooks_received.fetch_add(1, Ordering::Relaxed);

    tracing::info!(
        event_type = %event.event_type,
        flow_id = event.flow_id.as_deref().unwrap_or("N/A"),
        tracking_id = event.tracking_id.as_deref().unwrap_or("N/A"),
        "Webhook reçu"
    );

    match event.event_type.as_str() {
        "flow.received" => {
            tracing::info!(
                flow_id = event.flow_id.as_deref().unwrap_or("N/A"),
                "Flux reçu par la PDP distante"
            );
            // TODO: mettre à jour le statut du flux dans le TraceStore
        }
        "flow.ack.updated" => {
            tracing::info!(
                flow_id = event.flow_id.as_deref().unwrap_or("N/A"),
                status = event.status.as_deref().unwrap_or("N/A"),
                "Statut d'acquittement mis à jour"
            );
            // TODO: mettre à jour le statut du flux dans le TraceStore
        }
        other => {
            tracing::debug!(event_type = %other, "Type d'événement webhook non géré");
        }
    }

    (StatusCode::OK, Json(serde_json::json!({ "status": "ok" }))).into_response()
}

/// GET /metrics — Métriques au format Prometheus text exposition
async fn handle_metrics(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let flows_received = state.metrics.flows_received.load(Ordering::Relaxed);
    let flows_accepted = state.metrics.flows_accepted.load(Ordering::Relaxed);
    let flows_rejected = state.metrics.flows_rejected.load(Ordering::Relaxed);
    let webhooks_received = state.metrics.webhooks_received.load(Ordering::Relaxed);

    let body = format!(
        "# HELP pdp_flows_received_total Nombre total de flux reçus via HTTP\n\
         # TYPE pdp_flows_received_total counter\n\
         pdp_flows_received_total {}\n\
         # HELP pdp_flows_accepted_total Nombre total de flux acceptés\n\
         # TYPE pdp_flows_accepted_total counter\n\
         pdp_flows_accepted_total {}\n\
         # HELP pdp_flows_rejected_total Nombre total de flux rejetés\n\
         # TYPE pdp_flows_rejected_total counter\n\
         pdp_flows_rejected_total {}\n\
         # HELP pdp_webhooks_received_total Nombre total de webhooks reçus\n\
         # TYPE pdp_webhooks_received_total counter\n\
         pdp_webhooks_received_total {}\n",
        flows_received, flows_accepted, flows_rejected, webhooks_received,
    );

    (
        StatusCode::OK,
        [(axum::http::header::CONTENT_TYPE, "text/plain; version=0.0.4; charset=utf-8")],
        body,
    )
}

/// GET /v1/healthcheck — Health check
async fn handle_healthcheck(
    State(state): State<Arc<AppState>>,
) -> Json<HealthCheckResponse> {
    Json(HealthCheckResponse {
        status: "UP".to_string(),
        pdp_name: state.pdp_name.clone(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

// ============================================================
// Modèles webhook
// ============================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebhookCallbackEvent {
    pub event_type: String,
    pub flow_id: Option<String>,
    pub tracking_id: Option<String>,
    pub status: Option<String>,
    pub reason_code: Option<String>,
    pub reason_message: Option<String>,
    pub timestamp: Option<String>,
}

// ============================================================
// Vérification HMAC-SHA256
// ============================================================

/// Vérifie la signature HMAC-SHA256 d'un webhook.
///
/// La signature attendue est au format "sha256={hex_digest}"
fn verify_hmac_signature(secret: &[u8], body: &[u8], signature: &str) -> bool {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    type HmacSha256 = Hmac<Sha256>;

    // Strip le préfixe "sha256=" si présent
    let hex_sig = signature
        .strip_prefix("sha256=")
        .unwrap_or(signature);

    // Décoder la signature hex
    let expected_bytes = match hex::decode(hex_sig) {
        Ok(b) => b,
        Err(_) => return false,
    };

    // Calculer le HMAC
    let mut mac = match HmacSha256::new_from_slice(secret) {
        Ok(m) => m,
        Err(_) => return false,
    };
    mac.update(body);

    // Vérification en temps constant
    mac.verify_slice(&expected_bytes).is_ok()
}

// ============================================================
// Démarrage du serveur
// ============================================================

/// Configuration du serveur HTTP
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Adresse d'écoute (ex: "0.0.0.0")
    pub host: String,
    /// Port d'écoute
    pub port: u16,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 8080,
        }
    }
}

// ============================================================
// Handlers Directory Service — AFNOR XP Z12-013 Annexe B
// ============================================================

/// Macro pour extraire le store annuaire ou retourner 503
macro_rules! require_annuaire {
    ($state:expr) => {
        match &$state.annuaire_store {
            Some(s) => s,
            None => return (StatusCode::SERVICE_UNAVAILABLE, Json(serde_json::json!({
                "error": "Annuaire non configuré (PostgreSQL requis)"
            }))).into_response(),
        }
    };
}

/// GET /v1/siren/code-insee:{siren}
async fn handle_ds_get_siren(
    State(state): State<Arc<AppState>>,
    Path(siren): Path<String>,
) -> impl IntoResponse {
    let store = require_annuaire!(state);

    let ul = match store.lookup_unite_legale(&siren).await {
        Ok(Some(ul)) => ul,
        Ok(None) => return (StatusCode::NOT_FOUND, Json(serde_json::json!({
            "error": format!("SIREN {} non trouvé", siren)
        }))).into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    };

    (StatusCode::OK, Json(serde_json::json!({
        "siren": ul.siren.trim(),
        "raisonSociale": ul.nom,
        "typeEntite": ul.type_entite.as_code(),
        "statutAdministratif": ul.statut.as_code(),
    }))).into_response()
}

/// POST /v1/siren/search
async fn handle_ds_search_siren(
    State(state): State<Arc<AppState>>,
    Json(params): Json<serde_json::Value>,
) -> impl IntoResponse {
    let store = require_annuaire!(state);
    let siren = params.get("siren").and_then(|v| v.as_str()).unwrap_or("");

    if siren.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": "Le champ 'siren' est requis"}))).into_response();
    }

    let ul = match store.lookup_unite_legale(siren).await {
        Ok(Some(ul)) => ul,
        Ok(None) => return (StatusCode::OK, Json(serde_json::json!({"items": [], "total": 0}))).into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    };

    (StatusCode::OK, Json(serde_json::json!({
        "items": [{
            "siren": ul.siren.trim(),
            "raisonSociale": ul.nom,
            "typeEntite": ul.type_entite.as_code(),
            "statutAdministratif": ul.statut.as_code(),
        }],
        "total": 1
    }))).into_response()
}

/// GET /v1/siret/code-insee:{siret}
async fn handle_ds_get_siret(
    State(state): State<Arc<AppState>>,
    Path(siret): Path<String>,
) -> impl IntoResponse {
    let store = require_annuaire!(state);

    let siren = if siret.len() >= 9 { &siret[..9] } else { &siret };
    let etabs = match store.lookup_etablissements(siren).await {
        Ok(e) => e,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    };

    let etab = etabs.iter().find(|e| e.siret.trim() == siret);
    match etab {
        Some(e) => (StatusCode::OK, Json(serde_json::json!({
            "siret": e.siret.trim(),
            "siren": siren,
            "nic": &siret[9..],
            "raisonSociale": e.nom,
            "adresse": e.adresse_1,
            "codePostal": e.code_postal,
            "ville": e.localite,
        }))).into_response(),
        None => (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": format!("SIRET {} non trouvé", siret)}))).into_response(),
    }
}

/// POST /v1/siret/search
async fn handle_ds_search_siret(
    State(state): State<Arc<AppState>>,
    Json(params): Json<serde_json::Value>,
) -> impl IntoResponse {
    let store = require_annuaire!(state);
    let siren = params.get("siren").and_then(|v| v.as_str()).unwrap_or("");

    if siren.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": "Le champ 'siren' est requis"}))).into_response();
    }

    match store.lookup_etablissements(siren).await {
        Ok(etabs) => {
            let total = etabs.len();
            let items: Vec<_> = etabs.iter().map(|e| serde_json::json!({
                "siret": e.siret.trim(),
                "siren": siren,
                "raisonSociale": e.nom,
                "adresse": e.adresse_1,
                "codePostal": e.code_postal,
                "ville": e.localite,
            })).collect();
            (StatusCode::OK, Json(serde_json::json!({"items": items, "total": total}))).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

/// POST /v1/routing-code/search
async fn handle_ds_search_routing(
    State(state): State<Arc<AppState>>,
    Json(params): Json<serde_json::Value>,
) -> impl IntoResponse {
    let store = require_annuaire!(state);
    let siren = params.get("siren").and_then(|v| v.as_str()).unwrap_or("");
    let siret = params.get("siret").and_then(|v| v.as_str());
    let suffixe = params.get("suffixe").and_then(|v| v.as_str());

    if siren.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": "Le champ 'siren' est requis"}))).into_response();
    }

    let today = chrono::Local::now().format("%Y%m%d").to_string();
    match store.resolve_routing(siren, siret, None, suffixe, &today).await {
        Ok(Some(r)) => {
            (StatusCode::OK, Json(serde_json::json!({
                "items": [{
                    "siren": siren,
                    "siret": siret,
                    "idPdp": r.matricule_plateforme,
                    "nomPdp": r.nom_plateforme,
                    "codeRoutage": serde_json::Value::Null,
                }],
                "total": 1
            }))).into_response()
        }
        Ok(None) => (StatusCode::OK, Json(serde_json::json!({"items": [], "total": 0}))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

/// GET /v1/directory-line/code:{addressing_id}
async fn handle_ds_get_directory_line(
    State(state): State<Arc<AppState>>,
    Path(addressing_id): Path<String>,
) -> impl IntoResponse {
    let store = require_annuaire!(state);

    // L'addressing_id dans le fichier PPF est le SIREN
    let siren = &addressing_id;
    let today = chrono::Local::now().format("%Y%m%d").to_string();

    match store.resolve_routing(siren, None, None, None, &today).await {
        Ok(Some(r)) => {
            (StatusCode::OK, Json(serde_json::json!({
                "id": addressing_id,
                "siren": siren,
                "idPdp": r.matricule_plateforme,
                "nomPdp": r.nom_plateforme,
                "statut": "ACTIF",
            }))).into_response()
        }
        Ok(None) => (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": format!("Ligne d'annuaire {} non trouvée", addressing_id)}))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

/// POST /v1/directory-line/search
async fn handle_ds_search_directory_lines(
    State(state): State<Arc<AppState>>,
    Json(params): Json<serde_json::Value>,
) -> impl IntoResponse {
    let store = require_annuaire!(state);
    let siren = params.get("siren").and_then(|v| v.as_str()).unwrap_or("");

    if siren.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": "Le champ 'siren' est requis"}))).into_response();
    }

    let today = chrono::Local::now().format("%Y%m%d").to_string();
    match store.resolve_routing(siren, None, None, None, &today).await {
        Ok(Some(r)) => {
            (StatusCode::OK, Json(serde_json::json!({
                "items": [{
                    "id": siren,
                    "siren": siren,
                    "idPdp": r.matricule_plateforme,
                    "nomPdp": r.nom_plateforme,
                    "statut": "ACTIF",
                }],
                "total": 1
            }))).into_response()
        }
        Ok(None) => (StatusCode::OK, Json(serde_json::json!({"items": [], "total": 0}))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

// ============================================================
// Handlers internes — stats et plateformes
// ============================================================

async fn handle_annuaire_stats(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let store = require_annuaire!(state);

    match store.count_all().await {
        Ok(stats) => {
            let last_sync = store.last_sync_horodate().await.unwrap_or(None);
            (StatusCode::OK, Json(serde_json::json!({
                "unitesLegales": stats.unites_legales,
                "etablissements": stats.etablissements,
                "codesRoutage": stats.codes_routage,
                "plateformes": stats.plateformes,
                "lignesAnnuaire": stats.lignes_annuaire,
                "derniereSynchro": last_sync,
            }))).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

async fn handle_annuaire_plateformes(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let store = require_annuaire!(state);

    match store.list_plateformes().await {
        Ok(pfs) => (StatusCode::OK, Json(serde_json::json!({"plateformes": pfs, "total": pfs.len()}))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

/// Configuration TLS optionnelle pour le serveur HTTP
#[derive(Debug, Clone)]
pub struct TlsConfig {
    /// Chemin vers le certificat PEM
    pub cert_path: String,
    /// Chemin vers la clé privée PEM
    pub key_path: String,
}

/// Démarre le serveur HTTP avec arrêt gracieux (drain des requêtes en cours)
pub async fn start_server(
    config: ServerConfig,
    state: Arc<AppState>,
) -> Result<(), Box<dyn std::error::Error>> {
    let app = build_api_router(state);

    let addr = format!("{}:{}", config.host, config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    tracing::info!(
        address = %addr,
        "Serveur HTTP API AFNOR démarré"
    );

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    tracing::info!("Serveur HTTP arrêté proprement");
    Ok(())
}

/// Signal d'arrêt gracieux : attend Ctrl+C ou SIGTERM avec un délai de drain de 30s
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Impossible d'installer le handler Ctrl+C");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Impossible d'installer le handler SIGTERM")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("Signal Ctrl+C reçu, arrêt gracieux (drain 30s)...");
        }
        _ = terminate => {
            tracing::info!("Signal SIGTERM reçu, arrêt gracieux (drain 30s)...");
        }
    }

    // Laisser un délai de drain pour les requêtes en cours
    // (axum gère le drain automatiquement via with_graceful_shutdown,
    // ce timeout sert de borne maximale)
    tokio::time::sleep(std::time::Duration::from_secs(30)).await;
}

/// Démarre le serveur HTTP avec TLS (nécessite axum-server + rustls).
///
/// TODO: Ajouter les dépendances axum-server et rustls pour le support TLS complet.
/// Pour l'instant, cette fonction est un placeholder qui documente l'API cible.
pub async fn start_server_tls(
    _config: ServerConfig,
    _state: Arc<AppState>,
    _tls: TlsConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    // TODO: Implémenter avec axum-server + rustls :
    //
    // let rustls_config = RustlsConfig::from_pem_file(&tls.cert_path, &tls.key_path).await?;
    // let addr = SocketAddr::from(([0,0,0,0], config.port));
    // axum_server::bind_rustls(addr, rustls_config)
    //     .serve(app.into_make_service())
    //     .with_graceful_shutdown(shutdown_signal())
    //     .await?;

    Err("Support TLS non encore implémenté — ajouter axum-server et rustls".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use http_body_util::BodyExt;
    use tower::ServiceExt; // for `oneshot`

    /// Helper: create an AppState for tests (with webhook secret, no bearer auth)
    fn test_app_state() -> Arc<AppState> {
        let (tx, _rx) = tokio::sync::mpsc::channel(10);
        Arc::new(AppState {
            pdp_name: "PDP Test".to_string(),
            pdp_matricule: "9999".to_string(),
            flow_sender: tx,
            webhook_secret: Some("test-secret".to_string()),
            trace_store: None,
            bearer_tokens: None,
            metrics: Metrics::default(),
        })
    }

    /// Helper: create an AppState without webhook secret
    fn test_app_state_no_secret() -> Arc<AppState> {
        let (tx, _rx) = tokio::sync::mpsc::channel(10);
        Arc::new(AppState {
            pdp_name: "PDP Test".to_string(),
            pdp_matricule: "9999".to_string(),
            flow_sender: tx,
            webhook_secret: None,
            trace_store: None,
            bearer_tokens: None,
            metrics: Metrics::default(),
        })
    }

    /// Helper: create an AppState and also return the receiver for verifying sent flows
    fn test_app_state_with_rx() -> (Arc<AppState>, tokio::sync::mpsc::Receiver<InboundFlow>) {
        let (tx, rx) = tokio::sync::mpsc::channel(10);
        let state = Arc::new(AppState {
            pdp_name: "PDP Test".to_string(),
            pdp_matricule: "9999".to_string(),
            flow_sender: tx,
            webhook_secret: Some("test-secret".to_string()),
            trace_store: None,
            bearer_tokens: None,
            metrics: Metrics::default(),
        });
        (state, rx)
    }

    /// Helper: create an AppState with bearer token auth enabled
    fn test_app_state_with_auth() -> Arc<AppState> {
        let (tx, _rx) = tokio::sync::mpsc::channel(10);
        Arc::new(AppState {
            pdp_name: "PDP Test".to_string(),
            pdp_matricule: "9999".to_string(),
            flow_sender: tx,
            webhook_secret: None,
            trace_store: None,
            bearer_tokens: Some(vec!["valid-token-123".to_string(), "valid-token-456".to_string()]),
            metrics: Metrics::default(),
        })
    }

    /// Helper: build a multipart body with both flowInfo and file parts
    fn build_multipart_body(
        flow_info_json: &str,
        file_content: &[u8],
        file_name: &str,
    ) -> (String, Vec<u8>) {
        let boundary = "----TestBoundary7MA4YWxkTrZu0gW";
        let mut body = Vec::new();

        // flowInfo part
        body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
        body.extend_from_slice(
            b"Content-Disposition: form-data; name=\"flowInfo\"\r\n\
              Content-Type: application/json\r\n\r\n",
        );
        body.extend_from_slice(flow_info_json.as_bytes());
        body.extend_from_slice(b"\r\n");

        // file part
        body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
        body.extend_from_slice(
            format!(
                "Content-Disposition: form-data; name=\"file\"; filename=\"{}\"\r\n\
                 Content-Type: application/octet-stream\r\n\r\n",
                file_name
            )
            .as_bytes(),
        );
        body.extend_from_slice(file_content);
        body.extend_from_slice(b"\r\n");

        // closing boundary
        body.extend_from_slice(format!("--{}--\r\n", boundary).as_bytes());

        let content_type = format!("multipart/form-data; boundary={}", boundary);
        (content_type, body)
    }

    /// Helper: build a multipart body with only the file part (no flowInfo)
    fn build_multipart_body_file_only(file_content: &[u8], file_name: &str) -> (String, Vec<u8>) {
        let boundary = "----TestBoundary7MA4YWxkTrZu0gW";
        let mut body = Vec::new();

        body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
        body.extend_from_slice(
            format!(
                "Content-Disposition: form-data; name=\"file\"; filename=\"{}\"\r\n\
                 Content-Type: application/octet-stream\r\n\r\n",
                file_name
            )
            .as_bytes(),
        );
        body.extend_from_slice(file_content);
        body.extend_from_slice(b"\r\n");
        body.extend_from_slice(format!("--{}--\r\n", boundary).as_bytes());

        let content_type = format!("multipart/form-data; boundary={}", boundary);
        (content_type, body)
    }

    /// Helper: build a multipart body with only flowInfo (no file)
    fn build_multipart_body_flow_info_only(flow_info_json: &str) -> (String, Vec<u8>) {
        let boundary = "----TestBoundary7MA4YWxkTrZu0gW";
        let mut body = Vec::new();

        body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
        body.extend_from_slice(
            b"Content-Disposition: form-data; name=\"flowInfo\"\r\n\
              Content-Type: application/json\r\n\r\n",
        );
        body.extend_from_slice(flow_info_json.as_bytes());
        body.extend_from_slice(b"\r\n");
        body.extend_from_slice(format!("--{}--\r\n", boundary).as_bytes());

        let content_type = format!("multipart/form-data; boundary={}", boundary);
        (content_type, body)
    }

    /// Helper: compute HMAC-SHA256 signature for a body
    fn compute_hmac_signature(secret: &str, body: &[u8]) -> String {
        use hmac::{Hmac, Mac};
        use sha2::Sha256;
        type HmacSha256 = Hmac<Sha256>;

        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(body);
        let result = mac.finalize();
        format!("sha256={}", hex::encode(result.into_bytes()))
    }

    /// Helper: build a minimal valid flowInfo JSON string
    fn valid_flow_info_json() -> String {
        serde_json::json!({
            "trackingId": "TRACK-001",
            "name": "test-facture.xml"
        })
        .to_string()
    }

    /// Helper: build a flowInfo JSON with a specific sha256
    fn flow_info_json_with_sha256(sha256: &str) -> String {
        serde_json::json!({
            "trackingId": "TRACK-001",
            "name": "test-facture.xml",
            "sha256": sha256
        })
        .to_string()
    }

    // ---------------------------------------------------------------
    // Existing unit tests
    // ---------------------------------------------------------------

    #[test]
    fn test_hmac_signature_verification() {
        use hmac::{Hmac, Mac};
        use sha2::Sha256;

        type HmacSha256 = Hmac<Sha256>;

        let secret = b"my-secret-key";
        let body = b"test body content";

        // Générer une signature valide
        let mut mac = HmacSha256::new_from_slice(secret).unwrap();
        mac.update(body);
        let result = mac.finalize();
        let hex_sig = hex::encode(result.into_bytes());

        // Vérifier avec préfixe
        assert!(verify_hmac_signature(
            secret,
            body,
            &format!("sha256={}", hex_sig)
        ));

        // Vérifier sans préfixe
        assert!(verify_hmac_signature(secret, body, &hex_sig));

        // Signature invalide
        assert!(!verify_hmac_signature(secret, body, "sha256=00112233"));
    }

    #[test]
    fn test_healthcheck_response() {
        let resp = HealthCheckResponse {
            status: "UP".to_string(),
            pdp_name: "PDP Test".to_string(),
            version: "0.1.0".to_string(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("UP"));
        assert!(json.contains("PDP Test"));
    }

    // ---------------------------------------------------------------
    // Integration tests using axum oneshot
    // ---------------------------------------------------------------

    #[tokio::test]
    async fn test_healthcheck_endpoint() {
        let state = test_app_state();
        let app = build_api_router(state);

        let req = Request::builder()
            .uri("/v1/healthcheck")
            .method("GET")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let body_bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

        assert_eq!(json["pdpName"], "PDP Test");
        assert_eq!(json["status"], "UP");
        assert!(json["version"].is_string());
    }

    #[tokio::test]
    async fn test_receive_flow_valid() {
        let (state, mut rx) = test_app_state_with_rx();
        let app = build_api_router(state);

        let file_content = b"<Invoice>test</Invoice>";
        let flow_info = valid_flow_info_json();
        let (content_type, body) = build_multipart_body(&flow_info, file_content, "facture.xml");

        let req = Request::builder()
            .uri("/v1/flows")
            .method("POST")
            .header("Content-Type", &content_type)
            .body(Body::from(body))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::ACCEPTED);

        let body_bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

        assert_eq!(json["status"], "RECEIVED");
        assert!(json["flowId"].is_string());
        assert!(!json["flowId"].as_str().unwrap().is_empty());

        // Verify the flow was sent through the channel
        let inbound = rx.try_recv().unwrap();
        assert_eq!(inbound.flow_info.tracking_id, "TRACK-001");
        assert_eq!(inbound.filename, "facture.xml");
        assert_eq!(inbound.content, file_content);
    }

    #[tokio::test]
    async fn test_receive_flow_missing_flow_info() {
        let state = test_app_state();
        let app = build_api_router(state);

        let file_content = b"<Invoice>test</Invoice>";
        let (content_type, body) = build_multipart_body_file_only(file_content, "facture.xml");

        let req = Request::builder()
            .uri("/v1/flows")
            .method("POST")
            .header("Content-Type", &content_type)
            .body(Body::from(body))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

        let body_bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

        assert_eq!(json["error"], "MISSING_FLOW_INFO");
    }

    #[tokio::test]
    async fn test_receive_flow_missing_file() {
        let state = test_app_state();
        let app = build_api_router(state);

        let flow_info = valid_flow_info_json();
        let (content_type, body) = build_multipart_body_flow_info_only(&flow_info);

        let req = Request::builder()
            .uri("/v1/flows")
            .method("POST")
            .header("Content-Type", &content_type)
            .body(Body::from(body))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

        let body_bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

        assert_eq!(json["error"], "MISSING_FILE");
    }

    #[tokio::test]
    async fn test_receive_flow_invalid_sha256() {
        let state = test_app_state();
        let app = build_api_router(state);

        let file_content = b"<Invoice>test</Invoice>";
        let wrong_sha = "0000000000000000000000000000000000000000000000000000000000000000";
        let flow_info = flow_info_json_with_sha256(wrong_sha);
        let (content_type, body) = build_multipart_body(&flow_info, file_content, "facture.xml");

        let req = Request::builder()
            .uri("/v1/flows")
            .method("POST")
            .header("Content-Type", &content_type)
            .body(Body::from(body))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

        let body_bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

        assert_eq!(json["error"], "SHA256_MISMATCH");
    }

    #[tokio::test]
    async fn test_receive_flow_valid_sha256() {
        use sha2::{Digest, Sha256};

        let (state, mut rx) = test_app_state_with_rx();
        let app = build_api_router(state);

        let file_content = b"<Invoice>test</Invoice>";
        let correct_sha = format!("{:x}", Sha256::digest(file_content));
        let flow_info = flow_info_json_with_sha256(&correct_sha);
        let (content_type, body) = build_multipart_body(&flow_info, file_content, "facture.xml");

        let req = Request::builder()
            .uri("/v1/flows")
            .method("POST")
            .header("Content-Type", &content_type)
            .body(Body::from(body))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::ACCEPTED);

        let inbound = rx.try_recv().unwrap();
        assert_eq!(inbound.content, file_content);
    }

    #[tokio::test]
    async fn test_webhook_valid_signature() {
        let state = test_app_state();
        let app = build_api_router(state);

        let webhook_body = serde_json::json!({
            "eventType": "flow.received",
            "flowId": "test-flow-123",
            "trackingId": "TRACK-001"
        });
        let body_bytes = serde_json::to_vec(&webhook_body).unwrap();
        let signature = compute_hmac_signature("test-secret", &body_bytes);

        let req = Request::builder()
            .uri("/v1/webhooks/callback")
            .method("POST")
            .header("Content-Type", "application/json")
            .header("X-Webhook-Signature", &signature)
            .body(Body::from(body_bytes))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let resp_bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&resp_bytes).unwrap();
        assert_eq!(json["status"], "ok");
    }

    #[tokio::test]
    async fn test_webhook_invalid_signature() {
        let state = test_app_state();
        let app = build_api_router(state);

        let webhook_body = serde_json::json!({
            "eventType": "flow.received",
            "flowId": "test-flow-123"
        });
        let body_bytes = serde_json::to_vec(&webhook_body).unwrap();

        let req = Request::builder()
            .uri("/v1/webhooks/callback")
            .method("POST")
            .header("Content-Type", "application/json")
            .header("X-Webhook-Signature", "sha256=badbadbadbadbadbadbadbadbadbadbadbadbadbadbadbadbadbadbadbadbadb")
            .body(Body::from(body_bytes))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

        let resp_bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&resp_bytes).unwrap();
        assert_eq!(json["error"], "INVALID_SIGNATURE");
    }

    #[tokio::test]
    async fn test_webhook_missing_signature() {
        let state = test_app_state(); // webhook_secret = Some("test-secret")
        let app = build_api_router(state);

        let webhook_body = serde_json::json!({
            "eventType": "flow.received",
            "flowId": "test-flow-123"
        });
        let body_bytes = serde_json::to_vec(&webhook_body).unwrap();

        let req = Request::builder()
            .uri("/v1/webhooks/callback")
            .method("POST")
            .header("Content-Type", "application/json")
            .body(Body::from(body_bytes))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

        let resp_bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&resp_bytes).unwrap();
        assert_eq!(json["error"], "MISSING_SIGNATURE");
    }

    #[tokio::test]
    async fn test_webhook_no_secret_configured() {
        let state = test_app_state_no_secret(); // webhook_secret = None
        let app = build_api_router(state);

        let webhook_body = serde_json::json!({
            "eventType": "flow.received",
            "flowId": "test-flow-123"
        });
        let body_bytes = serde_json::to_vec(&webhook_body).unwrap();

        // No signature header — should still succeed since no secret is configured
        let req = Request::builder()
            .uri("/v1/webhooks/callback")
            .method("POST")
            .header("Content-Type", "application/json")
            .body(Body::from(body_bytes))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let resp_bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&resp_bytes).unwrap();
        assert_eq!(json["status"], "ok");
    }

    // ---------------------------------------------------------------
    // Tests d'authentification Bearer token
    // ---------------------------------------------------------------

    #[tokio::test]
    async fn test_auth_valid_bearer_token() {
        let state = test_app_state_with_auth();
        let app = build_api_router(state);

        let req = Request::builder()
            .uri("/v1/healthcheck")
            .method("GET")
            .body(Body::empty())
            .unwrap();

        // Le healthcheck ne doit PAS exiger d'auth
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_auth_flows_requires_token() {
        let state = test_app_state_with_auth();
        let app = build_api_router(state);

        let file_content = b"<Invoice>test</Invoice>";
        let flow_info = valid_flow_info_json();
        let (content_type, body) = build_multipart_body(&flow_info, file_content, "facture.xml");

        // Requête SANS token
        let req = Request::builder()
            .uri("/v1/flows")
            .method("POST")
            .header("Content-Type", &content_type)
            .body(Body::from(body))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

        let body_bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
        assert_eq!(json["error"], "MISSING_TOKEN");
    }

    #[tokio::test]
    async fn test_auth_flows_accepts_valid_token() {
        let (state, mut rx) = {
            let (tx, rx) = tokio::sync::mpsc::channel(10);
            let state = Arc::new(AppState {
                pdp_name: "PDP Test".to_string(),
                pdp_matricule: "9999".to_string(),
                flow_sender: tx,
                webhook_secret: None,
                trace_store: None,
                bearer_tokens: Some(vec!["my-secret-token".to_string()]),
                metrics: Metrics::default(),
            });
            (state, rx)
        };
        let app = build_api_router(state);

        let file_content = b"<Invoice>test</Invoice>";
        let flow_info = valid_flow_info_json();
        let (content_type, body) = build_multipart_body(&flow_info, file_content, "facture.xml");

        let req = Request::builder()
            .uri("/v1/flows")
            .method("POST")
            .header("Content-Type", &content_type)
            .header("Authorization", "Bearer my-secret-token")
            .body(Body::from(body))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::ACCEPTED);

        let inbound = rx.try_recv().unwrap();
        assert_eq!(inbound.flow_info.tracking_id, "TRACK-001");
    }

    #[tokio::test]
    async fn test_auth_flows_rejects_invalid_token() {
        let state = test_app_state_with_auth();
        let app = build_api_router(state);

        let file_content = b"<Invoice>test</Invoice>";
        let flow_info = valid_flow_info_json();
        let (content_type, body) = build_multipart_body(&flow_info, file_content, "facture.xml");

        let req = Request::builder()
            .uri("/v1/flows")
            .method("POST")
            .header("Content-Type", &content_type)
            .header("Authorization", "Bearer wrong-token")
            .body(Body::from(body))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

        let body_bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
        assert_eq!(json["error"], "INVALID_TOKEN");
    }

    #[tokio::test]
    async fn test_auth_no_tokens_configured_allows_all() {
        // bearer_tokens = None → pas d'auth requise, le flux doit passer
        let (tx, _rx) = tokio::sync::mpsc::channel(10);
        let state = Arc::new(AppState {
            pdp_name: "PDP Test".to_string(),
            pdp_matricule: "9999".to_string(),
            flow_sender: tx,
            webhook_secret: None,
            bearer_tokens: None,
            trace_store: None,
            metrics: Metrics::default(),
        });
        let app = build_api_router(state);

        let file_content = b"<Invoice>test</Invoice>";
        let flow_info = valid_flow_info_json();
        let (content_type, body) = build_multipart_body(&flow_info, file_content, "facture.xml");

        let req = Request::builder()
            .uri("/v1/flows")
            .method("POST")
            .header("Content-Type", &content_type)
            .body(Body::from(body))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::ACCEPTED);
    }

    // ---------------------------------------------------------------
    // Tests des endpoints admin (GET /v1/flows/{id}, GET /v1/stats, GET /v1/flows?status=error)
    // ---------------------------------------------------------------

    #[tokio::test]
    async fn test_get_flow_no_trace_store() {
        // Sans TraceStore configuré, doit retourner 501
        let state = test_app_state();
        let app = build_api_router(state);

        let req = Request::builder()
            .uri("/v1/flows/some-flow-id")
            .method("GET")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_IMPLEMENTED);

        let body_bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
        assert_eq!(json["error"], "NOT_IMPLEMENTED");
    }

    #[tokio::test]
    async fn test_get_flow_requires_auth() {
        // Avec auth activée, GET /v1/flows/{id} doit exiger un token
        let state = test_app_state_with_auth();
        let app = build_api_router(state);

        let req = Request::builder()
            .uri("/v1/flows/some-flow-id")
            .method("GET")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_stats_no_trace_store() {
        // Sans TraceStore, doit retourner 501
        let state = test_app_state();
        let app = build_api_router(state);

        let req = Request::builder()
            .uri("/v1/stats")
            .method("GET")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_IMPLEMENTED);
    }

    #[tokio::test]
    async fn test_stats_requires_auth() {
        let state = test_app_state_with_auth();
        let app = build_api_router(state);

        let req = Request::builder()
            .uri("/v1/stats")
            .method("GET")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_list_flows_no_trace_store() {
        let state = test_app_state();
        let app = build_api_router(state);

        let req = Request::builder()
            .uri("/v1/flows?status=error")
            .method("GET")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_IMPLEMENTED);
    }

    #[tokio::test]
    async fn test_list_flows_missing_status() {
        // Sans paramètre status, doit retourner 400
        let state = test_app_state();
        let app = build_api_router(state);

        let req = Request::builder()
            .uri("/v1/flows")
            .method("GET")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

        let body_bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
        assert_eq!(json["error"], "MISSING_FILTER");
    }

    #[tokio::test]
    async fn test_list_flows_invalid_status() {
        let state = test_app_state();
        let app = build_api_router(state);

        let req = Request::builder()
            .uri("/v1/flows?status=unknown")
            .method("GET")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

        let body_bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
        assert_eq!(json["error"], "INVALID_FILTER");
    }

    #[tokio::test]
    async fn test_list_flows_requires_auth() {
        let state = test_app_state_with_auth();
        let app = build_api_router(state);

        let req = Request::builder()
            .uri("/v1/flows?status=error")
            .method("GET")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    // ---------------------------------------------------------------
    // Tests des métriques Prometheus
    // ---------------------------------------------------------------

    #[tokio::test]
    async fn test_metrics_endpoint() {
        let state = test_app_state();
        let app = build_api_router(state);

        let req = Request::builder()
            .uri("/metrics")
            .method("GET")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let body_bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();

        // Vérifier le format Prometheus
        assert!(body_str.contains("# HELP pdp_flows_received_total"));
        assert!(body_str.contains("# TYPE pdp_flows_received_total counter"));
        assert!(body_str.contains("pdp_flows_received_total 0"));
        assert!(body_str.contains("pdp_flows_accepted_total 0"));
        assert!(body_str.contains("pdp_flows_rejected_total 0"));
        assert!(body_str.contains("pdp_webhooks_received_total 0"));
    }

    #[tokio::test]
    async fn test_metrics_no_auth_required() {
        // /metrics ne doit pas exiger d'authentification
        let state = test_app_state_with_auth();
        let app = build_api_router(state);

        let req = Request::builder()
            .uri("/metrics")
            .method("GET")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_metrics_increment_on_flow() {
        // Vérifier que les compteurs s'incrémentent lors de la réception d'un flux
        let (state, _rx) = test_app_state_with_rx();
        let state_ref = state.clone();
        let app = build_api_router(state);

        let file_content = b"<Invoice>test</Invoice>";
        let flow_info = valid_flow_info_json();
        let (content_type, body) = build_multipart_body(&flow_info, file_content, "facture.xml");

        let req = Request::builder()
            .uri("/v1/flows")
            .method("POST")
            .header("Content-Type", &content_type)
            .body(Body::from(body))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::ACCEPTED);

        assert_eq!(state_ref.metrics.flows_received.load(Ordering::Relaxed), 1);
        assert_eq!(state_ref.metrics.flows_accepted.load(Ordering::Relaxed), 1);
        assert_eq!(state_ref.metrics.flows_rejected.load(Ordering::Relaxed), 0);
    }

    // ---------------------------------------------------------------
    // Tests end-to-end : flux complet avec vraies factures
    // ---------------------------------------------------------------

    fn load_fixture(name: &str) -> Vec<u8> {
        let path = format!(
            "{}/../../tests/fixtures/{}",
            env!("CARGO_MANIFEST_DIR"),
            name
        );
        std::fs::read(&path).unwrap_or_else(|_| panic!("Fixture {} introuvable", path))
    }

    #[tokio::test]
    async fn test_e2e_submit_real_cii_invoice() {
        let (state, mut rx) = test_app_state_with_rx();
        let app = build_api_router(state);

        let cii_xml = load_fixture("cii/facture_cii_001.xml");
        let flow_info = serde_json::json!({
            "trackingId": "E2E-CII-001",
            "name": "facture_cii_001.xml"
        }).to_string();
        let (content_type, body) = build_multipart_body(&flow_info, &cii_xml, "facture_cii_001.xml");

        let req = Request::builder()
            .uri("/v1/flows")
            .method("POST")
            .header("Content-Type", &content_type)
            .body(Body::from(body))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::ACCEPTED);

        let resp_bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&resp_bytes).unwrap();
        assert_eq!(json["status"], "RECEIVED");

        // Vérifier le flux reçu via le channel
        let inbound = rx.try_recv().unwrap();
        assert_eq!(inbound.flow_info.tracking_id, "E2E-CII-001");
        assert_eq!(inbound.filename, "facture_cii_001.xml");
        assert_eq!(inbound.content.len(), cii_xml.len());
        // Vérifier que c'est du XML CII valide
        assert!(String::from_utf8_lossy(&inbound.content).contains("CrossIndustryInvoice"));
    }

    #[tokio::test]
    async fn test_e2e_submit_real_ubl_invoice() {
        let (state, mut rx) = test_app_state_with_rx();
        let app = build_api_router(state);

        let ubl_xml = load_fixture("ubl/facture_ubl_001.xml");
        let flow_info = serde_json::json!({
            "trackingId": "E2E-UBL-001",
            "name": "facture_ubl_001.xml"
        }).to_string();
        let (content_type, body) = build_multipart_body(&flow_info, &ubl_xml, "facture_ubl_001.xml");

        let req = Request::builder()
            .uri("/v1/flows")
            .method("POST")
            .header("Content-Type", &content_type)
            .body(Body::from(body))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::ACCEPTED);

        let inbound = rx.try_recv().unwrap();
        assert_eq!(inbound.flow_info.tracking_id, "E2E-UBL-001");
        assert!(String::from_utf8_lossy(&inbound.content).contains("Invoice"));
    }

    #[tokio::test]
    async fn test_e2e_submit_real_cii_with_sha256() {
        use sha2::{Digest, Sha256};

        let (state, mut rx) = test_app_state_with_rx();
        let app = build_api_router(state);

        let cii_xml = load_fixture("cii/facture_cii_001.xml");
        let sha = format!("{:x}", Sha256::digest(&cii_xml));
        let flow_info = serde_json::json!({
            "trackingId": "E2E-SHA-001",
            "name": "facture_cii_001.xml",
            "sha256": sha
        }).to_string();
        let (content_type, body) = build_multipart_body(&flow_info, &cii_xml, "facture_cii_001.xml");

        let req = Request::builder()
            .uri("/v1/flows")
            .method("POST")
            .header("Content-Type", &content_type)
            .body(Body::from(body))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::ACCEPTED);

        let inbound = rx.try_recv().unwrap();
        assert_eq!(inbound.flow_info.tracking_id, "E2E-SHA-001");
    }

    #[tokio::test]
    async fn test_e2e_submit_with_auth_flow() {
        let (tx, mut rx) = tokio::sync::mpsc::channel(10);
        let state = Arc::new(AppState {
            pdp_name: "PDP E2E".to_string(),
            pdp_matricule: "0042".to_string(),
            flow_sender: tx,
            webhook_secret: Some("e2e-secret".to_string()),
            trace_store: None,
            bearer_tokens: Some(vec!["e2e-token-valid".to_string()]),
            metrics: Metrics::default(),
        });
        let app = build_api_router(state);

        let cii_xml = load_fixture("cii/facture_cii_001.xml");
        let flow_info = serde_json::json!({
            "trackingId": "E2E-AUTH-001",
            "name": "facture_auth.xml"
        }).to_string();
        let (content_type, body) = build_multipart_body(&flow_info, &cii_xml, "facture_auth.xml");

        // Sans token → 401
        let req = Request::builder()
            .uri("/v1/flows")
            .method("POST")
            .header("Content-Type", &content_type)
            .body(Body::from(body.clone()))
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

        // Mauvais token → 401
        let req = Request::builder()
            .uri("/v1/flows")
            .method("POST")
            .header("Content-Type", &content_type)
            .header("Authorization", "Bearer wrong-token")
            .body(Body::from(body.clone()))
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

        // Bon token → 202
        let req = Request::builder()
            .uri("/v1/flows")
            .method("POST")
            .header("Content-Type", &content_type)
            .header("Authorization", "Bearer e2e-token-valid")
            .body(Body::from(body))
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::ACCEPTED);

        let inbound = rx.try_recv().unwrap();
        assert_eq!(inbound.flow_info.tracking_id, "E2E-AUTH-001");

        // Healthcheck toujours accessible sans token
        let req = Request::builder()
            .uri("/v1/healthcheck")
            .method("GET")
            .body(Body::empty())
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        // Métriques toujours accessibles sans token
        let req = Request::builder()
            .uri("/metrics")
            .method("GET")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_e2e_batch_submit_multiple_invoices() {
        let (state, mut rx) = test_app_state_with_rx();

        let fixtures = vec![
            ("cii/facture_cii_001.xml", "BATCH-CII-001"),
            ("ubl/facture_ubl_001.xml", "BATCH-UBL-001"),
        ];

        for (fixture, tracking_id) in &fixtures {
            let app = build_api_router(state.clone());
            let content = load_fixture(fixture);
            let filename = fixture.rsplit('/').next().unwrap();
            let flow_info = serde_json::json!({
                "trackingId": tracking_id,
                "name": filename
            }).to_string();
            let (content_type, body) = build_multipart_body(&flow_info, &content, filename);

            let req = Request::builder()
                .uri("/v1/flows")
                .method("POST")
                .header("Content-Type", &content_type)
                .body(Body::from(body))
                .unwrap();

            let resp = app.oneshot(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::ACCEPTED,
                "Flux {} rejeté", fixture);
        }

        // Vérifier que tous les flux sont passés par le channel
        for (_, tracking_id) in &fixtures {
            let inbound = rx.try_recv()
                .unwrap_or_else(|_| panic!("Flux {} non reçu dans le channel", tracking_id));
            assert_eq!(inbound.flow_info.tracking_id, *tracking_id);
        }
    }

    #[tokio::test]
    async fn test_e2e_metrics_after_batch() {
        let (state, _rx) = test_app_state_with_rx();
        let state_ref = state.clone();

        // Soumettre 3 flux
        for i in 0..3 {
            let app = build_api_router(state.clone());
            let flow_info = serde_json::json!({
                "trackingId": format!("METRIC-{}", i),
                "name": format!("facture_{}.xml", i)
            }).to_string();
            let (ct, body) = build_multipart_body(&flow_info, b"<Invoice/>", &format!("f{}.xml", i));

            let req = Request::builder()
                .uri("/v1/flows").method("POST")
                .header("Content-Type", &ct)
                .body(Body::from(body)).unwrap();

            let _ = app.oneshot(req).await.unwrap();
        }

        // Vérifier les métriques
        assert_eq!(state_ref.metrics.flows_received.load(Ordering::Relaxed), 3);
        assert_eq!(state_ref.metrics.flows_accepted.load(Ordering::Relaxed), 3);

        // Endpoint /metrics reflète les compteurs
        let app = build_api_router(state);
        let req = Request::builder()
            .uri("/metrics").method("GET")
            .body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let text = String::from_utf8(body.to_vec()).unwrap();
        assert!(text.contains("pdp_flows_received_total 3"));
        assert!(text.contains("pdp_flows_accepted_total 3"));
    }

}
