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

Génère un CDV après traitement d'une facture. Paramétré par `CdarMode` :

- **`CdarProcessor::emission()`** (PDP émettrice) :
  - **200 Déposée** si la facture est valide
  - **213 Rejetée** si la facture a des erreurs
- **`CdarProcessor::reception()`** (PDP réceptrice) :
  - **202 Reçue** si la facture est valide
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

## Codes motifs CDV

Référence : XP Z12-012 Annexe A V1.2, onglet "Tableau des motifs de STATUTS".

### Codes IRR — Irrecevabilité (CDV 501)

Utilisés par `IrrecevabiliteProcessor` quand les contrôles de réception échouent.

| Code | Libellé | Contrôle | Mapping |
|------|---------|----------|---------|
| `IRR_VIDE_F` | Contrôle de non vide sur les fichiers du flux | Fichier vide | REC-01 |
| `IRR_TYPE_F` | Contrôle de type et extension des fichiers du flux | Extension invalide (.csv, .txt...) | REC-02 |
| `IRR_SYNTAX` | Contrôle syntaxique des fichiers du flux | XML non parseable | Fallback |
| `IRR_TAILLE_F` | Contrôle de taille max des fichiers du flux | Fichier > 100 Mo | BR-FR-19 |
| `IRR_NOM_PJ` | Contrôle du nom des PJ (caractères spéciaux) | Nom invalide ou absent | REC-03/04 |
| `IRR_TAILLE_PJ` | Contrôle de taille des PJ | PJ trop volumineuse | (non implémenté) |
| `IRR_VID_PJ` | Contrôle de PJ non vide | PJ vide | (non implémenté) |
| `IRR_EXT_DOC` | Contrôle de l'extension des PJ | Extension PJ invalide | (non implémenté) |
| `IRR_ANTIVIRUS` | Contrôle anti-virus | Fichier infecté | (non implémenté) |

### Codes REJ — Rejet technique (CDV 213)

Utilisés par `CdarProcessor` quand la validation échoue. Le code est déterminé
par `classify_error_reason()` en analysant le message d'erreur.

| Code | Libellé | Pattern détecté |
|------|---------|-----------------|
| `REJ_SEMAN` | Rejet pour erreur sémantique | "syntax", "xml", "parse", "schematron", "br-", "rule", step "validate" |
| `REJ_UNI` | Rejet sur contrôle unicité | "xsd", "schema" |
| `REJ_COH` | Rejet sur contrôle cohérence de données | (disponible, non mappé automatiquement) |
| `REJ_ADR` | Rejet sur contrôle d'adressage | (disponible, non mappé automatiquement) |
| `REJ_CONT_B2G` | Rejet sur contrôles métier B2G | (disponible, non mappé automatiquement) |
| `REJ_REF_PJ` | Rejet sur référence de PJ | (disponible, non mappé automatiquement) |
| `REJ_ASS_PJ` | Rejet sur erreur d'association de la PJ | (disponible, non mappé automatiquement) |

### Codes métier — Refus, litige, etc. (CDV 210, 207, 206, 208)

Utilisés par l'acheteur ou le vendeur dans les CDV de phase Traitement (TypeCode 23).
Classés par `classify_error_reason()` quand le message contient les mots-clés.

| Code | Libellé | Pattern détecté |
|------|---------|-----------------|
| `DOUBLON` | Facture en doublon | "doublon", "duplicate" |
| `SIRET_ERR` | SIRET erroné ou absent | "siret", "siren" |
| `TX_TVA_ERR` | Taux de TVA erroné | "tva", "vat" |
| `MONTANTTOTAL_ERR` | Montant total erroné | "montant", "total", "amount" |
| `CALCUL_ERR` | Erreur de calcul | "calcul", "calculation" |
| `ADR_ERR` | Adresse de facturation erronée | "adresse", "address" |
| `DEST_ERR` | Erreur de destinataire | "destinataire", "recipient" |
| `DEST_INC` | Destinataire inconnu | (dans l'enum, non mappé auto) |
| `NON_CONFORME` | Mention légale manquante | Fallback (aucun pattern reconnu) |
| `COORD_BANC_ERR` | Erreur de coordonnées bancaires | (dans l'enum) |
| `TRANSAC_INC` | Transaction inconnue | (dans l'enum) |
| `EMMET_INC` | Émetteur inconnu | (dans l'enum) |
| `CONTRAT_TERM` | Contrat terminé | (dans l'enum) |
| `DOUBLE_FACT` | Double facture | (dans l'enum) |
| `CMD_ERR` | N° de commande incorrect | (dans l'enum) |
| `CODE_ROUTAGE_ERR` | Code routage absent ou erroné | (dans l'enum) |
| `REF_CT_ABSENT` | Référence contractuelle manquante | (dans l'enum) |
| `REF_ERR` | Référence incorrecte | (dans l'enum) |
| `PU_ERR` | Prix unitaires incorrects | (dans l'enum) |
| `REM_ERR` | Remise erronée | (dans l'enum) |
| `QTE_ERR` | Quantité facturée incorrecte | (dans l'enum) |
| `ART_ERR` | Article facturé incorrect | (dans l'enum) |
| `MODPAI_ERR` | Modalités de paiement incorrectes | (dans l'enum) |
| `QUALITE_ERR` | Qualité d'article incorrecte | (dans l'enum) |
| `LIVR_INCOMP` | Problème de livraison | (dans l'enum) |
| `ROUTAGE_ERR` | Erreur de routage | (dans l'enum) |
| `JUSTIF_ABS` | Justificatif absent | (dans l'enum) |
| `NON_TRANSMISE` | Destinataire non connecté | (dans l'enum) |
| `AUTRE` | Autre | (dans l'enum) |

## Pipelines émission et réception

### Pipeline Émission (PDP émettrice)

```
1. Réception          → ReceptionProcessor (taille, extension, nom, doublons)
2. Irrecevabilité     → IrrecevabiliteProcessor (CDAR 501 si échec, codes IRR_*)
3. Routage            → DocumentTypeRouter (facture vs CDAR vs e-reporting)
   ├─ Si CDAR         → parse CDV, set cdv.*, skip étapes 4-7
   └─ Si Facture      → continuer
4. Parsing            → ParseProcessor (UBL/CII/Factur-X → InvoiceData)
5. Validation         → ValidateProcessor + XmlValidateProcessor (EN16931, BR-FR)
6. Flux 1 PPF         → PpfFlux1Processor (TOUJOURS — données réglementaires)
7. Transformation     → TransformProcessor (UBL ↔ CII, Factur-X)
8. Génération CDV     → CdarProcessor::emission() (200 Déposée ou 213 Rejetée, codes REJ_*)
9. Routage            → RoutingResolverProcessor (Annuaire PPF → PPF / PDP / intra-PDP)
10. Distribution      → DynamicRoutingProducer (SFTP PPF, AFNOR Flow, ou intra-PDP)
```

### Pipeline Réception (PDP réceptrice)

```
1. Réception          → ReceptionProcessor (taille, extension, nom, doublons)
2. Irrecevabilité     → IrrecevabiliteProcessor (CDAR 501 si échec, codes IRR_*)
3. Routage            → DocumentTypeRouter (facture vs CDAR)
   ├─ Si CDAR         → CdvReceptionProcessor, set cdv.*, skip suite
   └─ Si Facture      → continuer
4. Parsing            → ParseProcessor (UBL/CII/Factur-X → InvoiceData)
5. Validation         → ValidateProcessor + XmlValidateProcessor
   PAS de Flux 1 PPF  (la PDP émettrice l'a déjà envoyé)
6. Transformation     → TransformProcessor (optionnel)
7. Génération CDV     → CdarProcessor::reception() (202 Reçue ou 213 Rejetée, codes REJ_*)
8. Livraison          → FileEndpoint (répertoire acheteur)
```

## Tests

147 tests couvrant :

- **model** (12) : statuts, rôles, codes action, parties, sérialisation
- **generator** (15) : génération XML pour tous les statuts (200, 202, 213, 501)
- **parser** (18) : parsing XML, fixtures officielles UC1-UC4
- **processor** (110) : CdarProcessor (émission/réception), CdvReceptionProcessor,
  IrrecevabiliteProcessor, DocumentTypeRouter, classify_error_reason (14 tests),
  map_reception_to_irrecevabilite
- **pipeline_error_tests** (23) : tests d'intégration pipeline complet avec fichiers
  invalides (vide, non-XML, XML mal formé, PDF sans XML, validation échouée, BR-FR)
  en mode émission et réception, vérification des codes motifs et messages
- **lifecycle_integration** (28) : CDV 200/202/213/501, émission vs réception,
  conformité AFNOR (recipients, issuer, status codes)
