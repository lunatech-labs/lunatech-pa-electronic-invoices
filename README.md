# PDP Facture - Plateforme de Dématérialisation Partenaire

Librairie modulaire en Rust pour la facturation électronique conforme à la réforme française (EN16931, Factur-X, PPF, AFNOR).

## Architecture modulaire

```
┌─────────────────────────────────────────────────────────────────────────┐
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌────────────┐ │
│  │   Parsing    │  │  Validation  │  │Transformation│  │ Génération │ │
│  │  pdp-invoice │  │ pdp-validate │  │pdp-transform │  │  Factur-X  │ │
│  │ UBL/CII/FX  │  │ XSD+Schematron│ │ UBL↔CII/PDF │  │  PDF/A-3a  │ │
│  └──────────────┘  └──────────────┘  └──────────────┘  └────────────┘ │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌────────────┐ │
│  │  CDV (CDAR)  │  │ E-Reporting  │  │   Archives   │  │   Client   │ │
│  │  pdp-cdar    │  │pdp-ereporting│  │  ZIP/tar.gz  │  │ PPF/AFNOR  │ │
│  └──────────────┘  └──────────────┘  └──────────────┘  └────────────┘ │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌────────────┐ │
│  │   PEPPOL     │  │    SFTP      │  │ Traçabilité  │  │   Modèle   │ │
│  │ pdp-peppol   │  │  pdp-sftp    │  │  pdp-trace   │  │  pdp-core  │ │
│  │  AS4/SBDH    │  │              │  │              │  │            │ │
│  └──────────────┘  └──────────────┘  └──────────────┘  └────────────┘ │
│  ┌──────────────┐  ┌──────────────┐                                   │
│  │Configuration │  │   Client     │                                   │
│  │ pdp-config   │  │  PPF/AFNOR   │                                   │
│  └──────────────┘  └──────────────┘                                   │
└─────────────────────────────────────────────────────────────────────────┘
```

```
Fichier/Archive ──▶ Réception ──▶ Routage ──▶ Parsing ──▶ Validation ──▶ Flux1 PPF ──▶ Transformation ──▶ Distribution
(SFTP/FS/PEPPOL)    (pdp-core)   (pdp-cdar)  (pdp-invoice) (pdp-validate) (pdp-transform) (pdp-transform)   (pdp-peppol)
                     │            │                │              │           │                │           (pdp-client)
                     ▼            ▼                ▼              ▼           ▼                ▼                │
                 CDAR 501     Facture?         InvoiceData    Rapport XSD  Base/Full_{num} Factur-X PDF    PEPPOL AS4
                 si irrecevable CDAR? ──▶ CDV   parsée       + Schematron  → SFTP PPF      ou CII/UBL    PPF SFTP tar.gz
```

## Crates

| Crate | Rôle | Tests |
|-------|------|-------|
| `pdp-core` | Modèle de données (`InvoiceData` EN16931), pipeline async, erreurs, archives ZIP/tar.gz, décompression auto à l'entrée | 47 |
| `pdp-invoice` | **Parsing** : UBL 2.1, CII D22B, Factur-X (PDF), détection auto (facture/CDAR/e-reporting), pièces jointes BG-24 | 111 |
| `pdp-validate` | **Validation** : XSD (libxml2) + Schematron (Saxon) — EN16931, BR-FR, Factur-X | 17 |
| `pdp-transform` | **Transformation** : UBL ↔ CII (XSLT), Factur-X PDF/A-3a, PDF visuel, Flux 1 PPF (Base + Full), SaxonC FFI | 107 |
| `pdp-cdar` | **CDV** : génération, parsing et routage CDAR D22B, statuts 200→212, `DocumentTypeRouter` | 108 |
| `pdp-ereporting` | **E-reporting** : flux 10.1/10.2/10.3/10.4 | 7 |
| `pdp-peppol` | **PEPPOL** : SBDH, SMP lookup, gateway AS4 (Oxalis/phase4), envoi/réception inter-PDP | 51 |
| `pdp-client` | **Communication** : PPF SFTP, AFNOR Flow, Annuaire PISTE | 33 |
| `pdp-sftp` | **SFTP** : consumer + producer, auth RSA | — |
| `pdp-trace` | **Traçabilité** : Elasticsearch (un index par SIREN), archivage XML+PDF | 3 |
| `pdp-config` | **Configuration** : YAML | 3 |
| `pdp-app` | **CLI** : binaire principal | — |

**Total : 490+ tests** (+ 5 tests ignorés pour génération de fixtures)

## Formats supportés

- **UBL** (Universal Business Language) — XML
- **CII** (Cross-Industry Invoice / UN/CEFACT D22B) — XML
- **Factur-X** — PDF/A-3a avec XML CII embarqué (conforme veraPDF)
- **PDF** — PDF visuel seul (sans XML embarqué)
- **CDV/CDAR** (D22B) — Compte-rendu De Vie
- **E-Reporting** (XSD PPF V1.0) — Flux 10.1/10.2/10.3/10.4

## Matrice de conversion

| Source ↓ / Cible → | CII | UBL | Factur-X | PDF |
|---------------------|-----|-----|----------|-----|
| **UBL**             | ✅ XSLT | — | ✅ XSLT+FOP+lopdf | ✅ FOP |
| **CII**             | — | ✅ XSLT | ✅ FOP+lopdf | ✅ FOP |
| **Factur-X**        | ✅ extraction | ✅ extraction+XSLT | — | ✅ retourne PDF |

Les pièces jointes (BG-24) sont préservées dans toutes les conversions : XML base64, PDF embarqué, extraction Factur-X.

## Démarrage rapide

```bash
# Prérequis macOS
brew install pkgconf saxon fop qpdf

# Elasticsearch (traçabilité + archivage)
docker run -d --name pdp-es -p 9200:9200 -e "discovery.type=single-node" -e "xpack.security.enabled=false" elasticsearch:8.15.0

# Build
cargo build --release

# Tests
cargo test --workspace    # 405 tests

# Benchmarks
cargo bench --workspace
```

```rust
use pdp_transform::{convert_to, OutputFormat};
use pdp_invoice::ubl::UblParser;

let xml = std::fs::read_to_string("facture.xml").unwrap();
let invoice = UblParser::new().parse(&xml).unwrap();
let result = convert_to(&invoice, OutputFormat::FacturX).unwrap();
std::fs::write("facture.pdf", &result.content).unwrap();
```

## Documentation

| Document | Contenu |
|----------|---------|
| [docs/api.md](docs/api.md) | API de conversion, exemples par format, pièces jointes BG-24 |
| [docs/performance.md](docs/performance.md) | Benchmarks Criterion (parsing, validation, transformation, pipeline) |
| [docs/facturx.md](docs/facturx.md) | Pipeline Factur-X PDF/A-3a, validation |
| [docs/tracabilite.md](docs/tracabilite.md) | Traçabilité Elasticsearch : architecture, index par SIREN, API |
| [docs/installation.md](docs/installation.md) | Prérequis, Elasticsearch, build, CLI, configuration |
| [docs/tests.md](docs/tests.md) | Tests par crate, benchmarks, fixtures, veraPDF |
| [docs/docker.md](docs/docker.md) | Docker/Podman, docker-compose |
| [docs/archives.md](docs/archives.md) | Archives ZIP/tar.gz : décompression auto à l'entrée, builders |
| [docs/cdar.md](docs/cdar.md) | CDV/CDAR D22B : routage entrant, sources (client/PDP/PPF), statuts 200→212 |
| [docs/peppol.md](docs/peppol.md) | PEPPOL AS4 : architecture 4 coins, SBDH, SMP, envoi/réception inter-PDP |
| [docs/flux1.md](docs/flux1.md) | Flux 1 PPF : XSLT CII/UBL → Base/Full, détection auto, différences Base vs Full, nommage SFTP |
| [docs/ppf-afnor.md](docs/ppf-afnor.md) | Communication PPF SFTP, annuaire PISTE, AFNOR |
| [docs/todo.md](docs/todo.md) | Roadmap : Typst, EndpointID, SFTP PPF, BR-FR, e-reporting, annuaire |

## Licence

MIT
