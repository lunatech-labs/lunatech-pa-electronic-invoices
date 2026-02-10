//! Intégration avec un gateway AS4 externe (Oxalis, phase4).
//!
//! Notre PDP ne gère pas directement le protocole AS4 (WS-Security,
//! signature XML, PKI PEPPOL). À la place, on délègue à un gateway
//! AS4 certifié qui gère la conformité protocolaire.
//!
//! # Gateways supportés
//!
//! - **[Oxalis](https://github.com/OxalisCommunity/oxalis)** — Java, open source, certifié PEPPOL
//! - **[phase4](https://github.com/phax/phase4)** — Java, open source, basé sur AS4
//!
//! # Modes d'intégration
//!
//! ## 1. Mode filesystem (recommandé pour Oxalis)
//!
//! ```text
//! Envoi :  Pipeline → SBDH XML → outbox/ → Oxalis poll → AS4 → PDP distante
//! Réception : PDP distante → AS4 → Oxalis → inbox/ → Pipeline poll
//! ```
//!
//! ## 2. Mode API REST (Oxalis REST API / phase4)
//!
//! ```text
//! Envoi :  Pipeline → POST /send (SBDH XML) → Gateway → AS4 → PDP distante
//! Réception : PDP distante → AS4 → Gateway → webhook POST → Pipeline
//! ```

use std::path::{Path, PathBuf};

use crate::error::PeppolError;
use crate::model::PeppolMessage;
use crate::sbdh;

// ============================================================
// Mode Filesystem
// ============================================================

/// Intégration filesystem avec un gateway AS4 (Oxalis).
///
/// Dépose les messages sortants dans un répertoire `outbox/` que le gateway
/// surveille, et lit les messages entrants depuis un répertoire `inbox/`.
pub struct FilesystemGateway {
    /// Répertoire de dépôt pour les messages sortants
    outbox_dir: PathBuf,
    /// Répertoire de lecture pour les messages entrants
    inbox_dir: PathBuf,
}

impl FilesystemGateway {
    pub fn new(outbox_dir: impl AsRef<Path>, inbox_dir: impl AsRef<Path>) -> Self {
        Self {
            outbox_dir: outbox_dir.as_ref().to_path_buf(),
            inbox_dir: inbox_dir.as_ref().to_path_buf(),
        }
    }

    /// Envoie un message en le déposant dans le répertoire outbox.
    ///
    /// Le gateway AS4 (Oxalis) surveille ce répertoire et transmet
    /// le fichier SBDH via AS4 vers l'Access Point destinataire.
    pub fn send(&self, message: &PeppolMessage) -> Result<String, PeppolError> {
        std::fs::create_dir_all(&self.outbox_dir)
            .map_err(|e| PeppolError::NetworkError(format!("Impossible de créer outbox : {}", e)))?;

        let sbdh_xml = sbdh::build_sbdh(message);

        // Nom de fichier : sender_receiver_messageId.xml
        let filename = format!(
            "{}_{}_{}_{}.xml",
            message.sender.value,
            message.receiver.value,
            message.document_type.to_string().to_lowercase(),
            &message.message_id[..8],
        );

        let filepath = self.outbox_dir.join(&filename);

        std::fs::write(&filepath, sbdh_xml.as_bytes())
            .map_err(|e| PeppolError::NetworkError(format!("Écriture outbox échouée : {}", e)))?;

        tracing::info!(
            filename = %filename,
            sender = %message.sender,
            receiver = %message.receiver,
            document_type = %message.document_type,
            "Message PEPPOL déposé dans outbox pour le gateway"
        );

        Ok(filename)
    }

    /// Lit les messages entrants depuis le répertoire inbox.
    ///
    /// Retourne les fichiers SBDH XML déposés par le gateway AS4 (Oxalis)
    /// après réception depuis une PDP distante.
    pub fn receive(&self) -> Result<Vec<InboxMessage>, PeppolError> {
        if !self.inbox_dir.exists() {
            return Ok(Vec::new());
        }

        let entries = std::fs::read_dir(&self.inbox_dir)
            .map_err(|e| PeppolError::NetworkError(format!("Lecture inbox échouée : {}", e)))?;

        let mut messages = Vec::new();

        for entry in entries {
            let entry = entry
                .map_err(|e| PeppolError::NetworkError(format!("Entrée inbox invalide : {}", e)))?;
            let path = entry.path();

            if !path.is_file() {
                continue;
            }

            let filename = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();

            // Lire uniquement les fichiers XML
            if !filename.ends_with(".xml") {
                continue;
            }

            let content = std::fs::read_to_string(&path)
                .map_err(|e| PeppolError::NetworkError(format!("Lecture {} échouée : {}", filename, e)))?;

            // Tenter de parser comme SBDH
            match sbdh::parse_sbdh(&content) {
                Ok(parsed) => {
                    tracing::info!(
                        filename = %filename,
                        sender = %parsed.sender,
                        receiver = %parsed.receiver,
                        "Message PEPPOL lu depuis inbox"
                    );

                    messages.push(InboxMessage {
                        filename: filename.clone(),
                        filepath: path.clone(),
                        sender: parsed.sender.to_smp_id(),
                        receiver: parsed.receiver.to_smp_id(),
                        instance_id: parsed.instance_id,
                        document_type_id: parsed.document_type_id.to_string(),
                        process_id: parsed.process_id.to_string(),
                        payload: parsed.payload,
                    });
                }
                Err(e) => {
                    tracing::warn!(
                        filename = %filename,
                        error = %e,
                        "Fichier inbox ignoré (SBDH invalide)"
                    );
                }
            }
        }

        Ok(messages)
    }

    /// Supprime un message de l'inbox après traitement.
    pub fn acknowledge(&self, filename: &str) -> Result<(), PeppolError> {
        let filepath = self.inbox_dir.join(filename);
        if filepath.exists() {
            std::fs::remove_file(&filepath)
                .map_err(|e| PeppolError::NetworkError(format!("Suppression {} échouée : {}", filename, e)))?;
            tracing::debug!(filename = %filename, "Message inbox acquitté (supprimé)");
        }
        Ok(())
    }

    /// Déplace un message de l'inbox vers un répertoire d'archive.
    pub fn archive(&self, filename: &str, archive_dir: &Path) -> Result<(), PeppolError> {
        std::fs::create_dir_all(archive_dir)
            .map_err(|e| PeppolError::NetworkError(format!("Création archive dir échouée : {}", e)))?;

        let src = self.inbox_dir.join(filename);
        let dst = archive_dir.join(filename);

        if src.exists() {
            std::fs::rename(&src, &dst)
                .map_err(|e| PeppolError::NetworkError(format!("Archivage {} échoué : {}", filename, e)))?;
            tracing::debug!(filename = %filename, "Message inbox archivé");
        }
        Ok(())
    }
}

/// Message lu depuis l'inbox du gateway.
#[derive(Debug, Clone)]
pub struct InboxMessage {
    /// Nom du fichier dans l'inbox
    pub filename: String,
    /// Chemin complet
    pub filepath: PathBuf,
    /// Identifiant de l'émetteur (scheme::value)
    pub sender: String,
    /// Identifiant du destinataire (scheme::value)
    pub receiver: String,
    /// Instance ID du SBDH
    pub instance_id: String,
    /// Document type identifier
    pub document_type_id: String,
    /// Process identifier
    pub process_id: String,
    /// Payload XML (facture ou CDAR, sans enveloppe SBDH)
    pub payload: String,
}

// ============================================================
// Mode API REST
// ============================================================

/// Client REST pour un gateway AS4 (Oxalis REST API, phase4).
///
/// Envoie les messages via HTTP POST et reçoit les notifications
/// de messages entrants via webhook ou polling.
pub struct RestGateway {
    /// URL de base du gateway (ex: "http://localhost:8080")
    base_url: String,
    http: reqwest::Client,
}

impl RestGateway {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            http: reqwest::Client::new(),
        }
    }

    /// Envoie un message via l'API REST du gateway.
    ///
    /// Pour Oxalis : POST /api/send avec le SBDH XML en body.
    pub async fn send(&self, message: &PeppolMessage) -> Result<GatewaySendResult, PeppolError> {
        let sbdh_xml = sbdh::build_sbdh(message);

        let url = format!("{}/api/send", self.base_url);

        tracing::info!(
            url = %url,
            sender = %message.sender,
            receiver = %message.receiver,
            "Envoi via gateway REST"
        );

        let response = self.http
            .post(&url)
            .header("Content-Type", "application/xml")
            .body(sbdh_xml)
            .send()
            .await
            .map_err(|e| PeppolError::NetworkError(format!("Requête gateway échouée : {}", e)))?;

        let status = response.status();
        let body = response.text().await.unwrap_or_default();

        if status.is_success() {
            tracing::info!(
                sender = %message.sender,
                receiver = %message.receiver,
                "Message envoyé via gateway"
            );
            Ok(GatewaySendResult {
                success: true,
                message_id: message.message_id.clone(),
                gateway_response: body,
            })
        } else {
            tracing::warn!(
                status = %status,
                body = %body,
                "Envoi gateway échoué"
            );
            Ok(GatewaySendResult {
                success: false,
                message_id: message.message_id.clone(),
                gateway_response: body,
            })
        }
    }

    /// Vérifie la santé du gateway.
    pub async fn health_check(&self) -> Result<bool, PeppolError> {
        let url = format!("{}/api/health", self.base_url);
        match self.http.get(&url).send().await {
            Ok(resp) => Ok(resp.status().is_success()),
            Err(_) => Ok(false),
        }
    }
}

/// Résultat d'un envoi via le gateway.
#[derive(Debug, Clone)]
pub struct GatewaySendResult {
    pub success: bool,
    pub message_id: String,
    pub gateway_response: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::*;
    use tempfile::TempDir;

    #[test]
    fn test_filesystem_gateway_send() {
        let tmp = TempDir::new().unwrap();
        let outbox = tmp.path().join("outbox");
        let inbox = tmp.path().join("inbox");

        let gw = FilesystemGateway::new(&outbox, &inbox);

        let sender = ParticipantId::from_siren("111111111");
        let receiver = ParticipantId::from_siren("222222222");
        let msg = PeppolMessage::ubl_invoice(
            sender,
            receiver,
            b"<Invoice><ID>FA-001</ID></Invoice>".to_vec(),
        );

        let filename = gw.send(&msg).unwrap();
        assert!(filename.contains("111111111"));
        assert!(filename.contains("222222222"));
        assert!(filename.ends_with(".xml"));

        // Vérifier que le fichier existe
        let filepath = outbox.join(&filename);
        assert!(filepath.exists());

        // Vérifier que le contenu est un SBDH valide
        let content = std::fs::read_to_string(&filepath).unwrap();
        assert!(content.contains("StandardBusinessDocument"));
        assert!(content.contains("FA-001"));
    }

    #[test]
    fn test_filesystem_gateway_send_cdar() {
        let tmp = TempDir::new().unwrap();
        let outbox = tmp.path().join("outbox");
        let inbox = tmp.path().join("inbox");

        let gw = FilesystemGateway::new(&outbox, &inbox);

        let sender = ParticipantId::from_siren("111111111");
        let receiver = ParticipantId::from_siren("222222222");
        let msg = PeppolMessage::cdar(
            sender,
            receiver,
            b"<CrossDomainAcknowledgementAndResponse/>".to_vec(),
        );

        let filename = gw.send(&msg).unwrap();
        assert!(filename.contains("cdar"));
    }

    #[test]
    fn test_filesystem_gateway_receive_empty() {
        let tmp = TempDir::new().unwrap();
        let gw = FilesystemGateway::new(tmp.path().join("out"), tmp.path().join("in"));
        let messages = gw.receive().unwrap();
        assert!(messages.is_empty());
    }

    #[test]
    fn test_filesystem_gateway_roundtrip() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("shared");

        // Simuler : le gateway dépose dans inbox = notre outbox
        let gw_send = FilesystemGateway::new(&dir, tmp.path().join("unused"));
        let gw_recv = FilesystemGateway::new(tmp.path().join("unused"), &dir);

        let sender = ParticipantId::from_siren("111111111");
        let receiver = ParticipantId::from_siren("222222222");
        let msg = PeppolMessage::ubl_invoice(
            sender,
            receiver,
            b"<Invoice><ID>ROUNDTRIP-001</ID></Invoice>".to_vec(),
        );

        // Envoyer (dépose dans dir/)
        let filename = gw_send.send(&msg).unwrap();

        // Recevoir (lit depuis dir/)
        let received = gw_recv.receive().unwrap();
        assert_eq!(received.len(), 1);
        assert_eq!(received[0].sender, "0002::111111111");
        assert_eq!(received[0].receiver, "0002::222222222");
        assert!(received[0].payload.contains("ROUNDTRIP-001"));

        // Acquitter
        gw_recv.acknowledge(&filename).unwrap();
        let after = gw_recv.receive().unwrap();
        assert!(after.is_empty(), "Après acquittement, inbox doit être vide");
    }

    #[test]
    fn test_filesystem_gateway_archive() {
        let tmp = TempDir::new().unwrap();
        let inbox = tmp.path().join("inbox");
        let archive = tmp.path().join("archive");

        let gw = FilesystemGateway::new(tmp.path().join("out"), &inbox);

        // Déposer un fichier SBDH dans l'inbox
        std::fs::create_dir_all(&inbox).unwrap();
        let sender = ParticipantId::from_siren("111111111");
        let receiver = ParticipantId::from_siren("222222222");
        let msg = PeppolMessage::ubl_invoice(sender, receiver, b"<Invoice/>".to_vec());
        let sbdh = sbdh::build_sbdh(&msg);
        std::fs::write(inbox.join("test.xml"), &sbdh).unwrap();

        // Archiver
        gw.archive("test.xml", &archive).unwrap();
        assert!(!inbox.join("test.xml").exists());
        assert!(archive.join("test.xml").exists());
    }

    #[test]
    fn test_filesystem_gateway_ignores_non_xml() {
        let tmp = TempDir::new().unwrap();
        let inbox = tmp.path().join("inbox");
        std::fs::create_dir_all(&inbox).unwrap();

        // Fichier non-XML
        std::fs::write(inbox.join("readme.txt"), "not xml").unwrap();

        let gw = FilesystemGateway::new(tmp.path().join("out"), &inbox);
        let messages = gw.receive().unwrap();
        assert!(messages.is_empty());
    }

    #[test]
    fn test_filesystem_gateway_ignores_invalid_sbdh() {
        let tmp = TempDir::new().unwrap();
        let inbox = tmp.path().join("inbox");
        std::fs::create_dir_all(&inbox).unwrap();

        // XML valide mais pas un SBDH
        std::fs::write(inbox.join("bad.xml"), "<Invoice/>").unwrap();

        let gw = FilesystemGateway::new(tmp.path().join("out"), &inbox);
        let messages = gw.receive().unwrap();
        assert!(messages.is_empty(), "SBDH invalide doit être ignoré");
    }

    #[test]
    fn test_rest_gateway_new() {
        let gw = RestGateway::new("http://localhost:8080/");
        assert_eq!(gw.base_url, "http://localhost:8080");
    }
}
