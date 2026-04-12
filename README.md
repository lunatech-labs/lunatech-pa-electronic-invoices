# PDP Facture - Plateforme de Dématérialisation Partenaire

Librairie modulaire en Rust pour la facturation électronique conforme à la réforme française (EN16931, Factur-X, PPF, AFNOR).

## Architecture modulaire

```
┌─────────────────────────────────────────────────────────────────────────┐
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌────────────┐ │
│  │   Parsing    │  │  Validation  │  │Transformation│  │ Génération │ │
│  │  pdp-invoice │  │ pdp-validate │  │pdp-transform │  │  Factur-X  │ │
│  │ UBL/CII/FX  │  │ XSD+Schematron│ │UBL↔CII/Typst│  │  PDF/A-3a  │ │
│  └──────────────┘  └──────────────┘  └──────────────┘  └────────────┘ │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌────────────┐ │
│  │  CDV (CDAR)  │  │ E-Reporting  │  │   Client     │  │ Serveur    │ │
│  │  pdp-cdar    │  │pdp-ereporting│  │  PPF/AFNOR   │  │  pdp-app   │ │
│  │              │  │              │  │ Batch/Retry  │  │ REST/Auth  │ │
│  └──────────────┘  └──────────────┘  └──────────────┘  └────────────┘ │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌────────────┐ │
│  │   PEPPOL     │  │    SFTP      │  │ Traçabilité  │  │   Modèle   │ │
│  │ pdp-peppol   │  │  pdp-sftp    │  │  pdp-trace   │  │  pdp-core  │ │
│  │ AS4/Oxalis   │  │              │  │ ES + Dédup   │  │ Pipeline   │ │
│  └──────────────┘  └──────────────┘  └──────────────┘  └────────────┘ │
│  ┌──────────────┐  ┌──────────────┐                                   │
│  │Configuration │  │ Multi-tenant │                                   │
│  │ pdp-config   │  │  Registry    │                                   │
│  │ YAML/Tenant  │  │  par SIREN   │                                   │
│  └──────────────┘  └──────────────┘                                   │
└─────────────────────────────────────────────────────────────────────────┘
```

```
Fichier/Archive ──▶ Réception ──▶ Dédup ──▶ Routage ──▶ Parsing ──▶ Validation ──▶ Flux1 PPF ──▶ Transformation ──▶ Distribution
(SFTP/FS/HTTP/      (pdp-core)  (pdp-trace) (pdp-cdar) (pdp-invoice)(pdp-validate)(pdp-transform)(pdp-transform)   (pdp-peppol)
 PEPPOL/Oxalis)      │            │           │              │           │                │           (pdp-client)
                     ▼            ▼           ▼              ▼           ▼                ▼                │
                 CDAR 501     Doublon?    Facture?       InvoiceData  Rapport XSD  Base/Full_{num} Factur-X PDF    PEPPOL AS4
                 si irrecevable          CDAR? ──▶ CDV   parsée      + Schematron  → SFTP PPF      ou CII/UBL    PPF SFTP tar.gz
                                                                                   (BatchProducer) (RetryProducer)
```

## Crates

| Crate | Rôle | Tests |
|-------|------|-------|
| `pdp-core` | Modèle de données (`InvoiceData` EN16931), pipeline async, erreurs, archives ZIP/tar.gz, `ChannelConsumer`, `RetryProducer`, `DeadLetterProducer`, `TenantTagProcessor` | 91 |
| `pdp-invoice` | **Parsing** : UBL 2.1, CII D22B, Factur-X (PDF), détection auto (facture/CDAR/e-reporting), pièces jointes BG-24, validation BR-FR-12/13 | 125 |
| `pdp-validate` | **Validation** : XSD (libxml2) + Schematron (Saxon) — EN16931, BR-FR, Factur-X | 24 |
| `pdp-transform` | **Transformation** : UBL ↔ CII (XSLT), Factur-X PDF/A-3a, PDF visuel (Typst), Flux 1 PPF (Base + Full), SaxonC FFI | 183 |
| `pdp-cdar` | **CDV** : génération, parsing et routage CDAR D22B, 21 statuts (200→501), `DocumentTypeRouter`, `CdvReturnProcessor` | 96 |
| `pdp-ereporting` | **E-reporting** : flux 10.1/10.2/10.3/10.4 | 88 |
| `pdp-peppol` | **PEPPOL** : SBDH, SMP lookup (MD5), gateway AS4 (Oxalis), envoi/réception inter-PDP | 60 |
| `pdp-annuaire` | **Annuaire PPF** : parser streaming F14 (10 Go en 81s), PostgreSQL, API Directory Service AFNOR, routage 4 mailles | 8 |
| `pdp-client` | **Communication** : PPF SFTP, AFNOR Flow, Annuaire PISTE, `BatchProducer` | 66 |
| `pdp-sftp` | **SFTP** : consumer + producer, auth RSA, vérification known_hosts | 8 |
| `pdp-trace` | **Traçabilité** : Elasticsearch (un index par SIREN), archivage XML+PDF, `DuplicateCheckProcessor` | 6 |
| `pdp-config` | **Configuration** : YAML, multi-tenant (`TenantRegistry`, discovery par SIREN) | 17 |
| `pdp-app` | **Serveur HTTP** : API REST (AFNOR Flow Service), auth Bearer, métriques Prometheus, graceful shutdown | 28 |

**Total : 836 tests** (+ 8 tests ignorés pour génération de fixtures et FOP Java)

## Formats supportés

- **UBL** (Universal Business Language) — XML
- **CII** (Cross-Industry Invoice / UN/CEFACT D22B) — XML
- **Factur-X** — PDF/A-3a avec XML CII embarqué (conforme veraPDF)
- **PDF** — PDF visuel seul (sans XML embarqué)
- **CDV/CDAR** (D22B) — Cycle De Vie
- **E-Reporting** (XSD PPF V1.0) — Flux 10.1/10.2/10.3/10.4

## Matrice de conversion

| Source ↓ / Cible → | CII | UBL | Factur-X | PDF |
|---------------------|-----|-----|----------|-----|
| **UBL**             | ✅ XSLT | — | ✅ XSLT+Typst+lopdf | ✅ Typst |
| **CII**             | — | ✅ XSLT | ✅ Typst+lopdf | ✅ Typst |
| **Factur-X**        | ✅ extraction | ✅ extraction+XSLT | — | ✅ retourne PDF |

Les pièces jointes (BG-24) sont préservées dans toutes les conversions : XML base64, PDF embarqué, extraction Factur-X.

## Démarrage rapide

```bash
# Prérequis macOS
brew install pkgconf saxon qpdf

# Elasticsearch (traçabilité + archivage)
docker run -d --name pdp-es -p 9200:9200 -e "discovery.type=single-node" -e "xpack.security.enabled=false" elasticsearch:8.15.0

# Build
cargo build --release

# Tests
cargo test --workspace    # 836 tests

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

## Serveur HTTP (API AFNOR Flow Service)

```bash
# Lancer le serveur
cargo run -- start --config config.yaml

# Envoyer une facture via l'API
curl -X POST http://localhost:8080/v1/flows \
  -H "Authorization: Bearer <token>" \
  -F "flowInfo=@flow.json;type=application/json" \
  -F "file=@facture.xml;type=application/xml"

# Consulter les métriques
curl http://localhost:8080/metrics

# Healthcheck
curl http://localhost:8080/health
```

**Endpoints :**
- `POST /v1/flows` — Dépôt de flux (multipart, auth Bearer)
- `GET /v1/flows/{flowId}` — Détail d'un flux
- `GET /v1/flows?status=error` — Liste des flux en erreur
- `GET /v1/stats` — Statistiques agrégées
- `GET /metrics` — Métriques Prometheus
- `GET /health` — Healthcheck
- `POST /v1/webhooks/ppf` — Webhook PPF (CDV entrants)

## Multi-tenancy

Chaque tenant (entreprise) est identifié par son **SIREN** et dispose de son propre répertoire :

```
tenants/
  123456789/              # SIREN
    config.yaml           # Identité, routes, PPF, AFNOR
    sequence.txt          # Compteur séquence PPF (persisté)
    certs/                # Certificats SFTP/TLS (optionnel)
  987654321/
    config.yaml
    sequence.txt
```

**Configuration racine** (`config.yaml`) :
```yaml
tenants_dir: tenants      # Active le mode multi-tenant

# Mapping token Bearer → SIREN (résolution HTTP)
token_tenant_map:
  "token-abc-123": "123456789"
  "token-xyz-789": "987654321"
```

**Sans `tenants_dir`** : la PDP fonctionne en mode mono-tenant (rétrocompatible).

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
| [docs/specifications.md](docs/specifications.md) | Spécifications techniques complètes (architecture, pipeline, formats, sécurité) |
| [docs/todo.md](docs/todo.md) | Roadmap : Typst, EndpointID, SFTP PPF, BR-FR, e-reporting, annuaire |

## Spécifications de référence

Toutes les spécifications sont dans le répertoire [`specs/`](specs/).

### Normes AFNOR

| Document | Description |
|----------|-------------|
| [XP Z12-012](specs/afnor/XP_Z12-012_Socle_technique.pdf) | Socle technique — formats, profils, règles de validation |
| [XP Z12-013](specs/afnor/XP_Z12-013_Interoperabilite.pdf) | Interopérabilité — échanges inter-PDP, API Flow/Directory |
| [XP Z12-014](specs/afnor/XP_Z12-014_Cas_usage.pdf) | Cas d'usage B2B — scénarios métier |

### Cas d'usage (XP Z12-014 Annexe A V1.3)

| Document | Description |
|----------|-------------|
| [Annexe A FR (PDF)](specs/use-cases/XP_Z12-014_Annexe_A_V1.3_FR.pdf) | 42 cas d'usage — version française |
| [Annexe A EN (PDF)](specs/use-cases/XP_Z12-014_Annexe_A_V1.3_EN.pdf) | 42 use cases — English version |
| [Annexe A FR (Markdown)](specs/use-cases/XP_Z12-014_Annexe_A_V1.3_FR.md) | Version texte exploitable |
| [Annexe A EN (Markdown)](specs/use-cases/XP_Z12-014_Annexe_A_V1.3_EN.md) | Searchable text version |
| [Exigences conformité](specs/use-cases/XP_Z12_Exigences_conformite.pdf) | Exigences et conformité XP Z12 |

### API Swagger (XP Z12-013)

| Document | Description |
|----------|-------------|
| [Flow Service V1.2.0](specs/swagger/ANNEXE_A_XP_Z12-013_Flow_Service_V1.2.0.json) | API d'échange de flux entre PDP |
| [Directory Service V1.2.0](specs/swagger/ANNEXE_B_XP_Z12-013_Directory_Service_V1.2.0.json) | API annuaire / découverte |

### PPF / Chorus Pro

| Document | Description |
|----------|-------------|
| [DSE Chorus Pro](specs/ppf/DSE_Chorus_Pro.pdf) | Dossier de spécifications externes Chorus Pro |
| [DSE Document général](specs/ppf/DSE_Document_general.pdf) | Spécifications externes — document général |

### Code lists et matrices

| Document | Description |
|----------|-------------|
| [EN16931 Codelists v16](specs/en16931-codelists-v16-fx1.08.xlsx) | Code lists EN16931 / Factur-X 1.08 |
| [Formats & profils Z12-012](specs/codelists/XP_Z12-012_Formats_profils_reference.xlsx) | Document maître formats et profils |
| [Règles métier EN16931](specs/codelists/Regles_metier_EN16931.xlsx) | Règles métier et code lists |
| [Statuts facture G2B/B2G](specs/codelists/Statuts_facture_G2B_B2G.xlsx) | Codes statuts facture |
| [Statuts CDV mapping](specs/codelists/Statuts_CDV_mapping.xlsx) | Mapping statuts cycle de vie |
| [Flux F1 UBL/CII](specs/codelists/Flux_F1_UBL_CII.xlsx) | Flux 1 — formats UBL et CII |
| [Flux F13/F14](specs/codelists/Flux_F13_F14.xlsx) | Configuration flux F13 et F14 |
| [E-Reporting correspondance](specs/codelists/E-Reporting_flux_correspondance.xlsx) | Correspondance flux e-reporting |

### Autres

| Document | Description |
|----------|-------------|
| [UN/CEFACT BRS](specs/uncefact/UNCEFACT_BRS_Update.pdf) | Business Requirements Specification Update |
| [Factur-X XMP schema](specs/facturx-extension-schema.xmp.txt) | Extension XMP pour Factur-X PDF/A-3 |

## Licence

Apache-2.0
