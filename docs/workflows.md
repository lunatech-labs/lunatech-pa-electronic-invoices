# Workflows métier — Cas d'usage AFNOR XP Z12-014

Cette page décrit les **5 workflows fondamentaux** d'une PDP émission/réception
selon AFNOR XP Z12-014 V1.3 (Cas d'usage B2B), tels qu'implémentés dans Ferrite.
Chaque workflow est illustré par un diagramme de séquence Mermaid et la
liste précise des processors qui s'enchaînent.

> **Note architecture** : les diagrammes ci-dessous mentionnent `TraceProcessor::*` et
> `WebhookAckProcessor` — c'est l'ancienne nomenclature, conservée pour rester
> lisibles côté pédagogie. Depuis la V3 de la migration vers le bus, ces appels sont
> remplacés par `ExchangeSnapshotProcessor` (snapshot XML/PDF) + `LifecycleProcessor`
> (publication d'événements sur le bus). Les webhooks AFNOR et l'archivage des
> événements dans Elasticsearch sont des subscribers du bus. Voir [events.md](events.md).

| # | Workflow | UC AFNOR | Statut Ferrite |
|---|----------|----------|----------------|
| [1](#1-émission-classique-uc1) | Émission classique | UC1 § 2.1 | ✅ |
| [2](#2-rejet-à-lémission-uc2) | Rejet à l'émission (validation invalide) | UC2 § 2.2 | ✅ |
| [3](#3-réception-classique-uc3) | Réception classique (PDP destinataire) | UC3 § 2.3 / 2.4 | ✅ |
| [4](#4-routage-intra-pdp-uc4) | Routage intra-PDP (vendeur et acheteur sur la même PDP) | UC4 § 2.5 | ✅ |
| [5](#5-cdv-210212-avec-relais-ppf-flux-6) | CDV 210/212 avec relais PPF Flux 6 | UC5 § 2.6-2.7 | ✅ |

## Conventions

**Acteurs** :
- **SE** (Issuer) : émetteur du document (vendeur ou plateforme émettrice)
- **PA-E** : Plateforme Agréée Émettrice (anciennement PDP émettrice)
- **PA-R** : Plateforme Agréée Réceptrice (anciennement PDP destinataire)
- **PPF** : Portail Public de Facturation (AIFE / DGFiP)
- **BY** (Buyer) : acheteur

**Codes interface PPF** :
- `FFE0111A` : Flux 1 UBL (données réglementaires)
- `FFE0112A` : Flux 1 CII (données réglementaires)
- `FFE0614A` : Flux 6 — Cycle de vie facture (CDAR)
- `FFE0654A` : Flux 6 — Statuts obligatoires (CDV 210/212)
- `FFE0634A` : Flux 6 — CDV annuaire (réponse à F13)
- `FFE1025A` : Flux 10 — E-reporting (transactions + paiements)
- `FFE1235A` : Flux 13 — Actualisation annuaire (PDP→PPF)
- `FFE1435A` : Flux 14 — Export annuaire (PPF→PDP)

**Statuts CDV implémentés** :
- `200` Déposée, `202` Reçue, `204` Prise en charge
- `205` Approuvée, `207` En litige, `210` Refusée, `212` Encaissée
- `213` Rejetée (validation), `220` Annulée
- `221` Erreur de routage, `501` Irrecevable

---

## 1. Émission classique (UC1)

**Cas d'usage** : un vendeur dépose une facture conforme via Ferrite. La facture
est validée (XSD + Schematron + annuaire G1.63), transformée en Flux 1 PPF,
acquittée par CDV 200, puis routée vers la PDP destinataire (ou le PPF en
fallback). L'émission produit toujours un Flux 1 (obligation réglementaire).

### Pipeline

```
TraceProcessor::received → ReceptionProcessor::strict
  → IrrecevabiliteProcessor → DocumentTypeRouter
  → CdvPpfRelayProcessor → ParseProcessor
  → DuplicateCheckProcessor → ValidateProcessor
  → XmlValidateProcessor → TraceProcessor::validated
  → WebhookAckProcessor (Out, "Ok")
  → AnnuaireValidationProcessor (Emission, G1.63 BR-FR-10/11)
  → WebhookAckProcessor (Out)
  → PpfFlux1Processor (Base/Full XML, FFE0111A/FFE0112A)
  → TransformProcessor (optionnel UBL↔CII)
  → RoutingResolverProcessor (résolution annuaire)
  → RoutingValidationProcessor (vérifie producer disponible)
  → CdarProcessor::emission (CDV 200)
  → DynamicRoutingProducer (PPF / PDP-{matricule} / INTRA-PDP)
  → TraceProcessor::distributed
```

### Diagramme de séquence

```mermaid
sequenceDiagram
    autonumber
    participant SE as SE (Vendeur)
    participant PAE as PA-E (Ferrite émetteur)
    participant PPF as PPF / SAS SFTP
    participant PAR as PA-R (PDP destinataire)
    participant Webhook as Webhook subscribers

    SE->>PAE: Dépose facture (UBL/CII/Factur-X)<br/>POST /v1/flows ou SFTP {siren}/in/
    PAE->>PAE: ReceptionProcessor (taille, format)
    PAE->>PAE: ParseProcessor + DuplicateCheck
    PAE->>PAE: ValidateProcessor + XmlValidate (XSD + Schematron)

    Note over PAE: BR-FR-10 / BR-FR-11
    PAE->>PAE: AnnuaireValidationProcessor<br/>(vendeur + acheteur dans annuaire)
    PAE->>Webhook: flow.ack.updated (Validated → Ok)

    PAE->>PPF: Flux 1 Base/Full<br/>(FFE0111A ou FFE0112A)
    Note right of PPF: Données réglementaires<br/>(obligatoire)

    PAE->>PAE: RoutingResolverProcessor<br/>(annuaire local → PDP-XXXX)
    PAE->>PAE: RoutingValidationProcessor<br/>(producer AFNOR disponible ?)
    PAE->>PAE: CdarProcessor::emission<br/>(CDV 200 « Déposée »)

    PAE->>PAR: POST /v1/flows (AFNOR Flow Service)<br/>multipart flowInfo + facture
    PAR-->>PAE: 202 Accepted {flowId}
    PAE->>SE: Notification CDV 200<br/>(via webhook ou polling)
    PAE->>Webhook: flow.ack.updated (Distributed → Ok)
```

### Configuration minimale

```yaml
pdp:
  id: "PA-EXAMPLE-001"
  name: "Ferrite Example"
  matricule: "0001"     # nécessaire pour la détection intra-PDP

elasticsearch:
  url: "http://localhost:9200"

database:
  url: "postgres://postgres:postgres@localhost:5432/postgres"
  max_connections: 5

ppf:
  flux1_output_dir: "./output/flux1"
  flux1_profile: "Auto"   # Base, Full, Auto

routes:
  - id: "emission-tenant-acme"
    pipeline_mode: "Emission"
    source: { type: file, path: "./tenants/123456789/in" }
    destination: { type: ppf, path: "./tenants/123456789/out" }
    validate: true
    generate_cdar: true
```

---

## 2. Rejet à l'émission (UC2)

**Cas d'usage** : la facture déposée par le vendeur ne passe pas la validation
(Schematron BR-FR, BR-FR-10/11 annuaire, ou erreur sémantique). La PDP émettrice
génère un **CDV 213 Rejetée** avec un code motif précis (REJ_SEMAN, REJ_COH,
DEST_INC, ...) et le retourne au vendeur. La facture **n'est pas transmise**
au destinataire.

### Diagramme de séquence

```mermaid
sequenceDiagram
    autonumber
    participant SE as SE (Vendeur)
    participant PAE as PA-E (Ferrite émetteur)
    participant PPF as PPF
    participant Webhook as Webhook subscribers

    SE->>PAE: Dépose facture
    PAE->>PAE: ParseProcessor (OK)
    PAE->>PAE: ValidateProcessor / XmlValidateProcessor

    alt Erreur Schematron BR-FR
        PAE->>PAE: add_error("validate", "BR-FR-12 ...")
    else Vendeur inconnu annuaire
        PAE->>PAE: AnnuaireValidationProcessor<br/>add_error("annuaire-validation", "Vendeur inconnu")
    else Acheteur inconnu annuaire
        PAE->>PAE: AnnuaireValidationProcessor<br/>add_error("annuaire-validation", "Destinataire inconnu")
    end

    PAE->>PAE: CdarProcessor::emission<br/>has_errors() = true → CDV 213
    Note right of PAE: classify_error_reason → reason_code<br/>REJ_SEMAN / REJ_COH / DEST_INC

    PAE->>PPF: Flux 6 CDV 213 (FFE0614A)
    PAE->>SE: Retourne CDV 213<br/>(SFTP {siren}/out/cdv/ ou webhook)
    PAE->>Webhook: flow.ack.updated (Rejected → Error)

    Note over PAE: La facture n'est PAS transmise<br/>au destinataire
```

### Codes motifs typiques (mapping `classify_error_reason`)

| Step | Pattern message | Reason code |
|------|-----------------|-------------|
| `validate` | `BR-FR-12`, `BR-FR-...`, `schematron`, `rule` | `REJ_SEMAN` |
| `validate` | `xsd`, `schema` | `REJ_UNI` |
| `parsing` | `xml`, `parse` | `REJ_SEMAN` |
| `annuaire-validation` | `Vendeur inconnu` ou `Vendeur inactif` | `REJ_COH` |
| `annuaire-validation` | `Destinataire inconnu` ou `Destinataire inactif` | `DEST_INC` |
| `*` | `siret`, `siren` | `SIRET_ERR` |
| `*` | `tva`, `vat` | `TX_TVA_ERR` |
| `*` | `montant`, `total` | `MONTANTTOTAL_ERR` |

Voir [docs/cdar.md](cdar.md) pour la liste complète des 45 codes motifs.

### Recipients du CDV 213

| Mode | Sender | Issuer | Recipients |
|------|--------|--------|------------|
| **Émission** (PA-E) | PA-E | PA-E | SE + PPF |
| **Réception** (PA-R) | PA-R | PA-R | SE + BY (PAS de PPF) |

---

## 3. Réception classique (UC3)

**Cas d'usage** : Ferrite reçoit une facture d'une autre PDP via AFNOR Flow
Service (`POST /v1/flows`). Elle vérifie l'intégrité (SHA-256, taille),
parse, valide, et acquitte par CDV 202 Reçue. La facture est ensuite mise à
disposition de l'acheteur (CDV 203). **PAS de Flux 1 PPF en réception** (déjà
fait par la PA-E).

### Pipeline

```
HTTP POST /v1/flows → InboundFlow → ChannelConsumer → pipeline réception :
TraceProcessor::received → ReceptionProcessor::strict
  → IrrecevabiliteProcessor → DocumentTypeRouter → ParseProcessor
  → DuplicateCheckProcessor → ValidateProcessor → XmlValidateProcessor
  → TraceProcessor::validated → WebhookAckProcessor (In, "Ok")
  → AnnuaireValidationProcessor (Reception : vendeur uniquement)
  → CdarProcessor::reception (CDV 202)
  → FileEndpoint (livraison vers {siren}/out/)
  → TraceProcessor::distributed
```

### Diagramme de séquence

```mermaid
sequenceDiagram
    autonumber
    participant PAE as PA-E (PDP émettrice)
    participant PAR as PA-R (Ferrite récepteur)
    participant ES as Elasticsearch
    participant BY as BY (Acheteur)
    participant Webhook as Webhook subscribers

    PAE->>PAR: POST /v1/flows<br/>multipart {flowInfo, file}<br/>Authorization: Bearer ...

    PAR->>PAR: handle_receive_flow<br/>Vérifie SHA-256, taille (≤ max_flow_size_bytes)
    alt Taille dépassée
        PAR-->>PAE: 413 Payload Too Large
    else SHA-256 mismatch
        PAR-->>PAE: 400 Bad Request
    else OK
        PAR-->>PAE: 202 Accepted {flowId, submittedAt}
    end

    PAR->>Webhook: flow.received (Pending)
    PAR->>PAR: ParseProcessor + ValidateProcessor
    PAR->>PAR: AnnuaireValidationProcessor<br/>(vendeur uniquement en réception)
    PAR->>ES: TraceProcessor::validated<br/>(index pdp-traces-{SIREN})
    PAR->>Webhook: flow.ack.updated (Validated → Ok)

    PAR->>PAR: CdarProcessor::reception<br/>(CDV 202 « Reçue », Recipients = SE + BY)
    PAR->>PAE: Flux 6 CDV 202 (retour vers PA-E)
    Note over PAR: PAS de Flux 1 PPF<br/>(déjà fait par la PA-E)

    PAR->>BY: Met à disposition la facture<br/>{siren}/out/ ou notification webhook
    PAR->>Webhook: flow.ack.updated (Distributed → Ok)
```

### Différences PA-E vs PA-R

| Aspect | PA-E (émission) | PA-R (réception) |
|--------|-----------------|------------------|
| Flux 1 PPF | ✅ TOUJOURS | ❌ JAMAIS |
| CDV en succès | 200 Déposée | 202 Reçue |
| CDV en rejet | 213 Rejetée (SE + PPF) | 213 Rejetée (SE + BY, sans PPF) |
| Validation annuaire | Vendeur ET acheteur | Vendeur uniquement |
| Mode CdarProcessor | `Emission` | `Reception` |

---

## 4. Routage intra-PDP (UC4)

**Cas d'usage** : le vendeur et l'acheteur sont **tous deux sur Ferrite**.
Le routage détecte cette situation (matricule destinataire = matricule local)
et injecte directement la facture dans le pipeline réception via un canal
mpsc, **sans réseau ni SFTP**.

### Détection

`RoutingResolverProcessor::with_our_matricule(M)` compare le matricule résolu
de la PDP destinataire avec le matricule local. Si égal → `routing.destination
= "INTRA-PDP"`.

### Diagramme de séquence

```mermaid
sequenceDiagram
    autonumber
    participant SE as SE (Vendeur tenant ACME)
    participant E as Pipeline émission
    participant Channel as tokio mpsc<br/>intra_pdp_tx
    participant R as Pipeline réception
    participant BY as BY (Acheteur tenant XYZ<br/>même PDP)

    SE->>E: Dépose facture<br/>buyer = SIREN sur la même PDP
    E->>E: Validation + AnnuaireValidation
    E->>E: PpfFlux1Processor (Flux 1 vers PPF)
    E->>E: RoutingResolverProcessor<br/>resolution.matricule = our_matricule<br/>→ routing.destination = "INTRA-PDP"
    E->>E: CdarProcessor::emission<br/>(CDV 200 Déposée)
    E->>Channel: DynamicRoutingProducer<br/>tx.send(exchange)
    Note right of Channel: PAS de réseau,<br/>PAS de tar.gz SFTP

    Channel->>R: ChannelConsumer reçoit l'exchange
    R->>R: Pipeline réception complet<br/>(parse, valide, CDV 202)
    R->>BY: Met à disposition<br/>{xyz_siren}/out/
```

### Avantages

- **Latence quasi-nulle** : pas de SFTP, pas d'AFNOR Flow Service
- **Idempotent** : le canal mpsc avec backpressure garantit la livraison
- **Trace ES** : les deux pipelines indexent dans leurs SIREN respectifs

### Configuration

Activé automatiquement quand `pdp.matricule` est défini ET qu'au moins deux
tenants partagent la même PDP. Le canal est créé dans `cmd_start` :

```rust
let (intra_pdp_tx, intra_pdp_rx) = tokio::sync::mpsc::channel::<Exchange>(100);
// intra_pdp_tx → DynamicRoutingProducer.with_intra_pdp(tx)
// intra_pdp_rx → ChannelConsumer (pipeline réception "intra-pdp-reception")
```

---

## 5. CDV 210/212 avec relais PPF (Flux 6)

**Cas d'usage** : l'acheteur (BY) émet un CDV 210 (Refusée) ou 212 (Encaissée)
à destination du vendeur (SE). Conformément aux Acteurs CDV V1.2, ces deux
statuts doivent **également être relayés au PPF** via le code interface
**FFE0654A** (Statuts obligatoires) — ils impactent la TVA de l'État.

### Pipeline

Le `CdvPpfRelayProcessor` est ajouté dans `add_common_processors`, juste
après le `DocumentTypeRouter` qui a parsé le CDV entrant et renseigné les
propriétés `cdv.*`.

```
PpfReturnConsumer (CDV reçu via SFTP) ou réception AFNOR
  → DocumentTypeRouter (parse CDV, set cdv.status_code)
  → CdvPpfRelayProcessor : si status_code ∈ {210, 212} → relais PPF
  → ...
```

### Diagramme de séquence

```mermaid
sequenceDiagram
    autonumber
    participant BY as BY (Acheteur)
    participant PAR as PA-R (PDP acheteur)
    participant PAE as PA-E (PDP vendeur)
    participant Ferrite as Ferrite
    participant PPF as PPF (FFE0654A)
    participant SE as SE (Vendeur)

    BY->>PAR: Émet CDV 210 (Refusée)<br/>ou 212 (Encaissée)
    PAR->>PAE: Flux 6 CDV (FFE0614A)
    PAE->>Ferrite: Reçoit le CDV<br/>(SFTP retrait ou AFNOR)

    Ferrite->>Ferrite: DocumentTypeRouter<br/>parse → cdv.status_code = 210/212

    Note over Ferrite: CdvPpfRelayProcessor détecte 210/212

    Ferrite->>PPF: Flux 6 (FFE0654A)<br/>tar.gz CDV statuts obligatoires
    Note right of PPF: Statut TVA :<br/>refus → annulation TVA<br/>paiement → liquidation TVA

    Ferrite->>SE: Notification CDV<br/>(SFTP {se_siren}/out/ ou webhook)
```

### CDV NON relayés au PPF

Tous les autres statuts (200, 202, 204, 205, 207, 211, 213, 220, 224-228, 501)
restent **en interne** entre PA-E et PA-R, sans relais PPF. C'est volontaire :
seuls 210 et 212 ont un impact fiscal direct.

### Tests

Les 10 tests unitaires de `CdvPpfRelayProcessor` (dans `pdp-cdar/src/ppf_relay.rs`)
couvrent :
- Relais 210 / 212 → tar.gz FFE0654A déposé
- Skip pour tous les autres codes (200, 202, 213, 501, ...)
- Comportement non bloquant en cas d'erreur PPF (warning + continue)

---

## Annexe — État final d'un Exchange

À la fin d'un workflow, l'`Exchange` porte ces propriétés clés (utilisables
par les processors aval ou les webhooks) :

| Propriété | Valeur typique | Source |
|-----------|----------------|--------|
| `tracking_id` | `TRACK-20260502-001` | Header HTTP `Request-Id` ou `flowInfo.trackingId` |
| `flow.syntax` | `UBL` / `CII` / `FacturX` / `CDAR` | `ParseProcessor` ou `PpfReturnConsumer` |
| `cdv.status_code` | `200`, `202`, `213`, `221`, `501`, ... | `CdarProcessor` ou `DocumentTypeRouter` |
| `cdv.document_id` | UUID du CDV | `CdarProcessor` |
| `routing.destination` | `PPF-SE` / `INTRA-PDP` / `PDP-1234` | `RoutingResolverProcessor` |
| `routing.pdp_matricule` | `0000` (PPF) ou matricule PA-R | idem |
| `ppf.code_interface` | `FFE0111A`, `FFE0654A`, `FFE1435A`, ... | `PpfReturnConsumer` |
| `cdv.annuaire.status_code` | `400` / `401` (réponse F13) | `DocumentTypeRouter` |
| `annuaire.import.ok` | `"true"` (F14 ingéré) | `AnnuaireImportProcessor` |
| `webhook.last_ack_status` | `Ok` / `Error` | `WebhookAckProcessor` |

## Voir aussi

- [docs/cdar.md](cdar.md) — Détails CDV (codes, acteurs, motifs)
- [docs/http-api.md](http-api.md) — API REST + curl
- [docs/annuaire.md](annuaire.md) — Annuaire PPF + F13/F14
- [docs/ereporting.md](ereporting.md) — E-reporting Flux 10
- [docs/ppf-afnor.md](ppf-afnor.md) — Codes interface PPF, SFTP
- [rapport-conformite-pdp.md](../rapport-conformite-pdp.md) — Conformité 4 specs
