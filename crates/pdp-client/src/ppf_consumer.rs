//! Consumer SFTP pour les flux retour du PPF (PPF → PDP).
//!
//! Le SAS PPF expose des répertoires de **retrait** dans lesquels le PPF dépose
//! les flux à destination de la PDP :
//!
//! - **F6** (CDV) — accusés de réception PPF (`200` Reçu PPF) et irrecevabilités (`501`)
//! - **F14** — exports d'annuaire
//! - **F11** — flux complémentaires éventuels
//!
//! Ce consumer surveille un ou plusieurs chemins SAS de retrait, télécharge les
//! enveloppes `tar.gz`, les décompresse et émet un `Exchange` par fichier XML
//! extrait. Chaque exchange est annoté avec :
//!
//! - `source.protocol = "ppf-sftp-return"`
//! - `ppf.envelope` — nom de l'enveloppe `tar.gz`
//! - `ppf.code_interface` — code interface déduit du nom d'enveloppe
//! - `ppf.depot_path` — chemin SAS d'origine
//!
//! Specs externes v3.1, chapitres 3.3.2.1 (protocole SFTP) et 3.4.6 (nommage des flux).

use async_trait::async_trait;
use std::collections::HashMap;

use pdp_core::endpoint::Consumer;
use pdp_core::error::PdpResult;
use pdp_core::exchange::Exchange;
use pdp_sftp::{SftpConfig, SftpConsumer};

use crate::ppf::CodeInterface;

/// Configuration du consumer PPF SFTP retour.
///
/// Le consumer parcourt tous les chemins SAS donnés (au moins un) et émet
/// un `Exchange` par fichier XML extrait des enveloppes `tar.gz` retirées.
#[derive(Debug, Clone)]
pub struct PpfReturnConsumerConfig {
    /// Configuration SFTP de base (host, port, username, clé, known_hosts…).
    /// Le champ `remote_path` est ignoré et remplacé par les `paths` ci-dessous.
    pub sftp: SftpConfig,
    /// Liste de chemins SAS de retrait à surveiller (PPF → PDP).
    /// Chaque chemin est consulté à chaque appel à `poll()`.
    pub paths: Vec<String>,
    /// Mapping optionnel chemin SAS → code interface attendu sur ce chemin.
    /// Permet de pré-renseigner `ppf.code_interface` quand le PPF utilise
    /// un répertoire dédié par type de flux. Si absent, on déduit le code
    /// interface depuis le nom de l'enveloppe.
    pub code_interface_by_path: HashMap<String, String>,
    /// Si `Some(path)`, les enveloppes traitées sont déplacées dans ce
    /// répertoire SAS plutôt que supprimées ou laissées en place.
    pub archive_path: Option<String>,
    /// Si `true` (et `archive_path` absent), supprime les enveloppes après lecture.
    pub delete_after_read: bool,
}

/// Consumer SFTP qui ingère les flux retour PPF (PPF → PDP).
///
/// Crée un `SftpConsumer` par chemin SAS retrait et combine leurs poll().
/// Chaque exchange émis est enrichi avec les propriétés `ppf.*` et
/// `source.protocol = "ppf-sftp-return"`.
pub struct PpfReturnConsumer {
    name: String,
    consumers: Vec<(String, SftpConsumer)>,
    code_interface_by_path: HashMap<String, String>,
}

impl PpfReturnConsumer {
    /// Construit un consumer PPF retour à partir d'une config SFTP de base et
    /// d'une liste de chemins SAS retrait.
    pub fn new(name: &str, config: PpfReturnConsumerConfig) -> Self {
        let mut consumers = Vec::with_capacity(config.paths.len());
        for path in &config.paths {
            let mut sftp_cfg = config.sftp.clone();
            sftp_cfg.remote_path = path.clone();
            sftp_cfg.file_pattern = "*.tar.gz".to_string();
            sftp_cfg.archive_path = config.archive_path.clone();
            // Si un archive_path global est défini il prend le pas sur delete_after_read
            sftp_cfg.delete_after_read =
                config.archive_path.is_none() && config.delete_after_read;
            let consumer = SftpConsumer::new(
                &format!("{}-{}", name, sanitize_path_name(path)),
                sftp_cfg,
            );
            consumers.push((path.clone(), consumer));
        }
        Self {
            name: name.to_string(),
            consumers,
            code_interface_by_path: config.code_interface_by_path,
        }
    }

    /// Décode le code interface depuis le nom d'une enveloppe PPF.
    ///
    /// Format attendu : `{CODE_INTERFACE}_{CODE_APP}_{ID_FLUX}.tar.gz` (8+1+6+1+25+7).
    /// Retourne `None` si le format est inattendu.
    pub fn code_interface_from_envelope(envelope: &str) -> Option<CodeInterface> {
        let prefix = envelope.split('_').next()?;
        match prefix {
            "FFE0111A" => Some(CodeInterface::F1Ubl),
            "FFE0112A" => Some(CodeInterface::F1Cii),
            "FFE0614A" => Some(CodeInterface::F6Facture),
            "FFE0604A" => Some(CodeInterface::F6DonneesReglementaires),
            "FFE0654A" => Some(CodeInterface::F6StatutsObligatoires),
            "FFE0624A" => Some(CodeInterface::F6TransactionPaiement),
            "FFE0634A" => Some(CodeInterface::F6Annuaire),
            "FFE1025A" => Some(CodeInterface::F10TransactionPaiement),
            "FFE1235A" => Some(CodeInterface::F13Annuaire),
            "FFE1435A" => Some(CodeInterface::F14ExportAnnuaire),
            _ => None,
        }
    }

    /// Annote un exchange avec les métadonnées PPF retour
    /// (`source.protocol`, `ppf.envelope`, `ppf.code_interface`, `ppf.depot_path`,
    /// `flow.syntax`).
    ///
    /// Exposé publiquement pour permettre aux tests d'intégration de simuler
    /// l'ingestion d'un flux retour PPF sans dépendre d'un serveur SFTP réel.
    pub fn annotate(
        &self,
        mut exchange: Exchange,
        depot_path: &str,
    ) -> Exchange {
        exchange.set_header("source.protocol", "ppf-sftp-return");
        exchange.set_property("ppf.depot_path", depot_path);

        // 1) Récupère l'enveloppe d'origine si disponible (set par SftpConsumer)
        let envelope = exchange.get_property("source_archive").cloned();
        if let Some(ref env) = envelope {
            exchange.set_property("ppf.envelope", env);
        }

        // 2) Détermine le code interface
        //    - priorité : envelope name → mapping path → rien
        let code_interface = envelope
            .as_deref()
            .and_then(Self::code_interface_from_envelope)
            .map(|ci| ci.as_str().to_string())
            .or_else(|| self.code_interface_by_path.get(depot_path).cloned());

        if let Some(ref ci) = code_interface {
            exchange.set_property("ppf.code_interface", ci);
            // Indique en sus la syntaxe attendue (utile au DocumentTypeRouter aval)
            match ci.as_str() {
                "FFE0614A" | "FFE0604A" | "FFE0654A" | "FFE0624A" | "FFE0634A" => {
                    exchange.set_property("flow.syntax", "CDAR");
                }
                "FFE1025A" => {
                    exchange.set_property("flow.syntax", "FRR");
                }
                "FFE0111A" => {
                    exchange.set_property("flow.syntax", "UBL");
                }
                "FFE0112A" => {
                    exchange.set_property("flow.syntax", "CII");
                }
                _ => {}
            }
        }

        exchange
    }
}

#[async_trait]
impl Consumer for PpfReturnConsumer {
    fn name(&self) -> &str {
        &self.name
    }

    async fn poll(&self) -> PdpResult<Vec<Exchange>> {
        let mut all = Vec::new();
        for (path, consumer) in &self.consumers {
            tracing::debug!(
                consumer = %self.name,
                sas_path = %path,
                "Polling SAS retrait PPF"
            );
            match consumer.poll().await {
                Ok(exchanges) => {
                    let count = exchanges.len();
                    for exchange in exchanges {
                        all.push(self.annotate(exchange, path));
                    }
                    if count > 0 {
                        tracing::info!(
                            consumer = %self.name,
                            sas_path = %path,
                            count,
                            "Flux retour PPF ingérés"
                        );
                    }
                }
                Err(e) => {
                    tracing::error!(
                        consumer = %self.name,
                        sas_path = %path,
                        error = %e,
                        "Erreur polling SAS retrait PPF"
                    );
                    // On continue sur les autres paths plutôt que de faire échouer
                    // la totalité du polling.
                }
            }
        }
        Ok(all)
    }
}

/// Construit un identifiant utilisable dans un nom de consumer à partir d'un path SAS.
fn sanitize_path_name(path: &str) -> String {
    path.trim_matches('/')
        .replace('/', "-")
        .replace(' ', "_")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code_interface_from_envelope_f6() {
        let envelope = "FFE0614A_AAA123_AAA1230614000000000000001.tar.gz";
        let ci = PpfReturnConsumer::code_interface_from_envelope(envelope);
        assert_eq!(ci, Some(CodeInterface::F6Facture));
    }

    #[test]
    fn test_code_interface_from_envelope_f1_ubl() {
        let envelope = "FFE0111A_AAA123_AAA1230111000000000000001.tar.gz";
        let ci = PpfReturnConsumer::code_interface_from_envelope(envelope);
        assert_eq!(ci, Some(CodeInterface::F1Ubl));
    }

    #[test]
    fn test_code_interface_from_envelope_f14() {
        let envelope = "FFE1435A_AAA123_AAA1231435000000000000007.tar.gz";
        let ci = PpfReturnConsumer::code_interface_from_envelope(envelope);
        assert_eq!(ci, Some(CodeInterface::F14ExportAnnuaire));
    }

    #[test]
    fn test_code_interface_from_envelope_unknown() {
        let envelope = "GARBAGE.tar.gz";
        let ci = PpfReturnConsumer::code_interface_from_envelope(envelope);
        assert_eq!(ci, None);
    }

    #[test]
    fn test_sanitize_path_name() {
        assert_eq!(sanitize_path_name("/sas/retrait/F6"), "sas-retrait-F6");
        assert_eq!(sanitize_path_name("retrait"), "retrait");
        assert_eq!(sanitize_path_name("/a b/c"), "a_b-c");
    }

    #[test]
    fn test_annotate_sets_source_protocol_and_path() {
        let consumer = PpfReturnConsumer::new(
            "ppf-return-test",
            PpfReturnConsumerConfig {
                sftp: SftpConfig::default(),
                paths: vec!["/sas/retrait/F6".to_string()],
                code_interface_by_path: HashMap::new(),
                archive_path: None,
                delete_after_read: false,
            },
        );

        let mut ex = Exchange::new(b"<CDV/>".to_vec()).with_filename("retour.xml");
        ex.set_property(
            "source_archive",
            "FFE0614A_AAA123_AAA1230614000000000000001.tar.gz",
        );

        let annotated = consumer.annotate(ex, "/sas/retrait/F6");

        assert_eq!(
            annotated.get_header("source.protocol").map(|s| s.as_str()),
            Some("ppf-sftp-return")
        );
        assert_eq!(
            annotated.get_property("ppf.depot_path").map(|s| s.as_str()),
            Some("/sas/retrait/F6")
        );
        assert_eq!(
            annotated.get_property("ppf.code_interface").map(|s| s.as_str()),
            Some("FFE0614A")
        );
        assert_eq!(
            annotated.get_property("flow.syntax").map(|s| s.as_str()),
            Some("CDAR")
        );
    }

    #[test]
    fn test_annotate_falls_back_to_path_mapping() {
        let mut by_path = HashMap::new();
        by_path.insert("/sas/retrait/F14".to_string(), "FFE1435A".to_string());

        let consumer = PpfReturnConsumer::new(
            "ppf-return-test",
            PpfReturnConsumerConfig {
                sftp: SftpConfig::default(),
                paths: vec!["/sas/retrait/F14".to_string()],
                code_interface_by_path: by_path,
                archive_path: None,
                delete_after_read: false,
            },
        );

        // Pas de source_archive : on doit retomber sur le mapping par path
        let ex = Exchange::new(b"<Annuaire/>".to_vec()).with_filename("annuaire.xml");
        let annotated = consumer.annotate(ex, "/sas/retrait/F14");

        assert_eq!(
            annotated.get_property("ppf.code_interface").map(|s| s.as_str()),
            Some("FFE1435A")
        );
    }
}
