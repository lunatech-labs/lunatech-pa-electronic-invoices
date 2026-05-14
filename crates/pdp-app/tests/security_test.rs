//! Tests d'isolation tenant et d'authentification.
//!
//! Vérifient que :
//! 1. Sans token, toute route protégée retourne 401 (ou redirige vers /login pour /ui)
//! 2. Un token `Tenant` ne voit QUE ses SIRENs autorisés (403 sur les autres)
//! 3. Un token `PdpAdmin` peut consulter n'importe quel SIREN
//! 4. `get_exchange` filtre par siren côté backend (un mauvais siren → None)
//!
//! Stratégie : on monte un AppState complet avec un mock InMemoryTraceBackend
//! (cohérent avec ui_test.rs) et on envoie des requêtes via `tower::oneshot`.
//! Pas d'Elasticsearch ni de Postgres.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;

use pdp_app::security::{Role, SecurityContext};
use pdp_app::server::{build_api_router, AppState, Metrics};
use pdp_app::webhooks::WebhookStore;
use pdp_config::model::UserConfig;
use pdp_core::error::PdpResult;
use pdp_trace::store::{
    EventEntry, ExchangeDocument, ExchangeSummary, TraceStats,
};
use pdp_trace::TraceBackend;

// ---------------------------------------------------------------------------
// Mock minimal (copie réduite de ui_test.rs::InMemoryTraceBackend, pour ne
// pas dépendre du fichier voisin — cargo n'autorise pas le partage entre
// fichiers de tests sans `mod common`).
// ---------------------------------------------------------------------------

struct InMemBackend(Vec<ExchangeDocument>);

#[async_trait]
impl TraceBackend for InMemBackend {
    async fn get_stats(&self) -> PdpResult<TraceStats> {
        Ok(TraceStats {
            total_exchanges: self.0.len() as i64,
            total_errors: 0,
            total_distributed: 0,
        })
    }
    async fn get_stats_for_siren(&self, siren: &str) -> PdpResult<TraceStats> {
        let n = self
            .0
            .iter()
            .filter(|d| {
                d.seller_siren.as_deref() == Some(siren)
                    || d.buyer_siren.as_deref() == Some(siren)
            })
            .count() as i64;
        Ok(TraceStats {
            total_exchanges: n,
            total_errors: 0,
            total_distributed: n,
        })
    }
    async fn get_tenant_name(&self, siren: &str) -> Option<String> {
        self.0
            .iter()
            .find(|d| d.seller_siren.as_deref() == Some(siren))
            .and_then(|d| d.seller_name.clone())
    }
    async fn list_exchanges(
        &self,
        siren: &str,
        _status: Option<&str>,
        _from: Option<&str>,
        _to: Option<&str>,
        _page: usize,
        _page_size: usize,
        direction: Option<&str>,
    ) -> PdpResult<Vec<ExchangeSummary>> {
        Ok(self
            .0
            .iter()
            .filter(|d| match direction {
                Some("emises") => d.seller_siren.as_deref() == Some(siren),
                Some("recues") => d.buyer_siren.as_deref() == Some(siren),
                _ => d.seller_siren.as_deref() == Some(siren)
                    || d.buyer_siren.as_deref() == Some(siren),
            })
            .map(to_summary)
            .collect())
    }
    async fn count_exchanges(
        &self,
        siren: &str,
        _status: Option<&str>,
        _from: Option<&str>,
        _to: Option<&str>,
        direction: Option<&str>,
    ) -> PdpResult<i64> {
        Ok(self
            .0
            .iter()
            .filter(|d| match direction {
                Some("emises") => d.seller_siren.as_deref() == Some(siren),
                Some("recues") => d.buyer_siren.as_deref() == Some(siren),
                _ => d.seller_siren.as_deref() == Some(siren)
                    || d.buyer_siren.as_deref() == Some(siren),
            })
            .count() as i64)
    }
    async fn get_exchange(
        &self,
        exchange_id: &str,
        siren: Option<&str>,
    ) -> PdpResult<Option<ExchangeDocument>> {
        let found = self.0.iter().find(|d| {
            d.exchange_id == exchange_id
                && siren
                    .map(|s| {
                        d.seller_siren.as_deref() == Some(s)
                            || d.buyer_siren.as_deref() == Some(s)
                    })
                    .unwrap_or(true)
        });
        Ok(found.map(clone_doc))
    }
    async fn get_error_flows(&self) -> PdpResult<Vec<ExchangeSummary>> {
        Ok(Vec::new())
    }
}

fn to_summary(d: &ExchangeDocument) -> ExchangeSummary {
    ExchangeSummary {
        exchange_id: d.exchange_id.clone(),
        flow_id: d.flow_id.clone(),
        source_filename: d.source_filename.clone(),
        invoice_number: d.invoice_number.clone(),
        seller_name: d.seller_name.clone(),
        buyer_name: d.buyer_name.clone(),
        seller_siret: d.seller_siret.clone(),
        buyer_siret: d.buyer_siret.clone(),
        seller_siren: d.seller_siren.clone(),
        buyer_siren: d.buyer_siren.clone(),
        status: d.status.clone(),
        error_count: d.error_count,
        created_at: d.created_at.clone(),
        attachment_count: d.attachment_count,
        cdv_status_code: d.cdv_status_code,
    }
}

fn clone_doc(d: &ExchangeDocument) -> ExchangeDocument {
    let json = serde_json::to_value(d).unwrap();
    serde_json::from_value(json).unwrap()
}

fn doc_for(seller_siren: &str, buyer_siren: &str, invoice: &str) -> ExchangeDocument {
    ExchangeDocument {
        exchange_id: format!("ex-{}-{}", seller_siren, invoice),
        flow_id: format!("flow-{}-{}", seller_siren, invoice),
        source_filename: Some(format!("{invoice}.xml")),
        invoice_number: Some(invoice.to_string()),
        invoice_key: None,
        seller_name: Some(format!("Vendeur {seller_siren}")),
        buyer_name: Some(format!("Acheteur {buyer_siren}")),
        seller_siret: Some(format!("{seller_siren}00001")),
        buyer_siret: Some(format!("{buyer_siren}00002")),
        seller_siren: Some(seller_siren.to_string()),
        buyer_siren: Some(buyer_siren.to_string()),
        source_format: Some("UBL".into()),
        total_ht: Some(100.0),
        total_ttc: Some(120.0),
        total_tax: Some(20.0),
        currency: Some("EUR".into()),
        issue_date: Some("2026-04-01".into()),
        status: "VALIDÉ".into(),
        error_count: 0,
        cdv_status_code: None,
        generated_cdv_xml: None,
        generated_cdv_status_code: None,
        raw_xml: Some("<Invoice/>".into()),
        raw_pdf_base64: None,
        converted_xml: None,
        converted_format: None,
        attachment_count: 0,
        attachment_filenames: Vec::new(),
        events: vec![EventEntry {
            id: "e".into(),
            route_id: "rt".into(),
            status: "VALIDÉ".into(),
            message: "ok".into(),
            error_detail: None,
            timestamp: "2026-04-01T10:00:00Z".into(),
        }],
        errors: Vec::new(),
        validation_warnings: Vec::new(),
        created_at: "2026-04-01T10:00:00Z".into(),
        updated_at: "2026-04-01T10:00:00Z".into(),
    }
}

// ---------------------------------------------------------------------------
// Helpers : montage d'un AppState avec une table de tokens contrôlée
// ---------------------------------------------------------------------------

struct StateBuilder {
    tokens: HashMap<String, SecurityContext>,
    users: Vec<UserConfig>,
    docs: Vec<ExchangeDocument>,
}

impl StateBuilder {
    fn new() -> Self {
        Self {
            tokens: HashMap::new(),
            users: Vec::new(),
            docs: vec![
                doc_for("123456789", "987654321", "INV-A"),
                doc_for("999999999", "888888888", "INV-B"),
            ],
        }
    }

    fn user(mut self, email: &str, password: &str, principal: &str, sirens: &[&str], role: Role) -> Self {
        self.users.push(UserConfig {
            email: email.into(),
            password: password.into(),
            principal: principal.into(),
            allowed_sirens: sirens.iter().map(|s| s.to_string()).collect(),
            role,
        });
        self
    }

    fn tenant_token(mut self, token: &str, sirens: &[&str]) -> Self {
        self.tokens.insert(
            token.to_string(),
            SecurityContext {
                principal: format!("tenant:{token}"),
                allowed_sirens: sirens.iter().map(|s| s.to_string()).collect(),
                role: Role::Tenant,
            },
        );
        self
    }

    fn admin_token(mut self, token: &str) -> Self {
        self.tokens.insert(
            token.to_string(),
            SecurityContext {
                principal: format!("admin:{token}"),
                allowed_sirens: Vec::new(),
                role: Role::PdpAdmin,
            },
        );
        self
    }

    fn build(self) -> Arc<AppState> {
        let (tx, _rx) = tokio::sync::mpsc::channel(10);
        Arc::new(AppState {
            pdp_name: "Test PDP".into(),
            pdp_matricule: "0001".into(),
            flow_sender: tx,
            webhook_secret: None,
            trace_store: Some(Arc::new(InMemBackend(self.docs))),
            tokens: self.tokens,
            users: self.users,
            session_secret: b"test-session-secret-32-bytes-padding-padding".to_vec(),
            session_ttl_secs: 3600, revocations: std::sync::Arc::new(pdp_app::session::RevocationList::new()),
            metrics: Metrics::default(),
            annuaire_store: None,
            webhook_store: Arc::new(WebhookStore::new()),
            event_bus: None,
            max_flow_size_bytes: 100 * 1024 * 1024,
            request_timeout: std::time::Duration::from_secs(30),
            rate_limiter: None,
            tenants_dir: None,
        })
    }
}

async fn send(state: Arc<AppState>, uri: &str, bearer: Option<&str>) -> (StatusCode, String) {
    let app = build_api_router(state);
    let mut req = Request::builder().uri(uri).method("GET");
    if let Some(t) = bearer {
        req = req.header("Authorization", format!("Bearer {t}"));
    }
    let resp = app.oneshot(req.body(Body::empty()).unwrap()).await.unwrap();
    let status = resp.status();
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    (status, String::from_utf8_lossy(&bytes).to_string())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn no_token_redirects_to_login_on_ui() {
    // Phase B : routes UI sans auth → 303 vers /login (au lieu de 401 brut).
    let state = StateBuilder::new().tenant_token("tok-a", &["123456789"]).build();
    let (status, _) = send(state, "/ui/emises?siren=123456789", None).await;
    assert_eq!(status, StatusCode::SEE_OTHER);
}

#[tokio::test]
async fn unknown_token_redirects_to_login_on_ui() {
    let state = StateBuilder::new().tenant_token("tok-a", &["123456789"]).build();
    let (status, _) = send(state, "/ui/emises?siren=123456789", Some("tok-zzz")).await;
    assert_eq!(status, StatusCode::SEE_OTHER);
}

#[tokio::test]
async fn no_token_rejected_with_401_on_api() {
    // L'API garde 401 JSON (pas de redirect — clients non navigateur).
    let state = StateBuilder::new().tenant_token("tok-a", &["123456789"]).build();
    let (status, _) = send(state, "/v1/stats?siren=123456789", None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn tenant_token_can_access_own_siren() {
    let state = StateBuilder::new().tenant_token("tok-a", &["123456789"]).build();
    let (status, body) =
        send(state, "/ui/emises?siren=123456789", Some("tok-a")).await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.contains("INV-A"), "doit lister INV-A du tenant 123456789");
}

#[tokio::test]
async fn tenant_token_cannot_access_other_siren() {
    // tok-a est lié à 123456789 → demander ?siren=999999999 doit être 403.
    let state = StateBuilder::new().tenant_token("tok-a", &["123456789"]).build();
    let (status, body) =
        send(state, "/ui/emises?siren=999999999", Some("tok-a")).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert!(body.contains("SIREN_NOT_AUTHORIZED"));
}

#[tokio::test]
async fn admin_token_sees_all_sirens() {
    let state = StateBuilder::new()
        .admin_token("tok-admin")
        .tenant_token("tok-a", &["123456789"]) // pour cohabitation
        .build();
    let (status, body) =
        send(state, "/ui/emises?siren=999999999", Some("tok-admin")).await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.contains("INV-B"));
}

#[tokio::test]
async fn v1_stats_requires_siren() {
    // Avec un admin token et sans `?siren=`, l'extractor AuthorizedSiren
    // retourne 400 (le check d'autorisation n'est jamais atteint).
    let state = StateBuilder::new().admin_token("tok-admin").build();
    let (status, body) = send(state, "/v1/stats", Some("tok-admin")).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body.contains("SIREN_REQUIRED"));
}

#[tokio::test]
async fn v1_stats_rejects_cross_tenant() {
    let state = StateBuilder::new().tenant_token("tok-a", &["123456789"]).build();
    let (status, _) = send(state, "/v1/stats?siren=999999999", Some("tok-a")).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn get_exchange_filters_by_siren() {
    // Le doc INV-B appartient à 999999999. Un tenant 123456789 qui devine
    // l'exchange_id ne doit PAS pouvoir le récupérer via /v1/flows/{id}.
    let state = StateBuilder::new().tenant_token("tok-a", &["123456789"]).build();
    let (status, _) = send(
        state,
        "/v1/flows/ex-999999999-INV-B?siren=123456789",
        Some("tok-a"),
    )
    .await;
    // L'exchange_id existe (côté store) mais le filtre tenant le rend invisible
    // pour ce SIREN → 404 NOT_FOUND.
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn get_exchange_visible_to_owner() {
    let state = StateBuilder::new().tenant_token("tok-a", &["123456789"]).build();
    let (status, body) = send(
        state,
        "/v1/flows/ex-123456789-INV-A?siren=123456789",
        Some("tok-a"),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.contains("INV-A"));
}

// ---------------------------------------------------------------------------
// Phase B : login web + session cookie
// ---------------------------------------------------------------------------

async fn post_form(state: Arc<AppState>, uri: &str, body: &str) -> axum::response::Response {
    let app = build_api_router(state);
    let req = Request::builder()
        .uri(uri)
        .method("POST")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(Body::from(body.to_string()))
        .unwrap();
    app.oneshot(req).await.unwrap()
}

#[tokio::test]
async fn login_page_is_public() {
    let state = StateBuilder::new().build();
    let (status, body) = send(state, "/login", None).await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.contains("<form"));
    assert!(body.contains("Se connecter"));
}

#[tokio::test]
async fn login_with_valid_credentials_sets_cookie_and_redirects() {
    let state = StateBuilder::new()
        .user("alice@tc", "pwd123", "alice", &["123456789"], Role::Tenant)
        .build();
    let resp = post_form(state, "/login", "email=alice%40tc&password=pwd123").await;
    assert_eq!(resp.status(), StatusCode::SEE_OTHER);
    let cookie = resp
        .headers()
        .get("set-cookie")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert!(
        cookie.starts_with("ferrite_session="),
        "doit poser le cookie ferrite_session, got {}",
        cookie
    );
    assert!(cookie.contains("HttpOnly"));
    assert!(cookie.contains("SameSite=Lax"));
    let location = resp
        .headers()
        .get("location")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert_eq!(location, "/ui");
}

#[tokio::test]
async fn login_with_wrong_password_returns_401_with_form() {
    let state = StateBuilder::new()
        .user("alice@tc", "pwd123", "alice", &["123"], Role::Tenant)
        .build();
    let resp = post_form(state, "/login", "email=alice%40tc&password=WRONG").await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    let body = String::from_utf8_lossy(&bytes);
    assert!(body.contains("Email ou mot de passe incorrect"));
}

#[tokio::test]
async fn session_cookie_authenticates_ui_request() {
    // 1. Login pour récupérer un cookie valide.
    let state = StateBuilder::new()
        .user("alice@tc", "pwd", "alice", &["123456789"], Role::Tenant)
        .build();
    let resp =
        post_form(state.clone(), "/login", "email=alice%40tc&password=pwd").await;
    let cookie = resp
        .headers()
        .get("set-cookie")
        .and_then(|v| v.to_str().ok())
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .to_string();

    // 2. Requête UI avec le cookie : doit passer (200), pas de redirect.
    let app = build_api_router(state);
    let req = Request::builder()
        .uri("/ui/emises?siren=123456789")
        .header("Cookie", cookie.clone())
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn session_cookie_blocks_cross_tenant() {
    // Le cookie d'alice ne donne accès qu'à 123456789 — siren foreign → 403.
    let state = StateBuilder::new()
        .user("alice@tc", "pwd", "alice", &["123456789"], Role::Tenant)
        .build();
    let resp =
        post_form(state.clone(), "/login", "email=alice%40tc&password=pwd").await;
    let cookie = resp
        .headers()
        .get("set-cookie")
        .and_then(|v| v.to_str().ok())
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .to_string();

    let app = build_api_router(state);
    let req = Request::builder()
        .uri("/ui/emises?siren=999999999")
        .header("Cookie", cookie)
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn cookie_marked_secure_when_x_forwarded_proto_https() {
    // Quand un proxy HTTPS injecte X-Forwarded-Proto: https, le cookie
    // doit porter le flag `Secure` pour éviter qu'il soit envoyé en clair
    // si l'utilisateur tape `http://...`.
    let state = StateBuilder::new()
        .user("alice@tc", "pwd", "alice", &["123456789"], Role::Tenant)
        .build();
    let app = build_api_router(state);
    let req = Request::builder()
        .uri("/login")
        .method("POST")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header("X-Forwarded-Proto", "https")
        .body(Body::from("email=alice%40tc&password=pwd".to_string()))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    let cookie = resp
        .headers()
        .get("set-cookie")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert!(
        cookie.contains("Secure"),
        "cookie doit porter `Secure` derrière HTTPS, got {cookie}"
    );
}

#[tokio::test]
async fn cookie_not_marked_secure_in_plain_http() {
    // Sans signal HTTPS (cas démo locale), le cookie ne doit PAS être
    // Secure (sinon le navigateur ne le renvoie pas).
    let state = StateBuilder::new()
        .user("alice@tc", "pwd", "alice", &["123456789"], Role::Tenant)
        .build();
    let resp = post_form(state, "/login", "email=alice%40tc&password=pwd").await;
    let cookie = resp
        .headers()
        .get("set-cookie")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert!(
        !cookie.contains("Secure"),
        "en HTTP la flag Secure ne doit pas être posée"
    );
}

#[tokio::test]
async fn cookie_invalidated_after_logout_even_if_replayed() {
    // Logout server-side : un attaquant qui rejoue le cookie après /logout
    // ne doit PAS pouvoir accéder à /ui (la signature est dans la
    // revocation list jusqu'à expiration naturelle).
    let state = StateBuilder::new()
        .user("alice@tc", "pwd", "alice", &["123456789"], Role::Tenant)
        .build();
    let resp =
        post_form(state.clone(), "/login", "email=alice%40tc&password=pwd").await;
    let cookie = resp
        .headers()
        .get("set-cookie")
        .and_then(|v| v.to_str().ok())
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .to_string();

    // 1. Cookie valide → 200
    let app = build_api_router(state.clone());
    let req = Request::builder()
        .uri("/ui/emises?siren=123456789")
        .header("Cookie", cookie.clone())
        .body(Body::empty())
        .unwrap();
    assert_eq!(app.oneshot(req).await.unwrap().status(), StatusCode::OK);

    // 2. Logout avec ce cookie
    let app = build_api_router(state.clone());
    let req = Request::builder()
        .uri("/logout")
        .method("POST")
        .header("Cookie", cookie.clone())
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::SEE_OTHER);

    // 3. Replay du même cookie → doit être rejeté (303 → /login)
    let app = build_api_router(state);
    let req = Request::builder()
        .uri("/ui/emises?siren=123456789")
        .header("Cookie", cookie)
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(
        resp.status(),
        StatusCode::SEE_OTHER,
        "cookie révoqué doit redirige vers /login (rejected)"
    );
}

#[tokio::test]
async fn logout_clears_cookie_and_redirects() {
    let state = StateBuilder::new().build();
    let resp = post_form(state, "/logout", "").await;
    assert_eq!(resp.status(), StatusCode::SEE_OTHER);
    let cookie = resp
        .headers()
        .get("set-cookie")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert!(cookie.starts_with("ferrite_session="));
    assert!(cookie.contains("Max-Age=0"));
    let location = resp
        .headers()
        .get("location")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert_eq!(location, "/login");
}

#[tokio::test]
async fn security_headers_present_on_all_responses() {
    // Les headers de sécurité sont posés par le middleware sur TOUTES les
    // réponses — y compris une 303 vers /login pour une requête /ui non
    // authentifiée. C'est suffisant pour valider la couverture.
    let state = StateBuilder::new().build();
    let app = build_api_router(state);
    let req = Request::builder().uri("/ui").body(Body::empty()).unwrap();
    let resp = app.oneshot(req).await.unwrap();
    let h = resp.headers();
    assert!(h.contains_key("content-security-policy"));
    assert!(h.contains_key("strict-transport-security"));
    assert_eq!(h.get("x-frame-options").unwrap(), "DENY");
    assert_eq!(h.get("x-content-type-options").unwrap(), "nosniff");
    assert!(h.contains_key("referrer-policy"));
}

#[tokio::test]
async fn annuaire_remains_public() {
    // Confirmation qu'on n'a pas régressé : `/annuaire` doit rester accessible
    // sans token et sans cookie (choix produit explicite).
    let state = StateBuilder::new().build();
    let (status, _) = send(state, "/annuaire", None).await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn flow_detail_ui_403_on_other_tenant() {
    let state = StateBuilder::new().tenant_token("tok-a", &["123456789"]).build();
    let (status, _) = send(
        state,
        "/ui/flows/ex-999999999-INV-B?siren=999999999",
        Some("tok-a"),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}
