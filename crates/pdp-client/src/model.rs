use serde::{Deserialize, Serialize};

// ============================================================
// AFNOR XP Z12-013 Flow Service — POST /v1/flows
// ============================================================

/// Syntaxe du flux AFNOR
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum FlowSyntax {
    CII,
    UBL,
    #[serde(rename = "Factur-X")]
    FacturX,
    CDAR,
    FRR,
}

impl std::fmt::Display for FlowSyntax {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CII => write!(f, "CII"),
            Self::UBL => write!(f, "UBL"),
            Self::FacturX => write!(f, "Factur-X"),
            Self::CDAR => write!(f, "CDAR"),
            Self::FRR => write!(f, "FRR"),
        }
    }
}

/// Profil du flux AFNOR
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum FlowProfile {
    Basic,
    CIUS,
    #[serde(rename = "Extended-CTC-FR")]
    ExtendedCtcFr,
}

/// Règle de traitement AFNOR
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum ProcessingRule {
    B2B,
    B2BInt,
    B2C,
    OutOfScope,
    ArchiveOnly,
    NotApplicable,
}

/// Type de flux AFNOR
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FlowType {
    CustomerInvoice,
    SupplierInvoice,
    StateInvoice,
    CustomerInvoiceLC,
    SupplierInvoiceLC,
    StateCustomerInvoiceLC,
    StateSupplierInvoiceLC,
    AggregatedCustomerTransactionReport,
    IndividualCustomerTransactionReport,
    AggregatedCustomerPaymentReport,
    UnitaryCustomerPaymentReport,
    UnitarySupplierTransactionReport,
    MultiFlowReport,
}

/// Métadonnées du flux AFNOR (flowInfo)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AfnorFlowInfo {
    /// Identifiant de suivi (UUID)
    pub tracking_id: String,
    /// Nom du flux
    pub name: String,
    /// Règle de traitement
    pub processing_rule: ProcessingRule,
    /// Syntaxe du flux
    pub flow_syntax: FlowSyntax,
    /// Profil du flux
    pub flow_profile: FlowProfile,
    /// Type de flux
    pub flow_type: FlowType,
    /// SHA-256 du fichier
    pub sha256: String,
    /// URL de callback (optionnel)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub callback_url: Option<String>,
}

/// Statut d'acquittement AFNOR
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub enum AckStatus {
    Pending,
    Ok,
    Error,
}

/// Détail d'acquittement AFNOR
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AckDetail {
    pub level: Option<String>,
    pub item: Option<String>,
    pub reason_code: Option<String>,
    pub reason_message: Option<String>,
}

/// Réponse de création d'un flux AFNOR
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AfnorFlowCreateResponse {
    pub flow_id: String,
    pub acknowledgement: Option<AfnorAcknowledgement>,
}

/// Acquittement AFNOR
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AfnorAcknowledgement {
    pub status: AckStatus,
    pub details: Option<Vec<AckDetail>>,
}

/// Réponse de recherche de flux AFNOR
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AfnorFlowSearchResponse {
    pub items: Option<Vec<AfnorFlowItem>>,
    pub total: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AfnorFlowItem {
    pub flow_id: Option<String>,
    pub tracking_id: Option<String>,
    pub name: Option<String>,
    pub flow_type: Option<String>,
    pub flow_syntax: Option<String>,
    pub acknowledgement: Option<AfnorAcknowledgement>,
    pub updated_at: Option<String>,
}

/// Code raison d'erreur AFNOR
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AfnorReasonCode {
    EmptyAttachement,
    AttachmentTypeError,
    EmptyFlow,
    OtherTechnicalError,
    InvalidSchema,
    FileSizeExceeded,
    FlowTypeError,
    AlreadyExistingFlow,
    VirusFound,
    ChecksumMismatch,
    InvoiceLCInvalidStatus,
    InvoiceLCStatusError,
    InvoiceLCRuleError,
    InvoiceLCAccessDenied,
    InvoiceLCAmountError,
}

// ============================================================
// PPF Annuaire
// ============================================================

/// Environnement PPF
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum PpfEnvironment {
    Dev,
    Int,
    Rec,
    PreProd,
    Prod,
}

impl PpfEnvironment {
    pub fn subdomain(&self) -> &str {
        match self {
            Self::Dev => "env.dev.",
            Self::Int => "env.int.",
            Self::Rec => "env.rec.",
            Self::PreProd => "env.pre.prod.",
            Self::Prod => "api.",
        }
    }

    pub fn base_url(&self) -> String {
        format!(
            "https://{}aife.economie.gouv.fr",
            self.subdomain()
        )
    }
}
