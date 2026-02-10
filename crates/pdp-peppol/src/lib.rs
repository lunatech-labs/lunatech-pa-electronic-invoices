//! # pdp-peppol — Access Point PEPPOL AS4
//!
//! Module d'envoi et de réception de factures et CDAR entre PDP via le réseau
//! PEPPOL, conforme au profil AS4 v2.0 et à l'architecture 4 coins.
//!
//! # Architecture 4 coins
//!
//! ```text
//! Émetteur (C1) → Access Point (C2) ──[AS4/SBDH]──→ Access Point (C3) → Destinataire (C4)
//!                      ↕                                    ↕
//!                  SML/SMP (découverte dynamique des endpoints)
//! ```
//!
//! Notre PDP joue le rôle de **C2** (envoi) et **C3** (réception).
//!
//! # Composants
//!
//! - **[`model`]** — Identifiants PEPPOL (`ParticipantId`, `DocumentTypeId`, `ProcessId`),
//!   messages (`PeppolMessage`), configuration (`PeppolConfig`)
//! - **[`sbdh`]** — Construction et parsing du Standard Business Document Header (enveloppe)
//! - **[`smp`]** — Client SMP pour la découverte dynamique des endpoints AS4
//! - **[`as4`]** — Client AS4 (envoi SOAP/MIME) et parsing des messages entrants
//! - **[`processor`]** — Processors pipeline : [`PeppolSendProcessor`], [`PeppolReceiveProcessor`]
//! - **[`error`]** — Types d'erreurs PEPPOL
//!
//! # Protocoles et standards
//!
//! | Standard | Usage |
//! |----------|-------|
//! | AS4 (ebMS 3.0) | Transport SOAP/MIME entre Access Points |
//! | SBDH | Enveloppe du document métier (routing metadata) |
//! | SMP/SML | Découverte dynamique des endpoints |
//! | PKI Peppol | Certificats X.509 pour signature et chiffrement |
//! | TLS 1.2+ | Sécurité transport (port 443) |
//!
//! # Exemple : envoyer une facture
//!
//! ```no_run
//! use pdp_peppol::model::*;
//! use pdp_peppol::as4::As4Client;
//! use pdp_peppol::smp::SmpClient;
//! use pdp_peppol::sbdh;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let sender = ParticipantId::from_siren("111111111");
//! let receiver = ParticipantId::from_siren("222222222");
//! let xml = std::fs::read("facture.xml")?;
//!
//! // 1. Créer le message
//! let message = PeppolMessage::ubl_invoice(sender, receiver.clone(), xml);
//!
//! // 2. Découvrir l'endpoint du destinataire via SMP
//! let smp = SmpClient::test();
//! let lookup = smp.lookup(&receiver, &message.document_type_id, &message.process_id).await?;
//!
//! // 3. Envoyer via AS4
//! let config = PeppolConfig::test();
//! let client = As4Client::new(config);
//! let result = client.send(&message, &lookup.endpoint).await?;
//! println!("Envoyé : {} (succès: {})", result.message_id, result.success);
//! # Ok(())
//! # }
//! ```
//!
//! # Exemple : recevoir un message (dans le pipeline)
//!
//! ```no_run
//! use pdp_peppol::processor::PeppolReceiveProcessor;
//! use pdp_core::processor::Processor;
//! use pdp_core::exchange::Exchange;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let processor = PeppolReceiveProcessor::new();
//! let mut exchange = Exchange::new(b"<Invoice/>".to_vec());
//! exchange.set_property("peppol.sender", "0002::111111111");
//! exchange.set_property("peppol.receiver", "0002::222222222");
//! exchange.set_property("peppol.message_id", "msg-001@AP");
//! let result = processor.process(exchange).await?;
//! # Ok(())
//! # }
//! ```

pub mod model;
pub mod sbdh;
pub mod smp;
pub mod as4;
pub mod gateway;
pub mod processor;
pub mod error;

pub use model::{
    ParticipantId, DocumentTypeId, ProcessId, PeppolDocumentType,
    PeppolMessage, PeppolConfig, SmpEndpoint, SmpLookupResult, As4SendResult,
};
pub use gateway::{FilesystemGateway, RestGateway};
pub use processor::{PeppolSendProcessor, PeppolReceiveProcessor};
pub use error::PeppolError;
