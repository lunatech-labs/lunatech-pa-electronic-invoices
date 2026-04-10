//! Serveur HTTP API conforme AFNOR XP Z12-013
//!
//! Expose les endpoints suivants :
//! - POST /v1/flows — Réception de flux entrants (factures, CDV, e-reporting)
//! - POST /v1/flows/search — Recherche de flux
//! - GET /v1/flows/{flowId} — Consultation d'un flux
//! - POST /v1/webhooks/callback — Réception de notifications webhook
//! - GET /v1/healthcheck — Health check
//!
//! Ce serveur implémente le rôle de "PDP réceptrice" dans l'architecture AFNOR.

use std::sync::Arc;

use axum::{
    Router,
    extract::{Multipart, Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json,
};
use serde::{Deserialize, Serialize};
use tracing;

// ============================================================
// État partagé du serveur
// ============================================================

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
    Router::new()
        // AFNOR Flow Service endpoints
        .route("/v1/flows", post(handle_receive_flow))
        .route("/v1/flows/{flow_id}", get(handle_get_flow))
        // Webhook callback
        .route("/v1/webhooks/callback", post(handle_webhook_callback))
        // Health check
        .route("/v1/healthcheck", get(handle_healthcheck))
        // État partagé
        .with_state(state)
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

/// GET /v1/flows/{flowId} — Consultation d'un flux
async fn handle_get_flow(
    State(state): State<Arc<AppState>>,
    Path(flow_id): Path<String>,
) -> impl IntoResponse {
    // Pour l'instant, retourner un 501 Not Implemented
    // TODO: implémenter la consultation via le TraceStore
    tracing::debug!(flow_id = %flow_id, "Consultation flux (non implémenté)");

    (
        StatusCode::NOT_IMPLEMENTED,
        Json(ErrorResponse {
            error: "NOT_IMPLEMENTED".to_string(),
            message: "La consultation de flux sera implémentée prochainement".to_string(),
        }),
    )
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

/// Démarre le serveur HTTP en arrière-plan
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

    axum::serve(listener, app).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use http_body_util::BodyExt;
    use tower::ServiceExt; // for `oneshot`

    /// Helper: create an AppState for tests (with webhook secret)
    fn test_app_state() -> Arc<AppState> {
        let (tx, _rx) = tokio::sync::mpsc::channel(10);
        Arc::new(AppState {
            pdp_name: "PDP Test".to_string(),
            pdp_matricule: "9999".to_string(),
            flow_sender: tx,
            webhook_secret: Some("test-secret".to_string()),
            trace_store: None,
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
        });
        (state, rx)
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

}
