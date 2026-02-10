//! Types d'erreurs pour le module PEPPOL.

use pdp_core::error::PdpError;

/// Erreurs spécifiques au module PEPPOL.
#[derive(Debug, thiserror::Error)]
pub enum PeppolError {
    /// Erreur de lookup SMP (découverte dynamique)
    #[error("Erreur SMP : {0}")]
    SmpError(String),

    /// Erreur de transport AS4
    #[error("Erreur AS4 : {0}")]
    As4Error(String),

    /// Erreur de construction/parsing SBDH
    #[error("Erreur SBDH : {0}")]
    SbdhError(String),

    /// Erreur de configuration
    #[error("Erreur de configuration PEPPOL : {0}")]
    ConfigError(String),

    /// Erreur réseau (HTTP, TLS)
    #[error("Erreur réseau : {0}")]
    NetworkError(String),

    /// Participant non trouvé dans l'annuaire
    #[error("Participant non trouvé : {0}")]
    ParticipantNotFound(String),

    /// Document type non supporté par le destinataire
    #[error("Type de document non supporté : {0}")]
    DocumentTypeNotSupported(String),
}

impl From<PeppolError> for PdpError {
    fn from(e: PeppolError) -> Self {
        PdpError::DistributionError(e.to_string())
    }
}

impl From<crate::sbdh::SbdhError> for PeppolError {
    fn from(e: crate::sbdh::SbdhError) -> Self {
        PeppolError::SbdhError(e.to_string())
    }
}
