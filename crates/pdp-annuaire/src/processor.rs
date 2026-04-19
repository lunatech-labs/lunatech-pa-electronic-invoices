//! Processor de validation annuaire PPF pour le pipeline PDP.
//!
//! Vérifie que le vendeur (et l'acheteur en émission) existe et est actif
//! dans l'annuaire local avant d'accepter une facture.
//!
//! Conforme à la règle AFNOR G1.63 :
//! "Le SIREN du vendeur (BT-30) et de l'acheteur (BT-47) sont obligatoires
//! et doivent exister dans l'annuaire."

use std::sync::Arc;

use async_trait::async_trait;
use tracing;

use pdp_core::error::{PdpError, PdpResult};
use pdp_core::exchange::Exchange;
use pdp_core::processor::Processor;

use crate::service::AnnuaireService;

/// Mode de validation : émission vérifie vendeur + acheteur,
/// réception vérifie vendeur uniquement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationMode {
    /// PDP émettrice : vérifie vendeur ET acheteur
    Emission,
    /// PDP réceptrice : vérifie vendeur uniquement
    Reception,
}

/// Processor de validation des parties dans l'annuaire PPF (G1.63).
///
/// Position dans le pipeline : après parsing et validation, avant Flux 1.
/// Si le service annuaire n'est pas configuré (pas de PostgreSQL), le
/// processor est un no-op.
pub struct AnnuaireValidationProcessor {
    service: Option<Arc<AnnuaireService>>,
    mode: ValidationMode,
}

impl AnnuaireValidationProcessor {
    pub fn new(service: Option<Arc<AnnuaireService>>, mode: ValidationMode) -> Self {
        Self { service, mode }
    }
}

#[async_trait]
impl Processor for AnnuaireValidationProcessor {
    fn name(&self) -> &str {
        "AnnuaireValidationProcessor"
    }

    async fn process(&self, mut exchange: Exchange) -> PdpResult<Exchange> {
        // Skip si pas de service annuaire (PostgreSQL non configuré)
        let service = match &self.service {
            Some(s) => s,
            None => return Ok(exchange),
        };

        // Skip si pas d'invoice parsée
        if exchange.invoice.is_none() {
            return Ok(exchange);
        }

        // Skip si document CDAR ou e-reporting
        if let Some(doc_type) = exchange.get_property("document.type") {
            match doc_type.as_str() {
                "CDAR" | "EREPORTING" => {
                    tracing::debug!(
                        exchange_id = %exchange.id,
                        "AnnuaireValidationProcessor: skip ({})", doc_type
                    );
                    return Ok(exchange);
                }
                _ => {}
            }
        }

        // Extraire les SIREN (clonés pour éviter les conflits de borrow)
        let seller_siren = exchange.invoice.as_ref().unwrap()
            .seller_siret.as_deref()
            .map(|s| if s.len() >= 9 { s[..9].to_string() } else { s.to_string() });
        let buyer_siren = exchange.invoice.as_ref().unwrap()
            .buyer_siret.as_deref()
            .map(|s| if s.len() >= 9 { s[..9].to_string() } else { s.to_string() });

        // Vérifier le vendeur (émission ET réception)
        if let Some(ref siren) = seller_siren {
            match service.get_unite_legale(siren).await {
                Ok(Some(ul)) => {
                    if ul.statut != crate::model::Statut::Actif {
                        tracing::warn!(
                            exchange_id = %exchange.id,
                            siren = %siren,
                            nom = %ul.nom,
                            "Vendeur inactif dans l'annuaire PPF (BR-FR-10)"
                        );
                        exchange.add_error(
                            "annuaire-validation",
                            &PdpError::ValidationError(format!(
                                "Vendeur inactif dans l'annuaire PPF (BR-FR-10) (SIREN: {})", siren
                            )),
                        );
                    }
                }
                Ok(None) => {
                    tracing::warn!(
                        exchange_id = %exchange.id,
                        siren = %siren,
                        "Vendeur inconnu dans l'annuaire PPF (BR-FR-10)"
                    );
                    exchange.add_error(
                        "annuaire-validation",
                        &PdpError::ValidationError(format!(
                            "Vendeur inconnu dans l'annuaire PPF (BR-FR-10) (SIREN: {})", siren
                        )),
                    );
                }
                Err(e) => {
                    tracing::error!(
                        exchange_id = %exchange.id,
                        error = %e,
                        "Erreur de connexion annuaire lors de la vérification vendeur"
                    );
                    // On ne bloque pas en cas d'erreur technique — on continue
                }
            }
        }

        // Vérifier l'acheteur (émission uniquement)
        if self.mode == ValidationMode::Emission {
            if let Some(ref siren) = buyer_siren {
                match service.get_unite_legale(siren).await {
                    Ok(Some(ul)) => {
                        if ul.statut != crate::model::Statut::Actif {
                            tracing::warn!(
                                exchange_id = %exchange.id,
                                siren = %siren,
                                nom = %ul.nom,
                                "Destinataire inactif dans l'annuaire PPF (BR-FR-11)"
                            );
                            exchange.add_error(
                                "annuaire-validation",
                                &PdpError::ValidationError(format!(
                                    "Destinataire inactif dans l'annuaire PPF (BR-FR-11) (SIREN: {})", siren
                                )),
                            );
                        }
                    }
                    Ok(None) => {
                        tracing::warn!(
                            exchange_id = %exchange.id,
                            siren = %siren,
                            "Destinataire inconnu dans l'annuaire PPF (BR-FR-11)"
                        );
                        exchange.add_error(
                            "annuaire-validation",
                            &PdpError::ValidationError(format!(
                                "Destinataire inconnu dans l'annuaire PPF (BR-FR-11) (SIREN: {})", siren
                            )),
                        );
                    }
                    Err(e) => {
                        tracing::error!(
                            exchange_id = %exchange.id,
                            error = %e,
                            "Erreur de connexion annuaire lors de la vérification acheteur"
                        );
                    }
                }
            }
        }

        Ok(exchange)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pdp_core::model::{InvoiceData, InvoiceFormat};

    #[tokio::test]
    async fn test_skip_when_no_service() {
        let processor = AnnuaireValidationProcessor::new(None, ValidationMode::Emission);
        let mut exchange = Exchange::new(b"<test/>".to_vec());
        exchange.invoice = Some(InvoiceData::new("F-001".to_string(), InvoiceFormat::CII));

        let result = processor.process(exchange).await.unwrap();
        assert!(!result.has_errors(), "Pas d'erreur si pas de service annuaire");
    }

    #[tokio::test]
    async fn test_skip_when_no_invoice() {
        let processor = AnnuaireValidationProcessor::new(None, ValidationMode::Emission);
        let exchange = Exchange::new(b"<test/>".to_vec());

        let result = processor.process(exchange).await.unwrap();
        assert!(!result.has_errors());
    }

    #[tokio::test]
    async fn test_skip_when_cdar() {
        let processor = AnnuaireValidationProcessor::new(None, ValidationMode::Emission);
        let mut exchange = Exchange::new(b"<test/>".to_vec());
        exchange.set_property("document.type", "CDAR");
        exchange.invoice = Some(InvoiceData::new("F-001".to_string(), InvoiceFormat::CII));

        let result = processor.process(exchange).await.unwrap();
        assert!(!result.has_errors());
    }

    #[tokio::test]
    async fn test_skip_when_ereporting() {
        let processor = AnnuaireValidationProcessor::new(None, ValidationMode::Emission);
        let mut exchange = Exchange::new(b"<test/>".to_vec());
        exchange.set_property("document.type", "EREPORTING");

        let result = processor.process(exchange).await.unwrap();
        assert!(!result.has_errors());
    }
}
