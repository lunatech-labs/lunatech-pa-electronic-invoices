# Interface web — Suivi des factures (Phase 1)

Ferrite expose une interface HTML server-rendered pour permettre aux clients
et à l'administrateur PDP de consulter l'état des factures sans utiliser
l'API HTTP directement. Phase 1 = lecture seule (dashboard + liste + détail).

## URLs

| Route | Description | Bearer requis |
|-------|-------------|---------------|
| `GET /ui` | Dashboard avec KPIs du tenant | Non (public) |
| `GET /ui/flows` | Liste paginée des factures avec filtres | Non |
| `GET /ui/flows/{flowId}` | Détail facture + timeline pipeline | Non |
| `GET /annuaire` | UI de recherche annuaire (déjà existante) | Non |

## Démarrage rapide

```bash
# 1. Démarrer le serveur (mode receiver ou both)
cargo run --bin pdp -- start --config config.yaml --mode receiver

# 2. Ouvrir un navigateur
open http://localhost:8080/ui?siren=123456789
```

## Multi-tenant

L'index Elasticsearch est par tenant (`pdp-{SIREN}`). Toutes les pages
prennent un paramètre query `?siren=123456789` pour cibler le tenant.

Sans paramètre, un sélecteur SIREN s'affiche pour choisir le tenant à charger.

> **Phase 1 — admin only** : pas d'authentification web, l'accès est contrôlé
> au niveau réseau (proxy / firewall). L'authentification utilisateur (login,
> session) viendra dans une phase ultérieure.

## Pages

### Dashboard `/ui?siren={SIREN}`

KPIs calculés à partir de l'index Elasticsearch `pdp-{SIREN}` :
- **Total flux** — `count(*)` de tous les exchanges
- **Distribués** — exchanges avec `status = DISTRIBUÉ` (succès)
- **En attente** — exchanges ni distribués ni en erreur
- **En erreur** — exchanges avec `error_count > 0`

Plus une section "Actions" avec liens rapides vers la liste, les erreurs,
le healthcheck et les métriques Prometheus.

### Liste `/ui/flows?siren={SIREN}`

Tableau paginé (50 par page) avec colonnes :
N° facture · Vendeur · Acheteur · Statut (badge coloré) · Erreurs · Date.

Filtres :
- **Statut** — `DISTRIBUÉ` / `ERREUR` / `EN_ATTENTE` / tous
- **Du / Au** — bornes sur `issue_date` (format `YYYY-MM-DD`)

Pagination : navigation Précédent / Suivant en bas de page.

Exemples d'URLs :
```
/ui/flows?siren=123456789&status=ERREUR
/ui/flows?siren=123456789&from=2025-11-01&to=2025-11-30
/ui/flows?siren=123456789&page=2
```

### Détail `/ui/flows/{flowId}?siren={SIREN}`

Quatre sections :

1. **Métadonnées** — flowId, exchangeId, n° facture, vendeur/acheteur (nom + SIRET), format (UBL/CII/Factur-X), totaux HT/TVA/TTC, devise, date émission, statut (badge), date de réception
2. **Erreurs** (si `error_count > 0`) — liste step + message
3. **Pièces jointes** — extraites **à la volée** depuis `raw_xml` (UBL/CII)
   ou `raw_pdf_base64` (Factur-X). Affiche un tableau ID · Fichier · Description ·
   MIME · Taille. Si `raw_xml` indisponible, fallback sur la liste indexée
   (`attachment_filenames`). **Les PJ ne sont pas stockées en base** —
   l'extraction est faite par les parsers UBL/CII/Factur-X au moment de l'affichage.
4. **Timeline du pipeline** — événements horodatés par route et statut

Le `flow_id` est accepté ou son alias `exchange_id`.

## Architecture technique

```
crates/pdp-app/src/ui.rs       — handlers HTML (3 fonctions)
crates/pdp-app/src/server.rs   — wiring routes /ui* dans build_api_router
crates/pdp-trace/src/store.rs  — list_exchanges() + get_stats_for_siren()
                                 (méthodes ajoutées pour l'UI)
```

**Stack** :
- HTML server-rendered (cohérent avec `/annuaire`)
- CSS inline (300 lignes, thème sombre/clair, responsive)
- Pas de framework JS — HTMX optionnel pour les futures interactions
- Pas de build front, pas de bundler

**Pourquoi server-rendered ?**
- Cohérent avec `/annuaire` existant
- Zero build pour le développement
- Indexable, accessible, fonctionne sans JavaScript
- Peut évoluer vers HTMX (interactions partielles) ou full SPA si besoin

## Sources de données

| Source | Usage |
|--------|-------|
| Elasticsearch `pdp-{siren}` | KPIs (count), liste paginée, détail (raw_xml + events + errors) |
| `TraceStore::get_stats_for_siren()` | Dashboard counters |
| `TraceStore::list_exchanges()` | Liste paginée avec filtres |
| `TraceStore::get_exchange()` | Détail facture (ExchangeDocument complet) |
| `parse_attachments_from_doc()` (ui.rs) | PJ extraites à la volée — `UblParser` / `CiiParser` / `FacturXParser` selon `source_format`. Pas de stockage des PJ en base. |

## Captures (référence)

```
┌──────────────────────────────────────────────────────────┐
│ Ferrite — Suivi des factures                             │
│ Dashboard  Factures  Annuaire                            │
├──────────────────────────────────────────────────────────┤
│  Tenant : 123456789                                      │
│                                                          │
│  ┌────────────┐ ┌────────────┐ ┌────────────┐ ┌────────┐│
│  │ Total flux │ │ Distribués │ │ En attente │ │ Erreur ││
│  │   1247     │ │   1198     │ │     13     │ │   36   ││
│  └────────────┘ └────────────┘ └────────────┘ └────────┘│
│                                                          │
│  Actions                                                 │
│  → Voir toutes les factures                              │
│  → Voir uniquement les erreurs                           │
└──────────────────────────────────────────────────────────┘
```

## Phases suivantes (todo §3bis)

| Phase | Contenu |
|-------|---------|
| **1 (livré)** | Lecture seule : dashboard + liste + détail + timeline |
| **2** | Soumission factures (upload UBL/CII/Factur-X), émission CDV manuels |
| **3** | Admin PDP (multi-tenant, alertes, métriques) |
| **4** | Notifications live (WebSocket / SSE) |

## Voir aussi

- [docs/http-api.md](http-api.md) — API REST complète (curl + OpenAPI)
- [docs/tracabilite.md](tracabilite.md) — Architecture Elasticsearch (un index par SIREN)
- [docs/workflows.md](workflows.md) — Workflows métier (UC1-UC5)
- [docs/todo.md §3bis](todo.md) — Roadmap interface web (4 phases)
