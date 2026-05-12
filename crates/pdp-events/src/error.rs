//! Erreurs du crate `pdp-events`.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum EventError {
    #[error("erreur base de données: {0}")]
    Database(#[from] sqlx::Error),

    #[error("sérialisation JSON: {0}")]
    Json(#[from] serde_json::Error),

    #[error("erreur subscriber `{subscriber}`: {message}")]
    Subscriber {
        subscriber: String,
        message: String,
    },

    #[error("{0}")]
    Other(String),
}

pub type EventResult<T> = std::result::Result<T, EventError>;
