use serde::{Deserialize, Serialize};

// ============================================================
// AFNOR XP Z12-013 — Common types
// ============================================================

/// Headers optionnels pour les requêtes AFNOR (XP Z12-013 §5.2)
#[derive(Debug, Clone, Default)]
pub struct AfnorRequestHeaders {
    /// Identifiant de corrélation pour le suivi des logs (UUID)
    pub request_id: Option<String>,
    /// Identifiant de l'organisation (multi-tenant)
    pub organization_id: Option<String>,
    /// Langue de réponse préférée
    pub accept_language: Option<String>,
}

/// Réponse d'erreur structurée AFNOR (XP Z12-013 §5.5)
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AfnorErrorResponse {
    /// Code d'erreur applicatif
    pub error_code: Option<String>,
    /// Message d'erreur lisible
    pub error_message: Option<String>,
    /// Détails supplémentaires
    pub details: Option<Vec<AfnorErrorDetail>>,
}

/// Détail d'une erreur AFNOR
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AfnorErrorDetail {
    pub field: Option<String>,
    pub message: Option<String>,
    pub code: Option<String>,
}

/// Statut de santé du service
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthCheckResponse {
    pub status: String,
    pub version: Option<String>,
}

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
    B2G,
    B2GInt,
    OutOfScope,
    B2GOutOfScope,
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
    UnitaryCustomerTransactionReport,
    AggregatedCustomerPaymentReport,
    UnitaryCustomerPaymentReport,
    UnitarySupplierTransactionReport,
    MultiFlowReport,
}

/// Direction du flux AFNOR
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum FlowDirection {
    Inbound,
    Outbound,
}

/// Type de document demandé pour GET /v1/flows/{flowId} (paramètre docType)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum DocType {
    /// Métadonnées du flux (JSON)
    Metadata,
    /// Document original tel que soumis
    Original,
    /// Document converti par la PDP réceptrice
    Converted,
    /// Vue lisible (PDF)
    ReadableView,
}

impl std::fmt::Display for DocType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Metadata => write!(f, "Metadata"),
            Self::Original => write!(f, "Original"),
            Self::Converted => write!(f, "Converted"),
            Self::ReadableView => write!(f, "ReadableView"),
        }
    }
}

/// Statut d'acquittement pour la recherche de flux
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum FlowAckStatus {
    Pending,
    Ok,
    Error,
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
    /// Type de flux (non envoyé dans la requête AFNOR — déduit par le serveur)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flow_type: Option<FlowType>,
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

/// Réponse complète de création d'un flux AFNOR (202 Accepted)
/// Correspond au schéma FullFlowInfo de la spec XP Z12-013
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AfnorFlowCreateResponse {
    pub flow_id: String,
    pub tracking_id: Option<String>,
    pub name: Option<String>,
    pub flow_syntax: Option<String>,
    pub flow_profile: Option<String>,
    pub flow_type: Option<String>,
    pub processing_rule: Option<String>,
    pub flow_direction: Option<FlowDirection>,
    pub submitted_at: Option<String>,
    pub acknowledgement: Option<AfnorAcknowledgement>,
}

/// Acquittement AFNOR
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AfnorAcknowledgement {
    pub status: AckStatus,
    pub details: Option<Vec<AckDetail>>,
}

/// Réponse de recherche de flux AFNOR (XP Z12-013 §5.3.2)
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AfnorFlowSearchResponse {
    pub limit: Option<u32>,
    pub filters: Option<serde_json::Value>,
    pub results: Option<Vec<AfnorFlowItem>>,
    // Compat ancien format
    pub items: Option<Vec<AfnorFlowItem>>,
    pub total: Option<u64>,
}

/// Élément de flux AFNOR (schéma Flow)
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AfnorFlowItem {
    pub flow_id: Option<String>,
    pub tracking_id: Option<String>,
    pub name: Option<String>,
    pub flow_type: Option<String>,
    pub flow_syntax: Option<String>,
    pub flow_profile: Option<String>,
    pub processing_rule: Option<String>,
    pub flow_direction: Option<String>,
    pub submitted_at: Option<String>,
    pub acknowledgement: Option<AfnorAcknowledgement>,
    pub updated_at: Option<String>,
}

/// Filtres typés pour la recherche de flux (XP Z12-013 §5.3.2)
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchFlowFilters {
    /// Filtrer les flux mis à jour après cette date (strict >)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_after: Option<String>,
    /// Filtrer les flux mis à jour avant cette date (strict <)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_before: Option<String>,
    /// Filtrer par règle de traitement
    #[serde(skip_serializing_if = "Option::is_none")]
    pub processing_rule: Option<Vec<ProcessingRule>>,
    /// Filtrer par type de flux
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flow_type: Option<Vec<FlowType>>,
    /// Filtrer par direction
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flow_direction: Option<Vec<FlowDirection>>,
    /// Filtrer par identifiant de suivi
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracking_id: Option<String>,
    /// Filtrer par statut d'acquittement
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ack_status: Option<FlowAckStatus>,
}

/// Paramètres de recherche de flux (XP Z12-013 §5.3.2)
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchFlowParams {
    /// Nombre maximum de résultats (défaut 25, max 100)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// Filtres de recherche
    #[serde(rename = "where")]
    pub filters: SearchFlowFilters,
}

// ============================================================
// AFNOR XP Z12-013 Flow Service — Webhooks
// ============================================================

/// Événement déclencheur de webhook
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WebhookEvent {
    /// Nouveau flux entrant disponible
    #[serde(rename = "flow.received")]
    FlowReceived,
    /// Statut d'acquittement mis à jour
    #[serde(rename = "flow.ack.updated")]
    FlowAckUpdated,
}

/// Requête de création de webhook (POST /v1/webhooks)
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WebhookCreateRequest {
    /// URL de callback HTTPS à appeler
    pub callback_url: String,
    /// Événements auxquels s'abonner
    pub events: Vec<WebhookEvent>,
    /// Secret partagé pour la signature HMAC (optionnel)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret: Option<String>,
}

/// Requête de mise à jour de webhook (PATCH /v1/webhooks/{uid})
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WebhookUpdateRequest {
    /// URL de callback HTTPS (optionnel)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub callback_url: Option<String>,
    /// Événements auxquels s'abonner (optionnel)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub events: Option<Vec<WebhookEvent>>,
    /// Actif ou non
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
}

/// Webhook AFNOR (réponse)
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AfnorWebhook {
    pub uid: Option<String>,
    pub callback_url: Option<String>,
    pub events: Option<Vec<String>>,
    pub active: Option<bool>,
    pub secret: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

/// Réponse de liste de webhooks
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AfnorWebhookListResponse {
    pub items: Option<Vec<AfnorWebhook>>,
    pub total: Option<u64>,
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
    /// Parse un code d'environnement (insensible à la casse)
    pub fn from_code(code: &str) -> Option<Self> {
        match code.to_lowercase().as_str() {
            "dev" => Some(Self::Dev),
            "int" => Some(Self::Int),
            "rec" => Some(Self::Rec),
            "preprod" | "pre-prod" | "pre_prod" => Some(Self::PreProd),
            "prod" => Some(Self::Prod),
            _ => None,
        }
    }

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

// ============================================================
// AFNOR XP Z12-013 Directory Service — Search schemas
// ============================================================

/// Paramètres de recherche d'entreprises par SIREN (POST /v1/siren/search)
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchSirenParams {
    /// Filtres de recherche
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filters: Option<serde_json::Value>,
    /// Champs à retourner
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<Vec<String>>,
    /// Tri
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sorting: Option<Vec<SortParam>>,
    /// Nombre maximum de résultats
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// Nombre de résultats à ignorer
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<u32>,
}

/// Paramètre de tri
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SortParam {
    pub field: String,
    /// "asc" ou "desc"
    pub order: String,
}

/// Paramètres de recherche d'établissements par SIRET (POST /v1/siret/search)
pub type SearchSiretParams = SearchSirenParams;
