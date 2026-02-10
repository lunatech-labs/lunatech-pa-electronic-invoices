use thiserror::Error;

#[derive(Error, Debug)]
pub enum PdpError {
    #[error("Erreur de parsing: {0}")]
    ParseError(String),

    #[error("Erreur de transformation: {source_format} -> {target_format}: {message}")]
    TransformError {
        source_format: String,
        target_format: String,
        message: String,
    },

    #[error("Erreur SFTP: {0}")]
    SftpError(String),

    #[error("Erreur de validation: {0}")]
    ValidationError(String),

    #[error("Erreur de configuration: {0}")]
    ConfigError(String),

    #[error("Erreur de route: {route_id}: {message}")]
    RouteError { route_id: String, message: String },

    #[error("Erreur de traçabilité: {0}")]
    TraceError(String),

    #[error("Erreur CDAR: {0}")]
    CdarError(String),

    #[error("Format de facture non supporté: {0}")]
    UnsupportedFormat(String),

    #[error("Facture non trouvée: {0}")]
    InvoiceNotFound(String),

    #[error("Erreur IO: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Erreur de routage: {0}")]
    RoutingError(String),

    #[error("Erreur de distribution: {0}")]
    DistributionError(String),

    #[error("Erreur interne: {0}")]
    Internal(String),
}

pub type PdpResult<T> = Result<T, PdpError>;
