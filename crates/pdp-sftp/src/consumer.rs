use async_trait::async_trait;
use pdp_core::endpoint::Consumer;
use pdp_core::error::{PdpError, PdpResult};
use pdp_core::exchange::Exchange;

use crate::config::SftpConfig;

/// Consumer SFTP : lit des fichiers depuis un serveur SFTP
pub struct SftpConsumer {
    name: String,
    config: SftpConfig,
}

impl SftpConsumer {
    pub fn new(name: &str, config: SftpConfig) -> Self {
        Self {
            name: name.to_string(),
            config,
        }
    }
}

#[async_trait]
impl Consumer for SftpConsumer {
    fn name(&self) -> &str {
        &self.name
    }

    async fn poll(&self) -> PdpResult<Vec<Exchange>> {
        tracing::info!(
            consumer = %self.name,
            host = %self.config.host,
            path = %self.config.remote_path,
            "Polling SFTP"
        );

        // Connexion SSH
        let ssh_config = russh::client::Config::default();
        let sh = ClientHandler::new(
            self.config.known_hosts_path.as_deref(),
            &self.config.host,
        );

        let mut session = russh::client::connect(
            std::sync::Arc::new(ssh_config),
            (self.config.host.as_str(), self.config.port),
            sh,
        )
        .await
        .map_err(|e| PdpError::SftpError(format!("Connexion SSH échouée: {}", e)))?;

        // Authentification
        let authenticated = if let Some(ref password) = self.config.password {
            session
                .authenticate_password(&self.config.username, password)
                .await
                .map_err(|e| PdpError::SftpError(format!("Auth password échouée: {}", e)))?
        } else if let Some(ref key_path) = self.config.private_key_path {
            let key = russh_keys::load_secret_key(key_path, None)
                .map_err(|e| PdpError::SftpError(format!("Clé privée invalide: {}", e)))?;
            session
                .authenticate_publickey(&self.config.username, std::sync::Arc::new(key))
                .await
                .map_err(|e| PdpError::SftpError(format!("Auth clé publique échouée: {}", e)))?
        } else {
            return Err(PdpError::SftpError(
                "Aucune méthode d'authentification configurée (password ou clé privée)".to_string(),
            ));
        };

        if !authenticated {
            return Err(PdpError::SftpError("Authentification SFTP refusée".to_string()));
        }

        // Ouvrir un canal SFTP
        let channel = session
            .channel_open_session()
            .await
            .map_err(|e| PdpError::SftpError(format!("Ouverture canal échouée: {}", e)))?;

        channel
            .request_subsystem(true, "sftp")
            .await
            .map_err(|e| PdpError::SftpError(format!("Subsystem SFTP échoué: {}", e)))?;

        let sftp = russh_sftp::client::SftpSession::new(channel.into_stream())
            .await
            .map_err(|e| PdpError::SftpError(format!("Session SFTP échouée: {}", e)))?;

        // Lister les fichiers
        let entries = sftp
            .read_dir(&self.config.remote_path)
            .await
            .map_err(|e| PdpError::SftpError(format!("Lecture répertoire échouée: {}", e)))?;

        // Collecter les fichiers candidats avec leur taille
        let mut candidates: Vec<(String, u64)> = Vec::new();
        for entry in entries {
            let filename = entry.file_name();
            if !matches_pattern(&filename, &self.config.file_pattern) {
                continue;
            }
            let size = entry.metadata().len();
            candidates.push((filename, size));
        }

        // Vérification de stabilité : attendre puis revérifier la taille
        if self.config.stable_delay_ms > 0 && !candidates.is_empty() {
            tokio::time::sleep(std::time::Duration::from_millis(self.config.stable_delay_ms)).await;

            let mut stable_candidates = Vec::new();
            for (filename, initial_size) in candidates {
                let remote_file = format!("{}/{}", self.config.remote_path, filename);
                match sftp.metadata(&remote_file).await {
                    Ok(meta) => {
                        let current_size = meta.len();
                        if current_size == initial_size {
                            stable_candidates.push(filename);
                        } else {
                            tracing::debug!(
                                consumer = %self.name,
                                filename = %filename,
                                initial_size,
                                current_size,
                                "Fichier ignoré (encore en cours d'écriture)"
                            );
                        }
                    }
                    Err(_) => {
                        // Fichier disparu entre-temps, on l'ignore
                        tracing::debug!(
                            consumer = %self.name,
                            filename = %filename,
                            "Fichier disparu entre les deux vérifications"
                        );
                    }
                }
            }
            candidates = stable_candidates.into_iter().map(|f| (f, 0)).collect();
        }

        let mut exchanges = Vec::new();

        for (filename, _) in &candidates {
            let remote_file = format!("{}/{}", self.config.remote_path, filename);

            // Lire le fichier
            let data = sftp
                .read(&remote_file)
                .await
                .map_err(|e| PdpError::SftpError(format!("Lecture fichier {} échouée: {}", filename, e)))?;

            let exchange = Exchange::new(data).with_filename(filename);

            tracing::info!(
                consumer = %self.name,
                filename = %filename,
                exchange_id = %exchange.id,
                "Fichier SFTP lu"
            );

            exchanges.push(exchange);

            // Archiver ou supprimer le fichier traité
            if let Some(ref archive_path) = self.config.archive_path {
                let archive_file = format!("{}/{}", archive_path, filename);
                let _ = sftp.rename(&remote_file, &archive_file).await;
                tracing::debug!(from = %remote_file, to = %archive_file, "Fichier archivé");
            } else if self.config.delete_after_read {
                let _ = sftp.remove_file(&remote_file).await;
                tracing::debug!(file = %remote_file, "Fichier supprimé après lecture");
            }
        }

        tracing::info!(
            consumer = %self.name,
            count = exchanges.len(),
            "Polling SFTP terminé"
        );

        Ok(exchanges)
    }
}

/// Vérifie si un nom de fichier correspond à un pattern simple (*, *.xml, *.pdf)
fn matches_pattern(filename: &str, pattern: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    if let Some(ext) = pattern.strip_prefix("*.") {
        return filename.to_lowercase().ends_with(&format!(".{}", ext.to_lowercase()));
    }
    filename == pattern
}

/// Handler SSH client avec vérification optionnelle de la clé serveur
struct ClientHandler {
    /// Clé publique SSH attendue du serveur (encodée en hexadécimal)
    /// Si None, la vérification est désactivée (dev mode)
    known_host_key: Option<String>,
}

impl ClientHandler {
    fn new(known_hosts_path: Option<&str>, host: &str) -> Self {
        let known_host_key = known_hosts_path.and_then(|path| {
            crate::hostkey::lookup_host_key(path, host)
        });
        Self { known_host_key }
    }
}

#[async_trait]
impl russh::client::Handler for ClientHandler {
    type Error = anyhow::Error;

    async fn check_server_key(
        &mut self,
        server_public_key: &russh_keys::key::PublicKey,
    ) -> Result<bool, Self::Error> {
        match &self.known_host_key {
            Some(expected_key) => {
                let actual = crate::hostkey::key_to_string(server_public_key);
                if actual == *expected_key {
                    tracing::info!("Clé serveur SSH vérifiée (conforme au known_hosts)");
                    Ok(true)
                } else {
                    tracing::error!(
                        expected = %expected_key,
                        actual = %actual,
                        "Clé serveur SSH invalide — possible attaque MITM"
                    );
                    Ok(false)
                }
            }
            None => {
                tracing::warn!(
                    "Vérification de la clé serveur SSH désactivée (known_hosts non configuré). \
                     En production, configurez known_hosts_path dans SftpConfig."
                );
                Ok(true)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches_pattern_wildcard() {
        assert!(matches_pattern("facture.xml", "*"));
        assert!(matches_pattern("anything.pdf", "*"));
    }

    #[test]
    fn test_matches_pattern_extension() {
        assert!(matches_pattern("facture.xml", "*.xml"));
        assert!(matches_pattern("FACTURE.XML", "*.xml"));
        assert!(!matches_pattern("facture.pdf", "*.xml"));
    }

    #[test]
    fn test_matches_pattern_exact() {
        assert!(matches_pattern("facture.xml", "facture.xml"));
        assert!(!matches_pattern("other.xml", "facture.xml"));
    }
}
