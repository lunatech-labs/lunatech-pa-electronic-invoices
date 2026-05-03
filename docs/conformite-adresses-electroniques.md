# Conformité — Adresses électroniques (BT-34, BT-49)

> Document **spec-driven** (cf. [spec-driven.md](spec-driven.md)). Source
> de vérité : EN16931 + XP Z12-012 + DSE AIFE. Chaque règle est mappée vers
> son code Rust, son Schematron et ses tests.

## Contexte

EN16931 définit deux champs d'adresse électronique sur une facture :

- **BT-34** — *Seller electronic address* : identifiant de routage du
  vendeur, paire `(value, scheme_id)` (`BT-34-1`).
- **BT-49** — *Buyer electronic address* : identifiant de routage de
  l'acheteur, paire `(value, scheme_id)` (`BT-49-1`).

Le `scheme_id` est un code d'**EAS list** (Electronic Address Scheme) géré
par le CEF — par exemple `0009` pour SIRET français, `0088` pour GLN
international, `0225` pour le SIREN dans le contexte PPF.

En France (XP Z12-012 §4.2), pour un flux **B2B BAR** (Business As a
Receiver) entre un vendeur et un acheteur tous deux assujettis :

- Le BT-49 (acheteur) doit être au format `<SIREN>+...` avec
  `scheme_id = "0225"` (BR-FR-21).
- En autofacture, le BT-34 (vendeur) suit la même règle (BR-FR-22).

## « Désactivé en démo » — qu'est-ce que ça veut dire ?

Le `config-ui-demo.yaml` (utilisé par les screenshots et `pdp demo populate`)
porte :

```yaml
validation:
  specs_dir: ./specs
  xsd_enabled: false        # XSD UBL / CII non vérifiés
  en16931_enabled: false    # règles européennes BR-* non appliquées
  br_fr_enabled: false      # règles françaises BR-FR-* non appliquées
```

Concrètement, le `SchematronValidator` (cf. [pdp-validate](../crates/pdp-validate/src))
**ne tourne pas** quand ces flags sont à `false` : il retourne un rapport
vide. Les seules vérifications appliquées sont alors :

1. les contrôles de réception fichier `REC-01` à `REC-05`
   ([pdp-core/src/reception.rs](../crates/pdp-core/src/reception.rs))
2. les **validations Rust en dur** dans `pdp-invoice/src/validator.rs`
   (BR-FR-04, BR-FR-12, BR-FR-13)
3. la déduplication BR-FR-12/13 sur `invoice_key`
4. la validation annuaire G1.63 (`AnnuaireValidationProcessor`)

C'est volontaire : la démo charge ~280 fixtures dont certaines sont des
exemples synthétiques qui **ne passent pas EN16931 strict** (totaux légèrement
incohérents, codes de catégorie TVA simplifiés, etc.). Les activer rejetterait
la majorité.

**En production, ces flags doivent être à `true`** (cf.
[config-pdp.yaml](../config-pdp.yaml) si présent). Une PDP qui accepte une
facture non conforme EN16931 risque de transmettre au PPF un flux qui sera
ensuite rejeté par CDV 213 — et le tenant émetteur ne sait pas pourquoi sa
facture n'arrive pas chez l'acheteur.

## Tableau de conformité

| Règle | Source spec | Implémentation Ferrite | Tests | État |
|---|---|---|---|---|
| **BR-62** — BT-34 doit avoir un `scheme_id` | EN16931 §5.5 | Schematron [Factur-X_1.08_EN16931.sch:639](../specs/schematron/facturx-1.08/EN16931/Factur-X_1.08_EN16931.sch) | (Schematron) | ✅ via XSLT — désactivé en `config-ui-demo.yaml` |
| **BR-63** — BT-49 doit avoir un `scheme_id` | EN16931 §5.5 | Schematron ligne 641 | (Schematron) | ✅ via XSLT — désactivé en `config-ui-demo.yaml` |
| **BR-FR-12** — BT-34-1 ∈ liste EAS | XP Z12-012 §3.5 | Rust [validator.rs:138-154](../crates/pdp-invoice/src/validator.rs#L138) + Schematron `BR-FR-Flux2-Schematron-CII_V1.2.0.sch:525` | `pdp-invoice/src/validator.rs::tests::test_br_fr_12_*` | ✅ Rust **toujours actif** + Schematron |
| **BR-FR-13** — BT-49-1 ∈ liste EAS | XP Z12-012 §3.5 | Rust [validator.rs:156-169](../crates/pdp-invoice/src/validator.rs#L156) + Schematron ligne 538 | `pdp-invoice/src/validator.rs::tests::test_br_fr_13_*` | ✅ idem |
| **BR-FR-21** — BT-49 = SIREN(BT-47) + BT-49-1 = "0225" en B2B BAR | XP Z12-012 §4.2 | Schematron [`BR-FR-Flux2-Schematron-CII_V1.2.0.sch:668-682`](../specs/schematron/br-fr-1.2.0/20251114_BR-FR-Flux2-Schematron-CII_V1.2.0.sch) **uniquement** | aucun test Rust direct | ⚠️ **Couvert seulement par Schematron, désactivable** |
| **BR-FR-22** — idem BT-34 en autofacture | XP Z12-012 §4.2 | Schematron ligne 688 uniquement | aucun test Rust direct | ⚠️ idem |
| **G1.63** — SIREN vendeur/acheteur existe dans l'annuaire | XP Z12-013 §G1 | Rust [pdp-annuaire/src/processor.rs](../crates/pdp-annuaire/src/processor.rs) — `AnnuaireValidationProcessor` | `pdp-annuaire/tests/annuaire_validation_test.rs` | ✅ EMMET_INC / DEST_INC |
| **Lookup ligne d'annuaire BT-49** — l'EndpointID acheteur correspond à une ligne d'annuaire active (SIRET + routing_code) | XP Z12-013 §5.4 + §G1 | `AnnuaireService.lookup_code_routage` existe ([db.rs:496](../crates/pdp-annuaire/src/db.rs#L496)) mais **n'est pas câblé** dans le pipeline pour valider BT-49 | aucun | ❌ **Manquant** — voir [todo.md §X](todo.md) |
| **Liste EAS** (41 valeurs) | EN16931 + CEF code list | Rust `is_valid_eas_scheme` ([validator.rs:334](../crates/pdp-invoice/src/validator.rs#L334)) | `pdp-invoice/src/validator.rs::tests::test_eas_*` | ✅ Liste codée en dur — **à synchroniser** avec mises à jour CEF |

## Détail des règles

### BR-FR-12 / BR-FR-13 — schemeID dans la liste EAS

**Rust** [crates/pdp-invoice/src/validator.rs:138-169](../crates/pdp-invoice/src/validator.rs#L138) :

```rust
// BR-FR-12 : schemeID du point d'échange vendeur (BT-34-1)
if let Some(ref scheme) = invoice.seller_endpoint_scheme {
    if !is_valid_eas_scheme(scheme) {
        errors.push(ValidationIssue {
            rule_id: "BR-FR-12".to_string(),
            severity: Severity::Error,
            field: "seller_endpoint_scheme".to_string(),
            message: format!(
                "BR-FR-12 : Le schemeID '{}' du point d'échange vendeur (BT-34-1) n'est pas conforme à la liste EAS",
                scheme
            ),
        });
    }
}
```

Cette validation **tourne toujours**, indépendamment du flag
`validation.br_fr_enabled` du YAML — c'est une vérification structurelle
faite après parsing. Le flag concerne uniquement les Schematron.

### BR-FR-21 — BT-49 = SIREN+0225 en B2B BAR

**Schematron uniquement** ([specs/schematron/br-fr-1.2.0/20251114_BR-FR-Flux2-Schematron-CII_V1.2.0.sch:681](../specs/schematron/br-fr-1.2.0/20251114_BR-FR-Flux2-Schematron-CII_V1.2.0.sch)) :

```xml
<assert test="not($isB2B and not($isExcludedDocType))
              or (starts-with($endpointID, $siren) and $endpointSchemeID = '0225')"
        flag="warning"
        id="BR-FR-21_BT-49">
  BR-FR-21/BT-49 : Si le traitement est BAR/B2B et que le type de
  document (BT-3) n'est pas en autofacture (389, 501, 500, 471, 473,
  261, 502), alors le BT-49 (EndpointID) doit commencer par le SIREN
  (BT-47) et le BT-49-1 (schemeID) doit être égal à "0225".
</assert>
```

**Limites actuelles** :

1. La validation passe uniquement quand `validation.br_fr_enabled: true`
   et que le XML brut est disponible (les fixtures parsées sans XML brut
   passent en validation structurelle minimale).
2. Le code Rust ne dédouble pas cette vérification → si Schematron
   désactivé, **la règle n'est pas vérifiée**.
3. Pas de test Rust ciblé `test_br_fr_21_*`.

**Action proposée** : porter BR-FR-21/22 en Rust dans `validator.rs`
(condition simple : si `flow_type == B2B` et `doc_type ∉ autofactures`,
alors `seller_siren ∈ buyer_endpoint_id` et `buyer_endpoint_scheme == "0225"`).

### Lookup annuaire du BT-49 (manquant)

XP Z12-013 §G1 prévoit que la PDP émettrice **vérifie l'adresse de
routage** du destinataire avant émission : le couple (SIRET acheteur +
routing_code) doit correspondre à une ligne d'annuaire active sinon le
flux est rejetable avec `DEST_INC`.

L'infrastructure existe :

- [`AnnuaireStore::lookup_code_routage(siret, code)`](../crates/pdp-annuaire/src/db.rs#L496)
  retourne la `RoutingCodeRow` si elle existe.
- L'endpoint Directory Service `GET /v1/routing-code/siret:{siret}/code:{code}`
  l'expose côté API.

Mais le pipeline d'émission ne fait que valider l'**existence du SIREN**
(via `AnnuaireValidationProcessor`, mode `Emission`) — pas la concordance
entre BT-49 (EndpointID acheteur) et la ligne d'annuaire.

**Action proposée** : étendre `AnnuaireValidationProcessor` (mode
`Emission`) pour décomposer le BT-49, lookuper la routing_code, et
émettre `DEST_INC` si absent. Voir [todo.md §X.Y](todo.md) (à créer).

## Liste EAS (Electronic Address Scheme)

Codée en dur dans [validator.rs:334](../crates/pdp-invoice/src/validator.rs#L334) — 41 valeurs.

Subset le plus utilisé en France :

| Code | Description | Usage typique |
|---|---|---|
| `0002` | System Information et Repertoire des Entreprise et des Etablissements: SIRENE | Identifiant SIREN sans contexte routage |
| `0009` | SIRET-CODE | Identifiant SIRET vendeur/acheteur |
| `0088` | EAN Location Code (GLN) | Logistique internationale |
| `0142` | RFC 5322 (email address) | Adresse email simple |
| `0225` | FR:CPRO (Code PPF) | **Identifiant unique PPF basé sur SIREN — recommandé pour B2B France** |

La liste complète CEF est mise à jour ~2x/an. **Action recommandée** :
ajouter un script `scripts/sync-eas-list.py` qui télécharge le code list
officiel et regénère le tableau Rust (TODO).

## Tests à ajouter

- [ ] `test_br_fr_21_b2b_bar_endpoint_must_match_siren` — accepte un
      BT-49 conforme, rejette les non conformes
- [ ] `test_br_fr_22_autofacture_seller_endpoint_must_match_siren`
- [ ] `test_annuaire_validation_dest_inc_when_routing_unknown` —
      facture émise avec un BT-49 qui ne correspond à aucune ligne
      d'annuaire → erreur DEST_INC

## Voir aussi

- [spec-driven.md](spec-driven.md) — convention générale
- [todo.md §3](todo.md) — réception inter-PDP / routage
- [annuaire.md](annuaire.md) — copie locale F14, `lookup_code_routage`
- EN16931:2017 (norme européenne, payante) — section 5.5 pour BT-34/BT-49
- AFNOR XP Z12-012 V1.2 (PPF formats & profils) — §4.2 contraintes B2B
- AFNOR XP Z12-013 V1.2 (PPF APIs Flow/Directory) — §G1.63 contrôle annuaire
