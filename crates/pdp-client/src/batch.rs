//! Producer par lots pour le PPF SFTP.
//!
//! Le PPF autorise le regroupement de plusieurs fichiers de même nature/format
//! dans un seul tar.gz (taille max 1 Go, 120 Mo par fichier). Ce module fournit
//! un [`BatchProducer`] qui accumule les exchanges et les envoie en batch dans
//! une unique archive, réduisant le nombre de connexions SFTP.
//!
//! # Contraintes PPF (Specs externes v3.1, §3.4.6)
//!
//! - Un flux tar.gz ne doit contenir que des fichiers **de même nature et format**
//!   (même `CodeInterface`)
//! - Taille maximale du flux : 1 Go
//! - Taille maximale par fichier : 120 Mo
//!
//! # Exemple
//!
//! ```ignore
//! use pdp_client::batch::BatchProducer;
//!
//! let batch = BatchProducer::new(ppf_producer, 100, 500 * 1024 * 1024);
//! // Accumule les exchanges...
//! batch.send(exchange1).await?;
//! batch.send(exchange2).await?;
//! // Flush explicite (shutdown)
//! batch.flush().await?;
//! ```

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use async_trait::async_trait;
use tokio::sync::Mutex;
use tracing;

use pdp_core::endpoint::Producer;
use pdp_core::error::{PdpError, PdpResult};
use pdp_core::exchange::Exchange;

use crate::ppf::{
    CodeInterface, FluxFile, build_tar_gz, flux_envelope_name, f1_inner_filename,
};
use crate::producer::PpfSftpProducer;

// ============================================================
// BatchProducer — accumulation et envoi par lots
// ============================================================

/// Entrée bufferisée : nom de fichier dans l'archive + contenu + exchange original.
struct BufferedEntry {
    /// Nom du fichier à l'intérieur du tar.gz
    inner_filename: String,
    /// Contenu du fichier (body de l'exchange)
    content: Vec<u8>,
    /// Exchange original, retourné enrichi après le flush
    exchange: Exchange,
    /// Code interface résolu pour cet exchange
    code_interface: CodeInterface,
}

/// Producer qui accumule les exchanges et les envoie en batch dans un seul tar.gz.
///
/// # Comportement
///
/// - Accumule les exchanges dans un buffer interne
/// - Flush automatiquement quand le nombre max ou la taille max est atteint
/// - Flush manuel via [`flush()`](BatchProducer::flush) pour le shutdown
/// - Chaque batch produit un seul tar.gz avec tous les fichiers
///
/// # Regroupement par code interface
///
/// Le PPF exige que tous les fichiers d'un flux soient de même nature/format.
/// Le `BatchProducer` regroupe les fichiers par [`CodeInterface`] : si un exchange
/// a un code interface différent du batch courant, le batch existant est flushé
/// automatiquement avant d'ajouter le nouvel exchange.
///
/// # Limites
///
/// - `max_count` : nombre maximal de fichiers par batch (ex: 100)
/// - `max_bytes` : taille cumulée maximale des fichiers par batch (ex: 500 Mo)
///
/// Les deux limites respectent les contraintes PPF (1 Go max par flux, 120 Mo par fichier).
pub struct BatchProducer {
    /// Producer PPF sous-jacent pour la génération de séquences et l'envoi SFTP
    inner: Arc<PpfSftpProducer>,
    /// Buffer des exchanges en attente de flush
    buffer: Mutex<Vec<BufferedEntry>>,
    /// Nombre maximal de fichiers par batch
    max_count: usize,
    /// Taille cumulée maximale des contenus par batch (en octets)
    max_bytes: usize,
    /// Taille cumulée courante du buffer (accès atomique pour vérification rapide)
    current_bytes: AtomicUsize,
    /// Code interface du batch courant (None si le buffer est vide)
    current_code_interface: Mutex<Option<CodeInterface>>,
}

impl BatchProducer {
    /// Crée un nouveau `BatchProducer` wrappant un `PpfSftpProducer`.
    ///
    /// # Arguments
    ///
    /// - `inner` : le producer PPF SFTP sous-jacent
    /// - `max_count` : flush après N fichiers (ex: 100)
    /// - `max_bytes` : flush après N octets cumulés (ex: 500 * 1024 * 1024)
    pub fn new(inner: Arc<PpfSftpProducer>, max_count: usize, max_bytes: usize) -> Self {
        Self {
            inner,
            buffer: Mutex::new(Vec::new()),
            max_count,
            max_bytes,
            current_bytes: AtomicUsize::new(0),
            current_code_interface: Mutex::new(None),
        }
    }

    /// Flush le buffer : construit un tar.gz avec tous les fichiers accumulés
    /// et l'envoie via le producer SFTP sous-jacent.
    ///
    /// Retourne les exchanges enrichis avec les métadonnées PPF.
    /// Si le buffer est vide, retourne un vecteur vide sans erreur.
    pub async fn flush(&self) -> PdpResult<Vec<Exchange>> {
        let entries = {
            let mut buf = self.buffer.lock().await;
            let entries = std::mem::take(&mut *buf);
            self.current_bytes.store(0, Ordering::SeqCst);
            *self.current_code_interface.lock().await = None;
            entries
        };

        if entries.is_empty() {
            return Ok(Vec::new());
        }

        let code_interface = entries[0].code_interface;
        let count = entries.len();

        // Générer le numéro de séquence et le nom d'enveloppe via le producer interne
        let sequence = self.inner.next_sequence(code_interface);
        let envelope_name = flux_envelope_name(
            code_interface,
            &self.inner.flux_config.code_application,
            &sequence,
        )
        .map_err(|e| PdpError::RoutingError(format!("Nommage flux PPF batch: {}", e)))?;

        // Construire les FluxFile pour le tar.gz
        let flux_files: Vec<FluxFile> = entries
            .iter()
            .map(|e| FluxFile {
                filename: e.inner_filename.clone(),
                content: e.content.clone(),
            })
            .collect();

        let tar_gz = build_tar_gz(&flux_files)
            .map_err(|e| PdpError::RoutingError(format!("Construction tar.gz batch: {}", e)))?;

        tracing::info!(
            envelope = %envelope_name,
            code_interface = %code_interface,
            files = count,
            tar_gz_size = tar_gz.len(),
            "Flush batch PPF via SFTP"
        );

        // Créer un exchange pour le dépôt SFTP
        let sftp_exchange = Exchange::new(tar_gz)
            .with_filename(&envelope_name);

        // Envoyer via SFTP
        self.inner
            .sftp_producer
            .send(sftp_exchange)
            .await
            .map_err(|e| PdpError::SftpError(format!("Dépôt SFTP PPF batch: {}", e)))?;

        tracing::info!(
            envelope = %envelope_name,
            files = count,
            "Batch PPF déposé via SFTP"
        );

        // Enrichir les exchanges originaux avec les métadonnées
        let results: Vec<Exchange> = entries
            .into_iter()
            .map(|e| {
                let mut result = e.exchange;
                result.set_property("ppf.envelope", &envelope_name);
                result.set_property("ppf.code_interface", code_interface.as_str());
                result.set_property("ppf.sequence", &sequence);
                result.set_property("ppf.deposit.status", "OK");
                result.set_property("ppf.batch.count", &count.to_string());
                result
            })
            .collect();

        Ok(results)
    }

    /// Nombre de fichiers actuellement dans le buffer.
    pub async fn buffered_count(&self) -> usize {
        self.buffer.lock().await.len()
    }

    /// Taille cumulée des contenus dans le buffer (en octets).
    pub fn buffered_bytes(&self) -> usize {
        self.current_bytes.load(Ordering::SeqCst)
    }
}

#[async_trait]
impl Producer for BatchProducer {
    fn name(&self) -> &str {
        self.inner.name()
    }

    /// Ajoute l'exchange au buffer et flush automatiquement si les seuils sont atteints.
    ///
    /// Le flush automatique se déclenche dans deux cas :
    /// 1. Le code interface de l'exchange diffère du batch courant
    /// 2. Le nombre de fichiers ou la taille cumulée dépasse les limites
    ///
    /// L'exchange retourné est enrichi des métadonnées PPF uniquement après le flush.
    /// Si l'exchange est bufferisé (pas encore flushé), il est retourné tel quel avec
    /// la propriété `ppf.batch.buffered = true`.
    async fn send(&self, exchange: Exchange) -> PdpResult<Exchange> {
        let code_interface = PpfSftpProducer::resolve_code_interface(&exchange);
        let profil = self.inner.resolve_profil(&exchange);

        // Construire le nom du fichier interne
        let base_name = PpfSftpProducer::inner_filename(&exchange);
        let inner_name = match code_interface {
            CodeInterface::F1Ubl | CodeInterface::F1Cii => {
                f1_inner_filename(profil, &base_name)
            }
            _ => base_name,
        };

        let content_len = exchange.body.len();

        // Vérifier si le code interface change — flush du batch courant
        {
            let current_ci = self.current_code_interface.lock().await;
            if let Some(ci) = *current_ci {
                if ci != code_interface {
                    drop(current_ci); // libérer le lock avant le flush
                    tracing::info!(
                        current = %ci,
                        new = %code_interface,
                        "Code interface différent — flush du batch courant"
                    );
                    self.flush().await?;
                }
            }
        }

        // Ajouter au buffer
        let should_flush = {
            let mut buf = self.buffer.lock().await;
            buf.push(BufferedEntry {
                inner_filename: inner_name,
                content: exchange.body.clone(),
                exchange: exchange.clone(),
                code_interface,
            });
            self.current_bytes.fetch_add(content_len, Ordering::SeqCst);
            *self.current_code_interface.lock().await = Some(code_interface);

            let count = buf.len();
            let total_bytes = self.current_bytes.load(Ordering::SeqCst);
            count >= self.max_count || total_bytes >= self.max_bytes
        };

        if should_flush {
            let flush_count = self.buffer.lock().await.len();
            let flush_bytes = self.current_bytes.load(Ordering::SeqCst);
            tracing::info!(
                count = flush_count,
                bytes = flush_bytes,
                "Seuil de batch atteint — flush automatique"
            );
            self.flush().await?;
        }

        // Retourner l'exchange avec un marqueur de buffering
        let mut result = exchange;
        result.set_property("ppf.batch.buffered", "true");
        Ok(result)
    }
}

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use pdp_core::archive::read_tar_gz;

    // Comme PpfSftpProducer a besoin d'une vraie connexion SFTP, on teste
    // la logique de buffering et de construction du tar.gz séparément.

    #[test]
    fn test_batch_tar_gz_construction() {
        // Vérifier qu'on peut construire un tar.gz avec plusieurs FluxFile
        let files = vec![
            FluxFile {
                filename: "Base_facture_001.xml".to_string(),
                content: b"<Invoice><ID>FA-001</ID></Invoice>".to_vec(),
            },
            FluxFile {
                filename: "Base_facture_002.xml".to_string(),
                content: b"<Invoice><ID>FA-002</ID></Invoice>".to_vec(),
            },
            FluxFile {
                filename: "Base_facture_003.xml".to_string(),
                content: b"<Invoice><ID>FA-003</ID></Invoice>".to_vec(),
            },
        ];

        let tar_gz = build_tar_gz(&files).unwrap();
        assert!(!tar_gz.is_empty());

        // Vérifier le contenu via roundtrip
        let entries = read_tar_gz(&tar_gz).unwrap();
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].filename, "Base_facture_001.xml");
        assert_eq!(entries[1].filename, "Base_facture_002.xml");
        assert_eq!(entries[2].filename, "Base_facture_003.xml");
        assert_eq!(entries[0].content, b"<Invoice><ID>FA-001</ID></Invoice>");
    }

    #[test]
    fn test_batch_code_interface_resolution() {
        // Vérifier que le code interface est correctement résolu
        let exchange = Exchange::new(b"<Invoice/>".to_vec());
        let ci = PpfSftpProducer::resolve_code_interface(&exchange);
        // Par défaut sans facture parsée -> F1 CII
        assert_eq!(ci, CodeInterface::F1Cii);

        // Avec propriété explicite
        let mut exchange = Exchange::new(b"<Invoice/>".to_vec());
        exchange.set_property("ppf.code_interface", "FFE0111A");
        let ci = PpfSftpProducer::resolve_code_interface(&exchange);
        assert_eq!(ci, CodeInterface::F1Ubl);
    }

    #[test]
    fn test_batch_inner_filename() {
        let exchange = Exchange::new(b"<Invoice/>".to_vec())
            .with_filename("facture_001.xml");
        let name = PpfSftpProducer::inner_filename(&exchange);
        assert_eq!(name, "facture_001.xml");

        // Sans filename -> utilise l'ID
        let exchange = Exchange::new(b"<Invoice/>".to_vec());
        let name = PpfSftpProducer::inner_filename(&exchange);
        assert!(name.ends_with(".xml"));
    }

    #[test]
    fn test_batch_multiple_files_tar_gz_roundtrip() {
        // Simuler un batch de 10 factures
        let mut files = Vec::new();
        for i in 0..10 {
            files.push(FluxFile {
                filename: format!("Base_facture_{:03}.xml", i),
                content: format!("<Invoice><ID>FA-{:03}</ID></Invoice>", i).into_bytes(),
            });
        }

        let tar_gz = build_tar_gz(&files).unwrap();
        let entries = read_tar_gz(&tar_gz).unwrap();
        assert_eq!(entries.len(), 10);

        for (i, entry) in entries.iter().enumerate() {
            assert_eq!(entry.filename, format!("Base_facture_{:03}.xml", i));
            let expected = format!("<Invoice><ID>FA-{:03}</ID></Invoice>", i);
            assert_eq!(entry.content, expected.as_bytes());
        }
    }

    #[test]
    fn test_batch_respects_file_size_limit() {
        // Vérifier que build_tar_gz rejette les fichiers > 120 Mo
        let large_content = vec![0u8; 121 * 1024 * 1024]; // 121 Mo
        let files = vec![FluxFile {
            filename: "too_large.xml".to_string(),
            content: large_content,
        }];
        let result = build_tar_gz(&files);
        assert!(result.is_err());
    }

    #[test]
    fn test_batch_rejects_empty_file() {
        let files = vec![FluxFile {
            filename: "empty.xml".to_string(),
            content: Vec::new(),
        }];
        let result = build_tar_gz(&files);
        assert!(result.is_err());
    }

    #[test]
    fn test_batch_rejects_empty_batch() {
        let files: Vec<FluxFile> = Vec::new();
        let result = build_tar_gz(&files);
        assert!(result.is_err());
    }
}
