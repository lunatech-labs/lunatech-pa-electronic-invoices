# Webhooks — AFNOR Flow Service (XP Z12-013 §5.4)

Ferrite expose les 5 endpoints standards de gestion des webhooks selon
**AFNOR XP Z12-013 V1.2.0** (Flow Service), permettant aux clients de
s'abonner aux événements de la PDP.

## Endpoints

| Méthode | Path | Réponse | Description |
|---------|------|---------|-------------|
| POST | `/v1/webhooks` | 201 Created | Crée un abonnement webhook |
| GET | `/v1/webhooks` | 200 OK | Liste les UIDs des webhooks |
| GET | `/v1/webhooks/{webhookId}` | 200 OK | Détails d'un webhook |
| PATCH | `/v1/webhooks/{webhookId}` | 204 No Content | Mise à jour partielle |
| DELETE | `/v1/webhooks/{webhookId}` | 204 No Content | Désabonnement |

Tous les endpoints sont **protégés par Bearer token** (mêmes règles que
`/v1/flows`). Authentification via `Authorization: Bearer <token>`.

## Modèle de données

### Création (POST /v1/webhooks)

```json
{
  "callback": {
    "url": "https://my-app.example.com/webhook",
    "headers": [
      { "headerName": "X-Custom-Header", "headerValue": "value" }
    ],
    "authentication": {
      "authType": "BASIC",
      "userId": "user",
      "userPassword": "pass"
    },
    "signature": {
      "algo": "HS256",
      "key": "<base64-encoded-secret>"
    }
  },
  "metadata": {
    "flowType": "CustomerInvoice",
    "flowDirection": "In",
    "processingRule": "STANDARD",
    "ackStatus": "Pending"
  }
}
```

**Réponse 201** :
```json
{ "webhookId": "550e8400-e29b-41d4-a716-446655440000" }
```

### Champs

#### `callback`
| Champ | Type | Obligatoire | Description |
|-------|------|-------------|-------------|
| `url` | string | OUI | URL HTTPS du callback (HTTP accepté en dev) |
| `headers` | array | NON | En-têtes HTTP personnalisés à injecter |
| `authentication` | object | NON | Auth du callback (BASIC ou OAUTH2) |
| `signature` | object | NON | Signature des payloads (HS256 supporté) |

#### `metadata` (filtres)
| Champ | Type | Obligatoire | Valeurs |
|-------|------|-------------|---------|
| `flowType` | string | OUI | `CustomerInvoice`, `SupplierInvoice`, `Cdar`, `EReporting` |
| `flowDirection` | string | OUI | `In` ou `Out` |
| `processingRule` | string | NON | Règle de traitement |
| `ackStatus` | string | NON | `Pending`, `Ok`, `Error` |

### Détails (GET /v1/webhooks/{webhookId})

```json
{
  "webhookId": "550e8400-e29b-41d4-a716-446655440000",
  "callback": { ... },
  "metadata": { ... }
}
```

### Liste (GET /v1/webhooks)

```json
{ "webhookIds": ["uuid1", "uuid2", "uuid3"] }
```

### Mise à jour (PATCH /v1/webhooks/{webhookId})

```json
{
  "headers": [{ "headerName": "X-New", "headerValue": "v" }],
  "authentication": { "authType": "BASIC", "userId": "u", "userPassword": "p" },
  "signature": { "algo": "HS256", "key": "<base64>" }
}
```

Les champs absents ne sont pas modifiés.

## Événements

| Événement | Déclencheur | Charge utile |
|-----------|-------------|--------------|
| `flow.received` | Réception de flux via `POST /v1/flows` | flowId, flowType, flowDirection, ackStatus |
| `flow.ack.updated` | Statut d'acquittement modifié | idem |

### Payload envoyé au callback

```json
{
  "event": "flow.received",
  "webhookId": "uuid",
  "flowId": "FLOW-12345",
  "flowType": "CustomerInvoice",
  "flowDirection": "In",
  "ackStatus": "Pending",
  "timestamp": "2026-04-26T14:30:00Z"
}
```

### En-têtes HTTP du callback

| Header | Valeur |
|--------|--------|
| `X-Webhook-Event` | Type d'événement (`flow.received`, `flow.ack.updated`) |
| `X-Webhook-Id` | UUID du webhook |
| `X-Webhook-Signature` | `sha256=<hex>` si signature `HS256` configurée |
| Headers personnalisés | Tels que définis dans `callback.headers` |

## Filtrage des abonnements

Quand un événement se produit, le `WebhookDispatcher` envoie le payload
**à tous les webhooks dont les `metadata` correspondent** :

- `flowType` : doit matcher exactement
- `flowDirection` : doit matcher exactement
- `ackStatus` : si défini sur le webhook, doit matcher l'événement ; si non défini, accepte tous les statuts

Exemple : un webhook avec `flowType=CustomerInvoice, flowDirection=In, ackStatus=null` recevra tous les événements `In` pour des `CustomerInvoice`, peu importe leur statut.

## Sécurité

### Signature HMAC-SHA256

Si `callback.signature.algo = "HS256"`, chaque requête contient un header :

```
X-Webhook-Signature: sha256=<hex>
```

Le client doit vérifier cette signature en calculant :

```
HMAC-SHA256(payload_body, base64_decode(signature.key))
```

### Authentification basique

Si `callback.authentication.authType = "BASIC"`, le header `Authorization: Basic <base64(user:pass)>` est ajouté.

### OAuth2

Réservé pour évolution future (champ supporté en lecture mais pas appliqué).

## Code de retour côté callback

Le webhook est considéré comme **livré avec succès** si la réponse HTTP est dans la plage `2xx`. Sinon, l'erreur est loguée mais ne bloque pas le pipeline.

**Pas de retry automatique** dans la version actuelle — le client doit gérer ses propres reprises sur erreurs.

## Storage

Les webhooks sont actuellement stockés **en mémoire** (HashMap protégé par `RwLock`). Cela signifie :

- Les abonnements sont **perdus au redémarrage** de la PDP
- Pas de partage entre instances (pas multi-nœud)

**Migration future** : passer à PostgreSQL pour persistance et partage multi-instance.

## Exemples curl

### Créer un webhook

```bash
curl -X POST http://localhost:8080/v1/webhooks \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{
    "callback": {"url": "https://my-app.example.com/webhook"},
    "metadata": {"flowType": "CustomerInvoice", "flowDirection": "In"}
  }'
```

### Lister

```bash
curl http://localhost:8080/v1/webhooks \
  -H "Authorization: Bearer <token>"
```

### Détails

```bash
curl http://localhost:8080/v1/webhooks/550e8400-e29b-41d4-a716-446655440000 \
  -H "Authorization: Bearer <token>"
```

### Mise à jour

```bash
curl -X PATCH http://localhost:8080/v1/webhooks/550e8400-e29b-41d4-a716-446655440000 \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{"headers": [{"headerName": "X-Env", "headerValue": "prod"}]}'
```

### Suppression

```bash
curl -X DELETE http://localhost:8080/v1/webhooks/550e8400-e29b-41d4-a716-446655440000 \
  -H "Authorization: Bearer <token>"
```

## Implémentation

| Composant | Fichier |
|-----------|---------|
| Modèles | `crates/pdp-app/src/webhooks.rs` (CallbackParameters, WebhookMetadata, etc.) |
| Store | `WebhookStore` (in-memory, thread-safe) |
| Handlers HTTP | `handle_create_webhook`, `handle_list_webhooks`, etc. |
| Dispatcher | `WebhookDispatcher` (reqwest, HMAC, headers) |
| Routes Axum | `crates/pdp-app/src/server.rs` (`build_api_router`) |
| Trigger flow.received | `handle_receive_flow` (tokio::spawn pour non-bloquant) |

## Tests

20+ tests dans `crates/pdp-app/src/webhooks.rs` (unit) et `server.rs` (intégration HTTP) :

- `test_store_create_and_get`, `test_store_list`, `test_store_update_partial`, `test_store_delete`, `test_store_get_not_found`
- `test_matching_filters`, `test_matching_with_ack_status`
- `test_event_type_str`
- `test_webhook_create_201`, `test_webhook_create_invalid_url`, `test_webhook_create_invalid_direction`
- `test_webhook_list_empty`, `test_webhook_list_after_create`
- `test_webhook_get_by_id`, `test_webhook_get_not_found`
- `test_webhook_patch_204`
- `test_webhook_delete_204`, `test_webhook_delete_not_found`

## Conformité

| Exigence XP Z12-013 V1.2.0 §5.4 | Statut |
|----------------------------------|--------|
| POST /v1/webhooks → 201 Created | ✅ |
| GET /v1/webhooks → liste UIDs | ✅ |
| GET /v1/webhooks/{uid} → détails | ✅ |
| PATCH /v1/webhooks/{uid} → 204 | ✅ |
| DELETE /v1/webhooks/{uid} → 204 | ✅ |
| Authentification Bearer | ✅ |
| Validation URL HTTPS | ⚠️ (HTTP accepté pour dev) |
| Validation flowType/flowDirection | ✅ |
| Filtrage par metadata | ✅ |
| Signature HMAC-SHA256 | ✅ |
| Auth BASIC sur callback | ✅ |
| Auth OAUTH2 sur callback | ⚠️ (modèle accepté, exécution future) |
| Persistance | ❌ (in-memory pour l'instant) |
| Headers `Request-Id` / `Organization-Id` | ❌ (à ajouter) |
| Retry sur erreur callback | ❌ (à ajouter) |
