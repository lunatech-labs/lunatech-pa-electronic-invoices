use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing;

use crate::error::{ClientError, ClientResult};

/// Configuration d'authentification PISTE (OAuth2 client_credentials)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PisteAuthConfig {
    /// URL du token endpoint PISTE
    /// Ex: "https://oauth.piste.gouv.fr/api/oauth/token"
    pub token_url: String,
    /// Client ID fourni par PISTE
    pub client_id: String,
    /// Client Secret fourni par PISTE
    pub client_secret: String,
    /// Scopes demandés (ex: "openid")
    #[serde(default = "default_scope")]
    pub scope: String,
}

fn default_scope() -> String {
    "openid".to_string()
}

/// Token d'accès PISTE
#[derive(Debug, Clone)]
struct AccessToken {
    token: String,
    expires_at: DateTime<Utc>,
}

/// Gestionnaire d'authentification PISTE avec renouvellement automatique du token
pub struct PisteAuth {
    config: PisteAuthConfig,
    http: reqwest::Client,
    token: RwLock<Option<AccessToken>>,
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    #[serde(default)]
    token_type: String,
    /// Durée de validité en secondes
    expires_in: i64,
    #[serde(default)]
    scope: String,
}

impl PisteAuth {
    pub fn new(config: PisteAuthConfig) -> Self {
        Self {
            config,
            http: reqwest::Client::new(),
            token: RwLock::new(None),
        }
    }

    /// Retourne un token valide, en le renouvelant si nécessaire.
    /// Marge de sécurité de 60 secondes avant expiration.
    pub async fn get_token(&self) -> ClientResult<String> {
        {
            let guard = self.token.read().await;
            if let Some(ref token) = *guard {
                if token.expires_at > Utc::now() + Duration::seconds(60) {
                    return Ok(token.token.clone());
                }
            }
        }

        self.refresh_token().await
    }

    /// Force le renouvellement du token
    async fn refresh_token(&self) -> ClientResult<String> {
        tracing::debug!(
            token_url = %self.config.token_url,
            client_id = %self.config.client_id,
            "Renouvellement du token PISTE"
        );

        let params = [
            ("grant_type", "client_credentials"),
            ("client_id", &self.config.client_id),
            ("client_secret", &self.config.client_secret),
            ("scope", &self.config.scope),
        ];

        let response = self
            .http
            .post(&self.config.token_url)
            .form(&params)
            .send()
            .await
            .map_err(|e| ClientError::AuthError(format!("Requête token échouée: {}", e)))?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(ClientError::AuthError(format!(
                "Token PISTE refusé (HTTP {}): {}",
                status, body
            )));
        }

        let token_resp: TokenResponse = response
            .json()
            .await
            .map_err(|e| ClientError::AuthError(format!("Réponse token invalide: {}", e)))?;

        let expires_at = Utc::now() + Duration::seconds(token_resp.expires_in);

        tracing::info!(
            token_type = %token_resp.token_type,
            expires_in = token_resp.expires_in,
            scope = %token_resp.scope,
            "Token PISTE obtenu"
        );

        let token_str = token_resp.access_token.clone();

        {
            let mut guard = self.token.write().await;
            *guard = Some(AccessToken {
                token: token_resp.access_token,
                expires_at,
            });
        }

        Ok(token_str)
    }

    /// Invalide le token courant (force un renouvellement au prochain appel)
    pub async fn invalidate(&self) {
        let mut guard = self.token.write().await;
        *guard = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_piste_auth_config_defaults() {
        let config = PisteAuthConfig {
            token_url: "https://oauth.piste.gouv.fr/api/oauth/token".to_string(),
            client_id: "test-client".to_string(),
            client_secret: "test-secret".to_string(),
            scope: default_scope(),
        };
        assert_eq!(config.scope, "openid");
    }

    #[tokio::test]
    async fn test_piste_auth_no_token_initially() {
        let auth = PisteAuth::new(PisteAuthConfig {
            token_url: "https://fake.example.com/token".to_string(),
            client_id: "test".to_string(),
            client_secret: "test".to_string(),
            scope: "openid".to_string(),
        });

        let guard = auth.token.read().await;
        assert!(guard.is_none());
    }
}
