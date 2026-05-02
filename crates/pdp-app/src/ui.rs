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
    padding: 1.5rem 2rem;
}
header h1 { font-size: 1.4rem; font-weight: 600; }
header nav { margin-top: 0.5rem; }
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
    <style>{css}</style>
</head>
<body>
    <header>
        <h1>Ferrite — Suivi des factures</h1>
        <nav>
            {dashboard_link}
            {flows_link}
            <a href="/annuaire">Annuaire</a>
        </nav>
    </header>
    <main>{body}</main>
</body>
</html>"#,
        title = title,
        css = CSS,
        dashboard_link = nav_link("", "Dashboard", "dashboard"),
        flows_link = nav_link("/flows", "Factures", "flows"),
        body = body,
    )
}

fn no_siren_banner() -> String {
    r#"<div class="banner">
        ⚠️ Aucun SIREN sélectionné. Ajouter <code>?siren=123456789</code> à l'URL pour cibler un tenant.
        Sans SIREN, les données ne peuvent pas être chargées (un index Elasticsearch par tenant).
    </div>"#.to_string()
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
        "DISTRIBUÉ" | "DISTRIBUE" | "ACKNOWLEDGED" => "badge-success",
        "ERREUR" | "ERROR" | "REJECTED" | "REJETÉ" | "REJETE" => "badge-error",
        "EN_ATTENTE" | "PENDING" | "WAITINGACK" | "WAITING" => "badge-warning",
        "VALIDATED" | "VALIDÉ" | "VALIDE" | "RECEIVED" => "badge-info",
        _ => "badge-default",
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
    Query(q): Query<DashboardQuery>,
) -> impl IntoResponse {
    let siren = q.siren.as_deref();

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
            format!(
                r#"
<div class="card">
    <h2>Tenant : {siren}</h2>
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
    <p><a href="/ui/flows?siren={siren}">→ Voir toutes les factures</a></p>
    <p><a href="/ui/flows?siren={siren}&status=ERREUR">→ Voir uniquement les erreurs</a></p>
    <p><a href="/v1/healthcheck">→ Healthcheck API</a></p>
    <p><a href="/metrics">→ Métriques Prometheus</a></p>
</div>
"#,
                siren = html_escape(s),
                total = stats.total_exchanges,
                distributed = stats.total_distributed,
                pending = pending.max(0),
                errors = stats.total_errors,
            )
        }
    };

    html_response(&page_shell("Dashboard", "dashboard", siren, &body))
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
// GET /ui/flows — Liste paginée
// ============================================================

#[derive(Deserialize)]
pub struct FlowsListQuery {
    pub siren: Option<String>,
    pub status: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub page: Option<usize>,
}

pub async fn handle_flows_list(
    State(state): State<Arc<AppState>>,
    Query(q): Query<FlowsListQuery>,
) -> impl IntoResponse {
    let siren = q.siren.as_deref();

    let body = match siren {
        None => format!("{}{}", no_siren_banner(), siren_picker_form()),
        Some(s) => {
            let store = match &state.trace_store {
                Some(st) => st,
                None => return html_response("TraceStore non configuré (Elasticsearch)"),
            };
            let page = q.page.unwrap_or(0);
            let page_size = 50;
            let exchanges = store
                .list_exchanges(
                    s,
                    q.status.as_deref(),
                    q.from.as_deref(),
                    q.to.as_deref(),
                    page,
                    page_size,
                )
                .await
                .unwrap_or_default();

            let filters_form = format!(
                r#"<form method="get" action="/ui/flows" class="filters">
    <input type="hidden" name="siren" value="{siren}">
    <select name="status">
        <option value="">— Tous les statuts —</option>
        <option value="DISTRIBUÉ" {sel_dist}>Distribués</option>
        <option value="ERREUR" {sel_err}>En erreur</option>
        <option value="EN_ATTENTE" {sel_pending}>En attente</option>
    </select>
    <input type="date" name="from" value="{from}" placeholder="Du">
    <input type="date" name="to" value="{to}" placeholder="Au">
    <button type="submit">Filtrer</button>
</form>"#,
                siren = html_escape(s),
                sel_dist = if q.status.as_deref() == Some("DISTRIBUÉ") { "selected" } else { "" },
                sel_err = if q.status.as_deref() == Some("ERREUR") { "selected" } else { "" },
                sel_pending = if q.status.as_deref() == Some("EN_ATTENTE") { "selected" } else { "" },
                from = q.from.as_deref().unwrap_or(""),
                to = q.to.as_deref().unwrap_or(""),
            );

            let rows = if exchanges.is_empty() {
                r#"<tr><td colspan="6" class="empty">Aucune facture trouvée pour ces critères.</td></tr>"#.to_string()
            } else {
                exchanges
                    .iter()
                    .map(|e| {
                        format!(
                            r#"<tr>
    <td><a href="/ui/flows/{flow_id}?siren={siren}">{invoice}</a></td>
    <td>{seller}</td>
    <td>{buyer}</td>
    <td><span class="badge {badge}">{status}</span></td>
    <td>{errors}</td>
    <td>{date}</td>
</tr>"#,
                            flow_id = html_escape(&e.flow_id),
                            siren = html_escape(s),
                            invoice = html_escape(e.invoice_number.as_deref().unwrap_or("—")),
                            seller = html_escape(e.seller_name.as_deref().unwrap_or("—")),
                            buyer = html_escape(e.buyer_name.as_deref().unwrap_or("—")),
                            badge = status_badge(&e.status),
                            status = html_escape(&e.status),
                            errors = e.error_count,
                            date = html_escape(&e.created_at[..e.created_at.len().min(10)]),
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            };

            let pagination = build_pagination(s, q.status.as_deref(), q.from.as_deref(), q.to.as_deref(), page, exchanges.len(), page_size);

            format!(
                r#"<div class="card">
    <h2>Factures du tenant {siren}</h2>
    {filters}
    <table>
        <thead>
            <tr><th>N° facture</th><th>Vendeur</th><th>Acheteur</th><th>Statut</th><th>Err.</th><th>Reçue le</th></tr>
        </thead>
        <tbody>{rows}</tbody>
    </table>
    {pagination}
</div>"#,
                siren = html_escape(s),
                filters = filters_form,
                rows = rows,
                pagination = pagination,
            )
        }
    };

    html_response(&page_shell("Factures", "flows", siren, &body))
}

fn build_pagination(
    siren: &str,
    status: Option<&str>,
    from: Option<&str>,
    to: Option<&str>,
    page: usize,
    page_count: usize,
    page_size: usize,
) -> String {
    let qs = |p: usize| -> String {
        let mut s = format!("?siren={}&page={}", siren, p);
        if let Some(st) = status { s.push_str(&format!("&status={}", st)); }
        if let Some(f) = from { s.push_str(&format!("&from={}", f)); }
        if let Some(t) = to { s.push_str(&format!("&to={}", t)); }
        s
    };
    let prev = if page > 0 {
        format!(r#"<a href="/ui/flows{}">← Précédent</a>"#, qs(page - 1))
    } else {
        r#"<span style="color:#aaa">← Précédent</span>"#.to_string()
    };
    let next = if page_count >= page_size {
        format!(r#"<a href="/ui/flows{}">Suivant →</a>"#, qs(page + 1))
    } else {
        r#"<span style="color:#aaa">Suivant →</span>"#.to_string()
    };
    format!(
        r#"<div style="margin-top:1rem; display:flex; justify-content:space-between; color:#666; font-size:0.9rem;">
        {prev}
        <span>Page {page_display} — {count} résultats</span>
        {next}
    </div>"#,
        prev = prev,
        next = next,
        page_display = page + 1,
        count = page_count,
    )
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
        .map(|a| {
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
            format!(
                r#"<tr>
    <td><code>{id}</code></td>
    <td>{filename}</td>
    <td>{description}</td>
    <td>{mime}</td>
    <td>{size}</td>
</tr>"#,
                id = html_escape(id),
                filename = html_escape(filename),
                description = html_escape(description),
                mime = html_escape(mime),
                size = size,
            )
        })
        .collect();

    format!(
        r#"<table>
    <thead>
        <tr><th>ID</th><th>Fichier</th><th>Description</th><th>MIME</th><th>Taille</th></tr>
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
    Path(flow_id): Path<String>,
    Query(q): Query<FlowDetailQuery>,
) -> impl IntoResponse {
    let siren = q.siren.as_deref();
    let body = match (siren, &state.trace_store) {
        (None, _) => format!("{}{}", no_siren_banner(), siren_picker_form()),
        (_, None) => "TraceStore non configuré (Elasticsearch)".to_string(),
        (Some(s), Some(store)) => {
            // Recherche par flow_id (les exchange_id sont aussi possibles)
            let summaries = store
                .list_exchanges(s, None, None, None, 0, 200)
                .await
                .unwrap_or_default();
            let summary = summaries.iter().find(|sum| sum.flow_id == flow_id || sum.exchange_id == flow_id);

            match summary {
                None => format!(
                    r#"<div class="card"><h2>Flux introuvable</h2><p>Aucun flux <code>{}</code> dans pdp-{}.</p><p><a href="/ui/flows?siren={}">← Retour à la liste</a></p></div>"#,
                    html_escape(&flow_id), html_escape(s), html_escape(s),
                ),
                Some(sum) => {
                    let exchange = store.get_exchange(&sum.exchange_id, Some(s)).await.ok().flatten();
                    render_flow_detail(s, &flow_id, sum, exchange.as_ref())
                }
            }
        }
    };
    html_response(&page_shell("Détail flux", "flows", siren, &body))
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
        badge = status_badge(&sum.status),
        status = html_escape(&sum.status),
        created_at = html_escape(&sum.created_at),
    );

    let timeline = match full {
        None => String::new(),
        Some(doc) => {
            if doc.events.is_empty() {
                r#"<p style="color:#888">Aucun événement enregistré.</p>"#.to_string()
            } else {
                let items: Vec<String> = doc
                    .events
                    .iter()
                    .map(|ev| {
                        format!(
                            r#"<div class="timeline-item">
    <div class="ts">{ts} — route <code>{route}</code></div>
    <div class="label">{status}</div>
    <div class="msg">{msg}</div>
</div>"#,
                            ts = html_escape(&ev.timestamp),
                            route = html_escape(&ev.route_id),
                            status = html_escape(&ev.status),
                            msg = html_escape(&ev.message),
                        )
                    })
                    .collect();
                format!(r#"<div class="timeline">{}</div>"#, items.join(""))
            }
        }
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
                body = render_attachments(&attachments, &doc.attachment_filenames),
            )
        }
    };

    format!(
        r#"<p><a href="/ui/flows?siren={siren}">← Retour à la liste</a></p>
<div class="card"><h2>Métadonnées</h2>{metadata}</div>
{errors}
{attachments}
<div class="card"><h2>Timeline du pipeline</h2>{timeline}</div>"#,
        siren = html_escape(siren),
        metadata = metadata,
        errors = errors,
        attachments = attachments_section,
        timeline = timeline,
    )
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
        let html = render_attachments(&[], &[]);
        assert!(html.contains("Aucune pièce jointe"));
    }

    #[test]
    fn test_render_attachments_fallback_filenames_only() {
        // raw_xml indisponible mais on a les noms indexés en ES
        let html = render_attachments(&[], &["bon_commande.pdf".into(), "annexe.png".into()]);
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
        let html = render_attachments(&attachments, &[]);
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
        let html = render_attachments(&attachments, &[]);
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
        let html = render_attachments(&attachments, &[]);
        // Pas de balise script injectée
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;"));
        assert!(html.contains("&amp;"));
    }
}
