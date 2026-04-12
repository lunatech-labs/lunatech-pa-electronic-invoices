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
let cdar    = DocumentTypeId::cdar();               // CDAR D22B

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
| CDAR | `urn:un:unece:...:CrossDomainAcknowledgementAndResponse:100::...::D23B` | CDAR D22B |

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

## Docker — Environnement PEPPOL local

Le projet inclut un setup Docker Compose complet pour tester les échanges PEPPOL AS4 en local, avec un SMP (annuaire) et deux Access Points Oxalis.

```bash
# Démarrer tous les services PEPPOL
podman compose --profile peppol up -d smp oxalis oxalis-remote

# Enregistrer les participants dans le SMP
bash ./docker/peppol-setup.sh

# Vérifier les statuts
curl http://localhost:8888/public   # SMP
curl http://localhost:8080/status   # Oxalis PDP_A
curl http://localhost:8081/status   # Oxalis PDP_B

# Arrêter
podman compose --profile peppol down
```

### Architecture complète

```
                        docker compose --profile peppol
 ┌─────────────────────────────────────────────────────────────────────────┐
 │                                                                         │
 │  ┌─────────────────────┐                                                │
 │  │   phoss-smp (SMP)   │  Annuaire des participants                     │
 │  │   :8888 → :8080     │  - Enregistre PDP_B (0002:987654321)           │
 │  │   phelger/phoss-     │  - Endpoint → oxalis-remote:8080/as4          │
 │  │   smp-xml:latest     │  - Certificat AP de test                      │
 │  └────────┬────────────┘                                                │
 │           │ lookup (StaticLocator)                                       │
 │           │                                                              │
 │  ┌────────▼────────────┐         AS4/HTTP          ┌──────────────────┐ │
 │  │  oxalis (PDP_A)     │ ════════════════════════▶ │ oxalis-remote    │ │
 │  │  Access Point envoi │     SBDH + UBL Invoice    │ (PDP_B)          │ │
 │  │  :8080 (HTTP)       │                           │ Access Point     │ │
 │  │  :8443 (HTTPS/AS4)  │                           │ réception        │ │
 │  └────────┬────────────┘                           │ :8081 → :8080    │ │
 │           │                                         │ :8444 → :8443    │ │
 │           │ volumes Docker                          └────────┬─────────┘ │
 │           │                                                  │           │
 │  ┌────────▼──────────┐                            ┌─────────▼────────┐  │
 │  │ peppol-outbound   │  PDP → Oxalis (envoi)      │ peppol-remote-   │  │
 │  │ peppol-inbound    │  Oxalis → PDP (réception)  │ inbound          │  │
 │  └────────┬──────────┘                            │ (messages reçus) │  │
 │           │                                        └──────────────────┘  │
 │           │                                                              │
 └───────────┼──────────────────────────────────────────────────────────────┘
             │
    ┌────────▼──────────┐
    │   PDP (Rust)      │  Notre application
    │   FilesystemGW    │  - Dépose SBDH dans outbox
    │   ou RestGW       │  - Lit SBDH depuis inbox
    │                   │  - PeppolSendProcessor
    │                   │  - PeppolReceiveProcessor
    └───────────────────┘
```

### Services

| Service | Image | Ports | Rôle |
|---------|-------|-------|------|
| **smp** | `phelger/phoss-smp-xml` | `:8888` | Annuaire SMP local (lookup participants → endpoints) |
| **oxalis** | `norstella/oxalis-as4:7.2.0` | `:8080`, `:8443` | Access Point PDP_A (envoi AS4) |
| **oxalis-remote** | `norstella/oxalis-as4:7.2.0` | `:8081`, `:8444` | Access Point PDP_B (réception AS4) |

### Volumes

| Volume | Usage |
|--------|-------|
| `peppol-outbound` | PDP dépose des SBDH XML → Oxalis les envoie via AS4 |
| `peppol-inbound` | Oxalis reçoit via AS4 → PDP lit les SBDH XML |
| `peppol-remote-inbound` | Messages reçus par PDP_B (oxalis-remote) |
| `smp-data` | Données persistantes du SMP (participants, endpoints) |

### Flux d'envoi AS4 (PDP_A → PDP_B)

```
1. PDP Rust construit un PeppolMessage + SBDH
2. FilesystemGateway.send() dépose le SBDH dans peppol-outbound/
3. Oxalis (PDP_A) lit le SBDH
4. Oxalis consulte le SMP : "où envoyer pour 0002:987654321 ?"
5. SMP répond : "oxalis-remote:8080/as4" + certificat
6. Oxalis (PDP_A) envoie le message AS4 à oxalis-remote
7. Oxalis-remote (PDP_B) reçoit et dépose dans peppol-remote-inbound/
```

### Configuration

- **Oxalis** : `docker/oxalis/oxalis.conf` — StaticLocator vers SMP local, keystore test
- **SMP** : configuré via `docker/peppol-setup.sh` (REST API, participants + endpoints)
- **Keystore** : `docker/oxalis/oxalis-keystore.jks` — certificat auto-signé RSA 2048 (test uniquement)

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

7 tests d'intégration simulant l'envoi d'une facture entre deux PDP via le réseau PEPPOL :

| Test | Docker | Description |
|------|--------|-------------|
| `test_envoi_facture_pdp_a_vers_pdp_b_filesystem` | Non | Roundtrip complet : UBL → SBDH → `FilesystemGateway` → `PeppolReceiveProcessor` → vérifie ID, contenu, statut `Received` |
| `test_sbdh_roundtrip_avec_vraie_facture` | Non | Build SBDH + parse : vérifie sender/receiver/document_type/process_id + payload intact |
| `test_envoi_cdar_pdp_a_vers_pdp_b_filesystem` | Non | Même roundtrip avec un CDV (CDAR) — vérifie détection type `CDAR` |
| `test_oxalis_rest_gateway_health` | Oui | Health check Oxalis REST (conditionnel, `OXALIS_URL`) |
| `test_oxalis_envoi_facture_via_rest_gateway` | Oui | Envoi via REST gateway Oxalis (conditionnel, `OXALIS_URL`) |
| `test_as4_envoi_reel_vers_oxalis_remote` | Oui | **Envoi AS4 réel** : `As4Client` → SOAP+MIME → `oxalis-remote:8081/as4` |
| `test_smp_lookup_participant` | Oui | **Lookup SMP** : vérifie que PDP_B (0002:987654321) est enregistré dans phoss-smp |

#### Exécution sans Docker (toujours disponible)

Les 3 premiers tests utilisent le `FilesystemGateway` et ne nécessitent aucune infrastructure :

```bash
cargo test -p pdp-peppol --test peppol_integration
```

#### Exécution avec Docker (SMP + Oxalis)

Les 4 derniers tests nécessitent l'environnement PEPPOL complet :

```bash
# 1. Démarrer SMP + Oxalis PDP_A + Oxalis PDP_B
podman compose --profile peppol up -d smp oxalis oxalis-remote

# 2. Attendre le démarrage (~40s) puis enregistrer les participants
sleep 40 && bash ./docker/peppol-setup.sh

# 3. Exécuter tous les tests (7/7)
OXALIS_URL=http://localhost:8080 cargo test -p pdp-peppol --test peppol_integration

# 4. Arrêter
podman compose --profile peppol down
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
- SBDH contient les scopes DOCUMENTID, PROCESSID et COUNTRY_C1
- Le payload (facture UBL ou CDAR) est transmis intact
- `PeppolReceiveProcessor` détecte le bon type de document
- Le statut passe à `FlowStatus::Received`
- L'acquittement supprime le message de l'inbox

### Benchmark AS4

Performances mesurées en local (Docker, même machine) avec `oxalis-standalone` 7.2.0 :

| Mode | Durée (10 msg) | Moy/msg | Débit |
|------|----------------|---------|-------|
| **Séquentiel** (1 JVM par envoi) | 17.5s | 1752ms | 0.5 msg/s |
| **Parallèle** (`--repeat`, 1 JVM) | 2.2s | 991ms | **10 msg/s** |
| **Concurrent** (10 JVM shell) | 54.8s | 5481ms | 0.1 msg/s |

- **Facture** : 7 KB (UBL), 8.4 KB avec enveloppe SBDH
- **Transport** : AS4 signé (WS-Security XML-DSIG) via HTTP
- **Persistance** : fichier XML dans `/oxalis/inbound/` du récepteur

Le mode **parallèle** (pool de threads interne Oxalis) est optimal : **~1s par message, 10 msg/s**.
Le séquentiel est pénalisé par le démarrage JVM (~650ms) à chaque invocation.
Le concurrent shell est le pire car 10 JVM lourdes saturent le CPU.

```bash
# Exécuter le benchmark (N = nombre de messages, défaut 10)
bash ./docker/bench-as4.sh 10
```

---

## Spécifications détaillées pour l'implémentation Rust (remplacement Oxalis)

Les sections suivantes documentent les aspects techniques nécessaires pour implémenter un Access Point Peppol complet en Rust, sans dépendance à Oxalis ou à la JVM.

### Protocole AS4 — ebMS 3.0

#### Enveloppe SOAP 1.2

Chaque message AS4 est un multipart MIME contenant :
- **Part 1** : enveloppe SOAP 1.2 (XML) avec les headers ebMS et WS-Security
- **Part 2** : le payload (SBDH + facture ou CDAR), compressé en gzip

Structure de l'enveloppe SOAP :

```xml
<soap:Envelope xmlns:soap="http://www.w3.org/2003/05/soap-envelope"
               xmlns:eb="http://docs.oasis-open.org/ebxml-msg/ebms/v3.0/ns/core/200704/"
               xmlns:wsse="http://docs.oasis-open.org/wss/2004/01/oasis-200401-wss-wssecurity-secext-1.0.xsd"
               xmlns:wsu="http://docs.oasis-open.org/wss/2004/01/oasis-200401-wss-wssecurity-utility-1.0.xsd">
  <soap:Header>
    <eb:Messaging>
      <eb:UserMessage>
        <eb:MessageInfo>
          <eb:Timestamp>2025-01-15T10:30:00.000Z</eb:Timestamp>
          <eb:MessageId>unique-uuid@sender-ap.example.com</eb:MessageId>
        </eb:MessageInfo>
        <eb:PartyInfo>
          <eb:From><eb:PartyId type="urn:fdc:peppol.eu:2017:identifiers:ap">POP000123</eb:PartyId>
                   <eb:Role>http://docs.oasis-open.org/ebxml-msg/ebms/v3.0/ns/core/200704/initiator</eb:Role></eb:From>
          <eb:To><eb:PartyId type="urn:fdc:peppol.eu:2017:identifiers:ap">POP000456</eb:PartyId>
                 <eb:Role>http://docs.oasis-open.org/ebxml-msg/ebms/v3.0/ns/core/200704/responder</eb:Role></eb:To>
        </eb:PartyInfo>
        <eb:CollaborationInfo>
          <eb:AgreementRef>urn:fdc:peppol.eu:2017:agreements:tia:ap_provider</eb:AgreementRef>
          <eb:Service type="urn:fdc:peppol.eu:2017:identifiers:proc-id">
            urn:fdc:peppol.eu:2017:poacc:billing:01:1.0
          </eb:Service>
          <eb:Action>busdox-docid-qns::...</eb:Action>
          <eb:ConversationId>conversation-uuid</eb:ConversationId>
        </eb:CollaborationInfo>
        <eb:PayloadInfo>
          <eb:PartInfo href="cid:sbdh-payload">
            <eb:PartProperties>
              <eb:Property name="MimeType">application/xml</eb:Property>
              <eb:Property name="CompressionType">application/gzip</eb:Property>
            </eb:PartProperties>
          </eb:PartInfo>
        </eb:PayloadInfo>
      </eb:UserMessage>
    </eb:Messaging>
    <wsse:Security soap:mustUnderstand="true">
      <!-- Voir section WS-Security ci-dessous -->
    </wsse:Security>
  </soap:Header>
  <soap:Body/>
</soap:Envelope>
```

#### Multipart MIME

```
Content-Type: multipart/related; boundary="boundary-uuid"; type="application/soap+xml"

--boundary-uuid
Content-Type: application/soap+xml; charset=UTF-8
Content-Transfer-Encoding: binary

[Enveloppe SOAP XML]

--boundary-uuid
Content-Type: application/octet-stream
Content-Transfer-Encoding: binary
Content-Id: <sbdh-payload>

[Payload gzippé : SBDH + facture/CDAR]

--boundary-uuid--
```

#### Message Exchange Pattern (MEP)

Peppol utilise le MEP **One-Way/Push** :
1. L'émetteur (C2) envoie un `UserMessage` via HTTP POST
2. Le récepteur (C3) répond avec un `SignalMessage` (receipt ou error) dans la réponse HTTP
3. Pas de polling — c'est du push pur

#### MessageId

- Format : `{uuid}@{ap-domain}` (ex: `a7f3c2e1-4b5d-6789-abcd-ef0123456789@pdp.example.com`)
- Doit être **globalement unique** (UUID v4 recommandé)
- Sert de clé d'idempotence pour la déduplication côté récepteur

#### Receipts (accusés de réception)

Quand un AP reçoit un message, il doit répondre avec un `SignalMessage` :

```xml
<eb:Messaging>
  <eb:SignalMessage>
    <eb:MessageInfo>
      <eb:Timestamp>2025-01-15T10:30:01.000Z</eb:Timestamp>
      <eb:MessageId>receipt-uuid@receiver-ap.example.com</eb:MessageId>
      <eb:RefToMessageId>original-message-uuid@sender-ap.example.com</eb:RefToMessageId>
    </eb:MessageInfo>
    <eb:Receipt>
      <ebbp:NonRepudiationInformation>
        <ebbp:MessagePartNRInformation>
          <ds:Reference URI="cid:sbdh-payload">
            <ds:DigestMethod Algorithm="http://www.w3.org/2001/04/xmlenc#sha256"/>
            <ds:DigestValue>base64-du-hash-sha256</ds:DigestValue>
          </ds:Reference>
        </ebbp:MessagePartNRInformation>
      </ebbp:NonRepudiationInformation>
    </eb:Receipt>
  </eb:SignalMessage>
</eb:Messaging>
```

#### Signaux d'erreur AS4

En cas d'erreur, le récepteur répond avec un `eb:Error` au lieu d'un receipt :

| Code | Catégorie | Description |
|------|-----------|-------------|
| `EBMS:0001` | Content | Pas trouvé |
| `EBMS:0002` | Content | Format invalide |
| `EBMS:0003` | Content | Erreur de décompression |
| `EBMS:0004` | Content | Payload non conforme |
| `EBMS:0010` | Processing | Processing mode mismatch |
| `EBMS:0011` | Processing | Opération non supportée |
| `EBMS:0101` | Security | Échec de vérification de signature |
| `EBMS:0102` | Security | Échec de déchiffrement |
| `EBMS:0103` | Security | Certificat non fiable/expiré |
| `EBMS:0301` | Communication | Erreur de livraison (timeout) |
| `EBMS:0302` | Communication | MessageId dupliqué |
| `EBMS:0303` | Communication | Erreur de décompression |

### WS-Security & Signature XML-DSIG

#### Structure du header Security

```xml
<wsse:Security soap:mustUnderstand="true">
  <wsu:Timestamp wsu:Id="_ts">
    <wsu:Created>2025-01-15T10:30:00.000Z</wsu:Created>
    <wsu:Expires>2025-01-15T10:35:00.000Z</wsu:Expires>
  </wsu:Timestamp>
  <wsse:BinarySecurityToken
      wsu:Id="_bst"
      ValueType="http://docs.oasis-open.org/wss/2004/01/oasis-200401-wss-x509-token-profile-1.0#X509v3"
      EncodingType="http://docs.oasis-open.org/wss/2004/01/oasis-200401-wss-soap-message-security-1.0#Base64Binary">
    [Certificat X.509 en base64]
  </wsse:BinarySecurityToken>
  <ds:Signature>
    <ds:SignedInfo>
      <ds:CanonicalizationMethod Algorithm="http://www.w3.org/2001/10/xml-exc-c14n#"/>
      <ds:SignatureMethod Algorithm="http://www.w3.org/2001/04/xmldsig-more#rsa-sha256"/>
      <ds:Reference URI="#_messaging">
        <ds:Transforms><ds:Transform Algorithm="http://www.w3.org/2001/10/xml-exc-c14n#"/></ds:Transforms>
        <ds:DigestMethod Algorithm="http://www.w3.org/2001/04/xmlenc#sha256"/>
        <ds:DigestValue>...</ds:DigestValue>
      </ds:Reference>
      <ds:Reference URI="#_body">
        <ds:Transforms><ds:Transform Algorithm="http://www.w3.org/2001/10/xml-exc-c14n#"/></ds:Transforms>
        <ds:DigestMethod Algorithm="http://www.w3.org/2001/04/xmlenc#sha256"/>
        <ds:DigestValue>...</ds:DigestValue>
      </ds:Reference>
      <ds:Reference URI="#_ts">
        <ds:Transforms><ds:Transform Algorithm="http://www.w3.org/2001/10/xml-exc-c14n#"/></ds:Transforms>
        <ds:DigestMethod Algorithm="http://www.w3.org/2001/04/xmlenc#sha256"/>
        <ds:DigestValue>...</ds:DigestValue>
      </ds:Reference>
      <ds:Reference URI="cid:sbdh-payload">
        <ds:Transforms><ds:Transform Algorithm="http://docs.oasis-open.org/wss/oasis-wss-SwAProfile-1.1#Attachment-Content-Signature-Transform"/></ds:Transforms>
        <ds:DigestMethod Algorithm="http://www.w3.org/2001/04/xmlenc#sha256"/>
        <ds:DigestValue>...</ds:DigestValue>
      </ds:Reference>
    </ds:SignedInfo>
    <ds:SignatureValue>...</ds:SignatureValue>
    <ds:KeyInfo>
      <wsse:SecurityTokenReference>
        <wsse:Reference URI="#_bst"/>
      </wsse:SecurityTokenReference>
    </ds:KeyInfo>
  </ds:Signature>
</wsse:Security>
```

#### Éléments signés

La signature DOIT couvrir :
1. `eb:Messaging` (header ebMS — identité des parties, action, message ID)
2. `soap:Body` (vide mais signé par intégrité)
3. `wsu:Timestamp` (anti-replay)
4. Le payload SBDH (via `cid:` reference — Attachment-Content-Signature-Transform)

#### Algorithmes requis

| Usage | Algorithme | URI |
|-------|------------|-----|
| Canonicalisation | Exclusive C14N | `http://www.w3.org/2001/10/xml-exc-c14n#` |
| Signature | RSA-SHA256 | `http://www.w3.org/2001/04/xmldsig-more#rsa-sha256` |
| Digest | SHA-256 | `http://www.w3.org/2001/04/xmlenc#sha256` |
| Attachment | SwA Profile 1.1 | `oasis-wss-SwAProfile-1.1#Attachment-Content-Signature-Transform` |

#### Validation de signature à la réception

1. Extraire le `BinarySecurityToken` (certificat X.509)
2. Vérifier la chaîne de confiance (voir PKI ci-dessous)
3. Vérifier l'expiration du certificat
4. Recalculer les digests de chaque `Reference` (C14N + SHA-256)
5. Vérifier la `SignatureValue` avec la clé publique du certificat
6. Vérifier le `Timestamp` (Created < now < Expires, tolérance de 5 min)

### PKI Peppol — Chaîne de certificats

#### Hiérarchie

```
OpenPeppol Root CA (auto-signé, durée 20 ans)
  └── Peppol Intermediate CA (AP) (durée 5 ans)
        └── Certificat Access Point (CN = AP_ID, durée 2 ans)
```

En test (pilotage/ACC) :
```
Peppol Pilot Root CA
  └── Peppol Pilot AP CA
        └── Certificat AP de test
```

#### Gestion des certificats

| Tâche | Détail |
|-------|--------|
| **Format** | PKCS#12 (.p12/.pfx) contenant clé privée + certificat + chaîne CA |
| **CN** | Doit correspondre au `AP_ID` enregistré chez OpenPeppol (ex: `POP000123`) |
| **Validation** | Chaîne complète : AP cert → Intermediate CA → Root CA |
| **Expiration** | Le certificat AP expire tous les 2 ans — renouvellement auprès d'OpenPeppol |
| **Révocation** | CRL téléchargeable depuis le certificat (extension CRL Distribution Points) |
| **Stockage** | Keystore protégé par mot de passe, fichier avec permissions 600 |
| **Truststore** | Contient les CA Peppol (Root + Intermediate) pour valider les certificats distants |

#### Validation d'un certificat distant

À la réception d'un message AS4, valider :
1. Le certificat est signé par la Peppol Intermediate CA
2. L'Intermediate CA est signée par la Peppol Root CA
3. Le certificat n'est pas expiré (`notBefore ≤ now ≤ notAfter`)
4. Le certificat n'est pas révoqué (vérification CRL ou OCSP)
5. Le CN correspond à un AP_ID valide

#### Implémentation Rust

Crates recommandées pour la PKI :
- **`rustls`** — TLS sans OpenSSL
- **`webpki`** — validation de chaîne de certificats X.509
- **`p12`** ou **`pkcs12`** — lecture des keystores PKCS#12
- **`x509-parser`** — parsing des certificats X.509
- **`ring`** — RSA-SHA256, HMAC, hashing (bas niveau, performant)

### Enregistrement SMP

Le SMP (Service Metadata Publisher) est un annuaire REST qui associe un participant à ses capacités de réception.

#### Lookup (déjà implémenté)

```
1. Résoudre l'URL du SMP via le SML (DNS)
   → B-{md5(participantId)}.iso6523-actorid-upis.{sml-zone}
   → CNAME vers l'URL du SMP hébergeant ce participant

2. GET /iso6523-actorid-upis::{scheme}::{id}/services/{docTypeId}
   → ServiceMetadata XML avec l'endpoint URL + certificat de l'AP récepteur
```

#### Registration (à implémenter)

Pour que notre AP reçoive des messages, il faut :

1. **Enregistrer le participant dans le SMP** :
```
PUT /iso6523-actorid-upis::{scheme}::{id}
Content-Type: application/xml

<ServiceGroup>
  <ParticipantIdentifier scheme="iso6523-actorid-upis">{scheme}::{id}</ParticipantIdentifier>
</ServiceGroup>
```

2. **Publier les capacités de réception** (pour chaque type de document supporté) :
```
PUT /iso6523-actorid-upis::{scheme}::{id}/services/{docTypeId}
Content-Type: application/xml

<ServiceMetadata>
  <ServiceInformation>
    <ParticipantIdentifier scheme="iso6523-actorid-upis">{scheme}::{id}</ParticipantIdentifier>
    <DocumentIdentifier scheme="busdox-docid-qns">{docTypeId}</DocumentIdentifier>
    <ProcessList>
      <Process>
        <ProcessIdentifier scheme="cenbii-procid-ubl">{processId}</ProcessIdentifier>
        <ServiceEndpointList>
          <Endpoint transportProfile="peppol-transport-as4-v2_0">
            <EndpointURI>https://pdp.example.com/as4</EndpointURI>
            <Certificate>[certificat AP en base64]</Certificate>
            <ServiceActivationDate>2025-01-01T00:00:00Z</ServiceActivationDate>
            <ServiceExpirationDate>2027-01-01T00:00:00Z</ServiceExpirationDate>
          </Endpoint>
        </ServiceEndpointList>
      </Process>
    </ProcessList>
  </ServiceInformation>
</ServiceMetadata>
```

3. **Authentification SMP** : HTTP Basic ou certificat client TLS selon l'opérateur SMP

#### Multi-SIREN

Un AP peut enregistrer **plusieurs participants** (SIRENs) dans le SMP. Chaque SIREN a son propre `ServiceGroup` mais pointe vers le même endpoint URL (notre AP). L'AP doit router les messages reçus vers le bon tenant en inspectant le `Receiver` dans le SBDH.

### Routage dual PPF / Peppol

La PDP doit décider pour chaque facture : **réseau français (PPF/AFNOR)** ou **réseau Peppol** ?

#### Matrice de routage

| Émetteur | Destinataire | Réseau | Raison |
|----------|-------------|--------|--------|
| France (SIREN) | France (SIREN) sur même PDP | Local | Pas besoin de réseau externe |
| France (SIREN) | France (SIREN) sur autre PDP | PPF + AFNOR | Réseau domestique obligatoire |
| France (SIREN) | France (SIREN) sur Peppol uniquement | Peppol | Si la PDP du destinataire n'est pas sur AFNOR |
| France (SIREN) | UE (hors France) | Peppol | International |
| France (SIREN) | Administration (B2G) | PPF (Chorus Pro) | Obligation légale |
| UE (hors France) | France (SIREN) | Peppol → notre AP | Réception internationale |

#### Algorithme de routage

```
fn resolve_network(invoice) -> Network {
    let buyer_country = invoice.buyer_country;
    let buyer_siret = invoice.buyer_siret;

    // 1. B2G → toujours PPF (Chorus Pro)
    if is_public_entity(buyer_siret) {
        return Network::PPF;
    }

    // 2. International → Peppol
    if buyer_country != "FR" {
        return Network::Peppol;
    }

    // 3. France : chercher dans l'annuaire PISTE
    if let Some(pdp) = annuaire.lookup(buyer_siret) {
        // Le destinataire est sur une PDP connue
        if pdp.supports_afnor() {
            return Network::Afnor(pdp.matricule);
        }
    }

    // 4. Fallback : chercher dans le SMP Peppol
    if let Ok(endpoint) = smp.lookup(buyer_participant_id) {
        return Network::Peppol;
    }

    // 5. Aucun réseau trouvé → erreur
    return Network::Unknown;
}
```

#### Obligations réglementaires françaises

- **2026-09-01** : obligation de réception des factures électroniques pour toutes les entreprises
- **2027-09-01** : obligation d'émission pour les GE et ETI
- **2028-09-01** : obligation d'émission pour les PME et TPE
- Les factures B2G passent déjà par Chorus Pro (portail PPF)
- Les factures B2B domestiques doivent transiter par le PPF ou une PDP immatriculée
- Peppol est utilisable pour les échanges internationaux et comme réseau complémentaire

### Gestion des erreurs et retry

#### Stratégie de retry

| Type d'erreur | Retryable ? | Stratégie |
|---------------|-------------|-----------|
| HTTP 503 Service Unavailable | Oui | Backoff exponentiel, max 5 tentatives |
| HTTP 500 Internal Server Error | Oui | Backoff exponentiel, max 3 tentatives |
| HTTP 4xx (sauf 408) | Non | Erreur permanente → dead letter |
| HTTP 408 Request Timeout | Oui | Retry immédiat, max 2 tentatives |
| Timeout connexion TCP | Oui | Retry avec délai, max 3 tentatives |
| `EBMS:0101` Signature invalide | Non | Erreur permanente → alerte critique |
| `EBMS:0103` Certificat non fiable | Non | Erreur permanente → vérifier les certificats |
| `EBMS:0302` MessageId dupliqué | Non | Ignoré (le message a déjà été livré) |
| `EBMS:0301` Erreur de livraison | Oui | Retry avec backoff, max 5 tentatives |

#### Backoff exponentiel

```
delay(attempt) = min(initial_delay × multiplier^attempt, max_delay)

Paramètres par défaut :
  initial_delay = 5s
  multiplier = 2.0
  max_delay = 300s (5 min)
  max_retries = 5
```

Séquence : 5s → 10s → 20s → 40s → 80s (total ~2.5 min avant abandon)

#### Déduplication des messages

Côté récepteur, chaque `MessageId` reçu doit être mémorisé pendant **7 jours** minimum pour détecter les retransmissions. Implémentation :
- En mémoire (HashMap avec TTL) pour le mode dev
- En Elasticsearch ou base de données pour la production

### Persistance et fiabilité

#### Exactly-once delivery

Peppol ne garantit pas nativement l'exactly-once delivery. L'implémentation doit assurer :

1. **Écriture atomique dans l'outbox** — utiliser `write()` + `rename()` pour éviter les fichiers partiels
2. **Déduplication à la réception** — vérifier le `MessageId` avant traitement
3. **Acquittement après traitement** — ne supprimer de l'inbox qu'après succès complet du pipeline
4. **Journalisation** — chaque envoi/réception est tracé dans Elasticsearch avec le `MessageId`

#### Convention de nommage des fichiers

```
Outbox : {timestamp}_{messageId}_{sender}_{receiver}.sbdh.xml
Inbox  : {timestamp}_{messageId}_{sender}_{receiver}.sbdh.xml
```

#### Quarantaine

Les messages qui échouent au parsing SBDH ou à la validation de signature sont déplacés dans un répertoire `quarantine/` avec un rapport `.alert.json` (intégration avec le système d'alertes de la PDP).

### Monitoring et observabilité

#### Métriques à collecter

| Métrique | Type | Description |
|----------|------|-------------|
| `peppol.messages.sent.total` | Counter | Messages envoyés (par statut : success, error) |
| `peppol.messages.received.total` | Counter | Messages reçus (par type : invoice, credit_note, cdar) |
| `peppol.send.latency_ms` | Histogram | Latence d'envoi AS4 (p50, p95, p99) |
| `peppol.smp.lookup.latency_ms` | Histogram | Latence des lookups SMP |
| `peppol.smp.lookup.errors` | Counter | Erreurs de lookup SMP |
| `peppol.retry.count` | Counter | Nombre de retries (par type d'erreur) |
| `peppol.deadletter.count` | Counter | Messages en dead letter |
| `peppol.outbox.pending` | Gauge | Messages en attente d'envoi |
| `peppol.inbox.pending` | Gauge | Messages en attente de traitement |
| `peppol.certificate.days_remaining` | Gauge | Jours avant expiration du certificat AP |

#### Alertes

| Alerte | Seuil | Sévérité |
|--------|-------|----------|
| Taux d'erreur envoi > 10% | Sur 5 min | Critical |
| Certificat AP expire dans < 30 jours | Quotidien | Warning |
| Certificat AP expire dans < 7 jours | Quotidien | Critical |
| SMP lookup failures > 5 consécutifs | Immédiat | Critical |
| Dead letter queue > 10 messages | Sur 1h | Warning |
| Outbox pending > 100 messages | Sur 15 min | Warning |

### Performances cibles (implémentation Rust)

| Métrique | Oxalis (Java) | Cible Rust | Gain attendu |
|----------|--------------|------------|--------------|
| Latence envoi (p50) | ~1000ms | < 200ms | 5x |
| Latence envoi (p99) | ~2500ms | < 500ms | 5x |
| Débit soutenu | 10 msg/s | 100 msg/s | 10x |
| Mémoire | ~512 MB (JVM) | < 50 MB | 10x |
| Démarrage | ~5s (JVM) | < 100ms | 50x |
| Connexions concurrentes | ~100 | ~10 000 | 100x |

Les gains sont attendus grâce à :
- Pas de JVM (démarrage instantané, pas de GC)
- Async I/O natif (tokio, pas de thread pool Java)
- Zéro-copy pour le parsing XML (roxmltree)
- Réutilisation des connexions TCP (connection pooling hyper)

### Multi-tenancy Peppol

Un seul AP Peppol peut servir plusieurs tenants (SIRENs). Considérations :

| Aspect | Approche |
|--------|----------|
| **Certificat** | Un seul certificat AP (CN = AP_ID). Tous les tenants partagent l'AP. |
| **SMP** | Un `ServiceGroup` par SIREN, tous pointant vers le même endpoint URL |
| **Réception** | Router le message vers le bon tenant en inspectant le `Receiver` dans le SBDH |
| **Outbox/Inbox** | `tenants/{siren}/peppol/outbox/` et `tenants/{siren}/peppol/inbox/` |
| **Isolation** | Les données sont séparées par tenant mais le transport est mutualisé |
| **Ajout/suppression** | Dynamique : enregistrer/désenregistrer dans le SMP via l'API REST |

### Migration Oxalis → Rust

#### Phases de migration

```
Phase 1 : Mode shadow
  Rust AP reçoit une copie des messages mais ne répond pas
  Oxalis reste le AP principal
  → Valider le parsing, la vérification de signature, le routage

Phase 2 : Mode canary
  Rust AP traite un pourcentage du trafic (10%, 25%, 50%)
  Oxalis traite le reste
  → Valider la fiabilité en conditions réelles

Phase 3 : Mode principal
  Rust AP est le AP principal
  Oxalis en standby pour rollback si nécessaire
  → SMP mis à jour pour pointer vers le Rust AP

Phase 4 : Décommissionnement Oxalis
  Rust AP seul
  → Supprimer les containers Docker Oxalis
```

#### Compatibilité

- **Keystores** : lire les fichiers PKCS#12 existants (même format qu'Oxalis)
- **Inbox/Outbox** : même convention de répertoires pour migration sans perte
- **SMP** : mise à jour de l'endpoint URL (même AP_ID, nouvelle URL)
- **Certificat** : réutilisable (le certificat est lié à l'AP_ID, pas à l'implémentation)

### Conformité OpenPeppol

#### Spécifications de référence

| Spécification | Version | Lien |
|---------------|---------|------|
| Peppol AS4 Profile | v2.0 | [docs.peppol.eu/edelivery/as4/spec](https://docs.peppol.eu/edelivery/as4/spec/) |
| CEF eDelivery AS4 | v1.14 | [ec.europa.eu/digital-building-blocks](https://ec.europa.eu/digital-building-blocks/) |
| Peppol Envelope (SBDH) | v1.2 | [docs.peppol.eu/edelivery/envelope](https://docs.peppol.eu/edelivery/envelope/) |
| OASIS BDXR SMP | v1.0 | [docs.oasis-open.org/bdxr](http://docs.oasis-open.org/bdxr/) |
| Peppol BIS Billing | v3.0 | [docs.peppol.eu/poacc/billing](https://docs.peppol.eu/poacc/billing/) |
| UN/CEFACT SBDH | v1.3 | [unece.org/trade/uncefact](https://unece.org/trade/uncefact) |

#### Certification AP

Pour être un Access Point Peppol officiel :
1. Adhérer à OpenPeppol (ou via une autorité Peppol nationale — en France : AIFE/DGFiP)
2. Obtenir un certificat AP signé par la Peppol CA
3. Passer les tests d'interopérabilité (envoi/réception avec d'autres AP)
4. Enregistrer l'AP dans le réseau de production
5. Renouveler l'adhésion et le certificat annuellement

### Crates Rust recommandées

| Besoin | Crate | Rôle |
|--------|-------|------|
| HTTP client/serveur | `hyper` + `axum` | Transport AS4 |
| TLS | `rustls` | Chiffrement transport |
| XML parsing | `roxmltree` | Parsing SOAP, SBDH, réponses SMP |
| XML writing | `quick-xml` | Construction SOAP, SBDH |
| XML C14N | `xmlsec` ou custom | Canonicalisation pour signature |
| XML-DSIG | `xmlsec` ou custom | Signature et vérification |
| PKCS#12 | `p12` | Lecture des keystores |
| X.509 | `x509-parser` | Parsing et validation des certificats |
| RSA | `ring` ou `rsa` | Opérations cryptographiques |
| SHA-256 | `ring` | Hashing |
| gzip | `flate2` | Compression/décompression payload |
| MIME multipart | `multer` ou custom | Parsing/construction MIME |
| DNS | `trust-dns` | Résolution SML |
| UUID | `uuid` | Génération de MessageId |
