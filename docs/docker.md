# Conteneurisation (Docker / Podman)

Le projet est conteneurisé avec toutes les dépendances (Saxon-HE 12.9, qpdf, libxml2). La génération PDF utilise Typst (compilé en natif, aucune JRE requise).
La traçabilité et l'archivage utilisent **Elasticsearch** (un index par SIREN).

## Prérequis : Elasticsearch

```bash
# Lancer Elasticsearch en local (développement)
docker run -d --name pdp-es \
  -p 9200:9200 \
  -e "discovery.type=single-node" \
  -e "xpack.security.enabled=false" \
  -e "ES_JAVA_OPTS=-Xms512m -Xmx512m" \
  elasticsearch:8.15.0
```

## Builder et lancer les tests

```bash
# Docker (nécessite Elasticsearch)
docker build -t pdp-facture-test -f Dockerfile.test .
docker run --rm -e ELASTICSEARCH_URL=http://host.docker.internal:9200 pdp-facture-test

# Podman
podman build -t pdp-facture-test -f Dockerfile.test .
podman run --rm -e ELASTICSEARCH_URL=http://host.docker.internal:9200 pdp-facture-test

# Ou via docker-compose (lance ES automatiquement)
docker compose --profile test run pdp-test
```

## Builder l'image de production

```bash
docker build -t pdp-facture .
```

## Lancer le PDP

```bash
# Mode polling (continu)
docker run -d --name pdp \
  -e ELASTICSEARCH_URL=http://host.docker.internal:9200 \
  -v ./data/in:/app/data/in \       # accepte XML, tar.gz, zip (décompression auto)
  -v ./data/out:/app/data/out \
  -v ./config.yaml:/app/config.yaml:ro \
  pdp-facture start

# Exécution unique
docker run --rm \
  -e ELASTICSEARCH_URL=http://host.docker.internal:9200 \
  -v ./data/in:/app/data/in \
  -v ./data/out:/app/data/out \
  -v ./config.yaml:/app/config.yaml:ro \
  pdp-facture run
```

## Avec docker-compose

```bash
docker compose up -d elasticsearch              # Démarrer Elasticsearch
docker compose up -d pdp                        # Mode polling (attend ES healthy)
docker compose run --rm pdp-run                  # Exécution unique
docker compose --profile test run pdp-test       # Tests
docker compose --profile monitoring up -d kibana # Kibana (optionnel, http://localhost:5601)
```

## PEPPOL — Gateway Oxalis AS4

Pour tester les échanges inter-PDP via PEPPOL, le projet inclut deux instances Oxalis :

```bash
# Démarrer Oxalis (notre AP + AP distante simulée)
docker compose --profile peppol up -d

# Vérifier le statut
curl http://localhost:8080/oxalis/status    # Notre AP
curl http://localhost:8081/oxalis/status    # AP distante

# Logs
docker compose --profile peppol logs -f oxalis
```

### Architecture Docker PEPPOL

```
┌──────────┐     peppol-outbound     ┌──────────┐
│   PDP    │ ──────────────────────▶ │  Oxalis  │ ──AS4──▶ Réseau PEPPOL
│  (Rust)  │                         │  :8080   │
│          │ ◀────────────────────── │  :8443   │ ◀──AS4──
└──────────┘     peppol-inbound      └──────────┘

┌──────────────────┐
│  Oxalis Remote   │  (simule une PDP distante pour les tests)
│  :8081 / :8444   │
└──────────────────┘
```

### Volumes partagés

| Volume | Chemin PDP | Chemin Oxalis | Direction |
|--------|-----------|---------------|-----------|
| `peppol-outbound` | `/app/peppol/outbox` | `/oxalis/outbound` | PDP → Oxalis (envoi) |
| `peppol-inbound` | `/app/peppol/inbox` | `/oxalis/inbound` | Oxalis → PDP (réception) |

### Configuration Oxalis

Les fichiers de configuration sont dans `docker/oxalis/` :
- `oxalis.conf` — configuration principale (keystore, SML test, persistence filesystem)
- `logback.xml` — logging

## Services docker-compose

| Service | Image | Port | Description |
|---------|-------|------|-------------|
| `elasticsearch` | elasticsearch:8.15.0 | 9200 | Traçabilité + archivage (un index par SIREN) |
| `kibana` | kibana:8.15.0 | 5601 | Visualisation (profil `monitoring`, optionnel) |
| `pdp` | build local | — | PDP en mode polling |
| `pdp-run` | build local | — | Exécution unique (profil `run`) |
| `pdp-test` | build local | — | Tests (profil `test`) |
| `oxalis` | norstella/oxalis-as4:7.2.0 | 8080, 8443 | Gateway AS4 PEPPOL (profil `peppol`) |
| `oxalis-remote` | norstella/oxalis-as4:7.2.0 | 8081, 8444 | PDP distante simulée (profil `peppol`) |
