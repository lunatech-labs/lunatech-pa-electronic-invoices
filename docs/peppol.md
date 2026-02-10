# PEPPOL AS4 — Échanges inter-PDP

Module `pdp-peppol` : envoi et réception de factures et CDAR entre PDP via le réseau PEPPOL.

## Architecture 4 coins

```
Émetteur (C1) → Access Point (C2) ──[AS4/SBDH]──→ Access Point (C3) → Destinataire (C4)
  (vendeur)       (notre PDP)                        (PDP distante)      (acheteur)
                       ↕                                   ↕
                   SML/SMP (découverte dynamique des endpoints)
```

Notre PDP s'intègre avec un **gateway AS4 externe** (Oxalis, phase4) qui gère
la conformité protocolaire (WS-Security, signature XML, PKI PEPPOL, TLS).

```
                    ┌──────────────────────────────────────────────────┐
                    │                  Notre PDP                       │
                    │                                                  │
  Fichiers/SFTP ──▶ │  Pipeline (parse → validate → transform)        │
                    │       │                                          │
                    │       ▼                                          │
                    │  ┌──────────┐     ┌─────────────────────┐       │
                    │  │pdp-peppol│◄───►│ Gateway AS4 (Oxalis)│◄──AS4──► Autres PDP
                    │  │  SBDH    │     │ WS-Security, PKI    │       │
                    │  │  SMP     │     │ Signature XML       │       │
                    │  └──────────┘     └─────────────────────┘       │
                    └──────────────────────────────────────────────────┘
```

## Gateway AS4

Le protocole AS4 PEPPOL est complexe (WS-Security, signature XML, PKI,
compression, receipts). Plutôt que de le réimplémenter, on délègue à un
gateway certifié :

| Gateway | Langage | Licence | Lien |
|---------|---------|---------|------|
| **Oxalis** | Java | EUPL | [github.com/OxalisCommunity/oxalis](https://github.com/OxalisCommunity/oxalis) |
| **phase4** | Java | Apache 2.0 | [github.com/phax/phase4](https://github.com/phax/phase4) |

### Modes d'intégration

#### 1. Mode filesystem (recommandé)

```
Envoi :     Pipeline → SBDH XML → outbox/ → Oxalis poll → AS4 → PDP distante
Réception : PDP distante → AS4 → Oxalis → inbox/ → Pipeline poll
```

#### 2. Mode API REST

```
Envoi :     Pipeline → POST /api/send (SBDH XML) → Gateway → AS4 → PDP distante
Réception : PDP distante → AS4 → Gateway → inbox/ ou webhook → Pipeline
```

## Standards et protocoles

| Standard | Rôle |
|----------|------|
| **AS4** (ebMS 3.0) | Transport SOAP/MIME entre Access Points (One-Way/Push) |
| **SBDH** | Standard Business Document Header — enveloppe de routage |
| **SMP** | Service Metadata Publisher — annuaire des participants |
| **SML** | Service Metadata Locator — DNS des SMP |
| **PKI Peppol** | Certificats X.509 pour signature et chiffrement |
| **TLS 1.2+** | Sécurité transport (port 443 obligatoire) |

## Modules

### `model` — Identifiants et messages

```rust
use pdp_peppol::model::*;

// Identifiant participant (SIREN, SIRET, endpoint français)
let sender = ParticipantId::from_siren("111111111");
let receiver = ParticipantId::from_siret("22222222200015");
let endpoint = ParticipantId::from_french_endpoint("123456789_FACTURES");

// Types de documents PEPPOL
let ubl_inv = DocumentTypeId::ubl_invoice();       // UBL 2.1 Invoice
let ubl_cn  = DocumentTypeId::ubl_credit_note();   // UBL 2.1 CreditNote
let cii     = DocumentTypeId::cii_invoice();        // CII D16B
let cdar    = DocumentTypeId::cdar();               // CDAR D23B

// Processus
let billing = ProcessId::billing();  // Peppol BIS Billing 3.0
```

#### Schémas d'identification

| Schéma | Type | Exemple |
|--------|------|---------|
| `0002` | SIREN | `0002::123456789` |
| `0009` | SIRET | `0009::12345678901234` |
| `0225` | Endpoint français | `0225::123456789_FACTURES` |
| `0088` | EAN/GLN | `0088::7300010000001` |

### `sbdh` — Enveloppe SBDH

Le SBDH encapsule le document métier (facture ou CDAR) avec les métadonnées de routage.

```rust
use pdp_peppol::{sbdh, model::*};

// Construction
let msg = PeppolMessage::ubl_invoice(sender, receiver, xml_bytes);
let sbdh_xml = sbdh::build_sbdh(&msg);

// Parsing
let parsed = sbdh::parse_sbdh(&sbdh_xml).unwrap();
println!("Sender: {}", parsed.sender);
println!("Payload: {}", &parsed.payload[..100]);
```

Structure XML produite :

```xml
<StandardBusinessDocument>
  <StandardBusinessDocumentHeader>
    <HeaderVersion>1.0</HeaderVersion>
    <Sender><Identifier Authority="0002">111111111</Identifier></Sender>
    <Receiver><Identifier Authority="0002">222222222</Identifier></Receiver>
    <DocumentIdentification>...</DocumentIdentification>
    <BusinessScope>
      <Scope><Type>DOCUMENTID</Type>...</Scope>
      <Scope><Type>PROCESSID</Type>...</Scope>
    </BusinessScope>
  </StandardBusinessDocumentHeader>
  <!-- Facture UBL/CII ou CDAR ici -->
</StandardBusinessDocument>
```

### `smp` — Découverte dynamique

Le SMP résout un participant + type de document en endpoint AS4 (URL + certificat).

```rust
use pdp_peppol::smp::SmpClient;
use pdp_peppol::model::*;

let smp = SmpClient::test();  // ou SmpClient::production()

// Résolution DNS SML
let host = smp.resolve_smp_host(&receiver);
// → "B-<hash>.iso6523-actorid-upis.acc.edelivery.tech.ec.europa.eu"

// Lookup complet
let result = smp.lookup(&receiver, &DocumentTypeId::ubl_invoice(), &ProcessId::billing()).await?;
println!("Endpoint: {}", result.endpoint.endpoint_url);
println!("Certificat: {}", &result.endpoint.certificate[..50]);
```

#### Environnements SML

| Environnement | Zone SML |
|---------------|----------|
| **Test** | `acc.edelivery.tech.ec.europa.eu` |
| **Production** | `edelivery.tech.ec.europa.eu` |

### `gateway` — Intégration avec Oxalis / phase4

#### Mode filesystem (recommandé)

```rust
use pdp_peppol::gateway::FilesystemGateway;
use pdp_peppol::model::*;

let gw = FilesystemGateway::new("/var/peppol/outbox", "/var/peppol/inbox");

// Envoi : dépose un SBDH XML dans outbox/ → Oxalis le transmet via AS4
let msg = PeppolMessage::ubl_invoice(sender, receiver, xml);
let filename = gw.send(&msg)?;

// Réception : lit les SBDH XML déposés par Oxalis dans inbox/
let messages = gw.receive()?;
for msg in &messages {
    println!("De {} : {}", msg.sender, msg.payload.len());
    gw.acknowledge(&msg.filename)?;  // supprime après traitement
}
```

#### Mode API REST

```rust
use pdp_peppol::gateway::RestGateway;

let gw = RestGateway::new("http://localhost:8080");

// Vérifier la santé du gateway
let ok = gw.health_check().await?;

// Envoyer via l'API REST du gateway
let result = gw.send(&message).await?;
```

### `as4` — Utilitaires AS4

Module bas niveau pour la construction/parsing de messages AS4 (SOAP, MIME, SBDH).
Utilisé en interne par le gateway et les processors. Contient aussi les constantes
P-Mode PEPPOL (Agreement, MEP, Party type, MPC).

### `processor` — Intégration pipeline

#### PeppolSendProcessor

Envoie une facture ou un CDAR vers une autre PDP. Résout automatiquement :
- **Sender** : depuis `peppol.sender` ou `invoice.seller_siret`
- **Receiver** : depuis `peppol.receiver` ou `invoice.buyer_siret`
- **Document type** : UBL Invoice/CreditNote, CII, ou CDAR selon le contenu

```rust
use pdp_peppol::{PeppolSendProcessor, PeppolConfig};

let processor = PeppolSendProcessor::new(PeppolConfig::test());
// S'insère dans le pipeline après transformation
```

Propriétés ajoutées à l'exchange :

| Propriété | Description |
|-----------|-------------|
| `peppol.message_id` | ID du message AS4 envoyé |
| `peppol.endpoint_url` | URL de l'AP destinataire |
| `peppol.status` | `sent` ou `error` |
| `peppol.error` | Message d'erreur (si échec) |
| `peppol.timestamp` | Horodatage de l'envoi |

#### PeppolReceiveProcessor

Traite un message AS4 entrant et l'injecte dans le pipeline.

```rust
use pdp_peppol::PeppolReceiveProcessor;

let processor = PeppolReceiveProcessor::new();
// S'insère en début de pipeline pour les messages PEPPOL entrants
```

Propriétés lues/ajoutées :

| Propriété | Description |
|-----------|-------------|
| `peppol.sender` | Identifiant de l'émetteur (entrée) |
| `peppol.receiver` | Identifiant du destinataire (entrée) |
| `peppol.message_id` | ID du message AS4 (entrée) |
| `peppol.received` | `true` (sortie) |
| `peppol.document_type` | `Invoice`, `CreditNote` ou `CDAR` (sortie) |

## Configuration

```yaml
peppol:
  ap_id: "POP000123"                    # CN du certificat AP
  participant_id: "0002::123456789"      # Notre identifiant PEPPOL
  endpoint_url: "https://pdp.example.com:443/as4"
  sml_url: "https://edelivery.tech.ec.europa.eu/edelivery-sml"
  certificate_path: "/etc/peppol/ap-cert.p12"
  certificate_password: "${PEPPOL_CERT_PASSWORD}"
  truststore_path: "/etc/peppol/peppol-ca.jks"
  test_mode: false
```

## Documents supportés

| Type | DocumentTypeId | Format |
|------|---------------|--------|
| Facture UBL | `urn:oasis:...:Invoice-2::Invoice##...billing:3.0::2.1` | UBL 2.1 |
| Avoir UBL | `urn:oasis:...:CreditNote-2::CreditNote##...billing:3.0::2.1` | UBL 2.1 |
| Facture CII | `urn:un:unece:...:CrossIndustryInvoice:100::...::D16B` | CII D16B |
| CDAR | `urn:un:unece:...:CrossDomainAcknowledgementAndResponse:100::...::D23B` | CDAR D23B |

## Flux de données

### Envoi d'une facture

```
InvoiceData → XML (UBL/CII) → Pipeline
    │
    ├─ 1. Résoudre sender/receiver (SIREN → ParticipantId)
    ├─ 2. Construire SBDH (enveloppe)
    └─ 3. Déposer dans outbox/ (ou POST API gateway)
              │
              └─ Gateway AS4 (Oxalis) → AS4 → PDP distante
```

### Réception d'une facture

```
PDP distante → AS4 → Gateway AS4 (Oxalis) → inbox/
    │
    ├─ 1. Lire SBDH XML depuis inbox/
    ├─ 2. Parser SBDH (sender, receiver, payload)
    ├─ 3. Extraire le document métier
    ├─ 4. Injecter dans le pipeline → Parsing → Validation → ...
    └─ 5. Acquitter (supprimer ou archiver le fichier)
```

### Envoi d'un CDAR d'irrecevabilité

```
ReceptionProcessor (erreur) → IrrecevabiliteProcessor (CDAR 501)
    → FilesystemGateway.send() → outbox/ → Oxalis → AS4 → PDP émettrice
```

## Docker — Oxalis AS4

Le projet inclut un setup Docker Compose avec Oxalis pour tester les échanges PEPPOL :

```bash
# Démarrer Oxalis (profil peppol)
docker compose --profile peppol up -d

# Vérifier le statut
curl http://localhost:8080/status

# Tout démarrer (ES + PDP + Oxalis)
docker compose --profile peppol up -d elasticsearch pdp oxalis
```

### Architecture

```
┌──────────┐     peppol-outbound     ┌──────────┐
│   PDP    │ ──────────────────────▶ │  Oxalis  │ ──AS4──▶ Réseau PEPPOL
│  (Rust)  │                         │  :8080   │
│          │ ◀────────────────────── │  :8443   │ ◀──AS4──
└──────────┘     peppol-inbound      └──────────┘
```

- **`peppol-outbound`** : PDP dépose des SBDH XML → Oxalis les envoie via AS4
- **`peppol-inbound`** : Oxalis reçoit via AS4 → PDP lit les SBDH XML
- **`oxalis-remote`** (`:8081`) : simule une PDP distante pour les tests inter-PDP

Configuration Oxalis : `docker/oxalis/oxalis.conf` (SML test, keystore test, persistence filesystem).

Voir [docs/docker.md](docker.md) pour plus de détails.

## Tests

### Tests unitaires

51 tests (49 unit + 2 doc-tests) couvrant :

- **model** (12) : ParticipantId, DocumentTypeId, ProcessId, PeppolMessage, PeppolConfig
- **sbdh** (6) : build, parse, roundtrip, extraction doc info
- **smp** (7) : résolution DNS, parsing réponse XML, environnements
- **as4** (7) : SOAP envelope, MIME multipart, parsing, receipts, errors, roundtrip
- **gateway** (8) : filesystem send/receive/roundtrip/archive, REST, filtrage non-XML/SBDH invalide
- **processor** (7) : détection type document, réception, résolution sender/receiver
- **doc-tests** (2) : exemples de la documentation

```bash
cargo test -p pdp-peppol
```

### Tests d'intégration PEPPOL (inter-PDP)

5 tests d'intégration simulant l'envoi d'une facture entre deux PDP via le réseau PEPPOL :

| Test | Description |
|------|-------------|
| `test_envoi_facture_pdp_a_vers_pdp_b_filesystem` | Roundtrip complet : charge une facture UBL → `PeppolMessage` → SBDH → `FilesystemGateway` outbox → inbox → `PeppolReceiveProcessor` → vérifie ID, contenu, métadonnées, statut `Received` |
| `test_sbdh_roundtrip_avec_vraie_facture` | Build SBDH + parse : vérifie sender/receiver/document_type/process_id + payload intact |
| `test_envoi_cdar_pdp_a_vers_pdp_b_filesystem` | Même roundtrip avec un CDV (CDAR) — vérifie détection type `CDAR` |
| `test_oxalis_rest_gateway_health` | Health check Oxalis REST (conditionnel, `OXALIS_URL`) |
| `test_oxalis_envoi_facture_via_rest_gateway` | Envoi via REST gateway Oxalis (conditionnel, `OXALIS_URL`) |

#### Exécution sans Docker (toujours disponible)

Les 3 premiers tests utilisent le `FilesystemGateway` et ne nécessitent aucune infrastructure :

```bash
cargo test -p pdp-peppol --test peppol_integration
```

#### Exécution avec Docker Oxalis

Les 2 derniers tests nécessitent un gateway Oxalis accessible :

```bash
# 1. Démarrer Oxalis
docker compose --profile peppol up -d

# 2. Vérifier qu'Oxalis est prêt
curl http://localhost:8080/status

# 3. Exécuter tous les tests (y compris REST gateway)
OXALIS_URL=http://localhost:8080 cargo test -p pdp-peppol --test peppol_integration

# 4. Arrêter Oxalis
docker compose --profile peppol down
```

#### Flux testé

```
PDP_A (émetteur)                          PDP_B (destinataire)
┌─────────────────┐                       ┌─────────────────┐
│ 1. Charger UBL  │                       │                 │
│ 2. PeppolMessage│                       │                 │
│ 3. SBDH build   │                       │                 │
│ 4. Gateway.send │──── outbox/ ────────▶│ 5. Gateway.recv │
│    (outbox)     │  (volume partagé /   │    (inbox)      │
│                 │   transport AS4)      │ 6. ReceiveProc  │
│                 │                       │ 7. Vérification │
└─────────────────┘                       └─────────────────┘
```

Vérifications effectuées :
- SBDH contient les bons sender/receiver (SIREN)
- SBDH contient les scopes DOCUMENTID et PROCESSID
- Le payload (facture UBL ou CDAR) est transmis intact
- `PeppolReceiveProcessor` détecte le bon type de document
- Le statut passe à `FlowStatus::Received`
- L'acquittement supprime le message de l'inbox
