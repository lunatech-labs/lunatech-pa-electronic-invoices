# TODO — Ferrite (PDP Facture)

Liste des tâches restantes et améliorations prévues, par ordre de priorité.

**Dernière mise à jour** : 2026-05-02
**Tests** : 1095 tests workspace, 0 échec, 9 ignorés

## Vue d'ensemble — conformité AFNOR

| Spec | Score | Manque principal |
|------|-------|------------------|
| **XP Z12-012** (Formats & Profils) | 97% | Multi-vendeurs §4.5.4, Flux 11 (V1.3 annuaire publiable) |
| **XP Z12-013** (APIs Flow/Directory) | 95% | Plomberie de production seulement |
| **XP Z12-014** (Cas d'usage B2B) | 69% | 13 cas partiels (notes de frais, escomptes TVA, compensations, régimes de marge) |
| **DSE AIFE** (Specifications externes) | 96% | Orchestration cron F13/F14 (briques OK) |

## Restes consolidés (par bloc)

### Annuaire — orchestration manquante (briques livrées)
Voir [§5](#5-annuaire-ppf--copie-locale-flux-1413) :
- Configuration `PpfReturnConsumerConfig.paths` avec chemin SAS retrait F14 dédié
- Cron / déclencheur métier qui appelle le F13 generator (onboarding, changement de PA)
- Application du flux différentiel quotidien

### E-reporting — boucle prod
Voir [§11](#11-e-reporting-flux-10-) :
- CLI `generate102 / generate104` (paiements — source DB métier client requise)
- Cron scheduler pour génération mensuelle automatique
- Envoi SFTP automatique via `PpfSftpProducer` avec `FFE1025A`

### Réception inter-PDP — affinage
Voir [§3](#3-réception-inter-pdp--affinage) :
- Livraison au bon tenant (`{siren}/out/` selon SIREN acheteur)
- Notification acheteur (webhook, email, polling)
- Relais CDV retour acheteur→vendeur (CDV 204, 210)

### Codes IRR pièces jointes
Voir [§4](#4-codes-irr-pièces-jointes-cdv-501) — bloqué par : pas encore de support PJ dans le pipeline.

### Sécurité / multi-tenant
- Voir [§9](#9-autorisation-et-déclaration-des-tenants) — habilitation, mandat signé, F13 onboarding
- Voir [§9ter](#9ter-authentification-sécurité--isolation-tenant) — auth UI/API, RBAC, isolation tenant, audit log
- Voir [§10](#10-rate-limiting-http-) — rate limit par tenant
- Vérification clé serveur SSH en prod (actuellement désactivée en dev)

### Architecture / qualité
- Voir [§8](#8-document-darchitecture-globale) — vision cible, déploiement
- Voir [§12](#12-abstraction-object-store) — S3/MinIO
- Voir [§13](#13-convention-de-nommage-fichiers-cdar-et-factures)
- Renommage **PDP→PA** / **OD→SC** (terminologie V1.2)

### Conformité réglementaire restante (XP Z12-012/014)
- **Multi-vendeurs** §4.5.4 — sub-lines / multi-seller invoices
- **Flux 11** (NOUVEAU V1.3) — annuaire publiable PPF→PA→utilisateurs
- **13 cas d'usage partiels** XP Z12-014 (notes de frais, escomptes TVA, compensations, régimes de marge)

### Gros chantiers produit
- Voir [§3bis](#3bis-interface-web-de-suivi-des-factures) — interface web (4 phases)
- Voir [§14](#14-réécriture-oxalis-access-point-peppol-en-rust) — Peppol AS4 en Rust
- Voir [§15](#15-factur-x-basic-wl--structuré) — Factur-X BASIC WL

---

## Livré entre 2026-04-26 et 2026-05-02

### XP Z12-013 — APIs Flow / Directory (55% → 95%)
- [x] Webhooks AFNOR : 5 endpoints (POST/GET/PATCH/DELETE /v1/webhooks)
- [x] Webhooks : retry exponentiel (3 tentatives, backoff 500ms × 2)
- [x] Webhooks : OAUTH2 client_credentials avec bearer_auth automatique
- [x] Webhooks : trigger `flow.received` et `flow.ack.updated`
- [x] Webhooks : persistance PostgreSQL (table `webhooks` JSONB) + 8 tests testcontainers
- [x] POST /v1/flows : headers `Request-Id`, `Organization-Id`, FullFlowInfo en réponse
- [x] Codes HTTP fins : 408 (TimeoutLayer), 413 (max_flow_size_bytes), 429 (RateLimiter Bearer/IP avec Retry-After), 501 (NotImplemented)
- [x] Directory Service complet : GET /v1/routing-code/siret:{siret}/code:{id}

### DSE AIFE — E-reporting + Annuaire (85% → 96%)
- [x] BR-FR-MAP-23 : `normalize_date_yyyymmdd()` appliqué partout (factures, paiements, périodes)
- [x] CLI : `pdp ereporting generate101 / generate103` avec autodétection UBL/CII/Factur-X
- [x] Helpers `payment_invoice` / `payment_transaction` pour Flux 10.2/10.4
- [x] F13 generator : `generate_f13_xml()` + `build_ligne_for_f13()` (Création/Modification/Suppression)
- [x] F14 auto-import : `AnnuaireImportProcessor` détecte `FFE1435A` et déclenche `ingest_f14`
- [x] CDV F6 annuaire : `AnnuaireStatusCode { Acceptee=400, Rejetee=401 }` + détection dans `DocumentTypeRouter`

### XP Z12-012 — Acteurs CDV (97% conservé)
- [x] CDV 221 ERREUR_ROUTAGE : `RoutingValidationProcessor` détecte les PDP injoignables, `CdarProcessor` génère 221 (ROUTAGE_ERR / CODE_ROUTAGE_ERR)

### Pipeline
- [x] Wiring : RoutingResolver → RoutingValidator → CdarProcessor (pour permettre 221)
- [x] Wiring : `AnnuaireImportProcessor` dans la route `ppf-sftp-return`

### Documentation
- [x] `docs/http-api.md` : référence REST exhaustive avec curl, codes HTTP, diagrammes Mermaid (émission/réception/webhook/429), observabilité (Request-Id, Kibana, Prometheus), multi-tenant, conformité AFNOR (mapping endpoint→spec)
- [x] `docs/openapi.yaml` : OpenAPI 3.1 (16 paths, schémas Webhook/Flow/Error), importable Swagger UI / openapi-generator
- [x] `docs/bruno-collection/` : collection file-based (14 requêtes, 4 dossiers Health/Flow/Webhooks/Directory/Errors)
- [x] `docs/ereporting.md` : référence Flux 10.1-10.4, BR-FR-MAP, exemples CLI + Rust
- [x] `docs/annuaire.md` : sections F13 generator + F14 auto-import + CDV F6 annuaire
- [x] `rapport-conformite-pdp.md` : scores mis à jour (XP Z12-013 95%, DSE AIFE 96%)

## Fait (antérieur)

- [x] Multi-tenancy par SIREN (`TenantRegistry`, `TenantEntry`, auto-config)
- [x] Routes auto-générées par tenant (`{siren}/in → pipeline → {siren}/out`)
- [x] Validation XSD du Flux 1 PPF avant envoi (bloque si invalide)
- [x] Système d'alertes (`AlertErrorHandler`, classification Critical/Warning/Info)
- [x] Rapports d'alerte JSON avec actions recommandées
- [x] Webhook de notification pour alertes critiques
- [x] Documentation Peppol étendue (protocole AS4, WS-Security, PKI, migration Oxalis)
- [x] Documentation annuaire PPF (F14 complet/différentiel, F13 actualisation, copie locale)
- [x] Import F14 streaming via channel mpsc (mémoire bornée ~5 Mo au lieu de ~7 Go)
- [x] Recherche annuaire enrichie (adresses, B2G, codes routage, plateformes)
- [x] Logo Ferrite (SVG responsive + PNG)
- [x] Diagrammes Mermaid dans docs/cdar.md (architecture, pipelines, séquence)

### Séparation PDP émettrice / PDP réceptrice

- [x] `PipelineMode` enum (Emission/Reception) dans la config des routes
- [x] Pipeline émission : validation → Flux 1 PPF (TOUJOURS) → CDAR 200 → routage
- [x] Pipeline réception : validation → PAS de Flux 1 → CDAR 202 "Reçue" → livraison
- [x] `CdarProcessor::emission()` (CDV 200) et `::reception()` (CDV 202)
- [x] Détection intra-PDP dans `RoutingResolverProcessor` (our_matricule)
- [x] Canal mpsc intra-PDP (émission → réception locale sans réseau)
- [x] CLI `--mode emitter|receiver|both` sur `start` et `run`
- [x] Route HTTP inbound corrigée → pipeline réception (plus de Flux 1)
- [x] Route `intra-pdp-reception` via `ChannelConsumer`
- [x] Tests unitaires CDV 202, CliMode, PipelineMode, Destination::IntraPdp

### Conformité AFNOR (Acteurs CDV + Motifs de STATUTS)

- [x] CDV 213 émission : SE + PPF, Issuer=PA-E (conforme onglet Acteurs CDV)
- [x] CDV 213 réception : SE + BY (PAS de PPF — PA-R n'envoie jamais au PPF)
- [x] CDV 501 : Sender=PA-R, Issuer=PA-R, Recipients=PA-E (pas de PPF)
- [x] CDV 202 : SE + BY (pas de PPF, conforme fixture UC1)
- [x] Tableaux complets des 45 codes motifs (Annexe A V1.2) dans docs/cdar.md
- [x] 14 tests `classify_error_reason` (REJ_*, IRR_*, codes métier)
- [x] 23 tests pipeline erreur (XML mal formé, non-XML, PDF, validations BR-FR)

### Service Annuaire + validation G1.63

- [x] `AnnuaireService` (couche service au-dessus de `AnnuaireStore`)
- [x] `AnnuaireValidationProcessor` : BR-FR-10 vendeur + BR-FR-11 acheteur
- [x] `lookup_etablissement_by_siret()` ajouté à `AnnuaireStore`
- [x] Codes erreur : vendeur absent → REJ_COH, acheteur absent → DEST_INC
- [x] `classify_error_reason` : mapping pour les erreurs annuaire
- [x] Intégration dans `main.rs` : `build_annuaire_service` + câblage émission/réception (routes principales, tenants, http-inbound, intra-pdp)

### Relais CDV → PPF (Flux 6)

- [x] `CdvPpfRelayProcessor` : 210 (Refusée) et 212 (Encaissée) → FFE0654A → PPF
- [x] Tous les autres CDV : pas de relais (testés explicitement)
- [x] Non-bloquant si erreur PPF
- [x] Intégré dans `add_common_processors` après `DocumentTypeRouter`
- [x] 10 tests unitaires (relay 210/212, skip pour autres codes, erreur PPF)

### Directory Service AFNOR (XP Z12-013 Annexe B) — endpoints complets

- [x] GET /v1/siren/code-insee:{siren}
- [x] POST /v1/siren/search
- [x] GET /v1/siret/code-insee:{siret}
- [x] POST /v1/siret/search
- [x] GET /v1/routing-code/siret:{siret}/code:{routing-id} (nouveau)
- [x] POST /v1/routing-code/search
- [x] GET /v1/directory-line/code:{addressing-id}
- [x] POST /v1/directory-line/search
- [x] GET /v1/healthcheck
- [x] AnnuaireStore.lookup_code_routage() + 3 tests testcontainers PostgreSQL

### Webhooks AFNOR (XP Z12-013 §5.4)

- [x] Modèles conformes Swagger : `CallbackParameters`, `WebhookMetadata`, etc.
- [x] Store thread-safe in-memory (`WebhookStore`)
- [x] 5 endpoints : POST/GET /v1/webhooks, GET/PATCH/DELETE /v1/webhooks/{uid}
- [x] Validation URL, flowType, flowDirection (In/Out)
- [x] Authentification Bearer (réutilise middleware existant)
- [x] `WebhookDispatcher` : envoie POST avec headers, auth BASIC, signature HMAC-SHA256
- [x] Trigger `flow.received` sur POST /v1/flows accepté (tokio::spawn non-bloquant)
- [x] Filtrage par metadata (flowType + flowDirection + ackStatus)
- [x] 20+ tests unitaires + intégration HTTP (round-trip create/get/list/update/delete)
- [x] Documentation `docs/webhooks.md`

## Haute priorité

### 1. Tests d'intégration `AnnuaireValidationProcessor` ✅

Le processor est wiré dans `main.rs` (helper `build_annuaire_service`, câblé sur
toutes les routes émission/réception, tenants, http-inbound, intra-pdp).

- [x] Tests intégration avec PostgreSQL via testcontainers
      (10 tests, vendeur/acheteur connu/inconnu/inactif, modes Emission/Reception).
- [x] Tests pipeline complet AnnuaireValidation → CdarProcessor (5 tests) :
      vendeur connu → CDV 200, vendeur inconnu → CDV 213/REJ_COH, acheteur
      inconnu → CDV 213/DEST_INC, réception OK → CDV 202, réception vendeur
      inconnu → CDV 213/REJ_COH.

> Ces 15 tests d'intégration tournent à chaque `cargo test` — prérequis :
> Docker ou Podman démarré sur la machine.

### 2. Workflows complets documentés ✅

Voir [docs/workflows.md](workflows.md) — 5 workflows AFNOR XP Z12-014 avec
diagrammes Mermaid (émission classique, rejet à l'émission, réception
classique, intra-PDP, CDV 210/212 avec relais PPF).

### 3. Réception inter-PDP — affinage

L'architecture émission/réception est en place. Reste à affiner :

- [ ] Livraison au bon tenant en réception (`{siren}/out/` selon l'acheteur)
- [ ] Notification de l'acheteur après réception (webhook, email, ou polling)
- [ ] Gestion du CDV retour acheteur→vendeur (CDV 204, 210, etc. à relayer)

### 3bis. Interface web de suivi des factures

Application web permettant aux clients (vendeurs et acheteurs) et à
l'administrateur PDP de suivre les factures émises et reçues, leur cycle
de vie (CDV), et les éventuelles erreurs/rejets.

**Phase 1 — Lecture seule ✅** (voir [docs/ui.md](ui.md))
- [x] Module `ui.rs` : 3 handlers HTML server-rendered (cohérent avec `/annuaire`)
- [x] `GET /ui` — Dashboard avec 4 KPIs (total / distribués / en attente / en erreur)
- [x] `GET /ui/flows` — Liste paginée avec filtres (statut, dates) + 50/page
- [x] `GET /ui/flows/{flowId}` — Détail facture (métadonnées, totaux) + timeline pipeline
- [x] **Pièces jointes** : extraites à la volée du `raw_xml` (UBL/CII) ou
      `raw_pdf_base64` (Factur-X) — pas stockées en base. Tableau
      ID/Fichier/Description/MIME/Taille avec fallback sur `attachment_filenames`
      indexés si `raw_xml` indisponible.
- [x] Style CSS inline cohérent avec `/annuaire`, badges colorés par statut
- [x] Multi-tenant via `?siren=` (sélecteur SIREN si paramètre absent)
- [x] `TraceStore::list_exchanges()` + `get_stats_for_siren()`
- [x] 15 tests (HTTP routes UI + helpers PJ extraction/render/escape)

#### Phase 1.5 — Enhancements identifiés sur l'UI lecture seule

- [ ] **Sémantique des statuts à revoir** — le pipeline expose 14 valeurs
      (`REÇU` → `PARSING` → `PARSÉ` → `VALIDATION` → `VALIDÉ` →
      `TRANSFORMATION` → `TRANSFORMÉ` → `DISTRIBUTION` → `DISTRIBUÉ` →
      `ATTENTE_ACK` → `ACQUITTÉ` + `REJETÉ`/`ANNULÉ`/`ERREUR`). Mais :
      - les statuts intermédiaires (`PARSING`, `VALIDATION`,
        `TRANSFORMATION`, `DISTRIBUTION`) ne sont **jamais persistés** car
        leur durée est trop courte (le processor d'après écrase l'état avec
        son statut terminal) → bruit dans l'enum
      - le pipeline ne **s'arrête pas sur erreur** : un flux avec
        `error_count > 0` peut atteindre `DISTRIBUÉ` puis être rejeté en
        aval par le `CdarProcessor` (CDV 213). L'UI palie en remplaçant le
        badge par "ERREUR" mais le `status` brut reste trompeur.
      - manque les statuts métier de l'AFNOR XP Z12-013 §6.4 (états du
        cycle de vie facture côté acheteur : `Reçue`, `Approuvée`, `Refusée`,
        `Suspendue`, `Litigieuse`, `Comptabilisée`, `Mise en paiement`,
        `Encaissée` — actuellement gérés via les codes CDV 200-212 mais
        pas exposés comme statut de flux dans la trace)
      - **Action** : trier les statuts en deux groupes orthogonaux —
        (a) état du **traitement pipeline** (en cours, terminé, en erreur)
        et (b) état du **cycle de vie métier** (CDV reçue, approuvée,
        refusée, payée…). Modéliser comme deux champs distincts dans
        `ExchangeDocument` plutôt qu'un seul `status` confus.
- [ ] **Visibilité des envois vers le PPF** — les Flux 1 (déclaration TVA)
      et Flux 2/4 (e-reporting transactions / paiements) sont émis par
      `PpfFlux1Processor` / `PpfSftpProducer` mais **n'apparaissent pas
      dans l'UI**. L'utilisateur ne voit pas qu'une facture a été
      doublement traitée (vers le destinataire ET vers le PPF).
      - tracer chaque envoi PPF comme un événement de la timeline
        (`status = ENVOYÉ_PPF`, route_id distinct, message contenant le
        nom du fichier `FFE0654A...` et la date)
      - section dédiée sur la page détail "Reporting PPF" listant les
        envois (date, type FFE, status retour PPF si CDV 213 reçu)
      - filtre / KPI dashboard "Envoyés au PPF" pour vérifier d'un coup
        d'œil la couverture e-reporting du tenant

#### Écrans utilisateur (par tenant) — phases suivantes

- [x] ~~**Liste factures reçues** vs émises~~ — filtre `direction` livré (Phase 1)
- [x] ~~**Téléchargement** : XML facture, PDF Factur-X, PJ~~ — endpoints
      `/ui/flows/{id}/download/{xml|pdf|attachment}` livrés (Phase 1)
- [ ] **Soumission de factures** : upload UBL/CII/Factur-X via formulaire web
      (pour fournisseurs sans intégration API)
- [ ] **Émission de CDV manuels** : pour acheteurs (CDV 204/205/207/210, etc.)
- [ ] **Notifications** : alertes en cas de rejet, refus, ou changement de statut
- [ ] **Recherche full-text** dans `raw_xml` (déjà supporté par `TraceStore::search_xml`)

#### Écrans administrateur PDP

- [ ] **Dashboard global** : volumétrie multi-tenant, performances pipeline,
      taux de rejet par étape (réception, validation, annuaire, distribution)
- [ ] **Tracking flux** : recherche par flowId / trackingId, état dans le pipeline
- [ ] **Logs et erreurs** : par tenant, par flux, par étape
- [ ] **Alertes** : liste des alertes Critical/Warning/Info avec filtre temporel
- [ ] **Annuaire PPF** : interface de recherche déjà existante (`/annuaire`),
      enrichir avec stats et requêtes par SIREN/SIRET

#### Architecture technique

- [ ] Choix du stack frontend (React/Vue/Svelte ou autre)
- [ ] API REST dédiée pour le frontend (ou GraphQL ?) — au-dessus de l'API AFNOR
- [ ] Authentification (Bearer token réutilisé ou OAuth2/OIDC dédié)
- [ ] Multi-tenancy : un utilisateur ne voit que ses propres factures (par SIREN)
- [ ] Endpoints `/v1/admin/*` pour les fonctions PDP-admin (séparés)
- [ ] WebSocket ou Server-Sent Events pour les notifications live ?
- [ ] Servir le frontend statique depuis pdp-app (style `/annuaire`) ou app séparée

#### Sources de données

L'interface s'appuie sur :

- **Elasticsearch** (`pdp-trace`) : événements pipeline, recherche full-text XML
- **PostgreSQL annuaire** : résolution SIREN/SIRET
- **Filesystem tenant** : `{siren}/in/`, `{siren}/out/`, archives
- **In-memory webhooks** (à terme PostgreSQL) : abonnements

Pas besoin d'une nouvelle base : tout existe déjà dans `pdp-trace`.

#### Phases de livraison

- [x] Phase 1 : écrans lecture seule (dashboard + liste + détail) — voir [docs/ui.md](ui.md)
- [ ] Phase 2 : actions (soumission factures, émission CDV manuels)
- [ ] Phase 3 : admin PDP (multi-tenant, alertes, métriques)
- [ ] Phase 4 : notifications live (WebSocket/SSE)

### 4. Codes IRR pièces jointes (CDV 501)

4 codes IRR_* spec sont dans l'enum mais pas implémentés (pas de support PJ) :

- [ ] `IRR_TAILLE_PJ` : contrôle de taille des PJ
- [ ] `IRR_VID_PJ` : PJ non vide
- [ ] `IRR_EXT_DOC` : extension des PJ
- [ ] `IRR_ANTIVIRUS` : contrôle antivirus
- [ ] Mise à jour `map_reception_to_irrecevabilite` avec les nouveaux codes REC-*

### 5. Annuaire PPF — Copie locale (Flux 14/13)

Maintenir une copie locale de l'annuaire PPF pour le routage offline et performant (voir `docs/annuaire.md`).

- [x] Parser XML F14 streaming (crate `pdp-annuaire`, quick-xml, 10 Go en 81s)
- [x] Stockage PostgreSQL (5 tables, batch insert, index routage)
- [x] Résolution de routage locale 4 mailles (suffixe > code routage > SIRET > SIREN)
- [x] CLI import (`pdp annuaire import <fichier>`)
- [x] CLI consultation (`pdp annuaire stats/lookup/route`)
- [x] API Directory Service conforme AFNOR XP Z12-013 Annexe B
- [x] PostgreSQL dans docker-compose, config `database` dans PdpConfig
- [x] Tests unitaires (7) + test intégration fichier réel 10 Go
- [x] AnnuaireService + AnnuaireValidationProcessor (G1.63)
- [x] Auto-import F14 quand reçu sur SAS retrait PPF (`AnnuaireImportProcessor`)
- [x] Émetteur F13 — generator XML (`generate_f13_xml` + `build_ligne_for_f13`)
- [x] Traitement CDV F6 annuaire (statuts 400 Acceptée / 401 Rejetée) dans `DocumentTypeRouter`
- [ ] Configuration SAS retrait F14 dédié dans `PpfReturnConsumerConfig.paths`
- [ ] Cron / déclencheur métier qui appelle le F13 generator (onboarding tenant, changement de PA)
- [ ] Application du flux différentiel quotidien (24h)

### 6. CDV 221 (ERREUR_ROUTAGE) ✅

- [x] `RoutingValidationProcessor` détecte `PDP-{matricule}` non configurée
- [x] `CdarProcessor` génère CDV 221 (au lieu de 213) si une erreur step="routage" est présente
- [x] Codes motifs `ROUTAGE_ERR` et `CODE_ROUTAGE_ERR` (Acteurs CDV V1.2)
- [x] Wiring : RoutingResolver → RoutingValidator → CdarProcessor (ordre inversé)
- [x] 10 tests unitaires (RoutingValidator + CdarProcessor + classify)

### 7. Nettoyage du répertoire specs/

Le répertoire `specs/` contient ~13 MB de duplication et des versions multiples inutilisées.

- [x] Supprimer `specs/xsd/specs-externes-v3.1/` (doublon, -6.8 MB)
- [x] Supprimer `specs/xsd/e-invoicing/` (doublon de cii+ubl, -6.7 MB)
- [x] Supprimer les anciennes versions Schematron/XSLT EN16931 (1.3.14.2, 1.3.15)
- [x] Supprimer les variantes CDAR inutilisées (d22b-uncoupled, d23b, d23b-uncoupled)
- [x] Renommer tous les répertoires avec numéros de version
- [x] Mettre à jour tous les chemins dans le code (xsd.rs, schematron.rs)
- [ ] Vérifier que tous les tests passent après nettoyage

### 8. Document d'architecture globale

Créer un vrai document d'architecture système (pas juste la liste des crates).

- [ ] Nicolas décrit sa vision de l'architecture cible
- [ ] Composants et leur déploiement (mono-binaire vs micro-services)
- [ ] Infrastructure (stockage, messaging, monitoring)
- [ ] Diagrammes de flux de données
- [ ] Séparation des responsabilités

### 8bis. Conformité réglementaire restante (XP Z12-012 / 014)

Items pondérés "haute priorité" pour finir la conformité aux normes AFNOR.

#### Multi-vendeurs (XP Z12-012 §4.5.4)
- [ ] Modèle InvoiceData : support sub-lines / multi-seller
- [ ] Parsers UBL/CII : extraction des plusieurs vendeurs par facture
- [ ] CDV : émission par vendeur ou agrégée selon spec
- [ ] Tests avec fixtures multi-vendeurs

#### Flux 11 — Annuaire publiable (NOUVEAU V1.3)
- [ ] Spec : annuaire publiable PPF → PA → utilisateurs
- [ ] Code interface PPF (à confirmer dans la spec V1.3)
- [ ] Producer/Consumer + parser
- [ ] Tests

#### Cas d'usage partiels XP Z12-014 (13 cas restants)
Voir détail dans `rapport-conformite-pdp.md` §3.2 :
- [ ] Notes de frais (cas 35-37)
- [ ] Escomptes TVA (cas 38)
- [ ] Compensations (cas 39-40)
- [ ] Régimes de marge — calcul bénéfice (cas 33)
- [ ] TVA déjà collectée B2C → B2B (cas 30, lien rétrospectif)
- [ ] Factures mixtes — routage (cas 31)
- [ ] Cas avancés et spécialisés (35-42 partiels)

### 8ter. Renommage terminologie V1.2 (PDP→PA, OD→SC)

XP Z12-012 V1.2 introduit les termes officiels « Plateforme Agréée » (PA-E
émettrice / PA-R réceptrice) et « Service de Conformité » (SC, ex-OD).

- [ ] Audit des occurrences "PDP" et "OD" dans le code et la doc
- [ ] Renommage progressif (compatibilité ascendante via `pub use`)
- [ ] Mise à jour CLI, configs, logs, propriétés Exchange
- [ ] Mise à jour README, docs/*.md, exemples

## Moyenne priorité

### 9. Autorisation et déclaration des tenants

Actuellement les tenants sont auto-configurés (juste un répertoire SIREN suffit). Il faudra vérifier qu'un tenant est autorisé à utiliser la PDP.

- [ ] Accord formel de choix de plateforme (mandat signé)
- [ ] Vérification de l'habilitation avant traitement
- [ ] Enregistrement dans l'annuaire PPF (F13) lors de l'onboarding
- [ ] Workflow de changement de PDP (clôture des lignes de l'ancienne PA)

### 9bis. Sécurité SFTP — vérification clé serveur

- [ ] Activer la vérification de la clé serveur SSH en prod (actuellement
      désactivée en dev pour faciliter les tests)
- [ ] Configuration `known_hosts` par tenant ou globale
- [ ] Documentation procédure de rotation des clés

### 9ter. Authentification, sécurité & isolation tenant

**Phase A livrée** ([crates/pdp-app/src/security.rs](../crates/pdp-app/src/security.rs)) :

- [x] `SecurityContext` (principal + `allowed_sirens` + `Role`) injecté par
      `auth_middleware` dans `Request::extensions`
- [x] Config `tokens: Vec<TokenConfig>` (avec liaison
      `principal`/`allowed_sirens`/`role`) ; `bearer_tokens:` deprecated
      avec migration douce vers `PdpAdmin`
- [x] Mode `dev_open: true` réservé à la démo locale (assume admin sans
      token) ; fail-closed en prod
- [x] Extractor `AuthorizedSiren` qui rejette en 403 tout `?siren=X`
      hors scope du porteur
- [x] Helper `authorize_optional_siren` pour les pages UI qui acceptent
      un siren absent (sélecteur)
- [x] UI `/ui/*` passe par `auth_middleware` (avant : routes publiques
      sans aucun contrôle)
- [x] API : `/v1/stats`, `/v1/flows?status=error`, `/v1/flows/{id}`
      exigent `?siren=...` ET la propagent au store
- [x] `TraceBackend::get_exchange` filtre désormais `seller_siren OR
      buyer_siren = X` (avant : un mauvais siren retournait quand même
      le document — ignoré)
- [x] 11 tests d'isolation tenant sans Elasticsearch
      ([tests/security_test.rs](../crates/pdp-app/tests/security_test.rs))

**Phase B livrée** ([crates/pdp-app/src/session.rs](../crates/pdp-app/src/session.rs)) :

- [x] Headers de sécurité posés sur **toutes** les réponses (CSP, HSTS,
      X-Frame-Options, X-Content-Type-Options, Referrer-Policy)
- [x] Module `session` : cookie HMAC-SHA256 signé, TTL configurable,
      pas de stockage serveur (stateless)
- [x] Config `users:` (email + password + principal + allowed_sirens +
      role) ; `session_secret` et `session_ttl_secs` configurables
- [x] Pages `/login` (GET form, POST handler) et `/logout`
- [x] Cookie `ferrite_session` (HttpOnly, SameSite=Lax, Max-Age)
- [x] `auth_middleware` accepte cookie OU Bearer (priorité cookie pour
      l'UI). UI sans auth → redirect 303 vers `/login?next=...`. API
      sans auth → 401 JSON.
- [x] L'annuaire (`/annuaire`, `/v1/annuaire/search`) reste public — choix
      produit
- [x] 9 tests Phase B (login OK / KO, cookie roundtrip, isolation cross-
      tenant via cookie, logout, headers de sécurité, annuaire public)

**Phase B.5 livrée** :

- [x] **argon2id** pour les passwords stockés. Backward-compat plaintext
      (warn au démarrage). CLI `pdp tools hash-password "..."` pour
      générer le hash, `pdp tools gen-session-secret` pour le secret.
- [x] **Invalidation de session côté serveur** au logout
      (`RevocationList` in-memory bornée 5000 entrées avec purge
      paresseuse des entrées expirées). Test : un cookie rejoué après
      logout retourne 303 → /login.
- [x] **Cookie `Secure`** quand `X-Forwarded-Proto: https` est présent
      (proxy nginx/traefik). Omis en HTTP local pour que le navigateur
      envoie le cookie sur la démo.

**Reste à faire (Phase B.6+)** :

- [ ] **2FA TOTP** optionnel sur les comptes `tenant_admin` / `pdp_*`
      (nécessite stockage du secret partagé par user + UI d'enrôlement)
- [ ] **OIDC / OAuth2** en plus du login local (Keycloak, Auth0,
      FranceConnect+ Pro pour les assujettis)
- [ ] **CSRF token** sur les actions POST (Phase 2 UI : soumission
      factures, émission CDV)
- [ ] **Revocation list persistée** (Postgres) — la liste est aujourd'hui
      en mémoire et perdue au redémarrage

#### Modèle d'autorisation (RBAC)

- [ ] **Rôles** :
  - `tenant_user` — voit/édite uniquement les flux où son SIREN est
    `seller_siren` OR `buyer_siren`. Ne voit **jamais** les flux d'un
    autre tenant.
  - `tenant_admin` — comme `tenant_user` + gestion des utilisateurs et
    webhooks de son tenant.
  - `pdp_operator` — accès lecture multi-tenant (support, debug). Doit
    être journalisé (audit log obligatoire pour toute consultation
    cross-tenant).
  - `pdp_admin` — superuser (configuration globale, rotation clés,
    annuaire).
- [ ] **Liaison utilisateur ↔ SIREN(s)** : un user peut être attaché à 1
      ou N tenants (ex. comptable externe gérant plusieurs clients). La
      session porte la liste des SIREN autorisés ; le sélecteur de
      tenant n'expose que ceux-là.
- [ ] **Permissions spéciales** : un user peut recevoir un accès cross-
      tenant explicite, limité dans le temps, avec consentement du
      tenant cible (mandat signé, cf §9). Tracé dans l'audit log avec
      la raison (incident, audit légal, support).

#### Isolation effective des données

- [ ] **Filtre serveur obligatoire** : les handlers UI doivent rejeter
      tout `?siren=X` qui n'est pas dans la liste des SIREN de la
      session. Aujourd'hui le filtre est purement *côté presentation*
      (le SIREN est lu depuis la query string) — un user peut taper
      n'importe quoi dans l'URL.
- [ ] **Vérification au niveau du store** : `TraceBackend` devrait
      prendre la liste des SIREN autorisés en paramètre (ou via un
      `SecurityContext` injecté) et garantir qu'aucune requête ES ne
      retourne des docs hors scope. Belt-and-braces vs un seul check
      handler.
- [ ] **Cross-tenant queries verrouillées** : `get_stats()` /
      `get_error_flows()` (qui ratissent `pdp-*`) ne doivent être
      accessibles qu'aux rôles `pdp_operator` / `pdp_admin`.
- [ ] **API HTTP** : Bearer token doit aussi porter une identité +
      permissions (actuellement `bearer_tokens: Vec<String>` est une
      simple allowlist sans notion de tenant — un client peut appeler
      `/v1/flows?siren=X` pour n'importe quel X).

#### Audit & journalisation

- [ ] Audit log structuré (JSON) pour chaque accès UI/API : timestamp,
      user, role, tenant cible, action, ressource, IP, status.
- [ ] Rétention conforme aux exigences PPF (5 ans recommandé pour les
      accès aux données fiscales).
- [ ] Endpoint `/v1/audit/search` réservé `pdp_admin` pour investiguer.
- [ ] Alertes sur patterns suspects (énumération de SIREN, accès
      massifs hors heures, échecs d'authentification répétés).

#### Secrets & configuration

- [ ] Plus de bearer tokens en clair dans `config.yaml` — utiliser un
      secret manager (HashiCorp Vault, AWS Secrets Manager, ou même
      `sops`-encrypted YAML).
- [ ] Hash + salt + KDF (argon2) pour les mots de passe locaux.
- [ ] Rotation des clés JWT, des secrets HMAC webhook, des credentials
      SFTP — procédures documentées et testables.

### 10. Rate limiting HTTP ✅

- [x] Limiter le nombre de requêtes par tenant/token (clé Bearer / IP)
- [x] Réponse 429 Too Many Requests avec Retry-After
- [x] Configuration globale via `HttpServerConfig.rate_limit_per_minute`
- [ ] Configuration par tenant (actuellement globale)

### 11. E-reporting (Flux 10) ✅

- [x] Modèle de données pour transactions et paiements (TransactionInvoice, PaymentInvoice, AggregatedTransaction)
- [x] Sérialisation au format XSD PPF V1.0 (10.1, 10.2, 10.3, 10.4)
- [x] Règles BR-FR-MAP (01, 04, 06, 08, 10, 12, 14, 15, 16, 17, 18, 19, 23)
- [x] Tests générateur (22 tests) + 10 tests CLI E2E
- [x] Source automatique des factures depuis Elasticsearch `pdp-{siren}` (`TraceStore::get_invoices_by_period`, re-parsing UBL/CII/Factur-X)
- [ ] CLI 10.2/10.4 (paiements — nécessite source de paiements depuis DB métier)
- [ ] Cron / scheduler pour génération mensuelle automatique
- [ ] Envoi SFTP via `PpfSftpProducer` avec `FFE1025A`

### 12. Abstraction object store

SFTP comme couche mince vers un object store (S3/MinIO).

- [ ] Interface `ObjectStore` (put, get, list, delete)
- [ ] Implémentation filesystem (actuelle)
- [ ] Implémentation S3/MinIO
- [ ] Le protocole SFTP sauvegarde dans l'object store au lieu du filesystem
- [ ] Les répertoires tenant `{siren}/in/` et `{siren}/out/` deviennent des préfixes S3

### 13. Convention de nommage fichiers CDAR et factures

Revoir et formaliser la convention de nommage pour les fichiers CDAR et les factures (identifiants de documents, noms de fichiers retour, nommage SFTP). À discuter avec Nicolas.

- [ ] Définir la convention pour les noms de fichiers CDAR retournés (`CDV_{id}.xml`)
- [ ] Définir la convention pour les noms de fichiers factures (entrée/sortie)
- [ ] Aligner le `document_id` (MDT-4) et le `document_name` (MDT-5) avec les specs AFNOR
- [ ] Documenter les conventions dans `docs/cdar.md`

## Basse priorité

### 14. Réécriture Oxalis (Access Point Peppol en Rust)

Remplacer le gateway Java Oxalis par une implémentation Rust intégrée (voir `docs/peppol.md`).

- [ ] Implémentation AS4 (SOAP 1.2, ebMS 3.0, MIME multipart)
- [ ] WS-Security (XML-DSIG RSA-SHA256, BinarySecurityToken)
- [ ] PKI Peppol (validation chaîne de certificats, CRL)
- [ ] Enregistrement SMP (publication des capacités de réception)
- [ ] Receipts et signaux d'erreur AS4
- [ ] Retry avec backoff exponentiel
- [ ] Déduplication des messages (MessageId, 7 jours)
- [ ] Migration progressive (shadow → canary → principal → décommissionnement Oxalis)
- [ ] Tests d'interopérabilité avec Oxalis et phase4

### 15. Factur-X BASIC WL → structuré

- [ ] Génération de lignes à partir de la ventilation TVA (toléré jusqu'au 01/09/2027)
- [ ] Marquage du document comme converti

### 16. Interface d'administration (succinct — voir aussi §3bis)

Voir section "3bis. Interface web de suivi des factures" (haute priorité)
pour les écrans détaillés. Cette section couvre uniquement les besoins
admin/exploitation au-delà du suivi facture.

- [ ] Gestion des tenants (ajout, suppression, configuration)
- [ ] Consultation des logs système (par tenant)
- [ ] Suivi des alertes critiques
- [ ] Métriques Prometheus visualisées (Grafana ou intégré)
- [ ] Configuration runtime (sans redémarrage de la PDP)
