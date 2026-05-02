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

// ============================================================
// AnnuaireImportProcessor — ingestion automatique des F14 reçus du PPF
// ============================================================

/// Processor qui ingère automatiquement un flux F14 (export annuaire PPF)
/// reçu via le `PpfReturnConsumer`.
///
/// Conditions de déclenchement :
/// - L'exchange porte la propriété `ppf.code_interface = FFE1435A`
/// - Le `body` contient le XML F14 décompressé
/// - Un `AnnuaireStore` est configuré (sinon le processor logue et passe)
///
/// Produit après traitement :
/// - `annuaire.import.ok = "true"` ou `annuaire.import.error = <message>`
/// - `annuaire.import.unites_legales`, `.etablissements`, `.codes_routage`,
///   `.plateformes`, `.lignes_annuaire` (compteurs)
///
/// Code interface PPF : `FFE1435A`
/// Spécifications externes DSE AIFE V3.1, §3.4 (Flux 14 — export annuaire).
pub struct AnnuaireImportProcessor {
    store: Option<Arc<crate::AnnuaireStore>>,
    /// Si `Some`, vérifie que l'horodate du F14 correspond à la valeur attendue
    /// (utile pour les imports différentiels successifs).
    expect_horodate: Option<String>,
}

impl AnnuaireImportProcessor {
    pub fn new(store: Option<Arc<crate::AnnuaireStore>>) -> Self {
        Self {
            store,
            expect_horodate: None,
        }
    }

    pub fn with_expected_horodate(mut self, horodate: &str) -> Self {
        self.expect_horodate = Some(horodate.to_string());
        self
    }
}

#[async_trait]
impl Processor for AnnuaireImportProcessor {
    fn name(&self) -> &str {
        "AnnuaireImportProcessor"
    }

    async fn process(&self, mut exchange: Exchange) -> PdpResult<Exchange> {
        // Skip si pas un F14
        let is_f14 = exchange
            .get_property("ppf.code_interface")
            .map(|s| s.as_str() == "FFE1435A")
            .unwrap_or(false);
        if !is_f14 {
            return Ok(exchange);
        }

        // Skip si pas de store
        let store = match &self.store {
            Some(s) => s.clone(),
            None => {
                tracing::warn!(
                    exchange_id = %exchange.id,
                    "F14 reçu mais aucun AnnuaireStore configuré — ingestion ignorée"
                );
                exchange.set_property(
                    "annuaire.import.error",
                    "AnnuaireStore non configuré",
                );
                return Ok(exchange);
            }
        };

        // Skip si body vide
        if exchange.body.is_empty() {
            exchange.set_property("annuaire.import.error", "F14 body vide");
            return Ok(exchange);
        }

        let bytes = exchange.body.clone();
        let reader = std::io::Cursor::new(bytes);

        tracing::info!(
            exchange_id = %exchange.id,
            size = exchange.body.len(),
            "Ingestion F14 reçu du PPF"
        );

        match crate::ingest::ingest_f14(reader, &store, self.expect_horodate.as_deref()).await {
            Ok(stats) => {
                tracing::info!(
                    unites_legales = stats.unites_legales,
                    etablissements = stats.etablissements,
                    codes_routage = stats.codes_routage,
                    plateformes = stats.plateformes,
                    lignes_annuaire = stats.lignes_annuaire,
                    "F14 ingéré avec succès"
                );
                exchange.set_property("annuaire.import.ok", "true");
                exchange.set_property(
                    "annuaire.import.unites_legales",
                    &stats.unites_legales.to_string(),
                );
                exchange.set_property(
                    "annuaire.import.etablissements",
                    &stats.etablissements.to_string(),
                );
                exchange.set_property(
                    "annuaire.import.codes_routage",
                    &stats.codes_routage.to_string(),
                );
                exchange.set_property(
                    "annuaire.import.plateformes",
                    &stats.plateformes.to_string(),
                );
                exchange.set_property(
                    "annuaire.import.lignes_annuaire",
                    &stats.lignes_annuaire.to_string(),
                );
            }
            Err(e) => {
                tracing::error!(
                    exchange_id = %exchange.id,
                    error = %e,
                    "Échec ingestion F14"
                );
                exchange.set_property("annuaire.import.error", &e.to_string());
                exchange.add_error(
                    "annuaire-import",
                    &PdpError::ConfigError(format!("Ingestion F14 échouée: {}", e)),
                );
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

    // ============================================================
    // AnnuaireImportProcessor — tests sans Postgres
    // ============================================================

    #[tokio::test]
    async fn test_import_skips_non_f14() {
        // Pas de ppf.code_interface → skip propre
        let processor = AnnuaireImportProcessor::new(None);
        let exchange = Exchange::new(b"<not-f14/>".to_vec());

        let result = processor.process(exchange).await.unwrap();
        assert!(result.get_property("annuaire.import.ok").is_none());
        assert!(result.get_property("annuaire.import.error").is_none());
    }

    #[tokio::test]
    async fn test_import_skips_other_code_interface() {
        // Code interface différent (FFE0614A = F6 facture) → skip
        let processor = AnnuaireImportProcessor::new(None);
        let mut exchange = Exchange::new(b"<not-f14/>".to_vec());
        exchange.set_property("ppf.code_interface", "FFE0614A");

        let result = processor.process(exchange).await.unwrap();
        assert!(result.get_property("annuaire.import.ok").is_none());
        assert!(result.get_property("annuaire.import.error").is_none());
    }

    #[tokio::test]
    async fn test_import_f14_no_store_logs_warning() {
        // F14 reconnu mais pas de store → annuaire.import.error renseigné, pas d'erreur fatale
        let processor = AnnuaireImportProcessor::new(None);
        let mut exchange = Exchange::new(b"<irrelevant/>".to_vec());
        exchange.set_property("ppf.code_interface", "FFE1435A");

        let result = processor.process(exchange).await.unwrap();
        assert_eq!(
            result.get_property("annuaire.import.error").map(|s| s.as_str()),
            Some("AnnuaireStore non configuré")
        );
        // Pas d'erreur fatale — on continue le pipeline (par ex. archivage du flux)
        assert!(!result.has_errors());
    }

    #[test]
    fn test_import_processor_with_expected_horodate() {
        let processor = AnnuaireImportProcessor::new(None)
            .with_expected_horodate("20260501120000");
        assert_eq!(processor.expect_horodate.as_deref(), Some("20260501120000"));
    }
}
