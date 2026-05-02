# API HTTP — Référence et exemples curl

Ferrite expose une API REST conforme **AFNOR XP Z12-013 V1.2.0** (Flow Service
+ Directory Service) plus quelques endpoints techniques (santé, métriques,
interface annuaire). Cette page liste tous les endpoints avec des exemples
curl pour tester en local.

## Démarrer le serveur

```bash
# 1. Adapter config.yaml (cf. installation.md). Section minimale :
#
# http_server:
#   host: "0.0.0.0"
#   port: 8080
#   bearer_tokens: ["test-token-123"]   # auth Bearer (omettre = mode dev sans auth)
#   max_flow_size_bytes: 104857600       # 100 MB (défaut)
#   request_timeout_secs: 30             # 408 au-delà
#   rate_limit_per_minute: 100           # 429 au-delà (omettre = désactivé)

# 2. Démarrer
cargo run --bin pdp -- start --config config.yaml --mode receiver

# Le serveur écoute sur http://localhost:8080
```

## Authentification

Tous les endpoints `/v1/*` (sauf `/v1/healthcheck`, `/metrics`, `/annuaire`,
`/v1/annuaire/search`) sont **protégés par Bearer token** :

```bash
export TOKEN="test-token-123"
curl -H "Authorization: Bearer $TOKEN" http://localhost:8080/v1/flows
```

Si `bearer_tokens` n'est pas configuré, l'authentification est désactivée
(mode dev).

## Codes HTTP gérés (XP Z12-013 §5.5)

| Code | Quand |
|------|-------|
| `200 OK` | Lecture réussie |
| `201 Created` | Création (webhook) |
| `202 Accepted` | Flux accepté pour traitement asynchrone |
| `204 No Content` | Mise à jour ou suppression réussie |
| `400 Bad Request` | Payload invalide, headers manquants, SHA-256 erroné |
| `401 Unauthorized` | Token manquant ou invalide |
| `404 Not Found` | Ressource inconnue (flowId, webhookId, SIREN) |
| `408 Request Timeout` | Requête dépassant `request_timeout_secs` |
| `413 Payload Too Large` | Flux > `max_flow_size_bytes` |
| `429 Too Many Requests` | Quota dépassé — `Retry-After` indique le délai |
| `500 Internal Server Error` | Erreur interne (consulter logs) |
| `501 Not Implemented` | Fonction non disponible (TraceStore absent, etc.) |
| `503 Service Unavailable` | Pipeline indisponible |

## Endpoints publics

### `GET /v1/healthcheck`
```bash
curl http://localhost:8080/v1/healthcheck
# → {"status":"ok","version":"...","pdp_name":"...","pdp_matricule":"..."}
```

### `GET /metrics` — Prometheus
```bash
curl http://localhost:8080/metrics
# pdp_flows_received_total ...
# pdp_flows_accepted_total ...
# pdp_flows_rejected_total ...
# pdp_webhooks_received_total ...
```

### `GET /annuaire` — UI HTML
Interface de recherche annuaire PPF (HTML, accessible navigateur).

### `GET /v1/annuaire/search?q=...` — JSON public
```bash
curl "http://localhost:8080/v1/annuaire/search?q=LUNATECH&limit=10"
```

## Flow Service (XP Z12-013 §5.1-5.3)

### `POST /v1/flows` — Dépôt de flux entrant (multipart)

Soumet une facture, un CDV ou un e-reporting. Réponse `202 Accepted` avec un
`flowId` AFNOR.

```bash
# Préparer flowInfo.json
cat > flowInfo.json <<'EOF'
{
  "trackingId": "TRACK-20260502-001",
  "name": "facture_001.xml",
  "flowType": "CustomerInvoice",
  "flowSyntax": "UBL",
  "flowProfile": "EN16931",
  "processingRule": "STANDARD",
  "sha256": "a1b2c3..."
}
EOF

# Déposer
curl -X POST http://localhost:8080/v1/flows \
  -H "Authorization: Bearer $TOKEN" \
  -H "Request-Id: req-12345" \
  -H "Organization-Id: 123456789" \
  -F "flowInfo=@flowInfo.json;type=application/json" \
  -F "file=@tests/fixtures/ubl/facture_ubl_001.xml;type=application/xml"

# → 202 Accepted
# {
#   "flowId": "...",
#   "submittedAt": "2026-05-02T10:00:00Z",
#   "trackingId": "TRACK-20260502-001",
#   "name": "facture_001.xml",
#   "flowType": "CustomerInvoice",
#   "flowSyntax": "UBL",
#   "flowProfile": "EN16931",
#   "processingRule": "STANDARD",
#   "status": "RECEIVED",
#   "message": "..."
# }
```

**Headers** :
- `Authorization: Bearer <token>` — obligatoire si auth activée
- `Request-Id` — corrélation logs (echo dans la réponse)
- `Organization-Id` — SIREN du tenant (multi-tenant)

**Erreurs** :
- `400` payload invalide / SHA mismatch / flowInfo manquant / file manquant
- `413` taille > `max_flow_size_bytes`
- `503` pipeline saturé

### `GET /v1/flows?status=...&from=...&to=...` — Liste

```bash
curl -H "Authorization: Bearer $TOKEN" \
  "http://localhost:8080/v1/flows?status=error&from=2026-05-01&to=2026-05-02"
```

### `GET /v1/flows/{flowId}` — Détail

```bash
curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:8080/v1/flows/abc-123-def
```

### `GET /v1/stats` — Statistiques agrégées

```bash
curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:8080/v1/stats
# → {"totalExchanges":123,"totalErrors":4,"totalDistributed":119}
```

Retourne `501 Not Implemented` si `TraceStore` n'est pas configuré
(Elasticsearch).

## Webhooks (XP Z12-013 §5.4)

Voir [webhooks.md](webhooks.md) pour la documentation complète des
événements et payloads. Récap des endpoints :

```bash
# CRÉER un abonnement
curl -X POST http://localhost:8080/v1/webhooks \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "callback": {
      "url": "https://my-app.example.com/hook"
    },
    "metadata": {
      "flowType": "CustomerInvoice",
      "flowDirection": "In"
    }
  }'
# → 201 {"webhookId": "550e..."}

# LISTER
curl -H "Authorization: Bearer $TOKEN" http://localhost:8080/v1/webhooks
# → {"webhookIds": [...]}

# DÉTAIL
curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:8080/v1/webhooks/550e...

# MISE À JOUR (PATCH partiel)
curl -X PATCH http://localhost:8080/v1/webhooks/550e... \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"headers": [{"headerName": "X-API-Key", "headerValue": "..."}]}'
# → 204 No Content

# SUPPRIMER
curl -X DELETE http://localhost:8080/v1/webhooks/550e... \
  -H "Authorization: Bearer $TOKEN"
# → 204 No Content
```

### `POST /v1/webhooks/callback` — Réception PPF

Endpoint où le PPF pousse les CDV entrants. Vérification HMAC-SHA256
si `webhook_secret` est configuré.

## Directory Service (XP Z12-013 Annexe B)

Recherche et résolution dans l'annuaire PPF (nécessite PostgreSQL +
ingestion préalable du F14 — voir [annuaire.md](annuaire.md)).

### SIREN

```bash
# Détail SIREN
curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:8080/v1/siren/code-insee:123456789

# Recherche SIREN (multi-critères)
curl -X POST http://localhost:8080/v1/siren/search \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"name": "LUNATECH"}'
```

### SIRET

```bash
curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:8080/v1/siret/code-insee:12345678901234

curl -X POST http://localhost:8080/v1/siret/search \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"siren": "123456789"}'
```

### Codes de routage

```bash
# Recherche
curl -X POST http://localhost:8080/v1/routing-code/search \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"siret": "12345678901234"}'

# Code spécifique
curl -H "Authorization: Bearer $TOKEN" \
  "http://localhost:8080/v1/routing-code/siret:12345678901234/code:0224ABC"
```

### Lignes annuaire

```bash
curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:8080/v1/directory-line/code:ABC-123

curl -X POST http://localhost:8080/v1/directory-line/search \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"siren": "123456789"}'
```

### Endpoints internes annuaire

```bash
# Stats globales
curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:8080/v1/annuaire/stats

# Liste des PDP (plateformes)
curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:8080/v1/annuaire/plateformes
```

## Tester les codes d'erreur HTTP

### 401 Unauthorized
```bash
curl -i http://localhost:8080/v1/flows         # sans token
curl -i -H "Authorization: Bearer wrong" \
  http://localhost:8080/v1/flows               # mauvais token
```

### 404 Not Found
```bash
curl -i -H "Authorization: Bearer $TOKEN" \
  http://localhost:8080/v1/flows/inexistant
```

### 413 Payload Too Large
```bash
# Configurer max_flow_size_bytes: 1024 dans config.yaml puis :
dd if=/dev/urandom of=big.xml bs=1k count=10
curl -i -X POST http://localhost:8080/v1/flows \
  -H "Authorization: Bearer $TOKEN" \
  -F "flowInfo=@flowInfo.json" \
  -F "file=@big.xml"
# → HTTP/1.1 413 Payload Too Large
```

### 408 Request Timeout
```bash
# Configurer request_timeout_secs: 1 puis envoyer un gros flux qui prend
# du temps à traiter
```

### 429 Too Many Requests
```bash
# Configurer rate_limit_per_minute: 5 puis :
for i in {1..10}; do
  curl -i -H "Authorization: Bearer $TOKEN" \
    http://localhost:8080/v1/webhooks
done
# → la 6e requête retourne 429 avec header Retry-After
```

## Workflow complet de test (5 minutes)

```bash
# 1. Démarrer
cargo run --bin pdp -- start --config config.yaml --mode receiver &

# 2. Healthcheck
curl http://localhost:8080/v1/healthcheck

# 3. Préparer flowInfo
SHA=$(shasum -a 256 tests/fixtures/ubl/facture_ubl_001.xml | cut -d' ' -f1)
cat > /tmp/flowInfo.json <<EOF
{
  "trackingId": "TEST-001",
  "name": "facture_test.xml",
  "flowType": "CustomerInvoice",
  "flowSyntax": "UBL",
  "sha256": "$SHA"
}
EOF

# 4. Déposer
curl -X POST http://localhost:8080/v1/flows \
  -H "Authorization: Bearer test-token-123" \
  -F "flowInfo=@/tmp/flowInfo.json;type=application/json" \
  -F "file=@tests/fixtures/ubl/facture_ubl_001.xml;type=application/xml"

# 5. Créer un webhook
curl -X POST http://localhost:8080/v1/webhooks \
  -H "Authorization: Bearer test-token-123" \
  -H "Content-Type: application/json" \
  -d '{
    "callback": {"url": "https://webhook.site/xxx"},
    "metadata": {"flowType": "CustomerInvoice", "flowDirection": "In"}
  }'

# 6. Métriques
curl http://localhost:8080/metrics | grep pdp_flows
```

## CLI alternative (sans HTTP)

Pour un test rapide sans serveur HTTP :

```bash
# Parser
cargo run --bin pdp -- parse tests/fixtures/ubl/facture_ubl_001.xml

# Valider
cargo run --bin pdp -- validate tests/fixtures/ubl/facture_ubl_001.xml

# Transformer UBL ↔ CII
cargo run --bin pdp -- transform tests/fixtures/ubl/facture_ubl_001.xml \
  --to CII -o /tmp/converted.xml

# Exécuter une route configurée
cargo run --bin pdp -- run-route route-ubl-reception
```

## Voir aussi

- [tests.md](tests.md) — Tests automatisés (1052 tests workspace)
- [webhooks.md](webhooks.md) — Détail des événements et payloads webhook
- [annuaire.md](annuaire.md) — Ingestion F14 et schéma annuaire
- [installation.md](installation.md) — Build, config, prérequis
