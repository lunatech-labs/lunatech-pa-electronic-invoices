//! Tests d'intégration de l'UI (interface web de suivi des flux).
//!
//! Approche : on teste les handlers Axum via `tower::ServiceExt::oneshot`
//! avec un [`InMemoryTraceBackend`] mocké (zéro dépendance externe — pas
//! d'Elasticsearch, pas de Postgres). Le mock implémente la même sémantique
//! de filtrage que [`pdp_trace::TraceStore`] (statuts terminaux OK / FAIL,
//! `error_count`, plages de dates, pagination) afin que ces tests détectent
//! les régressions sur la *vraie* logique des handlers.

use std::sync::Arc;

use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;

use pdp_app::server::{build_api_router, AppState, Metrics};
use pdp_app::webhooks::WebhookStore;
use pdp_core::error::PdpResult;
use pdp_trace::store::{
    EventEntry, ExchangeDocument, ExchangeSummary, TraceStats,
};
use pdp_trace::TraceBackend;

// ---------------------------------------------------------------------------
// Mock : InMemoryTraceBackend
// ---------------------------------------------------------------------------

/// Statuts considérés comme terminaux réussis (cf. `TraceStore::list_exchanges`).
const TERMINAL_OK: &[&str] = &[
    "VALIDÉ",
    "TRANSFORMÉ",
    "DISTRIBUTION",
    "DISTRIBUÉ",
    "ATTENTE_ACK",
    "ACQUITTÉ",
];

/// Statuts considérés comme terminaux d'échec.
const TERMINAL_FAIL: &[&str] = &["REJETÉ", "ANNULÉ", "ERREUR"];

/// Implémentation 100 % en mémoire de [`TraceBackend`]. Stocke les documents
/// dans un `Vec` et applique les filtres en Rust pur.
pub struct InMemoryTraceBackend {
    docs: Vec<ExchangeDocument>,
}

impl InMemoryTraceBackend {
    pub fn new(docs: Vec<ExchangeDocument>) -> Self {
        Self { docs }
    }

    /// Un doc concerne un tenant si le SIREN matche **vendeur OU acheteur**.
    fn matches_tenant(d: &ExchangeDocument, siren: &str) -> bool {
        d.seller_siren.as_deref() == Some(siren)
            || d.buyer_siren.as_deref() == Some(siren)
    }

    fn matches_status(d: &ExchangeDocument, status: &str) -> bool {
        match status.to_uppercase().as_str() {
            "OK" | "DISTRIBUÉ" | "DISTRIBUE" | "VALIDÉ" | "VALIDE" => {
                d.error_count == 0 && TERMINAL_OK.contains(&d.status.as_str())
            }
            "ERREUR" | "ERROR" => {
                d.error_count > 0 || TERMINAL_FAIL.contains(&d.status.as_str())
            }
            "EN_ATTENTE" | "ATTENTE" | "PENDING" => {
                d.error_count == 0
                    && !TERMINAL_OK.contains(&d.status.as_str())
                    && !TERMINAL_FAIL.contains(&d.status.as_str())
            }
            other => d.status == other,
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
}

#[async_trait]
impl TraceBackend for InMemoryTraceBackend {
    async fn get_stats(&self) -> PdpResult<TraceStats> {
        Ok(TraceStats {
            total_exchanges: self.docs.len() as i64,
            total_errors: self.docs.iter().filter(|d| d.error_count > 0).count() as i64,
            total_distributed: self
                .docs
                .iter()
                .filter(|d| d.status == "DISTRIBUÉ")
                .count() as i64,
        })
    }

    async fn get_stats_for_siren(&self, siren: &str) -> PdpResult<TraceStats> {
        // Sémantique alignée sur list_exchanges + filtres UI :
        //  - "errors" = error_count > 0 OU status terminal d'échec
        //  - "distributed" = error_count = 0 ET status terminal OK
        let scoped: Vec<&ExchangeDocument> = self
            .docs
            .iter()
            .filter(|d| Self::matches_tenant(d, siren))
            .collect();
        let errors = scoped
            .iter()
            .filter(|d| d.error_count > 0 || TERMINAL_FAIL.contains(&d.status.as_str()))
            .count() as i64;
        let distributed = scoped
            .iter()
            .filter(|d| d.error_count == 0 && TERMINAL_OK.contains(&d.status.as_str()))
            .count() as i64;
        Ok(TraceStats {
            total_exchanges: scoped.len() as i64,
            total_errors: errors,
            total_distributed: distributed,
        })
    }

    async fn get_tenant_name(&self, siren: &str) -> Option<String> {
        // D'abord chercher un doc où le tenant est *vendeur* (raison sociale
        // = seller_name). Sinon, retomber sur un doc où il est acheteur.
        if let Some(name) = self
            .docs
            .iter()
            .find(|d| d.seller_siren.as_deref() == Some(siren))
            .and_then(|d| d.seller_name.clone())
        {
            return Some(name);
        }
        self.docs
            .iter()
            .find(|d| d.buyer_siren.as_deref() == Some(siren))
            .and_then(|d| d.buyer_name.clone())
    }

    async fn list_exchanges(
        &self,
        siren: &str,
        status: Option<&str>,
        from_date: Option<&str>,
        to_date: Option<&str>,
        page: usize,
        page_size: usize,
    ) -> PdpResult<Vec<ExchangeSummary>> {
        let mut hits: Vec<&ExchangeDocument> = self
            .docs
            .iter()
            .filter(|d| Self::matches_tenant(d, siren))
            .filter(|d| status.map(|s| Self::matches_status(d, s)).unwrap_or(true))
            .filter(|d| match (from_date, d.issue_date.as_deref()) {
                (Some(f), Some(date)) => date >= f,
                (Some(_), None) => false,
                _ => true,
            })
            .filter(|d| match (to_date, d.issue_date.as_deref()) {
                (Some(t), Some(date)) => date <= t,
                (Some(_), None) => false,
                _ => true,
            })
            .collect();
        // Tri created_at desc (comme ES)
        hits.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        let from = page * page_size;
        let summaries: Vec<ExchangeSummary> = hits
            .into_iter()
            .skip(from)
            .take(page_size)
            .map(Self::to_summary)
            .collect();
        Ok(summaries)
    }

    async fn count_exchanges(
        &self,
        siren: &str,
        status: Option<&str>,
        from_date: Option<&str>,
        to_date: Option<&str>,
    ) -> PdpResult<i64> {
        // Réutilise la même logique de filtrage que list_exchanges (sans pagination).
        let count = self
            .docs
            .iter()
            .filter(|d| Self::matches_tenant(d, siren))
            .filter(|d| status.map(|s| Self::matches_status(d, s)).unwrap_or(true))
            .filter(|d| match (from_date, d.issue_date.as_deref()) {
                (Some(f), Some(date)) => date >= f,
                (Some(_), None) => false,
                _ => true,
            })
            .filter(|d| match (to_date, d.issue_date.as_deref()) {
                (Some(t), Some(date)) => date <= t,
                (Some(_), None) => false,
                _ => true,
            })
            .count();
        Ok(count as i64)
    }

    async fn get_exchange(
        &self,
        exchange_id: &str,
        siren: Option<&str>,
    ) -> PdpResult<Option<ExchangeDocument>> {
        // Lookup par exchange_id ET vérification que le SIREN demandé est
        // partie au flux (vendeur ou acheteur). Sans ce filtre, un user qui
        // devine un exchange_id pourrait lire les factures d'un autre tenant.
        let found = self.docs.iter().find(|d| {
            d.exchange_id == exchange_id
                && siren
                    .map(|s| Self::matches_tenant(d, s))
                    .unwrap_or(true)
        });
        Ok(found.map(clone_doc))
    }

    async fn get_error_flows(&self) -> PdpResult<Vec<ExchangeSummary>> {
        let mut hits: Vec<&ExchangeDocument> = self
            .docs
            .iter()
            .filter(|d| d.error_count > 0 || TERMINAL_FAIL.contains(&d.status.as_str()))
            .collect();
        hits.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(hits.into_iter().map(Self::to_summary).collect())
    }
}

fn clone_doc(d: &ExchangeDocument) -> ExchangeDocument {
    // Sérialise/désérialise pour cloner (ExchangeDocument ne dérive pas Clone).
    let json = serde_json::to_value(d).expect("serialize ExchangeDocument");
    serde_json::from_value(json).expect("roundtrip ExchangeDocument")
}

// ---------------------------------------------------------------------------
// Helpers : construction d'AppState et de documents de seed
// ---------------------------------------------------------------------------

fn build_state(backend: Arc<dyn TraceBackend>) -> Arc<AppState> {
    let (tx, _rx) = tokio::sync::mpsc::channel(10);
    Arc::new(AppState {
        pdp_name: "Ferrite Test".to_string(),
        pdp_matricule: "0001".to_string(),
        flow_sender: tx,
        webhook_secret: None,
        trace_store: Some(backend),
        tokens: std::collections::HashMap::new(), dev_open: true,
        metrics: Metrics::default(),
        annuaire_store: None,
        webhook_store: Arc::new(WebhookStore::new()),
        max_flow_size_bytes: 100 * 1024 * 1024,
        request_timeout: std::time::Duration::from_secs(30),
        rate_limiter: None,
    })
}

/// Crée un document minimal avec les champs utiles aux tests UI.
fn doc(
    exchange_id: &str,
    seller_siren: &str,
    buyer_siren: &str,
    invoice_number: &str,
    status: &str,
    error_count: i32,
    issue_date: &str,
    created_at: &str,
) -> ExchangeDocument {
    ExchangeDocument {
        exchange_id: exchange_id.to_string(),
        flow_id: format!("flow-{}", exchange_id),
        source_filename: Some(format!("{}.xml", invoice_number)),
        invoice_number: Some(invoice_number.to_string()),
        invoice_key: Some(format!("{}/{}/2026", seller_siren, invoice_number)),
        seller_name: Some(format!("Vendeur {}", seller_siren)),
        buyer_name: Some(format!("Acheteur {}", buyer_siren)),
        seller_siret: Some(format!("{}00001", seller_siren)),
        buyer_siret: Some(format!("{}00002", buyer_siren)),
        seller_siren: Some(seller_siren.to_string()),
        buyer_siren: Some(buyer_siren.to_string()),
        source_format: Some("UBL".to_string()),
        total_ht: Some(1000.0),
        total_ttc: Some(1200.0),
        total_tax: Some(200.0),
        currency: Some("EUR".to_string()),
        issue_date: Some(issue_date.to_string()),
        status: status.to_string(),
        error_count,
        raw_xml: Some(format!("<Invoice id=\"{}\"/>", invoice_number)),
        raw_pdf_base64: None,
        converted_xml: None,
        converted_format: None,
        attachment_count: 0,
        attachment_filenames: Vec::new(),
        events: vec![EventEntry {
            id: "evt-1".to_string(),
            route_id: "route-test".to_string(),
            status: status.to_string(),
            message: "Test event".to_string(),
            error_detail: None,
            timestamp: created_at.to_string(),
        }],
        errors: Vec::new(),
        validation_warnings: Vec::new(),
        created_at: created_at.to_string(),
        updated_at: created_at.to_string(),
    }
}

/// Jeu de données type : 5 docs pour SIREN 123456789, plus 1 pour 999999999.
fn seed() -> Vec<ExchangeDocument> {
    vec![
        // 3 distribués sans erreur (OK)
        doc(
            "ex-1", "123456789", "987654321", "INV-001", "DISTRIBUÉ", 0,
            "2026-04-01", "2026-04-01T10:00:00Z",
        ),
        doc(
            "ex-2", "123456789", "987654321", "INV-002", "VALIDÉ", 0,
            "2026-04-15", "2026-04-15T10:00:00Z",
        ),
        doc(
            "ex-3", "123456789", "987654321", "INV-003", "ACQUITTÉ", 0,
            "2026-05-01", "2026-05-01T10:00:00Z",
        ),
        // 1 en erreur (error_count > 0)
        {
            let mut d = doc(
                "ex-err", "123456789", "987654321", "INV-ERR", "VALIDÉ", 2,
                "2026-04-20", "2026-04-20T10:00:00Z",
            );
            d.errors = vec![pdp_trace::store::ErrorEntry {
                step: "validation".to_string(),
                message: "Champ obligatoire manquant".to_string(),
                detail: None,
                timestamp: "2026-04-20T10:00:01Z".to_string(),
            }];
            d
        },
        // 1 en attente (status PARSING — ni OK ni FAIL terminal)
        doc(
            "ex-pending", "123456789", "987654321", "INV-PEND", "PARSING", 0,
            "2026-04-25", "2026-04-25T10:00:00Z",
        ),
        // 1 doc avec PJ pour tester le badge "📎"
        {
            let mut d = doc(
                "ex-pj", "123456789", "987654321", "INV-PJ", "VALIDÉ", 0,
                "2026-04-10", "2026-04-10T10:00:00Z",
            );
            d.attachment_count = 2;
            d.attachment_filenames =
                vec!["bon_commande.pdf".into(), "specs.txt".into()];
            d
        },
        // 1 doc sur un autre tenant — ne doit jamais apparaître pour 123456789
        doc(
            "ex-other", "999999999", "111111111", "OTHER-001", "DISTRIBUÉ", 0,
            "2026-04-01", "2026-04-01T10:00:00Z",
        ),
        // 2 factures REÇUES par 123456789 (tenant = acheteur, pas vendeur)
        doc(
            "ex-recu-1", "738492012", "123456789", "REC-PLO-001", "VALIDÉ", 0,
            "2026-04-05", "2026-04-05T10:00:00Z",
        ),
        {
            let mut d = doc(
                "ex-recu-2", "415263748", "123456789", "REC-CE-014", "VALIDÉ", 0,
                "2026-04-08", "2026-04-08T10:00:00Z",
            );
            d.attachment_count = 1;
            d.attachment_filenames = vec!["devis_signe.pdf".into()];
            d
        },
    ]
}

async fn body_text(resp: axum::response::Response) -> String {
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    String::from_utf8_lossy(&bytes).to_string()
}

async fn get_html(state: Arc<AppState>, uri: &str) -> (StatusCode, String) {
    let app = build_api_router(state);
    let req = Request::builder().uri(uri).body(Body::empty()).unwrap();
    let resp = app.oneshot(req).await.unwrap();
    let status = resp.status();
    let body = body_text(resp).await;
    (status, body)
}

fn make_state() -> Arc<AppState> {
    build_state(Arc::new(InMemoryTraceBackend::new(seed())))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_dashboard_kpi_with_data() {
    let state = make_state();
    let (status, body) = get_html(state, "/ui?siren=123456789").await;
    assert_eq!(status, StatusCode::OK);

    // Le tenant 123456789 a 6 docs : 1 distribué + 1 validé + 1 acquitté + 1 erreur + 1 pending + 1 PJ
    assert!(body.contains("123456789"), "doit afficher le SIREN");
    // Raison sociale lue depuis seller_name du premier doc
    assert!(
        body.contains("Vendeur 123456789"),
        "doit afficher la raison sociale tenant"
    );
    // KPI total = 6 (5 + 1 PJ)
    assert!(body.contains(">6<"), "KPI total doit valoir 6 ; body=\n{body}");
    // KPI erreurs = 1
    assert!(body.contains(">1<"), "KPI erreurs doit valoir 1");
}

#[tokio::test]
async fn test_flows_list_default_shows_all_for_tenant() {
    let state = make_state();
    let (status, body) =
        get_html(state, "/ui/flows?siren=123456789").await;
    assert_eq!(status, StatusCode::OK);

    // 6 docs du tenant doivent apparaître
    for inv in &["INV-001", "INV-002", "INV-003", "INV-ERR", "INV-PEND", "INV-PJ"] {
        assert!(body.contains(inv), "facture {inv} doit être listée");
    }
    // L'autre tenant ne doit JAMAIS apparaître
    assert!(
        !body.contains("OTHER-001"),
        "facture d'un autre tenant ne doit pas apparaître"
    );
}

#[tokio::test]
async fn test_filter_status_ok_excludes_errors_and_pending() {
    let state = make_state();
    let (_status, body) =
        get_html(state, "/ui/flows?siren=123456789&status=OK").await;

    // OK = error_count=0 ET status terminal OK
    for inv in &["INV-001", "INV-002", "INV-003", "INV-PJ"] {
        assert!(
            body.contains(inv),
            "OK doit inclure {inv} ; body court = {}",
            &body[..400.min(body.len())]
        );
    }
    assert!(!body.contains("INV-ERR"), "OK ne doit pas inclure les erreurs");
    assert!(!body.contains("INV-PEND"), "OK ne doit pas inclure les en-attente");
}

#[tokio::test]
async fn test_filter_status_erreur_returns_only_errors() {
    let state = make_state();
    let (_status, body) =
        get_html(state, "/ui/flows?siren=123456789&status=ERREUR").await;

    assert!(body.contains("INV-ERR"), "ERREUR doit inclure INV-ERR");
    for inv in &["INV-001", "INV-002", "INV-003", "INV-PJ", "INV-PEND"] {
        assert!(
            !body.contains(inv),
            "ERREUR ne doit pas inclure {inv}"
        );
    }
}

#[tokio::test]
async fn test_filter_status_en_attente_returns_only_pending() {
    let state = make_state();
    let (_status, body) =
        get_html(state, "/ui/flows?siren=123456789&status=EN_ATTENTE").await;

    assert!(body.contains("INV-PEND"), "EN_ATTENTE doit inclure INV-PEND");
    for inv in &["INV-001", "INV-002", "INV-003", "INV-PJ", "INV-ERR"] {
        assert!(
            !body.contains(inv),
            "EN_ATTENTE ne doit pas inclure {inv}"
        );
    }
}

#[tokio::test]
async fn test_filter_date_range() {
    let state = make_state();
    // Plage couvrant uniquement INV-002 (15/04) et INV-ERR (20/04) et INV-PEND (25/04)
    let (_status, body) = get_html(
        state,
        "/ui/flows?siren=123456789&from=2026-04-15&to=2026-04-25",
    )
    .await;

    for inv in &["INV-002", "INV-ERR", "INV-PEND"] {
        assert!(body.contains(inv), "{inv} dans la plage");
    }
    for inv in &["INV-001", "INV-003", "INV-PJ"] {
        assert!(
            !body.contains(inv),
            "{inv} hors plage et ne doit pas apparaître"
        );
    }
}

#[tokio::test]
async fn test_dedup_invoice_number_default_and_show_duplicates() {
    // Deux exchanges avec le MÊME invoice_number : seul le plus récent
    // doit être visible par défaut.
    let mut docs = seed();
    docs.push(doc(
        "ex-1-dup",
        "123456789",
        "987654321",
        "INV-001", // doublon
        "DISTRIBUÉ",
        0,
        "2026-04-01",
        "2026-04-02T10:00:00Z", // plus récent
    ));
    let state = build_state(Arc::new(InMemoryTraceBackend::new(docs)));

    // Par défaut : un seul "INV-001" — comptage du nombre de liens vers ce numéro
    let (_status, body) =
        get_html(state.clone(), "/ui/flows?siren=123456789").await;
    let count = body.matches("INV-001<").count() + body.matches("INV-001 ").count() + body.matches(">INV-001<").count();
    assert!(
        count <= 2,
        "INV-001 ne doit apparaître qu'une fois (dedup par défaut), trouvé {count}"
    );

    // Avec ?show_duplicates=true : les 2 exchanges doivent apparaître
    let (_status, body_dup) = get_html(
        state,
        "/ui/flows?siren=123456789&show_duplicates=true",
    )
    .await;
    // Les exchange_id distincts doivent tous deux apparaître dans les liens
    assert!(
        body_dup.contains("flow-ex-1") && body_dup.contains("flow-ex-1-dup"),
        "show_duplicates=true doit afficher les 2 exchanges"
    );
}

#[tokio::test]
async fn test_flow_detail_renders_metadata() {
    let state = make_state();
    let (status, body) =
        get_html(state, "/ui/flows/flow-ex-1?siren=123456789").await;
    assert_eq!(status, StatusCode::OK);

    // Métadonnées clés
    assert!(body.contains("INV-001"), "numéro de facture");
    assert!(body.contains("Vendeur 123456789"), "nom vendeur");
    assert!(body.contains("Acheteur 987654321"), "nom acheteur");
    assert!(body.contains("DISTRIBUÉ"), "statut");
    // Timeline événement
    assert!(body.contains("Test event") || body.contains("route-test"));
}

#[tokio::test]
async fn test_flow_detail_attachments_section() {
    let state = make_state();
    let (status, body) =
        get_html(state, "/ui/flows/flow-ex-pj?siren=123456789").await;
    assert_eq!(status, StatusCode::OK);

    assert!(body.contains("INV-PJ"));
    // L'un des deux noms de PJ déclarés dans `attachment_filenames`
    assert!(
        body.contains("bon_commande.pdf") || body.contains("specs.txt"),
        "doit lister une des pièces jointes"
    );
}

#[tokio::test]
async fn test_flows_list_includes_received_invoices() {
    // Pour le tenant 123456789, les factures dont il est *acheteur* (REC-PLO-001,
    // REC-CE-014) doivent apparaître dans sa liste — même si elles sont
    // indexées sous l'index du fournisseur en réalité.
    let state = make_state();
    let (_status, body) = get_html(state, "/ui/flows?siren=123456789").await;

    for inv in &["REC-PLO-001", "REC-CE-014"] {
        assert!(
            body.contains(inv),
            "{inv} (facture reçue) doit apparaître dans la liste du tenant ; \
             body court=\n{}",
            &body[..400.min(body.len())]
        );
    }
    // Les émises restent visibles
    assert!(body.contains("INV-001"));
}

#[tokio::test]
async fn test_flow_detail_received_invoice_renders() {
    // La page détail d'une facture *reçue* doit fonctionner même quand le
    // SIREN dans l'URL est celui de l'acheteur (et non du vendeur qui contient
    // le doc dans son index).
    let state = make_state();
    let (status, body) =
        get_html(state, "/ui/flows/flow-ex-recu-1?siren=123456789").await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.contains("REC-PLO-001"), "numéro de la facture reçue");
    assert!(body.contains("Vendeur 738492012"));
    assert!(body.contains("Acheteur 123456789"));
}

#[tokio::test]
async fn test_dashboard_kpis_match_filter_results() {
    // Cohérence : le KPI "Distribués" du dashboard doit valoir le nombre de
    // factures listées sous le filtre "OK", et idem pour "Erreurs" / "ERREUR"
    // et "En attente" / "EN_ATTENTE". Sinon l'utilisateur voit un dashboard
    // qui contredit la liste.
    let state = make_state();

    // Compteurs par catégorie selon le seed (cf. fonction seed())
    // OK : INV-001 (DISTRIBUÉ), INV-002 (VALIDÉ), INV-003 (ACQUITTÉ),
    //      INV-PJ (VALIDÉ), REC-PLO-001 (VALIDÉ), REC-CE-014 (VALIDÉ) → 6
    // ERREUR : INV-ERR (error_count=2) → 1
    // EN_ATTENTE : INV-PEND (PARSING) → 1
    // Total tenant 123456789 = 8

    let (_, dashboard) = get_html(state.clone(), "/ui?siren=123456789").await;
    let extract = |kpi_class: &str, body: &str| -> usize {
        let needle = format!(r#"<div class="kpi-value {}">"#, kpi_class);
        body.split(&needle)
            .nth(1)
            .and_then(|s| s.split('<').next())
            .and_then(|n| n.trim().parse::<usize>().ok())
            .unwrap_or_else(|| panic!("KPI '{kpi_class}' introuvable dans dashboard"))
    };
    let dist = extract("success", &dashboard);
    let err = extract("error", &dashboard);
    let pending = extract("warning", &dashboard);

    // Liste avec filtre OK
    let (_, ok_html) =
        get_html(state.clone(), "/ui/flows?siren=123456789&status=OK").await;
    let ok_count = ["INV-001", "INV-002", "INV-003", "INV-PJ", "REC-PLO-001", "REC-CE-014"]
        .iter()
        .filter(|inv| ok_html.contains(*inv))
        .count();
    assert_eq!(
        dist, ok_count,
        "KPI Distribués ({dist}) doit correspondre au nb de factures listées avec filtre OK ({ok_count})"
    );

    // Liste avec filtre ERREUR
    let (_, err_html) =
        get_html(state.clone(), "/ui/flows?siren=123456789&status=ERREUR").await;
    let err_count = ["INV-ERR"]
        .iter()
        .filter(|inv| err_html.contains(*inv))
        .count();
    assert_eq!(err, err_count, "KPI Erreurs ({err}) ≠ liste ?status=ERREUR ({err_count})");

    // Liste avec filtre EN_ATTENTE
    let (_, pending_html) =
        get_html(state, "/ui/flows?siren=123456789&status=EN_ATTENTE").await;
    let pending_count = ["INV-PEND"]
        .iter()
        .filter(|inv| pending_html.contains(*inv))
        .count();
    assert_eq!(
        pending, pending_count,
        "KPI En attente ({pending}) ≠ liste ?status=EN_ATTENTE ({pending_count})"
    );
}

#[tokio::test]
async fn test_page_size_selector_limits_results_and_paginates() {
    // Crée 30 docs pour le tenant 123456789 → on peut tester page_size=25 :
    // page 0 contient 25 lignes, page 1 contient 5.
    let mut docs = Vec::new();
    for i in 0..30 {
        docs.push(doc(
            &format!("ex-page-{i:03}"),
            "123456789",
            "987654321",
            &format!("INV-PG-{i:03}"),
            "DISTRIBUÉ",
            0,
            "2026-04-01",
            // Trier desc par created_at : les plus récents en premier
            &format!("2026-04-{:02}T10:00:00Z", 30 - i % 28),
        ));
    }
    let state = build_state(Arc::new(InMemoryTraceBackend::new(docs)));

    let count_invoices = |body: &str| -> usize {
        (0..30).filter(|i| body.contains(&format!("INV-PG-{i:03}"))).count()
    };

    // page_size=25, page=0 → 25 invoices visibles
    let (_, p0) = get_html(
        state.clone(),
        "/ui/flows?siren=123456789&page_size=25&page=0",
    )
    .await;
    assert_eq!(count_invoices(&p0), 25, "page 0 / size 25 → 25 lignes");

    // page_size=25, page=1 → reste = 5
    let (_, p1) = get_html(
        state.clone(),
        "/ui/flows?siren=123456789&page_size=25&page=1",
    )
    .await;
    assert_eq!(count_invoices(&p1), 5, "page 1 / size 25 → 5 lignes");

    // Le sélecteur HTML doit refléter la valeur courante
    assert!(
        p0.contains(r#"<option value="25" selected>25 / page</option>"#),
        "select page_size doit avoir 25 selected"
    );
    // Les liens de pagination doivent porter page_size pour préserver le choix
    assert!(
        p0.contains("page_size=25"),
        "lien Suivant doit conserver page_size=25"
    );
}

#[tokio::test]
async fn test_pagination_shows_total_count_and_page_count() {
    // Crée 30 factures pour 123456789 → avec page_size=25 on a 2 pages,
    // et le total doit valoir 30 sur les deux pages.
    let mut docs = Vec::new();
    for i in 0..30 {
        docs.push(doc(
            &format!("ex-tot-{i:03}"),
            "123456789",
            "987654321",
            &format!("INV-T-{i:03}"),
            "DISTRIBUÉ",
            0,
            "2026-04-01",
            &format!("2026-04-{:02}T10:00:00Z", (i % 28) + 1),
        ));
    }
    let state = build_state(Arc::new(InMemoryTraceBackend::new(docs)));

    // Page 0 : "1–25 sur 30 ... page 1/2"
    let (_, p0) = get_html(
        state.clone(),
        "/ui/flows?siren=123456789&page_size=25&page=0",
    )
    .await;
    assert!(p0.contains("1–25"), "doit afficher la plage 1–25 ; body={p0}");
    assert!(p0.contains("sur <strong>30</strong>"), "doit afficher le total 30");
    assert!(p0.contains("page 1/2"), "page 1/2 ; body court={}", &p0[..400.min(p0.len())]);

    // Page 1 : "26–30 sur 30 ... page 2/2", pas de Suivant
    let (_, p1) = get_html(
        state.clone(),
        "/ui/flows?siren=123456789&page_size=25&page=1",
    )
    .await;
    assert!(p1.contains("26–30"), "plage 26–30 ; got body");
    assert!(p1.contains("sur <strong>30</strong>"));
    assert!(p1.contains("page 2/2"));
    // Suivant désactivé sur la dernière page
    assert!(
        p1.contains(r#"<span style="color:#aaa">Suivant →</span>"#),
        "Suivant doit être désactivé sur la dernière page"
    );
}

#[tokio::test]
async fn test_pagination_total_respects_filters() {
    // Avec un filtre status=ERREUR, le total doit refléter le nombre de
    // factures FILTRÉES, pas le nombre total du tenant.
    let state = make_state();
    let (_, body) =
        get_html(state, "/ui/flows?siren=123456789&status=ERREUR").await;
    // Le seed contient 1 erreur (INV-ERR) sous le tenant 123456789
    assert!(
        body.contains("sur <strong>1</strong>"),
        "total filtré = 1 ; got body court = {}",
        &body[..400.min(body.len())]
    );
}

#[tokio::test]
async fn test_page_size_invalid_falls_back_to_default() {
    // ?page_size=10000 (hors liste autorisée) → retombe sur 50 par défaut.
    let mut docs = Vec::new();
    for i in 0..60 {
        docs.push(doc(
            &format!("ex-fb-{i:03}"),
            "123456789",
            "987654321",
            &format!("INV-FB-{i:03}"),
            "DISTRIBUÉ",
            0,
            "2026-04-01",
            &format!("2026-04-{:02}T10:00:00Z", (i % 28) + 1),
        ));
    }
    let state = build_state(Arc::new(InMemoryTraceBackend::new(docs)));
    let (_, body) =
        get_html(state, "/ui/flows?siren=123456789&page_size=10000").await;
    let count = (0..60)
        .filter(|i| body.contains(&format!("INV-FB-{i:03}")))
        .count();
    assert_eq!(count, 50, "page_size hors plage → défaut 50, pas 10000");
}

#[tokio::test]
async fn test_timeline_truncates_after_first_error() {
    // Sur une facture en erreur, la timeline ne doit pas afficher des statuts
    // pipeline post-erreur (DISTRIBUÉ après une annuaire-validation KO),
    // sinon l'utilisateur croit que la facture est partie.
    let mut docs = seed();
    let target_id = "ex-with-pipeline-errors";
    let mut d = doc(
        target_id, "123456789", "987654321", "INV-PIPELINE-ERR", "DISTRIBUÉ", 1,
        "2026-04-30", "2026-04-30T10:00:00Z",
    );
    d.events = vec![
        pdp_trace::store::EventEntry {
            id: "e1".into(), route_id: "rt".into(), status: "REÇU".into(),
            message: "reçu".into(), error_detail: None,
            timestamp: "2026-04-30T10:00:01Z".into(),
        },
        pdp_trace::store::EventEntry {
            id: "e2".into(), route_id: "rt".into(), status: "PARSÉ".into(),
            message: "parsé".into(), error_detail: None,
            timestamp: "2026-04-30T10:00:02Z".into(),
        },
        pdp_trace::store::EventEntry {
            id: "e3".into(), route_id: "rt".into(), status: "VALIDÉ".into(),
            message: "validé".into(), error_detail: None,
            timestamp: "2026-04-30T10:00:03Z".into(),
        },
        // L'événement DISTRIBUÉ ne doit PAS apparaître dans la timeline
        // car une erreur s'est produite avant.
        pdp_trace::store::EventEntry {
            id: "e4".into(), route_id: "rt".into(), status: "DISTRIBUÉ".into(),
            message: "distribué".into(), error_detail: None,
            timestamp: "2026-04-30T10:00:05Z".into(),
        },
    ];
    d.errors = vec![pdp_trace::store::ErrorEntry {
        step: "annuaire-validation".into(),
        message: "Vendeur inconnu (BR-FR-10)".into(),
        detail: None,
        timestamp: "2026-04-30T10:00:04Z".into(),
    }];
    docs.push(d);

    let state = build_state(Arc::new(InMemoryTraceBackend::new(docs)));
    let (_, body) =
        get_html(state, &format!("/ui/flows/flow-{target_id}?siren=123456789")).await;

    // Timeline : REÇU/PARSÉ/VALIDÉ visibles + 1 entrée d'erreur ; DISTRIBUÉ NON
    let timeline_start = body.find(r#"class="timeline""#).expect("bloc timeline");
    let timeline_end = body[timeline_start..].find("</div></div>").map(|p| timeline_start + p).unwrap_or(body.len());
    let timeline = &body[timeline_start..timeline_end];
    assert!(timeline.contains("REÇU"));
    assert!(timeline.contains("PARSÉ"));
    assert!(timeline.contains("VALIDÉ"));
    assert!(timeline.contains("annuaire-validation"));
    assert!(timeline.contains("Vendeur inconnu"));
    assert!(timeline.contains("timeline-error"), "doit avoir une entrée timeline-error");
    assert!(
        !timeline.contains("DISTRIBUÉ"),
        "DISTRIBUÉ est postérieur à l'erreur — ne doit pas apparaître dans la timeline ; got=\n{timeline}"
    );
}

#[tokio::test]
async fn test_status_with_errors_shows_erreur_not_raw_status() {
    // Une facture avec error_count > 0 doit afficher le badge "ERREUR" (rouge)
    // dans la liste ET le détail, même si son statut brut du pipeline est
    // "DISTRIBUÉ" / "VALIDÉ" (le pipeline ne s'arrête pas sur les erreurs
    // non bloquantes — c'est l'UI qui rend l'état métier).
    let mut docs = seed();
    // Mute INV-002 qui est VALIDÉ err=0 → simule une "VALIDÉ avec erreur"
    if let Some(d) = docs.iter_mut().find(|d| d.invoice_number.as_deref() == Some("INV-002")) {
        d.error_count = 1;
        d.errors = vec![pdp_trace::store::ErrorEntry {
            step: "annuaire-validation".into(),
            message: "Émetteur inconnu (test)".into(),
            detail: None,
            timestamp: "2026-04-15T11:00:00Z".into(),
        }];
    }
    let state = build_state(Arc::new(InMemoryTraceBackend::new(docs)));

    // Liste : la ligne INV-002 doit avoir le badge ERREUR (rouge)
    let (_, list) = get_html(state.clone(), "/ui/flows?siren=123456789").await;
    // Trouve le bloc HTML autour de INV-002 (raisonnablement local)
    let pos = list.find("INV-002").expect("INV-002 dans la liste");
    let window = &list[pos.saturating_sub(400)..(pos + 200).min(list.len())];
    assert!(
        window.contains("badge-error"),
        "INV-002 (err=1) doit avoir badge-error dans la liste ; window=\n{window}"
    );
    assert!(
        window.contains("ERREUR"),
        "INV-002 (err=1) doit afficher 'ERREUR' dans la liste"
    );
    assert!(
        !window.contains(">VALIDÉ<"),
        "Le statut brut VALIDÉ ne doit PAS être affiché quand error_count > 0"
    );

    // Détail : pareil + section Erreurs visible
    let (_, detail) = get_html(state, "/ui/flows/flow-ex-2?siren=123456789").await;
    assert!(detail.contains(r#"<span class="badge badge-error">ERREUR</span>"#));
    assert!(detail.contains("Émetteur inconnu (test)"));
}

#[tokio::test]
async fn test_filter_direction_emises_recues() {
    // direction=emises → seules les factures où le tenant est vendeur
    // direction=recues → seules celles où le tenant est acheteur
    let state = make_state();

    let (_st, body_em) =
        get_html(state.clone(), "/ui/flows?siren=123456789&direction=emises").await;
    assert!(body_em.contains("INV-001"), "émise");
    assert!(!body_em.contains("REC-PLO-001"), "reçue ne doit pas apparaître");
    assert!(!body_em.contains("REC-CE-014"));

    let (_st, body_rc) =
        get_html(state, "/ui/flows?siren=123456789&direction=recues").await;
    assert!(body_rc.contains("REC-PLO-001"));
    assert!(body_rc.contains("REC-CE-014"));
    assert!(!body_rc.contains("INV-001"), "émise ne doit pas apparaître");
}

#[tokio::test]
async fn test_filter_empty_strings_treated_as_no_filter() {
    // Le formulaire HTML soumet `?siren=X&direction=&status=&from=&to=`
    // (champs vides → `Some("")`). Cela ne doit PAS retourner une liste vide :
    // on doit voir tous les flux du tenant comme si aucun filtre n'était posé.
    let state = make_state();
    let (status, body) = get_html(
        state,
        "/ui/flows?siren=123456789&direction=&status=&from=&to=",
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // Toutes les factures du tenant doivent apparaître (idem default sans filtres)
    for inv in &[
        "INV-001", "INV-002", "INV-003", "INV-ERR", "INV-PEND", "INV-PJ",
        "REC-PLO-001", "REC-CE-014",
    ] {
        assert!(
            body.contains(inv),
            "{inv} doit apparaître malgré les params vides ; \
             body court = {}",
            &body[..400.min(body.len())]
        );
    }
}

#[tokio::test]
async fn test_filter_empty_status_then_real_status_works() {
    // Combinaison réelle : un user clique "Erreur" dans un formulaire dont
    // les autres champs sont vides → URL `?siren=X&direction=&status=ERREUR&from=&to=`
    let state = make_state();
    let (_status, body) = get_html(
        state,
        "/ui/flows?siren=123456789&direction=&status=ERREUR&from=&to=",
    )
    .await;
    assert!(body.contains("INV-ERR"), "ERREUR malgré direction= et from/to vides");
    assert!(!body.contains("INV-001"));
}

#[tokio::test]
async fn test_favicon_and_logo_routes() {
    let state = make_state();
    let app = build_api_router(state);

    for path in &["/favicon.ico", "/favicon.png", "/ui/static/ferrite-icon.png"] {
        let req = Request::builder().uri(*path).body(Body::empty()).unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(
            resp.status(),
            StatusCode::OK,
            "{path} doit répondre 200"
        );
        let ctype = resp
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();
        assert!(
            ctype.starts_with("image/"),
            "{path} doit servir une image, ctype={ctype}"
        );
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        assert!(!bytes.is_empty(), "{path} ne doit pas être vide");
    }
}
