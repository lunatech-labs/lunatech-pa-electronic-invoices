# Installation et développement local

## Prérequis

- **Rust** 1.93+
- **Elasticsearch** 8.x (traçabilité + archivage, un index par SIREN)
- **libxml2-dev** + pkg-config (validation XSD)
- **SaxonC-HE** 12.9 natif (XSLT 2.0 : validation Schematron + transformation PDF)
  - Si `libsaxonc-he` est installée (`/usr/local/lib`), les transformations XSLT s'exécutent **in-process via FFI** (pas de fork/exec, ~25-30% plus rapide)
  - Sinon, fallback automatique vers le CLI `transform` ou `saxon`
- **Apache FOP** 2.11 (génération PDF Factur-X)
- **qpdf** (correction header binaire PDF/A)
- **veraPDF** (optionnel, validation PDF/A-3a)

```bash
# macOS
brew install pkgconf saxon fop qpdf
brew install verapdf  # optionnel, pour la validation PDF/A

# Debian/Ubuntu
apt-get install pkg-config libxml2-dev default-jre-headless qpdf
# Saxon et FOP : voir Dockerfile pour les URLs de téléchargement
```

## Elasticsearch

```bash
# Démarrer Elasticsearch via Docker (recommandé)
docker run -d --name pdp-es \
  -p 9200:9200 \
  -e "discovery.type=single-node" \
  -e "xpack.security.enabled=false" \
  -e "ES_JAVA_OPTS=-Xms512m -Xmx512m" \
  elasticsearch:8.15.0

# Ou via docker-compose
docker compose up -d elasticsearch

# Variable d'environnement (optionnel, défaut: http://localhost:9200)
export ELASTICSEARCH_URL=http://localhost:9200

# Kibana (optionnel, pour visualiser les index)
docker compose --profile monitoring up -d kibana
# → http://localhost:5601
```

## Build

```bash
cargo build --release
```

## Utilisation CLI

```bash
# Parser une facture
cargo run --bin pdp -- parse tests/fixtures/ubl/facture_ubl_001.xml

# Valider une facture
cargo run --bin pdp -- validate tests/fixtures/ubl/facture_ubl_001.xml

# Transformer UBL -> CII
cargo run --bin pdp -- transform tests/fixtures/ubl/facture_ubl_001.xml --to CII -o output.xml

# Exécuter toutes les routes
cargo run --bin pdp -- run

# Démarrer en mode polling
cargo run --bin pdp -- start

# Statistiques et erreurs
cargo run --bin pdp -- stats
cargo run --bin pdp -- errors
```

## Configuration

La configuration se fait via `config.yaml` :

```yaml
pdp:
  id: PDP-DEMO-001
  name: "PDP Démo Rust"

elasticsearch:
  url: "http://localhost:9200"

validation:
  specs_dir: ./specs          # ou $PDP_SPECS_DIR
  xsd_enabled: true
  en16931_enabled: true
  br_fr_enabled: true

peppol:
  ap_id: "POP000123"                    # CN du certificat Access Point
  participant_id: "0002::123456789"      # Notre identifiant PEPPOL (SIREN)
  endpoint_url: "https://pdp.example.com:443/as4"
  sml_url: "https://acc.edelivery.tech.ec.europa.eu/edelivery-sml"  # test
  certificate_path: "/etc/peppol/ap-cert.p12"
  certificate_password: "${PEPPOL_CERT_PASSWORD}"
  test_mode: true                        # false en production

routes:
  - id: route-ubl-reception
    source: { type: file, path: ./data/in/ubl }  # accepte XML, tar.gz, zip
    destination: { type: file, path: ./data/out/processed }
    validate: true
    generate_cdar: true
```

> **PEPPOL** : pour les échanges inter-PDP via le réseau PEPPOL AS4, voir [docs/peppol.md](peppol.md).
