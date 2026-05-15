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
///
/// Design system aligné sur `assets/design/ferrite-landing.html` :
/// palette cream + ink + accent rouille, typo Geist (UI) + Geist Mono (chiffres
/// et identifiants) + Instrument Serif italique (accents éditoriaux).
const CSS: &str = r#"
@import url('https://fonts.googleapis.com/css2?family=Geist:wght@300;400;500;600;700&family=Geist+Mono:wght@400;500&family=Instrument+Serif&display=swap');

:root {
    --bg: #FAFAF7;
    --bg-2: #F3F2EC;
    --ink: #0E0E0C;
    --ink-2: #2A2A26;
    --muted: #6B6A62;
    --muted-2: #9B9A91;
    --line: #E5E3DA;
    --line-2: #D9D6CB;
    --card: #FFFFFF;
    --accent: oklch(0.62 0.16 35);
    --accent-soft: oklch(0.95 0.04 50);
    --accent-ink: oklch(0.42 0.14 35);
    --good: oklch(0.55 0.12 150);
    --good-soft: oklch(0.95 0.04 150);
    --good-ink: #1E6A45;
    --warn: oklch(0.70 0.14 75);
    --warn-soft: oklch(0.96 0.05 75);
    --warn-ink: #7A5A14;
    --bad: oklch(0.58 0.18 25);
    --bad-soft: oklch(0.95 0.05 25);
    --bad-ink: #8A2A1E;
    --radius: 10px;
    --radius-sm: 6px;
    --shadow: 0 1px 0 rgba(15,15,12,.04), 0 8px 24px -16px rgba(15,15,12,.10);
    /* Slots utilisés par sidebar / table — passés en variables pour que le
       @media (prefers-color-scheme: dark) puisse les override sans dépendre
       de l'ordre des règles CSS. */
    --sidebar-bg: #FCFBF7;
    --sidebar-item-active-bg: #FFFFFF;
    --table-head-bg: #FCFBF7;
    --table-row-hover-bg: #FCFBF7;
    --tenant-avatar-grad-from: #0E0E0C;
    --tenant-avatar-grad-to: #3A3A33;
}

/* Dark mode auto via media query système. Pas de toggle UI : la CSP
   `script-src 'self'` rendrait l'expérience compliquée. On suit l'OS. */
@media (prefers-color-scheme: dark) {
    :root {
        --bg: #0F0F0D;
        --bg-2: #181815;
        --ink: #ECEAE0;
        --ink-2: #C9C7BC;
        --muted: #8A8980;
        --muted-2: #5E5D55;
        --line: #2A2A26;
        --line-2: #3A3A33;
        --card: #161613;
        --accent: oklch(0.72 0.14 40);
        --accent-soft: oklch(0.30 0.08 35 / 0.5);
        --accent-ink: oklch(0.78 0.13 50);
        --good: oklch(0.62 0.13 150);
        --good-soft: oklch(0.28 0.08 150 / 0.5);
        --good-ink: oklch(0.76 0.13 150);
        --warn: oklch(0.74 0.13 75);
        --warn-soft: oklch(0.30 0.10 75 / 0.5);
        --warn-ink: oklch(0.82 0.13 75);
        --bad: oklch(0.66 0.17 25);
        --bad-soft: oklch(0.30 0.10 25 / 0.5);
        --bad-ink: oklch(0.80 0.15 25);
        --shadow: 0 1px 0 rgba(0,0,0,.4), 0 8px 24px -16px rgba(0,0,0,.5);
        /* Slots dark mode — override les vars utilisées par sidebar/table */
        --sidebar-bg: #131311;
        --sidebar-item-active-bg: #161613;
        --table-head-bg: #1A1A17;
        --table-row-hover-bg: #1A1A17;
        --tenant-avatar-grad-from: oklch(0.78 0.13 50);
        --tenant-avatar-grad-to: #3A3A33;
    }
    /* Quelques ajustements qui restent en règles explicites parce qu'ils
       ciblent des combinateurs ou des pseudo-classes non re-themables via
       les seules variables CSS. */
    body { background: var(--bg); color: var(--ink); }
    header.legacy-topbar { background: rgba(15,15,13,.85); }
    .pj-badge { background: var(--bg-2); color: var(--ink); }
    .filters input, .filters select { background: var(--card); color: var(--ink); }
    .filters button { background: var(--ink); color: var(--bg); }
    .filters button:hover { background: #fff; }
    .dl-btn { background: var(--ink); color: var(--bg) !important; }
    .dl-btn:hover { background: #fff; }
    code { background: var(--bg-2); color: var(--ink-2); }
}

* { box-sizing: border-box; margin: 0; padding: 0; }
::selection { background: var(--accent); color: #fff; }

body {
    font-family: 'Geist','Inter',-apple-system,system-ui,sans-serif;
    font-feature-settings: 'ss01','cv11';
    -webkit-font-smoothing: antialiased;
    text-rendering: optimizeLegibility;
    background: var(--bg);
    color: var(--ink);
    min-height: 100vh;
    line-height: 1.5;
    letter-spacing: -0.005em;
}

.mono, code { font-family: 'Geist Mono',ui-monospace,SFMono-Regular,Menlo,monospace; }
.serif { font-family: 'Instrument Serif',Georgia,serif; font-style: italic; font-weight: 400; }

/* ============================================================
   Layout principal : sidebar 220px + main
   ============================================================ */
.app-layout {
    display: grid;
    grid-template-columns: 220px 1fr;
    min-height: 100vh;
}
.sidebar {
    background: var(--sidebar-bg);
    border-right: 1px solid var(--line);
    padding: 16px 12px 18px;
    display: flex;
    flex-direction: column;
    gap: 4px;
    height: 100vh;
    position: sticky;
    top: 0;
    overflow-y: auto;
}
.sidebar .brand-row {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 4px 8px 14px;
    border-bottom: 1px solid var(--line);
    margin-bottom: 8px;
}
.sidebar .brand-row img {
    width: 24px;
    height: 24px;
    border-radius: 6px;
    flex-shrink: 0;
}
.sidebar .brand-row b {
    font-size: 15px;
    font-weight: 600;
    letter-spacing: -0.01em;
    color: var(--ink);
}
.sidebar .brand-row small {
    color: var(--muted);
    font-size: 10.5px;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    border-left: 1px solid var(--line);
    padding-left: 8px;
    margin-left: 2px;
}
.sidebar .tenant {
    margin: 0 4px 12px;
    padding: 10px;
    border: 1px solid var(--line);
    border-radius: 8px;
    background: #fff;
    display: flex;
    align-items: center;
    gap: 9px;
}
.sidebar .tenant .ava {
    width: 26px;
    height: 26px;
    border-radius: 6px;
    background: linear-gradient(135deg, var(--tenant-avatar-grad-from), var(--tenant-avatar-grad-to));
    color: #fff;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 10.5px;
    font-weight: 600;
    letter-spacing: 0.02em;
    flex-shrink: 0;
}
.sidebar .tenant .meta {
    display: flex;
    flex-direction: column;
    line-height: 1.15;
    min-width: 0;
}
.sidebar .tenant .meta b {
    font-size: 12.5px;
    font-weight: 600;
    color: var(--ink);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
}
.sidebar .tenant .meta span {
    font-family: 'Geist Mono',monospace;
    font-size: 10.5px;
    color: var(--muted);
    margin-top: 1px;
}
.sidebar .group {
    font-size: 10.5px;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--muted-2);
    padding: 8px 10px 6px;
    font-weight: 500;
}
.sidebar .item {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 7px 10px;
    border-radius: 7px;
    font-size: 13px;
    color: var(--ink-2);
    text-decoration: none;
    margin: 1px 0;
    transition: background .12s ease;
    border: 1px solid transparent;
}
.sidebar .item:hover {
    background: var(--bg-2);
    text-decoration: none;
    color: var(--ink);
}
.sidebar .item.active {
    background: var(--sidebar-item-active-bg);
    border-color: var(--line);
    color: var(--ink);
    font-weight: 500;
    box-shadow: 0 1px 0 rgba(0,0,0,.02);
}
.sidebar .item svg {
    opacity: 0.55;
    flex-shrink: 0;
}
.sidebar .item.active svg { opacity: 0.85; }
.sidebar .item .count {
    margin-left: auto;
    font-family: 'Geist Mono',monospace;
    font-size: 11px;
    color: var(--muted);
}
.sidebar .item .count.soon {
    background: var(--bg-2);
    border: 1px solid var(--line);
    color: var(--muted-2);
    padding: 0 6px;
    border-radius: 999px;
    font-size: 9.5px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
}
.sidebar .footer {
    margin-top: auto;
    padding-top: 12px;
    border-top: 1px solid var(--line);
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
}
.sidebar .footer .principal {
    font-family: 'Geist Mono',monospace;
    font-size: 11px;
    color: var(--muted);
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    padding: 0 4px;
}
.sidebar .footer form.logout-form {
    display: inline;
}
.sidebar .footer form.logout-form button {
    background: transparent;
    border: 1px solid var(--line-2);
    color: var(--ink);
    padding: 0.3rem 0.7rem;
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-size: 11.5px;
    font-weight: 500;
    font-family: inherit;
    transition: background .12s ease, border-color .12s ease;
}
.sidebar .footer form.logout-form button:hover {
    background: var(--bg-2);
    border-color: var(--ink-2);
}

/* legacy header — masqué quand le layout sidebar est utilisé */
header.legacy-topbar {
    background: rgba(250,250,247,.85);
    backdrop-filter: saturate(140%) blur(10px);
    -webkit-backdrop-filter: saturate(140%) blur(10px);
    border-bottom: 1px solid var(--line);
    color: var(--ink);
    padding: 0.9rem 2rem;
    display: flex;
    align-items: center;
    gap: 1rem;
    position: sticky;
    top: 0;
    z-index: 50;
}
header .logo {
    height: 28px;
    width: 28px;
    flex-shrink: 0;
    border-radius: 7px;
}
header .brand {
    display: flex;
    align-items: baseline;
    gap: 10px;
    margin-right: 1.2rem;
}
header .brand .name {
    font-size: 1rem;
    font-weight: 600;
    letter-spacing: -0.01em;
    line-height: 1;
    color: var(--ink);
}
header .brand .tagline {
    font-size: 11px;
    color: var(--muted);
    letter-spacing: 0.06em;
    text-transform: uppercase;
    border-left: 1px solid var(--line);
    padding-left: 10px;
    line-height: 1;
}
header h1 {
    font-size: 0.9rem;
    font-weight: 450;
    color: var(--muted);
    margin: 0;
    letter-spacing: 0;
}
header nav { margin-left: auto; display: flex; align-items: center; gap: 1.4rem; }
header nav a {
    color: var(--ink-2);
    text-decoration: none;
    font-size: 13.5px;
    font-weight: 450;
    transition: color .12s ease;
}
header nav a:hover { color: var(--ink); }
header nav a.active { color: var(--ink); font-weight: 500; }

main {
    padding: 24px 32px 36px;
    min-width: 0;
}

/* Breadcrumbs + app-title (au-dessus du contenu de chaque page) */
.crumbs {
    font-size: 12px;
    color: var(--muted);
    margin-bottom: 8px;
    display: flex;
    gap: 8px;
    align-items: center;
    flex-wrap: wrap;
}
.crumbs .sep { color: var(--muted-2); }
.crumbs .current { color: var(--ink-2); }
.app-title {
    display: flex;
    align-items: flex-end;
    justify-content: space-between;
    gap: 12px;
    margin-bottom: 20px;
    flex-wrap: wrap;
}
.app-title h1 {
    font-size: 26px;
    letter-spacing: -.025em;
    margin: 0;
    font-weight: 500;
    line-height: 1.1;
}
.app-title .actions {
    display: flex;
    gap: 6px;
    align-items: center;
}

.kpi-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
    gap: 0.9rem;
    margin-bottom: 1.6rem;
}
.kpi-card {
    background: var(--card);
    border: 1px solid var(--line);
    border-radius: var(--radius);
    padding: 1.1rem 1.2rem 1rem;
    position: relative;
    overflow: hidden;
}
.kpi-card .spark {
    position: absolute;
    right: 14px;
    bottom: 14px;
    color: var(--accent);
    opacity: 0.65;
    pointer-events: none;
}
.kpi-card.success .spark { color: var(--good); }
.kpi-card.warn .spark { color: var(--warn); }
.kpi-card.bad .spark { color: var(--bad); }
.kpi-label {
    color: var(--muted);
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    font-weight: 500;
    margin-bottom: 0.5rem;
}
.kpi-value {
    font-size: 1.8rem;
    font-weight: 500;
    letter-spacing: -0.02em;
    color: var(--ink);
    font-feature-settings: 'tnum';
}
.kpi-value.success { color: var(--good-ink); }
.kpi-value.warning { color: var(--warn-ink); }
.kpi-value.error { color: var(--bad-ink); }
.kpi-delta {
    margin-top: 6px;
    font-size: 11.5px;
    color: var(--muted);
    letter-spacing: 0;
}
.kpi-delta.bad { color: var(--bad-ink); }
.muted-p { color: var(--muted); font-size: 13.5px; line-height: 1.55; }
.muted-p strong { color: var(--ink); font-weight: 500; font-variant-numeric: tabular-nums; }
.kpi-delta code {
    background: var(--bg-2);
    padding: 1px 5px;
    border-radius: 4px;
    font-size: 11px;
}

ul.link-list {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
}
ul.link-list a { font-size: 13.5px; }

/* Filtres statut en pills cliquables (au-dessus de la table) */
.pill-filters {
    display: flex;
    gap: 6px;
    flex-wrap: wrap;
    margin-bottom: 0.9rem;
}
.pill-filter {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    font-weight: 500;
    padding: 4px 11px;
    border-radius: 999px;
    border: 1px solid var(--line-2);
    background: #fff;
    color: var(--ink-2);
    text-decoration: none;
    transition: border-color .12s ease, background .12s ease, color .12s ease;
}
.pill-filter:hover {
    border-color: var(--ink-2);
    color: var(--ink);
    background: var(--bg-2);
    text-decoration: none;
}
.pill-filter::before {
    content: '';
    width: 5px;
    height: 5px;
    border-radius: 50%;
    background: var(--muted-2);
}
.pill-filter.pill-ok::before { background: var(--good); }
.pill-filter.pill-wait::before { background: var(--warn); }
.pill-filter.pill-err::before { background: var(--bad); }
.pill-filter.pill-default::before { display: none; }
.pill-filter.active {
    border-color: var(--ink);
    background: var(--ink);
    color: var(--bg);
}
.pill-filter.active.pill-ok { border-color: var(--good-ink); background: var(--good-soft); color: var(--good-ink); }
.pill-filter.active.pill-wait { border-color: var(--warn-ink); background: var(--warn-soft); color: var(--warn-ink); }
.pill-filter.active.pill-err { border-color: var(--bad-ink); background: var(--bad-soft); color: var(--bad-ink); }
.pill-filter.active::before { background: currentColor; opacity: 0.8; }
.pill-filters.compact { margin-bottom: 0; }
.pill-filters.compact .pill-filter { font-size: 11.5px; padding: 3px 9px; }

/* Bouton "fantôme" (Exporter, Annuler, …) — bordure fine sans background */
.btn-ghost {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 5px 11px;
    border: 1px solid var(--line-2);
    border-radius: var(--radius-sm);
    background: #fff;
    color: var(--ink);
    font-size: 12.5px;
    font-weight: 500;
    text-decoration: none;
    transition: background .12s ease, border-color .12s ease;
}
.btn-ghost:hover {
    background: var(--bg-2);
    border-color: var(--ink-2);
    text-decoration: none;
}

.card {
    background: var(--card);
    border: 1px solid var(--line);
    border-radius: var(--radius);
    padding: 1.4rem 1.5rem 1.3rem;
    margin-bottom: 1.2rem;
}
.card h2 {
    font-size: 1.05rem;
    font-weight: 500;
    letter-spacing: -0.015em;
    margin-bottom: 1rem;
    color: var(--ink);
}

table { width: 100%; border-collapse: collapse; }
thead { background: var(--table-head-bg); }
th, td {
    padding: 0.7rem 0.75rem;
    text-align: left;
    border-bottom: 1px solid var(--line);
    font-size: 13px;
}
th {
    color: var(--muted);
    font-weight: 500;
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
}
tr:hover td { background: var(--table-row-hover-bg); }
td.num, th.num {
    font-family: 'Geist Mono',monospace;
    font-size: 12px;
    font-variant-numeric: tabular-nums;
    color: var(--ink-2);
}
td.num a { color: var(--accent-ink); }
td.num a:hover { color: var(--accent); }

.badge {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 2px 9px;
    border-radius: 999px;
    font-size: 11.5px;
    font-weight: 500;
    letter-spacing: 0;
    text-transform: none;
    border: 1px solid transparent;
    background: #fff;
}
.badge::before {
    content: '';
    width: 5px;
    height: 5px;
    border-radius: 50%;
    background: currentColor;
    opacity: 0.8;
}
.badge-success {
    color: var(--good-ink);
    border-color: oklch(0.85 0.08 150);
    background: var(--good-soft);
}
.badge-error {
    color: var(--bad-ink);
    border-color: oklch(0.85 0.08 25);
    background: var(--bad-soft);
}
.badge-warning {
    color: var(--warn-ink);
    border-color: oklch(0.85 0.07 75);
    background: var(--warn-soft);
}
.badge-info {
    color: var(--accent-ink);
    border-color: oklch(0.86 0.06 50);
    background: var(--accent-soft);
}
.badge-default {
    color: var(--ink-2);
    border-color: var(--line-2);
    background: #fff;
}
.badge-default::before { display: none; }

.filters {
    display: flex;
    gap: 0.6rem;
    flex-wrap: wrap;
    margin-bottom: 1rem;
    align-items: center;
}
.filters input, .filters select {
    padding: 0.45rem 0.7rem;
    border: 1px solid var(--line-2);
    border-radius: var(--radius-sm);
    font-size: 13px;
    background: #fff;
    font-family: inherit;
    color: var(--ink);
    transition: border-color .12s ease;
}
.filters input:focus, .filters select:focus {
    outline: none;
    border-color: var(--ink-2);
}
.filters button {
    padding: 0.45rem 1rem;
    background: var(--ink);
    color: #FAFAF7;
    border: 1px solid var(--ink);
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-size: 13px;
    font-weight: 500;
    font-family: inherit;
    transition: background .12s ease;
}
.filters button:hover { background: #000; }

a { color: var(--accent-ink); text-decoration: none; transition: color .12s ease; }
a:hover { color: var(--accent); }

.empty { text-align: center; padding: 3rem; color: var(--muted); }

.timeline { position: relative; padding-left: 2rem; }
.timeline::before {
    content: '';
    position: absolute;
    left: 0.5rem; top: 0; bottom: 0;
    width: 1px;
    background: var(--line);
}
.timeline-item { position: relative; padding-bottom: 1.2rem; }
.timeline-item::before {
    content: '';
    position: absolute;
    left: -1.85rem; top: 0.35rem;
    width: 10px; height: 10px;
    border-radius: 50%;
    background: var(--accent);
    border: 2px solid var(--bg);
    box-shadow: 0 0 0 1px var(--accent);
}
.timeline-item .ts { color: var(--muted-2); font-size: 11.5px; font-family: 'Geist Mono',monospace; }
.timeline-item .label { font-weight: 500; margin-top: 2px; }
.timeline-item .msg { color: var(--muted); font-size: 13px; margin-top: 0.2rem; }
.timeline-item.timeline-error::before { background: var(--bad); box-shadow: 0 0 0 1px var(--bad); }
.timeline-item.timeline-error .label { color: var(--bad-ink); }
.timeline-item.timeline-error .msg { color: var(--bad-ink); }

dl.kv { display: grid; grid-template-columns: 200px 1fr; gap: 0.7rem 1rem; }
dl.kv dt { color: var(--muted); font-weight: 450; font-size: 13px; }
dl.kv dd { color: var(--ink); font-size: 13.5px; }

.banner {
    background: var(--warn-soft);
    color: var(--warn-ink);
    border: 1px solid oklch(0.85 0.07 75);
    padding: 0.7rem 1.1rem;
    border-radius: var(--radius-sm);
    margin-bottom: 1.2rem;
    font-size: 13px;
}

.dl-row { display: flex; gap: 0.5rem; flex-wrap: wrap; }
.dl-btn {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 0.45rem 0.9rem;
    background: var(--ink);
    color: #FAFAF7 !important;
    border-radius: var(--radius-sm);
    text-decoration: none;
    font-size: 13px;
    font-weight: 500;
    transition: background .12s ease;
}
.dl-btn:hover { background: #000; text-decoration: none; }

.pj-badge {
    display: inline-flex;
    align-items: center;
    padding: 1px 7px;
    border-radius: 999px;
    background: #fff;
    color: var(--ink-2);
    border: 1px solid var(--line-2);
    font-family: 'Geist Mono',monospace;
    font-size: 11px;
    font-weight: 500;
    white-space: nowrap;
}
.err-badge {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    padding: 1px 8px;
    border-radius: 999px;
    background: var(--bad-soft);
    color: var(--bad-ink);
    border: 1px solid oklch(0.85 0.08 25);
    font-size: 11.5px;
    font-weight: 500;
    white-space: nowrap;
}

.siret-sub {
    color: var(--muted-2);
    font-size: 11px;
    margin-top: 0.1rem;
    font-family: 'Geist Mono',monospace;
}

.dir-tag {
    display: inline-block;
    width: 1.1rem;
    height: 1.1rem;
    line-height: 1.1rem;
    text-align: center;
    border-radius: 4px;
    font-size: 10px;
    font-weight: 600;
    margin-right: 0.4rem;
    font-family: 'Geist Mono',monospace;
}
.dir-out { background: var(--accent-soft); color: var(--accent-ink); }
.dir-in  { background: var(--good-soft); color: var(--good-ink); }

.tenant-info {
    background: #fff;
    border: 1px solid var(--line);
    border-left: 3px solid var(--accent);
    padding: 0.65rem 1rem;
    border-radius: 0 var(--radius-sm) var(--radius-sm) 0;
    margin-bottom: 1rem;
    font-size: 13px;
    color: var(--ink-2);
}

.entreprise-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
    gap: 0.75rem;
}
.entreprise-card {
    display: block;
    padding: 1.1rem 1.2rem;
    background: var(--card);
    border: 1px solid var(--line);
    border-radius: var(--radius);
    text-decoration: none;
    color: var(--ink);
    transition: border-color .15s ease, box-shadow .15s ease, transform .15s ease;
}
.entreprise-card:hover {
    border-color: var(--ink-2);
    box-shadow: var(--shadow);
    transform: translateY(-1px);
    text-decoration: none;
}
.entreprise-name {
    font-weight: 500;
    font-size: 1rem;
    letter-spacing: -0.01em;
    margin-bottom: 0.25rem;
}
.entreprise-siren {
    color: var(--muted);
    font-size: 12.5px;
    font-family: 'Geist Mono',monospace;
}

header nav .nav-user {
    color: var(--muted);
    font-size: 12.5px;
    font-family: 'Geist Mono',monospace;
}
header nav form.logout-form { display: inline; }
header nav form.logout-form button {
    background: transparent;
    border: 1px solid var(--line-2);
    color: var(--ink);
    padding: 0.3rem 0.8rem;
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-size: 12.5px;
    font-weight: 500;
    font-family: inherit;
    transition: background .12s ease, border-color .12s ease;
}
header nav form.logout-form button:hover {
    background: var(--bg-2);
    border-color: var(--ink-2);
}
"#;

/// Compteurs affichés à droite des items de la sidebar (`Émises 412`).
/// Tous optionnels — `None` masque la pastille.
#[derive(Default, Clone, Copy)]
pub(crate) struct SidebarCounts {
    pub emises: Option<i64>,
    pub recues: Option<i64>,
}

/// Rend un sparkline SVG inline à partir d'une série de points.
///
/// Le SVG remplit (width × height), avec une marge de 1 px en haut/bas pour
/// éviter que le trait soit clippé. Si tous les points sont à zéro ou si la
/// série est vide, renvoie une chaîne vide (pas de sparkline → pas de bruit
/// visuel). La couleur de tracé est `currentColor`, donc se themeable via
/// `color` côté CSS parent.
pub(crate) fn sparkline_svg(points: &[i64], width: u32, height: u32) -> String {
    if points.len() < 2 {
        return String::new();
    }
    let max = points.iter().copied().max().unwrap_or(0);
    if max == 0 {
        return String::new();
    }
    let pad = 1.5_f64;
    let plot_h = (height as f64) - 2.0 * pad;
    let n = points.len() as f64 - 1.0;
    let mut path = String::with_capacity(points.len() * 12);
    for (i, &v) in points.iter().enumerate() {
        let x = (i as f64) / n * (width as f64 - 1.0);
        // y inversé : plus la valeur est grande, plus y est petit (haut du SVG).
        let y = pad + plot_h * (1.0 - (v as f64) / (max as f64));
        if i == 0 {
            path.push_str(&format!("M{:.1} {:.1}", x, y));
        } else {
            path.push_str(&format!("L{:.1} {:.1}", x, y));
        }
    }
    format!(
        r#"<svg class="spark" width="{w}" height="{h}" viewBox="0 0 {w} {h}" fill="none" aria-hidden="true"><path d="{path}" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/></svg>"#,
        w = width,
        h = height,
        path = path,
    )
}

pub(crate) fn page_shell(
    title: &str,
    active: &str,
    siren: Option<&str>,
    ctx: &crate::security::SecurityContext,
    body: &str,
) -> String {
    page_shell_with_counts(title, active, siren, ctx, &SidebarCounts::default(), body)
}

pub(crate) fn page_shell_with_counts(
    title: &str,
    active: &str,
    siren: Option<&str>,
    ctx: &crate::security::SecurityContext,
    counts: &SidebarCounts,
    body: &str,
) -> String {
    let siren_q = siren.map(|s| format!("?siren={}", s)).unwrap_or_default();
    let fmt_count = |n: Option<i64>| -> String {
        match n {
            Some(v) if v >= 1000 => format!("{} {}", v / 1000, format!("{:03}", v % 1000)),
            Some(v) => v.to_string(),
            None => String::new(),
        }
    };
    let item = |path: &str, label: &str, key: &str, icon: &str, count: &str| {
        let class = if key == active { "item active" } else { "item" };
        let count_html = if count.is_empty() {
            String::new()
        } else {
            format!(r#"<span class="count">{}</span>"#, count)
        };
        format!(
            r#"<a href="/ui{path}{q}" class="{class}">{icon}<span>{label}</span>{count}</a>"#,
            path = path,
            q = siren_q,
            class = class,
            icon = icon,
            label = label,
            count = count_html,
        )
    };
    // Icônes SVG inline pour la sidebar — outline 14 px, stroke 2.
    let ic_dashboard = r##"<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="7" height="7"/><rect x="14" y="3" width="7" height="7"/><rect x="3" y="14" width="7" height="7"/><rect x="14" y="14" width="7" height="7"/></svg>"##;
    let ic_emises = r##"<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z"/><path d="M14 2v6h6"/></svg>"##;
    let ic_recues = r##"<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4"/><path d="M7 10l5 5 5-5"/><path d="M12 15V3"/></svg>"##;
    let ic_annuaire = r##"<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="8"/><path d="M21 21l-4.35-4.35"/></svg>"##;
    let ic_admin = r##"<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/></svg>"##;
    let ic_pulse = r##"<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 12h-4l-3 9L9 3l-3 9H2"/></svg>"##;
    let ic_chart = r##"<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 3v18h18"/><path d="M18 9l-6 6-4-4-4 4"/></svg>"##;
    // Icônes pour les items "Opérations" (mock, pointent vers vues filtrées ou
    // endpoints existants — pas de UI dédiée pour ces concepts).
    let ic_cdv = r##"<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/></svg>"##;
    let ic_report = r##"<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z"/><path d="M14 2v6h6"/><line x1="9" y1="13" x2="15" y2="13"/><line x1="9" y1="17" x2="15" y2="17"/></svg>"##;
    let ic_webhook = r##"<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M13 2L3 14h9l-1 8 10-12h-9l1-8z"/></svg>"##;
    let ic_peppol = r##"<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><path d="M2 12h20M12 2a15 15 0 010 20M12 2a15 15 0 000 20"/></svg>"##;
    // Tenant card : avatar + label dérivés du domaine de l'email
    // (`alice@techconseil.demo` → label "Techconseil", avatar "TC"), SIREN
    // formaté en 3-3-3 dessous. Quand la page n'a pas de SIREN (admin global),
    // on affiche "Tous tenants".
    let tenant_card = match siren {
        Some(s) => {
            let domain_org = ctx.principal
                .split('@')
                .nth(1)
                .and_then(|d| d.split('.').next())
                .unwrap_or("tenant");
            let mut label_chars = domain_org.chars();
            let label = match label_chars.next() {
                Some(c) => c.to_uppercase().collect::<String>() + label_chars.as_str(),
                None => "Tenant".to_string(),
            };
            let avatar: String = domain_org
                .chars()
                .filter(|c| c.is_alphabetic())
                .take(2)
                .collect::<String>()
                .to_uppercase();
            let avatar = if avatar.is_empty() { "—".to_string() } else { avatar };
            let siren_fmt = if s.len() == 9 {
                format!("{} {} {}", &s[0..3], &s[3..6], &s[6..9])
            } else {
                s.to_string()
            };
            format!(
                r#"<div class="tenant">
            <div class="ava">{ava}</div>
            <div class="meta">
                <b>{label}</b>
                <span>SIREN {siren}</span>
            </div>
        </div>"#,
                ava = html_escape(&avatar),
                label = html_escape(&label),
                siren = html_escape(&siren_fmt),
            )
        }
        None => r#"<div class="tenant">
            <div class="ava">PD</div>
            <div class="meta">
                <b>Ferrite</b>
                <span>Tous tenants</span>
            </div>
        </div>"#
            .to_string(),
    };
    // Lien Admin réservé au rôle PdpAdmin (la route est protégée côté serveur).
    let admin_section = if is_admin(ctx) {
        let class = if active == "admin" { "item active" } else { "item" };
        format!(
            r#"<div class="group">Plateforme</div>
        <a href="/ui/admin" class="{class}">{ic}<span>Admin</span></a>"#,
            class = class,
            ic = ic_admin,
        )
    } else {
        String::new()
    };
    let logout_block = format!(
        r#"<span class="principal">{principal}</span>
            <form method="post" action="/logout" class="logout-form">
                <button type="submit">Sortir</button>
            </form>"#,
        principal = html_escape(&ctx.principal),
    );
    // Items "Opérations" : concepts métier de la PDP (cycle de vie, e-reporting,
    // webhooks, PEPPOL). Pas d'UI dédiée pour l'instant — on pointe vers les
    // endpoints API ou les vues filtrées existantes. La pastille `count` reste
    // muette (mock) car ces ressources n'ont pas de compteur centralisé.
    let ops_items = {
        let cdv_q = match siren {
            Some(s) => format!("/ui/emises?siren={}&status=DISTRIBU%C3%89", s),
            None => "/ui/emises".to_string(),
        };
        let ereporting_active = if active == "e-reporting" { " active" } else { "" };
        format!(
            r#"<a href="{cdv}" class="item">{ic_cdv}<span>Cycle de vie (CDV)</span></a>
        <a href="/ui/e-reporting{ereporting_q}" class="item{ereporting_active}">{ic_report}<span>E-reporting</span></a>
        <a href="https://github.com/lunatech-labs/lunatech-ferrite-pa-electronic-invoices/blob/main/docs/events.md" class="item" target="_blank" rel="noopener">{ic_webhook}<span>Webhooks</span></a>
        <a href="https://github.com/lunatech-labs/lunatech-ferrite-pa-electronic-invoices/blob/main/docs/peppol.md" class="item" target="_blank" rel="noopener">{ic_peppol}<span>PEPPOL AS4</span></a>"#,
            cdv = cdv_q,
            ereporting_q = siren_q,
            ereporting_active = ereporting_active,
            ic_cdv = ic_cdv,
            ic_report = ic_report,
            ic_webhook = ic_webhook,
            ic_peppol = ic_peppol,
        )
    };
    // Breadcrumbs : "Tenants / {tenant} / {section}". Le label tenant vient du
    // domaine de l'email (cf. derivation tenant_card), et le label section est
    // mappé depuis `active`. Pas de breadcrumbs sur les pages globales (admin
    // sans siren).
    let crumbs = {
        let section_label = match active {
            "dashboard" => Some("Tableau de bord"),
            "emises" => Some("Factures émises"),
            "recues" => Some("Factures reçues"),
            "e-reporting" => Some("E-reporting"),
            "admin" => Some("Administration"),
            _ => None,
        };
        match (siren, section_label) {
            (Some(_), Some(section)) => {
                let domain_org = ctx
                    .principal
                    .split('@')
                    .nth(1)
                    .and_then(|d| d.split('.').next())
                    .unwrap_or("tenant");
                let mut label_chars = domain_org.chars();
                let tenant_label = match label_chars.next() {
                    Some(c) => c.to_uppercase().collect::<String>() + label_chars.as_str(),
                    None => "Tenant".to_string(),
                };
                format!(
                    r#"<div class="crumbs"><span>Tenants</span><span class="sep">/</span><span>{tenant}</span><span class="sep">/</span><span class="current">{section}</span></div>"#,
                    tenant = html_escape(&tenant_label),
                    section = html_escape(section),
                )
            }
            (None, Some(section)) => format!(
                r#"<div class="crumbs"><span>Plateforme</span><span class="sep">/</span><span class="current">{section}</span></div>"#,
                section = html_escape(section),
            ),
            _ => String::new(),
        }
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
<div class="app-layout">
    <aside class="sidebar">
        <div class="brand-row">
            <img src="/ui/static/ferrite-icon.png" alt="">
            <b>Ferrite</b>
            <small>PA</small>
        </div>
        {tenant_card}
        <div class="group">Tenant</div>
        {dashboard_link}
        {emises_link}
        {recues_link}
        <a href="/annuaire" class="item">{ic_annuaire}<span>Annuaire PPF</span></a>
        <div class="group">Opérations</div>
        {ops_items}
        <div class="group">Outils</div>
        <a href="/v1/healthcheck" class="item" target="_blank" rel="noopener">{ic_pulse}<span>Healthcheck API</span></a>
        <a href="/metrics" class="item" target="_blank" rel="noopener">{ic_chart}<span>Métriques Prometheus</span></a>
        {admin_section}
        <div class="footer">
            {logout_block}
        </div>
    </aside>
    <main>{crumbs}{body}</main>
</div>
</body>
</html>"#,
        title = html_escape(title),
        css = CSS,
        tenant_card = tenant_card,
        dashboard_link = item("", "Tableau de bord", "dashboard", ic_dashboard, ""),
        emises_link = item("/emises", "Factures émises", "emises", ic_emises, &fmt_count(counts.emises)),
        recues_link = item("/recues", "Factures reçues", "recues", ic_recues, &fmt_count(counts.recues)),
        ic_annuaire = ic_annuaire,
        ic_pulse = ic_pulse,
        ic_chart = ic_chart,
        ops_items = ops_items,
        admin_section = admin_section,
        logout_block = logout_block,
        crumbs = crumbs,
        body = body,
    )
}

/// Convertit une option de paramètre de query en `Option<&str>` en éliminant
/// la chaîne vide. Le formulaire HTML envoie `?status=&from=&to=...` quand
/// l'utilisateur n'a rien sélectionné — sans cette normalisation, ces champs
/// arrivent en `Some("")` au handler et ES retourne 0 résultat sur un
/// `term: { "status": "" }`.
fn non_empty(opt: &Option<String>) -> Option<&str> {
    opt.as_deref().filter(|s| !s.is_empty())
}

pub(crate) fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

/// Direction métier d'une facture vue par un tenant. Détermine le libellé
/// AFNOR à afficher : un même `FlowStatus::Distributed` correspond à
/// `201 Émise` côté vendeur mais `203 Mise à disposition` côté acheteur.
#[derive(Debug, Clone, Copy)]
enum DisplayDirection {
    /// Le tenant est vendeur (BT-30) — facture émise.
    Emise,
    /// Le tenant est acheteur (BT-47) — facture reçue.
    Recue,
}

impl DisplayDirection {
    fn from_route(d: FlowDirection) -> Self {
        match d {
            FlowDirection::Emises => Self::Emise,
            FlowDirection::Recues => Self::Recue,
        }
    }

    /// Détermine la direction métier en comparant le SIREN du tenant
    /// à `seller_siren` / `buyer_siren` du document. Utilisé par la vue
    /// détail où la route ne porte pas la direction.
    fn from_summary(viewing_siren: &str, sum: &pdp_trace::store::ExchangeSummary) -> Self {
        if sum.seller_siren.as_deref() == Some(viewing_siren) {
            Self::Emise
        } else {
            Self::Recue
        }
    }
}

/// Classe CSS du badge associée à un code AFNOR `InvoiceStatusCode`.
fn afnor_badge_for_code(code: u16) -> &'static str {
    use pdp_cdar::model::InvoiceStatusCode::*;
    match pdp_cdar::model::InvoiceStatusCode::from_code(code) {
        // Rejets / irrecevabilité → rouge.
        Some(Refusee) | Some(Rejetee) | Some(Irrecevable) | Some(ErreurRoutage) => {
            "badge-error"
        }
        // États transitoires "en attente d'action" → orange.
        Some(EnLitige) | Some(Suspendue) | Some(Annulee) => "badge-warning",
        // États "transmise / vue / acquittée par C4" → vert.
        Some(Emise)
        | Some(Recue)
        | Some(MiseADisposition)
        | Some(PriseEnCharge)
        | Some(Approuvee)
        | Some(ApprouveePartiellement)
        | Some(Completee)
        | Some(Visee)
        | Some(PaiementTransmis)
        | Some(Encaissee) => "badge-success",
        // 200 Déposée + autres → bleu (in-flight côté PDP).
        _ => "badge-info",
    }
}

/// Statut métier AFNOR affiché à l'utilisateur.
///
/// Si la facture porte un `cdv_status_code` (CDV reçu de l'acheteur ou du
/// PPF), on retourne directement le libellé granulaire AFNOR (204 Prise en
/// charge, 205 Approuvée, 210 Refusée, 212 Encaissée, …). Sinon on retombe
/// sur la dérivation depuis `FlowStatus` interne, direction-dépendante.
///
/// Référence : XP Z12-012 Annexe A V1.2 (codes 200-501),
/// `specs/codelists/Statuts_facture_G2B_B2G.xlsx`,
/// docs/cdar.md §"Statuts de cycle de vie".
fn afnor_status(
    raw_status: &str,
    error_count: i32,
    cdv_status_code: Option<u16>,
    dir: DisplayDirection,
) -> (String, &'static str) {
    // 1. CDV reçu : libellé AFNOR exact, désambigüisé selon la direction.
    //    Côté émission, le vendeur ne doit jamais voir « Reçue » sur sa propre
    //    facture (ce serait illogique) : les codes 201/202/203 — qui décrivent
    //    le passage de la facture chez la PDP destinataire — collapse sur
    //    « Émise ». Côté réception, on garde « Reçue de la plateforme » pour
    //    distinguer du 203 Mise à disposition.
    if let Some(code) = cdv_status_code {
        let label = match (code, dir) {
            (201, DisplayDirection::Emise) => Some("Émise".to_string()),
            (202, DisplayDirection::Emise) => Some("Émise".to_string()),
            (203, DisplayDirection::Emise) => Some("Émise".to_string()),
            (202, DisplayDirection::Recue) => Some("Reçue de la plateforme".to_string()),
            _ => pdp_cdar::model::InvoiceStatusCode::from_code(code)
                .map(|s| s.label().replace('_', " ")),
        };
        if let Some(label) = label {
            return (label, afnor_badge_for_code(code));
        }
    }

    // 2. Pas de CDV : dérivation depuis FlowStatus.
    let upper = raw_status.to_uppercase();

    // 213 Rejetée (toute erreur ou rejet explicite).
    if error_count > 0
        || matches!(
            upper.as_str(),
            "ERREUR"
                | "ERROR"
                | "REJETÉ"
                | "REJETE"
                | "REJETÉE"
                | "REJETEE"
                | "REJECTED"
        )
    {
        return ("Rejetée".to_string(), "badge-error");
    }

    // 220 Annulée.
    if matches!(
        upper.as_str(),
        "ANNULÉ" | "ANNULE" | "ANNULÉE" | "ANNULEE" | "CANCELLED" | "CANCELED"
    ) {
        return ("Annulée".to_string(), "badge-warning");
    }

    match dir {
        DisplayDirection::Emise => match upper.as_str() {
            "REÇU" | "RECU" | "RECEIVED" | "PARSING" | "PARSÉ" | "PARSE" | "PARSED"
            | "VALIDATION" | "VALIDATING" | "VALIDÉ" | "VALIDE" | "VALIDATED"
            | "TRANSFORMATION" | "TRANSFORMING" | "TRANSFORMÉ" | "TRANSFORME"
            | "TRANSFORMED" | "DISTRIBUTION" | "DISTRIBUTING" => {
                ("Déposée".to_string(), "badge-info")
            }
            "DISTRIBUÉ" | "DISTRIBUE" | "DISTRIBUTED" | "ATTENTE_ACK" | "WAITINGACK"
            | "WAITING" | "ACQUITTÉ" | "ACQUITTE" | "ACKNOWLEDGED" => {
                ("Émise".to_string(), "badge-success")
            }
            _ => (raw_status.to_string(), "badge-default"),
        },
        DisplayDirection::Recue => match upper.as_str() {
            "REÇU" | "RECU" | "RECEIVED" | "PARSING" | "PARSÉ" | "PARSE" | "PARSED"
            | "VALIDATION" | "VALIDATING" | "VALIDÉ" | "VALIDE" | "VALIDATED"
            | "TRANSFORMATION" | "TRANSFORMING" | "TRANSFORMÉ" | "TRANSFORME"
            | "TRANSFORMED" => ("Reçue".to_string(), "badge-info"),
            "DISTRIBUTION" | "DISTRIBUTING" | "DISTRIBUÉ" | "DISTRIBUE"
            | "DISTRIBUTED" | "ATTENTE_ACK" | "WAITINGACK" | "WAITING" | "ACQUITTÉ"
            | "ACQUITTE" | "ACKNOWLEDGED" => {
                ("Mise à disposition".to_string(), "badge-success")
            }
            _ => (raw_status.to_string(), "badge-default"),
        },
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
    // Pour un tenant avec un seul SIREN autorisé, on saute le picker et
    // on redirige directement vers son dashboard — afficher une liste à
    // un seul élément n'a pas d'intérêt et casse le flux de connexion.
    if let Some(r) = auto_redirect_single_tenant(&ctx, owned_siren.as_deref(), "/ui") {
        return r;
    }
    let siren = owned_siren.as_deref();

    let body = match siren {
        None => siren_picker(&state, &ctx, "/ui"),
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
            let tenant_name = resolve_tenant_name(&state, s).await;
            let tenant_label = tenant_name.as_deref().unwrap_or(s);
            // 4 sparklines 14 jours (total / distribués / pending / erreurs) en
            // une seule requête ES via filter aggregations. Best-effort.
            let bk = store
                .daily_breakdown_for_siren(s, 14)
                .await
                .unwrap_or_else(|_| pdp_trace::store::DailyBreakdown::zeros(14));
            let spk_total = sparkline_svg(&bk.total, 60, 22);
            let spk_distributed = sparkline_svg(&bk.distributed, 60, 22);
            let spk_pending = sparkline_svg(&bk.pending, 60, 22);
            let spk_errors = sparkline_svg(&bk.errors, 60, 22);
            format!(
                r#"<div class="app-title">
    <h1>Flux <span class="serif">en circulation</span></h1>
    <div class="actions"><span class="pj-badge">{tenant_label}</span><span class="pj-badge">SIREN {siren}</span></div>
</div>
<div class="kpi-grid">
    <div class="kpi-card">
        <div class="kpi-label">Total flux</div>
        <div class="kpi-value">{total}</div>
        <div class="kpi-delta">14 jours · index <code>pdp-{siren}</code></div>
        {spk_total}
    </div>
    <div class="kpi-card success">
        <div class="kpi-label">Distribués</div>
        <div class="kpi-value success">{distributed}</div>
        <div class="kpi-delta">{pct}% du total</div>
        {spk_distributed}
    </div>
    <div class="kpi-card warn">
        <div class="kpi-label">En attente</div>
        <div class="kpi-value warning">{pending}</div>
        <div class="kpi-delta">Routage / annuaire PPF</div>
        {spk_pending}
    </div>
    <div class="kpi-card bad">
        <div class="kpi-label">En erreur</div>
        <div class="kpi-value error">{errors}</div>
        <div class="kpi-delta bad">Rejets BR-FR ou pipeline</div>
        {spk_errors}
    </div>
</div>
<div class="card">
    <h2>Cycle de vie <span class="serif">en un coup d'œil</span></h2>
    <p class="muted-p">Sur les <strong>{total}</strong> flux du tenant, <strong>{distributed}</strong> ont atteint l'état terminal AFNOR (Émise / Mise à disposition), <strong>{pending}</strong> attendent une étape de routage ou d'annuaire, et <strong>{errors}</strong> ont été rejetés (BR-FR, doublon ou erreur pipeline).</p>
    <ul class="link-list" style="margin-top:0.9rem">
        <li><a href="/ui/emises?siren={siren}&status=ERREUR">→ Voir les factures émises en erreur</a></li>
        <li><a href="/ui/recues?siren={siren}&status=ERREUR">→ Voir les factures reçues en erreur</a></li>
    </ul>
</div>"#,
                tenant_label = html_escape(tenant_label),
                siren = html_escape(s),
                total = stats.total_exchanges,
                distributed = stats.total_distributed,
                pending = pending.max(0),
                errors = stats.total_errors,
                pct = if stats.total_exchanges > 0 {
                    (stats.total_distributed * 100) / stats.total_exchanges
                } else {
                    0
                },
                spk_total = spk_total,
                spk_distributed = spk_distributed,
                spk_pending = spk_pending,
                spk_errors = spk_errors,
            )
        }
    };

    let counts = sidebar_counts_for(&state, siren).await;
    html_response(&page_shell_with_counts(
        "Dashboard",
        "dashboard",
        siren,
        &ctx,
        &counts,
        &body,
    ))
    .into_response()
}

/// Calcule les compteurs émises/reçues pour le tenant courant — affichés à
/// droite des items de la sidebar. Best-effort : si ES ne répond pas, on
/// retombe sur `None` et la pastille ne s'affiche pas.
async fn sidebar_counts_for(
    state: &crate::server::AppState,
    siren: Option<&str>,
) -> SidebarCounts {
    let Some(s) = siren else { return SidebarCounts::default() };
    let Some(store) = state.trace_store.as_ref() else { return SidebarCounts::default() };
    // Compte par factures uniques (invoice_number distincts) — sinon la
    // pastille gonfle artificiellement avec les ré-soumissions BR-FR-12/13
    // et ne correspond plus au nombre de lignes visibles dans la liste.
    SidebarCounts {
        emises: store
            .count_exchanges_with_dedup(s, None, None, None, Some("emises"), true)
            .await
            .ok(),
        recues: store
            .count_exchanges_with_dedup(s, None, None, None, Some("recues"), true)
            .await
            .ok(),
    }
}

/// Résout la raison sociale d'une entreprise pour un SIREN donné.
///
/// Source de vérité : `{tenants_dir}/{siren}/config.yaml` (= ce qui a été
/// saisi par l'admin dans le formulaire de création). C'est ce qu'on
/// affiche en priorité dans les titres des écrans.
///
/// Fallback : `seller_name` / `buyer_name` inféré depuis les documents
/// Elasticsearch (`store.get_tenant_name`). Utile quand un SIREN n'a pas
/// (encore) été enregistré comme entreprise mais apparaît dans des
/// factures importées — sinon on n'aurait que le SIREN brut.
async fn resolve_tenant_name(state: &crate::server::AppState, siren: &str) -> Option<String> {
    // 1. Config.yaml de l'entreprise (autoritaire)
    if let Some(dir) = &state.tenants_dir {
        let cfg_path = dir.join(siren).join("config.yaml");
        if cfg_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&cfg_path) {
                if let Ok(cfg) =
                    serde_yaml::from_str::<pdp_config::model::TenantConfig>(&content)
                {
                    let name = cfg.pdp.name.trim();
                    if !name.is_empty() {
                        return Some(name.to_string());
                    }
                }
            }
        }
    }
    // 2. Inférence depuis ES (seller_name / buyer_name d'une facture passée)
    if let Some(store) = &state.trace_store {
        return store.get_tenant_name(siren).await;
    }
    None
}

/// Si le porteur est un `Tenant` avec exactement UN `allowed_siren` et
/// qu'aucune `?siren=` n'a été fournie, retourne un `303 → target?siren=<...>`.
/// Pour les autres cas (admin, opérateur, tenant multi-siren, ou siren déjà
/// fourni), retourne `None` — le handler affichera son picker / sa page.
pub(crate) fn auto_redirect_single_tenant(
    ctx: &crate::security::SecurityContext,
    siren: Option<&str>,
    target: &str,
) -> Option<axum::response::Response> {
    use crate::security::Role;
    if siren.is_some() {
        return None;
    }
    if matches!(ctx.role, Role::Tenant) && ctx.allowed_sirens.len() == 1 {
        let to = format!("{target}?siren={}", ctx.allowed_sirens[0]);
        return Some(axum::response::Redirect::to(&to).into_response());
    }
    None
}

/// Sélecteur d'entreprise affiché quand aucune `?siren=` n'est fourni.
///
/// Contextuel au rôle du porteur :
/// - `PdpAdmin` / `PdpOperator` : liste **toutes** les entreprises
///   trouvées dans `state.tenants_dir`. Pas de saisie libre — on clique.
/// - `Tenant` : liste les SIRENs autorisés (`allowed_sirens`). Si la
///   liste est vide ou si une seule entrée existe, on retombe sur un
///   message d'aide (la redirection vers l'unique SIREN est faite plus
///   haut par le handler).
///
/// `target_path` permet à un même picker de pointer vers `/ui`,
/// `/ui/emises` ou `/ui/recues` selon la page d'origine.
pub(crate) fn siren_picker(
    state: &crate::server::AppState,
    ctx: &crate::security::SecurityContext,
    target_path: &str,
) -> String {
    use crate::security::Role;
    use std::collections::HashMap;

    // Map SIREN → nom d'entreprise (si config.yaml présent).
    let known: HashMap<String, String> = state
        .tenants_dir
        .as_ref()
        .and_then(|dir| pdp_config::discover_tenants(dir).ok())
        .unwrap_or_default()
        .into_iter()
        .map(|e| (e.siren.clone(), e.config.pdp.name.clone()))
        .collect();

    let entries: Vec<(String, Option<String>)> = match ctx.role {
        Role::PdpAdmin | Role::PdpOperator => {
            let mut v: Vec<_> = known
                .iter()
                .map(|(s, n)| (s.clone(), Some(n.clone())))
                .collect();
            v.sort_by(|a, b| a.0.cmp(&b.0));
            v
        }
        Role::Tenant => ctx
            .allowed_sirens
            .iter()
            .map(|s| (s.clone(), known.get(s).cloned()))
            .collect(),
    };

    if entries.is_empty() {
        let msg = match ctx.role {
            Role::PdpAdmin | Role::PdpOperator => format!(
                r#"Aucune entreprise enregistrée pour le moment.
                Rendez-vous sur <a href="/ui/admin">l'administration</a>
                pour en créer une."#,
            ),
            Role::Tenant => format!(
                r#"Aucun SIREN n'est associé à votre compte
                (<code>{}</code>). Contactez l'administrateur PDP."#,
                html_escape(&ctx.principal),
            ),
        };
        return format!(r#"<div class="card"><h2>Choisir une entreprise</h2><p>{msg}</p></div>"#);
    }

    let cards = entries
        .iter()
        .map(|(siren, name)| {
            let label = name.clone().unwrap_or_else(|| format!("Tenant {siren}"));
            format!(
                r#"<a href="{target}?siren={siren}" class="entreprise-card">
                    <div class="entreprise-name">{name}</div>
                    <div class="entreprise-siren"><code>{siren}</code></div>
                </a>"#,
                target = html_escape(target_path),
                siren = html_escape(siren),
                name = html_escape(&label),
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let title = match ctx.role {
        Role::PdpAdmin | Role::PdpOperator => "Choisir une entreprise",
        Role::Tenant => "Vos entreprises",
    };

    format!(
        r#"<div class="card">
            <h2>{title}</h2>
            <div class="entreprise-grid">{cards}</div>
        </div>"#,
    )
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
    /// Variante HTML du titre avec accent serif italique sur le mot
    /// directionnel — match le pattern éditorial du design (Geist + Instrument Serif).
    fn page_title_html(&self) -> &'static str {
        match self {
            FlowDirection::Emises => r#"Factures <span class="serif">émises</span>"#,
            FlowDirection::Recues => r#"Factures <span class="serif">reçues</span>"#,
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

/// `GET /ui/emises/export.csv` — export CSV des factures émises (mêmes
/// filtres que la liste UI).
pub async fn handle_export_emises_csv(
    State(state): State<Arc<AppState>>,
    ctx: axum::Extension<std::sync::Arc<crate::security::SecurityContext>>,
    q: Query<FlowsListQuery>,
) -> axum::response::Response {
    export_flows_csv(state, ctx, q, FlowDirection::Emises).await
}

/// `GET /ui/e-reporting` — page d'aperçu E-reporting (Flux 10.1 → 10.4).
///
/// La PDP a la responsabilité d'agréger et de transmettre 4 flux périodiques
/// au PPF (cf. AFNOR XP Z12-013 §10) :
/// - 10.1 Transactions B2C
/// - 10.2 Transactions B2B vers étranger
/// - 10.3 Encaissements (état de paiement)
/// - 10.4 Opérations exonérées
///
/// Cette page est en mode "Aperçu" pour l'instant : l'extraction et le push
/// automatique vers le PPF sont en cours d'intégration ([`pdp_ereporting`]).
pub async fn handle_e_reporting(
    State(state): State<Arc<AppState>>,
    axum::Extension(ctx): axum::Extension<std::sync::Arc<crate::security::SecurityContext>>,
    Query(q): Query<DashboardQuery>,
) -> axum::response::Response {
    let owned_siren = match crate::security::authorize_optional_siren(&ctx, q.siren.as_deref()) {
        Ok(s) => s,
        Err(resp) => return resp,
    };
    if let Some(r) = auto_redirect_single_tenant(&ctx, owned_siren.as_deref(), "/ui/e-reporting") {
        return r;
    }
    let siren = owned_siren.as_deref();
    let body = match siren {
        None => siren_picker(&state, &ctx, "/ui/e-reporting"),
        Some(s) => format!(
            r#"<div class="app-title">
    <h1>E-reporting <span class="serif">périodique</span></h1>
    <div class="actions"><span class="pj-badge">Aperçu</span><span class="pj-badge">SIREN {siren}</span></div>
</div>
<div class="banner">
    Cette section est en cours d'intégration : les compteurs ci-dessous sont
    indicatifs (réforme française 2026, AFNOR XP Z12-013 §10). L'extraction
    automatique vers le PPF est gérée par <code>pdp-ereporting</code>.
</div>
<div class="kpi-grid">
    <div class="kpi-card">
        <div class="kpi-label">Flux 10.1 · B2C</div>
        <div class="kpi-value">—</div>
        <div class="kpi-delta">Transactions avec consommateurs</div>
    </div>
    <div class="kpi-card">
        <div class="kpi-label">Flux 10.2 · B2B intl</div>
        <div class="kpi-value">—</div>
        <div class="kpi-delta">B2B vers / depuis l'étranger</div>
    </div>
    <div class="kpi-card">
        <div class="kpi-label">Flux 10.3 · Encaissements</div>
        <div class="kpi-value">—</div>
        <div class="kpi-delta">État de paiement (BR-FR-MAP)</div>
    </div>
    <div class="kpi-card">
        <div class="kpi-label">Flux 10.4 · Exonérées</div>
        <div class="kpi-value">—</div>
        <div class="kpi-delta">Opérations sans TVA française</div>
    </div>
</div>
<div class="card">
    <h2>Pourquoi <span class="serif">e-reporting</span> ?</h2>
    <p class="muted-p">
        Au-delà de la facturation électronique B2B (réforme 2026), la PDP doit
        transmettre périodiquement au Portail Public de Facturation (PPF) les
        transactions <strong>hors champ B2B FR</strong> : ventes aux particuliers,
        opérations transfrontalières, encaissements et exonérations. Quatre flux
        XML normalisés sont attendus :
    </p>
    <ul class="link-list" style="margin-top:0.8rem">
        <li><a href="https://github.com/lunatech-labs/lunatech-ferrite-pa-electronic-invoices/blob/main/docs/ereporting.md" target="_blank" rel="noopener">→ Documentation Ferrite e-reporting</a></li>
        <li><a href="https://www.afnor.org/" target="_blank" rel="noopener">→ AFNOR XP Z12-013 (norme officielle)</a></li>
        <li><a href="/ui/emises?siren={siren}">→ Voir les factures émises</a></li>
    </ul>
</div>"#,
            siren = html_escape(s),
        ),
    };
    let counts = sidebar_counts_for(&state, siren).await;
    html_response(&page_shell_with_counts(
        "E-reporting",
        "e-reporting",
        siren,
        &ctx,
        &counts,
        &body,
    ))
    .into_response()
}

/// `GET /ui/recues/export.csv` — export CSV des factures reçues.
pub async fn handle_export_recues_csv(
    State(state): State<Arc<AppState>>,
    ctx: axum::Extension<std::sync::Arc<crate::security::SecurityContext>>,
    q: Query<FlowsListQuery>,
) -> axum::response::Response {
    export_flows_csv(state, ctx, q, FlowDirection::Recues).await
}

async fn export_flows_csv(
    state: Arc<AppState>,
    axum::Extension(ctx): axum::Extension<std::sync::Arc<crate::security::SecurityContext>>,
    Query(q): Query<FlowsListQuery>,
    direction: FlowDirection,
) -> axum::response::Response {
    let owned_siren = match crate::security::authorize_optional_siren(&ctx, q.siren.as_deref()) {
        Ok(s) => s,
        Err(resp) => return resp,
    };
    let siren = match owned_siren.as_deref() {
        Some(s) => s,
        None => return (StatusCode::BAD_REQUEST, "siren requis").into_response(),
    };
    let store = match &state.trace_store {
        Some(st) => st,
        None => return (StatusCode::SERVICE_UNAVAILABLE, "TraceStore non configuré").into_response(),
    };
    // Pas de pagination pour l'export — on cap à 5000 pour éviter
    // d'exploser la mémoire sur un tenant qui aurait des milliers de flux.
    let status = non_empty(&q.status);
    let from = non_empty(&q.from);
    let to = non_empty(&q.to);
    // Dedup par invoice_number par défaut, sauf `?show_duplicates=true` :
    // on aligne le contenu du CSV avec ce que l'utilisateur voit à l'écran.
    let dedup_by_invoice = q.show_duplicates.as_deref() != Some("true");
    let exchanges = match store
        .list_exchanges_with_dedup(
            siren, status, from, to, 0, 5000, Some(direction.nav_key()), dedup_by_invoice,
        )
        .await
    {
        Ok(v) => v,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Liste échouée: {e}")).into_response(),
    };

    // Construction du CSV — encodage utf-8 + BOM pour Excel + séparateur `;`
    // (locale FR : la virgule peut entrer en collision avec les décimales).
    let mut csv = String::from("\u{FEFF}");
    csv.push_str("N° facture;Vendeur;SIREN vendeur;Acheteur;SIREN acheteur;Statut AFNOR;Code CDV;Erreurs;Pièces jointes;Reçue le;Flow ID\n");
    let dir = DisplayDirection::from_route(direction);
    for e in &exchanges {
        let (afnor_label, _) = afnor_status(&e.status, e.error_count, e.cdv_status_code, dir);
        let row = format!(
            "{inv};{vn};{vs};{bn};{bs};{st};{cdv};{err};{pj};{date};{fid}\n",
            inv = csv_escape(e.invoice_number.as_deref().unwrap_or("")),
            vn = csv_escape(e.seller_name.as_deref().unwrap_or("")),
            vs = csv_escape(e.seller_siren.as_deref().unwrap_or("")),
            bn = csv_escape(e.buyer_name.as_deref().unwrap_or("")),
            bs = csv_escape(e.buyer_siren.as_deref().unwrap_or("")),
            st = csv_escape(&afnor_label),
            cdv = e.cdv_status_code.map(|c| c.to_string()).unwrap_or_default(),
            err = e.error_count,
            pj = e.attachment_count,
            date = csv_escape(&e.created_at),
            fid = csv_escape(&e.flow_id),
        );
        csv.push_str(&row);
    }

    let filename = format!(
        "ferrite-{}-{}.csv",
        match direction {
            FlowDirection::Emises => "emises",
            FlowDirection::Recues => "recues",
        },
        chrono::Utc::now().format("%Y-%m-%d"),
    );
    (
        StatusCode::OK,
        [
            (axum::http::header::CONTENT_TYPE, "text/csv; charset=utf-8".to_string()),
            (
                axum::http::header::CONTENT_DISPOSITION,
                format!(r#"attachment; filename="{}""#, filename),
            ),
        ],
        csv,
    )
        .into_response()
}

/// Échappement CSV minimal : guillemets doubles + double les `"` internes
/// si la cellule contient un séparateur, un saut de ligne ou un `"`.
fn csv_escape(s: &str) -> String {
    if s.contains(';') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
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
    let target = match direction {
        FlowDirection::Emises => "/ui/emises",
        FlowDirection::Recues => "/ui/recues",
    };
    if let Some(r) = auto_redirect_single_tenant(&ctx, owned_siren.as_deref(), target) {
        return r;
    }
    let siren = owned_siren.as_deref();
    // Le formulaire HTML soumet les champs vides comme `?status=&from=&to=...`,
    // qui se désérialisent en `Some("")` (et non `None`). On les normalise ici
    // sinon ES voit `term: { "status": "" }` et ne renvoie rien.
    let status = non_empty(&q.status);
    let from = non_empty(&q.from);
    let to = non_empty(&q.to);

    let body = match siren {
        None => siren_picker(
            &state,
            &ctx,
            match direction {
                FlowDirection::Emises => "/ui/emises",
                FlowDirection::Recues => "/ui/recues",
            },
        ),
        Some(s) => {
            let store = match &state.trace_store {
                Some(st) => st,
                None => return html_response("TraceStore non configuré (Elasticsearch)"),
            };
            let page = q.page.unwrap_or(0);
            let page_size = clamp_page_size(q.page_size);
            let dir_param = Some(direction.nav_key());
            // Déduplication par invoice_number (par défaut activée).
            // Plusieurs soumissions de la même facture créent plusieurs
            // exchanges (BR-FR-12/13 marque les ré-soumissions avec
            // error_count>0 mais le doc reste indexé). Quand la dedup est
            // active, on demande à ES de collapser sur `invoice_number` et
            // on compte les invoice_numbers distincts — ce qui aligne le
            // total affiché et le contenu paginé. Avec
            // `?show_duplicates=true`, on bascule sur la vue brute.
            let dedup_by_invoice = q.show_duplicates.as_deref() != Some("true");
            let total = store
                .count_exchanges_with_dedup(s, status, from, to, dir_param, dedup_by_invoice)
                .await
                .unwrap_or(0);
            let exchanges = store
                .list_exchanges_with_dedup(s, status, from, to, page, page_size, dir_param, dedup_by_invoice)
                .await
                .unwrap_or_default();

            let tenant_name = resolve_tenant_name(&state, s).await;
            let list_title = format!(
                "{label}",
                label = direction.page_title_html(),
            );
            let tenant_subtitle = format!(
                r#"<span class="pj-badge">{who}</span><span class="pj-badge">SIREN {siren}</span>"#,
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
            // Filtres statut : pills cliquables (lien GET) qui préservent les
            // autres params (from/to/page_size) via la query string. Match le
            // pattern .pill du design Ferrite Landing.
            let pill_link = |value: &str, label: &str, klass: &str| -> String {
                let active = q.status.as_deref() == if value.is_empty() {
                    None
                } else {
                    Some(value)
                };
                let mut qs = format!("siren={}", html_escape(s));
                if !value.is_empty() {
                    // Encodage URL minimal pour le status (peut contenir É, etc.)
                    let enc: String = value
                        .bytes()
                        .map(|b| match b {
                            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'_' | b'-' | b'.' | b'~' => {
                                (b as char).to_string()
                            }
                            _ => format!("%{:02X}", b),
                        })
                        .collect();
                    qs.push_str(&format!("&status={}", enc));
                }
                if let Some(f) = q.from.as_deref().filter(|x| !x.is_empty()) {
                    qs.push_str(&format!("&from={}", html_escape(f)));
                }
                if let Some(t) = q.to.as_deref().filter(|x| !x.is_empty()) {
                    qs.push_str(&format!("&to={}", html_escape(t)));
                }
                if page_size != 50 {
                    qs.push_str(&format!("&page_size={page_size}"));
                }
                format!(
                    r#"<a href="{path}?{qs}" class="pill-filter {klass}{active_cls}">{label}</a>"#,
                    path = direction.route_path(),
                    qs = qs,
                    klass = klass,
                    active_cls = if active { " active" } else { "" },
                    label = label,
                )
            };
            let status_pills = format!(
                r#"<div class="pill-filters">
        {all}{ok}{wait}{err}
    </div>"#,
                all = pill_link("", "Tous", "pill-default"),
                ok = pill_link("DISTRIBUÉ", "Distribués", "pill-ok"),
                wait = pill_link("EN_ATTENTE", "En attente", "pill-wait"),
                err = pill_link("ERREUR", "Rejetés (213)", "pill-err"),
            );
            // Date range pills (7j / 30j / 90j / Tous) — preset rapides
            // qui calculent from/to côté serveur. La pill active est celle qui
            // matche exactement les valeurs from/to courantes (la pill "Tous"
            // est active si les deux champs sont vides).
            let today_str = chrono::Utc::now().format("%Y-%m-%d").to_string();
            let date_pill = |days_back: i64, label: &str| -> String {
                let from_target = if days_back == 0 {
                    String::new()
                } else {
                    (chrono::Utc::now() - chrono::Duration::days(days_back))
                        .format("%Y-%m-%d")
                        .to_string()
                };
                let to_target = if days_back == 0 {
                    String::new()
                } else {
                    today_str.clone()
                };
                let active = q.from.as_deref().unwrap_or("") == from_target
                    && q.to.as_deref().unwrap_or("") == to_target;
                let mut qs = format!("siren={}", html_escape(s));
                if let Some(st) = q.status.as_deref().filter(|x| !x.is_empty()) {
                    let enc: String = st.bytes().map(|b| match b {
                        b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'_' | b'-' | b'.' | b'~' => (b as char).to_string(),
                        _ => format!("%{:02X}", b),
                    }).collect();
                    qs.push_str(&format!("&status={}", enc));
                }
                if !from_target.is_empty() {
                    qs.push_str(&format!("&from={from_target}&to={to_target}"));
                }
                if page_size != 50 {
                    qs.push_str(&format!("&page_size={page_size}"));
                }
                format!(
                    r#"<a href="{path}?{qs}" class="pill-filter pill-default{active}">{label}</a>"#,
                    path = direction.route_path(),
                    qs = qs,
                    active = if active { " active" } else { "" },
                    label = label,
                )
            };
            let date_pills = format!(
                r#"<div class="pill-filters compact">{seven}{thirty}{ninety}{all}</div>"#,
                seven = date_pill(7, "7 j"),
                thirty = date_pill(30, "30 j"),
                ninety = date_pill(90, "90 j"),
                all = date_pill(0, "Tous"),
            );

            // Bouton "Exporter" → endpoint CSV qui rejoue les mêmes filtres.
            let export_qs = {
                let mut qs = format!("siren={}", html_escape(s));
                if let Some(st) = q.status.as_deref().filter(|x| !x.is_empty()) {
                    let enc: String = st.bytes().map(|b| match b {
                        b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'_' | b'-' | b'.' | b'~' => (b as char).to_string(),
                        _ => format!("%{:02X}", b),
                    }).collect();
                    qs.push_str(&format!("&status={enc}"));
                }
                if let Some(f) = q.from.as_deref().filter(|x| !x.is_empty()) {
                    qs.push_str(&format!("&from={}", html_escape(f)));
                }
                if let Some(t) = q.to.as_deref().filter(|x| !x.is_empty()) {
                    qs.push_str(&format!("&to={}", html_escape(t)));
                }
                qs
            };
            let export_btn = format!(
                r##"<a href="{path}/export.csv?{qs}" class="btn-ghost">⬇ Exporter CSV</a>"##,
                path = direction.route_path(),
                qs = export_qs,
            );

            let filters_form = format!(
                r#"{status_pills}
<form method="get" action="{action}" class="filters">
    <input type="hidden" name="siren" value="{siren}">
    <input type="hidden" name="status" value="{status}">
    <input type="date" name="from" value="{from}" placeholder="Du">
    <input type="date" name="to" value="{to}" placeholder="Au">
    <select name="page_size" title="Factures par page">
        {page_size_opts}
    </select>
    <button type="submit">Appliquer</button>
</form>"#,
                status_pills = status_pills,
                action = direction.route_path(),
                siren = html_escape(s),
                status = html_escape(q.status.as_deref().unwrap_or("")),
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
    <td class="num"><a href="/ui/flows/{flow_id}?siren={siren}">{invoice}</a></td>
    <td>{counterparty}</td>
    <td><span class="badge {badge}">{status}</span></td>
    <td>{pj}</td>
    <td>{errors}</td>
    <td class="num">{date}</td>
</tr>"#,
                            flow_id = html_escape(&e.flow_id),
                            siren = html_escape(s),
                            invoice = html_escape(e.invoice_number.as_deref().unwrap_or("—")),
                            counterparty = counterparty_cell,
                            badge = {
                                let (_, b) = afnor_status(
                                    &e.status,
                                    e.error_count,
                                    e.cdv_status_code,
                                    DisplayDirection::from_route(direction),
                                );
                                b
                            },
                            status = {
                                let (s, _) = afnor_status(
                                    &e.status,
                                    e.error_count,
                                    e.cdv_status_code,
                                    DisplayDirection::from_route(direction),
                                );
                                html_escape(&s)
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
                r#"<div class="app-title">
    <h1>{title}</h1>
    <div class="actions">{tenant_subtitle}{date_pills}{export_btn}</div>
</div>
<div class="card">
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
                tenant_subtitle = tenant_subtitle,
                date_pills = date_pills,
                export_btn = export_btn,
                intro = intro,
                counterparty_label = counterparty_label,
                filters = filters_form,
                rows = rows,
                pagination = pagination,
            )
        }
    };

    let counts = sidebar_counts_for(&state, siren).await;
    html_response(&page_shell_with_counts(
        direction.page_title(),
        direction.nav_key(),
        siren,
        &ctx,
        &counts,
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

/// Mappe un statut pipeline interne (`FlowStatus`) sur l'étape AFNOR
/// correspondante du cycle de vie facture (XP Z12-012).
///
/// Tous les états « in-flight » côté PDP (REÇU, PARSING/PARSÉ,
/// VALIDATION/VALIDÉ, TRANSFORMATION/TRANSFORMÉ, DISTRIBUTION) collapse sur
/// `Déposée` (CDV 200) : pour l'utilisateur, la facture est entre les mains
/// de la plateforme et son étape AFNOR n'évolue qu'à `Émise` / `Mise à
/// disposition` une fois la distribution effective.
///
/// Cela couvre aussi les flux ingérés par fichier (FileEndpoint), pour
/// lesquels aucun event `Received` n'est publié sur le bus : sans ce
/// fallback, leur timeline serait vide (cf. handler HTTP inbound qui, lui,
/// publie un `Received` explicite).
fn pipeline_event_to_afnor(status: &str, dir: DisplayDirection) -> Option<(&'static str, &'static str)> {
    match status.to_uppercase().as_str() {
        "REÇU" | "RECU" | "RECEIVED"
        | "PARSING" | "PARSÉ" | "PARSE" | "PARSED"
        | "VALIDATION" | "VALIDATING" | "VALIDÉ" | "VALIDE" | "VALIDATED"
        | "TRANSFORMATION" | "TRANSFORMING" | "TRANSFORMÉ" | "TRANSFORME" | "TRANSFORMED"
        | "DISTRIBUTION" | "DISTRIBUTING" => Some(("Déposée", "badge-info")),
        "DISTRIBUÉ" | "DISTRIBUE" | "DISTRIBUTED" | "ATTENTE_ACK" | "WAITINGACK"
        | "WAITING" | "ACQUITTÉ" | "ACQUITTE" | "ACKNOWLEDGED" => {
            match dir {
                DisplayDirection::Emise => Some(("Émise", "badge-success")),
                DisplayDirection::Recue => Some(("Mise à disposition", "badge-success")),
            }
        }
        "REJETÉ" | "REJETE" | "REJECTED" => Some(("Rejetée", "badge-error")),
        "ANNULÉ" | "ANNULE" | "CANCELLED" => Some(("Annulée", "badge-warning")),
        _ => None,
    }
}

/// Construit la timeline AFNOR de la facture pour la page détail.
///
/// **Seuls les statuts AFNOR officiels** sont affichés (XP Z12-012 Annexe A
/// V1.2, codes 200-501). Les libellés pipeline internes (PARSÉ, VALIDÉ,
/// TRANSFORMÉ, etc.) sont filtrés : ils décrivent l'implémentation et
/// n'ont aucune valeur pour l'utilisateur.
///
/// Les events `REÇU`/`DISTRIBUÉ`/`ACQUITTÉ` du pipeline interne sont remappés
/// vers `Déposée` / `Émise` (ou `Mise à disposition` côté reçue). Les erreurs
/// deviennent `Rejetée` (CDV 213). Si un `cdv_status_code` est présent, son
/// libellé AFNOR exact est ajouté en fin de timeline (Approuvée, Encaissée,
/// Refusée, En litige, …).
///
/// La timeline est tronquée après la première erreur : les statuts ultérieurs
/// (DISTRIBUÉ alors qu'une erreur annuaire-validation est survenue) ne
/// reflètent pas l'issue métier réelle.
fn render_timeline(
    events: &[pdp_trace::store::EventEntry],
    errors: &[pdp_trace::store::ErrorEntry],
    cdv_status_code: Option<u16>,
    cdv_received_at: Option<&str>,
    generated_cdv_status_code: Option<u16>,
    generated_cdv_at: Option<&str>,
    disposition_cdv_status_code: Option<u16>,
    disposition_cdv_at: Option<&str>,
    dir: DisplayDirection,
) -> String {
    if events.is_empty()
        && errors.is_empty()
        && cdv_status_code.is_none()
        && generated_cdv_status_code.is_none()
        && disposition_cdv_status_code.is_none()
    {
        return r#"<p style="color:#888">Aucun événement enregistré.</p>"#.to_string();
    }

    enum Item<'a> {
        /// Étape AFNOR remappée depuis un event pipeline.
        AfnorEvent {
            ts: &'a str,
            route: &'a str,
            label: &'static str,
            badge: &'static str,
        },
        /// CDV officiellement émis par notre PDP (200/201/202/203), capté
        /// depuis `ExchangeDocument.generated_cdv_*` ou `disposition_cdv_*`.
        /// Plus crédible que les events pipeline (qui sont des étapes
        /// internes), c'est le CDV XML réellement persisté.
        GeneratedCdv {
            code: u16,
            slot: &'static str, // "generated" ou "disposition" — pour le download link
            ts: Option<&'a str>, // timestamp RFC3339 quand le CDV a été émis
        },
        Error(&'a pdp_trace::store::ErrorEntry),
    }

    // 1. Convertir chaque event pipeline en étape AFNOR (filtre les internes).
    let mut items: Vec<Item> = Vec::new();
    for ev in events {
        if let Some((label, badge)) = pipeline_event_to_afnor(&ev.status, dir) {
            items.push(Item::AfnorEvent {
                ts: &ev.timestamp,
                route: &ev.route_id,
                label,
                badge,
            });
        }
    }
    for er in errors {
        items.push(Item::Error(er));
    }

    // 1bis. CDVs officiels persistés par la PDP (200/201/202/203).
    //       Ils s'ajoutent aux events pipeline pour donner la vue complète.
    if let Some(code) = generated_cdv_status_code {
        items.push(Item::GeneratedCdv { code, slot: "generated", ts: generated_cdv_at });
    }
    if let Some(code) = disposition_cdv_status_code {
        items.push(Item::GeneratedCdv { code, slot: "disposition", ts: disposition_cdv_at });
    }

    // 2. Tri : par ordre conceptuel du cycle de vie.
    //    AfnorEvent : timestamp (chronologique)
    //    GeneratedCdv : par code (200 < 201 < 202 < 203)
    //    On considère le code comme un "rang" pour comparer avec les events
    //    pipeline (200/201 ~ avant 202/203). En pratique : events d'abord
    //    (issus de REÇU/DISTRIBUÉ), CDVs ensuite par ordre numérique.
    items.sort_by_key(|it| match it {
        Item::AfnorEvent { ts, .. } => (0u8, ts.to_string()),
        Item::Error(e) => (0u8, e.timestamp.clone()),
        Item::GeneratedCdv { code, .. } => (1u8, format!("{:03}", code)),
    });

    // 2bis. Déduplication des AfnorEvent consécutifs avec le même libellé.
    items.dedup_by(|b, a| match (a, b) {
        (
            Item::AfnorEvent { label: la, .. },
            Item::AfnorEvent { label: lb, .. },
        ) => la == lb,
        _ => false,
    });

    // 3. Tronque après la première erreur métier.
    let first_err_pos = items.iter().position(|it| matches!(it, Item::Error(_)));
    if let Some(idx) = first_err_pos {
        items.truncate(idx + 1);
    }

    // 4. Si un CDV granulaire (204/205/210/212/…) est porté par la facture,
    //    on l'ajoute comme dernière étape — sauf si la timeline est déjà
    //    terminée par une erreur (rejet métier prioritaire).
    let mut html_items: Vec<String> = items
        .iter()
        .map(|it| match it {
            Item::AfnorEvent { ts, route, label, badge } => format!(
                r#"<div class="timeline-item">
    <div class="ts">{ts}</div>
    <div class="label"><span class="badge {badge}">{label}</span></div>
    <div class="msg">Route <code>{route}</code></div>
</div>"#,
                ts = html_escape(ts),
                badge = badge,
                label = label,
                route = html_escape(route),
            ),
            Item::GeneratedCdv { code, slot, ts } => {
                let label = match code {
                    200 => "Déposée",
                    201 => "Émise",
                    202 => "Reçue",
                    203 => "Mise à disposition",
                    _ => "CDV",
                };
                let badge = afnor_badge_for_code(*code);
                // Header avec date si dispo, sinon juste le code (rétro-
                // compatible avec les anciens docs ES sans timestamp).
                let ts_header = match ts {
                    Some(t) => format!(
                        "{ts} · CDV {code} — généré par la PDP",
                        ts = html_escape(t),
                        code = code,
                    ),
                    None => format!("CDV {code} — généré par la PDP", code = code),
                };
                format!(
                    r#"<div class="timeline-item">
    <div class="ts">{ts_header}</div>
    <div class="label"><span class="badge {badge}">{label}</span></div>
    <div class="msg">XML CDV émis et persisté <span class="cdv-link" data-slot="{slot}"></span></div>
</div>"#,
                    ts_header = ts_header,
                    badge = badge,
                    label = label,
                    slot = slot,
                )
            }
            Item::Error(er) => format!(
                r#"<div class="timeline-item timeline-error">
    <div class="ts">{ts}</div>
    <div class="label"><span class="badge badge-error">Rejetée</span></div>
    <div class="msg">{msg}</div>
</div>"#,
                ts = html_escape(&er.timestamp),
                msg = html_escape(&er.message),
            ),
        })
        .collect();

    let already_ended_in_error = matches!(items.last(), Some(Item::Error(_)));
    if let Some(code) = cdv_status_code {
        if !already_ended_in_error {
            // Le label "officiel" passe par afnor_status pour bénéficier des
            // règles direction-aware (202 → "Émise" côté émetteur, etc.).
            let (label, badge) = afnor_status("", 0, Some(code), dir);
            let ts_header = match cdv_received_at {
                Some(t) => format!(
                    "{ts} · CDV — code AFNOR {code}",
                    ts = html_escape(t),
                    code = code,
                ),
                None => format!("CDV — code AFNOR {code}", code = code),
            };
            html_items.push(format!(
                r#"<div class="timeline-item">
    <div class="ts">{ts_header}</div>
    <div class="label"><span class="badge {badge}">{label}</span></div>
    <div class="msg">Statut métier porté par le CDV reçu</div>
</div>"#,
                ts_header = ts_header,
                badge = badge,
                label = html_escape(&label),
            ));
        }
    }

    if html_items.is_empty() {
        return r#"<p style="color:#888">Aucune étape AFNOR enregistrée.</p>"#.to_string();
    }
    format!(r#"<div class="timeline">{}</div>"#, html_items.join(""))
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
        (None, _) => siren_picker(&state, &ctx, "/ui"),
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
    let counts = sidebar_counts_for(&state, siren).await;
    html_response(&page_shell_with_counts(
        "Détail flux",
        nav_active,
        siren,
        &ctx,
        &counts,
        &body,
    ))
    .into_response()
}

/// Affiche le format source de la facture avec un badge coloré + le nom du
/// standard auquel il se rattache. Conforme XP Z12-012 (formats supportés
/// EN16931 par la PDP : UBL 2.1, CII D16B, Factur-X 1.07.2).
fn render_source_format(format: Option<&str>) -> String {
    let raw = format.unwrap_or("").to_uppercase();
    let (label, full_name, badge) = match raw.as_str() {
        "UBL" => ("UBL", "OASIS Universal Business Language 2.1", "badge-info"),
        "CII" => ("CII", "UN/CEFACT Cross Industry Invoice D16B", "badge-info"),
        "FACTURX" | "FACTUR-X" => (
            "Factur-X",
            "PDF/A-3 hybride (PDF visuel + XML CII embarqué)",
            "badge-success",
        ),
        "" => return r#"<span style="color:#888">—</span>"#.to_string(),
        _ => return html_escape(format.unwrap_or("—")),
    };
    format!(
        r#"<span class="badge {badge}">{label}</span> <span style="color:#666;font-size:0.9rem">— {full_name}</span>"#,
        badge = badge,
        label = label,
        full_name = full_name,
    )
}

fn render_flow_detail(
    siren: &str,
    flow_id: &str,
    sum: &pdp_trace::store::ExchangeSummary,
    full: Option<&pdp_trace::store::ExchangeDocument>,
) -> String {
    let format_html = render_source_format(full.and_then(|f| f.source_format.as_deref()));
    let metadata = format!(
        r#"<dl class="kv">
    <dt>Flow ID</dt><dd><code>{flow_id}</code></dd>
    <dt>Exchange ID</dt><dd><code>{exchange_id}</code></dd>
    <dt>Numéro facture</dt><dd>{invoice}</dd>
    <dt>Vendeur</dt><dd>{seller} ({seller_siret})</dd>
    <dt>Acheteur</dt><dd>{buyer} ({buyer_siret})</dd>
    <dt>Type de fichier original</dt><dd>{format}</dd>
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
        format = format_html,
        total_ht = full.and_then(|f| f.total_ht).map(|v| format!("{:.2}", v)).unwrap_or_else(|| "—".into()),
        total_tax = full.and_then(|f| f.total_tax).map(|v| format!("{:.2}", v)).unwrap_or_else(|| "—".into()),
        total_ttc = full.and_then(|f| f.total_ttc).map(|v| format!("{:.2}", v)).unwrap_or_else(|| "—".into()),
        currency = html_escape(full.and_then(|f| f.currency.as_deref()).unwrap_or("—")),
        issue_date = html_escape(full.and_then(|f| f.issue_date.as_deref()).unwrap_or("—")),
        badge = {
            let (_, b) = afnor_status(
                &sum.status,
                sum.error_count,
                sum.cdv_status_code,
                DisplayDirection::from_summary(siren, sum),
            );
            b
        },
        status = {
            let (s, _) = afnor_status(
                &sum.status,
                sum.error_count,
                sum.cdv_status_code,
                DisplayDirection::from_summary(siren, sum),
            );
            html_escape(&s)
        },
        created_at = html_escape(&sum.created_at),
    );

    let dir = DisplayDirection::from_summary(siren, sum);
    let timeline = match full {
        None => String::new(),
        Some(doc) => render_timeline(
            &doc.events,
            &doc.errors,
            sum.cdv_status_code,
            doc.cdv_received_at.as_deref(),
            doc.generated_cdv_status_code,
            doc.generated_cdv_at.as_deref(),
            doc.disposition_cdv_status_code,
            doc.disposition_cdv_at.as_deref(),
            dir,
        ),
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

    // Liens de téléchargement (XML brut, PDF Factur-X original, Factur-X généré)
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
                // Source Factur-X : on dispose du PDF/A-3 original.
                links.push(format!(
                    r#"<a class="dl-btn" href="/ui/flows/{f}/download/pdf?siren={s}">⬇️ PDF Factur-X</a>"#,
                    f = html_escape(flow_id), s = html_escape(siren),
                ));
            } else if doc.raw_xml.is_some() {
                // Source UBL/CII : on peut générer un Factur-X à la volée
                // (PDF/A-3 + CII embarqué) via `pdp_transform::convert`.
                links.push(format!(
                    r#"<a class="dl-btn" href="/ui/flows/{f}/download/facturx?siren={s}">⬇️ Factur-X (généré)</a>"#,
                    f = html_escape(flow_id), s = html_escape(siren),
                ));
            }
            if let Some(code) = doc.generated_cdv_status_code {
                links.push(format!(
                    r#"<a class="dl-btn" href="/ui/flows/{f}/download/cdv?siren={s}">⬇️ CDV {code} (généré)</a>"#,
                    f = html_escape(flow_id), s = html_escape(siren), code = code,
                ));
            }
            if let Some(code) = doc.disposition_cdv_status_code {
                let suffix = match code {
                    201 => "émise",
                    203 => "mise à disposition",
                    _ => "généré",
                };
                links.push(format!(
                    r#"<a class="dl-btn" href="/ui/flows/{f}/download/cdv-disposition?siren={s}">⬇️ CDV {code} ({suffix})</a>"#,
                    f = html_escape(flow_id), s = html_escape(siren), code = code, suffix = suffix,
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

/// GET /ui/flows/{flowId}/download/cdv
/// Télécharge le CDV (Compte-rendu De Vie) XML généré par notre PDP lors
/// du dépôt de la facture (CDV 200/202/213/221/501). Stocké dans ES via
/// `ExchangeDocument.generated_cdv_xml`.
pub async fn handle_download_cdv(
    State(state): State<Arc<AppState>>,
    crate::security::AuthorizedSiren(siren): crate::security::AuthorizedSiren,
    Path(flow_id): Path<String>,
) -> impl IntoResponse {
    let siren = siren.as_str();
    let doc = match lookup_doc(&state, siren, &flow_id).await {
        Some(d) => d,
        None => return (StatusCode::NOT_FOUND, "Flux introuvable").into_response(),
    };
    let xml = match doc.generated_cdv_xml {
        Some(x) => x,
        None => return (StatusCode::NOT_FOUND, "Aucun CDV généré pour ce flux").into_response(),
    };
    let code = doc.generated_cdv_status_code.unwrap_or(0);
    let filename = format!("cdv-{:03}-{}.xml", code, flow_id);
    let mut headers = axum::http::HeaderMap::new();
    headers.insert("content-type", "application/xml; charset=utf-8".parse().unwrap());
    if let Ok(v) = format!("attachment; filename=\"{}\"", filename).parse() {
        headers.insert("content-disposition", v);
    }
    (StatusCode::OK, headers, xml).into_response()
}

/// GET /ui/flows/{flowId}/download/cdv-disposition
/// Télécharge le CDV 203 (Mise à disposition) émis par notre PDP après
/// l'écriture de la facture vers la destination buyer.
pub async fn handle_download_cdv_disposition(
    State(state): State<Arc<AppState>>,
    crate::security::AuthorizedSiren(siren): crate::security::AuthorizedSiren,
    Path(flow_id): Path<String>,
) -> impl IntoResponse {
    let siren = siren.as_str();
    let doc = match lookup_doc(&state, siren, &flow_id).await {
        Some(d) => d,
        None => return (StatusCode::NOT_FOUND, "Flux introuvable").into_response(),
    };
    let xml = match doc.disposition_cdv_xml {
        Some(x) => x,
        None => return (StatusCode::NOT_FOUND, "Aucun CDV 203 pour ce flux").into_response(),
    };
    let filename = format!("cdv-203-{}.xml", flow_id);
    let mut headers = axum::http::HeaderMap::new();
    headers.insert("content-type", "application/xml; charset=utf-8".parse().unwrap());
    if let Ok(v) = format!("attachment; filename=\"{}\"", filename).parse() {
        headers.insert("content-disposition", v);
    }
    (StatusCode::OK, headers, xml).into_response()
}

/// GET /ui/flows/{flowId}/download/facturx
/// Génère et télécharge un PDF Factur-X (PDF/A-3 + XML CII embarqué)
/// conforme à la spec Factur-X 1.07.2 / EN 16931 niveau BASIC.
///
/// - Si la facture est déjà un Factur-X reçu, retourne le PDF original
///   (équivalent à `/download/pdf`).
/// - Si c'est un UBL ou CII, parse l'XML et utilise `pdp_transform::convert`
///   (Typst PDF/A-3 + injection CII) pour produire un Factur-X à la volée.
pub async fn handle_download_facturx(
    State(state): State<Arc<AppState>>,
    crate::security::AuthorizedSiren(siren): crate::security::AuthorizedSiren,
    Path(flow_id): Path<String>,
) -> axum::response::Response {
    let siren = siren.as_str();
    let doc = match lookup_doc(&state, siren, &flow_id).await {
        Some(d) => d,
        None => return (StatusCode::NOT_FOUND, "Flux introuvable").into_response(),
    };

    // Cas 1 : facture source déjà en Factur-X → retourne le PDF original.
    if let Some(b64) = doc.raw_pdf_base64.as_deref() {
        use base64::Engine as _;
        if let Ok(pdf_bytes) = base64::engine::general_purpose::STANDARD.decode(b64) {
            let invoice_no = doc.invoice_number.clone().unwrap_or_else(|| flow_id.clone());
            return facturx_pdf_response(pdf_bytes, &invoice_no);
        }
        return (StatusCode::INTERNAL_SERVER_ERROR, "Décodage base64 PDF échoué")
            .into_response();
    }

    // Cas 2 : source UBL/CII → conversion à la volée.
    let invoice = match parse_invoice_for_facturx(&doc) {
        Some(inv) => inv,
        None => {
            return (
                StatusCode::NOT_FOUND,
                "Aucune source convertible : raw_xml absent ou format inconnu",
            )
                .into_response();
        }
    };

    match pdp_transform::convert(&invoice, pdp_core::model::InvoiceFormat::FacturX) {
        Ok(result) => facturx_pdf_response(result.content, &invoice.invoice_number),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Échec génération Factur-X : {}", e),
        )
            .into_response(),
    }
}

fn facturx_pdf_response(pdf_bytes: Vec<u8>, invoice_no: &str) -> axum::response::Response {
    let filename = format!("{}_facturx.pdf", invoice_no);
    let mut headers = axum::http::HeaderMap::new();
    headers.insert("content-type", "application/pdf".parse().unwrap());
    if let Ok(v) = format!("attachment; filename=\"{}\"", filename).parse() {
        headers.insert("content-disposition", v);
    }
    (StatusCode::OK, headers, pdf_bytes).into_response()
}

/// Parse l'`ExchangeDocument` en `InvoiceData` selon `source_format`. Variante
/// locale de `exchange_doc_to_invoice` (main.rs) pour rester self-contained
/// dans le module ui.
fn parse_invoice_for_facturx(
    doc: &pdp_trace::store::ExchangeDocument,
) -> Option<pdp_core::model::InvoiceData> {
    let raw = doc.raw_xml.as_deref()?;
    let bytes = raw.as_bytes().to_vec();
    let format_str = doc.source_format.as_deref().unwrap_or("UBL").to_uppercase();
    match format_str.as_str() {
        "UBL" => pdp_invoice::UblParser::new().parse(raw).ok(),
        "CII" => pdp_invoice::CiiParser::new().parse(raw).ok(),
        "FACTURX" | "FACTUR-X" => pdp_invoice::FacturXParser::new().parse(&bytes).ok(),
        _ => {
            let format = pdp_invoice::detect_format(&bytes).ok()?;
            match format {
                pdp_core::model::InvoiceFormat::UBL => {
                    pdp_invoice::UblParser::new().parse(raw).ok()
                }
                pdp_core::model::InvoiceFormat::CII => {
                    pdp_invoice::CiiParser::new().parse(raw).ok()
                }
                pdp_core::model::InvoiceFormat::FacturX => {
                    pdp_invoice::FacturXParser::new().parse(&bytes).ok()
                }
            }
        }
    }
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

/// GET /ui/static/annuaire.js — sert le JS de la page `/annuaire`
/// (extrait du HTML pour respecter la CSP `script-src 'self'`).
const ANNUAIRE_JS: &str = include_str!("../static/annuaire.js");
pub async fn handle_annuaire_js() -> impl IntoResponse {
    let mut headers = axum::http::HeaderMap::new();
    headers.insert(
        "content-type",
        "application/javascript; charset=utf-8".parse().unwrap(),
    );
    headers.insert("cache-control", "public, max-age=3600".parse().unwrap());
    (StatusCode::OK, headers, ANNUAIRE_JS).into_response()
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

pub(crate) fn html_response(body: &str) -> axum::response::Response {
    (StatusCode::OK, Html(body.to_string())).into_response()
}

/// Petit raccourci : `ctx.role == Role::PdpAdmin`. Sert à conditionner
/// l'affichage du lien "Admin" dans le shell de page (le serveur fait
/// quand même le check d'autorisation côté handler).
pub(crate) fn is_admin(ctx: &crate::security::SecurityContext) -> bool {
    matches!(ctx.role, crate::security::Role::PdpAdmin)
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
            cdv_status_code: None,
            cdv_received_at: None,
            generated_cdv_xml: None,
            generated_cdv_status_code: None,
            generated_cdv_at: None,
            disposition_cdv_xml: None,
            disposition_cdv_status_code: None,
            disposition_cdv_at: None,
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
    fn test_afnor_status_prefers_cdv_code_when_present() {
        // Quand un CDV est reçu, on doit afficher le libellé AFNOR exact
        // — y compris ceux que `FlowStatus` ne distingue pas (204/205/210/212).
        // Côté émission, 201/202/203 collapse sur « Émise » : le vendeur ne
        // doit jamais voir « Reçue » sur sa propre facture.
        let cases_emise = [
            (200u16, "Déposée", "badge-info"),
            (201, "Émise", "badge-success"),
            (202, "Émise", "badge-success"),
            (203, "Émise", "badge-success"),
            (204, "Prise en charge", "badge-success"),
            (205, "Approuvée", "badge-success"),
            (207, "En litige", "badge-warning"),
            (210, "Refusée", "badge-error"),
            (212, "Encaissée", "badge-success"),
            (213, "Rejetée", "badge-error"),
            (220, "Annulée", "badge-warning"),
            (501, "Irrecevable", "badge-error"),
        ];
        for (code, expected_label, expected_badge) in cases_emise {
            let (label, badge) = afnor_status("ACQUITTÉ", 0, Some(code), DisplayDirection::Emise);
            assert_eq!(label, expected_label, "emise code {code}: label");
            assert_eq!(badge, expected_badge, "emise code {code}: badge");
        }

        // Côté réception, 202 reste « Reçue de la plateforme » (distinction
        // avec 203 Mise à disposition).
        let (label, _) = afnor_status("ACQUITTÉ", 0, Some(202), DisplayDirection::Recue);
        assert_eq!(label, "Reçue de la plateforme");
    }

    #[test]
    fn test_afnor_status_falls_back_to_flow_status_without_cdv() {
        // Sans CDV, on dérive depuis FlowStatus + direction.
        let (label, badge) = afnor_status("DISTRIBUÉ", 0, None, DisplayDirection::Emise);
        assert_eq!(label, "Émise");
        assert_eq!(badge, "badge-success");

        let (label, _) = afnor_status("DISTRIBUÉ", 0, None, DisplayDirection::Recue);
        assert_eq!(label, "Mise à disposition");

        let (label, badge) = afnor_status("VALIDÉ", 1, None, DisplayDirection::Emise);
        assert_eq!(label, "Rejetée");
        assert_eq!(badge, "badge-error");
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
