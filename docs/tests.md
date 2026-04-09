# Tests et benchmarks

## Lancer les tests

```bash
# Tous les tests (405 tests)
cargo test --workspace

# Tests par crate
cargo test -p pdp-core         #  47 tests (pipeline, routing, model, archive ZIP/tar.gz)
cargo test -p pdp-invoice      # 105 tests (parsing UBL/CII/Factur-X + validation métier)
cargo test -p pdp-validate     #  17 tests (XSD + Schematron EN16931 + BR-FR)
cargo test -p pdp-cdar         #  86 tests (génération + parsing + réception CDV)
cargo test -p pdp-ereporting   #   7 tests (transactions + paiements + XML)
cargo test -p pdp-client       #  33 tests (PPF nommage/tar.gz + AFNOR + annuaire)
cargo test -p pdp-config       #   3 tests
cargo test -p pdp-trace        #   3 tests
cargo test -p pdp-transform    #  93 tests (UBL↔CII↔Factur-X↔PDF + pièces jointes multi-types + veraPDF + SaxonC FFI)

# Tests avec sortie détaillée
cargo test --workspace -- --nocapture
```

## Benchmarks

Trois suites de benchmarks Criterion sont disponibles :

```bash
# Tous les benchmarks
cargo bench --workspace

# Benchmarks par crate
cargo bench -p pdp-invoice     # Parsing UBL, CII, Factur-X
cargo bench -p pdp-cdar        # Génération et parsing CDV
cargo bench -p pdp-transform   # Transformation UBL↔CII

# Un benchmark spécifique
cargo bench -p pdp-cdar -- parse_cdar
cargo bench -p pdp-invoice -- parse_ubl
```

Les rapports HTML Criterion sont générés dans `target/criterion/`.

## Génération de fixtures

Les fixtures Factur-X sont des PDF/A-3a conformes générés via le pipeline complet (Typst + lopdf + qpdf).

```bash
# Régénérer les fixtures Factur-X (3 fichiers : facture, avoir, rectificative)
cargo test -p pdp-invoice --test generate_facturx_fixtures -- --ignored --nocapture

# Exporter des exemples Factur-X dans output/
cargo test -p pdp-transform -- export_facturx_examples --ignored --nocapture

# Exporter des conversions avec pièces jointes dans output/
cargo test -p pdp-transform -- export_conversions_with_attachments --ignored --nocapture
```

## Validation veraPDF

```bash
# Valider un PDF Factur-X avec veraPDF (profil PDF/A-3A)
verapdf --flavour 3a --format text tests/fixtures/facturx/facture_facturx_001.pdf

# Valider toutes les fixtures
verapdf --flavour 3a --format text tests/fixtures/facturx/*.pdf
```

## Fixtures de test

- `tests/fixtures/ubl/` — Factures UBL (standard, avoir, rectificative 384, autofacture 389, marketplace A8, sous-traitance A4, représentant fiscal)
- `tests/fixtures/cii/` — Factures CII (standard, avoir 381, rectificative 384, acompte, définitive après acompte, marketplace A8, sous-traitance A4, remises multi-TVA)
- `tests/fixtures/facturx/` — Factures Factur-X PDF/A-3a conformes (facture, avoir, rectificative)
- `tests/fixtures/cdar/` — CDV déposée (200), rejetée (213), litige (207), paiement transmis (211)
- `tests/fixtures/errors/` — Factures invalides
- `specs/examples/en16931-ubl/` — Exemples officiels EN16931 UBL (15+)
- `specs/examples/en16931-cii/` — Exemples officiels EN16931 CII (15)
- `specs/examples/xp-z12-012/` — Cas d'usage XP Z12-012 (factures + CDV cycle de vie)
- `specs/examples/xp-z12-014/` — Cas d'usage XP Z12-014 (UC1-UC5 B2B)
