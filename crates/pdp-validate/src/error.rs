use serde::{Deserialize, Serialize};

/// Niveau de sévérité d'un problème de validation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationLevel {
    /// Erreur fatale : le document est invalide
    Fatal,
    /// Erreur : règle métier non respectée
    Error,
    /// Avertissement : recommandation non suivie
    Warning,
    /// Information
    Info,
}

impl std::fmt::Display for ValidationLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationLevel::Fatal => write!(f, "FATAL"),
            ValidationLevel::Error => write!(f, "ERROR"),
            ValidationLevel::Warning => write!(f, "WARNING"),
            ValidationLevel::Info => write!(f, "INFO"),
        }
    }
}

/// Un problème individuel détecté lors de la validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationIssue {
    /// Niveau de sévérité
    pub level: ValidationLevel,
    /// Identifiant de la règle (ex: "BR-01", "BR-FR-03", "XSD-001")
    pub rule_id: String,
    /// Message d'erreur
    pub message: String,
    /// Emplacement dans le document (XPath ou ligne)
    pub location: Option<String>,
    /// Source de la validation (XSD, EN16931, BR-FR, etc.)
    pub source: String,
}

impl std::fmt::Display for ValidationIssue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}][{}] {} - {}", self.level, self.source, self.rule_id, self.message)?;
        if let Some(ref loc) = self.location {
            write!(f, " (at {})", loc)?;
        }
        Ok(())
    }
}

/// Rapport complet de validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    /// Type de document validé
    pub document_type: String,
    /// Tous les problèmes détectés
    pub issues: Vec<ValidationIssue>,
}

impl ValidationReport {
    pub fn new(document_type: &str) -> Self {
        Self {
            document_type: document_type.to_string(),
            issues: Vec::new(),
        }
    }

    pub fn add_issue(&mut self, issue: ValidationIssue) {
        self.issues.push(issue);
    }

    pub fn merge(&mut self, other: ValidationReport) {
        self.issues.extend(other.issues);
    }

    /// Le document est-il valide ? (pas d'erreurs fatales ni d'erreurs)
    pub fn is_valid(&self) -> bool {
        !self.issues.iter().any(|i| matches!(i.level, ValidationLevel::Fatal | ValidationLevel::Error))
    }

    /// Nombre d'erreurs fatales
    pub fn fatal_count(&self) -> usize {
        self.issues.iter().filter(|i| i.level == ValidationLevel::Fatal).count()
    }

    /// Nombre d'erreurs
    pub fn error_count(&self) -> usize {
        self.issues.iter().filter(|i| i.level == ValidationLevel::Error).count()
    }

    /// Nombre d'avertissements
    pub fn warning_count(&self) -> usize {
        self.issues.iter().filter(|i| i.level == ValidationLevel::Warning).count()
    }

    /// Résumé textuel
    pub fn summary(&self) -> String {
        format!(
            "{}: {} (fatals={}, erreurs={}, warnings={})",
            self.document_type,
            if self.is_valid() { "VALIDE" } else { "INVALIDE" },
            self.fatal_count(),
            self.error_count(),
            self.warning_count(),
        )
    }
}
