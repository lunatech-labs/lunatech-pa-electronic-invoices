# Spec-driven development — convention Ferrite

## Principe

Le code de Ferrite implémente des **spécifications externes** : EN16931
(format facture européen), XP Z12-012/013/014 (PPF français), DSE AIFE
(spécifications externes), Factur-X 1.08, PEPPOL BIS Billing 3, etc.

Ces specs sont la **source de vérité** : chaque règle métier
(`BR-FR-12`, `BT-49`, `BR-CO-26`…) a une référence canonique dans un
document AFNOR ou européen. Notre travail consiste à mapper ces règles
vers du code Rust (parsing, validation, transformation) et à le tester.

Pour qu'un futur lecteur (ou nous-mêmes dans 6 mois) puisse remonter
**de chaque ligne de code à sa règle source** — et inversement —, on
applique les conventions suivantes :

## 1. Identifier les règles avec leur code spec

Chaque règle implémentée porte son code dans :

- le **commentaire** de la fonction ou du test
- le **rule_id** des structures de validation (`ValidationIssue.rule_id`)
- le **message d'erreur** envoyé à l'utilisateur

Exemple ([crates/pdp-invoice/src/validator.rs:138](../crates/pdp-invoice/src/validator.rs#L138)) :

```rust
// BR-FR-12 : schemeID du point d'échange vendeur (BT-34-1)
// Valeurs autorisées par la norme française : 0002, 0007, 0009, …, 0225, 0230
if let Some(ref scheme) = invoice.seller_endpoint_scheme {
    if !is_valid_eas_scheme(scheme) {
        errors.push(ValidationIssue {
            rule_id: "BR-FR-12".to_string(),
            // ...
        });
    }
}
```

## 2. Référencer la spec dans la doc

Chaque module ou fonction qui implémente une règle de spec **cite la
section** :

```rust
/// Vérifie que le SIREN du vendeur (BT-30) et de l'acheteur (BT-47)
/// existent dans l'annuaire PPF.
///
/// Source : AFNOR XP Z12-013 G1.63 / DSE AIFE §5.2.3.
/// Codes IRR : `EMMET_INC` (vendeur), `DEST_INC` (acheteur).
pub struct AnnuaireValidationProcessor { ... }
```

## 3. Tables de conformité par domaine

Chaque domaine fonctionnel doit avoir un tableau dans `docs/` qui
liste **toutes les règles applicables** + leur état + le code qui les
implémente. Format minimal :

| Règle | Source spec | Implémentation | Tests | État |
|---|---|---|---|---|
| `BR-FR-12` | XP Z12-012 §3.5 | [validator.rs:138](../crates/pdp-invoice/src/validator.rs#L138) | `test_br_fr_12_valid_eas` | ✅ |
| `BR-FR-21` | XP Z12-012 §4.2 | Schematron uniquement | (Schematron) | ⚠️ Rust manquant |

États possibles :

- ✅ **livré** — implémenté + testé + actif en prod
- ⚠️ **partiel** — couvert par une voie (ex. Schematron) mais pas
  toutes (Rust direct), ou désactivable
- ❌ **manquant** — règle connue, pas encore codée
- 🔒 **désactivé** — code présent mais flag de config off (et pourquoi)

**Inventaire des domaines** (objectif : un tableau de conformité par
ligne, voir [todo.md §16ter](todo.md#16ter-chantier-spec-driven--généraliser-à-tout-le-projet) pour le chantier de
généralisation) :

| Domaine | Doc | État du tableau de conformité |
|---|---|---|
| Adresses électroniques (BT-34/BT-49) | [conformite-adresses-electroniques.md](conformite-adresses-electroniques.md) | ✅ format complet (référence) |
| Conformité globale AFNOR | [todo.md §Vue d'ensemble](todo.md#vue-densemble--conformité-afnor) | ⚠️ score uniquement, pas de mapping |
| Acteurs CDV / CDAR | [cdar.md](cdar.md) | ⚠️ à reformater |
| Codes IRR (CDV 501) | dispersé dans cdar.md | ❌ à créer |
| E-reporting Flux 10 | [ereporting.md](ereporting.md) | ⚠️ à reformater |
| Annuaire F13/F14 | [annuaire.md](annuaire.md) | ⚠️ à reformater |
| Réception fichier (REC-*, BR-FR-19) | commentaires `pdp-core/src/reception.rs` | ❌ à créer |
| EN16931 (BR-* / BR-CO-* / BR-CL-*) | Schematron dans `specs/` | ❌ pas de récap Rust |
| BR-FR (XP Z12-012) | Rust + Schematron dispersés | ❌ à consolider |
| Transformation UBL ↔ CII | [api.md](api.md) | ⚠️ partiel |
| Factur-X (PDF/A-3 + XML) | [facturx.md](facturx.md) | ⚠️ à reformater |
| HTTP API (XP Z12-013 §5) | [http-api.md](http-api.md) | ⚠️ à reformater |
| Webhooks (XP Z12-013 §5.4) | [webhooks.md](webhooks.md) | ⚠️ à reformater |
| Auth / RBAC / isolation tenant | [ui.md](ui.md) | ⚠️ à compléter |
| PEPPOL AS4 / Oxalis | [peppol.md](peppol.md) | ⚠️ à reformater |

## 4. Tests nommés par règle

Les tests qui valident une règle de spec doivent porter le code dans
leur nom :

```rust
#[test]
fn test_br_fr_12_rejects_unknown_scheme_id() { /* … */ }

#[test]
fn test_br_co_26_seller_identifier_required() { /* … */ }
```

Cela permet `cargo test br_fr_12` pour cibler une règle, et
`grep -r "BR-FR-12"` pour retrouver toutes ses traces (impl + tests +
docs + commentaires).

## 5. Specs vendor copiées dans `specs/`

Les specs externes (Schematron AFNOR, XSD UN/CEFACT, code lists, etc.)
sont **copiées dans le repo** sous [`specs/`](../specs/) :

```
specs/
├── schematron/                    # Règles XSLT compilées
│   ├── facturx-1.08/EN16931/      # 200+ règles BR-* / BR-CL-* / BR-CO-*
│   └── br-fr-1.2.0/               # 36 règles BR-FR-*
├── xsd/                           # Schémas XML UBL et CII
└── afnor/                         # Specs PDF (référence)
```

Avantage : un `grep -r BR-FR-12 specs/` retrouve immédiatement la
définition canonique de la règle. L'utilitaire CLI `pdp validate`
applique ces Schematron sur le XML brut.

## 6. Convention de naming des codes

| Préfixe | Source | Exemple | Domaine |
|---|---|---|---|
| `BR-CO-*` | EN16931 (Core) | BR-CO-26 | Cohérence champs |
| `BR-CL-*` | EN16931 (Code List) | BR-CL-21 | Code list (pays, devise) |
| `BR-DEC-*` | EN16931 (Decimals) | BR-DEC-02 | Précision décimale |
| `BR-S-*` | EN16931 (TVA) | BR-S-08 | Catégorie TVA standard |
| `BR-Z-*` / `BR-O-*` / `BR-E-*` | EN16931 (TVA) | BR-Z-01 | TVA 0% / hors champ / exonérée |
| `BR-AE-*` / `BR-IC-*` / `BR-G-*` | EN16931 (TVA) | BR-AE-04 | Reverse charge / intracom / export |
| `BR-FR-*` | XP Z12-012 | BR-FR-12 | Spécifique France |
| `BR-IT-*` / `BR-DE-*` | National | BR-IT-09 | Spécifique Italie / Allemagne |
| `BT-*` | EN16931 (Business Term) | BT-49 | Identifiant de champ |
| `BG-*` | EN16931 (Business Group) | BG-25 | Groupe de champs |
| `IRR_*` | XP Z12-013 (CDV 501) | IRR_TYPE_F | Motif d'irrecevabilité |
| `REC-*` | Ferrite (interne) | REC-05 | Contrôle de réception fichier |
| `EMMET_INC` / `DEST_INC` | XP Z12-013 G1.63 | — | Annuaire (sender/receiver inconnu) |

## 7. Flux de travail spec-driven

Lorsqu'on ajoute une règle :

1. **Lire la spec** (PDF AFNOR, EN16931, etc.) — pas le résumé,
   l'original.
2. **Citer le code** de la règle (`BR-FR-XX`) et la **section** du doc
   source dans le commentaire de la fonction/du test.
3. **Ajouter au tableau de conformité** du domaine (créer le fichier
   dans `docs/` s'il n'existe pas).
4. **Écrire un test nommé** `test_<rule_id_lowercase>_<comportement>`.
5. **Si la règle est dans un Schematron du repo**, vérifier qu'il
   n'y a pas de double implémentation incohérente (le Schematron est
   alors la source faisant autorité ; le code Rust ne peut être plus
   strict).

Lorsqu'on découvre un manque :

1. Ajouter une ligne ⚠️ ou ❌ dans le tableau de conformité.
2. Référencer la ligne dans `docs/todo.md` avec un lien direct.
3. Si bloquant pour la prod, mentionner dans la **vue d'ensemble**
   du todo (table de scores).

## 8. Outils internes

- `cargo test <rule_code>` — lance les tests d'une règle (grâce au
  naming convention)
- `grep -rn "BR-FR-12" crates/ specs/ docs/` — trouve toutes les
  traces (impl + spec + doc + tests + messages d'erreur)
- `pdp validate <fichier>` — applique les Schematron, retourne la
  liste des `rule_id` violés
- `pdp tools schematron-rules` (à venir) — extrait toutes les règles
  des Schematron `specs/` et liste celles non couvertes par un test
  Rust

## Voir aussi

- [todo.md §Vue d'ensemble](todo.md) — score de conformité par spec
- [conformite-adresses-electroniques.md](conformite-adresses-electroniques.md) — exemple complet de tableau spec→code pour BT-34/BT-49
- [tracabilite.md](tracabilite.md) — comment chaque erreur de validation atterrit dans Elasticsearch avec son `rule_id`
