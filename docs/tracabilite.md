# Traçabilité et archivage — Elasticsearch

Le module `pdp-trace` assure la traçabilité complète des flux de facturation et l'archivage des documents (XML, PDF) dans **Elasticsearch**.

## Architecture

```
                    ┌─────────────────────────────────────────┐
                    │           Elasticsearch 8.x             │
                    │                                         │
  Exchange ──────▶  │  pdp-123456789   (SIREN vendeur A)     │
  (facture)         │  pdp-987654321   (SIREN vendeur B)     │
                    │  pdp-unknown     (SIREN non identifié) │
                    │                                         │
                    └─────────────────────────────────────────┘
```

**Un index par SIREN** : chaque client (identifié par son numéro SIREN = 9 premiers chiffres du SIRET vendeur) dispose de son propre index Elasticsearch `pdp-{siren}`.

Avantages :
- **Isolation des données** par client
- **Archivage naturel** : tout est dans ES (XML brut, PDF base64, métadonnées)
- **Recherche full-text** dans les XML de facturation
- **Scalabilité** : un index par client, sharding natif ES

## Document Elasticsearch

Chaque facture traitée produit un document ES avec la structure suivante :

| Champ | Type ES | Description |
|-------|---------|-------------|
| `exchange_id` | keyword | ID unique de l'exchange (UUID) |
| `flow_id` | keyword | ID du flux (regroupe les événements) |
| `source_filename` | keyword | Nom du fichier source |
| `invoice_number` | keyword | Numéro de facture (BT-1) |
| `invoice_key` | keyword | Clé métier SIREN/NUMERO/ANNEE |
| `seller_name` | text+keyword | Nom du vendeur |
| `buyer_name` | text+keyword | Nom de l'acheteur |
| `seller_siret` / `buyer_siret` | keyword | SIRET vendeur/acheteur |
| `seller_siren` / `buyer_siren` | keyword | SIREN (9 premiers chiffres) |
| `source_format` | keyword | UBL, CII, FacturX |
| `total_ht` / `total_ttc` / `total_tax` | double | Montants |
| `currency` | keyword | Devise (EUR) |
| `issue_date` | date | Date d'émission (BT-2) |
| `status` | keyword | Statut courant du flux |
| `error_count` | integer | Nombre d'erreurs |
| `raw_xml` | text | **XML brut complet** (searchable full-text) |
| `raw_pdf_base64` | binary | **PDF encodé en base64** |
| `attachment_count` | integer | Nombre de pièces jointes |
| `attachment_filenames` | keyword | Noms des PJ |
| `events` | nested | Événements de traitement (horodatés) |
| `errors` | nested | Erreurs (step, message, detail) |
| `created_at` / `updated_at` | date | Timestamps |

## Nommage des index

| SIRET vendeur | SIREN extrait | Index ES |
|---------------|---------------|----------|
| `12345678901234` | `123456789` | `pdp-123456789` |
| `98765432100028` | `987654321` | `pdp-987654321` |
| absent / invalide | — | `pdp-unknown` |

```rust
TraceStore::index_name("123456789")      // → "pdp-123456789"
TraceStore::siren_from_siret("12345678901234") // → Some("123456789")
```

## API TraceStore

### Connexion

```rust
// Connexion à Elasticsearch
let store = TraceStore::new("http://localhost:9200").await?;

// Ou via variable d'environnement ELASTICSEARCH_URL
let store = TraceStore::for_test().await?;
```

### Enregistrement

```rust
// Enregistrer un exchange complet (facture + XML + PDF + métadonnées)
// L'index est déterminé automatiquement par le SIREN vendeur
store.record_exchange(&exchange).await?;

// Enregistrer un événement de flux (ajouté au document existant)
let event = FlowEvent::new(flow_id, "parse", FlowStatus::Parsed, "Facture parsée");
store.record_event(&event).await?;
```

### Requêtes

```rust
// Statistiques globales (tous les index pdp-*)
let stats = store.get_stats().await?;
println!("Total: {}, Erreurs: {}, Distribués: {}",
    stats.total_exchanges, stats.total_errors, stats.total_distributed);

// Flux en erreur
let errors = store.get_error_flows().await?;

// Événements d'un flux
let events = store.get_flow_events(flow_id).await?;

// Recherche full-text dans les XML
let results = store.search_xml("FA-2025-001", None).await?;           // tous les index
let results = store.search_xml("FA-2025-001", Some("123456789")).await?; // un SIREN

// Récupérer un document complet (avec XML + PDF)
let doc = store.get_exchange("uuid-xxx", Some("123456789")).await?;

// Lister tous les SIREN connus
let sirens = store.list_sirens().await?;
```

## Intégration pipeline

Le `TraceProcessor` enregistre automatiquement chaque étape du pipeline :

```
Réception → TraceProcessor::received()
    │
Parsing → TraceProcessor::parsed()
    │
Validation → TraceProcessor::validated()
    │
Transformation → TraceProcessor::transformed()
    │
Distribution → TraceProcessor::distributed()
```

Chaque étape ajoute un événement horodaté dans le document ES de l'exchange.

## Configuration

```yaml
# config.yaml
elasticsearch:
  url: "http://localhost:9200"
```

Variable d'environnement : `ELASTICSEARCH_URL` (prioritaire sur le YAML).

## Kibana

Pour visualiser les données, activer Kibana via docker-compose :

```bash
docker compose --profile monitoring up -d kibana
# → http://localhost:5601
```

Dans Kibana :
1. **Management → Index Patterns** : créer `pdp-*`
2. **Discover** : explorer les factures par SIREN, statut, date
3. **Dashboard** : créer des tableaux de bord (volume, erreurs, top clients)

## Exemples de requêtes ES directes

```bash
# Nombre de factures par SIREN
curl -s 'localhost:9200/pdp-*/_count' | jq .count

# Recherche full-text dans les XML
curl -s 'localhost:9200/pdp-*/_search' -H 'Content-Type: application/json' -d '{
  "query": { "match": { "raw_xml": "FA-2025-001" } },
  "_source": ["invoice_number", "seller_name", "status"]
}'

# Factures en erreur
curl -s 'localhost:9200/pdp-*/_search' -H 'Content-Type: application/json' -d '{
  "query": { "range": { "error_count": { "gt": 0 } } },
  "sort": [{ "created_at": "desc" }]
}'

# Lister les index (= SIREN connus)
curl -s 'localhost:9200/_cat/indices/pdp-*?v'
```

## Tests

Les tests d'intégration nécessitent une instance Elasticsearch :

```bash
# Lancer ES pour les tests
docker run -d --name pdp-es -p 9200:9200 \
  -e "discovery.type=single-node" \
  -e "xpack.security.enabled=false" \
  elasticsearch:8.15.0

# Tests unitaires (pas besoin d'ES)
cargo test -p pdp-trace -- test_index_name test_siren_from_siret

# Tests d'intégration (nécessitent ES)
cargo test -p pdp-trace
```
