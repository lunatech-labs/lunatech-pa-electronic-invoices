//! Modèle de données PEPPOL.
//!
//! Identifiants de participants, types de documents, processus métier,
//! et structure des messages échangés via le réseau PEPPOL AS4.

use serde::{Deserialize, Serialize};

// ============================================================
// Identifiants PEPPOL
// ============================================================

/// Identifiant d'un participant PEPPOL (émetteur ou destinataire).
///
/// Format : `scheme::value`
/// Exemple France : `0009:12345678901234` (SIRET) ou `0002:123456789` (SIREN)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ParticipantId {
    /// Schéma d'identification (ex: "0009" = SIRET, "0002" = SIREN, "0088" = EAN)
    pub scheme: String,
    /// Valeur de l'identifiant
    pub value: String,
}

impl ParticipantId {
    pub fn new(scheme: &str, value: &str) -> Self {
        Self {
            scheme: scheme.to_string(),
            value: value.to_string(),
        }
    }

    /// Crée un ParticipantId à partir d'un SIREN (schéma 0002)
    pub fn from_siren(siren: &str) -> Self {
        Self::new("0002", siren)
    }

    /// Crée un ParticipantId à partir d'un SIRET (schéma 0009)
    pub fn from_siret(siret: &str) -> Self {
        Self::new("0009", siret)
    }

    /// Crée un ParticipantId à partir de l'adresse électronique française (schéma 0225)
    pub fn from_french_endpoint(endpoint: &str) -> Self {
        Self::new("0225", endpoint)
    }

    /// Format pour le SMP lookup : `scheme::value`
    pub fn to_smp_id(&self) -> String {
        format!("{}::{}", self.scheme, self.value)
    }

    /// Parse un identifiant au format `scheme::value`
    pub fn parse(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.splitn(2, "::").collect();
        if parts.len() == 2 {
            Some(Self::new(parts[0], parts[1]))
        } else {
            None
        }
    }
}

impl std::fmt::Display for ParticipantId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}::{}", self.scheme, self.value)
    }
}

/// Identifiant de type de document PEPPOL.
///
/// Définit le format et la version du document échangé.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DocumentTypeId {
    /// Schéma (ex: "busdox-docid-qns")
    pub scheme: String,
    /// Valeur complète du type de document
    pub value: String,
}

impl DocumentTypeId {
    pub fn new(scheme: &str, value: &str) -> Self {
        Self {
            scheme: scheme.to_string(),
            value: value.to_string(),
        }
    }

    /// Facture UBL 2.1 — Peppol BIS Billing 3.0
    pub fn ubl_invoice() -> Self {
        Self::new(
            "busdox-docid-qns",
            "urn:oasis:names:specification:ubl:schema:xsd:Invoice-2::Invoice##urn:cen.eu:en16931:2017#compliant#urn:fdc:peppol.eu:2017:poacc:billing:3.0::2.1",
        )
    }

    /// Avoir UBL 2.1 — Peppol BIS Billing 3.0
    pub fn ubl_credit_note() -> Self {
        Self::new(
            "busdox-docid-qns",
            "urn:oasis:names:specification:ubl:schema:xsd:CreditNote-2::CreditNote##urn:cen.eu:en16931:2017#compliant#urn:fdc:peppol.eu:2017:poacc:billing:3.0::2.1",
        )
    }

    /// Facture CII D16B — Peppol BIS Billing 3.0
    pub fn cii_invoice() -> Self {
        Self::new(
            "busdox-docid-qns",
            "urn:un:unece:uncefact:data:standard:CrossIndustryInvoice:100::CrossIndustryInvoice##urn:cen.eu:en16931:2017#compliant#urn:fdc:peppol.eu:2017:poacc:billing:3.0::D16B",
        )
    }

    /// CDAR D23B (Cycle de Vie)
    pub fn cdar() -> Self {
        Self::new(
            "busdox-docid-qns",
            "urn:un:unece:uncefact:data:standard:CrossDomainAcknowledgementAndResponse:100::CrossDomainAcknowledgementAndResponse##urn:fdc:peppol.eu:2017:poacc:billing:3.0::D23B",
        )
    }

    /// Format pour l'en-tête AS4 Action : `scheme::value`
    pub fn to_action(&self) -> String {
        format!("{}::{}", self.scheme, self.value)
    }
}

impl std::fmt::Display for DocumentTypeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}::{}", self.scheme, self.value)
    }
}

/// Identifiant de processus PEPPOL.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProcessId {
    /// Schéma (ex: "cenbii-procid-ubl")
    pub scheme: String,
    /// Valeur du processus
    pub value: String,
}

impl ProcessId {
    pub fn new(scheme: &str, value: &str) -> Self {
        Self {
            scheme: scheme.to_string(),
            value: value.to_string(),
        }
    }

    /// Processus Peppol BIS Billing 3.0
    pub fn billing() -> Self {
        Self::new(
            "cenbii-procid-ubl",
            "urn:fdc:peppol.eu:2017:poacc:billing:01:1.0",
        )
    }
}

impl std::fmt::Display for ProcessId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}::{}", self.scheme, self.value)
    }
}

// ============================================================
// Message PEPPOL
// ============================================================

/// Type de document transporté
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PeppolDocumentType {
    /// Facture (UBL Invoice ou CII CrossIndustryInvoice)
    Invoice,
    /// Avoir (UBL CreditNote)
    CreditNote,
    /// Statut de cycle de vie (CDAR)
    Cdar,
}

impl std::fmt::Display for PeppolDocumentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Invoice => write!(f, "Invoice"),
            Self::CreditNote => write!(f, "CreditNote"),
            Self::Cdar => write!(f, "CDAR"),
        }
    }
}

/// Message PEPPOL complet prêt à être envoyé ou reçu.
///
/// Contient l'enveloppe SBDH + le document métier (facture ou CDAR).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeppolMessage {
    /// Identifiant unique du message
    pub message_id: String,
    /// Émetteur (Corner 1)
    pub sender: ParticipantId,
    /// Destinataire (Corner 4)
    pub receiver: ParticipantId,
    /// Type de document
    pub document_type: PeppolDocumentType,
    /// Identifiant du type de document PEPPOL
    pub document_type_id: DocumentTypeId,
    /// Identifiant du processus
    pub process_id: ProcessId,
    /// Document XML (facture UBL/CII ou CDAR)
    pub payload: Vec<u8>,
    /// Date-heure de création
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Métadonnées additionnelles
    pub metadata: std::collections::HashMap<String, String>,
}

impl PeppolMessage {
    pub fn new(
        sender: ParticipantId,
        receiver: ParticipantId,
        document_type: PeppolDocumentType,
        document_type_id: DocumentTypeId,
        process_id: ProcessId,
        payload: Vec<u8>,
    ) -> Self {
        Self {
            message_id: uuid::Uuid::new_v4().to_string(),
            sender,
            receiver,
            document_type,
            document_type_id,
            process_id,
            payload,
            created_at: chrono::Utc::now(),
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Crée un message pour une facture UBL
    pub fn ubl_invoice(
        sender: ParticipantId,
        receiver: ParticipantId,
        xml: Vec<u8>,
    ) -> Self {
        Self::new(
            sender,
            receiver,
            PeppolDocumentType::Invoice,
            DocumentTypeId::ubl_invoice(),
            ProcessId::billing(),
            xml,
        )
    }

    /// Crée un message pour une facture CII
    pub fn cii_invoice(
        sender: ParticipantId,
        receiver: ParticipantId,
        xml: Vec<u8>,
    ) -> Self {
        Self::new(
            sender,
            receiver,
            PeppolDocumentType::Invoice,
            DocumentTypeId::cii_invoice(),
            ProcessId::billing(),
            xml,
        )
    }

    /// Crée un message pour un CDAR
    pub fn cdar(
        sender: ParticipantId,
        receiver: ParticipantId,
        xml: Vec<u8>,
    ) -> Self {
        Self::new(
            sender,
            receiver,
            PeppolDocumentType::Cdar,
            DocumentTypeId::cdar(),
            ProcessId::billing(),
            xml,
        )
    }
}

// ============================================================
// Résultat SMP Lookup
// ============================================================

/// Endpoint d'un Access Point découvert via SMP.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmpEndpoint {
    /// URL de l'Access Point destinataire
    pub endpoint_url: String,
    /// Certificat X.509 de l'AP destinataire (PEM ou DER base64)
    pub certificate: String,
    /// Profil de transport (doit être "peppol-transport-as4-v2_0")
    pub transport_profile: String,
    /// Date d'activation du service
    pub service_activation_date: Option<String>,
    /// Date d'expiration du service
    pub service_expiration_date: Option<String>,
}

/// Résultat d'un lookup SMP complet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmpLookupResult {
    /// Participant recherché
    pub participant: ParticipantId,
    /// Type de document
    pub document_type_id: DocumentTypeId,
    /// Processus
    pub process_id: ProcessId,
    /// Endpoint AS4 trouvé
    pub endpoint: SmpEndpoint,
}

// ============================================================
// Résultat d'envoi AS4
// ============================================================

/// Résultat d'un envoi AS4.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct As4SendResult {
    /// Identifiant du message AS4 (eb:MessageId)
    pub message_id: String,
    /// Succès ou échec
    pub success: bool,
    /// Message d'erreur éventuel
    pub error: Option<String>,
    /// Timestamp de l'envoi
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// URL de l'endpoint destinataire
    pub endpoint_url: String,
}

// ============================================================
// Configuration PEPPOL
// ============================================================

/// Configuration d'un Access Point PEPPOL.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeppolConfig {
    /// Identifiant de l'Access Point (CN du certificat, ex: "POP000123")
    pub ap_id: String,
    /// Identifiant du participant (notre PDP)
    pub participant_id: ParticipantId,
    /// URL de notre endpoint AS4 (pour la réception)
    pub endpoint_url: String,
    /// URL du SML (Service Metadata Locator)
    pub sml_url: String,
    /// Chemin vers le certificat PKCS#12 de l'AP
    pub certificate_path: String,
    /// Mot de passe du certificat
    pub certificate_password: Option<String>,
    /// Chemin vers le truststore (CA Peppol)
    pub truststore_path: Option<String>,
    /// Mode test (utilise le SML de test)
    pub test_mode: bool,
}

impl Default for PeppolConfig {
    fn default() -> Self {
        Self {
            ap_id: String::new(),
            participant_id: ParticipantId::from_siren("000000000"),
            endpoint_url: "https://localhost:8443/as4".to_string(),
            sml_url: "https://edelivery.tech.ec.europa.eu/edelivery-sml".to_string(),
            certificate_path: String::new(),
            certificate_password: None,
            truststore_path: None,
            test_mode: true,
        }
    }
}

impl PeppolConfig {
    /// Configuration pour l'environnement de test PEPPOL
    pub fn test() -> Self {
        Self {
            sml_url: "https://acc.edelivery.tech.ec.europa.eu/edelivery-sml".to_string(),
            test_mode: true,
            ..Default::default()
        }
    }

    /// Configuration pour l'environnement de production PEPPOL
    pub fn production() -> Self {
        Self {
            sml_url: "https://edelivery.tech.ec.europa.eu/edelivery-sml".to_string(),
            test_mode: false,
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_participant_id_siren() {
        let pid = ParticipantId::from_siren("123456789");
        assert_eq!(pid.scheme, "0002");
        assert_eq!(pid.value, "123456789");
        assert_eq!(pid.to_smp_id(), "0002::123456789");
        assert_eq!(pid.to_string(), "0002::123456789");
    }

    #[test]
    fn test_participant_id_siret() {
        let pid = ParticipantId::from_siret("12345678901234");
        assert_eq!(pid.scheme, "0009");
        assert_eq!(pid.to_smp_id(), "0009::12345678901234");
    }

    #[test]
    fn test_participant_id_french_endpoint() {
        let pid = ParticipantId::from_french_endpoint("123456789_FACTURES");
        assert_eq!(pid.scheme, "0225");
        assert_eq!(pid.value, "123456789_FACTURES");
    }

    #[test]
    fn test_participant_id_parse() {
        let pid = ParticipantId::parse("0002::123456789").unwrap();
        assert_eq!(pid.scheme, "0002");
        assert_eq!(pid.value, "123456789");
        assert!(ParticipantId::parse("invalid").is_none());
    }

    #[test]
    fn test_document_type_id_ubl_invoice() {
        let dt = DocumentTypeId::ubl_invoice();
        assert_eq!(dt.scheme, "busdox-docid-qns");
        assert!(dt.value.contains("Invoice"));
        assert!(dt.to_action().starts_with("busdox-docid-qns::"));
    }

    #[test]
    fn test_document_type_id_cii() {
        let dt = DocumentTypeId::cii_invoice();
        assert!(dt.value.contains("CrossIndustryInvoice"));
    }

    #[test]
    fn test_document_type_id_cdar() {
        let dt = DocumentTypeId::cdar();
        assert!(dt.value.contains("CrossDomainAcknowledgementAndResponse"));
    }

    #[test]
    fn test_process_id_billing() {
        let pid = ProcessId::billing();
        assert_eq!(pid.scheme, "cenbii-procid-ubl");
        assert!(pid.value.contains("billing"));
    }

    #[test]
    fn test_peppol_message_ubl_invoice() {
        let sender = ParticipantId::from_siren("111111111");
        let receiver = ParticipantId::from_siren("222222222");
        let msg = PeppolMessage::ubl_invoice(sender.clone(), receiver.clone(), b"<Invoice/>".to_vec());
        assert_eq!(msg.sender, sender);
        assert_eq!(msg.receiver, receiver);
        assert_eq!(msg.document_type, PeppolDocumentType::Invoice);
        assert!(!msg.message_id.is_empty());
    }

    #[test]
    fn test_peppol_message_cdar() {
        let sender = ParticipantId::from_siren("111111111");
        let receiver = ParticipantId::from_siren("222222222");
        let msg = PeppolMessage::cdar(sender, receiver, b"<CDAR/>".to_vec());
        assert_eq!(msg.document_type, PeppolDocumentType::Cdar);
    }

    #[test]
    fn test_peppol_config_default() {
        let cfg = PeppolConfig::default();
        assert!(cfg.test_mode);
        assert!(cfg.endpoint_url.contains("localhost"));
    }

    #[test]
    fn test_peppol_config_production() {
        let cfg = PeppolConfig::production();
        assert!(!cfg.test_mode);
        assert!(cfg.sml_url.contains("edelivery.tech.ec.europa.eu"));
    }
}
