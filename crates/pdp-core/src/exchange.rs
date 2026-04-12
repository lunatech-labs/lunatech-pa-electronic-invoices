use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::PdpError;
use crate::model::{FlowStatus, InvoiceData};

/// L'Exchange est le message qui circule dans le pipeline.
/// Inspiré du concept d'Exchange d'Apache Camel.
/// Il contient le body (données brutes ou parsées), des headers,
/// des propriétés, et l'état du flux.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Exchange {
    /// Identifiant unique de l'exchange
    pub id: Uuid,
    /// Identifiant du flux (peut regrouper plusieurs exchanges)
    pub flow_id: Uuid,
    /// Corps du message : données brutes
    pub body: Vec<u8>,
    /// Nom du fichier source
    pub source_filename: Option<String>,
    /// Headers (métadonnées de transport)
    pub headers: HashMap<String, String>,
    /// Propriétés (métadonnées de traitement)
    pub properties: HashMap<String, String>,
    /// Données de facture parsées (rempli après parsing)
    pub invoice: Option<InvoiceData>,
    /// Statut courant du flux
    pub status: FlowStatus,
    /// Erreurs accumulées
    pub errors: Vec<ExchangeError>,
    /// Timestamp de création
    pub created_at: DateTime<Utc>,
    /// Timestamp de dernière modification
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeError {
    pub timestamp: DateTime<Utc>,
    pub step: String,
    pub message: String,
    pub detail: Option<String>,
}

impl Exchange {
    pub fn new(body: Vec<u8>) -> Self {
        let now = Utc::now();
        let flow_id = Uuid::new_v4();
        Self {
            id: Uuid::new_v4(),
            flow_id,
            body,
            source_filename: None,
            headers: HashMap::new(),
            properties: HashMap::new(),
            invoice: None,
            status: FlowStatus::Received,
            errors: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn with_filename(mut self, filename: &str) -> Self {
        self.source_filename = Some(filename.to_string());
        self.headers.insert("source.filename".to_string(), filename.to_string());
        self
    }

    pub fn with_flow_id(mut self, flow_id: Uuid) -> Self {
        self.flow_id = flow_id;
        self
    }

    pub fn set_header(&mut self, key: &str, value: &str) {
        self.headers.insert(key.to_string(), value.to_string());
        self.updated_at = Utc::now();
    }

    pub fn get_header(&self, key: &str) -> Option<&String> {
        self.headers.get(key)
    }

    pub fn set_property(&mut self, key: &str, value: &str) {
        self.properties.insert(key.to_string(), value.to_string());
        self.updated_at = Utc::now();
    }

    pub fn get_property(&self, key: &str) -> Option<&String> {
        self.properties.get(key)
    }

    pub fn set_status(&mut self, status: FlowStatus) {
        self.status = status;
        self.updated_at = Utc::now();
    }

    pub fn add_error(&mut self, step: &str, error: &PdpError) {
        self.errors.push(ExchangeError {
            timestamp: Utc::now(),
            step: step.to_string(),
            message: error.to_string(),
            detail: Some(format!("{:?}", error)),
        });
        self.status = FlowStatus::Error;
        self.updated_at = Utc::now();
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Retourne le body comme string UTF-8
    pub fn body_as_str(&self) -> Result<&str, PdpError> {
        std::str::from_utf8(&self.body)
            .map_err(|e| PdpError::ParseError(format!("Body n'est pas UTF-8: {}", e)))
    }

    /// Remplace le body
    pub fn set_body(&mut self, body: Vec<u8>) {
        self.body = body;
        self.updated_at = Utc::now();
    }

    /// Retourne le SIREN du tenant associe a cet exchange
    pub fn tenant_siren(&self) -> Option<&str> {
        self.get_property("tenant.siren").map(|s| s.as_str())
    }

    /// Definit le SIREN du tenant pour cet exchange
    pub fn set_tenant_siren(&mut self, siren: &str) {
        self.set_property("tenant.siren", siren);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tenant_siren_none_by_default() {
        let exchange = Exchange::new(b"test".to_vec());
        assert!(exchange.tenant_siren().is_none());
    }

    #[test]
    fn test_set_and_get_tenant_siren() {
        let mut exchange = Exchange::new(b"test".to_vec());
        exchange.set_tenant_siren("123456789");
        assert_eq!(exchange.tenant_siren(), Some("123456789"));
    }

    #[test]
    fn test_tenant_siren_overwrites() {
        let mut exchange = Exchange::new(b"test".to_vec());
        exchange.set_tenant_siren("111111111");
        exchange.set_tenant_siren("222222222");
        assert_eq!(exchange.tenant_siren(), Some("222222222"));
    }
}
