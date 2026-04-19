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
| 221 | Erreur de routage | Error |
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

## Acteurs CDV (qui émet quoi)

Référence : XP Z12-012 Annexe A V1.2, onglet "Acteurs CDV".

PA-E = PDP émettrice, PA-R = PDP réceptrice, WK = Plateforme,
C1 = Émetteur (vendeur), C4 = Destinataire (acheteur).

### Statuts émis par la PDP

| Code | Statut | Émetteur CDV | Issuer | Sender | Recipients |
|------|--------|-------------|--------|--------|------------|
| 200 | Déposée | PA-E | WK | WK | Émetteur (C1) |
| 201 | Émise | PA-E | WK | WK | Émetteur (C1) |
| 202 | Reçue | PA-R | WK | WK | Émetteur (C1) + Destinataire (C4) |
| 203 | Mise à disposition | PA-R | WK | WK | Émetteur (C1) + Destinataire (C4) |
| 213 | Rejetée (réception) | PA-R | WK | WK | Émetteur (C1) + Destinataire (C4) |
| 213 | Rejetée (émission) | PA-E | PA-E | WK | Émetteur (C1) |
| 221 | Erreur routage | PA-R | PA-R | PA-R | PA-E |
| 501 | Irrecevable | PA-R | PA-R | PA-R | PA-E |

### Statuts émis par l'acheteur/vendeur (relayés par la PDP)

| Code | Statut | Émetteur CDV | Issuer | Recipients |
|------|--------|-------------|--------|------------|
| 204 | Prise en charge | Destinataire | Destinataire | Émetteur (C1) |
| 205 | Approuvée | Destinataire | Destinataire | Émetteur (C1) |
| 206 | Approuvée partiellement | Destinataire | Destinataire | Émetteur (C1) |
| 207 | En litige | Destinataire | Destinataire | Émetteur (C1) |
| 208 | Suspendue | Destinataire | Destinataire | Émetteur (C1) |
| 209 | Complétée | Émetteur | Émetteur | Destinataire (C4) |
| 210 | Refusée | Destinataire | Destinataire | Émetteur (C1) |
| 211 | Paiement transmis | Acheteur | Acheteur | Vendeur |
| 212 | Encaissée | Vendeur | Vendeur | Acheteur |
| 214 | Visée | Agent vendeur/acheteur | Agent | Émetteur + Destinataire |
| 220 | Annulée | Émetteur | Émetteur | Destinataire (C4) |

## Codes motifs CDV

Référence : XP Z12-012 Annexe A V1.2, onglet "Tableau des motifs de STATUTS".

Les codes motifs ont des applicabilités différentes selon le contexte (émission,
réception, PPF). Un code applicable en "REJETÉE Émission" n'est pas forcément
applicable en "REJETÉE Réception" ou en "REFUSÉE".

### Codes IRR — Irrecevabilité (CDV 501)

Utilisés par `IrrecevabiliteProcessor` quand les contrôles de réception échouent.
Applicables en émission ET en réception (même contrôles dans les deux pipelines).

| Code | Libellé | Mapping | Implémenté |
|------|---------|---------|------------|
| `IRR_VIDE_F` | Fichier du flux vide | REC-01 | oui |
| `IRR_TYPE_F` | Type/extension du fichier invalide | REC-02 | oui |
| `IRR_SYNTAX` | Fichier syntaxiquement invalide | Fallback | oui |
| `IRR_TAILLE_F` | Fichier > 100 Mo | BR-FR-19 | oui |
| `IRR_NOM_PJ` | Nom de fichier invalide (caractères, absent) | REC-03/04 | oui |
| `IRR_TAILLE_PJ` | Pièce jointe trop volumineuse | — | non |
| `IRR_VID_PJ` | Pièce jointe vide | — | non |
| `IRR_EXT_DOC` | Extension de pièce jointe invalide | — | non |
| `IRR_ANTIVIRUS` | Fichier infecté | — | non |

### Codes REJ — Rejet technique (CDV 213)

Contrôles automatiques effectués par la PDP (pas par l'acheteur).
Applicables en **REJETÉE Émission** ET **REJETÉE Réception**.
Provenance : contrôles PDP (pas B2G).

| Code | Libellé | Pattern `classify_error_reason()` | Émission | Réception |
|------|---------|-----------------------------------|----------|-----------|
| `REJ_SEMAN` | Erreur sémantique | "syntax", "xml", "parse", "schematron", "br-", step "validate" | X | X |
| `REJ_UNI` | Contrôle unicité | "xsd", "schema" | X | X |
| `REJ_COH` | Cohérence de données | (dans l'enum) | X | X |
| `REJ_ADR` | Contrôle d'adressage | (dans l'enum) | X | X |
| `REJ_CONT_B2G` | Contrôles métier B2G | (dans l'enum) | X | X |
| `REJ_REF_PJ` | Référence de PJ | (dans l'enum) | X | X |
| `REJ_ASS_PJ` | Association de la PJ | (dans l'enum) | X | X |

### Codes applicables en REJETÉE Émission + Réception + REFUSÉE

Ces codes peuvent être générés par la PDP (rejet) ou par l'acheteur (refus).

| Code | Libellé | Pattern | Rej. Émi | Rej. Réc | Refusée |
|------|---------|---------|----------|----------|---------|
| `DOUBLON` | Facture en doublon | "doublon", "duplicate" | X | X | X |
| `MONTANTTOTAL_ERR` | Montant total erroné | "montant", "total", "amount" | X | X | X |
| `CALCUL_ERR` | Erreur de calcul | "calcul", "calculation" | X | X | X |
| `ADR_ERR` | Adresse de facturation erronée | "adresse", "address" | X | X | X |

### Codes applicables uniquement en REJETÉE Émission

| Code | Libellé | Pattern | Commentaire |
|------|---------|---------|-------------|
| `DEST_INC` | Destinataire inconnu | (dans l'enum) | Annuaire PPF : SIREN introuvable |

### Codes métier — Refusée / En litige (CDV 210, 207, 206, 208)

Utilisés par l'**acheteur** dans les CDV de phase Traitement (TypeCode 23).
La PDP les reçoit mais ne les génère pas elle-même.
Provenance : B2G (acheteur public) ou IMR/CDAR (intermédiaire).

| Code | Libellé | Refusée | En litige | Approv. part. | Suspendue |
|------|---------|---------|-----------|---------------|-----------|
| `TX_TVA_ERR` | Taux de TVA erroné | X | | | |
| `NON_CONFORME` | Mention légale manquante | X | X | | |
| `DEST_ERR` | Erreur de destinataire | X | X | | |
| `TRANSAC_INC` | Transaction inconnue | X | X | | |
| `EMMET_INC` | Émetteur inconnu | X | X | | |
| `CONTRAT_TERM` | Contrat terminé | X | X | | |
| `DOUBLE_FACT` | Double facture | X | X | | |
| `CMD_ERR` | Commande incorrecte | X | X | X | X |
| `COORD_BANC_ERR` | Coordonnées bancaires | X | | X | |
| `SIRET_ERR` | SIRET erroné | | X | X | X |
| `CODE_ROUTAGE_ERR` | Code routage erroné | | X | X | X |
| `REF_CT_ABSENT` | Référence contractuelle absente | X | X | X | X |
| `REF_ERR` | Référence incorrecte | | X | X | X |
| `PU_ERR` | Prix unitaires incorrects | | X | X | |
| `REM_ERR` | Remise erronée | | X | X | |
| `QTE_ERR` | Quantité incorrecte | | X | X | |
| `ART_ERR` | Article incorrect | | X | X | |
| `MODPAI_ERR` | Modalités paiement incorrectes | | X | X | |
| `QUALITE_ERR` | Qualité incorrecte | | X | X | |
| `LIVR_INCOMP` | Problème de livraison | | X | X | |
| `JUSTIF_ABS` | Justificatif absent | | | | X |
| `AUTRE` | Autre motif | X | X | | |

### Codes spéciaux

| Code | Libellé | Statut applicable | Commentaire |
|------|---------|-------------------|-------------|
| `NON_TRANSMISE` | Destinataire non connecté | DÉPOSÉE (200) | Statut spécial : facture déposée mais pas transmissible |
| `ROUTAGE_ERR` | Erreur de routage | ERREUR_ROUTAGE (221) | Erreur technique de routage |

## Pipelines émission et réception

### Pipeline Émission (PDP émettrice)

```
 1. Réception          → ReceptionProcessor (taille, extension, nom, doublons)
 2. Irrecevabilité     → IrrecevabiliteProcessor (CDAR 501 si échec, codes IRR_*)
 3. Routage            → DocumentTypeRouter (facture vs CDAR vs e-reporting)
    ├─ Si CDAR         → parse CDV, set cdv.*, skip étapes 4-9
    └─ Si Facture      → continuer
 4. Parsing            → ParseProcessor (UBL/CII/Factur-X → InvoiceData)
 5. Doublons           → DuplicateCheckProcessor (BR-FR-12/13)
 6. Validation         → ValidateProcessor + XmlValidateProcessor (EN16931, BR-FR, Schematron)
 7. Annuaire           → AnnuaireValidationProcessor(Emission) (BR-FR-10 vendeur + BR-FR-11 acheteur)
                          Vendeur absent → REJ_COH, Acheteur absent → DEST_INC
 8. Flux 1 PPF         → PpfFlux1Processor (TOUJOURS — données réglementaires)
 9. Transformation     → TransformProcessor (UBL ↔ CII, Factur-X)
10. Génération CDV     → CdarProcessor::emission()
                          Succès → 200 Déposée (Recipients: SE + PPF)
                          Erreur → 213 Rejetée à l'émission (Recipients: SE seul)
11. Routage            → RoutingResolverProcessor (Annuaire PPF → PPF / PDP / intra-PDP)
12. Distribution       → DynamicRoutingProducer (SFTP PPF, AFNOR Flow, ou intra-PDP)
```

### Pipeline Réception (PDP réceptrice)

```
 1. Réception          → ReceptionProcessor (taille, extension, nom, doublons)
 2. Irrecevabilité     → IrrecevabiliteProcessor (CDAR 501, Sender=PA-R)
 3. Routage            → DocumentTypeRouter (facture vs CDAR)
    ├─ Si CDAR         → CdvReceptionProcessor, set cdv.*, skip suite
    └─ Si Facture      → continuer
 4. Parsing            → ParseProcessor (UBL/CII/Factur-X → InvoiceData)
 5. Doublons           → DuplicateCheckProcessor
 6. Validation         → ValidateProcessor + XmlValidateProcessor (EN16931, BR-FR, Schematron)
 7. Annuaire           → AnnuaireValidationProcessor(Reception) (BR-FR-10 vendeur uniquement)
                          Vendeur absent → REJ_COH (PAS de check acheteur en réception)
    PAS de Flux 1 PPF  (la PDP émettrice l'a déjà envoyé)
 8. Transformation     → TransformProcessor (optionnel)
 9. Génération CDV     → CdarProcessor::reception()
                          Succès → 202 Reçue (Recipients: SE + BY, pas de PPF)
                          Erreur → 213 Rejetée en réception (Recipients: SE + BY)
10. Livraison          → FileEndpoint (répertoire acheteur)
```

### Ordre des contrôles et codes d'erreur

| Étape | Contrôle | Code erreur | CDV |
|-------|----------|-------------|-----|
| 1. Réception | Fichier vide | IRR_VIDE_F | 501 |
| 1. Réception | Extension invalide | IRR_TYPE_F | 501 |
| 1. Réception | Fichier > 100 Mo | IRR_TAILLE_F | 501 |
| 1. Réception | Nom fichier invalide | IRR_NOM_PJ | 501 |
| 2. Irrecevabilité | XML non parseable | IRR_SYNTAX | 501 |
| 5. Doublons | Facture en doublon | DOUBLON | 213 |
| 6. Validation | Erreur sémantique | REJ_SEMAN | 213 |
| 6. Validation | Erreur XSD | REJ_UNI | 213 |
| 6. Validation | Montant erroné | MONTANTTOTAL_ERR | 213 |
| 6. Validation | Erreur de calcul | CALCUL_ERR | 213 |
| 6. Validation | Adresse erronée | ADR_ERR | 213 |
| 7. Annuaire | Vendeur absent (BR-FR-10) | REJ_COH | 213 |
| 7. Annuaire | Acheteur absent (BR-FR-11) | DEST_INC | 213 |

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
