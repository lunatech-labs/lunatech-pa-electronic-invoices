use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use async_trait::async_trait;
use tracing;

use pdp_core::endpoint::Producer;
use pdp_core::error::{PdpError, PdpResult};
use pdp_core::exchange::Exchange;
use pdp_core::model::InvoiceFormat;

use crate::afnor::AfnorFlowClient;
use crate::model::*;
use crate::ppf::{
    CodeInterface, FluxFile, PpfFluxConfig, ProfilF1,
    build_tar_gz, flux_envelope_name, f1_inner_filename, recommended_sequence,
};

// ============================================================
// PPF SFTP Producer — dépôt de flux tar.gz via SFTP
// ============================================================

/// Configuration du producer PPF SFTP.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PpfSftpProducerConfig {
    /// Configuration du nommage des flux PPF (code application)
    pub code_application: String,
    /// Profil F1 par défaut (Base ou Full)
    #[serde(default = "default_profil")]
    pub default_profil: String,
}

fn default_profil() -> String {
    "Base".to_string()
}

/// Producer qui construit des archives tar.gz conformes aux specs PPF
/// et les dépose via SFTP sur le serveur du PPF.
///
/// Specs externes v3.1, chapitres 3.3.2.1 (protocole SFTP) et 3.4.6 (nommage des flux).
pub struct PpfSftpProducer {
    name: String,
    flux_config: PpfFluxConfig,
    default_profil: ProfilF1,
    /// Compteur de séquence atomique pour générer des identifiants de flux uniques
    sequence_counter: AtomicU64,
    /// Producer SFTP sous-jacent pour le transport
    sftp_producer: pdp_sftp::SftpProducer,
}

impl PpfSftpProducer {
    pub fn new(
        name: &str,
        config: PpfSftpProducerConfig,
        sftp_config: pdp_sftp::SftpConfig,
        initial_sequence: u64,
    ) -> Result<Self, PdpError> {
        let flux_config = PpfFluxConfig::new(&config.code_application)
            .map_err(|e| PdpError::ConfigError(e.to_string()))?;

        let default_profil = match config.default_profil.as_str() {
            "Full" => ProfilF1::Full,
            _ => ProfilF1::Base,
        };

        Ok(Self {
            name: name.to_string(),
            flux_config,
            default_profil,
            sequence_counter: AtomicU64::new(initial_sequence),
            sftp_producer: pdp_sftp::SftpProducer::new(
                &format!("{}-sftp", name),
                sftp_config,
            ),
        })
    }

    /// Génère le prochain numéro de séquence et l'incrémente atomiquement.
    fn next_sequence(&self, code_interface: CodeInterface) -> String {
        let counter = self.sequence_counter.fetch_add(1, Ordering::SeqCst);
        recommended_sequence(code_interface, counter)
    }

    /// Détermine le code interface à partir de l'exchange.
    fn resolve_code_interface(exchange: &Exchange) -> CodeInterface {
        // Priorité 1 : propriété explicite
        if let Some(ci) = exchange.get_property("ppf.code_interface") {
            match ci.as_str() {
                "FFE0111A" => return CodeInterface::F1Ubl,
                "FFE0112A" => return CodeInterface::F1Cii,
                "FFE0614A" => return CodeInterface::F6Facture,
                "FFE0604A" => return CodeInterface::F6DonneesReglementaires,
                "FFE0654A" => return CodeInterface::F6StatutsObligatoires,
                "FFE0624A" => return CodeInterface::F6TransactionPaiement,
                "FFE0634A" => return CodeInterface::F6Annuaire,
                "FFE1025A" => return CodeInterface::F10TransactionPaiement,
                "FFE1235A" => return CodeInterface::F13Annuaire,
                _ => {}
            }
        }

        // Priorité 2 : flow.syntax sur l'exchange
        if let Some(syntax) = exchange.get_property("flow.syntax") {
            match syntax.as_str() {
                "CDAR" => return CodeInterface::F6Facture,
                "FRR" => return CodeInterface::F10TransactionPaiement,
                _ => {}
            }
        }

        // Priorité 3 : format de la facture parsée
        if let Some(ref inv) = exchange.invoice {
            return match inv.source_format {
                InvoiceFormat::UBL => CodeInterface::F1Ubl,
                InvoiceFormat::CII | InvoiceFormat::FacturX => CodeInterface::F1Cii,
            };
        }

        // Par défaut : F1 CII
        CodeInterface::F1Cii
    }

    /// Détermine le profil F1 à partir de l'exchange.
    fn resolve_profil(&self, exchange: &Exchange) -> ProfilF1 {
        if let Some(p) = exchange.get_property("ppf.profil") {
            match p.as_str() {
                "Full" => return ProfilF1::Full,
                "Base" => return ProfilF1::Base,
                _ => {}
            }
        }
        self.default_profil
    }

    /// Détermine le nom de base du fichier XML dans l'archive.
    fn inner_filename(exchange: &Exchange) -> String {
        exchange
            .source_filename
            .clone()
            .unwrap_or_else(|| format!("{}.xml", exchange.id))
    }
}

#[async_trait]
impl Producer for PpfSftpProducer {
    fn name(&self) -> &str {
        &self.name
    }

    async fn send(&self, exchange: Exchange) -> PdpResult<Exchange> {
        let code_interface = Self::resolve_code_interface(&exchange);
        let sequence = self.next_sequence(code_interface);

        // Construire le nom de l'enveloppe tar.gz
        let envelope_name = flux_envelope_name(
            code_interface,
            &self.flux_config.code_application,
            &sequence,
        )
        .map_err(|e| PdpError::RoutingError(format!("Nommage flux PPF: {}", e)))?;

        // Construire le nom du fichier à l'intérieur de l'archive
        let base_name = Self::inner_filename(&exchange);
        let inner_name = match code_interface {
            CodeInterface::F1Ubl | CodeInterface::F1Cii => {
                let profil = self.resolve_profil(&exchange);
                f1_inner_filename(profil, &base_name)
            }
            _ => base_name,
        };

        // Construire l'archive tar.gz
        let flux_files = vec![FluxFile {
            filename: inner_name.clone(),
            content: exchange.body.clone(),
        }];

        let tar_gz = build_tar_gz(&flux_files)
            .map_err(|e| PdpError::RoutingError(format!("Construction tar.gz: {}", e)))?;

        tracing::info!(
            exchange_id = %exchange.id,
            envelope = %envelope_name,
            inner_file = %inner_name,
            code_interface = %code_interface,
            tar_gz_size = tar_gz.len(),
            "Dépôt flux PPF via SFTP"
        );

        // Créer un exchange temporaire avec le tar.gz comme body et le bon filename
        let sftp_exchange = Exchange::new(tar_gz)
            .with_filename(&envelope_name)
            .with_flow_id(exchange.flow_id);

        // Envoyer via SFTP
        self.sftp_producer
            .send(sftp_exchange)
            .await
            .map_err(|e| PdpError::SftpError(format!("Dépôt SFTP PPF: {}", e)))?;

        tracing::info!(
            exchange_id = %exchange.id,
            envelope = %envelope_name,
            "Flux PPF déposé via SFTP"
        );

        // Enrichir l'exchange original avec les métadonnées
        let mut result = exchange;
        result.set_property("ppf.envelope", &envelope_name);
        result.set_property("ppf.code_interface", code_interface.as_str());
        result.set_property("ppf.sequence", &sequence);
        result.set_property("ppf.deposit.status", "OK");

        Ok(result)
    }
}

// ============================================================
// AFNOR Flow Service Producer — POST /v1/flows
// ============================================================

/// Producer qui envoie les factures/CDV/e-reporting vers une PDP distante
/// via le Flow Service AFNOR XP Z12-013 (POST /v1/flows multipart)
pub struct AfnorFlowProducer {
    name: String,
    client: Arc<AfnorFlowClient>,
}

impl AfnorFlowProducer {
    pub fn new(name: &str, client: Arc<AfnorFlowClient>) -> Self {
        Self {
            name: name.to_string(),
            client,
        }
    }

    /// Construit le FlowInfo à partir des propriétés de l'exchange
    fn build_flow_info(exchange: &Exchange) -> (AfnorFlowInfo, String) {
        let filename = exchange
            .source_filename
            .clone()
            .unwrap_or_else(|| format!("{}.xml", exchange.id));

        // Déterminer la syntaxe du flux
        let syntax = exchange
            .get_property("flow.syntax")
            .and_then(|s| match s.as_str() {
                "CII" => Some(FlowSyntax::CII),
                "UBL" => Some(FlowSyntax::UBL),
                "Factur-X" | "FacturX" => Some(FlowSyntax::FacturX),
                "CDAR" => Some(FlowSyntax::CDAR),
                "FRR" => Some(FlowSyntax::FRR),
                _ => None,
            })
            .or_else(|| {
                exchange.invoice.as_ref().map(|inv| match inv.source_format {
                    InvoiceFormat::CII => FlowSyntax::CII,
                    InvoiceFormat::UBL => FlowSyntax::UBL,
                    InvoiceFormat::FacturX => FlowSyntax::FacturX,
                })
            })
            .unwrap_or(FlowSyntax::CII);

        // Déterminer le profil
        let profile = exchange
            .get_property("flow.profile")
            .and_then(|s| match s.as_str() {
                "Basic" => Some(FlowProfile::Basic),
                "CIUS" => Some(FlowProfile::CIUS),
                "Extended-CTC-FR" => Some(FlowProfile::ExtendedCtcFr),
                _ => None,
            })
            .unwrap_or(FlowProfile::CIUS);

        // Déterminer la règle de traitement
        let processing_rule = exchange
            .get_property("flow.processing_rule")
            .and_then(|s| match s.as_str() {
                "B2B" => Some(ProcessingRule::B2B),
                "B2BInt" => Some(ProcessingRule::B2BInt),
                "B2C" => Some(ProcessingRule::B2C),
                _ => None,
            })
            .unwrap_or(ProcessingRule::B2B);

        // Déterminer le type de flux
        let flow_type = exchange
            .get_property("flow.type")
            .and_then(|s| match s.as_str() {
                "CustomerInvoice" => Some(FlowType::CustomerInvoice),
                "SupplierInvoice" => Some(FlowType::SupplierInvoice),
                "CustomerInvoiceLC" => Some(FlowType::CustomerInvoiceLC),
                "SupplierInvoiceLC" => Some(FlowType::SupplierInvoiceLC),
                _ => None,
            })
            .unwrap_or(FlowType::CustomerInvoice);

        let info = AfnorFlowInfo {
            tracking_id: exchange.id.to_string(),
            name: filename.clone(),
            processing_rule,
            flow_syntax: syntax,
            flow_profile: profile,
            flow_type,
            sha256: String::new(), // Computed by the client before sending
            callback_url: None,
        };

        (info, filename)
    }
}

#[async_trait]
impl Producer for AfnorFlowProducer {
    fn name(&self) -> &str {
        &self.name
    }

    async fn send(&self, exchange: Exchange) -> PdpResult<Exchange> {
        let (flow_info, filename) = Self::build_flow_info(&exchange);

        tracing::info!(
            exchange_id = %exchange.id,
            tracking_id = %flow_info.tracking_id,
            syntax = %flow_info.flow_syntax,
            flow_type = ?flow_info.flow_type,
            "Envoi du flux vers AFNOR Flow Service"
        );

        let response = self
            .client
            .envoyer_flux(&flow_info, &filename, &exchange.body)
            .await
            .map_err(|e| PdpError::RoutingError(format!("AFNOR Flow Service: {}", e)))?;

        tracing::info!(
            exchange_id = %exchange.id,
            flow_id = %response.flow_id,
            "Flux envoyé via AFNOR Flow Service"
        );

        let mut result = exchange;
        result.set_property("afnor.flow.id", &response.flow_id);
        if let Some(ref ack) = response.acknowledgement {
            result.set_property("afnor.ack.status", &format!("{:?}", ack.status));
        }

        Ok(result)
    }
}

// ============================================================
// Destination enum — pour le routage dynamique
// ============================================================

/// Destination de routage pour un exchange
#[derive(Debug, Clone)]
pub enum Destination {
    /// Envoyer vers le PPF via le Système d'Échange
    PpfSe,
    /// Envoyer vers une PDP distante via AFNOR Flow Service
    AfnorPdp {
        /// Matricule de la PDP destinataire
        matricule: String,
        /// URL du Flow Service
        flow_service_url: String,
    },
    /// Écrire sur le filesystem local
    File { path: String },
}

impl std::fmt::Display for Destination {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PpfSe => write!(f, "PPF-SE"),
            Self::AfnorPdp { matricule, .. } => write!(f, "PDP-{}", matricule),
            Self::File { path } => write!(f, "FILE:{}", path),
        }
    }
}
