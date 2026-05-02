# Collection Bruno — Ferrite PDP HTTP API

Collection de tests HTTP pour [Bruno](https://www.usebruno.com/) (alternative
file-based à Postman, versionnable en git).

## Installer Bruno

```bash
brew install bruno          # macOS
# Ou télécharger : https://www.usebruno.com/downloads
```

## Ouvrir la collection

1. Lancer Bruno
2. **Open Collection** → sélectionner ce dossier (`docs/bruno-collection`)
3. Choisir l'environnement **local** (en haut à droite)

## Variables (environnement `local`)

Éditer `environments/local.bru` :

| Variable | Défaut | Description |
|----------|--------|-------------|
| `baseUrl` | `http://localhost:8080` | URL du serveur |
| `token` | `test-token-123` | Bearer token (doit être dans `bearer_tokens` de `config.yaml`) |
| `siren` | `123456789` | SIREN de test |
| `siret` | `12345678901234` | SIRET de test |
| `flowId` | (auto) | Rempli par `Submit Flow` |
| `webhookId` | (auto) | Rempli par `Create Webhook` |

## Workflow de test recommandé

1. **`00-Health/Healthcheck`** — vérifie que le serveur tourne
2. **`01-Flow/Submit Flow`** — dépose une facture (capture `flowId`)
3. **`01-Flow/Get Flow by ID`** — relit le flux
4. **`02-Webhooks/Create Webhook`** — crée un abonnement (capture `webhookId`)
5. **`02-Webhooks/Update Webhook`** → **`Get`** → **`Delete`** — cycle complet
6. **`04-Errors/*`** — vérifie 401, 404

## Lancer toute la collection en CLI

```bash
npm install -g @usebruno/cli
bru run --env local
```
