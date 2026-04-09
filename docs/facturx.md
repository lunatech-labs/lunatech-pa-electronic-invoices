# Génération Factur-X (PDF/A-3a)

Le pipeline Factur-X génère des PDFs conformes **PDF/A-3a** (Tagged PDF, accessibilité) validés par veraPDF (155 règles, 37 234 vérifications, 0 échec).

## Pipeline

```
CII/UBL XML ──▶ Saxon (cii-xr.xsl) ──▶ Saxon (xr-pdf.xsl) ──▶ FOP (PDF/A-3a)
                                                                     │
                                                              lopdf (embed XML)
                                                                     │
                                                              qpdf (fix header)
                                                                     ▼
                                                              Factur-X PDF/A-3a
```

- **Apache FOP** — génère le PDF visuel avec fonts embarquées (SourceSansPro/SourceSerifPro TTF), profil ICC sRGB, Tagged PDF
- **lopdf** — embarque `factur-x.xml` (AFRelationship=Data), pièces jointes BG-24, métadonnées XMP Factur-X
- **qpdf** — corrige le header binaire PDF/A (lopdf ne l'écrit pas)

## Générer des exemples

```bash
# Exemples Factur-X (sans pièces jointes) dans output/
cargo test -p pdp-transform -- export_facturx_examples --ignored --nocapture

# Exemples avec pièces jointes (PDF, PNG, CSV) dans output/
cargo test -p pdp-transform -- export_conversions_with_attachments --ignored --nocapture
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
- `specs/xslt/mustang/` — Stylesheets Mustang (CII/UBL → XR → FO), fonts TTF, profils ICC
- `specs/fop/` — Configuration Apache FOP pour PDF/A-3a
- `specs/examples/` — Exemples officiels EN16931 et XP Z12
