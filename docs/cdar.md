# CDV / CDAR — Comptes-rendus De Vie

Module `pdp-cdar` : génération, parsing et routage des statuts de cycle de vie des factures,
conforme au format UN/CEFACT CrossDomainAcknowledgementAndResponse (CDAR) D23B.

## Architecture

```
                    ┌─────────────────────────────────────────────────────────┐
                    │                     Pipeline PDP                        │
                    │                                                         │
 Fichier entrant ──▶│  Réception → DocumentTypeRouter → Parsing → Validation  │
                    │                    │                                     │
                    │              Facture ? ──▶ traitement standard           │
                    │              CDAR ?   ──▶ parse CDV, set propriétés,     │
                    │                          skip Parse/Validate/Transform   │
                    └─────────────────────────────────────────────────────────┘
```

## Processors

### `DocumentTypeRouter`

Détecte le type de document entrant (facture, CDAR, e-reporting) et positionne
le header `document.type` sur l'exchange. Si c'est un CDAR, il est immédiatement
parsé et les propriétés `cdv.*` sont renseignées.

Les processors suivants (`ParseProcessor`, `ValidateProcessor`, `TransformProcessor`)
skipperont automatiquement si `document.type = "CDAR"`.

### `CdarProcessor`

Génère un CDV après traitement d'une facture :
- **200 Déposée** si la facture est valide
- **213 Rejetée** si la facture a des erreurs

### `IrrecevabiliteProcessor`

Génère un CDAR 501 (Irrecevable) si les contrôles de réception échouent
(fichier vide, trop gros, extension invalide, nom invalide, doublon).

### `CdvReceptionProcessor`

Parse un CDV entrant et met à jour le statut de l'exchange. Utilisé
indépendamment du `DocumentTypeRouter` pour des cas spécifiques.

## Sources de CDAR entrants

Les CDAR peuvent arriver de 4 sources différentes, identifiées par la
propriété `cdv.source` :

| Source | Valeur `cdv.source` | Comment |
|--------|-------------------|---------|
| **Client (émission)** | `client` | Le vendeur nous envoie un statut (ex: 212 Encaissée) |
| **Client (réception)** | `client` | L'acheteur nous envoie un statut (ex: 204 Prise en charge, 210 Refusée) |
| **Autre PDP (PEPPOL)** | `peppol` | CDV reçu via AS4 (header `source.protocol` = `peppol-as4`) |
| **Autre PDP (AFNOR)** | `afnor` | CDV reçu via Flow Service (header `source.protocol` = `afnor-flow`) |
| **PPF** | `ppf` | CDV reçu du PPF (nom de fichier `FFE06*` ou `CFE*`) |

## Propriétés de l'exchange après routage CDAR

| Propriété | Description |
|-----------|-------------|
| `cdv.received` | `"true"` si un CDV a été parsé |
| `cdv.document_id` | Identifiant du CDV |
| `cdv.type_code` | `"305"` (transmission) ou `"23"` (traitement) |
| `cdv.status_code` | Code statut (200, 201, 202, 204, 205, 207, 210, 212, 213…) |
| `cdv.invoice_id` | Numéro de la facture référencée |
| `cdv.process_condition` | Condition de traitement |
| `cdv.guideline_id` | Guideline ID du CDV |
| `cdv.source` | Source du CDAR (`client`, `peppol`, `afnor`, `ppf`) |
| `cdv.sender.id` | Identifiant de l'émetteur du CDV |
| `cdv.recipient.N.id` | Identifiant du destinataire N |

## Statuts de cycle de vie

### Phase Transmission (TypeCode 305)

| Code | Statut | FlowStatus |
|------|--------|------------|
| 200 | Déposée | Distributed |
| 201 | Émise | Distributed |
| 202 | Reçue | Acknowledged |
| 203 | Mise à disposition | Distributed |
| 213 | Rejetée | Rejected |
| 300 | Transmise PPF | Distributed |
| 301 | Transmise PDP | Distributed |
| 400 | Transmise destinataire | Distributed |
| 501 | Irrecevable | Rejected |

### Phase Traitement (TypeCode 23)

| Code | Statut | FlowStatus |
|------|--------|------------|
| 204 | Prise en charge | Acknowledged |
| 205 | Approuvée | Acknowledged |
| 206 | Approuvée partiellement | Acknowledged |
| 207 | En litige | WaitingAck |
| 208 | Suspendue | WaitingAck |
| 209 | Service fait | Acknowledged |
| 210 | Refusée | Rejected |
| 211 | Paiement transmis | Acknowledged |
| 212 | Encaissée | Acknowledged |
| 214 | Visée | Acknowledged |
| 220 | Annulée | Cancelled |

## Pipeline complet

```
1. Réception          → ReceptionProcessor (taille, extension, nom, doublons)
2. Irrecevabilité     → IrrecevabiliteProcessor (CDAR 501 si échec réception)
3. Routage            → DocumentTypeRouter (facture vs CDAR vs e-reporting)
   ├─ Si CDAR         → parse CDV, set cdv.*, skip étapes 4-6
   └─ Si Facture      → continuer
4. Parsing            → ParseProcessor (UBL/CII/Factur-X → InvoiceData)
5. Validation         → ValidateProcessor + XmlValidateProcessor
6. Transformation     → TransformProcessor (UBL ↔ CII, Factur-X)
7. Génération CDV     → CdarProcessor (200 Déposée ou 213 Rejetée)
8. Distribution       → destination (fichier, SFTP, PEPPOL, AFNOR)
```

## Tests

108 tests (106 unit + 2 doc-tests) couvrant :

- **model** (12) : statuts, rôles, codes action, parties, sérialisation
- **generator** (15) : génération XML pour tous les statuts
- **parser** (18) : parsing XML, fixtures officielles UC1-UC4
- **processor** (63) : CdarProcessor, CdvReceptionProcessor, IrrecevabiliteProcessor,
  DocumentTypeRouter (détection, sources, routage, skip)
