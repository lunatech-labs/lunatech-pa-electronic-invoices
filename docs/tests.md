# Tests et benchmarks

## Lancer les tests

```bash
# Tous les tests (897+ tests)
cargo test --workspace

# Tests par crate
cargo test -p pdp-core         #  91 tests (pipeline, routing, model, archive ZIP/tar.gz, reception)
cargo test -p pdp-invoice      # 125 tests (parsing UBL/CII/Factur-X + validation metier)
cargo test -p pdp-validate     #  24 tests (XSD + Schematron EN16931 + BR-FR)
cargo test -p pdp-cdar         # 122 tests (generation + parsing + reception CDV + fixtures XP Z12-012)
cargo test -p pdp-ereporting   #  88 tests (transactions + paiements + XML)
cargo test -p pdp-peppol       #  60 tests (Peppol BIS 3.0 + SMP + SML)
cargo test -p pdp-client       #  66 tests (PPF nommage/tar.gz + AFNOR + annuaire)
cargo test -p pdp-sftp         #   8 tests (transfert SFTP)
cargo test -p pdp-config       #  17 tests (YAML, multi-tenant)
cargo test -p pdp-trace        #   6 tests (Elasticsearch, deduplication)
cargo test -p pdp-transform    # 183 tests (UBL<->CII<->Factur-X<->PDF + profils + pieces jointes + Typst)
cargo test -p pdp-app          #  34 tests (serveur HTTP, auth, webhooks, e2e)

# Tests avec sortie detaillee
cargo test --workspace -- --nocapture
```

## Categories de tests

### Tests unitaires

Chaque crate contient des tests unitaires dans les modules `#[cfg(test)]`. Ils testent les fonctions individuelles en isolation.

```bash
# Exemple : tests du parser CII uniquement
cargo test -p pdp-invoice cii::tests
```

### Tests d'integration

Tests qui exercent plusieurs crates ensemble :

```bash
# Cycle de vie CDV complet (parse -> CDV 200/213/501 -> serialisation -> renvoi)
cargo test -p pdp-cdar --test lifecycle_integration

# Cas d'usage XP Z12-014 (UC1-UC5)
cargo test --test use_cases

# Routage client PPF/AFNOR
cargo test -p pdp-client --test routing_integration

# PEPPOL integration
cargo test -p pdp-peppol --test peppol_integration
```

### Tests end-to-end (API HTTP)

Tests du flux complet via l'API HTTP (axum oneshot). Soumettent de vraies factures CII/UBL via multipart, verifient la reception, l'auth Bearer, les metriques Prometheus.

```bash
cargo test -p pdp-app -- test_e2e
```

Cas couverts :
- Soumission facture CII reelle via `POST /v1/flows` -> verification channel
- Soumission facture UBL reelle
- Verification SHA-256 avec vraie facture
- Flux authentification complet (sans token -> 401, mauvais token -> 401, bon token -> 202)
- Soumission batch (CII + UBL) -> verification ordre dans le channel
- Metriques Prometheus apres batch (compteurs `pdp_flows_received_total`)

### Tests de conformite (fixtures officielles)

Les fixtures `tests/fixtures/xp-z12-014/` sont des copies identiques (byte-pour-byte) des exemples officiels AFNOR XP Z12-012 Annexe B V1.3. Les tests verifient que notre parser lit correctement ces fichiers de reference.

```bash
# Tests de parsing des fixtures officielles CDV
cargo test -p pdp-cdar --test lifecycle_integration -- test_fixture
```

Fixtures couvertes : CDV 200, 202, 203, 204, 205, 207 (avec MOTIF), 211 (MPA), 212 (MEN), 213 (DOUBLON), 200 POUR_PPF (guideline einvoicingF2).

## Benchmarks

Six suites de benchmarks Criterion sont disponibles :

```bash
# Tous les benchmarks
cargo bench --workspace

# Benchmarks par crate
cargo bench -p pdp-invoice      # Parsing UBL, CII, Factur-X, detection format
cargo bench -p pdp-cdar         # Generation et parsing CDV, throughput
cargo bench -p pdp-validate     # XSD + Schematron, pipeline complet
cargo bench -p pdp-transform    # XSLT UBL<->CII, pipeline, roundtrip
cargo bench -p pdp-core         # Archives tar.gz/ZIP, scalabilite 1-100 fichiers
cargo bench -p pdp-app          # Gros volumes : batch 10/100/1000, pipeline complet

# Un benchmark specifique
cargo bench -p pdp-cdar -- cdar_parse
cargo bench -p pdp-app -- batch_parse_cii
```

Les rapports HTML Criterion sont generes dans `target/criterion/`.

### Benchmarks gros volumes (`pdp-app`)

Mesurent la performance end-to-end sur des volumes realistes :

| Benchmark | Description |
|-----------|-------------|
| `batch_parse_cii/10..1000` | Parsing batch de N factures CII |
| `batch_parse_ubl/10..1000` | Parsing batch de N factures UBL |
| `pipeline_parse_to_cdv/single` | Pipeline unitaire : CII -> InvoiceData -> CDV 200 -> XML |
| `pipeline_parse_to_cdv/batch` | Pipeline batch (10, 100 factures) |
| `batch_cdv_generation/generate_serialize` | Generation + serialisation XML de N CDV |
| `batch_cdv_generation/roundtrip` | Roundtrip complet : generate -> serialize -> parse |
| `scaling_by_invoice_size/standard` | Facture CII standard |
| `scaling_by_invoice_size/large_50_lines` | Facture CII avec 50+ lignes |

## Generation de fixtures

Les fixtures Factur-X sont des PDF/A-3a conformes generes via le pipeline complet (Typst + lopdf + qpdf).

```bash
# Regenerer les fixtures Factur-X (3 fichiers : facture, avoir, rectificative)
cargo test -p pdp-invoice --test generate_facturx_fixtures -- --ignored --nocapture

# Exporter des exemples Factur-X dans output/
cargo test -p pdp-transform -- export_facturx_examples --ignored --nocapture

# Exporter des conversions avec pieces jointes dans output/
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

| Repertoire | Contenu |
|-----------|---------|
| `tests/fixtures/ubl/` | Factures UBL (standard, avoir, rectificative 384, autofacture 389, marketplace A8) |
| `tests/fixtures/cii/` | Factures CII (standard, avoir 381, rectificative, acompte, multi-TVA) |
| `tests/fixtures/facturx/` | Factures Factur-X PDF/A-3a conformes (facture, avoir, rectificative) |
| `tests/fixtures/cdar/` | CDV deposee (200), rejetee (213), litige (207), paiement (211) |
| `tests/fixtures/xp-z12-014/` | **Exemples officiels** AFNOR XP Z12-012 Annexe B V1.3 (UC1-UC5, CDV complets) |
| `tests/fixtures/errors/` | Factures invalides (pour tests de validation) |
| `specs/examples/en16931-ubl/` | Exemples officiels EN16931 UBL (15+) |
| `specs/examples/en16931-cii/` | Exemples officiels EN16931 CII (15) |

## Prerequis

```bash
# macOS
brew install pkgconf saxon qpdf

# Elasticsearch (pour pdp-trace — optionnel)
docker run -d --name pdp-es -p 9200:9200 \
  -e "discovery.type=single-node" \
  -e "xpack.security.enabled=false" \
  elasticsearch:8.15.0
```
