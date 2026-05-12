//! Modèle d'événement du bus interne.
//!
//! Un [`Event`] représente une transition dans le cycle de vie d'un flux
//! (facture, CDAR, e-reporting). Chaque événement est immuable et persisté
//! dans la table outbox `events` (cf. [`crate::store`]).
//!
//! La granularité retenue est de **14 événements de cycle de vie** ([`EventKind`]),
//! en correspondance 1-1 avec les variantes de [`pdp_core::model::FlowStatus`].
//! Le mapping est total : émettre un événement pour chaque transition de statut
//! produit un journal d'audit complet, exploitable pour répondre à
//! « qu'est-il arrivé à la facture X ? ».

use chrono::{DateTime, Utc};
use pdp_core::model::FlowStatus;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Nature de l'événement. 1-1 avec [`FlowStatus`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventKind {
    Received,
    Parsing,
    Parsed,
    Validating,
    Validated,
    Transforming,
    Transformed,
    Distributing,
    Distributed,
    WaitingAck,
    Acknowledged,
    Rejected,
    Cancelled,
    Error,
}

impl EventKind {
    /// Code court stable, utilisé pour la sérialisation en base et le routage.
    pub fn as_code(&self) -> &'static str {
        match self {
            Self::Received => "received",
            Self::Parsing => "parsing",
            Self::Parsed => "parsed",
            Self::Validating => "validating",
            Self::Validated => "validated",
            Self::Transforming => "transforming",
            Self::Transformed => "transformed",
            Self::Distributing => "distributing",
            Self::Distributed => "distributed",
            Self::WaitingAck => "waiting_ack",
            Self::Acknowledged => "acknowledged",
            Self::Rejected => "rejected",
            Self::Cancelled => "cancelled",
            Self::Error => "error",
        }
    }

    pub fn from_code(code: &str) -> Option<Self> {
        Some(match code {
            "received" => Self::Received,
            "parsing" => Self::Parsing,
            "parsed" => Self::Parsed,
            "validating" => Self::Validating,
            "validated" => Self::Validated,
            "transforming" => Self::Transforming,
            "transformed" => Self::Transformed,
            "distributing" => Self::Distributing,
            "distributed" => Self::Distributed,
            "waiting_ack" => Self::WaitingAck,
            "acknowledged" => Self::Acknowledged,
            "rejected" => Self::Rejected,
            "cancelled" => Self::Cancelled,
            "error" => Self::Error,
            _ => return None,
        })
    }

    /// `true` si l'événement représente un état terminal du flux.
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            Self::Acknowledged | Self::Rejected | Self::Cancelled | Self::Error
        )
    }
}

impl From<FlowStatus> for EventKind {
    fn from(s: FlowStatus) -> Self {
        match s {
            FlowStatus::Received => Self::Received,
            FlowStatus::Parsing => Self::Parsing,
            FlowStatus::Parsed => Self::Parsed,
            FlowStatus::Validating => Self::Validating,
            FlowStatus::Validated => Self::Validated,
            FlowStatus::Transforming => Self::Transforming,
            FlowStatus::Transformed => Self::Transformed,
            FlowStatus::Distributing => Self::Distributing,
            FlowStatus::Distributed => Self::Distributed,
            FlowStatus::WaitingAck => Self::WaitingAck,
            FlowStatus::Acknowledged => Self::Acknowledged,
            FlowStatus::Rejected => Self::Rejected,
            FlowStatus::Cancelled => Self::Cancelled,
            FlowStatus::Error => Self::Error,
        }
    }
}

/// Événement immuable du bus interne.
///
/// Une fois persisté, l'événement ne change plus. Les champs `sequence` et
/// `published_at` sont gérés par le store : ils sont absents tant que
/// l'événement n'est pas écrit en base.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: Uuid,
    pub flow_id: Uuid,
    pub kind: EventKind,
    /// Clef métier de la facture (`SIREN/NUMERO/ANNEE`) si connue.
    pub invoice_key: Option<String>,
    /// SIREN du tenant propriétaire (multi-tenant).
    pub tenant_siren: Option<String>,
    /// Identifiant de la route ayant produit l'événement.
    pub route_id: Option<String>,
    /// Étape métier (ex: « validation », « distribution »).
    pub step: Option<String>,
    /// Message lisible.
    pub message: Option<String>,
    /// Détail d'erreur si `kind = Error` (ou rejet).
    pub error_detail: Option<String>,
    /// Charge utile arbitraire (JSON) pour les détails spécifiques.
    pub payload: Option<serde_json::Value>,
    /// Horodatage métier (transition de statut côté pipeline).
    pub occurred_at: DateTime<Utc>,
    /// Numéro de séquence global (rempli par le store à l'insertion).
    pub sequence: Option<i64>,
}

impl Event {
    pub fn new(flow_id: Uuid, kind: EventKind) -> Self {
        Self {
            id: Uuid::new_v4(),
            flow_id,
            kind,
            invoice_key: None,
            tenant_siren: None,
            route_id: None,
            step: None,
            message: None,
            error_detail: None,
            payload: None,
            occurred_at: Utc::now(),
            sequence: None,
        }
    }

    pub fn with_invoice_key(mut self, key: impl Into<String>) -> Self {
        self.invoice_key = Some(key.into());
        self
    }

    pub fn with_tenant(mut self, siren: impl Into<String>) -> Self {
        self.tenant_siren = Some(siren.into());
        self
    }

    pub fn with_route(mut self, route_id: impl Into<String>) -> Self {
        self.route_id = Some(route_id.into());
        self
    }

    pub fn with_step(mut self, step: impl Into<String>) -> Self {
        self.step = Some(step.into());
        self
    }

    pub fn with_message(mut self, msg: impl Into<String>) -> Self {
        self.message = Some(msg.into());
        self
    }

    pub fn with_error(mut self, detail: impl Into<String>) -> Self {
        self.error_detail = Some(detail.into());
        self
    }

    pub fn with_payload(mut self, payload: serde_json::Value) -> Self {
        self.payload = Some(payload);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_kind_code_roundtrip() {
        for s in [
            FlowStatus::Received,
            FlowStatus::Parsing,
            FlowStatus::Parsed,
            FlowStatus::Validating,
            FlowStatus::Validated,
            FlowStatus::Transforming,
            FlowStatus::Transformed,
            FlowStatus::Distributing,
            FlowStatus::Distributed,
            FlowStatus::WaitingAck,
            FlowStatus::Acknowledged,
            FlowStatus::Rejected,
            FlowStatus::Cancelled,
            FlowStatus::Error,
        ] {
            let k: EventKind = s.into();
            assert_eq!(EventKind::from_code(k.as_code()), Some(k));
        }
    }

    #[test]
    fn terminal_kinds() {
        assert!(EventKind::Acknowledged.is_terminal());
        assert!(EventKind::Rejected.is_terminal());
        assert!(EventKind::Cancelled.is_terminal());
        assert!(EventKind::Error.is_terminal());
        assert!(!EventKind::Received.is_terminal());
        assert!(!EventKind::Validated.is_terminal());
    }

    #[test]
    fn builder_sets_fields() {
        let e = Event::new(Uuid::new_v4(), EventKind::Validated)
            .with_invoice_key("123456789/FA-001/2026")
            .with_tenant("123456789")
            .with_route("route-1")
            .with_step("validation")
            .with_message("XSD OK")
            .with_payload(serde_json::json!({"warnings": 0}));
        assert_eq!(e.invoice_key.as_deref(), Some("123456789/FA-001/2026"));
        assert_eq!(e.tenant_siren.as_deref(), Some("123456789"));
        assert_eq!(e.route_id.as_deref(), Some("route-1"));
        assert_eq!(e.step.as_deref(), Some("validation"));
        assert!(e.payload.is_some());
    }
}
