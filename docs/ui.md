# Interface web — Suivi des factures (Phase 1)

Ferrite expose une interface HTML server-rendered pour permettre aux clients
et à l'administrateur PDP de consulter l'état des factures sans utiliser
l'API HTTP directement. Phase 1 = lecture seule (dashboard + liste + détail).

## Authentification & isolation tenant

L'UI est protégée par l'`auth_middleware` (cf.
[crates/pdp-app/src/security.rs](../crates/pdp-app/src/security.rs)). Deux
voies d'authentification, dans l'ordre de résolution du middleware :

1. **Cookie de session** `ferrite_session` — issu du formulaire `/login`.
   Le cookie est HMAC-signé (HttpOnly, SameSite=Lax) et porte le
   `principal` du user + une expiration. Le `SecurityContext` est
   reconstruit à partir de `state.users` à chaque requête (lookup
   in-memory, pas de DB). TTL configurable (`session_ttl_secs`, défaut 8h).
2. **Bearer token** (clients API) — header `Authorization: Bearer <token>`,
   cherché dans `state.tokens` (table en mémoire alimentée par la config
   `http_server.tokens:`).

Pas de mode "bypass d'auth" : même la démo locale passe par `/login` avec
des utilisateurs de démo prédéfinis dans `config-ui-demo.yaml`.

Tout `?siren=X` passé en query est validé par l'extractor `AuthorizedSiren`
contre le `SecurityContext` du porteur :

| Cas | Route UI (`/ui/*`) | Route API (`/v1/*`) |
|---|---|---|
| Pas d'auth (mode normal) | `303 → /login?next=...` | `401 MISSING_TOKEN` |
| Cookie / token invalide | `303 → /login` | `401` |
| `?siren=` absent sur route obligatoire | `400 SIREN_REQUIRED` | `400 SIREN_REQUIRED` |
| `?siren=X` hors `allowed_sirens` (rôle `tenant`) | `403 SIREN_NOT_AUTHORIZED` | `403` |
| Rôle `pdp_operator` ou `pdp_admin` | passe quel que soit le SIREN | idem |

### Configuration tokens / users

```yaml
http_server:
  session_secret: "<32+ octets aléatoires, gardé secret>"
  session_ttl_secs: 28800   # 8h par défaut

  # Comptes pour le login web. Le `password` peut être :
  #  - un hash argon2id : `$argon2id$v=19$m=19456,t=2,p=1$...`
  #    (recommandé — généré via `pdp tools hash-password "..."`)
  #  - un mot de passe en clair (legacy v1, log un warning au démarrage)
  users:
    - email: "alice@techconseil.fr"
      password: "$argon2id$v=19$m=19456,t=2,p=1$WGarTI..."
      principal: "alice@tc"
      allowed_sirens: ["123456789"]
      role: tenant

  # Tokens Bearer pour les clients API
  tokens:
    - token: "tok-techconseil-prod"
      principal: "techconseil-app"
      allowed_sirens: ["123456789"]
      role: tenant
    - token: "tok-pdp-support"
      principal: "support-team"
      role: pdp_operator
```

### Routes publiques (hors auth)

- `/v1/healthcheck`, `/metrics` — supervision
- `/login`, `/logout` — formulaire de connexion (sinon impossible de
  se logger…)
- `/annuaire`, `/v1/annuaire/search` — choix produit explicite : la
  recherche annuaire reste accessible à tous (un assujetti consulte
  un fournisseur potentiel sans connexion)
- `/favicon.ico`, `/favicon.png`, `/ui/static/*` — assets

### Headers de sécurité

Tous les responses portent (ajoutés par `security_headers_middleware`) :

- `Content-Security-Policy: default-src 'self'; …; frame-ancestors 'none'`
- `Strict-Transport-Security: max-age=31536000; includeSubDomains`
- `X-Frame-Options: DENY`
- `X-Content-Type-Options: nosniff`
- `Referrer-Policy: strict-origin-when-cross-origin`

### Outils CLI

```bash
# Génère un hash argon2id pour un mot de passe (à coller dans users[].password)
pdp tools hash-password "monMotDePasse"
$argon2id$v=19$m=19456,t=2,p=1$WGarTI...

# Lit le mot de passe sur stdin (pour ne pas l'avoir dans l'historique shell)
echo -n "secret" | pdp tools hash-password -

# Génère un secret de session aléatoire (32 octets, base64) pour `session_secret`
pdp tools gen-session-secret
z6qZ/9XHRyuyOKL/cnuFu5nGpTgRzWzuNFEeLff+jtM=
```

### Détails session (Phase B.5)

- **Cookie HMAC-signé** stateless : `<principal_b64>.<expires_at>.<sig>`,
  signé avec `session_secret` (32+ octets).
- **`Secure` flag** posé automatiquement quand le proxy injecte
  `X-Forwarded-Proto: https`. En HTTP brut (démo locale), le flag est
  omis pour que le navigateur envoie quand même le cookie.
- **Logout server-side** : `POST /logout` ajoute la signature à une
  *revocation list* in-memory (`Mutex<HashMap<sig, expires_at>>`,
  bornée à 5000 entrées). Toute requête ultérieure qui rejouerait la
  signature est rejetée jusqu'à expiration naturelle. La liste est
  perdue au redémarrage (acceptable : les cookies expirent aussi car
  un `session_secret` non-fixe change toutes les sessions).

Côté store, [`TraceBackend::get_exchange`](../crates/pdp-trace/src/backend.rs)
ajoute un filtre `seller_siren = X OR buyer_siren = X` au lookup par
`exchange_id` : un client qui devine un ID arbitraire mais ne porte pas le
bon SIREN obtient `None` (`404` côté API), pas le document.

Tests d'isolation : [crates/pdp-app/tests/security_test.rs](../crates/pdp-app/tests/security_test.rs)
(11 tests, sans dépendance externe).


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
# Toutes les fixtures (manuelles + bulk générées) — ~280 factures
pdp demo populate

# Reset des indices ES avant soumission (état propre)
pdp demo populate --reset

# Avec serveur distant
pdp demo populate --server-url https://pdp.example.com

# Avec Bearer token si auth activée
pdp demo populate --token mon-token

# Avec un répertoire personnalisé (cherche dans ./mes-fixtures/ubl/ et ./mes-fixtures/cii/)
pdp demo populate --fixtures-dir ./mes-fixtures
```

**Génération de fixtures** :

```bash
# Fixtures manuelles ciblées (~12 factures dont 3 en erreur, mix émises/reçues, avec/sans PJ)
python3 tools/gen-fixtures-recues.py

# Fixtures bulk pour tester la pagination (240 factures par défaut)
python3 tools/gen-fixtures-bulk.py
python3 tools/gen-fixtures-bulk.py --count 1000
```

Le bulk génère des factures réparties sur ~24 partenaires de l'annuaire
F14_demo, ~10% avec PJ, ~3% avec un partenaire absent de l'annuaire
(déclenche EMMET_INC en réception). Les dates sont étalées sur 12 mois
(juin 2025 → mai 2026) pour pouvoir tester le filtre par plage.

Tenant principal de la démo : **123456789** (TechConseil SAS), à la fois
émetteur et destinataire dans les fixtures.

## Multi-tenant

L'index Elasticsearch est par tenant (`pdp-{SIREN}`). Toutes les pages
prennent un paramètre query `?siren=123456789` pour cibler le tenant.

Sans paramètre, un sélecteur SIREN s'affiche pour choisir le tenant à charger.

> **Phase 1 — admin only** : pas d'authentification web, l'accès est contrôlé
> au niveau réseau (proxy / firewall). L'authentification utilisateur (login,
> session) viendra dans une phase ultérieure.

## Pages

### Dashboard `/ui?siren={SIREN}`

Le tenant est identifié comme **vendeur OU acheteur** d'un flux (un même
SIREN voit donc ses émissions et ses réceptions, même si l'index ES est
keyé sur le seller_siren).

KPIs :
- **Total flux** — toutes les factures dont le tenant est partie
- **Distribués** — `error_count = 0` ET status terminal réussi (`VALIDÉ`,
  `TRANSFORMÉ`, `DISTRIBUTION`, `DISTRIBUÉ`, `ATTENTE_ACK`, `ACQUITTÉ`)
- **En erreur** — `error_count > 0` OU status terminal d'échec
  (`REJETÉ`, `ANNULÉ`, `ERREUR`)
- **En attente** — ni terminés ni en erreur (encore en pipeline)

> Les KPIs sont alignés avec les filtres de la liste : un dashboard
> "Distribués: 14" correspond exactement à `?status=OK` qui retourne 14.

### Liste `/ui/flows?siren={SIREN}`

Tableau paginé avec colonnes :
N° facture · Vendeur · Acheteur · Statut (badge coloré) · **PJ** (badge 📎 N) · Erreurs · Date.

La colonne **PJ** affiche `📎 N` quand la facture a N pièces jointes
(`attachment_count` indexé en ES) ou `—` sinon.

Le **statut affiché est dérivé** de `error_count` : si `error_count > 0`,
le badge montre **ERREUR** (rouge) quel que soit le statut brut du
pipeline — celui-ci continue après une erreur non bloquante mais l'UI
ne doit pas laisser croire qu'une facture rejetable est "DISTRIBUÉE".

Filtres :
- **Direction** — `Émises (vendeur)` / `Reçues (acheteur)` / toutes. Émises = le
  tenant est `seller_siren`, Reçues = le tenant est `buyer_siren`.
- **Statut** — `OK` / `ERREUR` / `EN_ATTENTE` / tous (correspond aux KPIs
  du dashboard, cf. ci-dessus)
- **Du / Au** — bornes sur `issue_date` (format `YYYY-MM-DD`)

Les champs vides du formulaire (`?status=&from=...`) sont normalisés en
**`None`** côté handler — sans cela ES voit `term: { "status": "" }` et
ne renvoie rien.

**Pagination** :
- Sélecteur **Factures par page** dans le formulaire de filtres : **25 / 50 / 100 / 200**.
  Toute autre valeur (ex. `?page_size=10000`) retombe sur 50 (sécurité).
- Bandeau bas de tableau : `1–50 sur 257 factures · page 1/6` + boutons
  Précédent / Suivant. Le total respecte les filtres actifs.
- Le `page_size` choisi est préservé dans les liens Précédent / Suivant.

Exemples d'URLs :
```
/ui/flows?siren=123456789&status=ERREUR
/ui/flows?siren=123456789&from=2025-11-01&to=2025-11-30
/ui/flows?siren=123456789&page=2&page_size=100
/ui/flows?siren=123456789&direction=recues
/ui/flows?siren=123456789&show_duplicates=true   # désactive la dedup par invoice_number
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
5. **Timeline du pipeline** — événements horodatés par route et statut.
   Les `events` (REÇU / PARSÉ / VALIDÉ / DISTRIBUÉ…) sont **fusionnés
   chronologiquement** avec les `errors`, et la timeline est **tronquée à la
   première erreur** (les statuts pipeline qui suivent — DISTRIBUÉ après
   un EMMET_INC par ex. — ne reflètent pas l'issue métier de la facture).
   Les entrées d'erreur s'affichent en rouge avec ❌ + le step (`annuaire-validation`,
   `validation`, etc.) et le message.

Le `flow_id` est accepté ou son alias `exchange_id`.

### Statuts du pipeline

Les statuts (`crates/pdp-core/src/model.rs::FlowStatus`) :

| Statut | Étape | Sens |
|---|---|---|
| `REÇU` | Réception | Fichier reçu (HTTP/SFTP), avant tout traitement |
| `PARSING` → `PARSÉ` | Parsing | Extraction XML → `InvoiceData` |
| `VALIDATION` → `VALIDÉ` | Validation | XSD + EN16931 + BR-FR + annuaire passés |
| `TRANSFORMATION` → `TRANSFORMÉ` | Transformation | Conversion UBL ↔ CII si demandée |
| `DISTRIBUTION` → `DISTRIBUÉ` | Routage sortie | Délivré au destinataire (PDP / SFTP / mailbox) |
| `ATTENTE_ACK` → `ACQUITTÉ` | Acquittement | Le destinataire a renvoyé un OK |
| `REJETÉ` / `ANNULÉ` | Échec terminal | CDAR 501 irrecevabilité, ou annulation |
| `ERREUR` | Échec interne | Exception non récupérable |

Le champ `status` du document est **le dernier statut atteint**. Le tableau
`events` contient la **séquence complète horodatée** (timeline). Le
pipeline ne s'arrête pas sur les erreurs non bloquantes — c'est le
`CdarProcessor` en aval qui émet une CDV 213 de rejet pour les flux dont
`error_count > 0`.

## Architecture technique

```
crates/pdp-app/src/ui.rs        — handlers HTML (dashboard, liste, détail, downloads)
crates/pdp-app/src/server.rs    — AppState + wiring routes /ui* dans build_api_router
crates/pdp-app/tests/ui_test.rs — tests d'intégration (mock InMemoryTraceBackend)
crates/pdp-trace/src/backend.rs — trait `TraceBackend` (lecture, dyn-compatible)
crates/pdp-trace/src/store.rs   — `TraceStore` (impl ES) + filtre tenant
                                  (seller_siren OR buyer_siren) + count_exchanges
crates/pdp-trace/src/processor.rs — ExchangeSnapshotProcessor (record_exchange seul ;
                                  les événements arrivent via TraceEventSubscriber
                                  qui consomme le bus pdp-events)
```

**Indirection lecture par trait** : l'UI ne dépend pas du `TraceStore`
concret mais d'un `Arc<dyn TraceBackend>`. En production c'est le store
Elasticsearch ; les tests d'intégration utilisent un `InMemoryTraceBackend`
qui ré-implémente la même sémantique de filtrage en Rust pur — zéro
dépendance externe.

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

Toutes les requêtes UI passent par le wildcard `pdp-*` filtré sur
`seller_siren OR buyer_siren = X`. C'est nécessaire pour qu'un tenant voie
ses **factures reçues** (qui sont indexées sous `pdp-{seller_siren}`, pas
sous `pdp-{tenant_siren}`).

| Source | Usage |
|--------|-------|
| `TraceBackend::get_stats_for_siren()` | Dashboard KPIs (total / distribués / erreur / attente) |
| `TraceBackend::list_exchanges()` | Liste paginée avec filtres status / dates / direction / page_size |
| `TraceBackend::count_exchanges()` | Total avec filtres pour `1–50 sur 257 · page 1/6` |
| `TraceBackend::get_exchange()` | Détail facture (ExchangeDocument complet) — wildcard inconditionnel |
| `TraceBackend::get_tenant_name()` | Raison sociale (seller_name si tenant=vendeur, sinon buyer_name) |
| `parse_attachments_from_doc()` (ui.rs) | PJ extraites à la volée — `UblParser` / `CiiParser` / `FacturXParser` selon `source_format`. Pas de stockage des PJ en base. |

## Captures (référence)

```
┌──────────────────────────────────────────────────────────────┐
│ Ferrite — Suivi des factures                                 │
│ Dashboard  Factures  Annuaire                                │
├──────────────────────────────────────────────────────────────┤
│  TechConseil SAS — SIREN 123456789                           │
│                                                              │
│  ┌────────────┐ ┌────────────┐ ┌────────────┐ ┌────────────┐│
│  │ Total flux │ │ Distribués │ │ En attente │ │ En erreur  ││
│  │    257     │ │    251     │ │     0      │ │     6      ││
│  └────────────┘ └────────────┘ └────────────┘ └────────────┘│
│                                                              │
│  Actions                                                     │
│  → Voir toutes les factures                                  │
│  → Voir uniquement les erreurs                               │
└──────────────────────────────────────────────────────────────┘

Liste (extrait) :
┌─────────────────────────────────────────────────────────────────┐
│ [— Toutes directions —] [— Tous —] [Du] [Au] [50/page▾] Filtrer │
├─────────────────────────────────────────────────────────────────┤
│ ↑ INV-001         TechConseil    IndustrieFrance   DISTRIBUÉ    │
│ ↑ FA-2025-PJ-005  TechConseil    IndustrieFrance   📎 4         │
│ ↓ REC-PLO-2026    Plomberie D.   TechConseil       DISTRIBUÉ    │
│ ↓ REC-INCONNU     Inconnu SARL   TechConseil       ERREUR ⚠️    │
├─────────────────────────────────────────────────────────────────┤
│ ← Précédent     1–50 sur 257 · page 1/6        Suivant →        │
└─────────────────────────────────────────────────────────────────┘
```

## Tests

Les handlers UI sont couverts par 22 tests d'intégration dans
[`crates/pdp-app/tests/ui_test.rs`](../crates/pdp-app/tests/ui_test.rs)
qui s'exécutent **sans Elasticsearch** :

```bash
cargo test -p pdp-app --test ui_test
```

Couverture :
- Dashboard avec données + cohérence KPIs ↔ filtres
- Liste : tous, OK, ERREUR, EN_ATTENTE, plage de dates, direction émises/reçues
- Filtre paramètres vides (`?direction=&status=&from=&to=`) → traités comme None
- Pagination : `page_size` 25/50/100/200, valeurs hors plage rejetées,
  total + plage `1–25 sur 30 · page 1/2`, total respecte les filtres
- Dedup par invoice_number par défaut + bypass `?show_duplicates=true`
- Détail : métadonnées, pièces jointes, factures reçues
- Timeline : tronquée à la première erreur
- Statut : `error_count > 0` → badge ERREUR (rouge) même si status brut DISTRIBUÉ
- Routes favicon / logo

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
