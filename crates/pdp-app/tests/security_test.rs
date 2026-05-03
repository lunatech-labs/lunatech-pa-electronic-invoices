//! Tests d'isolation tenant et d'authentification.
//!
//! Vérifient que :
//! 1. Sans token et sans `dev_open`, toute route protégée retourne 401
//! 2. Un token `Tenant` ne voit QUE ses SIRENs autorisés (403 sur les autres)
//! 3. Un token `PdpAdmin` peut consulter n'importe quel SIREN
//! 4. `get_exchange` filtre par siren côté backend (un mauvais siren → None)
//! 5. Le mode `dev_open: true` accepte sans token (pour la démo locale)
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
    ) -> PdpResult<Vec<ExchangeSummary>> {
        Ok(self
            .0
            .iter()
            .filter(|d| {
                d.seller_siren.as_deref() == Some(siren)
                    || d.buyer_siren.as_deref() == Some(siren)
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
    ) -> PdpResult<i64> {
        Ok(self
            .0
            .iter()
            .filter(|d| {
                d.seller_siren.as_deref() == Some(siren)
                    || d.buyer_siren.as_deref() == Some(siren)
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
    dev_open: bool,
    tokens: HashMap<String, SecurityContext>,
    docs: Vec<ExchangeDocument>,
}

impl StateBuilder {
    fn new() -> Self {
        Self {
            dev_open: false,
            tokens: HashMap::new(),
            docs: vec![
                doc_for("123456789", "987654321", "INV-A"),
                doc_for("999999999", "888888888", "INV-B"),
            ],
        }
    }

    fn dev_open(mut self) -> Self {
        self.dev_open = true;
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
            dev_open: self.dev_open,
            metrics: Metrics::default(),
            annuaire_store: None,
            webhook_store: Arc::new(WebhookStore::new()),
            max_flow_size_bytes: 100 * 1024 * 1024,
            request_timeout: std::time::Duration::from_secs(30),
            rate_limiter: None,
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
async fn no_token_rejected_when_dev_open_false() {
    let state = StateBuilder::new().tenant_token("tok-a", &["123456789"]).build();
    let (status, _) = send(state, "/ui/flows?siren=123456789", None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn unknown_token_rejected() {
    let state = StateBuilder::new().tenant_token("tok-a", &["123456789"]).build();
    let (status, _) = send(state, "/ui/flows?siren=123456789", Some("tok-zzz")).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn tenant_token_can_access_own_siren() {
    let state = StateBuilder::new().tenant_token("tok-a", &["123456789"]).build();
    let (status, body) =
        send(state, "/ui/flows?siren=123456789", Some("tok-a")).await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.contains("INV-A"), "doit lister INV-A du tenant 123456789");
}

#[tokio::test]
async fn tenant_token_cannot_access_other_siren() {
    // tok-a est lié à 123456789 → demander ?siren=999999999 doit être 403.
    let state = StateBuilder::new().tenant_token("tok-a", &["123456789"]).build();
    let (status, body) =
        send(state, "/ui/flows?siren=999999999", Some("tok-a")).await;
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
        send(state, "/ui/flows?siren=999999999", Some("tok-admin")).await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.contains("INV-B"));
}

#[tokio::test]
async fn dev_open_grants_admin_without_token() {
    let state = StateBuilder::new().dev_open().build();
    let (status, _) = send(state, "/ui/flows?siren=999999999", None).await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn v1_stats_requires_siren() {
    // Avec dev_open + sans ?siren=, l'extractor AuthorizedSiren retourne 400.
    let state = StateBuilder::new().dev_open().build();
    let (status, body) = send(state, "/v1/stats", None).await;
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
