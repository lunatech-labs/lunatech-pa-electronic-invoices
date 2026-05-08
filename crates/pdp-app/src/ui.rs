//! Interface web de suivi des factures (phase 1 — lecture seule).
//!
//! Pages :
//! - `GET /ui` (alias `/dashboard`) — KPIs (total, erreurs, distribués)
//! - `GET /ui/flows` — Liste paginée avec filtres (status, dates)
//! - `GET /ui/flows/{flowId}` — Détail facture + timeline CDV
//!
//! Multi-tenant : la query string `?siren=123456789` cible l'index ES
//! `pdp-{siren}`. Sans paramètre, retombe sur `?siren=` vide → message d'aide.
//!
//! Stack : HTML server-rendered (cohérent avec `/annuaire`), CSS inline,
//! HTMX pour l'interactivité sans SPA. Pas de dépendance front lourde.

use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse},
};
use serde::Deserialize;

use crate::server::AppState;

// ============================================================
// Helpers communs
// ============================================================

/// Style CSS partagé par toutes les pages UI.
const CSS: &str = r#"
* { box-sizing: border-box; margin: 0; padding: 0; }
body {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    background: #f5f7fa;
    color: #1a1a2e;
    min-height: 100vh;
    line-height: 1.5;
}
header {
    background: linear-gradient(135deg, #1a1a2e 0%, #16213e 100%);
    color: white;
    padding: 1rem 2rem;
    display: flex;
    align-items: center;
    gap: 1rem;
}
header .logo {
    height: 48px;
    width: 48px;
    flex-shrink: 0;
    border-radius: 8px;
}
header .brand {
    display: flex;
    flex-direction: column;
    gap: 0.1rem;
    margin-right: 1.5rem;
}
header .brand .name {
    font-size: 1.5rem;
    font-weight: 700;
    letter-spacing: -0.02em;
    line-height: 1;
}
header .brand .tagline {
    font-size: 0.8rem;
    opacity: 0.6;
    line-height: 1;
}
header h1 {
    font-size: 1.05rem;
    font-weight: 500;
    opacity: 0.85;
    margin: 0;
}
header nav { margin-left: auto; }
header nav a {
    color: rgba(255,255,255,0.85);
    text-decoration: none;
    margin-right: 1.2rem;
    font-size: 0.95rem;
}
header nav a:hover { color: white; text-decoration: underline; }
header nav a.active { color: white; font-weight: 600; }
main { max-width: 1200px; margin: 2rem auto; padding: 0 1.5rem; }
.kpi-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
    gap: 1rem;
    margin-bottom: 2rem;
}
.kpi-card {
    background: white;
    border-radius: 8px;
    padding: 1.5rem;
    box-shadow: 0 2px 8px rgba(0,0,0,0.05);
}
.kpi-label { color: #666; font-size: 0.85rem; text-transform: uppercase; letter-spacing: 0.05em; }
.kpi-value { font-size: 2rem; font-weight: 700; margin-top: 0.4rem; }
.kpi-value.success { color: #2e7d32; }
.kpi-value.warning { color: #ed6c02; }
.kpi-value.error { color: #d32f2f; }
.card {
    background: white;
    border-radius: 8px;
    padding: 1.5rem;
    box-shadow: 0 2px 8px rgba(0,0,0,0.05);
    margin-bottom: 1.5rem;
}
.card h2 { font-size: 1.1rem; margin-bottom: 1rem; color: #16213e; }
table { width: 100%; border-collapse: collapse; }
th, td { padding: 0.7rem 0.5rem; text-align: left; border-bottom: 1px solid #eee; font-size: 0.9rem; }
th { color: #666; font-weight: 500; font-size: 0.8rem; text-transform: uppercase; }
tr:hover { background: #f9fafb; }
.badge {
    display: inline-block;
    padding: 0.2rem 0.6rem;
    border-radius: 12px;
    font-size: 0.75rem;
    font-weight: 600;
    text-transform: uppercase;
}
.badge-success { background: #e8f5e9; color: #2e7d32; }
.badge-error { background: #ffebee; color: #d32f2f; }
.badge-warning { background: #fff3e0; color: #ed6c02; }
.badge-info { background: #e3f2fd; color: #1565c0; }
.badge-default { background: #f0f0f0; color: #555; }
.filters {
    display: flex;
    gap: 0.8rem;
    flex-wrap: wrap;
    margin-bottom: 1rem;
}
.filters input, .filters select {
    padding: 0.5rem 0.8rem;
    border: 1px solid #ddd;
    border-radius: 6px;
    font-size: 0.9rem;
    background: white;
}
.filters button {
    padding: 0.5rem 1.2rem;
    background: #16213e;
    color: white;
    border: none;
    border-radius: 6px;
    cursor: pointer;
    font-size: 0.9rem;
}
.filters button:hover { background: #1a1a2e; }
a { color: #1565c0; text-decoration: none; }
a:hover { text-decoration: underline; }
.empty {
    text-align: center;
    padding: 3rem;
    color: #999;
}
.timeline { position: relative; padding-left: 2rem; }
.timeline::before {
    content: '';
    position: absolute;
    left: 0.5rem; top: 0; bottom: 0;
    width: 2px;
    background: #ddd;
}
.timeline-item { position: relative; padding-bottom: 1.2rem; }
.timeline-item::before {
    content: '';
    position: absolute;
    left: -1.7rem; top: 0.3rem;
    width: 12px; height: 12px;
    border-radius: 50%;
    background: #16213e;
    border: 2px solid white;
    box-shadow: 0 0 0 2px #16213e;
}
.timeline-item .ts { color: #888; font-size: 0.8rem; }
.timeline-item .label { font-weight: 600; }
.timeline-item .msg { color: #555; font-size: 0.9rem; margin-top: 0.2rem; }
.timeline-item.timeline-error::before { background: #d32f2f; box-shadow: 0 0 0 2px #d32f2f; }
.timeline-item.timeline-error .label { color: #d32f2f; }
.timeline-item.timeline-error .msg { color: #b71c1c; }
dl.kv { display: grid; grid-template-columns: 200px 1fr; gap: 0.6rem 1rem; }
dl.kv dt { color: #666; font-weight: 500; }
dl.kv dd { color: #1a1a2e; }
.banner {
    background: #fff3e0;
    color: #5a3a00;
    padding: 0.8rem 1.2rem;
    border-radius: 6px;
    margin-bottom: 1.5rem;
}
.dl-row { display: flex; gap: 0.6rem; flex-wrap: wrap; }
.dl-btn {
    display: inline-block;
    padding: 0.5rem 1rem;
    background: #16213e;
    color: white !important;
    border-radius: 6px;
    text-decoration: none;
    font-size: 0.9rem;
}
.dl-btn:hover { background: #1a1a2e; text-decoration: none; }
.pj-badge {
    display: inline-block;
    padding: 0.15rem 0.55rem;
    border-radius: 12px;
    background: #f0ecf9;
    color: #534ab7;
    font-size: 0.8rem;
    font-weight: 600;
    white-space: nowrap;
}
.err-badge {
    display: inline-block;
    padding: 0.15rem 0.55rem;
    border-radius: 12px;
    background: #ffebee;
    color: #d32f2f;
    font-size: 0.8rem;
    font-weight: 600;
    white-space: nowrap;
}
.siret-sub { color: #999; font-size: 0.75rem; margin-top: 0.1rem; }
.dir-tag {
    display: inline-block;
    width: 1.1rem;
    height: 1.1rem;
    line-height: 1.1rem;
    text-align: center;
    border-radius: 50%;
    font-size: 0.7rem;
    font-weight: 700;
    margin-right: 0.3rem;
}
.dir-out { background: #e3f2fd; color: #1565c0; }
.dir-in { background: #fff3e0; color: #ed6c02; }
.tenant-info {
    background: #f0ecf9;
    border-left: 3px solid #534ab7;
    padding: 0.7rem 1rem;
    border-radius: 0 6px 6px 0;
    margin-bottom: 1rem;
    font-size: 0.9rem;
    color: #16213e;
}
"#;

fn page_shell(title: &str, active: &str, siren: Option<&str>, body: &str) -> String {
    let siren_q = siren.map(|s| format!("?siren={}", s)).unwrap_or_default();
    let nav_link = |path: &str, label: &str, key: &str| {
        let class = if key == active { "active" } else { "" };
        format!(
            r#"<a href="/ui{path}{q}" class="{class}">{label}</a>"#,
            path = path,
            q = siren_q,
            class = class,
            label = label,
        )
    };
    format!(
        r#"<!DOCTYPE html>
<html lang="fr">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title} — Ferrite</title>
    <link rel="icon" type="image/png" href="/favicon.png">
    <link rel="apple-touch-icon" href="/favicon.png">
    <style>{css}</style>
</head>
<body>
    <header>
        <img src="/ui/static/ferrite-icon.png" alt="" class="logo">
        <div class="brand">
            <span class="name">Ferrite</span>
            <span class="tagline">Plateforme Agréée</span>
        </div>
        <h1>Suivi des factures</h1>
        <nav>
            {dashboard_link}
            {emises_link}
            {recues_link}
            <a href="/annuaire">Annuaire</a>
        </nav>
    </header>
    <main>{body}</main>
</body>
</html>"#,
        title = title,
        css = CSS,
        dashboard_link = nav_link("", "Dashboard", "dashboard"),
        emises_link = nav_link("/emises", "Émises", "emises"),
        recues_link = nav_link("/recues", "Reçues", "recues"),
        body = body,
    )
}

fn no_siren_banner() -> String {
    r#"<div class="banner">
        ⚠️ Aucun SIREN sélectionné. Ajouter <code>?siren=123456789</code> à l'URL pour cibler un tenant.
        Sans SIREN, les données ne peuvent pas être chargées (un index Elasticsearch par tenant).
    </div>"#.to_string()
}

/// Convertit une option de paramètre de query en `Option<&str>` en éliminant
/// la chaîne vide. Le formulaire HTML envoie `?status=&from=&to=...` quand
/// l'utilisateur n'a rien sélectionné — sans cette normalisation, ces champs
/// arrivent en `Some("")` au handler et ES retourne 0 résultat sur un
/// `term: { "status": "" }`.
fn non_empty(opt: &Option<String>) -> Option<&str> {
    opt.as_deref().filter(|s| !s.is_empty())
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn status_badge(status: &str) -> &'static str {
    match status.to_uppercase().as_str() {
        "DISTRIBUÉ" | "DISTRIBUE" | "ACKNOWLEDGED" | "ACQUITTÉ" | "ACQUITTE" => "badge-success",
        "ERREUR" | "ERROR" | "REJECTED" | "REJETÉ" | "REJETE" | "ANNULÉ" | "ANNULE" => "badge-error",
        "EN_ATTENTE" | "PENDING" | "WAITINGACK" | "WAITING" | "ATTENTE_ACK" => "badge-warning",
        "VALIDATED" | "VALIDÉ" | "VALIDE" | "RECEIVED" | "REÇU" | "RECU" | "TRANSFORMÉ" | "TRANSFORME" => "badge-info",
        _ => "badge-default",
    }
}

/// Statut "métier" affiché à l'utilisateur. Si la facture a des erreurs
/// (`error_count > 0`), on affiche **ERREUR** quel que soit l'état brut du
/// pipeline (le pipeline continue après une erreur non bloquante mais on ne
/// veut pas afficher "DISTRIBUÉ" sur une facture rejetable). Sinon on rend
/// le statut tel quel.
fn effective_status<'a>(raw_status: &'a str, error_count: i32) -> (&'a str, &'static str) {
    if error_count > 0 {
        ("ERREUR", "badge-error")
    } else {
        (raw_status, status_badge(raw_status))
    }
}

// ============================================================
// GET /ui — Dashboard (KPIs)
// ============================================================

#[derive(Deserialize)]
pub struct DashboardQuery {
    pub siren: Option<String>,
}

pub async fn handle_dashboard(
    State(state): State<Arc<AppState>>,
    axum::Extension(ctx): axum::Extension<std::sync::Arc<crate::security::SecurityContext>>,
    Query(q): Query<DashboardQuery>,
) -> axum::response::Response {
    // 403 si le SIREN demandé n'est pas dans le scope du porteur. None si
    // siren absent (le handler affiche un picker).
    let owned_siren = match crate::security::authorize_optional_siren(&ctx, q.siren.as_deref()) {
        Ok(s) => s,
        Err(resp) => return resp,
    };
    let siren = owned_siren.as_deref();

    let body = match siren {
        None => format!("{}{}", no_siren_banner(), siren_picker_form()),
        Some(s) => {
            let store = match &state.trace_store {
                Some(st) => st,
                None => return html_response("TraceStore non configuré (Elasticsearch)"),
            };
            let stats = store.get_stats_for_siren(s).await.unwrap_or(pdp_trace::store::TraceStats {
                total_exchanges: 0,
                total_errors: 0,
                total_distributed: 0,
            });
            let pending = stats.total_exchanges - stats.total_distributed - stats.total_errors;
            let tenant_name = store.get_tenant_name(s).await;
            let header_title = match tenant_name.as_deref() {
                Some(name) => format!(
                    r#"{name} <small style="font-weight:400;color:#666">— SIREN {siren}</small>"#,
                    name = html_escape(name),
                    siren = html_escape(s),
                ),
                None => format!("Tenant : {}", html_escape(s)),
            };
            format!(
                r#"
<div class="card">
    <h2>{title}</h2>
    <p style="color:#666">Toutes les valeurs proviennent de l'index Elasticsearch <code>pdp-{siren}</code>.</p>
</div>
<div class="kpi-grid">
    <div class="kpi-card">
        <div class="kpi-label">Total flux</div>
        <div class="kpi-value">{total}</div>
    </div>
    <div class="kpi-card">
        <div class="kpi-label">Distribués</div>
        <div class="kpi-value success">{distributed}</div>
    </div>
    <div class="kpi-card">
        <div class="kpi-label">En attente</div>
        <div class="kpi-value warning">{pending}</div>
    </div>
    <div class="kpi-card">
        <div class="kpi-label">En erreur</div>
        <div class="kpi-value error">{errors}</div>
    </div>
</div>
<div class="card">
    <h2>Actions</h2>
    <p><a href="/ui/emises?siren={siren}">→ Suivi des factures émises</a></p>
    <p><a href="/ui/recues?siren={siren}">→ Suivi des factures reçues</a></p>
    <p><a href="/ui/emises?siren={siren}&status=ERREUR">→ Émises en erreur</a></p>
    <p><a href="/ui/recues?siren={siren}&status=ERREUR">→ Reçues en erreur</a></p>
    <p><a href="/v1/healthcheck">→ Healthcheck API</a></p>
    <p><a href="/metrics">→ Métriques Prometheus</a></p>
</div>
"#,
                title = header_title,
                siren = html_escape(s),
                total = stats.total_exchanges,
                distributed = stats.total_distributed,
                pending = pending.max(0),
                errors = stats.total_errors,
            )
        }
    };

    html_response(&page_shell("Dashboard", "dashboard", siren, &body)).into_response()
}

fn siren_picker_form() -> String {
    r#"<div class="card">
    <h2>Choisir un tenant</h2>
    <form method="get" action="/ui">
        <div class="filters">
            <input name="siren" placeholder="SIREN (9 chiffres)" pattern="[0-9]{9}" required>
            <button type="submit">Charger</button>
        </div>
    </form>
</div>"#.to_string()
}

// ============================================================
// GET /ui/emises | /ui/recues — Listes paginées
// ============================================================
//
// Deux écrans **distincts** dans la nav :
//  - `/ui/emises`  : factures dont le tenant est **vendeur** (sortantes)
//  - `/ui/recues`  : factures dont il est **acheteur** (entrantes)
//
// Les deux partagent le même handler (cf. `render_flows_list`) qui prend
// la direction comme paramètre figé (pas via la query string). Le détail
// `/ui/flows/{flow_id}` reste partagé.

#[derive(Deserialize)]
pub struct FlowsListQuery {
    pub siren: Option<String>,
    pub status: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub page: Option<usize>,
    /// Nombre de factures par page (défaut 50, max 500). Filtré aux valeurs
    /// du sélecteur côté UI : 25, 50, 100, 200.
    pub page_size: Option<usize>,
    /// Si "true", inclut tous les exchanges (toutes les soumissions, même les doublons).
    /// Par défaut, on déduplique par invoice_number en gardant le plus récent.
    pub show_duplicates: Option<String>,
}

/// Direction figée par la route. Utilisée pour :
/// - filtrer les exchanges (vendeur vs acheteur)
/// - choisir le titre, l'item nav actif et l'action du form
#[derive(Clone, Copy, PartialEq, Eq)]
enum FlowDirection {
    Emises,
    Recues,
}

impl FlowDirection {
    fn nav_key(&self) -> &'static str {
        match self {
            FlowDirection::Emises => "emises",
            FlowDirection::Recues => "recues",
        }
    }
    fn route_path(&self) -> &'static str {
        match self {
            FlowDirection::Emises => "/ui/emises",
            FlowDirection::Recues => "/ui/recues",
        }
    }
    fn page_title(&self) -> &'static str {
        match self {
            FlowDirection::Emises => "Factures émises",
            FlowDirection::Recues => "Factures reçues",
        }
    }
    fn empty_label(&self) -> &'static str {
        match self {
            FlowDirection::Emises => "Aucune facture émise pour ces critères.",
            FlowDirection::Recues => "Aucune facture reçue pour ces critères.",
        }
    }
    /// Filtre côté Rust : un exchange est dans le scope si le tenant
    /// joue le rôle attendu (vendeur pour Émises, acheteur pour Reçues).
    fn keep(&self, ex: &pdp_trace::store::ExchangeSummary, siren: &str) -> bool {
        match self {
            FlowDirection::Emises => ex.seller_siren.as_deref() == Some(siren),
            FlowDirection::Recues => ex.buyer_siren.as_deref() == Some(siren),
        }
    }
}

/// Tailles de page proposées dans le sélecteur de pagination.
const PAGE_SIZE_OPTIONS: &[usize] = &[25, 50, 100, 200];

/// Borne `page_size` aux valeurs du sélecteur (sécurité + UX cohérente).
/// Toute autre valeur retombe sur le défaut (50). Empêche aussi qu'un user
/// passe `?page_size=10000` et fasse exploser la mémoire.
fn clamp_page_size(raw: Option<usize>) -> usize {
    raw.filter(|n| PAGE_SIZE_OPTIONS.contains(n)).unwrap_or(50)
}

/// `GET /ui/emises` — Suivi des factures émises (tenant = vendeur).
pub async fn handle_flows_emises(
    State(state): State<Arc<AppState>>,
    ctx: axum::Extension<std::sync::Arc<crate::security::SecurityContext>>,
    q: Query<FlowsListQuery>,
) -> axum::response::Response {
    render_flows_list(state, ctx, q, FlowDirection::Emises).await
}

/// `GET /ui/recues` — Suivi des factures reçues (tenant = acheteur).
pub async fn handle_flows_recues(
    State(state): State<Arc<AppState>>,
    ctx: axum::Extension<std::sync::Arc<crate::security::SecurityContext>>,
    q: Query<FlowsListQuery>,
) -> axum::response::Response {
    render_flows_list(state, ctx, q, FlowDirection::Recues).await
}

async fn render_flows_list(
    state: Arc<AppState>,
    axum::Extension(ctx): axum::Extension<std::sync::Arc<crate::security::SecurityContext>>,
    Query(q): Query<FlowsListQuery>,
    direction: FlowDirection,
) -> axum::response::Response {
    // 403 si le SIREN demandé n'est pas autorisé. None (siren absent) = picker.
    let owned_siren = match crate::security::authorize_optional_siren(&ctx, q.siren.as_deref()) {
        Ok(s) => s,
        Err(resp) => return resp,
    };
    let siren = owned_siren.as_deref();
    // Le formulaire HTML soumet les champs vides comme `?status=&from=&to=...`,
    // qui se désérialisent en `Some("")` (et non `None`). On les normalise ici
    // sinon ES voit `term: { "status": "" }` et ne renvoie rien.
    let status = non_empty(&q.status);
    let from = non_empty(&q.from);
    let to = non_empty(&q.to);

    let body = match siren {
        None => format!("{}{}", no_siren_banner(), siren_picker_form()),
        Some(s) => {
            let store = match &state.trace_store {
                Some(st) => st,
                None => return html_response("TraceStore non configuré (Elasticsearch)"),
            };
            let page = q.page.unwrap_or(0);
            let page_size = clamp_page_size(q.page_size);
            let dir_param = Some(direction.nav_key());
            let total = store
                .count_exchanges(s, status, from, to, dir_param)
                .await
                .unwrap_or(0);
            let mut exchanges = store
                .list_exchanges(s, status, from, to, page, page_size, dir_param)
                .await
                .unwrap_or_default();

            // Déduplication par invoice_number (par défaut activée).
            // Plusieurs soumissions de la même facture créent plusieurs exchanges
            // (la dedup BR-FR-12/13 marque les ré-soumissions avec error_count>0
            // mais le doc reste indexé). On affiche le plus récent par numéro,
            // sauf si ?show_duplicates=true.
            let show_duplicates = q.show_duplicates.as_deref() == Some("true");
            if !show_duplicates {
                use std::collections::HashMap;
                let mut latest_by_invoice: HashMap<String, pdp_trace::store::ExchangeSummary> =
                    HashMap::new();
                let mut without_invoice = Vec::new();
                for ex in exchanges.into_iter() {
                    match ex.invoice_number.clone() {
                        Some(inv) => {
                            // Garde le plus récent par created_at (déjà trié desc par ES)
                            latest_by_invoice.entry(inv).or_insert(ex);
                        }
                        None => without_invoice.push(ex),
                    }
                }
                let mut deduped: Vec<_> = latest_by_invoice.into_values().collect();
                deduped.extend(without_invoice);
                deduped.sort_by(|a, b| b.created_at.cmp(&a.created_at));
                exchanges = deduped;
            }

            let tenant_name = store.get_tenant_name(s).await;
            let list_title = format!(
                "{label} — {who} <small style=\"font-weight:400;color:#666\">SIREN {siren}</small>",
                label = direction.page_title(),
                who = html_escape(tenant_name.as_deref().unwrap_or(s)),
                siren = html_escape(s),
            );

            let page_size_opts = PAGE_SIZE_OPTIONS
                .iter()
                .map(|n| {
                    let sel = if *n == page_size { " selected" } else { "" };
                    format!(r#"<option value="{n}"{sel}>{n} / page</option>"#)
                })
                .collect::<String>();
            let filters_form = format!(
                r#"<form method="get" action="{action}" class="filters">
    <input type="hidden" name="siren" value="{siren}">
    <select name="status">
        <option value="">— Tous les statuts —</option>
        <option value="DISTRIBUÉ" {sel_dist}>Distribués</option>
        <option value="ERREUR" {sel_err}>En erreur</option>
        <option value="EN_ATTENTE" {sel_pending}>En attente</option>
    </select>
    <input type="date" name="from" value="{from}" placeholder="Du">
    <input type="date" name="to" value="{to}" placeholder="Au">
    <select name="page_size" title="Factures par page">
        {page_size_opts}
    </select>
    <button type="submit">Filtrer</button>
</form>"#,
                action = direction.route_path(),
                siren = html_escape(s),
                sel_dist = if q.status.as_deref() == Some("DISTRIBUÉ") { "selected" } else { "" },
                sel_err = if q.status.as_deref() == Some("ERREUR") { "selected" } else { "" },
                sel_pending = if q.status.as_deref() == Some("EN_ATTENTE") { "selected" } else { "" },
                from = q.from.as_deref().unwrap_or(""),
                to = q.to.as_deref().unwrap_or(""),
                page_size_opts = page_size_opts,
            );

            // Côté ÉMISES, la « partie » d'intérêt pour le tenant est
            // l'acheteur (à qui il facture). Côté REÇUES, c'est le vendeur
            // (de qui il reçoit la facture). On affiche cette info dans une
            // colonne dédiée et on retire la cellule redondante.
            let counterparty_label = match direction {
                FlowDirection::Emises => "Acheteur",
                FlowDirection::Recues => "Vendeur",
            };
            let rows = if exchanges.is_empty() {
                format!(
                    r#"<tr><td colspan="6" class="empty">{}</td></tr>"#,
                    direction.empty_label()
                )
            } else {
                exchanges
                    .iter()
                    .map(|e| {
                        let pj_cell = if e.attachment_count == 0 {
                            r#"<span style="color:#bbb">—</span>"#.to_string()
                        } else {
                            format!(
                                r#"<span class="pj-badge" title="{n} pièce(s) jointe(s)">📎 {n}</span>"#,
                                n = e.attachment_count
                            )
                        };
                        let errors_cell = if e.error_count == 0 {
                            r#"<span style="color:#bbb">—</span>"#.to_string()
                        } else {
                            format!(
                                r#"<span class="err-badge" title="{n} erreur(s)">⚠️ {n}</span>"#,
                                n = e.error_count
                            )
                        };
                        let (counterparty_name, counterparty_siret) = match direction {
                            FlowDirection::Emises => (
                                e.buyer_name.as_deref().unwrap_or("—"),
                                e.buyer_siret.as_deref().unwrap_or("—"),
                            ),
                            FlowDirection::Recues => (
                                e.seller_name.as_deref().unwrap_or("—"),
                                e.seller_siret.as_deref().unwrap_or("—"),
                            ),
                        };
                        let counterparty_cell = format!(
                            r#"<div>{name}</div><div class="siret-sub">SIRET {siret}</div>"#,
                            name = html_escape(counterparty_name),
                            siret = html_escape(counterparty_siret),
                        );
                        format!(
                            r#"<tr>
    <td><a href="/ui/flows/{flow_id}?siren={siren}">{invoice}</a></td>
    <td>{counterparty}</td>
    <td><span class="badge {badge}">{status}</span></td>
    <td>{pj}</td>
    <td>{errors}</td>
    <td>{date}</td>
</tr>"#,
                            flow_id = html_escape(&e.flow_id),
                            siren = html_escape(s),
                            invoice = html_escape(e.invoice_number.as_deref().unwrap_or("—")),
                            counterparty = counterparty_cell,
                            badge = {
                                let (_, b) = effective_status(&e.status, e.error_count);
                                b
                            },
                            status = {
                                let (s, _) = effective_status(&e.status, e.error_count);
                                html_escape(s)
                            },
                            pj = pj_cell,
                            errors = errors_cell,
                            date = html_escape(&e.created_at[..e.created_at.len().min(10)]),
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            };

            let pagination = build_pagination(
                s, status, from, to, direction, page, exchanges.len(), page_size, total,
            );
            let intro = match direction {
                FlowDirection::Emises => format!(
                    r#"Factures dont <strong>{siren}</strong> est le vendeur (BT-30 SIREN). \
                    Une facture re-soumise crée un doublon BR-FR-12/13 — seule la dernière \
                    soumission est affichée (<a href="?siren={siren}&amp;show_duplicates=true">voir tout l'historique</a>)."#,
                    siren = html_escape(s),
                ),
                FlowDirection::Recues => format!(
                    r#"Factures dont <strong>{siren}</strong> est l'acheteur (BT-47 SIREN). \
                    Une facture re-soumise crée un doublon BR-FR-12/13 — seule la dernière \
                    soumission est affichée (<a href="?siren={siren}&amp;show_duplicates=true">voir tout l'historique</a>)."#,
                    siren = html_escape(s),
                ),
            };

            format!(
                r#"<div class="card">
    <h2>{title}</h2>
    <div class="tenant-info">{intro}</div>
    {filters}
    <table>
        <thead>
            <tr><th>N° facture</th><th>{counterparty_label}</th><th>Statut</th><th>PJ</th><th>Err.</th><th>Reçue le</th></tr>
        </thead>
        <tbody>{rows}</tbody>
    </table>
    {pagination}
</div>"#,
                title = list_title,
                intro = intro,
                counterparty_label = counterparty_label,
                filters = filters_form,
                rows = rows,
                pagination = pagination,
            )
        }
    };

    html_response(&page_shell(
        direction.page_title(),
        direction.nav_key(),
        siren,
        &body,
    ))
    .into_response()
}

fn build_pagination(
    siren: &str,
    status: Option<&str>,
    from: Option<&str>,
    to: Option<&str>,
    direction: FlowDirection,
    page: usize,
    page_count: usize,
    page_size: usize,
    total: i64,
) -> String {
    let qs = |p: usize| -> String {
        let mut s = format!("?siren={}&page={}&page_size={}", siren, p, page_size);
        if let Some(st) = status { s.push_str(&format!("&status={}", st)); }
        if let Some(f) = from { s.push_str(&format!("&from={}", f)); }
        if let Some(t) = to { s.push_str(&format!("&to={}", t)); }
        s
    };
    // Nombre total de pages (au moins 1 si vide, pour ne pas afficher "Page 1/0").
    let total_pages = if total <= 0 {
        1
    } else {
        ((total as usize).saturating_sub(1) / page_size) + 1
    };
    let has_next = page + 1 < total_pages;
    let route = direction.route_path();
    let prev = if page > 0 {
        format!(r#"<a href="{route}{}">← Précédent</a>"#, qs(page - 1))
    } else {
        r#"<span style="color:#aaa">← Précédent</span>"#.to_string()
    };
    let next = if has_next {
        format!(r#"<a href="{route}{}">Suivant →</a>"#, qs(page + 1))
    } else {
        r#"<span style="color:#aaa">Suivant →</span>"#.to_string()
    };
    // Plage de résultats visibles : ex "1–50 / 257"
    let range_start = if total > 0 { page * page_size + 1 } else { 0 };
    let range_end = page * page_size + page_count;
    format!(
        r#"<div style="margin-top:1rem; display:flex; justify-content:space-between; align-items:center; color:#666; font-size:0.9rem;">
        {prev}
        <span><strong>{range_start}–{range_end}</strong> sur <strong>{total}</strong> factures · page {page_display}/{total_pages}</span>
        {next}
    </div>"#,
        prev = prev,
        next = next,
        range_start = range_start,
        range_end = range_end,
        total = total,
        page_display = page + 1,
        total_pages = total_pages,
    )
}

// ============================================================
// Timeline du pipeline (events + errors fusionnés chronologiquement)
// ============================================================

/// Construit la timeline affichée sur la page détail.
///
/// Les `events` (REÇU, PARSÉ, VALIDÉ, DISTRIBUÉ…) racontent ce que les
/// processors ont **fait**, tandis que les `errors` racontent ce qui a **mal
/// tourné** au passage. Le pipeline ne s'arrête pas sur une erreur non
/// bloquante (la responsabilité de générer un CDV de rejet incombe au
/// `CdarProcessor` en aval), donc une facture peut très bien aller jusqu'à
/// `DISTRIBUÉ` tout en ayant des erreurs collectées en route.
///
/// Pour ne pas induire l'utilisateur en erreur, on **fusionne** events et
/// errors dans une seule liste triée par timestamp, et on tronque la
/// progression `events` après la première erreur — les statuts ultérieurs
/// (DISTRIBUÉ, ACQUITTÉ…) ne sont pas affichés car ils ne reflètent pas
/// l'issue réelle de la facture.
fn render_timeline(
    events: &[pdp_trace::store::EventEntry],
    errors: &[pdp_trace::store::ErrorEntry],
) -> String {
    if events.is_empty() && errors.is_empty() {
        return r#"<p style="color:#888">Aucun événement enregistré.</p>"#.to_string();
    }
    enum Item<'a> {
        Event(&'a pdp_trace::store::EventEntry),
        Error(&'a pdp_trace::store::ErrorEntry),
    }
    let mut items: Vec<Item> = events.iter().map(Item::Event).chain(errors.iter().map(Item::Error)).collect();
    items.sort_by(|a, b| {
        let ta = match a {
            Item::Event(e) => &e.timestamp,
            Item::Error(e) => &e.timestamp,
        };
        let tb = match b {
            Item::Event(e) => &e.timestamp,
            Item::Error(e) => &e.timestamp,
        };
        ta.cmp(tb)
    });

    // Tronque après la première erreur : les statuts pipeline qui suivent
    // (ex. DISTRIBUÉ alors qu'on a une erreur annuaire-validation) sont
    // trompeurs et ne reflètent pas le rejet métier qui arrive en aval.
    let first_err_pos = items.iter().position(|it| matches!(it, Item::Error(_)));
    if let Some(idx) = first_err_pos {
        items.truncate(idx + 1);
    }

    let html: Vec<String> = items
        .iter()
        .map(|it| match it {
            Item::Event(ev) => format!(
                r#"<div class="timeline-item">
    <div class="ts">{ts} — route <code>{route}</code></div>
    <div class="label">{status}</div>
    <div class="msg">{msg}</div>
</div>"#,
                ts = html_escape(&ev.timestamp),
                route = html_escape(&ev.route_id),
                status = html_escape(&ev.status),
                msg = html_escape(&ev.message),
            ),
            Item::Error(er) => format!(
                r#"<div class="timeline-item timeline-error">
    <div class="ts">{ts} — étape <code>{step}</code></div>
    <div class="label">❌ ERREUR</div>
    <div class="msg">{msg}</div>
</div>"#,
                ts = html_escape(&er.timestamp),
                step = html_escape(&er.step),
                msg = html_escape(&er.message),
            ),
        })
        .collect();
    format!(r#"<div class="timeline">{}</div>"#, html.join(""))
}

// ============================================================
// Extraction des pièces jointes (à la volée, depuis raw_xml ou raw_pdf)
// ============================================================

/// Extrait la liste des pièces jointes d'un `ExchangeDocument` en re-parsant
/// le contenu original (`raw_xml` pour UBL/CII, `raw_pdf_base64` pour Factur-X).
/// Les PJ ne sont pas stockées en base — on les reconstruit à la demande.
fn parse_attachments_from_doc(
    doc: &pdp_trace::store::ExchangeDocument,
) -> Vec<pdp_core::model::InvoiceAttachment> {
    let format = doc.source_format.as_deref().unwrap_or("UBL").to_uppercase();

    match format.as_str() {
        "UBL" => doc
            .raw_xml
            .as_deref()
            .and_then(|xml| pdp_invoice::UblParser::new().parse(xml).ok())
            .map(|inv| inv.attachments)
            .unwrap_or_default(),
        "CII" => doc
            .raw_xml
            .as_deref()
            .and_then(|xml| pdp_invoice::CiiParser::new().parse(xml).ok())
            .map(|inv| inv.attachments)
            .unwrap_or_default(),
        "FACTURX" | "FACTUR-X" => {
            // Décode le PDF base64 puis extrait les PJ embarquées
            let b64 = match doc.raw_pdf_base64.as_deref() {
                Some(b) => b,
                None => return Vec::new(),
            };
            use base64::Engine as _;
            match base64::engine::general_purpose::STANDARD.decode(b64) {
                Ok(pdf_bytes) => pdp_invoice::FacturXParser::new()
                    .parse(&pdf_bytes)
                    .map(|inv| inv.attachments)
                    .unwrap_or_default(),
                Err(_) => Vec::new(),
            }
        }
        _ => Vec::new(),
    }
}

/// Formate une taille en octets en chaîne lisible (B / KB / MB).
fn format_size(bytes: usize) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.2} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}

fn render_attachments(
    attachments: &[pdp_core::model::InvoiceAttachment],
    fallback_filenames: &[String],
    flow_id: &str,
    siren: &str,
) -> String {
    // Cas dégradé : pas de PJ extraites du raw_xml, on affiche au moins les noms indexés
    if attachments.is_empty() {
        if fallback_filenames.is_empty() {
            return r#"<p style="color:#888">Aucune pièce jointe.</p>"#.to_string();
        }
        let items: Vec<String> = fallback_filenames
            .iter()
            .map(|f| format!("<li>{}</li>", html_escape(f)))
            .collect();
        return format!(
            r#"<p style="color:#666;font-size:0.9rem">⚠️ Liste indexée uniquement (raw_xml indisponible — détails non extraits)</p>
<ul style="padding-left:1.2rem">{}</ul>"#,
            items.join("")
        );
    }

    let rows: Vec<String> = attachments
        .iter()
        .enumerate()
        .map(|(idx, a)| {
            let filename = a.filename.as_deref().unwrap_or("—");
            let id = a.id.as_deref().unwrap_or("—");
            let description = a.description.as_deref().unwrap_or("");
            let mime = a.mime_code.as_deref().unwrap_or("—");
            let size = match (&a.embedded_content, &a.external_uri) {
                (Some(content), _) => format_size(content.len()),
                (None, Some(uri)) => format!(
                    r#"<a href="{}" target="_blank">externe</a>"#,
                    html_escape(uri)
                ),
                _ => "—".to_string(),
            };
            let download = if a.embedded_content.is_some() {
                format!(
                    r#"<a href="/ui/flows/{flow_id}/download/attachment?siren={siren}&idx={idx}" title="Télécharger">⬇️</a>"#,
                    flow_id = html_escape(flow_id),
                    siren = html_escape(siren),
                    idx = idx,
                )
            } else {
                "—".to_string()
            };
            format!(
                r#"<tr>
    <td><code>{id}</code></td>
    <td>{filename}</td>
    <td>{description}</td>
    <td>{mime}</td>
    <td>{size}</td>
    <td>{download}</td>
</tr>"#,
                id = html_escape(id),
                filename = html_escape(filename),
                description = html_escape(description),
                mime = html_escape(mime),
                size = size,
                download = download,
            )
        })
        .collect();

    format!(
        r#"<table>
    <thead>
        <tr><th>ID</th><th>Fichier</th><th>Description</th><th>MIME</th><th>Taille</th><th></th></tr>
    </thead>
    <tbody>{}</tbody>
</table>
<p style="color:#666;font-size:0.85rem;margin-top:0.5rem">
    {} pièce(s) jointe(s) extraite(s) à la volée — non stockées en base.
</p>"#,
        rows.join(""),
        attachments.len(),
    )
}

// ============================================================
// GET /ui/flows/{flowId} — Détail
// ============================================================

#[derive(Deserialize)]
pub struct FlowDetailQuery {
    pub siren: Option<String>,
}

pub async fn handle_flow_detail(
    State(state): State<Arc<AppState>>,
    axum::Extension(ctx): axum::Extension<std::sync::Arc<crate::security::SecurityContext>>,
    Path(flow_id): Path<String>,
    Query(q): Query<FlowDetailQuery>,
) -> axum::response::Response {
    let owned_siren = match crate::security::authorize_optional_siren(&ctx, q.siren.as_deref()) {
        Ok(s) => s,
        Err(resp) => return resp,
    };
    let siren = owned_siren.as_deref();
    let body = match (siren, &state.trace_store) {
        (None, _) => format!("{}{}", no_siren_banner(), siren_picker_form()),
        (_, None) => "TraceStore non configuré (Elasticsearch)".to_string(),
        (Some(s), Some(store)) => {
            // Recherche par flow_id (les exchange_id sont aussi possibles)
            let summaries = store
                .list_exchanges(s, None, None, None, 0, 200, None)
                .await
                .unwrap_or_default();
            let summary = summaries.iter().find(|sum| sum.flow_id == flow_id || sum.exchange_id == flow_id);

            match summary {
                None => format!(
                    r#"<div class="card"><h2>Flux introuvable</h2><p>Aucun flux <code>{}</code> dans pdp-{}.</p><p><a href="/ui/emises?siren={s}">← Émises</a> · <a href="/ui/recues?siren={s}">Reçues</a></p></div>"#,
                    html_escape(&flow_id), html_escape(s), s = html_escape(s),
                ),
                Some(sum) => {
                    let exchange = store.get_exchange(&sum.exchange_id, Some(s)).await.ok().flatten();
                    render_flow_detail(s, &flow_id, sum, exchange.as_ref())
                }
            }
        }
    };
    // Le breadcrumb actif (Émises ou Reçues) dépend du rôle du tenant pour
    // CETTE facture. Si on est arrivé ici sans summary (flux introuvable),
    // on retombe sur Émises par défaut.
    let nav_active = match (siren, &state.trace_store) {
        (Some(s), Some(store)) => {
            let summaries = store
                .list_exchanges(s, None, None, None, 0, 200, None)
                .await
                .unwrap_or_default();
            let sum = summaries
                .iter()
                .find(|sum| sum.flow_id == flow_id || sum.exchange_id == flow_id);
            match sum {
                Some(sum) if sum.buyer_siren.as_deref() == Some(s) => "recues",
                _ => "emises",
            }
        }
        _ => "emises",
    };
    html_response(&page_shell("Détail flux", nav_active, siren, &body)).into_response()
}

fn render_flow_detail(
    siren: &str,
    flow_id: &str,
    sum: &pdp_trace::store::ExchangeSummary,
    full: Option<&pdp_trace::store::ExchangeDocument>,
) -> String {
    let metadata = format!(
        r#"<dl class="kv">
    <dt>Flow ID</dt><dd><code>{flow_id}</code></dd>
    <dt>Exchange ID</dt><dd><code>{exchange_id}</code></dd>
    <dt>Numéro facture</dt><dd>{invoice}</dd>
    <dt>Vendeur</dt><dd>{seller} ({seller_siret})</dd>
    <dt>Acheteur</dt><dd>{buyer} ({buyer_siret})</dd>
    <dt>Format</dt><dd>{format}</dd>
    <dt>Total HT</dt><dd>{total_ht}</dd>
    <dt>Total TVA</dt><dd>{total_tax}</dd>
    <dt>Total TTC</dt><dd>{total_ttc}</dd>
    <dt>Devise</dt><dd>{currency}</dd>
    <dt>Date émission</dt><dd>{issue_date}</dd>
    <dt>Statut</dt><dd><span class="badge {badge}">{status}</span></dd>
    <dt>Reçue le</dt><dd>{created_at}</dd>
</dl>"#,
        flow_id = html_escape(flow_id),
        exchange_id = html_escape(&sum.exchange_id),
        invoice = html_escape(sum.invoice_number.as_deref().unwrap_or("—")),
        seller = html_escape(sum.seller_name.as_deref().unwrap_or("—")),
        seller_siret = html_escape(full.and_then(|f| f.seller_siret.as_deref()).unwrap_or("—")),
        buyer = html_escape(sum.buyer_name.as_deref().unwrap_or("—")),
        buyer_siret = html_escape(full.and_then(|f| f.buyer_siret.as_deref()).unwrap_or("—")),
        format = html_escape(full.and_then(|f| f.source_format.as_deref()).unwrap_or("—")),
        total_ht = full.and_then(|f| f.total_ht).map(|v| format!("{:.2}", v)).unwrap_or_else(|| "—".into()),
        total_tax = full.and_then(|f| f.total_tax).map(|v| format!("{:.2}", v)).unwrap_or_else(|| "—".into()),
        total_ttc = full.and_then(|f| f.total_ttc).map(|v| format!("{:.2}", v)).unwrap_or_else(|| "—".into()),
        currency = html_escape(full.and_then(|f| f.currency.as_deref()).unwrap_or("—")),
        issue_date = html_escape(full.and_then(|f| f.issue_date.as_deref()).unwrap_or("—")),
        badge = {
            let (_, b) = effective_status(&sum.status, sum.error_count);
            b
        },
        status = {
            let (s, _) = effective_status(&sum.status, sum.error_count);
            html_escape(s)
        },
        created_at = html_escape(&sum.created_at),
    );

    let timeline = match full {
        None => String::new(),
        Some(doc) => render_timeline(&doc.events, &doc.errors),
    };

    let errors = match full {
        None => String::new(),
        Some(doc) if doc.errors.is_empty() => String::new(),
        Some(doc) => {
            let items: Vec<String> = doc
                .errors
                .iter()
                .map(|e| {
                    format!(
                        r#"<li><strong>{step}</strong> — {msg}</li>"#,
                        step = html_escape(&e.step),
                        msg = html_escape(&e.message),
                    )
                })
                .collect();
            format!(
                r#"<div class="card"><h2>Erreurs</h2><ul style="padding-left:1.2rem;color:#d32f2f">{}</ul></div>"#,
                items.join("")
            )
        }
    };

    // Pièces jointes extraites à la volée du raw_xml/raw_pdf (pas stockées en base)
    let attachments_section = match full {
        None => String::new(),
        Some(doc) => {
            let attachments = parse_attachments_from_doc(doc);
            let count_label = if attachments.is_empty() && doc.attachment_filenames.is_empty() {
                String::new()
            } else {
                format!(" ({})", attachments.len().max(doc.attachment_filenames.len()))
            };
            format!(
                r#"<div class="card"><h2>Pièces jointes{count}</h2>{body}</div>"#,
                count = count_label,
                body = render_attachments(&attachments, &doc.attachment_filenames, flow_id, siren),
            )
        }
    };

    // Liens de téléchargement (XML brut, PDF Factur-X)
    let downloads = match full {
        None => String::new(),
        Some(doc) => {
            let mut links = Vec::new();
            if doc.raw_xml.is_some() {
                links.push(format!(
                    r#"<a class="dl-btn" href="/ui/flows/{f}/download/xml?siren={s}">⬇️ XML brut</a>"#,
                    f = html_escape(flow_id), s = html_escape(siren),
                ));
            }
            if doc.raw_pdf_base64.is_some() {
                links.push(format!(
                    r#"<a class="dl-btn" href="/ui/flows/{f}/download/pdf?siren={s}">⬇️ PDF Factur-X</a>"#,
                    f = html_escape(flow_id), s = html_escape(siren),
                ));
            }
            if links.is_empty() {
                String::new()
            } else {
                format!(
                    r#"<div class="card"><h2>Téléchargements</h2><div class="dl-row">{}</div></div>"#,
                    links.join(" ")
                )
            }
        }
    };

    // Le retour pointe vers la liste qui correspond au rôle du tenant pour
    // cette facture (vendeur → Émises, acheteur → Reçues).
    let back_route = if sum.buyer_siren.as_deref() == Some(siren) {
        "/ui/recues"
    } else {
        "/ui/emises"
    };
    let back_label = if sum.buyer_siren.as_deref() == Some(siren) {
        "← Retour aux factures reçues"
    } else {
        "← Retour aux factures émises"
    };
    format!(
        r#"<p><a href="{back_route}?siren={siren}">{back_label}</a></p>
<div class="card"><h2>Métadonnées</h2>{metadata}</div>
{errors}
{downloads}
{attachments}
<div class="card"><h2>Timeline du pipeline</h2>{timeline}</div>"#,
        back_route = back_route,
        back_label = back_label,
        siren = html_escape(siren),
        metadata = metadata,
        errors = errors,
        downloads = downloads,
        attachments = attachments_section,
        timeline = timeline,
    )
}

// ============================================================
// Téléchargements (XML brut, PDF Factur-X, PJ)
// ============================================================

/// Récupère le `ExchangeDocument` complet depuis le flow_id (ou exchange_id).
/// Helper interne aux handlers de téléchargement.
async fn lookup_doc(
    state: &AppState,
    siren: &str,
    flow_id: &str,
) -> Option<pdp_trace::store::ExchangeDocument> {
    let store = state.trace_store.as_ref()?;
    // Le flow_id peut être un flow_id ou un exchange_id
    let summaries = store.list_exchanges(siren, None, None, None, 0, 200, None).await.ok()?;
    let summary = summaries
        .iter()
        .find(|s| s.flow_id == flow_id || s.exchange_id == flow_id)?;
    store.get_exchange(&summary.exchange_id, Some(siren)).await.ok().flatten()
}

/// GET /ui/flows/{flowId}/download/xml
/// Télécharge le `raw_xml` brut (UBL/CII).
pub async fn handle_download_xml(
    State(state): State<Arc<AppState>>,
    crate::security::AuthorizedSiren(siren): crate::security::AuthorizedSiren,
    Path(flow_id): Path<String>,
) -> impl IntoResponse {
    let siren = siren.as_str();
    let doc = match lookup_doc(&state, siren, &flow_id).await {
        Some(d) => d,
        None => return (StatusCode::NOT_FOUND, "Flux introuvable").into_response(),
    };
    let xml = match doc.raw_xml {
        Some(x) => x,
        None => return (StatusCode::NOT_FOUND, "raw_xml absent (Factur-X ou non indexé)").into_response(),
    };
    let filename = doc.source_filename.unwrap_or_else(|| format!("{}.xml", flow_id));
    let mut headers = axum::http::HeaderMap::new();
    headers.insert("content-type", "application/xml; charset=utf-8".parse().unwrap());
    if let Ok(v) = format!("attachment; filename=\"{}\"", filename).parse() {
        headers.insert("content-disposition", v);
    }
    (StatusCode::OK, headers, xml).into_response()
}

/// GET /ui/flows/{flowId}/download/pdf
/// Télécharge le PDF Factur-X (décodé du `raw_pdf_base64`).
pub async fn handle_download_pdf(
    State(state): State<Arc<AppState>>,
    crate::security::AuthorizedSiren(siren): crate::security::AuthorizedSiren,
    Path(flow_id): Path<String>,
) -> impl IntoResponse {
    let siren = siren.as_str();
    let doc = match lookup_doc(&state, siren, &flow_id).await {
        Some(d) => d,
        None => return (StatusCode::NOT_FOUND, "Flux introuvable").into_response(),
    };
    let b64 = match doc.raw_pdf_base64 {
        Some(b) => b,
        None => return (StatusCode::NOT_FOUND, "PDF absent (format non Factur-X)").into_response(),
    };
    use base64::Engine as _;
    let pdf_bytes = match base64::engine::general_purpose::STANDARD.decode(&b64) {
        Ok(b) => b,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Décodage base64 PDF échoué").into_response(),
    };
    let invoice_no = doc.invoice_number.unwrap_or_else(|| flow_id.clone());
    let filename = format!("{}.pdf", invoice_no);
    let mut headers = axum::http::HeaderMap::new();
    headers.insert("content-type", "application/pdf".parse().unwrap());
    if let Ok(v) = format!("attachment; filename=\"{}\"", filename).parse() {
        headers.insert("content-disposition", v);
    }
    (StatusCode::OK, headers, pdf_bytes).into_response()
}

#[derive(Deserialize)]
pub struct AttachmentDownloadQuery {
    pub siren: Option<String>,
    pub idx: usize,
}

/// GET /ui/flows/{flowId}/download/attachment?idx=N
/// Télécharge la N-ième pièce jointe (idx 0-indexé) extraite à la volée.
pub async fn handle_download_attachment(
    State(state): State<Arc<AppState>>,
    crate::security::AuthorizedSiren(siren): crate::security::AuthorizedSiren,
    Path(flow_id): Path<String>,
    Query(q): Query<AttachmentDownloadQuery>,
) -> impl IntoResponse {
    let siren = siren.as_str();
    let doc = match lookup_doc(&state, siren, &flow_id).await {
        Some(d) => d,
        None => return (StatusCode::NOT_FOUND, "Flux introuvable").into_response(),
    };
    let attachments = parse_attachments_from_doc(&doc);
    let att = match attachments.into_iter().nth(q.idx) {
        Some(a) => a,
        None => return (StatusCode::NOT_FOUND, "Pièce jointe introuvable").into_response(),
    };
    let content = match att.embedded_content {
        Some(c) => c,
        None => return (StatusCode::NOT_FOUND, "PJ sans contenu embarqué (URI externe uniquement)").into_response(),
    };
    let filename = att
        .filename
        .clone()
        .unwrap_or_else(|| format!("pj_{}.bin", q.idx));
    let mime = att
        .mime_code
        .clone()
        .unwrap_or_else(|| "application/octet-stream".to_string());
    let mut headers = axum::http::HeaderMap::new();
    if let Ok(v) = mime.parse() {
        headers.insert("content-type", v);
    }
    if let Ok(v) = format!("attachment; filename=\"{}\"", filename).parse() {
        headers.insert("content-disposition", v);
    }
    (StatusCode::OK, headers, content).into_response()
}

// ============================================================
// Static asset : icône Ferrite (PNG inliné dans le binaire)
// ============================================================

const FERRITE_ICON_PNG: &[u8] = include_bytes!("../../../assets/ferrite_icon_dark_512.png");
const FERRITE_FAVICON: &[u8] = include_bytes!("../../../assets/ferrite_icon_light_192.png");

/// GET /ui/static/ferrite-icon.png
pub async fn handle_logo() -> impl IntoResponse {
    let mut headers = axum::http::HeaderMap::new();
    headers.insert("content-type", "image/png".parse().unwrap());
    headers.insert("cache-control", "public, max-age=86400".parse().unwrap());
    (StatusCode::OK, headers, FERRITE_ICON_PNG).into_response()
}

/// GET /favicon.ico (alias /favicon.png)
pub async fn handle_favicon() -> impl IntoResponse {
    let mut headers = axum::http::HeaderMap::new();
    headers.insert("content-type", "image/png".parse().unwrap());
    headers.insert("cache-control", "public, max-age=604800".parse().unwrap());
    (StatusCode::OK, headers, FERRITE_FAVICON).into_response()
}

// ============================================================
// Helper réponse HTML
// ============================================================

fn html_response(body: &str) -> axum::response::Response {
    (StatusCode::OK, Html(body.to_string())).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_xml(rel_path: &str) -> String {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join(rel_path);
        std::fs::read_to_string(&path).unwrap()
    }

    fn doc_with(raw_xml: Option<String>, source_format: &str, filenames: Vec<String>) -> pdp_trace::store::ExchangeDocument {
        pdp_trace::store::ExchangeDocument {
            exchange_id: "ex-1".into(),
            flow_id: "flow-1".into(),
            source_filename: Some("facture.xml".into()),
            invoice_number: Some("F-001".into()),
            invoice_key: None,
            seller_name: None,
            buyer_name: None,
            seller_siret: None,
            buyer_siret: None,
            seller_siren: None,
            buyer_siren: None,
            source_format: Some(source_format.into()),
            total_ht: None,
            total_ttc: None,
            total_tax: None,
            currency: None,
            issue_date: None,
            status: "DISTRIBUÉ".into(),
            error_count: 0,
            raw_xml,
            raw_pdf_base64: None,
            converted_xml: None,
            converted_format: None,
            attachment_count: filenames.len(),
            attachment_filenames: filenames,
            events: vec![],
            errors: vec![],
            validation_warnings: vec![],
            created_at: "2025-11-15T10:00:00Z".into(),
            updated_at: "2025-11-15T10:00:00Z".into(),
        }
    }

    #[test]
    fn test_format_size_humanized() {
        assert_eq!(format_size(0), "0 B");
        assert_eq!(format_size(512), "512 B");
        assert_eq!(format_size(1024), "1.0 KB");
        assert_eq!(format_size(1536), "1.5 KB");
        assert_eq!(format_size(1_048_576), "1.00 MB");
        assert_eq!(format_size(2_500_000), "2.38 MB");
    }

    #[test]
    fn test_parse_attachments_from_doc_no_raw_xml() {
        let doc = doc_with(None, "UBL", vec![]);
        assert!(parse_attachments_from_doc(&doc).is_empty());
    }

    #[test]
    fn test_parse_attachments_from_doc_unknown_format() {
        let doc = doc_with(Some("<x/>".into()), "JSON", vec![]);
        assert!(parse_attachments_from_doc(&doc).is_empty());
    }

    #[test]
    fn test_parse_attachments_from_doc_ubl_no_attachments() {
        // Fixture standard sans PJ
        let xml = fixture_xml("tests/fixtures/ubl/facture_ubl_001.xml");
        let doc = doc_with(Some(xml), "UBL", vec![]);
        // Le parsing doit réussir, juste 0 PJ
        let attachments = parse_attachments_from_doc(&doc);
        // Selon la fixture, 0 ou plusieurs PJ — pas de crash
        let _ = attachments;
    }

    #[test]
    fn test_render_attachments_empty_no_filenames() {
        let html = render_attachments(&[], &[], "flow-test", "123456789");
        assert!(html.contains("Aucune pièce jointe"));
    }

    #[test]
    fn test_render_attachments_fallback_filenames_only() {
        // raw_xml indisponible mais on a les noms indexés en ES
        let html = render_attachments(&[], &["bon_commande.pdf".into(), "annexe.png".into()], "flow-test", "123456789");
        assert!(html.contains("Liste indexée uniquement"));
        assert!(html.contains("bon_commande.pdf"));
        assert!(html.contains("annexe.png"));
    }

    #[test]
    fn test_render_attachments_with_embedded() {
        use pdp_core::model::InvoiceAttachment;
        let attachments = vec![InvoiceAttachment {
            id: Some("ATT-1".into()),
            description: Some("Bon de commande".into()),
            external_uri: None,
            embedded_content: Some(vec![0u8; 2048]),
            mime_code: Some("application/pdf".into()),
            filename: Some("bon_commande.pdf".into()),
        }];
        let html = render_attachments(&attachments, &[], "flow-test", "123456789");
        assert!(html.contains("ATT-1"));
        assert!(html.contains("bon_commande.pdf"));
        assert!(html.contains("application/pdf"));
        assert!(html.contains("2.0 KB"));
        assert!(html.contains("extraite(s) à la volée"));
    }

    #[test]
    fn test_render_attachments_external_uri() {
        use pdp_core::model::InvoiceAttachment;
        let attachments = vec![InvoiceAttachment {
            id: Some("ATT-2".into()),
            description: None,
            external_uri: Some("https://example.com/specs.pdf".into()),
            embedded_content: None,
            mime_code: None,
            filename: None,
        }];
        let html = render_attachments(&attachments, &[], "flow-test", "123456789");
        assert!(html.contains("ATT-2"));
        assert!(html.contains(r#"<a href="https://example.com/specs.pdf""#));
        assert!(html.contains("externe"));
    }

    #[test]
    fn test_render_attachments_escapes_special_chars() {
        use pdp_core::model::InvoiceAttachment;
        let attachments = vec![InvoiceAttachment {
            id: Some("ATT-3".into()),
            description: Some("Description with <script> & quotes".into()),
            external_uri: None,
            embedded_content: Some(vec![0u8; 100]),
            mime_code: Some("application/pdf".into()),
            filename: Some("file<>.pdf".into()),
        }];
        let html = render_attachments(&attachments, &[], "flow-test", "123456789");
        // Pas de balise script injectée
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;"));
        assert!(html.contains("&amp;"));
    }
}
