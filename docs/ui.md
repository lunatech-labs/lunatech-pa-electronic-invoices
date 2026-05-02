# Interface web — Suivi des factures (Phase 1)

Ferrite expose une interface HTML server-rendered pour permettre aux clients
et à l'administrateur PDP de consulter l'état des factures sans utiliser
l'API HTTP directement. Phase 1 = lecture seule (dashboard + liste + détail).

## URLs

| Route | Description | Bearer requis |
|-------|-------------|---------------|
| `GET /ui` | Dashboard avec KPIs du tenant | Non (public) |
| `GET /ui/flows` | Liste paginée + filtres (direction, statut, dates) | Non |
| `GET /ui/flows/{flowId}` | Détail facture + PJ + timeline pipeline | Non |
| `GET /ui/flows/{flowId}/download/xml` | Télécharge le XML brut UBL/CII | Non |
| `GET /ui/flows/{flowId}/download/pdf` | Télécharge le PDF Factur-X | Non |
| `GET /ui/flows/{flowId}/download/attachment?idx=N` | Télécharge la N-ième PJ extraite à la volée | Non |
| `GET /annuaire` | UI de recherche annuaire (déjà existante) | Non |

## Démarrage rapide

```bash
# 1. Démarrer Elasticsearch (ou OpenSearch — API compatible, ARM-friendly)
docker run -d --name pdp-es -p 9200:9200 \
  -e "discovery.type=single-node" -e "DISABLE_SECURITY_PLUGIN=true" \
  opensearchproject/opensearch:2

# 2. Démarrer Ferrite (mode receiver, config minimale fournie)
cargo run --bin pdp -- --config config-ui-demo.yaml start --mode receiver

# 3. Peupler le dashboard avec toutes les fixtures (UBL + CII)
cargo run --bin pdp -- demo populate
# 📦 23 fixtures à soumettre
#   ✅ autofacture_cii_389.xml (CII)
#   ✅ avoir_cii_381.xml (CII)
#   ...
# 📊 Soumis : 23/23 (0 erreurs)

# 4. Ouvrir un navigateur (attendre ~60s pour le polling)
open http://localhost:8080/ui?siren=123456789
```

**Config minimale `config-ui-demo.yaml`** : un fichier prêt à l'emploi est
fourni à la racine du repo (sans Postgres ni routes complexes).

## Peupler le dashboard

```bash
# Toutes les fixtures (~23 factures sur 8 tenants distincts)
pdp demo populate

# Avec serveur distant
pdp demo populate --server-url https://pdp.example.com

# Avec Bearer token si auth activée
pdp demo populate --token mon-token

# Avec un répertoire personnalisé (cherche dans ./mes-fixtures/ubl/ et ./mes-fixtures/cii/)
pdp demo populate --fixtures-dir ./mes-fixtures
```

Les fixtures couvrent plusieurs SIREN (123456789, 222333444, 444555666,
111222333, 333444555, 512345678, 456789012, 333444555) — un index ES par
tenant. Naviguer entre tenants via `?siren=...` dans l'URL.

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
N° facture · Vendeur · Acheteur · Statut (badge coloré) · **PJ** (badge 📎 N) · Erreurs · Date.

La colonne **PJ** affiche `📎 N` quand la facture a N pièces jointes
(`attachment_count` indexé en ES) ou `—` sinon. Cliquer la facture pour
voir le détail des PJ (extraction à la volée du `raw_xml` / `raw_pdf`).

Filtres :
- **Direction** — `Émises (vendeur)` / `Reçues (acheteur)` / toutes. Émises = le
  tenant est `seller_siren`, Reçues = le tenant est `buyer_siren`.
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

Cinq sections :

1. **Métadonnées** — flowId, exchangeId, n° facture, vendeur/acheteur (nom + SIRET), format (UBL/CII/Factur-X), totaux HT/TVA/TTC, devise, date émission, statut (badge), date de réception
2. **Erreurs** (si `error_count > 0`) — liste step + message
3. **Téléchargements** — boutons :
   - `⬇️ XML brut` (si `raw_xml` présent) → `GET /ui/flows/{flowId}/download/xml`
   - `⬇️ PDF Factur-X` (si `raw_pdf_base64` présent) → `GET /ui/flows/{flowId}/download/pdf`
4. **Pièces jointes** — extraites **à la volée** depuis `raw_xml` (UBL/CII)
   ou `raw_pdf_base64` (Factur-X). Affiche un tableau ID · Fichier · Description ·
   MIME · Taille · ⬇️. Si `raw_xml` indisponible, fallback sur la liste indexée
   (`attachment_filenames`). Téléchargement individuel via
   `GET /ui/flows/{flowId}/download/attachment?siren=...&idx=N`.
   **Les PJ ne sont pas stockées en base** — l'extraction est faite par les
   parsers UBL/CII/Factur-X au moment de l'affichage.
5. **Timeline du pipeline** — événements horodatés par route et statut

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
