//! # Système d'alertes pour les erreurs critiques
//!
//! Classifie automatiquement les erreurs par niveau de sévérité et émet
//! des alertes structurées pour les erreurs critiques (transformation PPF,
//! validation XSD, envoi SFTP, etc.).
//!
//! # Niveaux d'alerte
//!
//! - **Info** : événement normal, pas d'action requise
//! - **Warning** : anomalie non bloquante (validation business, doublon)
//! - **Critical** : échec bloquant nécessitant une intervention manuelle
//!
//! # Exemple
//!
//! ```rust
//! use pdp_core::alert::{AlertLevel, AlertClassifier};
//! use pdp_core::error::PdpError;
//!
//! let error = PdpError::TransformError {
//!     source_format: "FacturX".to_string(),
//!     target_format: "F1Full".to_string(),
//!     message: "XSD validation failed".to_string(),
//! };
//!
//! let level = AlertClassifier::classify(&error);
//! assert_eq!(level, AlertLevel::Critical);
//! ```

use std::path::PathBuf;

use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::endpoint::Producer;
use crate::error::{PdpError, PdpResult};
use crate::exchange::Exchange;

// ---------------------------------------------------------------------------
// AlertLevel
// ---------------------------------------------------------------------------

/// Niveau de sévérité d'une alerte
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AlertLevel {
    /// Événement normal, pas d'action requise
    Info,
    /// Anomalie non bloquante
    Warning,
    /// Échec bloquant nécessitant une intervention manuelle
    Critical,
}

impl std::fmt::Display for AlertLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AlertLevel::Info => write!(f, "INFO"),
            AlertLevel::Warning => write!(f, "WARNING"),
            AlertLevel::Critical => write!(f, "CRITICAL"),
        }
    }
}

// ---------------------------------------------------------------------------
// AlertClassifier
// ---------------------------------------------------------------------------

/// Classifie automatiquement les erreurs PDP par niveau de sévérité.
pub struct AlertClassifier;

impl AlertClassifier {
    /// Classifie une erreur PDP en niveau d'alerte.
    ///
    /// **Critical** (intervention manuelle requise) :
    /// - `TransformError` — échec de transformation (Factur-X → PPF, XSLT, etc.)
    /// - `SftpError` — impossible d'envoyer/recevoir via SFTP
    /// - `DistributionError` — échec de distribution vers la destination
    /// - `RoutingError` — impossible de router vers le destinataire
    /// - `Internal` — erreur système interne
    ///
    /// **Warning** (anomalie non bloquante) :
    /// - `ValidationError` — facture invalide (rejectable, pas un bug système)
    /// - `CdarError` — erreur de génération CDAR
    /// - `ParseError` — facture illisible (probablement un problème client)
    /// - `UnsupportedFormat` — format non pris en charge
    ///
    /// **Info** :
    /// - `ConfigError` — erreur de configuration (détectée au démarrage)
    /// - `TraceError` — erreur de traçabilité (non bloquante)
    /// - `RouteError` — erreur de construction de route
    /// - `InvoiceNotFound` — facture introuvable
    /// - `IoError` — erreur IO générique
    pub fn classify(error: &PdpError) -> AlertLevel {
        match error {
            // Critical : échecs système nécessitant une intervention
            PdpError::TransformError { .. } => AlertLevel::Critical,
            PdpError::SftpError(_) => AlertLevel::Critical,
            PdpError::DistributionError(_) => AlertLevel::Critical,
            PdpError::RoutingError(_) => AlertLevel::Critical,
            PdpError::Internal(_) => AlertLevel::Critical,

            // Warning : erreurs business (problème côté client)
            PdpError::ValidationError(_) => AlertLevel::Warning,
            PdpError::CdarError(_) => AlertLevel::Warning,
            PdpError::ParseError(_) => AlertLevel::Warning,
            PdpError::UnsupportedFormat(_) => AlertLevel::Warning,

            // Info : erreurs non bloquantes
            PdpError::ConfigError(_) => AlertLevel::Info,
            PdpError::TraceError(_) => AlertLevel::Info,
            PdpError::RouteError { .. } => AlertLevel::Info,
            PdpError::InvoiceNotFound(_) => AlertLevel::Info,
            PdpError::IoError(_) => AlertLevel::Warning,
        }
    }

    /// Classifie les erreurs accumulées sur un exchange et retourne le niveau le plus élevé.
    pub fn classify_exchange(exchange: &Exchange) -> AlertLevel {
        if exchange.errors.is_empty() {
            return AlertLevel::Info;
        }

        // Chercher des indices dans les messages d'erreur
        let mut max_level = AlertLevel::Info;

        for error in &exchange.errors {
            let level = Self::classify_from_message(&error.step, &error.message);
            if level == AlertLevel::Critical {
                return AlertLevel::Critical;
            }
            if level == AlertLevel::Warning && max_level == AlertLevel::Info {
                max_level = AlertLevel::Warning;
            }
        }

        max_level
    }

    /// Classifie à partir du nom du processor et du message d'erreur (heuristique).
    fn classify_from_message(step: &str, message: &str) -> AlertLevel {
        let step_lower = step.to_lowercase();
        let msg_lower = message.to_lowercase();

        // Critical : transformation, SFTP, distribution
        if step_lower.contains("transform")
            || step_lower.contains("flux1")
            || step_lower.contains("ppf")
            || step_lower.contains("sftp")
            || step_lower.contains("distribution")
            || step_lower.contains("routing")
            || msg_lower.contains("transformation")
            || msg_lower.contains("xsd")
            || msg_lower.contains("invalide selon xsd")
        {
            return AlertLevel::Critical;
        }

        // Warning : validation, parsing
        if step_lower.contains("valid")
            || step_lower.contains("parse")
            || step_lower.contains("cdar")
        {
            return AlertLevel::Warning;
        }

        AlertLevel::Warning
    }
}

// ---------------------------------------------------------------------------
// Alert (structure de données)
// ---------------------------------------------------------------------------

/// Alerte émise par le système
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    /// Niveau de sévérité
    pub level: AlertLevel,
    /// Identifiant de l'exchange concerné
    pub exchange_id: String,
    /// Identifiant du flux
    pub flow_id: String,
    /// SIREN du tenant (si disponible)
    pub tenant_siren: Option<String>,
    /// Nom du fichier source
    pub source_filename: Option<String>,
    /// Numéro de facture (si connu)
    pub invoice_number: Option<String>,
    /// Route qui a produit l'erreur
    pub route_id: Option<String>,
    /// Étape du pipeline qui a échoué
    pub failed_step: Option<String>,
    /// Message d'erreur principal
    pub error_message: String,
    /// Détail complet de l'erreur
    pub error_detail: Option<String>,
    /// Horodatage
    pub timestamp: String,
    /// Action recommandée
    pub recommended_action: String,
}

impl Alert {
    /// Crée une alerte à partir d'un exchange en erreur.
    pub fn from_exchange(exchange: &Exchange, level: AlertLevel) -> Self {
        let last_error = exchange.errors.last();

        let failed_step = last_error.map(|e| e.step.clone());
        let error_message = last_error
            .map(|e| e.message.clone())
            .unwrap_or_else(|| "Erreur inconnue".to_string());
        let error_detail = last_error.and_then(|e| e.detail.clone());

        let invoice_number = exchange.invoice.as_ref().map(|i| i.invoice_number.clone());
        let tenant_siren = exchange.get_property("tenant.siren").map(|s| s.to_string());
        let route_id = exchange.get_header("route.id").map(|s| s.to_string());

        let recommended_action = Self::recommend_action(&failed_step, &error_message);

        Self {
            level,
            exchange_id: exchange.id.to_string(),
            flow_id: exchange.flow_id.to_string(),
            tenant_siren,
            source_filename: exchange.source_filename.clone(),
            invoice_number,
            route_id,
            failed_step,
            error_message,
            error_detail,
            timestamp: Utc::now().to_rfc3339(),
            recommended_action,
        }
    }

    /// Recommande une action en fonction de l'erreur.
    fn recommend_action(step: &Option<String>, message: &str) -> String {
        let step_str = step.as_deref().unwrap_or("");
        let msg_lower = message.to_lowercase();

        if msg_lower.contains("xsd") || msg_lower.contains("invalide") {
            return "Vérifier la facture source et la transformation XSLT. \
                    Le fichier Flux 1 généré ne respecte pas le schéma XSD PPF. \
                    Possible régression XSLT ou facture source non conforme."
                .to_string();
        }

        if step_str.contains("Transform") || step_str.contains("Flux1") {
            return "Vérifier la transformation XSLT et le fichier source. \
                    Retraiter manuellement après correction."
                .to_string();
        }

        if msg_lower.contains("sftp") || step_str.contains("sftp") {
            return "Vérifier la connexion SFTP (réseau, certificats, droits). \
                    Le fichier sera retraité automatiquement au prochain cycle."
                .to_string();
        }

        if step_str.contains("Valid") || step_str.contains("valid") {
            return "Facture rejetée à la validation. \
                    Contacter l'émetteur pour correction et réémission."
                .to_string();
        }

        "Examiner le rapport d'erreur et retraiter manuellement si nécessaire.".to_string()
    }
}

// ---------------------------------------------------------------------------
// AlertErrorHandler — Error handler enrichi avec alertes
// ---------------------------------------------------------------------------

/// Error handler qui écrit les exchanges en erreur avec un rapport d'alerte JSON.
///
/// Pour chaque exchange en erreur :
/// 1. Classifie l'erreur par niveau de sévérité
/// 2. Écrit le body original dans le répertoire d'erreurs
/// 3. Écrit un rapport JSON détaillé (alerte + métadonnées)
/// 4. Émet un log structuré pour les systèmes de monitoring
/// 5. Envoie un webhook si configuré (alertes Critical uniquement)
pub struct AlertErrorHandler {
    /// Répertoire de destination des erreurs
    error_dir: PathBuf,
    /// URL de webhook pour les alertes critiques (optionnel)
    webhook_url: Option<String>,
    /// Niveau minimum pour déclencher le webhook
    min_webhook_level: AlertLevel,
}

impl AlertErrorHandler {
    /// Crée un nouveau handler d'alertes.
    pub fn new(error_dir: PathBuf) -> Self {
        Self {
            error_dir,
            webhook_url: None,
            min_webhook_level: AlertLevel::Critical,
        }
    }

    /// Configure le webhook pour les alertes.
    pub fn with_webhook(mut self, url: &str) -> Self {
        self.webhook_url = Some(url.to_string());
        self
    }

    /// Configure le niveau minimum pour le webhook.
    pub fn with_min_webhook_level(mut self, level: AlertLevel) -> Self {
        self.min_webhook_level = level;
        self
    }

    /// Écrit le rapport d'alerte et le fichier source dans le répertoire d'erreurs.
    fn write_alert_report(&self, exchange: &Exchange, alert: &Alert) {
        // Créer le sous-répertoire par niveau
        let level_dir = self.error_dir.join(match alert.level {
            AlertLevel::Critical => "critical",
            AlertLevel::Warning => "warning",
            AlertLevel::Info => "info",
        });

        if let Err(e) = std::fs::create_dir_all(&level_dir) {
            tracing::error!(
                path = %level_dir.display(),
                error = %e,
                "Impossible de créer le répertoire d'alertes"
            );
            return;
        }

        let filename = exchange
            .source_filename
            .as_deref()
            .unwrap_or("unknown");
        let base_name = format!("{}_{}", exchange.id, filename);

        // Écrire le body original
        let body_path = level_dir.join(&base_name);
        if let Err(e) = std::fs::write(&body_path, &exchange.body) {
            tracing::error!(
                path = %body_path.display(),
                error = %e,
                "Impossible d'écrire le fichier en erreur"
            );
            return;
        }

        // Écrire le rapport JSON
        let report = serde_json::json!({
            "alert": alert,
            "exchange": {
                "id": exchange.id.to_string(),
                "flow_id": exchange.flow_id.to_string(),
                "source_filename": exchange.source_filename,
                "status": format!("{}", exchange.status),
                "headers": exchange.headers,
                "properties": exchange.properties,
                "errors": exchange.errors.iter().map(|e| serde_json::json!({
                    "timestamp": e.timestamp.to_rfc3339(),
                    "step": e.step,
                    "message": e.message,
                    "detail": e.detail,
                })).collect::<Vec<_>>(),
            },
        });

        let report_path = level_dir.join(format!("{}.alert.json", base_name));
        if let Err(e) = std::fs::write(
            &report_path,
            serde_json::to_string_pretty(&report).unwrap_or_default(),
        ) {
            tracing::error!(
                path = %report_path.display(),
                error = %e,
                "Impossible d'écrire le rapport d'alerte"
            );
            return;
        }

        tracing::info!(
            alert_level = %alert.level,
            exchange_id = %exchange.id,
            path = %report_path.display(),
            "Rapport d'alerte écrit"
        );
    }

    /// Émet un log structuré pour les systèmes de monitoring (Datadog, Grafana, etc.)
    fn emit_structured_log(&self, alert: &Alert) {
        match alert.level {
            AlertLevel::Critical => {
                tracing::error!(
                    alert_level = "CRITICAL",
                    alert_type = "pipeline_error",
                    exchange_id = %alert.exchange_id,
                    tenant_siren = ?alert.tenant_siren,
                    source_filename = ?alert.source_filename,
                    invoice_number = ?alert.invoice_number,
                    route_id = ?alert.route_id,
                    failed_step = ?alert.failed_step,
                    error = %alert.error_message,
                    action = %alert.recommended_action,
                    "🚨 ALERTE CRITIQUE — Intervention manuelle requise"
                );
            }
            AlertLevel::Warning => {
                tracing::warn!(
                    alert_level = "WARNING",
                    alert_type = "pipeline_error",
                    exchange_id = %alert.exchange_id,
                    tenant_siren = ?alert.tenant_siren,
                    source_filename = ?alert.source_filename,
                    invoice_number = ?alert.invoice_number,
                    failed_step = ?alert.failed_step,
                    error = %alert.error_message,
                    "⚠️ Alerte — Erreur non critique"
                );
            }
            AlertLevel::Info => {
                tracing::info!(
                    alert_level = "INFO",
                    alert_type = "pipeline_error",
                    exchange_id = %alert.exchange_id,
                    error = %alert.error_message,
                    "ℹ️ Erreur informative"
                );
            }
        }
    }

    /// Envoie un webhook HTTP pour les alertes critiques.
    async fn send_webhook(&self, alert: &Alert) {
        let url = match &self.webhook_url {
            Some(url) => url,
            None => return,
        };

        // Vérifier le niveau minimum
        let should_send = match (&self.min_webhook_level, &alert.level) {
            (AlertLevel::Critical, AlertLevel::Critical) => true,
            (AlertLevel::Warning, AlertLevel::Critical | AlertLevel::Warning) => true,
            (AlertLevel::Info, _) => true,
            _ => false,
        };

        if !should_send {
            return;
        }

        let payload = serde_json::to_string(alert).unwrap_or_default();

        // Utiliser un HTTP client minimal (pas de dépendance externe lourde)
        // On spawn en background pour ne pas bloquer le pipeline
        let url = url.clone();
        tokio::spawn(async move {
            match send_webhook_request(&url, &payload).await {
                Ok(_) => {
                    tracing::info!(
                        url = %url,
                        "Webhook d'alerte envoyé"
                    );
                }
                Err(e) => {
                    tracing::error!(
                        url = %url,
                        error = %e,
                        "Échec de l'envoi du webhook d'alerte"
                    );
                }
            }
        });
    }
}

/// Envoie une requête HTTP POST avec le payload JSON.
/// Utilise une connexion TCP brute pour éviter une dépendance sur reqwest/hyper.
async fn send_webhook_request(url: &str, payload: &str) -> Result<(), String> {
    // Parse l'URL pour extraire host et path
    let url = url.trim();
    let (scheme, rest) = if url.starts_with("https://") {
        ("https", &url[8..])
    } else if url.starts_with("http://") {
        ("http", &url[7..])
    } else {
        return Err(format!("URL invalide (doit commencer par http:// ou https://): {}", url));
    };

    let (host_port, path) = match rest.find('/') {
        Some(i) => (&rest[..i], &rest[i..]),
        None => (rest, "/"),
    };

    let (host, port) = match host_port.find(':') {
        Some(i) => (
            &host_port[..i],
            host_port[i + 1..].parse::<u16>().map_err(|e| format!("Port invalide: {}", e))?,
        ),
        None => (
            host_port,
            if scheme == "https" { 443 } else { 80 },
        ),
    };

    // Pour HTTPS il faudrait TLS — on log un warning et on fait du HTTP simple
    if scheme == "https" {
        tracing::warn!(
            "Webhook HTTPS non supporté sans dépendance TLS. \
             Utilisez HTTP en dev ou ajoutez reqwest pour la production."
        );
        return Err("HTTPS non supporté sans dépendance TLS".to_string());
    }

    let addr = format!("{}:{}", host, port);
    let stream = tokio::net::TcpStream::connect(&addr)
        .await
        .map_err(|e| format!("Connexion webhook {}: {}", addr, e))?;

    let request = format!(
        "POST {} HTTP/1.1\r\n\
         Host: {}\r\n\
         Content-Type: application/json\r\n\
         Content-Length: {}\r\n\
         Connection: close\r\n\
         \r\n\
         {}",
        path,
        host_port,
        payload.len(),
        payload
    );

    use tokio::io::AsyncWriteExt;
    let mut stream = stream;
    stream
        .write_all(request.as_bytes())
        .await
        .map_err(|e| format!("Écriture webhook: {}", e))?;
    stream
        .flush()
        .await
        .map_err(|e| format!("Flush webhook: {}", e))?;

    Ok(())
}

#[async_trait]
impl Producer for AlertErrorHandler {
    fn name(&self) -> &str {
        "AlertErrorHandler"
    }

    async fn send(&self, exchange: Exchange) -> PdpResult<Exchange> {
        let level = AlertClassifier::classify_exchange(&exchange);
        let alert = Alert::from_exchange(&exchange, level);

        // 1. Écrire le rapport d'alerte sur disque
        self.write_alert_report(&exchange, &alert);

        // 2. Émettre un log structuré
        self.emit_structured_log(&alert);

        // 3. Envoyer le webhook si configuré
        self.send_webhook(&alert).await;

        Ok(exchange)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::PdpError;

    #[test]
    fn test_classify_transform_error_is_critical() {
        let err = PdpError::TransformError {
            source_format: "FacturX".to_string(),
            target_format: "F1Full".to_string(),
            message: "XSD validation failed".to_string(),
        };
        assert_eq!(AlertClassifier::classify(&err), AlertLevel::Critical);
    }

    #[test]
    fn test_classify_sftp_error_is_critical() {
        let err = PdpError::SftpError("Connection refused".to_string());
        assert_eq!(AlertClassifier::classify(&err), AlertLevel::Critical);
    }

    #[test]
    fn test_classify_distribution_error_is_critical() {
        let err = PdpError::DistributionError("timeout".to_string());
        assert_eq!(AlertClassifier::classify(&err), AlertLevel::Critical);
    }

    #[test]
    fn test_classify_routing_error_is_critical() {
        let err = PdpError::RoutingError("no route".to_string());
        assert_eq!(AlertClassifier::classify(&err), AlertLevel::Critical);
    }

    #[test]
    fn test_classify_internal_error_is_critical() {
        let err = PdpError::Internal("panic".to_string());
        assert_eq!(AlertClassifier::classify(&err), AlertLevel::Critical);
    }

    #[test]
    fn test_classify_validation_error_is_warning() {
        let err = PdpError::ValidationError("BR-FR-01 failed".to_string());
        assert_eq!(AlertClassifier::classify(&err), AlertLevel::Warning);
    }

    #[test]
    fn test_classify_parse_error_is_warning() {
        let err = PdpError::ParseError("invalid XML".to_string());
        assert_eq!(AlertClassifier::classify(&err), AlertLevel::Warning);
    }

    #[test]
    fn test_classify_cdar_error_is_warning() {
        let err = PdpError::CdarError("generation failed".to_string());
        assert_eq!(AlertClassifier::classify(&err), AlertLevel::Warning);
    }

    #[test]
    fn test_classify_config_error_is_info() {
        let err = PdpError::ConfigError("missing field".to_string());
        assert_eq!(AlertClassifier::classify(&err), AlertLevel::Info);
    }

    #[test]
    fn test_classify_trace_error_is_info() {
        let err = PdpError::TraceError("ES down".to_string());
        assert_eq!(AlertClassifier::classify(&err), AlertLevel::Info);
    }

    #[test]
    fn test_classify_exchange_no_errors() {
        let exchange = Exchange::new(b"<Invoice/>".to_vec());
        assert_eq!(AlertClassifier::classify_exchange(&exchange), AlertLevel::Info);
    }

    #[test]
    fn test_classify_exchange_with_critical_error() {
        let mut exchange = Exchange::new(b"<Invoice/>".to_vec());
        let err = PdpError::TransformError {
            source_format: "FacturX".to_string(),
            target_format: "F1Full".to_string(),
            message: "XSLT failed".to_string(),
        };
        exchange.add_error("PpfFlux1Processor", &err);
        assert_eq!(
            AlertClassifier::classify_exchange(&exchange),
            AlertLevel::Critical
        );
    }

    #[test]
    fn test_classify_exchange_with_warning_error() {
        let mut exchange = Exchange::new(b"<Invoice/>".to_vec());
        let err = PdpError::ValidationError("BR-FR-01 failed".to_string());
        exchange.add_error("ValidateProcessor", &err);
        assert_eq!(
            AlertClassifier::classify_exchange(&exchange),
            AlertLevel::Warning
        );
    }

    #[test]
    fn test_alert_from_exchange() {
        let mut exchange = Exchange::new(b"<Invoice/>".to_vec())
            .with_filename("facture_001.xml");
        exchange.set_property("tenant.siren", "123456789");
        exchange.set_header("route.id", "tenant-123456789");

        let err = PdpError::TransformError {
            source_format: "FacturX".to_string(),
            target_format: "F1Full".to_string(),
            message: "Le Flux 1 PPF généré est invalide selon XSD".to_string(),
        };
        exchange.add_error("PpfFlux1Processor", &err);

        let alert = Alert::from_exchange(&exchange, AlertLevel::Critical);

        assert_eq!(alert.level, AlertLevel::Critical);
        assert_eq!(alert.tenant_siren.as_deref(), Some("123456789"));
        assert_eq!(alert.source_filename.as_deref(), Some("facture_001.xml"));
        assert_eq!(alert.route_id.as_deref(), Some("tenant-123456789"));
        assert_eq!(alert.failed_step.as_deref(), Some("PpfFlux1Processor"));
        assert!(alert.error_message.contains("invalide selon XSD"));
        assert!(alert.recommended_action.contains("XSD"));
    }

    #[test]
    fn test_recommend_action_xsd() {
        let action =
            Alert::recommend_action(&Some("PpfFlux1Processor".to_string()), "invalide selon XSD PPF");
        assert!(action.contains("XSD"));
        assert!(action.contains("XSLT"));
    }

    #[test]
    fn test_recommend_action_sftp() {
        let action =
            Alert::recommend_action(&Some("sftp-producer".to_string()), "SFTP connection refused");
        assert!(action.contains("SFTP"));
        assert!(action.contains("connexion"));
    }

    #[test]
    fn test_recommend_action_validation() {
        let action = Alert::recommend_action(
            &Some("ValidateProcessor".to_string()),
            "BR-FR-01 failed",
        );
        assert!(action.contains("validation"));
    }

    #[tokio::test]
    async fn test_alert_error_handler_writes_report() {
        let dir = tempfile::tempdir().unwrap();
        let handler = AlertErrorHandler::new(dir.path().join("errors"));

        let mut exchange = Exchange::new(b"<Invoice>test</Invoice>".to_vec())
            .with_filename("facture_critical.xml");
        exchange.set_property("tenant.siren", "123456789");

        let err = PdpError::TransformError {
            source_format: "FacturX".to_string(),
            target_format: "F1Full".to_string(),
            message: "XSD validation échouée".to_string(),
        };
        exchange.add_error("PpfFlux1Processor", &err);

        let result = handler.send(exchange.clone()).await;
        assert!(result.is_ok());

        // Vérifier que le répertoire critical/ existe
        let critical_dir = dir.path().join("errors/critical");
        assert!(critical_dir.exists(), "Le répertoire critical/ doit exister");

        // Vérifier le body
        let body_file = critical_dir.join(format!(
            "{}_facture_critical.xml",
            exchange.id
        ));
        assert!(body_file.exists(), "Le fichier body doit exister");
        let body = std::fs::read(&body_file).unwrap();
        assert_eq!(body, b"<Invoice>test</Invoice>");

        // Vérifier le rapport JSON
        let report_file = critical_dir.join(format!(
            "{}_facture_critical.xml.alert.json",
            exchange.id
        ));
        assert!(report_file.exists(), "Le rapport d'alerte doit exister");

        let report: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&report_file).unwrap()).unwrap();

        assert_eq!(report["alert"]["level"], "critical");
        assert_eq!(report["alert"]["tenant_siren"], "123456789");
        assert!(report["alert"]["error_message"]
            .as_str()
            .unwrap()
            .contains("XSD"));
        assert!(report["alert"]["recommended_action"]
            .as_str()
            .unwrap()
            .len() > 10);
        assert!(report["exchange"]["errors"].as_array().unwrap().len() > 0);
    }

    #[tokio::test]
    async fn test_alert_error_handler_warning_goes_to_warning_dir() {
        let dir = tempfile::tempdir().unwrap();
        let handler = AlertErrorHandler::new(dir.path().join("errors"));

        let mut exchange = Exchange::new(b"<Invoice/>".to_vec())
            .with_filename("facture_warn.xml");
        let err = PdpError::ValidationError("BR-FR-01 failed".to_string());
        exchange.add_error("ValidateProcessor", &err);

        let result = handler.send(exchange.clone()).await;
        assert!(result.is_ok());

        let warning_dir = dir.path().join("errors/warning");
        assert!(warning_dir.exists(), "Le répertoire warning/ doit exister");

        let body_file = warning_dir.join(format!("{}_facture_warn.xml", exchange.id));
        assert!(body_file.exists());
    }

    #[test]
    fn test_alert_level_display() {
        assert_eq!(AlertLevel::Critical.to_string(), "CRITICAL");
        assert_eq!(AlertLevel::Warning.to_string(), "WARNING");
        assert_eq!(AlertLevel::Info.to_string(), "INFO");
    }

    #[test]
    fn test_alert_serialization() {
        let alert = Alert {
            level: AlertLevel::Critical,
            exchange_id: "123".to_string(),
            flow_id: "456".to_string(),
            tenant_siren: Some("123456789".to_string()),
            source_filename: Some("test.xml".to_string()),
            invoice_number: Some("FA-001".to_string()),
            route_id: Some("tenant-123456789".to_string()),
            failed_step: Some("PpfFlux1Processor".to_string()),
            error_message: "XSD failed".to_string(),
            error_detail: None,
            timestamp: "2025-01-01T00:00:00Z".to_string(),
            recommended_action: "Fix it".to_string(),
        };

        let json = serde_json::to_string(&alert).unwrap();
        assert!(json.contains("\"level\":\"critical\""));
        assert!(json.contains("\"tenant_siren\":\"123456789\""));

        // Désérialisation round-trip
        let parsed: Alert = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.level, AlertLevel::Critical);
        assert_eq!(parsed.exchange_id, "123");
    }
}
