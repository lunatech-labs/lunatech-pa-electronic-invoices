# Génération Factur-X (PDF/A-3a)

Le pipeline Factur-X génère des PDFs conformes **PDF/A-3a** (Tagged PDF, accessibilité) validés par veraPDF (155 règles, 37 234 vérifications, 0 échec).

## Pipeline

```
CII/UBL XML ──▶ Typst (invoice.typ) ──▶ PDF visuel (~5 ms)
                                              │
                                       lopdf (embed XML + PJ)
                                              │
                                       qpdf (fix header)
                                              ▼
                                       Factur-X PDF/A-3a
```

- **Typst** — génère le PDF visuel in-process (~5 ms), fonts embarquées (SourceSansPro TTF), Tagged PDF
- **lopdf** — embarque `factur-x.xml` (AFRelationship=Data), pièces jointes BG-24, métadonnées XMP Factur-X
- **qpdf** — corrige le header binaire PDF/A (lopdf ne l'écrit pas)

## Générer des exemples

```bash
# 13 exemples PDF (CII, UBL, Factur-X, avec/sans PJ) dans output/sample_pdfs/
cargo run -p pdp-transform --example generate_sample_pdfs

# Exemples Factur-X multi-profils dans output/
cargo test -p pdp-transform -- export_facturx_examples --ignored --nocapture
```

## Validation

Le pipeline applique 3 niveaux de validation :

1. **Validation métier** — champs obligatoires, cohérence des montants
2. **Validation XSD** — conformité structurelle (UBL 2.1, CII D22B, CDAR D22B)
3. **Validation Schematron** — règles EN16931 V1.3.15 + BR-FR V1.2.0 (via Saxon XSLT 2.0)

Les spécifications sont dans `specs/` :
- `specs/xsd/` — Schémas XSD (UBL base/full, CII base/full, CDAR)
- `specs/schematron/` — Schematrons EN16931 et BR-FR
- `specs/xslt/` — XSLT compilés pour EN16931 et BR-FR
- `specs/fonts/` — Polices TTF Source Sans/Serif Pro pour la génération PDF
- `specs/typst/` — Template Typst pour la génération PDF
- `specs/examples/` — Exemples officiels EN16931 et XP Z12
