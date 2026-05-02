use thiserror::Error;

use crate::model::AfnorErrorResponse;

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("Erreur d'authentification PISTE: {0}")]
    AuthError(String),

    #[error("Token expiré, renouvellement nécessaire")]
    TokenExpired,

    #[error("Ressource non trouvée: {0}")]
    NotFound(String),

    #[error("Requête trop volumineuse: {0}")]
    PayloadTooLarge(String),

    #[error("Entité non traitable (422): {0}")]
    UnprocessableEntity(String),

    #[error("Trop de requêtes (429) — veuillez réessayer après {retry_after:?}")]
    RateLimited {
        message: String,
        /// Nombre de secondes avant de réessayer (header Retry-After)
        retry_after: Option<u64>,
    },

    #[error("Timeout de la requête (408): {0}")]
    RequestTimeout(String),

    #[error("Service indisponible (503): {0}")]
    ServiceUnavailable(String),

    #[error("Endpoint non implémenté (501): {0}")]
    NotImplemented(String),

    #[error("Erreur HTTP {status}: {message}")]
    HttpError { status: u16, message: String },

    #[error("Erreur de requête: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("Erreur de sérialisation JSON: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Erreur de réponse PPF: code={code}, message={message}")]
    PpfError { code: String, message: String },

    #[error("Erreur AFNOR Flow Service: {0}")]
    AfnorError(String),

    #[error("Erreur IO: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Erreur de configuration: {0}")]
    ConfigError(String),
}

impl ClientError {
    /// Construit l'erreur appropriée à partir d'un code de statut HTTP
    /// et du body de la réponse, conformément à XP Z12-013 §5.5
    pub fn from_http_response(
        status: u16,
        body: &str,
        operation: &str,
        retry_after: Option<u64>,
    ) -> Self {
        let context = format!("{}: {}", operation, body);

        // Tenter de parser la réponse d'erreur structurée AFNOR
        let _parsed: Option<AfnorErrorResponse> = serde_json::from_str(body).ok();

        match status {
            401 => Self::TokenExpired,
            404 => Self::NotFound(context),
            408 => Self::RequestTimeout(context),
            413 => Self::PayloadTooLarge(context),
            422 => Self::UnprocessableEntity(context),
            429 => Self::RateLimited {
                message: context,
                retry_after,
            },
            501 => Self::NotImplemented(context),
            503 => Self::ServiceUnavailable(context),
            _ => Self::HttpError {
                status,
                message: context,
            },
        }
    }

    /// Indique si l'erreur est retryable (temporaire)
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::TokenExpired
                | Self::RateLimited { .. }
                | Self::RequestTimeout(_)
                | Self::ServiceUnavailable(_)
        )
    }
}

pub type ClientResult<T> = Result<T, ClientError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_http_response_404() {
        let err = ClientError::from_http_response(404, "not found", "get_flow", None);
        assert!(matches!(err, ClientError::NotFound(_)));
    }

    #[test]
    fn test_from_http_response_408() {
        let err = ClientError::from_http_response(408, "timeout", "post_flow", None);
        assert!(matches!(err, ClientError::RequestTimeout(_)));
    }

    #[test]
    fn test_from_http_response_413() {
        let err = ClientError::from_http_response(413, "too big", "post_flow", None);
        assert!(matches!(err, ClientError::PayloadTooLarge(_)));
    }

    #[test]
    fn test_from_http_response_429_with_retry_after() {
        let err = ClientError::from_http_response(429, "throttled", "post_flow", Some(30));
        match err {
            ClientError::RateLimited { retry_after, .. } => assert_eq!(retry_after, Some(30)),
            _ => panic!("Expected RateLimited"),
        }
    }

    #[test]
    fn test_from_http_response_501() {
        let err = ClientError::from_http_response(501, "not impl", "search", None);
        assert!(matches!(err, ClientError::NotImplemented(_)));
    }

    #[test]
    fn test_from_http_response_unknown() {
        let err = ClientError::from_http_response(418, "teapot", "x", None);
        match err {
            ClientError::HttpError { status, .. } => assert_eq!(status, 418),
            _ => panic!("Expected HttpError"),
        }
    }

    #[test]
    fn test_is_retryable() {
        // Retryables (temporaires)
        assert!(ClientError::TokenExpired.is_retryable());
        assert!(ClientError::RateLimited {
            message: "x".into(),
            retry_after: None
        }
        .is_retryable());
        assert!(ClientError::RequestTimeout("x".into()).is_retryable());
        assert!(ClientError::ServiceUnavailable("x".into()).is_retryable());

        // Non-retryables (permanents)
        assert!(!ClientError::NotFound("x".into()).is_retryable());
        assert!(!ClientError::PayloadTooLarge("x".into()).is_retryable());
        assert!(!ClientError::NotImplemented("x".into()).is_retryable());
        assert!(!ClientError::UnprocessableEntity("x".into()).is_retryable());
    }
}
