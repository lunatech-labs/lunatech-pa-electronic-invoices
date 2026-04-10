# Spécifications techniques — PDP Facture

> Plateforme de Dématérialisation Partenaire conforme à la réforme française de la facturation électronique.

**Version** : 1.0  
**Date** : 2026-04-10  
**Normes de référence** : EN16931, XP Z12-012, XP Z12-013, XP Z12-014, Factur-X 1.0.7+

---

## Table des matières

1. [Vue d'ensemble](#1-vue-densemble)
2. [Architecture logicielle](#2-architecture-logicielle)
3. [Modèle de données](#3-modèle-de-données)
4. [Pipeline de traitement](#4-pipeline-de-traitement)
5. [Formats de facturation](#5-formats-de-facturation)
6. [Parsing et détection](#6-parsing-et-détection)
7. [Validation](#7-validation)
8. [Transformation et conversion](#8-transformation-et-conversion)
9. [Cycle de vie (CDV/CDAR)](#9-cycle-de-vie-cdvcdar)
10. [E-Reporting](#10-e-reporting)
11. [Communication PPF](#11-communication-ppf)
12. [Communication inter-PDP (AFNOR)](#12-communication-inter-pdp-afnor)
13. [Réseau PEPPOL](#13-réseau-peppol)
14. [Routage dynamique](#14-routage-dynamique)
15. [Serveur HTTP entrant](#15-serveur-http-entrant)
16. [Traçabilité et archivage](#16-traçabilité-et-archivage)
17. [Configuration](#17-configuration)
18. [Sécurité](#18-sécurité)
19. [Tests et qualité](#19-tests-et-qualité)

---

## 1. Vue d'ensemble

### 1.1 Objet

PDP Facture est une plateforme modulaire en Rust implémentant le rôle de **Plateforme de Dématérialisation Partenaire (PDP)** dans le cadre de la réforme française de la facturation électronique B2B. Elle assure :

- La **réception** de factures (fichiers, SFTP, API HTTP, PEPPOL AS4)
- Le **parsing** multi-format (UBL 2.1, CII D22B, Factur-X)
- La **validation** structurelle (XSD) et sémantique (Schematron EN16931 + BR-FR)
- La **transformation** inter-formats et la génération PDF/A-3a
- La **transmission** vers le Portail Public de Facturation (PPF) via SFTP
- L'**interopérabilité** avec d'autres PDP via le Flow Service AFNOR (XP Z12-013)
- L'échange via le **réseau PEPPOL** (AS4 4 coins)
- Le suivi du **cycle de vie** des factures (CDV/CDAR, 21 statuts)
- L'**e-reporting** (flux 10.1 à 10.4)
- La **traçabilité** complète dans Elasticsearch

### 1.2 Périmètre normatif

| Norme | Objet |
|-------|-------|
| **EN16931** | Modèle sémantique de la facture électronique européenne |
| **XP Z12-012** | Socle technique — formats, profils, règles de validation |
| **XP Z12-013** | Interopérabilité PDP↔PDP — API Flow Service, Directory Service |
| **XP Z12-014** | Cas d'usage B2B — 42 scénarios métier (Annexe A V1.3) |
| **Factur-X 1.0.7** | Profil franco-allemand PDF/A-3 + XML CII embarqué |
| **UBL 2.1** | OASIS Universal Business Language |
| **UN/CEFACT CII D22B** | Cross-Industry Invoice |
| **UN/CEFACT CDAR D22B** | Cross-Domain Acknowledgement Response (cycle de vie) |

### 1.3 Acteurs

```
┌──────────┐      ┌──────────────┐      ┌──────────────┐      ┌──────────┐
│ Émetteur │─────▶│  PDP Source  │─────▶│  PDP Dest.   │─────▶│  Acheteur│
│ (fournisseur)   │ (notre PDP)  │      │ (partenaire) │      │          │
└──────────┘      └──────┬───────┘      └──────────────┘      └──────────┘
                         │
                    ┌────▼────┐
                    │   PPF   │  Portail Public de Facturation
                    └─────────┘  (annuaire, e-reporting, archivage légal)
```

---

## 2. Architecture logicielle

### 2.1 Crates (modules)

Le projet est découpé en 12 crates Rust indépendantes :

| Crate | Responsabilité |
|-------|---------------|
| `pdp-core` | Modèle de données EN16931, moteur de pipeline async, gestion des `Exchange`, archives ZIP/tar.gz |
| `pdp-invoice` | Parsing UBL 2.1, CII D22B, Factur-X ; détection automatique du format et du type de document |
| `pdp-validate` | Validation XSD (libxml2) + Schematron (Saxon) : EN16931, BR-FR, Factur-X |
| `pdp-transform` | Conversion UBL↔CII (XSLT), génération Factur-X PDF/A-3a, PDF visuel (Typst), Flux 1 PPF |
| `pdp-cdar` | Génération, parsing et routage des CDAR D22B ; 21 statuts de cycle de vie |
| `pdp-ereporting` | Génération des flux e-reporting 10.1/10.2/10.3/10.4 |
| `pdp-peppol` | Protocole PEPPOL : SBDH, SMP/SML lookup, gateway AS4, échange inter-PDP |
| `pdp-client` | Communication PPF (SFTP tar.gz), AFNOR Flow Service (HTTP), Annuaire PISTE (OAuth2) |
| `pdp-sftp` | Consumer et Producer SFTP générique (russh), authentification RSA |
| `pdp-trace` | Archivage et traçabilité Elasticsearch (un index par SIREN) |
| `pdp-config` | Chargement de la configuration YAML |
| `pdp-app` | Binaire CLI, serveur HTTP axum, orchestration des routes |

### 2.2 Graphe de dépendances

```
pdp-app
 ├── pdp-core
 ├── pdp-config
 ├── pdp-invoice ── pdp-core
 ├── pdp-validate ── pdp-core
 ├── pdp-transform ── pdp-core, pdp-invoice
 ├── pdp-cdar ── pdp-core
 ├── pdp-ereporting ── pdp-core
 ├── pdp-peppol ── pdp-core
 ├── pdp-client ── pdp-core, pdp-sftp
 ├── pdp-sftp ── pdp-core
 └── pdp-trace ── pdp-core
```

---

## 3. Modèle de données

### 3.1 Exchange

Unité de travail transitant dans le pipeline. Encapsule un document (facture, CDV, e-reporting) et son contexte de traitement.

| Champ | Type | Description |
|-------|------|-------------|
| `id` | `Uuid` | Identifiant unique de l'échange |
| `flow_id` | `Uuid` | Identifiant du flux (groupe d'échanges liés) |
| `body` | `Vec<u8>` | Contenu brut du document (XML, PDF) |
| `source_filename` | `Option<String>` | Nom du fichier source |
| `headers` | `HashMap<String, String>` | Métadonnées de transport (`document.type`, `source.protocol`, etc.) |
| `properties` | `HashMap<String, String>` | Métadonnées de traitement (`routing.destination`, `buyer.siren`, etc.) |
| `invoice` | `Option<InvoiceData>` | Données structurées après parsing |
| `status` | `FlowStatus` | Statut courant du traitement |
| `errors` | `Vec<ExchangeError>` | Erreurs accumulées au fil du pipeline |
| `created_at` | `DateTime<Utc>` | Horodatage de création |
| `updated_at` | `DateTime<Utc>` | Horodatage de dernière modification |

### 3.2 FlowStatus

Cycle de vie d'un échange dans le pipeline interne :

```
Received → Parsing → Parsed → Validating → Validated → Transforming → Transformed
    → Distributing → Distributed → WaitingAck → Acknowledged
                                                      ↘ Rejected
                                             Error ← Cancelled
```

### 3.3 InvoiceData (EN16931)

Structure complète conforme au modèle sémantique EN16931, enrichie des extensions françaises (EXT-FR-FE).

#### Identification

| Champ | BT | Description |
|-------|-----|-------------|
| `invoice_number` | BT-1 | Numéro de facture (obligatoire) |
| `issue_date` | BT-2 | Date d'émission (YYYY-MM-DD) |
| `invoice_type_code` | BT-3 | Type : 380 (facture), 381 (avoir), 384 (correction), 386 (acompte), 389 (auto-facture) |
| `currency` | BT-5 | Code devise (EUR, USD, etc.) |
| `business_process` | BT-23 | Processus métier (S1, B1, A1, A4, A8, B8, S8, M8, etc.) |
| `profile` | BT-24 | Profil : Base (sans lignes) ou Full (avec lignes) |

#### Parties

| Groupe | BG | Champs principaux |
|--------|-----|-------------------|
| **Vendeur** | BG-4 | `seller_name`, `seller_siret`, `seller_vat_id`, `seller_address`, `seller_endpoint_id` |
| **Acheteur** | BG-7 | `buyer_name`, `buyer_siret`, `buyer_vat_id`, `buyer_address`, `buyer_endpoint_id` |
| **Bénéficiaire** | BG-10 | `payee_name`, `payee_id`, `payee_siret` |
| **Représentant fiscal** | BG-11 | `tax_representative_name`, `tax_representative_vat_id` |
| **Factureur** (rôle II) | EXT-FR-FE-BG-05 | `invoicer_name`, `invoicer_id`, `invoicer_vat_id` |
| **Destinataire adressé** (rôle IV) | EXT-FR-FE-BG-04 | `addressed_to_name`, `addressed_to_id` |
| **Mandant de facturation** | EXT-FR-FE | `billing_mandate_name`, `billing_mandate_id`, `billing_mandate_reference` |
| **Agent acheteur** (rôle AB) | EXT-FR-FE-BG-01 | `buyer_agent_name`, `buyer_agent_id` |
| **Payeur** | EXT-FR-FE | `payer_name`, `payer_id` |

#### Totaux (BG-22)

| Champ | BT | Description |
|-------|-----|-------------|
| `total_ht` | BT-109 | Total HT (somme des montants nets des lignes) |
| `total_ttc` | BT-112 | Total TTC |
| `total_tax` | BT-110 | Total TVA |
| `allowance_total_amount` | BT-107 | Total des remises document |
| `charge_total_amount` | BT-108 | Total des charges document |
| `prepaid_amount` | BT-113 | Montant prépayé |
| `payable_amount` | BT-115 | Montant à payer |

#### Ventilation TVA (BG-23)

Chaque `TaxBreakdown` contient :
- `taxable_amount` (BT-116) — base imposable
- `tax_amount` (BT-117) — montant TVA
- `category_code` (BT-118) — catégorie : S (standard), Z (taux zéro), E (exonéré), AE (autoliquidation), K (intra-communautaire), G (export), O (hors champ), L (Corse IGIC), M (Ceuta/Melilla IPSI)
- `percent` (BT-119) — taux appliqué

#### Lignes de facture (BG-25)

| Champ | BT | Description |
|-------|-----|-------------|
| `line_id` | BT-126 | Identifiant de ligne |
| `quantity` | BT-129 | Quantité facturée |
| `unit_code` | BT-130 | Code unité (UN/ECE Rec. 20) |
| `line_net_amount` | BT-131 | Montant net de la ligne |
| `price` | BT-146 | Prix unitaire net |
| `gross_price` | BT-148 | Prix unitaire brut |
| `item_name` | BT-153 | Nom de l'article |
| `tax_category_code` | BT-151 | Catégorie TVA de la ligne |
| `tax_percent` | BT-152 | Taux TVA de la ligne |
| `allowance_charges` | BG-27/28 | Remises et charges de ligne |
| `sub_lines` | EXT-FR-FE-162/163 | Sous-lignes récursives (multi-fournisseurs B8/S8/M8) |

#### Clé métier d'unicité (InvoiceKey)

```
{SIREN_VENDEUR}/{NUMERO_FACTURE}/{ANNEE_EMISSION}
```

Règle BR-FR-01 : une facture est unique par combinaison SIREN vendeur + numéro facture + année d'émission.

### 3.4 Pièces jointes (BG-24)

| Champ | BT | Description |
|-------|-----|-------------|
| `id` | BT-122 | Identifiant de la pièce jointe |
| `description` | BT-123 | Description |
| `external_uri` | BT-124 | URI externe |
| `embedded_content` | BT-125 | Contenu embarqué (base64 en XML, binaire en PDF) |
| `mime_code` | — | Type MIME |
| `filename` | — | Nom de fichier |

Les pièces jointes sont préservées dans toutes les conversions.

---

## 4. Pipeline de traitement

### 4.1 Architecture du pipeline

Le pipeline est construit autour de trois abstractions :

```rust
trait Consumer: Send + Sync {
    async fn poll(&self) -> PdpResult<Vec<Exchange>>;
}

trait Processor: Send + Sync {
    async fn process(&self, exchange: Exchange) -> PdpResult<Exchange>;
}

trait Producer: Send + Sync {
    async fn send(&self, exchange: Exchange) -> PdpResult<Exchange>;
}
```

Une **Route** chaîne un Consumer, une séquence de Processors, et un Producer :

```
Consumer ──poll()──▶ [Processor₁ → Processor₂ → … → Processorₙ] ──send()──▶ Producer
                                                                    │
                                                           error_handler (optionnel)
```

### 4.2 Types d'Endpoint

| Type | Consumer | Producer | Transport |
|------|----------|----------|-----------|
| `file` | Surveillance répertoire local (stabilité fichier) | Écriture fichier local | Filesystem |
| `sftp` | Polling SFTP distant, extraction tar.gz/zip | Upload SFTP distant | SSH/SFTP |
| `ppf` | — | Routage dynamique PPF SFTP ou PDP AFNOR | SFTP + HTTP |
| `http` | Serveur HTTP axum (POST /v1/flows) | — | HTTP/REST |

### 4.3 Pipeline type d'une facture entrante

```
1. Consumer (fichier/SFTP/HTTP)
   └── Extraction archives tar.gz/zip si nécessaire
2. DocumentTypeRouter
   └── Détection : facture / CDAR / e-reporting → header document.type
3. IrrecevabiliteProcessor
   └── Vérification réception (fichier non vide, format reconnu)
   └── Échec → génération CDAR 501 (Irrecevable)
4. ParseProcessor
   └── Détection auto format (UBL/CII/FacturX)
   └── Extraction InvoiceData complète
5. XmlValidateProcessor
   └── Validation XSD structurelle
   └── Validation Schematron EN16931 + BR-FR
6. PpfFlux1Processor
   └── Transformation vers profil Flux 1 (Base ou Full)
7. TransformProcessor (optionnel)
   └── Conversion vers format cible (UBL↔CII, Factur-X, PDF)
8. CdarProcessor
   └── Génération CDV 200 (Déposée) si valide
   └── Génération CDV 213 (Rejetée) si invalide
9. RoutingResolverProcessor
   └── Lookup Annuaire PISTE → destination (PPF ou PDP partenaire)
10. DynamicRoutingProducer
    └── Envoi PPF via SFTP tar.gz OU PDP partenaire via AFNOR Flow Service
```

---

## 5. Formats de facturation

### 5.1 Formats supportés

| Format | Norme | Encoding | Profils |
|--------|-------|----------|---------|
| **UBL 2.1** | OASIS UBL | XML | Invoice, CreditNote |
| **CII D22B** | UN/CEFACT | XML | CrossIndustryInvoice |
| **Factur-X** | Franco-allemand | PDF/A-3a + XML CII | MINIMUM, BASIC WL, BASIC, EN16931, EXTENDED |
| **CDAR D22B** | UN/CEFACT | XML | CrossDomainAcknowledgementResponse |
| **E-Reporting** | PPF V1.0 | XML | Flux 10.1/10.2/10.3/10.4 |

### 5.2 Matrice de conversion

| Source ↓ / Cible → | CII | UBL | Factur-X PDF/A-3 | PDF visuel |
|---------------------|-----|-----|-------------------|------------|
| **UBL** | XSLT | — | XSLT + Typst + lopdf | Typst |
| **CII** | — | XSLT | Typst + lopdf | Typst |
| **Factur-X** | extraction XML | extraction + XSLT | — | retour PDF existant |

### 5.3 Factur-X PDF/A-3a

La génération Factur-X produit un PDF conforme ISO 19005-3 (PDF/A-3a) :

- Rendu visuel via **Typst** (moteur embarqué, ~100ms)
- Embarquement du XML CII en tant que `factur-x.xml` (AFRelationship = Source)
- Embarquement des pièces jointes BG-24 (AFRelationship = Supplement)
- Métadonnées XMP avec extension Factur-X (conformLevel, documentType, etc.)
- Corrections PDF/A via **qpdf** (OutputIntents, headers)
- Conformité vérifiable par **veraPDF**

---

## 6. Parsing et détection

### 6.1 Détection automatique du format

La fonction `detect_format()` identifie le format source :

| Signal | Format détecté |
|--------|---------------|
| Élément racine `<Invoice>` ou `<CreditNote>` (namespace UBL) | UBL 2.1 |
| Élément racine `<CrossIndustryInvoice>` (namespace CII) | CII D22B |
| Fichier PDF avec XML embarqué `factur-x.xml` | Factur-X |

### 6.2 Détection du type de document

La fonction `detect_document_type()` distingue :

| Signal | Type |
|--------|------|
| `<Invoice>`, `<CreditNote>`, `<CrossIndustryInvoice>` | `Invoice` |
| `<CrossDomainAcknowledgementResponse>` | `Cdar` |
| Namespace e-reporting PPF | `EReporting` |

### 6.3 Parsers

Chaque parser extrait l'intégralité du modèle `InvoiceData` :

- **UblParser** : navigation XPath sur XML UBL, extraction des 90+ champs EN16931
- **CiiParser** : navigation XPath sur XML CII D22B
- **FacturXParser** : extraction du XML embarqué dans le PDF (via lopdf), puis parsing CII
- Prise en charge des sous-lignes récursives (XP Z12-012 §3.3.5)
- Extraction des pièces jointes BG-24

---

## 7. Validation

### 7.1 Niveaux de validation

La validation s'effectue en trois niveaux successifs :

#### Niveau 1 — Validation XSD (structurelle)

- Moteur : **libxml2** via bindings Rust
- Vérifie la conformité du XML aux schémas XSD
- Schémas supportés : CII D22B, UBL 2.1, Factur-X (EXTENDED, EN16931, BASIC)

#### Niveau 2 — Validation Schematron (sémantique)

- Moteur : **Saxon-HE** (XSLT 2.0)
- Règles EN16931 (BT-*, BG-*)
- Règles BR-FR (spécifiques France)
- Règles Factur-X (PEPPOL, éléments vides, code lists)

#### Niveau 3 — Validation métier

- **BR-FR-12** : Unicité du numéro de facture (SIREN + numéro + année)
- **BR-FR-13** : Détection de doublons
- Cohérence des montants (total_ttc = total_ht + total_tax)
- Champs obligatoires français (seller_siret, buyer_siret, etc.)
- Codes TVA conformes aux code lists EN16931

### 7.2 Rapport de validation

```rust
ValidationReport {
    is_valid: bool,
    level: ValidationLevel,  // XSD, SCHEMATRON, BUSINESS
    issues: Vec<ValidationIssue>,
}

ValidationIssue {
    level: ValidationLevel,
    rule_id: String,       // ex: "EN16931-BT1", "BR-FR-12"
    message: String,
    location: Option<String>,  // XPath
}
```

---

## 8. Transformation et conversion

### 8.1 Moteurs de transformation

| Moteur | Technologie | Rôle |
|--------|------------|------|
| **XSLT** | Saxon-HE | Conversion UBL↔CII bidirectionnelle |
| **Typst** | Embarqué (in-process) | Rendu PDF visuel (~100ms) |
| **lopdf** | Crate Rust | Assemblage PDF/A-3a, embarquement XML et pièces jointes |
| **qpdf** | Binaire externe | Corrections PDF/A (OutputIntents, headers) |

### 8.2 Flux 1 PPF

Le Flux 1 est le format de transmission des factures de la PDP vers la PPF.

#### Profils

| Profil | Contenu | Usage |
|--------|---------|-------|
| **Base** | Métadonnées sans lignes de facture | Factures simples, archivage PPF |
| **Full** | Toutes les données y compris les lignes | Factures complètes, destinataire PPF |

#### Détection automatique du profil

La stratégie peut être configurée (`auto`, `base`, `full`) :
- `auto` : Full si le destinataire est enregistré sur la PPF, Base sinon
- `base` / `full` : forcé

#### Nommage SFTP

```
{PROFIL}_{SEQUENCE}.xml

Exemples :
  Base_000001.xml
  Full_000042.xml
```

---

## 9. Cycle de vie (CDV/CDAR)

### 9.1 Format CDAR D22B

Le Compte-rendu De Vie (CDV) utilise le format UN/CEFACT `CrossDomainAcknowledgementResponse` (CDAR D22B). Il informe les parties de l'avancement du traitement d'une facture.

### 9.2 Types de CDV

| TypeCode | Catégorie | Objet |
|----------|-----------|-------|
| `305` | Transmission | Événements de transport (dépôt, routage, distribution) |
| `23` | Traitement | Événements métier (approbation, refus, paiement) |

### 9.3 Statuts de cycle de vie

#### Statuts de transmission (TypeCode 305)

| Code | Statut | Description |
|------|--------|-------------|
| 200 | Déposée | Facture déposée sur la PDP émettrice |
| 201 | Émise | Facture émise par la PDP émettrice |
| 202 | Reçue | Facture reçue par la PDP réceptrice |
| 203 | Mise à disposition | Facture disponible pour le destinataire |
| 213 | Rejetée | Facture rejetée (validation échouée) |
| 300 | Transmise PPF | Facture transmise à la PPF |
| 301 | Transmise PDP | Facture transmise à une PDP partenaire |
| 400 | Transmise destinataire | Facture délivrée au destinataire final |
| 501 | Irrecevable | Facture irrecevable (erreur de réception) |

#### Statuts de traitement (TypeCode 23)

| Code | Statut | Description |
|------|--------|-------------|
| 204 | Prise en charge | Facture prise en charge par le destinataire |
| 205 | Approuvée | Facture approuvée |
| 206 | Approuvée partiellement | Facture partiellement approuvée |
| 207 | En litige | Facture contestée |
| 208 | Suspendue | Traitement suspendu |
| 209 | Service fait | Service fait constaté |
| 210 | Refusée | Facture refusée |
| 211 | Paiement transmis | Ordre de paiement transmis |
| 212 | Encaissée | Paiement reçu |
| 214 | Visée | Facture visée |
| 220 | Annulée | Facture annulée |

#### Statuts d'affacturage

| Code | Statut |
|------|--------|
| 225 | Affacturée — Déposée |
| 226 | Affacturée — Émise |
| 227 | Affacturée — Reçue |
| 228 | Affacturée — Prise en charge |

### 9.4 Codes motif de rejet (StatusReasonCode)

46 codes motif normalisés, dont :

- `Doublon` — Facture en double
- `DestErr` — Destinataire erroné
- `SiretErr` — SIRET invalide
- `MontantTotalErr` — Total incorrect
- `IrrSyntax` — Erreur de syntaxe XML
- `IrrTaillePj` — Taille pièce jointe excessive
- `RejSeman` — Rejet sémantique (Schematron)

### 9.5 Rôles des parties (RoleCode)

| Code | Rôle | Description |
|------|------|-------------|
| SE | Seller | Vendeur / fournisseur |
| BY | Buyer | Acheteur |
| AB | Buyer Agent | Agent de l'acheteur |
| II | Invoicer | Factureur (délégation de facturation) |
| IV | Invoicee | Destinataire adressé |
| PE | Payee | Bénéficiaire du paiement |
| PR | Payer | Payeur |
| DL | Deliver | Livraison |
| SR | Tax Representative | Représentant fiscal |
| WK | Worker | Sous-traitant |
| DFH | Billing Mandate | Mandant de facturation |

### 9.6 Génération automatique

| Événement | CDV généré |
|-----------|-----------|
| Facture valide reçue | 200 Déposée |
| Validation échouée | 213 Rejetée (avec motifs) |
| Réception échouée (fichier corrompu, format inconnu) | 501 Irrecevable |
| Transmission PPF réussie | 300 Transmise PPF |
| Transmission PDP réussie | 301 Transmise PDP |

---

## 10. E-Reporting

### 10.1 Flux e-reporting

| Flux | Objet | Direction |
|------|-------|-----------|
| 10.1 | Transactions de ventes | PDP → PPF |
| 10.2 | Paiements de ventes | PDP → PPF |
| 10.3 | Transactions d'acquisitions | PDP → PPF |
| 10.4 | Paiements d'acquisitions | PDP → PPF |

### 10.2 Format

XML conforme au schéma XSD PPF V1.0. Chaque flux est encapsulé dans une archive tar.gz pour transmission SFTP vers la PPF.

---

## 11. Communication PPF

### 11.1 Protocole SFTP

La communication PDP→PPF s'effectue exclusivement par **SFTP** avec des archives **tar.gz**.

#### Convention de nommage

```
{CODE_INTERFACE}_{CODE_APP}_{IDENTIFIANT_FLUX}.tar.gz
```

| Paramètre | Description | Exemple |
|-----------|-------------|---------|
| `CODE_INTERFACE` | Code du flux | `FFE0111A` (Flux 1 UBL) |
| `CODE_APP` | Code application PISTE | `AAA123` |
| `IDENTIFIANT_FLUX` | Séquence incrémentale | `000042` |

#### Codes interface

| Code | Flux | Format |
|------|------|--------|
| `FFE0111A` | Flux 1 — Factures | UBL |
| `FFE0112A` | Flux 1 — Factures | CII |
| `FFE0614A` | Flux 6 — CDV factures | CDAR |
| `FFE0654A` | Flux 6 — CDV statuts | CDAR |
| `FFE1025A` | Flux 10 — E-reporting | XML PPF |

#### Contraintes

- Taille maximale d'une archive : **1 Go**
- Taille maximale par fichier dans l'archive : **120 Mo**
- Plusieurs fichiers de même nature et format peuvent être groupés dans un seul tar.gz

#### Authentification

- Clé privée RSA (X.509v3)
- Vérification `known_hosts`
- TLS 1.2+

### 11.2 Annuaire PISTE

L'Annuaire PISTE permet de déterminer la PDP d'inscription d'un assujetti à partir de son SIREN/SIRET.

- **Protocole** : REST API avec OAuth2 Client Credentials
- **Token endpoint** : `https://oauth.piste.gouv.fr/api/oauth/token`
- **Résultat** : matricule PDP (ex: `0000` = PPF, `1234` = PDP partenaire)

### 11.3 Environnements PPF

| Environnement | Sous-domaine |
|--------------|-------------|
| dev | `env.dev.aife.economie.gouv.fr` |
| int | `env.int.aife.economie.gouv.fr` |
| rec | `env.rec.aife.economie.gouv.fr` |
| preprod | `env.pre.prod.aife.economie.gouv.fr` |
| prod | `api.aife.economie.gouv.fr` |

---

## 12. Communication inter-PDP (AFNOR)

### 12.1 Flow Service (XP Z12-013 Annexe A)

L'échange de factures entre PDP s'effectue via le **Flow Service** AFNOR, une API REST conforme à la norme XP Z12-013.

#### Envoi d'un flux (POST /v1/flows)

```http
POST /v1/flows HTTP/1.1
Content-Type: multipart/form-data

--boundary
Content-Disposition: form-data; name="flowInfo"
Content-Type: application/json

{
  "tracking_id": "uuid",
  "sender_matricule": "1234",
  "receiver_matricule": "5678",
  "flow_type": "INVOICE",
  "document_type_code": "380",
  "format": "UBL",
  "file_name": "facture.xml",
  "file_hash": "sha256:abc123...",
  "metadata": { ... }
}

--boundary
Content-Disposition: form-data; name="file"; filename="facture.xml"
Content-Type: application/xml

[contenu XML ou PDF brut — PAS de tar.gz]
--boundary--
```

> **Important** : Les fichiers sont envoyés directement en XML ou PDF via multipart. Le tar.gz est réservé **exclusivement** à la communication SFTP avec la PPF.

#### Réception de la réponse

```json
{
  "flow_id": "uuid",
  "status": "RECEIVED",
  "message": "Flux accepté pour traitement"
}
```

### 12.2 Directory Service (XP Z12-013 Annexe B)

Permet la découverte des PDP partenaires et de leurs endpoints Flow Service.

### 12.3 Authentification

- OAuth2 via PISTE (même mécanisme que l'Annuaire)
- Token Bearer dans le header `Authorization`

---

## 13. Réseau PEPPOL

### 13.1 Architecture 4 coins

```
Corner 1         Corner 2           Corner 3         Corner 4
(Émetteur) ──▶  (AP émetteur) ──▶  (AP récepteur) ──▶ (Destinataire)
                 notre PDP          PDP partenaire
                     ↕                    ↕
                  SMP/SML             SMP/SML
```

### 13.2 Composants

#### Participant ID

```
scheme::identifier
Exemple : 0002::12345678901234  (SIRET français, scheme 0002)
```

#### SBDH (Standard Business Document Header)

Enveloppe XML qui encapsule la facture pour le transport AS4 :
- Sender / Receiver (Participant IDs)
- Document Type (UBL Invoice, Credit Note, etc.)
- Process ID
- Message ID et horodatage

#### SMP Lookup

1. Hash MD5 du Participant ID → requête SML
2. Résolution DNS → URL du SMP
3. Requête SMP → endpoint AS4 du récepteur (URL, certificat)

#### AS4 (ebMS 3.0)

- Protocole SOAP/MIME
- Signature numérique (certificats PKI PEPPOL)
- Chiffrement AES-128/256-CBC
- Compression gzip
- Accusés de réception (receipt)

---

## 14. Routage dynamique

### 14.1 Principe

Le routage dynamique détermine la destination d'une facture en fonction du SIREN de l'acheteur :

```
SIREN acheteur ──▶ Annuaire PISTE ──▶ matricule PDP
                                         │
                                  ┌──────┴──────┐
                                  ▼              ▼
                          matricule "0000"   matricule "XXXX"
                          = PPF              = PDP partenaire
                          → SFTP tar.gz      → AFNOR Flow Service HTTP
```

### 14.2 Composants

#### RoutingResolverProcessor

Processor léger qui consulte l'Annuaire et positionne les propriétés de routage sur l'Exchange :
- `routing.destination` : `ppf` ou `pdp:{matricule}`
- `routing.partner_name` : nom de la PDP partenaire
- `routing.flow_service_url` : URL du Flow Service

#### DynamicRoutingProducer

Producer qui exploite les propriétés de routage pour envoyer :
- **`ppf`** → `PpfSftpProducer` (SFTP tar.gz)
- **`pdp:{matricule}`** → `AfnorFlowProducer` correspondant (HTTP multipart)
- **Fallback** → écriture fichier local si la destination est inconnue

#### PartnerDirectory

Registre des PDP partenaires configurées, avec leurs URLs Flow Service. Chargé depuis la configuration YAML.

---

## 15. Serveur HTTP entrant

### 15.1 Endpoints

Le serveur HTTP (axum) implémente le rôle de **récepteur** du Flow Service AFNOR.

| Méthode | Endpoint | Description |
|---------|----------|-------------|
| `POST` | `/v1/flows` | Réception de flux (multipart : flowInfo JSON + fichier) |
| `POST` | `/v1/webhooks/callback` | Réception de notifications webhook |
| `GET` | `/v1/flows/{flowId}` | Consultation d'un flux (stub 501) |
| `GET` | `/v1/healthcheck` | Vérification de santé |

### 15.2 Réception de flux (POST /v1/flows)

1. Extraction du multipart (flowInfo JSON + fichier binaire)
2. Vérification d'intégrité SHA-256 (`file_hash` vs contenu reçu)
3. Injection dans le pipeline interne via canal mpsc
4. Réponse 202 Accepted avec `flow_id`

### 15.3 Webhooks (POST /v1/webhooks/callback)

1. Vérification de la signature HMAC-SHA256 (header `X-Webhook-Signature`)
2. Clé secrète configurée (`webhook_secret`)
3. Parsing du payload JSON (flow_id, status, reason_code, timestamp)

---

## 16. Traçabilité et archivage

### 16.1 Architecture Elasticsearch

- **Un index par SIREN** : `pdp-traces-{siren}` (ex: `pdp-traces-123456789`)
- Stockage des documents XML et PDF en champ binaire
- Recherche full-text sur les métadonnées

### 16.2 Données archivées

| Champ | Description |
|-------|-------------|
| `invoice_key` | Clé métier (SIREN/numéro/année) |
| `invoice_number` | Numéro de facture |
| `seller_siret` / `buyer_siret` | Identifiants des parties |
| `issue_date` | Date d'émission |
| `total_ttc` | Montant TTC |
| `source_format` | Format source (UBL/CII/FacturX) |
| `status` | Dernier statut connu |
| `xml_content` | XML source archivé |
| `pdf_content` | PDF archivé (si applicable) |
| `events` | Historique des événements de cycle de vie |

---

## 17. Configuration

### 17.1 Structure YAML

```yaml
# Identité de la PDP
pdp:
  id: "PDP-001"
  name: "Ma PDP"
  siret: "12345678901234"
  siren: "123456789"
  matricule: "1234"

# Elasticsearch
elasticsearch:
  url: "http://localhost:9200"

# Validation
validation:
  specs_dir: ./specs
  xsd_enabled: true
  en16931_enabled: true
  br_fr_enabled: true

# Polling
polling:
  interval_secs: 60

# Logging
logging:
  format: text      # text | json
  level: info       # debug | info | warn | error

# Serveur HTTP entrant (AFNOR Flow Service récepteur)
http_server:
  host: "0.0.0.0"
  port: 8080
  webhook_secret: "secret-hmac-key"

# Communication PPF
ppf:
  environment: prod           # dev | int | rec | preprod | prod
  code_interface: FFE0111A
  code_application_piste: AAA123
  flux1_output_dir: ./output/flux1
  flux1_profile: auto         # auto | base | full
  initial_sequence: 1
  auth:
    token_url: https://oauth.piste.gouv.fr/api/oauth/token
    client_id: "${PISTE_CLIENT_ID}"
    client_secret: "${PISTE_CLIENT_SECRET}"
    scope: "openid"
  sftp:
    host: sftp.ppf.gouv.fr
    port: 22
    username: pdp_aaa123
    private_key_path: /app/keys/id_rsa
    remote_path: /sas/depot
    known_hosts_path: /app/keys/known_hosts

# Communication inter-PDP (AFNOR)
afnor:
  flow_service_url: "https://flow.our-pdp.fr/v1"
  directory_service_url: "https://dir.our-pdp.fr/v1"
  auth:
    token_url: https://oauth.piste.gouv.fr/api/oauth/token
    client_id: "${AFNOR_CLIENT_ID}"
    client_secret: "${AFNOR_CLIENT_SECRET}"
    scope: "openid"
  partners:
    - matricule: "5678"
      name: "PDP Partenaire Alpha"
      flow_service_url: "https://flow.alpha-pdp.fr/v1"
    - matricule: "9012"
      name: "PDP Partenaire Beta"
      flow_service_url: "https://flow.beta-pdp.fr/v1"

# Routes de traitement
routes:
  - id: route-reception-ubl
    description: "Réception factures UBL depuis SFTP"
    enabled: true
    source:
      endpoint_type: sftp
      host: sftp.client.fr
      port: 22
      username: pdp-user
      private_key_path: ~/.ssh/client_key
      path: /invoices/outbox
      file_pattern: "*.xml"
    destination:
      endpoint_type: ppf        # routage dynamique
      path: ./output/fallback   # fallback si destination inconnue
    error_destination:
      endpoint_type: file
      path: ./output/errors
    validate: true
    generate_cdar: true
    transform_to: null
    cdar_receiver:
      pdp_id: "PPF"
      pdp_name: "Portail Public de Facturation"
```

---

## 18. Sécurité

### 18.1 Authentification

| Contexte | Mécanisme |
|----------|-----------|
| SFTP PPF | Clé privée RSA + certificat X.509v3, vérification known_hosts |
| SFTP clients | Clé privée RSA, vérification known_hosts |
| API PISTE (Annuaire, OAuth) | OAuth2 Client Credentials (JWT Bearer) |
| AFNOR Flow Service | OAuth2 Bearer Token via PISTE |
| Webhooks entrants | HMAC-SHA256 (header `X-Webhook-Signature`) |
| PEPPOL AS4 | Certificats PKI PEPPOL, signature XML, chiffrement AES |

### 18.2 Intégrité des données

| Contexte | Mécanisme |
|----------|-----------|
| Fichiers dans tar.gz PPF | Vérification à l'extraction |
| Flow Service réception | SHA-256 du fichier vérifié contre `file_hash` du flowInfo |
| PEPPOL AS4 | Signature numérique SOAP |

### 18.3 Variables d'environnement

Les secrets (client_id, client_secret, webhook_secret) supportent la substitution de variables d'environnement (`${VAR}`) dans la configuration YAML.

---

## 19. Tests et qualité

### 19.1 Couverture

| Crate | Tests | Couverture |
|-------|-------|-----------|
| `pdp-core` | 56 | Archives, Exchange, Route, Router, modèle InvoiceData |
| `pdp-invoice` | 104 | Parsing UBL/CII/FacturX, détection, validation BR-FR, pièces jointes |
| `pdp-validate` | 14 | XSD, Schematron EN16931 + BR-FR |
| `pdp-transform` | 184 | Conversions, Flux 1 (66 sous-cas), Factur-X, attachments |
| `pdp-cdar` | 69 | Génération, parsing, routage, 21 statuts, multi-références |
| `pdp-ereporting` | 88 | 4 types de flux, génération XSD PPF |
| `pdp-peppol` | 53 | SBDH, SMP lookup, AS4, inter-PDP |
| `pdp-client` | 42 | PPF SFTP, AFNOR Flow, Annuaire, routage |
| `pdp-sftp` | 7 | Consumer, Producer, patterns |
| `pdp-trace` | 2 | Archivage Elasticsearch |
| `pdp-config` | 3 | Chargement YAML |
| `pdp-app` | 12+ | Serveur HTTP (healthcheck, multipart, SHA-256, HMAC webhooks) |
| **Total** | **774+** | |

### 19.2 Types de tests

- **Tests unitaires** : dans chaque crate, modules `#[cfg(test)]`
- **Tests d'intégration** : dans `tests/` (ex: `routing_integration.rs`)
- **Tests round-trip** : tar.gz → extraction → vérification contenu
- **Tests Schematron** : validation complète EN16931 sur fixtures réelles
- **Tests PDF/A** : conformité veraPDF pour les Factur-X générés
- **Benchmarks Criterion** : parsing, validation, transformation, pipeline complet

### 19.3 Fixtures

Jeu complet de factures de test couvrant les 42 cas d'usage XP Z12-014 :
- Factures simples (380), avoirs (381), corrections (384), acomptes (386), auto-factures (389)
- Multi-fournisseurs (B8), délégation (S8), marketplace (A8), sous-traitance (A4)
- CDV pour chaque statut (200→501)
- Flux e-reporting (10.1→10.4)
- Formats UBL, CII, Factur-X pour chaque cas
