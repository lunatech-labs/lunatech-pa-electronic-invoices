use async_trait::async_trait;
use pdp_core::endpoint::Producer;
use pdp_core::error::{PdpError, PdpResult};
use pdp_core::exchange::Exchange;

use crate::config::SftpConfig;

/// Producer SFTP : envoie des fichiers vers un serveur SFTP
pub struct SftpProducer {
    name: String,
    config: SftpConfig,
}

impl SftpProducer {
    pub fn new(name: &str, config: SftpConfig) -> Self {
        Self {
            name: name.to_string(),
            config,
        }
    }
}

#[async_trait]
impl Producer for SftpProducer {
    fn name(&self) -> &str {
        &self.name
    }

    async fn send(&self, exchange: Exchange) -> PdpResult<Exchange> {
        tracing::info!(
            producer = %self.name,
            host = %self.config.host,
            path = %self.config.remote_path,
            exchange_id = %exchange.id,
            "Envoi SFTP"
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
                "Aucune méthode d'authentification configurée".to_string(),
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

        // Déterminer le nom du fichier de destination
        let id_string = exchange.id.to_string();
        let filename = exchange
            .source_filename
            .as_deref()
            .unwrap_or(&id_string);

        let remote_file = format!("{}/{}", self.config.remote_path, filename);

        // Écrire le fichier
        sftp.write(&remote_file, &exchange.body)
            .await
            .map_err(|e| PdpError::SftpError(format!("Écriture fichier {} échouée: {}", remote_file, e)))?;

        tracing::info!(
            producer = %self.name,
            filename = %filename,
            remote_path = %remote_file,
            size = exchange.body.len(),
            "Fichier SFTP envoyé"
        );

        Ok(exchange)
    }
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
