# Flux 1 PPF — Données réglementaires

## Vue d'ensemble

Le **Flux 1** est le format de transmission des données réglementaires de facturation à la **Plateforme Publique de Facturation (PPF)**. Il existe en deux profils :

- **Base** : version allégée sans lignes de facture, parties simplifiées, totaux réduits
- **Full** : version complète avec lignes, prix, remises/majorations, livraison

Le profil est configurable via `ppf.flux1_profile` (ou `$PDP_FLUX1_PROFILE`) :

| Valeur | Comportement |
|--------|-------------|
| **`auto`** (défaut) | Lignes présentes → Full, sinon Base |
| **`base`** | Toujours Base (sans lignes) |
| **`full`** | Toujours Full (fallback Base si pas de lignes dans la source) |

> Le profil Full sera obligatoire à terme. Il est indépendant du profil sémantique (EN16931 / EXTENDED) de la facture source.

## Architecture

```text
Facture CII ──XSLT──▶ Flux 1 Base/Full CII ──▶ output/flux1/{profil}_{num}.xml
Facture UBL ──XSLT──▶ Flux 1 Base/Full UBL ──▶ output/flux1/{profil}_{num}.xml
                                                          │
                                                    tar.gz + SFTP → PPF
```

## Différences Base vs Full

### Éléments communs (Base et Full)

| Élément | CII | UBL |
|---------|-----|-----|
| BT-24 (profil) | `#Base` ou `#Full` | `#Base` ou `#Full` |
| En-tête (ID, date, type, devise) | ✓ | ✓ |
| Parties vendeur/acheteur (allégées) | ✓ | ✓ |
| Représentant fiscal | ✓ | ✓ |
| Livraison (date seulement) | ✓ | ✓ |
| TVA (TaxTotal + ventilation) | ✓ | ✓ |
| Montant HT (TaxExclusiveAmount) | ✓ | ✓ |
| Période de facturation | ✓ | ✓ |
| Référence de facturation | ✓ | ✓ |

### Éléments ajoutés en Full

| Élément | CII | UBL |
|---------|-----|-----|
| **Lignes de facture** | IncludedSupplyChainTradeLineItem | InvoiceLine / CreditNoteLine |
| Quantité facturée (par ligne) | BilledQuantity | InvoicedQuantity / CreditedQuantity |
| Nom article (par ligne) | SpecifiedTradeProduct/Name | Item/Name |
| Prix unitaire (par ligne) | NetPriceProductTradePrice | Price/PriceAmount |
| **Remises/majorations document** | SpecifiedTradeAllowanceCharge | AllowanceCharge |
| **Remises/majorations ligne** | SpecifiedLineTradeAllowanceCharge | AllowanceCharge (dans ligne) |
| **Livraison avec adresse** | ShipToTradeParty/PostalTradeAddress | Delivery/DeliveryLocation/Address |

### Éléments exclus (ni Base ni Full)

| Élément | Raison |
|---------|--------|
| PaymentMeans / PaymentTerms | Non requis par l'administration |
| BuyerReference / OrderReference | Non requis |
| AdditionalDocumentReference | Non requis |
| ProjectReference / ContractDocumentReference | Non requis |
| PayeeParty | Non requis |
| PrepaidPayment | Non requis |
| UBLVersionID | Non requis |

### Éléments filtrés dans les sous-structures Full

Les XSD F1Full sont plus restrictifs que les XSD EN16931/EXTENDED :

| Sous-structure | Éléments exclus |
|----------------|----------------|
| **Party** | PartyName (pas de nom commercial) |
| **AllowanceCharge** | AllowanceChargeReasonCode, AllowanceChargeReason, MultiplierFactorNumeric |
| **LegalMonetaryTotal** | LineExtensionAmount, TaxInclusiveAmount, AllowanceTotalAmount, ChargeTotalAmount, PayableAmount |
| **InvoiceLine/CreditNoteLine** | ID, LineExtensionAmount |
| **Item** | ClassifiedTaxCategory, Description, et tous identifiants article |
| **Price** | BaseQuantity |

## Transformations XSLT

### CII

| Profil | Fichier XSLT |
|--------|-------------|
| Base | `specs/xslt/convert/CII-to-F1Base-CII.xslt` |
| Full | `specs/xslt/convert/CII-to-F1Full-CII.xslt` |

### UBL

| Profil | Fichier XSLT |
|--------|-------------|
| Base | `specs/xslt/convert/UBL-to-F1Base-UBL.xslt` |
| Full | `specs/xslt/convert/UBL-to-F1Full-UBL.xslt` |

Chaque XSLT construit explicitement le XML de sortie (pas d'identity transform) pour garantir la conformité stricte aux XSD F1.

## Processor

### `PpfFlux1Processor`

**Crate** : `pdp-transform`  
**Module** : `ppf_flux1`

Le processor :
1. Détecte le profil selon la présence de lignes (`lines.is_empty()` → Base, sinon Full)
2. Sélectionne la transformation XSLT correspondante (format × profil)
3. Applique la transformation XSLT
4. Nomme le fichier selon la convention PPF : `{profil}_{invoice_number}.xml`
5. Dépose le fichier dans le répertoire configuré
6. Positionne les propriétés sur l'exchange :
   - `ppf.flux1.path` : chemin absolu du fichier généré
   - `ppf.flux1.filename` : nom du fichier
   - `ppf.flux1.profile` : `Base` ou `Full`
   - `ppf.flux1.code_interface` : `FFE0111A` (UBL) ou `FFE0112A` (CII)

### Codes interface PPF

| Format source | Code interface |
|---------------|---------------|
| UBL | `FFE0111A` |
| CII / Factur-X | `FFE0112A` |

### Position dans le pipeline

```text
ReceptionProcessor → IrrecevabiliteProcessor → DocumentTypeRouter
  → ParseProcessor → ValidateProcessor → XmlValidateProcessor
  → PpfFlux1Processor ← (ici, après validation)
  → TransformProcessor → CdarProcessor → Destination
```

Le processor skip automatiquement les documents CDAR (`document.type = "CDAR"`).

## Configuration

```yaml
ppf:
  environment: dev
  code_interface: FFE0112A
  code_application_piste: AAA123
  flux1_output_dir: ./output/flux1    # défaut, ou $PDP_FLUX1_OUTPUT_DIR
  flux1_profile: auto                 # auto | base | full (défaut: auto, ou $PDP_FLUX1_PROFILE)
  auth:
    token_url: https://oauth.piste.gouv.fr/api/oauth/token
    client_id: $PISTE_CLIENT_ID
    client_secret: $PISTE_CLIENT_SECRET
```

## Nommage SFTP

Les fichiers Flux 1 sont ensuite archivés en tar.gz pour envoi SFTP :

```
{CODE_INTERFACE}_{CODE_APP}_{IDENTIFIANT_FLUX}.tar.gz
```

Exemple : `FFE0112A_AAA123_AAA1230112000000000000001.tar.gz`

Contenu : `Full_F202500003.xml`, `Base_F202500004.xml`, ...

Des fichiers de profils différents peuvent coexister dans un même flux.

## XSD de référence

- **CII Base** : `specs/xsd/e-invoicing/F1_BASE_CII_D22B/`
- **UBL Base** : `specs/xsd/e-invoicing/F1_BASE_UBL_2.1/`
- **CII Full** : `specs/xsd/e-invoicing/F1_FULL_CII_D22B/`
- **UBL Full** : `specs/xsd/e-invoicing/F1_FULL_UBL_2.1/`

## Tests

66 tests dans `pdp-transform::ppf_flux1::tests`, tous validés contre les XSD des spécifications externes PPF v3.1.

### Couverture par cas d'usage

| Cas d'usage | Type | CII Base | CII Full | UBL Base | UBL Full |
|-------------|------|:--------:|:--------:|:--------:|:--------:|
| Facture simple | 380 | ✅ | ✅ | ✅ | ✅ |
| Avoir | 381 | ✅ | ✅ | ✅ | ✅ |
| Facture rectificative | 384 | ✅ | ✅ | ✅ | ✅ |
| Acompte | 386 | ✅ | ✅ | ✅ | ✅ |
| Auto-facture | 389 | ✅ | ✅ | ✅ | ✅ |
| Définitive après acompte | 380 | ✅ | ✅ | ✅ | ✅ |
| Remises multi-TVA | 380 | ✅ | ✅ | ✅ | ✅ |
| Multi-vendeurs (B8) | 380 | ✅ | ✅ | ✅ | ✅ |
| Délégation (S8) | 380 | ✅ | ✅ | ✅ | ✅ |
| Marketplace (A8) | 380 | ✅ | ✅ | ✅ | ✅ |
| Sous-traitance (A4) | 380 | ✅ | ✅ | ✅ | ✅ |
| Représentant fiscal | 380 | — | — | ✅ | ✅ |
| UC1 officiel PPF | 380 | ✅ | ✅ | ✅ | ✅ |

### Compatibilité des formats source

| Format source | Lignes ? | Compatible Full ? |
|---------------|:--------:|:-----------------:|
| CII D22B (EN16931 / EXTENDED) | ✅ | ✅ |
| UBL 2.1 (EN16931 / EXTENDED) | ✅ | ✅ |
| Factur-X EN16931 / EXTENDED / BASIC | ✅ | ✅ |
| Factur-X BASIC WL / MINIMUM | ❌ | ❌ → Base seulement |

### Autres tests

- **Stratégie de profil** : `from_config("auto"|"base"|"full")`, AlwaysBase avec lignes, AlwaysFull sans lignes (fallback)
- **Détection de profil** : sans lignes → Base, avec lignes → Full
- **Convention de nommage** : `Base_` / `Full_` prefix
- **Direction XSLT** : CII/UBL/Factur-X × Base/Full
- **Processor intégration** : CII Base, UBL Base, CII Full, UBL Full (avec XSD)
- **Skip CDAR** : les documents CDAR sont ignorés
