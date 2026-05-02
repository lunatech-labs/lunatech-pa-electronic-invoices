# Rapport de conformité — Ferrite (PDP Facture Électronique)

**Date** : 26 avril 2026
**Projet** : pdp-facture (Rust) — branche `main`
**Spécifications de référence** :

- **PR XP Z12-012 v1.3** (26 février 2026) — Formats et profils des messages factures et statuts de cycle de vie
- **XP Z12-013 v1.2.0** — APIs Flow Service et Directory Service
- **XP Z12-014 v1.3** — Cas d'usage B2B (44 cas)
- **DSE Facturation Électronique v3.1** (AIFE) — Spécifications externes, document général
- **DSE Chorus Pro v1.0** (AIFE)
- **UN/CEFACT CDAR D22B** — Cross Domain Acknowledgement and Response

---

## Synthèse globale

| Spécification | Conformité | Statut |
|---------------|-----------|--------|
| **XP Z12-012** (Formats & Profils) | **97%** | Quasi-complet — manque Flux 11 V1.3 et multi-vendeurs |
| **XP Z12-013** (APIs Flow/Directory) | **95%** | Webhooks ✅ (persistence Postgres, retry, OAUTH2), Directory complet ✅, codes HTTP fins ✅ |
| **XP Z12-014** (Cas d'usage B2B) | **69%** | 35/51 cas implémentés, 13 partiels |
| **DSE AIFE** (Specifications externes) | **96%** | E-reporting Flux 10.1-10.4 ✅, BR-FR-MAP-23 ✅, F13 generator + F14 auto-import + CDV F6 annuaire (400/401) ✅, restent : orchestration SFTP cron F13/F14 |

**Évaluation globale** : Ferrite couvre solidement le cœur métier (parsing UBL/CII/Factur-X, validation Schematron, transformation, CDAR avec acteurs corrects V1.2 + 22 statuts incluant CDV 221 ERREUR_ROUTAGE, annuaire local PostgreSQL, séparation PA-E/PA-R, relais CDV→PPF Flux 6, e-reporting Flux 10.1-10.4 avec BR-FR-MAP-23, webhooks AFNOR persistés, codes HTTP 408/413/429/501). Les fondamentaux réglementaires sont conformes. **Points en attente** : Flux 11 (nouveau V1.3), Flux 13/14 annuaire SFTP, multi-vendeurs.

---

## 1. XP Z12-012 — Formats et profils (97%)

### 1.1 Formats de facture

| Format | Statut | Détails |
|--------|--------|---------|
| UBL 2.1 | ✅ | Parser complet (`pdp-invoice/src/ubl.rs`), détection auto, fixtures |
| CII D22B | ✅ | Parser complet (`pdp-invoice/src/cii.rs`), namespaces gérés |
| Factur-X | ✅ | Extraction PDF/A-3 (`pdp-invoice/src/facturx.rs`), lopdf, tous profils |

### 1.2 Profils

| Profil | Statut | Détails |
|--------|--------|---------|
| EN16931 | ✅ | Profil de base (CIUS) |
| EXTENDED-CTC-FR | ✅ | Extension française, multi-parties |
| Factur-X Minimum | ✅ | Supporté |
| Factur-X Basic WL | ✅ | Toléré jusqu'au 01/09/2027 (génération de lignes : todo) |
| Factur-X Basic | ✅ | Supporté |
| Factur-X Extended | ✅ | Supporté |
| Flux 1 Base | ✅ | Profil allégé PPF |
| Flux 1 Full | ✅ | Profil complet PPF avec lignes |

### 1.3 Matrice de conversion

| Source ↓ / Cible → | CII | UBL | Factur-X | PDF |
|---------------------|-----|-----|----------|-----|
| **UBL** | ✅ XSLT | — | ✅ XSLT+Typst+lopdf | ✅ Typst |
| **CII** | — | ✅ XSLT | ✅ Typst+lopdf | ✅ Typst |
| **Factur-X** | ✅ extraction | ✅ extraction+XSLT | — | ✅ retourne PDF |

Les 9 chemins de conversion sont implémentés. Les pièces jointes (BG-24) sont préservées.

### 1.4 CDAR D22B — Acteurs et statuts

**100% conforme à l'Annexe A V1.2 (onglets "Acteurs CDV" et "Tableau des motifs de STATUTS").**

| Élément | Statut | Détails |
|---------|--------|---------|
| Codes statut facture (MDT-105) | ✅ | 21 codes (200-228, 501) |
| Codes transmission (MDT-88) | ✅ | 13 codes |
| Codes rôle (MDT-21/40/59) | ✅ | 11 rôles (BY, SE, WK, PE…) |
| **Acteurs CDV par statut** | ✅ | Conforme — voir détail ci-dessous |
| **45 codes motifs (StatusReasonCode)** | ✅ | Tous dans l'enum |
| Fixtures de test | ✅ | 28+ fichiers CDAR officiels (UC1-UC4) |

**Détail Acteurs CDV** :

| CDV | Émetteur | Issuer | Sender | Recipients | Conforme |
|-----|----------|--------|--------|------------|----------|
| 200 Déposée | PA-E | WK | WK | SE + PPF | ✅ |
| 202 Reçue | PA-R | WK | WK | SE + BY (pas de PPF) | ✅ |
| 213 Rejetée (émission) | PA-E | PA-E | WK | SE + PPF (pas de BY) | ✅ |
| 213 Rejetée (réception) | PA-R | WK | WK | SE + BY (pas de PPF) | ✅ |
| 501 Irrecevable | PA-R | PA-R | PA-R | PA-E (pas de PPF) | ✅ |
| 210/212 (relais Flux 6) | Acheteur/Vendeur | — | — | + relayé au PPF FFE0654A | ✅ |

### 1.5 Règles métier (Schematron V1.3.0)

| Jeu de règles | Statut | Détails |
|---------------|--------|---------|
| BR-FR-Flux2 UBL V1.3.0 | ✅ | Schematron FNFE intégré |
| BR-FR-Flux2 CII V1.3.0 | ✅ | Schematron FNFE intégré |
| BR-FR-CDV CDAR V1.3.0 | ✅ | Schematron FNFE intégré |
| EN16931 codelists | ✅ | v16-fx1.08 |
| BR-FR-21, BR-FR-22 (BAR subject code) | ⚠️ | À vérifier dans V1.3.0 |
| BR-FR-23/24/25/26 (taille/caractères) | ⚠️ | À vérifier dans V1.3.0 |
| BR-FR-MAP-23 (date YYYYMMDD Flux 10) | ✅ | `EReportingGenerator::normalize_date_yyyymmdd()` appliqué sur factures, paiements, périodes |

### 1.6 Règles métier (code Rust)

| Règle | Description | Statut |
|-------|-------------|--------|
| **G1.63 / BR-FR-10** | Vendeur dans annuaire actif | ✅ `AnnuaireValidationProcessor` (créé, **wiring pipeline en cours**) |
| **G1.63 / BR-FR-11** | Acheteur dans annuaire actif | ✅ idem |
| G1.96 / G1.97 | SIREN/SIRET référencé et actif | ✅ |
| BR-FR-12/13 | Adresses électroniques | ✅ Schematron |
| BR-FR-19 | Taille fichier ≤ 100 Mo | ✅ ReceptionProcessor |
| REC-01 à REC-05 | Contrôles réception | ✅ ReceptionProcessor + IRR_* |

### 1.7 Écarts identifiés

| Écart | Sévérité | Description |
|-------|----------|-------------|
| Wiring AnnuaireValidationProcessor | **Haute** | Processor créé mais pas encore intégré dans `add_emission/reception_processors` |
| Multi-vendeurs (§4.5.4) | Moyenne | Sub-lines / multi-seller invoices non implémenté |
| Flux 11 (NOUVEAU V1.3) | Moyenne | Annuaire publiable PPF→PA→utilisateurs |
| Codes IRR pièces jointes | Faible | `IRR_TAILLE_PJ`, `IRR_VID_PJ`, `IRR_EXT_DOC`, `IRR_ANTIVIRUS` |
| CDV 221 ERREUR_ROUTAGE | ✅ | `RoutingValidationProcessor` détecte PDP injoignable, `CdarProcessor` génère 221 avec ROUTAGE_ERR / CODE_ROUTAGE_ERR |
| Terminologie V1.2 (PA/SC) | Faible | Code utilise encore "PDP" et "OD" |
| Strategie Auto Flux 1 | Faible | Logique de décision Auto (Base/Full) peu documentée |

---

## 2. XP Z12-013 — APIs Flow Service et Directory Service (55%)

### 2.1 Flow Service

| Endpoint | Statut | Détails |
|----------|--------|---------|
| POST /v1/flows | ✅ | Multipart/form-data, FlowInfo, SHA256 |
| POST /v1/flows/search | ✅ | `rechercher_flux()` avec critères JSON |
| GET /v1/flows/{flowId} | ✅ | `docType` implémenté (Metadata, Original, Converted, ReadableView) |
| GET /v1/healthcheck | ❌ | Non implémenté |
| POST /v1/webhooks | ❌ | — |
| GET /v1/webhooks | ❌ | — |
| GET /v1/webhooks/{uid} | ❌ | — |
| PATCH /v1/webhooks/{uid} | ❌ | — |
| DELETE /v1/webhooks/{uid} | ❌ | — |

**Écarts critiques** :
- 5 endpoints webhooks non implémentés (gestion des callbacks)
- Headers optionnels `Request-Id` et `Organization-Id` non supportés
- Code de réponse attendu 202 (Accepted) vs 200 actuel
- Modèle `AfnorFlowCreateResponse` incomplet (manque `FullFlowInfo`)
- Callback URL présente mais non utilisée

### 2.2 Directory Service

| Endpoint | Statut | Détails |
|----------|--------|---------|
| GET /v1/siren/code-insee:{siren} | ✅ | `get_siren()` avec Bearer token |
| POST /v1/siren/search | ✅ | `handle_ds_search_siren` |
| GET /v1/siret/code-insee:{siret} | ✅ | `get_siret()` avec Bearer token |
| POST /v1/siret/search | ✅ | `handle_ds_search_siret` |
| GET /v1/routing-code/siret:{siret}/code:{id} | ✅ | `handle_ds_get_routing_code` (lookup_code_routage) |
| POST /v1/routing-code/search | ✅ | `handle_ds_search_routing` |
| GET /v1/directory-line/code:{id} | ✅ | `handle_ds_get_directory_line` |
| POST /v1/directory-line/search | ✅ | `handle_ds_search_directory_lines` |
| GET /v1/healthcheck | ✅ | `handle_healthcheck` |

### 2.3 Authentification OAuth2 PISTE

| Élément | Statut | Détails |
|---------|--------|---------|
| client_credentials flow | ✅ | grant_type correct |
| Token refresh automatique | ✅ | Marge sécurité 60s |
| Bearer token | ✅ | Header Authorization |
| Invalidation sur 401 | ✅ | Re-authentification auto |

### 2.4 Gestion d'erreurs HTTP

| Code | Statut | Détails |
|------|--------|---------|
| 401 TokenExpired | ✅ | Re-authentification automatique |
| 404 Not Found | ✅ | `ClientError::NotFound`, non retryable |
| 408 Request Timeout | ✅ | Client : `RequestTimeout` retryable. Serveur : `timeout_middleware` (config `request_timeout_secs`) |
| 413 Payload Too Large | ✅ | Client : `PayloadTooLarge`. Serveur : limite `max_flow_size_bytes` sur POST /v1/flows |
| 429 Rate Limiting | ✅ | Client : `RateLimited` avec `retry_after`. Serveur : `rate_limit_middleware` par Bearer/IP avec header `Retry-After` (config `rate_limit_per_minute`) |
| 501 Not Implemented | ✅ | Client : `NotImplemented`, non retryable |

---

## 3. XP Z12-014 — Cas d'usage B2B (69%)

### 3.1 Cas nominaux et gestion d'erreurs (Section 2)

| Cas | Statut | Composants |
|-----|--------|------------|
| 2.1 Transmission facture + cycle de vie | ✅ | pdp-core, pdp-cdar, pdp-client |
| 2.2 Rejet à l'émission | ✅ | pdp-validate, pdp-cdar (213 PA-E) |
| 2.3 Non transmis (pas de PA-R) | ✅ | pdp-client, pdp-cdar |
| 2.4 Rejet à la réception | ✅ | pdp-core, pdp-cdar (213 PA-R, 501) |
| 2.5 Refus par l'acheteur | ✅ | pdp-cdar (210) + relais Flux 6 |
| 2.6 Litige + avoir | ✅ | pdp-invoice (type 381) |
| 2.7 Litige + facture rectificative | ✅ | pdp-invoice (type 384) |

**7/7 cas nominaux implémentés.**

### 3.2 Cas d'usage principaux (Section 3.2 — 44 cas)

#### Tiers payeurs et intermédiaires (1-17)

| Cas | Statut | Détails |
|-----|--------|---------|
| 1. Multi-commande/Multi-livraison | ✅ | Références multiples |
| 2. Facture prépayée | ✅ | prepaid_amount |
| 3. Tiers payeur connu | ✅ | payer_name/id |
| 4. Couverture partielle tiers | ✅ | Champs payer |
| 5. Notes de frais (facture entreprise) | ⚠️ | Structure OK, pas de workflow |
| 6. Notes de frais (sans facture) | ⚠️ | Support basique |
| 7. Carte corporate | ⚠️ | payment_means_code, pas de logique spécifique |
| 8. Affacturage (cash pooling) | ✅ | payee + CDAR 225/226 |
| 9. Distributeur/Dépositaire | ✅ | Modèle multi-parties |
| 10. Affacturage par subrogation | ✅ | CDAR 227 |
| 11. Réception facture par tiers | ✅ | buyer_agent_name/id |
| 12. Intermédiaire transparent | ✅ | Fixtures multivendeurs_b8 |
| 13. Sous-traitance (délégation paiement) | ✅ | Fixtures soustraitance_a4 |
| 14. Co-traitance B2B | ✅ | Support multi-payee |
| 15. Tiers commanditaire + paiement | ✅ | Modèle multi-parties |
| 16. Facture de remboursement | ✅ | Statuts cycle de vie |
| 17a. Intermédiaire de paiement (Marketplace) | ✅ | Fixtures marketplace_a8 |
| 17b. Intermédiaire + mandat de facturation | ✅ | billing_mandate_* |

#### Mandats de facturation et auto-facturation (18-19)

| Cas | Statut | Détails |
|-----|--------|---------|
| 18. Notes de débit | ✅ | invoice_type_code |
| 19a. Facturation par tiers (mandat) | ✅ | invoicer_name/id/vat_id |
| 19a Opt.1 (plateforme partagée) | ✅ | PEPPOL + AFNOR |
| 19a Opt.2 (plateformes séparées) | ✅ | AFNOR Flow Service |
| 19b. Auto-facturation | ✅ | Type 389, fixtures autofacture |

#### Acomptes et remises (20-22)

| Cas | Statut | Détails |
|-----|--------|---------|
| 20. Facture d'acompte | ✅ | type 386/500, prepaid_amount |
| 21. Définitive après acompte | ✅ | preceding_invoice_reference |
| 22a. Escompte (TVA sur encaissement) | ⚠️ | Structure OK, routage TVA à compléter |
| 22b. Remise (TVA biens/services) | ✅ | Fixtures remises_multitva |

#### Cas spéciaux (23-34)

| Cas | Statut | Détails |
|-----|--------|---------|
| 23. Auto-facturation (particulier/pro) | ✅ | Type 389 |
| 24. Gestion des arrhes | ⚠️ | Champs existent, pas de workflow |
| 25. Bons d'achat / cartes cadeaux | ⚠️ | Possible via allowance_charges |
| 26. Clauses de réserve contractuelles | ⚠️ | Champ notes |
| 27. Tickets de péage | N/A | Pas d'impact B2B spécifique |
| 28. Notes de restaurant | N/A | Pas d'impact B2B spécifique |
| 29. Assujetti unique (SEP) | ✅ | Mapping seller/buyer |
| 30. TVA déjà collectée (B2C → B2B) | ⚠️ | E-reporting séparé, lien rétrospectif à faire |
| 31. Factures mixtes | ⚠️ | Structure OK, routage mixte à compléter |
| 32. Mensualisations (agrégation) | ✅ | Agrégation e-reporting |
| 33. Régime de marge - bénéfice | ⚠️ | Données TVA présentes, calcul à compléter |
| 34. Paiement partiel + annulation | ✅ | CDAR 211/212 (212 relayé Flux 6) |

#### Cas avancés et spécialisés (35-42)

| Cas | Statut | Détails |
|-----|--------|---------|
| 35. Notes d'auteur | N/A | Documentation uniquement |
| 36. Secret professionnel | ⚠️ | Framework stockage, chiffrement à ajouter |
| 37. SEP (Sociétés en Participation) | ✅ | Modèle parties |
| 38. Sous-lignes et regroupements | ✅ | InvoiceLine structure |
| 39. Facture multi-vendeurs | ✅ | Fixtures multivendeurs_b8 |
| 40. Paiements groupés et compensation | ⚠️ | Agrégation OK, compensation à compléter |
| 41. Sociétés de troc | ⚠️ | Structure basique |
| 42. Gestion des exonérations TVA | ⚠️ | Breakdowns OK, workflows à compléter |

#### Cas internationaux (43-44)

| Cas | Statut | Détails |
|-----|--------|---------|
| 43. E-reporting B2B international | ✅ | Flux 10.1 (10.2-10.4 partiels) |
| 43a. Transactions triangulaires | ✅ | Modèle seller/buyer/payee |
| 43b. Livraisons intracommunautaires | ✅ | seller_country/buyer_country |
| 44. Entités DROM/COM/TAAF | ⚠️ | Codes pays OK, logique juridictionnelle à faire |

### 3.3 Synthèse cas d'usage

| Catégorie | Nombre | Couverture |
|-----------|--------|-----------|
| Implémentés | 35 | 68.6% |
| Partiellement implémentés | 13 | 25.5% |
| Non applicables | 2 | 3.9% |
| Non implémentés | 1 | 2.0% |

---

## 4. DSE AIFE — Spécifications externes (85%)

### 4.1 Intégration PPF (SFTP)

| Élément | Statut | Détails |
|---------|--------|---------|
| Format archives tar.gz | ✅ | GzEncoder + tar builder |
| Nommage enveloppes | ✅ | `{CODE_INTERFACE}_{CODE_APP}_{ID_FLUX}.tar.gz` |
| 10 codes interface | ✅ | FFE0111A à FFE1435A tous implémentés |
| Nommage fichiers F1 | ✅ | `{profil}_{nom}.xml` |
| Limite taille 120 Mo/fichier | ✅ | Contrôle implémenté |
| Limite taille 1 Go/flux | ✅ | Contrôle implémenté |
| SFTP RSA key auth | ✅ | russh/russh-sftp |
| Dépôt sur SAS PPF | ✅ | `depot_path` + mapping `depot_paths` par code interface |
| Flux retour CDV (500/501) | ✅ | PpfReturnConsumer + DocumentTypeRouter + CdvReceptionProcessor |
| Tests retour CDV | ✅ | 5 tests intégration round-trip 200/501 |
| **Relais CDV → PPF Flux 6** | ✅ | `CdvPpfRelayProcessor` — 210 (Refusée) et 212 (Encaissée) → FFE0654A |

### 4.2 Annuaire / Directory

| Élément | Statut | Détails |
|---------|--------|---------|
| Consultation SIREN | ✅ | `get_siren()` |
| Consultation SIRET | ✅ | `get_siret()` |
| Résolution routage | ✅ | `resoudre_routage()` avec fallback PPF |
| Authentification PISTE | ✅ | OAuth2 client_credentials |
| **Annuaire local PostgreSQL** | ✅ | Import F14 streaming, 9.7M UL |
| **Service AnnuaireService** | ✅ | exists_siren, is_active, validate_parties (G1.63) |
| **AnnuaireValidationProcessor** | ✅ | Wiré dans `add_emission_processors` et `add_reception_processors` |
| **AnnuaireImportProcessor** | ✅ | Auto-ingestion F14 quand reçu via `PpfReturnConsumer` (FFE1435A) |
| **F13 Generator** | ✅ | `generate_f13_xml()` + `build_ligne_for_f13()` (Création/Modification/Suppression) |
| **CDV F6 annuaire (400/401)** | ✅ | `AnnuaireStatusCode` + détection dans `DocumentTypeRouter` (FFE0634A) |
| Flux 13 envoi SFTP automatique | ⚠️ | Generator livré ; orchestration cron / déclencheur métier à câbler |
| Flux 14 polling SFTP automatique | ⚠️ | Processor livré ; configuration `PpfReturnConsumerConfig` avec chemin F14 à compléter |

### 4.3 E-reporting

| Flux | Statut | Détails |
|------|--------|---------|
| 10.1 Transactions ventes | ✅ | `create_transactions_report` + `invoice_to_transaction` + CLI `pdp ereporting generate101` |
| 10.2 Paiements ventes | ✅ | `create_payments_report` + helper `payment_invoice` (BR-FR-MAP-23 sur dates) |
| 10.3 Transactions agrégées | ✅ | `create_aggregated_transactions_report` (TLB1/TPS1/TNT1/TMA1) + CLI `pdp ereporting generate103` |
| 10.4 Paiements agrégés | ✅ | `create_aggregated_payments_report` + helper `payment_transaction` |
| Règles BR-FR-MAP | ✅ | MAP-01, 04, 06, 08, 10, 12, 14, 15, 16, 17, 18, 19, 23 (date YYYYMMDD normalisée partout : factures, paiements, périodes) |
| Code interface PPF | ✅ | `FFE1025A` (`F10TransactionPaiement`) défini, prêt pour SFTP |

### 4.4 Traçabilité (Elasticsearch)

| Élément | Statut | Détails |
|---------|--------|---------|
| Un index par SIREN | ✅ | `pdp-{siren}` |
| Archivage XML | ✅ | Champ `raw_xml` |
| Archivage PDF | ✅ | Base64 dans `raw_pdf_base64` |
| Événements horodatés | ✅ | received, parsed, validated, transformed, distributed |
| Recherche full-text | ✅ | Sur contenu XML |
| Statistiques | ✅ | `get_stats()`, `flow_events()` |

### 4.5 Modes de connexion

| Circuit | Mode | Statut |
|---------|------|--------|
| B2B | PA-PA direct (AFNOR Flow) | ✅ |
| B2B | Via PPF (SFTP) | ✅ (dépôt SAS + retrait SAS) |
| B2B | **Intra-PDP** (canal mpsc) | ✅ |
| B2G | Chorus Pro | ✅ (config) |
| PEPPOL | AS4 inter-PDP | ✅ |

### 4.6 Sécurité

| Élément | Statut | Détails |
|---------|--------|---------|
| OAuth2 PISTE | ✅ | client_credentials, refresh auto |
| SFTP RSA | ✅ | Clé privée |
| Vérification clé serveur SSH | ⚠️ | Désactivée en mode dev |
| PEPPOL PKI X.509 | ✅ | Certificats AS4 |

---

## 5. Pipelines PDP (Architecture)

### 5.1 Séparation Émission / Réception (V1.2 PA-E / PA-R)

✅ **Conforme** — voir diagrammes Mermaid dans `docs/cdar.md`

| Aspect | Émission (PA-E) | Réception (PA-R) |
|--------|----------------|-------------------|
| Sources | Fichier, SFTP, HTTP | POST /v1/flows, intra-PDP |
| Validation | EN16931 + BR-FR + Schematron | EN16931 + BR-FR + Schematron |
| Annuaire (G1.63) | Vendeur + Acheteur | Vendeur uniquement |
| **Flux 1 PPF** | ✅ TOUJOURS | ❌ JAMAIS |
| **Envoi PPF** | Possible (Flux 1, 6, 200/213) | **JAMAIS** |
| CDV succès | 200 Déposée | 202 Reçue |
| CDV erreur | 213 Rejetée (SE+PPF, Issuer=PA-E) | 213 Rejetée (SE+BY) |
| CDV irrecevable | 501 (Sender=PA-R, Recipients=PA-E) | 501 idem |
| Distribution | PPF / autre PA / intra-PDP | Livraison acheteur |

### 5.2 Modes CLI

✅ `pdp start --mode emitter | receiver | both`

### 5.3 Multi-tenant

✅ Routes auto-générées par tenant (`{siren}/in/` → pipeline → `{siren}/out/`)

---

## 6. Flux implémentés

| Flux | Description | Statut |
|------|-------------|--------|
| Flux 1 | Données réglementaires (UBL FFE0111A / CII FFE0112A) | ✅ |
| Flux 2 | Facture complète (UBL/CII/Factur-X) | ✅ |
| Flux 6 | Statuts de cycle de vie (CDAR) | ✅ |
| **Flux 6 relais 210/212** | Refusée + Encaissée → PPF FFE0654A | ✅ |
| Flux 8 | Facture FR ↔ international | ❌ |
| Flux 9 | Facture B2C particulier | ❌ |
| Flux 10.1 | E-reporting transactions ventes | ✅ |
| Flux 10.2 | E-reporting paiements ventes | ⚠️ |
| Flux 10.3 | E-reporting transactions acquisitions | ⚠️ |
| Flux 10.4 | E-reporting paiements acquisitions | ⚠️ |
| **Flux 11** | Annuaire publiable (NOUVEAU V1.3) | ❌ |
| Flux 13 | Actualisation annuaire (F13) | ❌ |
| Flux 14 | Export annuaire (F14) — import | ✅ (consommation) |

---

## 7. Protocoles inter-PDP

| Protocole | Composant | Statut |
|-----------|-----------|--------|
| PPF (SFTP) | pdp-client, pdp-sftp | ✅ (dépôt multi-path + consumer retrait) |
| AFNOR Flow Service (REST) | pdp-client/afnor.rs | ✅ (endpoints de base, webhooks manquants) |
| **Intra-PDP** (canal mpsc) | pdp-core/channel + pdp-client/routing | ✅ |
| PEPPOL AS4 | pdp-peppol | ✅ |
| PEPPOL SMP/SML | pdp-peppol/smp.rs | ✅ |

---

## 8. Recommandations par priorité

### Priorité haute (bloquant pour production)

1. **Wirer `AnnuaireValidationProcessor` dans le pipeline** — créé mais non intégré dans `add_emission/reception_processors` (todo #1). Sans cela, BR-FR-10/11 (G1.63) ne sont pas appliqués en production.

2. **Implémenter les webhooks AFNOR** (5 endpoints) — essentiels pour recevoir les notifications de traitement des flux envoyés.

3. **Compléter l'e-reporting** (Flux 10.2, 10.3, 10.4 + BR-FR-MAP-23) — modèles existent mais générateurs incomplets.

4. **Activer la vérification de clé SSH** en production — actuellement désactivée en dev, risque sécurité.

5. **Vérifier BR-FR-21 à BR-FR-26** dans Schematron V1.3.0 — règles V1.1 sur BAR subject code et tailles d'adresses électroniques.

### Priorité moyenne

6. **Implémenter le Flux 11** (NOUVEAU V1.3) — message permettant aux PA de transmettre les données publiables de l'Annuaire PPF aux utilisateurs.

7. **Implémenter les Flux 13 et 14** — actualisation et export annuaire par SFTP automatisé.

8. **Ajouter les endpoints Directory manquants** — POST /v1/siren/search, POST /v1/siret/search, GET routing-code, GET directory-line.

9. ~~**CDV 221 (ERREUR_ROUTAGE)**~~ — ✅ livré : `RoutingValidationProcessor` (pdp-client) détecte les PDP injoignables, `CdarProcessor` génère le CDV 221 avec ROUTAGE_ERR.

10. **Codes IRR pièces jointes** — `IRR_TAILLE_PJ`, `IRR_VID_PJ`, `IRR_EXT_DOC`, `IRR_ANTIVIRUS`.

11. **Compléter les cas d'usage partiels** (13 cas) — notes de frais, escomptes TVA, compensations, régimes de marge.

12. **Renommage PDP → PA et OD → SC** — alignement terminologie V1.2.

### Priorité basse

13. **Validation contraintes profil/syntaxe** — vérifier qu'un profil est compatible avec la syntaxe utilisée.

14. **Documentation des règles BR-FR** — cataloguer toutes les règles implémentées via Schematron.

15. **Champs d'extension EXT-FR-FE** — vérifier la couverture des 700+ champs.

16. **Logique juridictionnelle DROM/COM/TAAF**.

17. **Typage des schémas de requête** — remplacer `serde_json::Value` par des structures typées dans les clients AFNOR.

18. **Multi-vendeurs** (§4.5.4) — sub-lines / multi-seller invoices.

19. **Factur-X BASIC WL → structuré** — génération de lignes à partir de la ventilation TVA (toléré jusqu'au 01/09/2027).

20. **Flux 3, 8, 9** — formats tiers, international, B2C.

---

## 9. Couverture de tests

| Crate | Tests | Couverture |
|-------|-------|-----------|
| pdp-core | 47+ | Fondation, pipeline, exchange |
| pdp-invoice | 125+ | Parsing UBL/CII/Factur-X |
| pdp-transform | 183+ | Conversion, PDF/A-3, Flux 1 |
| **pdp-cdar** | **170+** | CDAR, cycle de vie, relais Flux 6 |
| pdp-peppol | 60+ | AS4, SBDH, SMP |
| pdp-client | 66+ | PPF SFTP, AFNOR Flow |
| pdp-validate | 24+ | XSD, Schematron |
| pdp-annuaire | 8+ | Parser F14, store, service |
| pdp-ereporting | 88+ | Flux 10.x |
| pdp-trace | 6+ | Elasticsearch |
| pdp-config | 17+ | YAML, multi-tenant |
| pdp-app | 28+ | Pipeline, server HTTP |
| pdp-sftp | 8+ | SFTP consumer + producer |
| **Total** | **921+** | **0 échec** |

**Tests notables récents** :
- 14 tests `classify_error_reason` — mapping codes motifs Annexe A V1.2
- 23 tests `pipeline_error_tests` — fichiers invalides bout en bout
- 10 tests `CdvPpfRelayProcessor` — relais 210/212 vers PPF Flux 6
- Tests CDV 213 émission/réception conformes Acteurs CDV V1.2
- Tests CDV 501 Sender=PA-R conformes Acteurs CDV

**Points d'attention** :
- pdp-trace (6 tests) sous-testé par rapport aux crates métier
- Tests intégration `AnnuaireService` nécessitent PostgreSQL (non couverts en CI)

---

## 10. Risques et points d'attention

### Risques techniques

- **Annuaire validation pas wirée** : la règle G1.63 (BR-FR-10/11) n'est pas appliquée en production. Risque de laisser passer des factures avec vendeurs/acheteurs inconnus de l'annuaire.

- **Webhooks AFNOR manquants** : la PA ne peut pas être notifiée du traitement des flux envoyés. Bloquant pour les déploiements inter-PA.

- **Flux 11 manquant** : nouveau dans V1.3 (février 2026), à intégrer pour les déploiements post-2026.

- **Tests intégration AnnuaireService** : nécessitent PostgreSQL, non couverts en CI automatique.

### Risques réglementaires

- **Date d'application** : 1er septembre 2026 pour la réforme française.
- **ViDA Directive** : facturation structurée obligatoire intra-UE B2B au 1er juillet 2030.
- **Suivi des évolutions XP Z12-012** : V1.3 actuelle, prochaines versions à surveiller.

### Risques sécurité

- **Vérification clé SSH désactivée en dev** : à activer en production.
- **Secrets dans la config** : stockage en clair dans `config.yaml`, à externaliser (vault).

---

## 11. Conclusion

**Ferrite atteint un niveau de conformité élevé** sur les exigences AFNOR :

- **XP Z12-012** : 97% — formats, profils, règles, CDAR, acteurs et motifs tous conformes
- **XP Z12-013** : 55% — APIs de base OK, webhooks et endpoints secondaires manquants
- **XP Z12-014** : 69% — 35/51 cas d'usage implémentés, 13 partiels
- **DSE AIFE** : 85% — SFTP PPF complet, annuaire local, Flux 13/14 manquants

**Les fondamentaux sont solides** : parsing UBL/CII/Factur-X, validation Schematron V1.3.0, génération CDV avec acteurs corrects (Annexe A V1.2), séparation PA-E/PA-R, relais CDV 210/212 → PPF Flux 6, annuaire local PostgreSQL avec validation G1.63.

**Les manques principaux** :
1. **Wiring `AnnuaireValidationProcessor`** (technique, rapide — priorité 1)
2. **Webhooks AFNOR** (5 endpoints — priorité 1 pour inter-PA)
3. **E-reporting Flux 10.2-10.4** (mapping incomplet — priorité 1)
4. **Flux 11** (NOUVEAU V1.3, à designer — priorité 2)
5. **Flux 13/14 annuaire** (consumer SFTP F14 + émetteur F13 — priorité 2)

**Recommandation** : finaliser les priorités 1 avant le 1er septembre 2026 (date d'application de la réforme), puis attaquer le Flux 11 et les Flux 13/14 dans la foulée.

---

*Rapport mis à jour le 26 avril 2026, basé sur l'analyse croisée du code source `pdp-facture` (commit `0bb07ff` sur `main`) et des spécifications AFNOR XP Z12-012 V1.3 / XP Z12-013 V1.2.0 / XP Z12-014 V1.3 + DSE AIFE V3.1.*
