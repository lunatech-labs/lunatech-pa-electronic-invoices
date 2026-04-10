//! RoutingProcessor — routage dynamique des factures vers PPF ou PDP distante.
//!
//! Ce processor consulte l'Annuaire PPF (PISTE) pour determiner la destination
//! d'une facture en fonction du SIREN/SIRET de l'acheteur :
//!
//! - Si l'acheteur est rattache au PPF (matricule "0000") ou aucune PDP trouvee
//!   -> route vers le PPF via SFTP (PpfSftpProducer)
//! - Sinon -> route vers la PDP distante via AFNOR Flow Service (AfnorFlowProducer)
//!
//! Le processor enrichit l'exchange avec les proprietes de routage :
//! - `routing.destination` : "PPF-SE" ou "PDP-{matricule}"
//! - `routing.pdp_matricule` : matricule de la PDP destinataire
//! - `routing.pdp_name` : nom de la PDP destinataire
//! - `routing.flow_service_url` : URL du Flow Service (si PDP distante)

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use tracing;

use pdp_core::error::{PdpError, PdpResult};
use pdp_core::exchange::Exchange;
use pdp_core::endpoint::Producer;
use pdp_core::processor::Processor;

use crate::annuaire::{AnnuaireClient, RoutingResolution};
use crate::producer::{Destination, PpfSftpProducer, AfnorFlowProducer};

/// Configuration des PDP partenaires connues (cache statique)
#[derive(Debug, Clone)]
pub struct PartnerDirectory {
    /// Matricule -> (nom, flow_service_url)
    partners: HashMap<String, (String, String)>,
}

impl PartnerDirectory {
    pub fn new() -> Self {
        Self {
            partners: HashMap::new(),
        }
    }

    pub fn add_partner(&mut self, matricule: &str, name: &str, flow_service_url: &str) {
        self.partners.insert(
            matricule.to_string(),
            (name.to_string(), flow_service_url.to_string()),
        );
    }

    /// Cherche l'URL du Flow Service d'une PDP par son matricule
    pub fn get_flow_service_url(&self, matricule: &str) -> Option<&str> {
        self.partners.get(matricule).map(|(_, url)| url.as_str())
    }

    /// Enrichit une RoutingResolution avec l'URL du Flow Service si connue
    pub fn enrich_resolution(&self, mut resolution: RoutingResolution) -> RoutingResolution {
        if resolution.flow_service_url.is_none() {
            if let Some((name, url)) = self.partners.get(&resolution.pdp_matricule) {
                resolution.flow_service_url = Some(url.clone());
                if resolution.pdp_name.is_empty() {
                    resolution.pdp_name = name.clone();
                }
            }
        }
        resolution
    }
}

/// Processor de routage dynamique.
///
/// Consulte l'Annuaire PPF pour determiner la destination de chaque facture,
/// puis l'envoie vers le bon producer (PPF SFTP ou AFNOR Flow Service).
pub struct RoutingProcessor {
    /// Client Annuaire PPF (PISTE) pour la resolution de routage
    annuaire: Arc<AnnuaireClient>,
    /// Producer PPF SFTP (pour les envois vers le PPF)
    ppf_producer: Arc<PpfSftpProducer>,
    /// Producers AFNOR Flow Service par matricule PDP
    /// Lazy-initialized a partir du partner_directory
    afnor_producers: HashMap<String, Arc<AfnorFlowProducer>>,
    /// Repertoire des PDP partenaires connues
    partner_directory: PartnerDirectory,
}

impl RoutingProcessor {
    pub fn new(
        annuaire: Arc<AnnuaireClient>,
        ppf_producer: Arc<PpfSftpProducer>,
        partner_directory: PartnerDirectory,
    ) -> Self {
        Self {
            annuaire,
            ppf_producer,
            afnor_producers: HashMap::new(),
            partner_directory,
        }
    }

    /// Ajoute un producer AFNOR Flow Service pour une PDP partenaire
    pub fn add_afnor_producer(&mut self, matricule: &str, producer: Arc<AfnorFlowProducer>) {
        self.afnor_producers.insert(matricule.to_string(), producer);
    }

    /// Extrait le SIREN de l'acheteur depuis l'exchange
    pub fn extract_buyer_siren(exchange: &Exchange) -> Option<String> {
        // Priorite 1 : propriete explicite
        if let Some(siren) = exchange.get_property("buyer.siren") {
            return Some(siren.clone());
        }

        // Priorite 2 : depuis la facture parsee (methode buyer_siren() = 9 premiers chars du SIRET)
        if let Some(ref invoice) = exchange.invoice {
            if let Some(siren) = invoice.buyer_siren() {
                return Some(siren);
            }
        }

        None
    }

    /// Extrait le SIRET de l'acheteur depuis l'exchange
    pub fn extract_buyer_siret(exchange: &Exchange) -> Option<String> {
        if let Some(siret) = exchange.get_property("buyer.siret") {
            return Some(siret.clone());
        }
        if let Some(ref invoice) = exchange.invoice {
            if let Some(ref siret) = invoice.buyer_siret {
                return Some(siret.clone());
            }
        }
        None
    }

    /// Resout la destination pour un exchange donne
    async fn resolve_destination(&self, exchange: &Exchange) -> PdpResult<Destination> {
        let buyer_siren = Self::extract_buyer_siren(exchange);
        let buyer_siret = Self::extract_buyer_siret(exchange);

        if buyer_siren.is_none() && buyer_siret.is_none() {
            tracing::warn!(
                exchange_id = %exchange.id,
                "Impossible de determiner le SIREN/SIRET de l'acheteur, envoi au PPF par defaut"
            );
            return Ok(Destination::PpfSe);
        }

        // Consulter l'Annuaire PPF
        let resolution = self
            .annuaire
            .resoudre_routage(
                buyer_siren.as_deref(),
                buyer_siret.as_deref(),
            )
            .await
            .map_err(|e| {
                PdpError::RoutingError(format!(
                    "Erreur consultation Annuaire PPF pour SIREN={}, SIRET={}: {}",
                    buyer_siren.as_deref().unwrap_or("N/A"),
                    buyer_siret.as_deref().unwrap_or("N/A"),
                    e
                ))
            })?;

        // Enrichir avec le repertoire des partenaires
        let resolution = self.partner_directory.enrich_resolution(resolution);

        if resolution.is_ppf {
            tracing::info!(
                exchange_id = %exchange.id,
                buyer_siren = buyer_siren.as_deref().unwrap_or("N/A"),
                "Routage vers le PPF (matricule 0000)"
            );
            Ok(Destination::PpfSe)
        } else {
            let flow_service_url = resolution.flow_service_url.clone().ok_or_else(|| {
                PdpError::RoutingError(format!(
                    "PDP {} ({}) trouvee mais aucune URL de Flow Service connue. \
                     Ajoutez cette PDP dans la configuration afnor.partners.",
                    resolution.pdp_matricule, resolution.pdp_name
                ))
            })?;

            tracing::info!(
                exchange_id = %exchange.id,
                buyer_siren = buyer_siren.as_deref().unwrap_or("N/A"),
                pdp_matricule = %resolution.pdp_matricule,
                pdp_name = %resolution.pdp_name,
                flow_service_url = %flow_service_url,
                "Routage vers PDP distante via AFNOR Flow Service"
            );

            Ok(Destination::AfnorPdp {
                matricule: resolution.pdp_matricule,
                flow_service_url,
            })
        }
    }

    /// Envoie l'exchange vers la destination resolue
    async fn send_to_destination(
        &self,
        exchange: Exchange,
        destination: &Destination,
    ) -> PdpResult<Exchange> {
        match destination {
            Destination::PpfSe => {
                self.ppf_producer.send(exchange).await
            }
            Destination::AfnorPdp { matricule, .. } => {
                if let Some(producer) = self.afnor_producers.get(matricule) {
                    producer.send(exchange).await
                } else {
                    Err(PdpError::RoutingError(format!(
                        "Aucun producer AFNOR configure pour la PDP matricule {}. \
                         Ajoutez cette PDP dans la configuration afnor.partners.",
                        matricule
                    )))
                }
            }
            Destination::File { path } => {
                // Fallback : ecrire dans un repertoire local
                let endpoint = pdp_core::endpoint::FileEndpoint::output("routing-file", path);
                endpoint.send(exchange).await
            }
        }
    }
}

#[async_trait]
impl Processor for RoutingProcessor {
    fn name(&self) -> &str {
        "routing"
    }

    async fn process(&self, exchange: Exchange) -> PdpResult<Exchange> {
        // Skip si le document n'est pas une facture (CDAR, e-reporting...)
        if let Some(doc_type) = exchange.get_property("document.type") {
            match doc_type.as_str() {
                "CDAR" | "EREPORTING" => {
                    tracing::debug!(
                        exchange_id = %exchange.id,
                        document_type = %doc_type,
                        "RoutingProcessor: skip (document non-facture)"
                    );
                    return Ok(exchange);
                }
                _ => {}
            }
        }

        // Skip si aucune facture parsee
        if exchange.invoice.is_none() {
            tracing::debug!(
                exchange_id = %exchange.id,
                "RoutingProcessor: skip (pas de facture parsee)"
            );
            return Ok(exchange);
        }

        // Resoudre la destination
        let destination = self.resolve_destination(&exchange).await?;

        // Enrichir l'exchange avec les proprietes de routage
        let mut exchange = exchange;
        exchange.set_property("routing.destination", &destination.to_string());

        match &destination {
            Destination::PpfSe => {
                exchange.set_property("routing.pdp_matricule", "0000");
                exchange.set_property("routing.pdp_name", "PPF");
            }
            Destination::AfnorPdp { matricule, flow_service_url } => {
                exchange.set_property("routing.pdp_matricule", matricule);
                exchange.set_property("routing.flow_service_url", flow_service_url);
            }
            Destination::File { path } => {
                exchange.set_property("routing.file_path", path);
            }
        }

        // Envoyer vers la destination
        let exchange = self.send_to_destination(exchange, &destination).await?;

        tracing::info!(
            exchange_id = %exchange.id,
            destination = %destination,
            invoice_number = exchange.invoice.as_ref().map(|i| i.invoice_number.as_str()).unwrap_or("N/A"),
            "Facture routee avec succes"
        );

        Ok(exchange)
    }
}

/// Processor de routage simplifie qui ne fait que resoudre la destination
/// sans envoyer (pour les cas ou le producer est gere separement par la route).
///
/// Enrichit l'exchange avec les proprietes `routing.*` pour que le producer
/// en aval puisse determiner la destination.
pub struct RoutingResolverProcessor {
    annuaire: Arc<AnnuaireClient>,
    partner_directory: PartnerDirectory,
}

impl RoutingResolverProcessor {
    pub fn new(
        annuaire: Arc<AnnuaireClient>,
        partner_directory: PartnerDirectory,
    ) -> Self {
        Self {
            annuaire,
            partner_directory,
        }
    }
}

#[async_trait]
impl Processor for RoutingResolverProcessor {
    fn name(&self) -> &str {
        "routing-resolver"
    }

    async fn process(&self, exchange: Exchange) -> PdpResult<Exchange> {
        // Skip si pas une facture
        if exchange.invoice.is_none() {
            return Ok(exchange);
        }

        let buyer_siren = RoutingProcessor::extract_buyer_siren(&exchange);
        let buyer_siret = RoutingProcessor::extract_buyer_siret(&exchange);

        if buyer_siren.is_none() && buyer_siret.is_none() {
            let mut exchange = exchange;
            exchange.set_property("routing.destination", "PPF-SE");
            exchange.set_property("routing.pdp_matricule", "0000");
            exchange.set_property("routing.pdp_name", "PPF");
            return Ok(exchange);
        }

        let resolution = self
            .annuaire
            .resoudre_routage(
                buyer_siren.as_deref(),
                buyer_siret.as_deref(),
            )
            .await
            .map_err(|e| {
                PdpError::RoutingError(format!("Erreur Annuaire: {}", e))
            })?;

        let resolution = self.partner_directory.enrich_resolution(resolution);

        let mut exchange = exchange;
        if resolution.is_ppf {
            exchange.set_property("routing.destination", "PPF-SE");
            exchange.set_property("routing.pdp_matricule", "0000");
            exchange.set_property("routing.pdp_name", "PPF");
        } else {
            exchange.set_property(
                "routing.destination",
                &format!("PDP-{}", resolution.pdp_matricule),
            );
            exchange.set_property("routing.pdp_matricule", &resolution.pdp_matricule);
            exchange.set_property("routing.pdp_name", &resolution.pdp_name);
            if let Some(ref url) = resolution.flow_service_url {
                exchange.set_property("routing.flow_service_url", url);
            }
        }

        tracing::info!(
            exchange_id = %exchange.id,
            destination = exchange.get_property("routing.destination").map(|s| s.as_str()).unwrap_or("N/A"),
            buyer_siren = buyer_siren.as_deref().unwrap_or("N/A"),
            "Destination resolue"
        );

        Ok(exchange)
    }
}

/// Producer dynamique qui route l'exchange vers PPF ou PDP
/// en fonction de la propriete `routing.destination` definie par le RoutingResolverProcessor.
pub struct DynamicRoutingProducer {
    name: String,
    ppf_producer: Arc<PpfSftpProducer>,
    afnor_producers: HashMap<String, Arc<AfnorFlowProducer>>,
    /// Fallback : ecrire sur le filesystem si aucun producer n'est configure
    fallback_path: Option<String>,
}

impl DynamicRoutingProducer {
    pub fn new(
        name: &str,
        ppf_producer: Arc<PpfSftpProducer>,
    ) -> Self {
        Self {
            name: name.to_string(),
            ppf_producer,
            afnor_producers: HashMap::new(),
            fallback_path: None,
        }
    }

    pub fn add_afnor_producer(&mut self, matricule: &str, producer: Arc<AfnorFlowProducer>) {
        self.afnor_producers.insert(matricule.to_string(), producer);
    }

    pub fn with_fallback_path(mut self, path: &str) -> Self {
        self.fallback_path = Some(path.to_string());
        self
    }
}

#[async_trait]
impl Producer for DynamicRoutingProducer {
    fn name(&self) -> &str {
        &self.name
    }

    async fn send(&self, exchange: Exchange) -> PdpResult<Exchange> {
        let destination = exchange
            .get_property("routing.destination")
            .cloned()
            .unwrap_or_else(|| "PPF-SE".to_string());

        if destination == "PPF-SE" {
            tracing::info!(
                exchange_id = %exchange.id,
                "DynamicRoutingProducer: envoi vers PPF via SFTP"
            );
            return self.ppf_producer.send(exchange).await;
        }

        // PDP-{matricule}
        if destination.starts_with("PDP-") {
            let matricule = &destination[4..];
            if let Some(producer) = self.afnor_producers.get(matricule) {
                tracing::info!(
                    exchange_id = %exchange.id,
                    pdp_matricule = %matricule,
                    "DynamicRoutingProducer: envoi vers PDP via AFNOR Flow Service"
                );
                return producer.send(exchange).await;
            } else {
                tracing::warn!(
                    exchange_id = %exchange.id,
                    pdp_matricule = %matricule,
                    "Aucun producer AFNOR pour cette PDP, fallback"
                );
            }
        }

        // Fallback : ecriture locale
        if let Some(ref path) = self.fallback_path {
            tracing::warn!(
                exchange_id = %exchange.id,
                destination = %destination,
                fallback_path = %path,
                "DynamicRoutingProducer: fallback vers filesystem"
            );
            let file_producer = pdp_core::endpoint::FileEndpoint::output("fallback", path);
            return file_producer.send(exchange).await;
        }

        Err(PdpError::RoutingError(format!(
            "Aucun producer disponible pour la destination '{}' et aucun fallback configure",
            destination
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_partner_directory() {
        let mut dir = PartnerDirectory::new();
        dir.add_partner("1111", "PDP UNO", "https://api.flow.uno.fr/flow-service");
        dir.add_partner("2222", "PDP DUO", "https://api.flow.duo.fr/flow-service");

        assert_eq!(
            dir.get_flow_service_url("1111"),
            Some("https://api.flow.uno.fr/flow-service")
        );
        assert_eq!(
            dir.get_flow_service_url("2222"),
            Some("https://api.flow.duo.fr/flow-service")
        );
        assert_eq!(dir.get_flow_service_url("9999"), None);
    }

    #[test]
    fn test_enrich_resolution_with_known_partner() {
        let mut dir = PartnerDirectory::new();
        dir.add_partner("1111", "PDP UNO", "https://api.flow.uno.fr/flow-service");

        let resolution = RoutingResolution {
            pdp_matricule: "1111".to_string(),
            pdp_name: String::new(),
            flow_service_url: None,
            is_ppf: false,
        };

        let enriched = dir.enrich_resolution(resolution);
        assert_eq!(
            enriched.flow_service_url.as_deref(),
            Some("https://api.flow.uno.fr/flow-service")
        );
        assert_eq!(enriched.pdp_name, "PDP UNO");
    }

    #[test]
    fn test_enrich_resolution_ppf() {
        let dir = PartnerDirectory::new();

        let resolution = RoutingResolution {
            pdp_matricule: "0000".to_string(),
            pdp_name: "PPF".to_string(),
            flow_service_url: None,
            is_ppf: true,
        };

        let enriched = dir.enrich_resolution(resolution);
        assert!(enriched.is_ppf);
        assert!(enriched.flow_service_url.is_none());
    }

    #[test]
    fn test_extract_buyer_siren_from_invoice() {
        let mut exchange = Exchange::new(b"<test/>".to_vec());
        let mut invoice = pdp_core::model::InvoiceData::new(
            "TEST-001".to_string(),
            pdp_core::model::InvoiceFormat::UBL,
        );
        invoice.buyer_siret = Some("12345678901234".to_string());
        exchange.invoice = Some(invoice);

        let siren = RoutingProcessor::extract_buyer_siren(&exchange);
        assert_eq!(siren.as_deref(), Some("123456789"));
    }

    #[test]
    fn test_extract_buyer_siren_from_property() {
        let mut exchange = Exchange::new(b"<test/>".to_vec());
        exchange.set_property("buyer.siren", "999888777");

        let siren = RoutingProcessor::extract_buyer_siren(&exchange);
        assert_eq!(siren.as_deref(), Some("999888777"));
    }

    #[test]
    fn test_extract_buyer_siren_none() {
        let exchange = Exchange::new(b"<test/>".to_vec());
        let siren = RoutingProcessor::extract_buyer_siren(&exchange);
        assert!(siren.is_none());
    }

    #[test]
    fn test_destination_display() {
        assert_eq!(Destination::PpfSe.to_string(), "PPF-SE");
        assert_eq!(
            Destination::AfnorPdp {
                matricule: "1111".to_string(),
                flow_service_url: "https://example.com".to_string(),
            }
            .to_string(),
            "PDP-1111"
        );
    }
}
