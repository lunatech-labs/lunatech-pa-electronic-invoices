# TODO — Ferrite (PDP Facture)

Liste des tâches restantes et améliorations prévues, par ordre de priorité.

**Dernière mise à jour** : 2026-04-26
**Tests** : 170+ tests pdp-cdar, 0 échec sur le workspace

---

## Fait

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

### 2. Workflows complets documentés

Nicolas doit décrire les workflows précis (cas d'usage AFNOR XP Z12-014) :

- [ ] Décrire le workflow émission classique (UC1)
- [ ] Décrire le workflow émission avec rejet à la validation
- [ ] Décrire le workflow réception classique
- [ ] Décrire le workflow intra-PDP
- [ ] Décrire le workflow CDV 210/212 avec relais PPF
- [ ] Documenter dans `docs/workflows.md` (nouveau fichier)
- [ ] Diagrammes de séquence Mermaid pour chaque cas

### 3. Réception inter-PDP — affinage

L'architecture émission/réception est en place. Reste à affiner :

- [ ] Livraison au bon tenant en réception (`{siren}/out/` selon l'acheteur)
- [ ] Notification de l'acheteur après réception (webhook, email, ou polling)
- [ ] Gestion du CDV retour acheteur→vendeur (CDV 204, 210, etc. à relayer)

### 3bis. Interface web de suivi des factures

Application web permettant aux clients (vendeurs et acheteurs) et à
l'administrateur PDP de suivre les factures émises et reçues, leur cycle
de vie (CDV), et les éventuelles erreurs/rejets.

#### Écrans utilisateur (par tenant)

- [ ] **Dashboard** : KPIs (total factures émises/reçues, en attente, en erreur,
      en litige, encaissées) sur la période sélectionnée
- [ ] **Liste factures émises** : filtres (date, statut CDV, acheteur, montant),
      tri, pagination, recherche full-text
- [ ] **Liste factures reçues** : idem côté acheteur
- [ ] **Détail facture** : metadata (BT-1, BT-2, montants, parties), historique
      CDV (200 → 202 → 204 → 205/210 → 212), pièces jointes, lien vers le XML
      brut et le PDF readable
- [ ] **Timeline CDV** : visualisation chronologique des statuts (timeline UI)
- [ ] **Téléchargement** : XML facture, PDF Factur-X, CDV individuels
- [ ] **Soumission de factures** : upload UBL/CII/Factur-X via formulaire web
      (pour fournisseurs sans intégration API)
- [ ] **Émission de CDV manuels** : pour acheteurs (CDV 204/205/207/210, etc.)
- [ ] **Notifications** : alertes en cas de rejet, refus, ou changement de statut

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

- [ ] Phase 1 : écrans lecture seule (dashboard + liste + détail)
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
- [ ] Consumer SFTP F14 (récupération automatique tar.gz depuis le PPF)
- [ ] Application du flux différentiel quotidien (24h)
- [ ] Émetteur F13 (actualisation des lignes d'annuaire de nos clients)
- [ ] Traitement CDV F6 annuaire (statuts 400 Acceptée / 401 Rejetée)

### 6. CDV 221 (ERREUR_ROUTAGE)

Quand le routage de la facture vers la PDP destinataire échoue (matricule inconnu,
PDP injoignable…), il faut émettre un CDV 221.

- [ ] Détection erreur routage dans `DynamicRoutingProducer`
- [ ] Génération CDV 221 par PA-R (Sender=PA-R, Issuer=PA-R, Recipients=PA-E)
- [ ] Code motif `ROUTAGE_ERR` dans le CDV
- [ ] Tests unitaires

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

## Moyenne priorité

### 9. Autorisation et déclaration des tenants

Actuellement les tenants sont auto-configurés (juste un répertoire SIREN suffit). Il faudra vérifier qu'un tenant est autorisé à utiliser la PDP.

- [ ] Accord formel de choix de plateforme (mandat signé)
- [ ] Vérification de l'habilitation avant traitement
- [ ] Enregistrement dans l'annuaire PPF (F13) lors de l'onboarding
- [ ] Workflow de changement de PDP (clôture des lignes de l'ancienne PA)

### 10. Rate limiting HTTP

- [ ] Limiter le nombre de requêtes par tenant/token
- [ ] Réponse 429 Too Many Requests avec Retry-After
- [ ] Configuration par tenant ou globale

### 11. E-reporting (Flux 10)

- [ ] Modèle de données pour transactions et paiements
- [ ] Sérialisation au format spécifique PPF
- [ ] Règles BR-FR-MAP-23 (conversion dates UBL → CII)
- [ ] Tests avec exemples officiels

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
