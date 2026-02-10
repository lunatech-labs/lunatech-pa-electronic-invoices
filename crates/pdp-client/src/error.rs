use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("Erreur d'authentification PISTE: {0}")]
    AuthError(String),

    #[error("Token expiré, renouvellement nécessaire")]
    TokenExpired,

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

pub type ClientResult<T> = Result<T, ClientError>;
