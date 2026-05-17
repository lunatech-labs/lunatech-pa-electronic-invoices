use async_trait::async_trait;
use crate::exchange::Exchange;
use crate::error::{PdpError, PdpResult};
use std::io::Read;

/// Type d'endpoint
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EndpointType {
    /// Source SFTP (lecture de fichiers)
    SftpIn,
    /// Destination SFTP (écriture de fichiers)
    SftpOut,
    /// Répertoire local (lecture)
    FileIn,
    /// Répertoire local (écriture)
    FileOut,
    /// API HTTP entrante
    HttpIn,
    /// API HTTP sortante (vers PPF par ex.)
    HttpOut,
    /// Endpoint interne (channel tokio)
    Direct,
    /// Timer/Cron pour polling
    Timer,
}

/// Un Endpoint est une source ou destination de messages.
/// Équivalent du Component/Endpoint dans Apache Camel.
#[async_trait]
pub trait Endpoint: Send + Sync {
    /// Nom de l'endpoint
    fn name(&self) -> &str;

    /// Type de l'endpoint
    fn endpoint_type(&self) -> EndpointType;

    /// URI de l'endpoint (ex: "sftp://host:22/path")
    fn uri(&self) -> &str;
}

/// Consumer : lit des messages depuis un endpoint source
#[async_trait]
pub trait Consumer: Send + Sync {
    /// Nom du consumer
    fn name(&self) -> &str;

    /// Poll : récupère les exchanges disponibles
    async fn poll(&self) -> PdpResult<Vec<Exchange>>;

    /// Démarre le consumer (pour les modes push)
    async fn start(&self) -> PdpResult<()> {
        Ok(())
    }

    /// Arrête le consumer
    async fn stop(&self) -> PdpResult<()> {
        Ok(())
    }
}

/// Producer : envoie des messages vers un endpoint destination
#[async_trait]
pub trait Producer: Send + Sync {
    /// Nom du producer
    fn name(&self) -> &str;

    /// Envoie un exchange vers la destination
    async fn send(&self, exchange: Exchange) -> PdpResult<Exchange>;
}

/// Endpoint basé sur le filesystem local (pour les tests et le dev)
pub struct FileEndpoint {
    name: String,
    path: String,
    endpoint_type: EndpointType,
    /// Délai de stabilité en millisecondes : on attend ce délai puis on revérifie
    /// la taille du fichier. Si elle n'a pas changé, le fichier est considéré
    /// comme entièrement écrit et peut être consommé. 0 = pas de vérification.
    stable_delay_ms: u64,
    /// Nom du sous-répertoire (relatif à `path`) où déplacer les fichiers après
    /// consommation. `None` = pas d'archivage (les fichiers restent dans `path`
    /// et seront re-lus au prochain poll — comportement legacy, utile pour
    /// quelques tests). Sans archivage, la pipeline génère un CDV 213
    /// (BR-FR-12/13) à chaque poll suivant car la facture est déjà indexée.
    archive_dir: Option<String>,
}

/// Délai de stabilité par défaut (1 seconde)
const DEFAULT_STABLE_DELAY_MS: u64 = 1000;

/// Sous-répertoire d'archive par défaut pour les inputs (préfixé `.` pour ne
/// pas polluer un listing `ls` du répertoire d'entrée et pour être ignoré par
/// la plupart des outils de scan).
const DEFAULT_ARCHIVE_DIR: &str = ".processed";

impl FileEndpoint {
    pub fn input(name: &str, path: &str) -> Self {
        Self {
            name: name.to_string(),
            path: path.to_string(),
            endpoint_type: EndpointType::FileIn,
            stable_delay_ms: DEFAULT_STABLE_DELAY_MS,
            archive_dir: Some(DEFAULT_ARCHIVE_DIR.to_string()),
        }
    }

    pub fn output(name: &str, path: &str) -> Self {
        Self {
            name: name.to_string(),
            path: path.to_string(),
            endpoint_type: EndpointType::FileOut,
            stable_delay_ms: 0, // pas de vérification en écriture
            archive_dir: None,  // pas d'archivage en sortie
        }
    }

    /// Configure le délai de stabilité (en ms). 0 = pas de vérification.
    pub fn with_stable_delay(mut self, delay_ms: u64) -> Self {
        self.stable_delay_ms = delay_ms;
        self
    }

    /// Configure le sous-répertoire (relatif à `path`) où déplacer les
    /// fichiers après consommation. Passer `None` désactive l'archivage
    /// (les fichiers seront re-lus au prochain poll).
    pub fn with_archive_dir(mut self, dir: Option<&str>) -> Self {
        self.archive_dir = dir.map(String::from);
        self
    }

    pub fn path(&self) -> &str {
        &self.path
    }
}

#[async_trait]
impl Endpoint for FileEndpoint {
    fn name(&self) -> &str {
        &self.name
    }

    fn endpoint_type(&self) -> EndpointType {
        self.endpoint_type.clone()
    }

    fn uri(&self) -> &str {
        &self.path
    }
}

#[async_trait]
impl Consumer for FileEndpoint {
    fn name(&self) -> &str {
        &self.name
    }

    async fn poll(&self) -> PdpResult<Vec<Exchange>> {
        let path = std::path::Path::new(&self.path);

        if !path.exists() {
            tracing::warn!(path = %self.path, "Répertoire source inexistant");
            return Ok(Vec::new());
        }

        // Collecter les fichiers candidats avec leur taille
        let mut candidates: Vec<(std::path::PathBuf, String, u64)> = Vec::new();
        let entries = std::fs::read_dir(path)
            .map_err(crate::error::PdpError::IoError)?;

        for entry in entries {
            let entry = entry.map_err(crate::error::PdpError::IoError)?;
            let file_path = entry.path();

            // Le sous-répertoire d'archive est un répertoire (donc déjà filtré
            // par `is_file()`), mais on l'ignore explicitement pour ne jamais
            // descendre dedans même si l'utilisateur en faisait un fichier par
            // erreur, et pour documenter l'intention.
            if let Some(archive) = self.archive_dir.as_deref() {
                if entry.file_name().to_str() == Some(archive) {
                    continue;
                }
            }

            if file_path.is_file() {
                let filename = file_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                let meta = std::fs::metadata(&file_path)
                    .map_err(crate::error::PdpError::IoError)?;

                candidates.push((file_path, filename, meta.len()));
            }
        }

        // Vérification de stabilité : attendre puis revérifier la taille
        if self.stable_delay_ms > 0 && !candidates.is_empty() {
            tokio::time::sleep(std::time::Duration::from_millis(self.stable_delay_ms)).await;

            candidates.retain(|(file_path, filename, initial_size)| {
                match std::fs::metadata(file_path) {
                    Ok(meta) => {
                        let current_size = meta.len();
                        if current_size == *initial_size {
                            true
                        } else {
                            tracing::debug!(
                                filename = %filename,
                                initial_size,
                                current_size,
                                "Fichier ignoré (encore en cours d'écriture)"
                            );
                            false
                        }
                    }
                    Err(_) => {
                        tracing::debug!(
                            filename = %filename,
                            "Fichier disparu entre les deux vérifications"
                        );
                        false
                    }
                }
            });
        }

        // Lire les fichiers stables, décompresser les archives
        let mut exchanges = Vec::new();
        for (file_path, filename, _) in &candidates {
            let lower = filename.to_lowercase();

            // `consumed` = au moins une donnée a été extraite du fichier (qu'il
            // soit normal ou archive). On n'archive que dans ce cas, pour que
            // les fichiers illisibles ou archives corrompues ne soient PAS
            // déplacés silencieusement — ils restent visibles dans `path` pour
            // diagnostic. Ils causent un avertissement à chaque poll suivant
            // mais pas de doublons puisqu'ils ne sont jamais consommés.
            let consumed;

            if lower.ends_with(".tar.gz") || lower.ends_with(".tgz") {
                // Décompresser tar.gz → un exchange par fichier extrait
                match Self::extract_tar_gz(file_path) {
                    Ok(extracted) => {
                        tracing::info!(
                            archive = %filename,
                            files = extracted.len(),
                            "Archive tar.gz décompressée"
                        );
                        for (name, body) in extracted {
                            let mut exchange = Exchange::new(body).with_filename(&name);
                            exchange.set_property("source_archive", filename);
                            tracing::info!(
                                filename = %name,
                                archive = %filename,
                                exchange_id = %exchange.id,
                                "Fichier extrait de l'archive tar.gz"
                            );
                            exchanges.push(exchange);
                        }
                        consumed = true;
                    }
                    Err(e) => {
                        tracing::error!(archive = %filename, error = %e, "Erreur décompression tar.gz");
                        consumed = false;
                    }
                }
            } else if lower.ends_with(".zip") {
                // Décompresser zip → un exchange par fichier extrait
                match Self::extract_zip(file_path) {
                    Ok(extracted) => {
                        tracing::info!(
                            archive = %filename,
                            files = extracted.len(),
                            "Archive ZIP décompressée"
                        );
                        for (name, body) in extracted {
                            let mut exchange = Exchange::new(body).with_filename(&name);
                            exchange.set_property("source_archive", filename);
                            tracing::info!(
                                filename = %name,
                                archive = %filename,
                                exchange_id = %exchange.id,
                                "Fichier extrait de l'archive ZIP"
                            );
                            exchanges.push(exchange);
                        }
                        consumed = true;
                    }
                    Err(e) => {
                        tracing::error!(archive = %filename, error = %e, "Erreur décompression ZIP");
                        consumed = false;
                    }
                }
            } else {
                // Fichier normal
                let body = std::fs::read(file_path)
                    .map_err(PdpError::IoError)?;

                let exchange = Exchange::new(body).with_filename(filename);

                tracing::info!(
                    filename = %filename,
                    exchange_id = %exchange.id,
                    "Fichier lu depuis le filesystem"
                );

                exchanges.push(exchange);
                consumed = true;
            }

            if consumed {
                self.archive_consumed_file(file_path, filename);
            }
        }

        Ok(exchanges)
    }
}

impl FileEndpoint {
    /// Déplace le fichier consommé vers le sous-répertoire d'archive,
    /// classé par date (`{archive_dir}/{YYYYMMDD}/{filename}`).
    ///
    /// Si `archive_dir` est `None`, la fonction est un no-op (le fichier reste
    /// dans `path` et sera re-lu au prochain poll — l'appelant a explicitement
    /// désactivé l'archivage).
    ///
    /// En cas d'échec (permissions, FS plein, etc.), on log un warning mais on
    /// laisse le fichier en place. Le prochain poll le re-traitera, ce qui
    /// peut générer des CDV 213 (doublon BR-FR-12/13) — moins grave qu'une
    /// suppression silencieuse de données entrantes.
    fn archive_consumed_file(&self, src: &std::path::Path, filename: &str) {
        let Some(archive_dir) = self.archive_dir.as_deref() else {
            return;
        };

        let date = chrono::Utc::now().format("%Y%m%d").to_string();
        let target_dir = std::path::Path::new(&self.path)
            .join(archive_dir)
            .join(&date);

        if let Err(e) = std::fs::create_dir_all(&target_dir) {
            tracing::warn!(
                archive_dir = %target_dir.display(),
                error = %e,
                filename = %filename,
                "Impossible de créer le répertoire d'archive — fichier laissé en place (sera re-traité au prochain poll)"
            );
            return;
        }

        // Choix d'un nom unique : si une archive précédente du même fichier
        // existe déjà ce jour (re-soumission), on suffixe avec un timestamp
        // intra-jour pour éviter l'écrasement.
        let mut dst = target_dir.join(filename);
        if dst.exists() {
            let ts = chrono::Utc::now().format("%H%M%S%3f").to_string();
            let (stem, ext) = match filename.rsplit_once('.') {
                Some((s, e)) => (s.to_string(), format!(".{}", e)),
                None => (filename.to_string(), String::new()),
            };
            dst = target_dir.join(format!("{}-{}{}", stem, ts, ext));
        }

        match std::fs::rename(src, &dst) {
            Ok(_) => {
                tracing::debug!(
                    src = %src.display(),
                    dst = %dst.display(),
                    "Fichier archivé après consommation"
                );
            }
            Err(e) => {
                // Fallback copy+remove : `rename` échoue cross-device (FS
                // montés différemment) ; on tente copy puis remove.
                match std::fs::copy(src, &dst).and_then(|_| std::fs::remove_file(src)) {
                    Ok(_) => tracing::debug!(
                        src = %src.display(),
                        dst = %dst.display(),
                        "Fichier archivé (copy+remove)"
                    ),
                    Err(e2) => tracing::warn!(
                        src = %src.display(),
                        dst = %dst.display(),
                        rename_error = %e,
                        copy_error = %e2,
                        "Impossible d'archiver le fichier — il sera re-traité au prochain poll (risque de boucle CDV 213)"
                    ),
                }
            }
        }
    }

    /// Extrait les fichiers d'une archive tar.gz en mémoire.
    /// Retourne un Vec de (nom_fichier, contenu).
    fn extract_tar_gz(path: &std::path::Path) -> PdpResult<Vec<(String, Vec<u8>)>> {
        let file = std::fs::File::open(path).map_err(PdpError::IoError)?;
        let gz = flate2::read::GzDecoder::new(file);
        let mut archive = tar::Archive::new(gz);

        let mut files = Vec::new();
        for entry in archive.entries().map_err(PdpError::IoError)? {
            let mut entry = entry.map_err(PdpError::IoError)?;
            // Ignorer les répertoires
            if entry.header().entry_type().is_dir() {
                continue;
            }
            let name = entry.path()
                .map_err(PdpError::IoError)?
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            if name.is_empty() {
                continue;
            }
            let mut body = Vec::new();
            entry.read_to_end(&mut body).map_err(PdpError::IoError)?;
            if !body.is_empty() {
                files.push((name, body));
            }
        }
        Ok(files)
    }

    /// Extrait les fichiers d'une archive ZIP en mémoire.
    /// Retourne un Vec de (nom_fichier, contenu).
    fn extract_zip(path: &std::path::Path) -> PdpResult<Vec<(String, Vec<u8>)>> {
        let file = std::fs::File::open(path).map_err(PdpError::IoError)?;
        let mut archive = zip::ZipArchive::new(file)
            .map_err(|e| PdpError::TraceError(format!("Erreur ouverture ZIP: {}", e)))?;

        let mut files = Vec::new();
        for i in 0..archive.len() {
            let mut entry = archive.by_index(i)
                .map_err(|e| PdpError::TraceError(format!("Erreur lecture ZIP entry: {}", e)))?;
            // Ignorer les répertoires
            if entry.is_dir() {
                continue;
            }
            let name = std::path::Path::new(entry.name())
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            if name.is_empty() {
                continue;
            }
            let mut body = Vec::new();
            entry.read_to_end(&mut body).map_err(PdpError::IoError)?;
            if !body.is_empty() {
                files.push((name, body));
            }
        }
        Ok(files)
    }
}

#[async_trait]
impl Producer for FileEndpoint {
    fn name(&self) -> &str {
        &self.name
    }

    async fn send(&self, exchange: Exchange) -> PdpResult<Exchange> {
        let dir = std::path::Path::new(&self.path);
        if !dir.exists() {
            std::fs::create_dir_all(dir)
                .map_err(|e| crate::error::PdpError::IoError(e))?;
        }

        let id_string = exchange.id.to_string();
        let filename = exchange
            .source_filename
            .as_deref()
            .unwrap_or(&id_string);

        let file_path = dir.join(filename);
        std::fs::write(&file_path, &exchange.body)
            .map_err(|e| crate::error::PdpError::IoError(e))?;

        tracing::info!(
            filename = %filename,
            path = %file_path.display(),
            "Fichier écrit sur le filesystem"
        );

        Ok(exchange)
    }
}

/// Stratégie de résolution du tenant cible pour [`TenantOutputProducer`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TenantRole {
    /// Écrit dans le `out/` du **vendeur** (utile pour les CDV retour
    /// 202/203 émis par la pipeline intra-PDP réception : le vendeur veut
    /// voir les accusés de l'acheteur dans son propre répertoire).
    Seller,
    /// Écrit dans le `out/` de l'**acheteur** (utile pour la facture
    /// elle-même quand elle est routée en intra-PDP vers le destinataire
    /// local).
    Buyer,
}

/// Producer qui écrit le `body` de l'exchange dans
/// `{tenants_dir}/{siren}/out/facture/{filename}`, où `siren` est résolu
/// depuis l'invoice de l'exchange (vendeur ou acheteur selon [`TenantRole`]).
/// Le sous-répertoire `facture/` sépare les factures (Flux 2) des CDV
/// (qui vont dans `out/cdv/`) pour que l'utilisateur s'y retrouve.
///
/// Si le tenant cible n'existe pas (pas de sous-répertoire), ou si la facture
/// n'est pas parsée, on retombe sur `fallback_dir` — comportement non bloquant
/// pour rester compatible avec les flux qui ne correspondent à aucun tenant
/// local.
pub struct TenantOutputProducer {
    name: String,
    tenants_dir: std::path::PathBuf,
    role: TenantRole,
    fallback_dir: std::path::PathBuf,
}

impl TenantOutputProducer {
    pub fn new(
        name: &str,
        tenants_dir: impl Into<std::path::PathBuf>,
        role: TenantRole,
        fallback_dir: impl Into<std::path::PathBuf>,
    ) -> Self {
        Self {
            name: name.to_string(),
            tenants_dir: tenants_dir.into(),
            role,
            fallback_dir: fallback_dir.into(),
        }
    }

    fn resolve_siren(&self, exchange: &Exchange) -> Option<String> {
        let invoice = exchange.invoice.as_ref()?;
        let siret = match self.role {
            TenantRole::Seller => invoice.seller_siret.as_deref(),
            TenantRole::Buyer => invoice.buyer_siret.as_deref(),
        }?;
        // Le SIREN est le préfixe 9 chiffres du SIRET.
        siret
            .chars()
            .filter(|c| c.is_ascii_digit())
            .take(9)
            .collect::<String>()
            .into()
    }
}

#[async_trait]
impl Producer for TenantOutputProducer {
    fn name(&self) -> &str {
        &self.name
    }

    async fn send(&self, exchange: Exchange) -> PdpResult<Exchange> {
        let dir = match self.resolve_siren(&exchange) {
            Some(siren) if !siren.is_empty() => {
                let tenant_out = self.tenants_dir.join(&siren).join("out").join("facture");
                let tenant_root = self.tenants_dir.join(&siren).join("out");
                if tenant_out.is_dir() || tenant_root.parent().map(|p| p.is_dir()).unwrap_or(false) {
                    tenant_out
                } else {
                    tracing::debug!(
                        siren = %siren,
                        "Tenant inconnu pour intra-PDP, fallback vers {}",
                        self.fallback_dir.display()
                    );
                    self.fallback_dir.clone()
                }
            }
            _ => self.fallback_dir.clone(),
        };
        if !dir.exists() {
            std::fs::create_dir_all(&dir)
                .map_err(|e| crate::error::PdpError::IoError(e))?;
        }
        let id_string = exchange.id.to_string();
        let filename = exchange.source_filename.as_deref().unwrap_or(&id_string);
        let file_path = dir.join(filename);
        std::fs::write(&file_path, &exchange.body)
            .map_err(|e| crate::error::PdpError::IoError(e))?;
        tracing::info!(
            filename = %filename,
            path = %file_path.display(),
            role = ?self.role,
            "Fichier écrit dans le répertoire tenant"
        );
        Ok(exchange)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_file_endpoint_poll_stable_files() {
        let dir = tempfile::tempdir().unwrap();
        let dir_path = dir.path().to_str().unwrap();

        // Écrire un fichier complet
        std::fs::write(dir.path().join("facture.xml"), b"<Invoice/>").unwrap();

        // Consumer avec un délai de stabilité court (50ms)
        let endpoint = FileEndpoint::input("test", dir_path)
            .with_stable_delay(50);

        let exchanges = endpoint.poll().await.unwrap();
        assert_eq!(exchanges.len(), 1);
        assert_eq!(exchanges[0].source_filename.as_deref(), Some("facture.xml"));
        assert_eq!(exchanges[0].body, b"<Invoice/>");
    }

    #[tokio::test]
    async fn test_file_endpoint_poll_no_stability_check() {
        let dir = tempfile::tempdir().unwrap();
        let dir_path = dir.path().to_str().unwrap();

        std::fs::write(dir.path().join("a.xml"), b"<A/>").unwrap();
        std::fs::write(dir.path().join("b.xml"), b"<B/>").unwrap();

        // Consumer sans vérification de stabilité
        let endpoint = FileEndpoint::input("test", dir_path)
            .with_stable_delay(0);

        let exchanges = endpoint.poll().await.unwrap();
        assert_eq!(exchanges.len(), 2);
    }

    #[tokio::test]
    async fn test_file_endpoint_poll_empty_dir() {
        let dir = tempfile::tempdir().unwrap();
        let dir_path = dir.path().to_str().unwrap();

        let endpoint = FileEndpoint::input("test", dir_path)
            .with_stable_delay(0);

        let exchanges = endpoint.poll().await.unwrap();
        assert!(exchanges.is_empty());
    }

    #[tokio::test]
    async fn test_file_endpoint_poll_nonexistent_dir() {
        let endpoint = FileEndpoint::input("test", "/tmp/nonexistent_pdp_test_dir_xyz")
            .with_stable_delay(0);

        let exchanges = endpoint.poll().await.unwrap();
        assert!(exchanges.is_empty());
    }

    #[test]
    fn test_file_endpoint_default_stable_delay() {
        let endpoint = FileEndpoint::input("test", "/tmp");
        assert_eq!(endpoint.stable_delay_ms, DEFAULT_STABLE_DELAY_MS);

        let endpoint_out = FileEndpoint::output("test", "/tmp");
        assert_eq!(endpoint_out.stable_delay_ms, 0);
    }

    /// Crée un tar.gz en mémoire contenant les fichiers donnés
    fn create_tar_gz(files: &[(&str, &[u8])]) -> Vec<u8> {
        use flate2::write::GzEncoder;
        use flate2::Compression;
        let buf = Vec::new();
        let enc = GzEncoder::new(buf, Compression::default());
        let mut tar = tar::Builder::new(enc);
        for (name, content) in files {
            let mut header = tar::Header::new_gnu();
            header.set_size(content.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            tar.append_data(&mut header, name, *content).unwrap();
        }
        let enc = tar.into_inner().unwrap();
        enc.finish().unwrap()
    }

    /// Crée un zip en mémoire contenant les fichiers donnés
    fn create_zip(files: &[(&str, &[u8])]) -> Vec<u8> {
        use std::io::Write;
        let buf = std::io::Cursor::new(Vec::new());
        let mut zip = zip::ZipWriter::new(buf);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);
        for (name, content) in files {
            zip.start_file(*name, options).unwrap();
            zip.write_all(content).unwrap();
        }
        zip.finish().unwrap().into_inner()
    }

    #[tokio::test]
    async fn test_file_endpoint_poll_tar_gz() {
        let dir = tempfile::tempdir().unwrap();
        let dir_path = dir.path().to_str().unwrap();

        let tar_gz = create_tar_gz(&[
            ("facture1.xml", b"<Invoice>1</Invoice>"),
            ("facture2.xml", b"<Invoice>2</Invoice>"),
        ]);
        std::fs::write(dir.path().join("lot.tar.gz"), &tar_gz).unwrap();

        let endpoint = FileEndpoint::input("test", dir_path).with_stable_delay(0);
        let exchanges = endpoint.poll().await.unwrap();

        assert_eq!(exchanges.len(), 2);
        let names: Vec<_> = exchanges.iter()
            .map(|e| e.source_filename.as_deref().unwrap_or(""))
            .collect();
        assert!(names.contains(&"facture1.xml"));
        assert!(names.contains(&"facture2.xml"));

        // Vérifier la propriété source_archive
        for ex in &exchanges {
            assert_eq!(ex.get_property("source_archive").map(|s| s.as_str()), Some("lot.tar.gz"));
        }
    }

    #[tokio::test]
    async fn test_file_endpoint_poll_tgz() {
        let dir = tempfile::tempdir().unwrap();
        let dir_path = dir.path().to_str().unwrap();

        let tar_gz = create_tar_gz(&[
            ("invoice.xml", b"<Invoice/>"),
        ]);
        std::fs::write(dir.path().join("archive.tgz"), &tar_gz).unwrap();

        let endpoint = FileEndpoint::input("test", dir_path).with_stable_delay(0);
        let exchanges = endpoint.poll().await.unwrap();

        assert_eq!(exchanges.len(), 1);
        assert_eq!(exchanges[0].source_filename.as_deref(), Some("invoice.xml"));
        assert_eq!(exchanges[0].body, b"<Invoice/>");
    }

    #[tokio::test]
    async fn test_file_endpoint_poll_zip() {
        let dir = tempfile::tempdir().unwrap();
        let dir_path = dir.path().to_str().unwrap();

        let zip_data = create_zip(&[
            ("facture_a.xml", b"<Invoice>A</Invoice>"),
            ("facture_b.xml", b"<Invoice>B</Invoice>"),
            ("facture_c.xml", b"<Invoice>C</Invoice>"),
        ]);
        std::fs::write(dir.path().join("lot.zip"), &zip_data).unwrap();

        let endpoint = FileEndpoint::input("test", dir_path).with_stable_delay(0);
        let exchanges = endpoint.poll().await.unwrap();

        assert_eq!(exchanges.len(), 3);
        let names: Vec<_> = exchanges.iter()
            .map(|e| e.source_filename.as_deref().unwrap_or(""))
            .collect();
        assert!(names.contains(&"facture_a.xml"));
        assert!(names.contains(&"facture_b.xml"));
        assert!(names.contains(&"facture_c.xml"));

        for ex in &exchanges {
            assert_eq!(ex.get_property("source_archive").map(|s| s.as_str()), Some("lot.zip"));
        }
    }

    #[tokio::test]
    async fn test_file_endpoint_poll_mixed_archives_and_files() {
        let dir = tempfile::tempdir().unwrap();
        let dir_path = dir.path().to_str().unwrap();

        // 1 fichier XML normal
        std::fs::write(dir.path().join("direct.xml"), b"<Direct/>").unwrap();

        // 1 tar.gz avec 2 fichiers
        let tar_gz = create_tar_gz(&[
            ("from_tar_1.xml", b"<Tar1/>"),
            ("from_tar_2.xml", b"<Tar2/>"),
        ]);
        std::fs::write(dir.path().join("batch.tar.gz"), &tar_gz).unwrap();

        // 1 zip avec 1 fichier
        let zip_data = create_zip(&[
            ("from_zip.xml", b"<Zip/>"),
        ]);
        std::fs::write(dir.path().join("lot.zip"), &zip_data).unwrap();

        let endpoint = FileEndpoint::input("test", dir_path).with_stable_delay(0);
        let exchanges = endpoint.poll().await.unwrap();

        // 1 direct + 2 tar.gz + 1 zip = 4
        assert_eq!(exchanges.len(), 4);

        let names: Vec<_> = exchanges.iter()
            .map(|e| e.source_filename.as_deref().unwrap_or(""))
            .collect();
        assert!(names.contains(&"direct.xml"));
        assert!(names.contains(&"from_tar_1.xml"));
        assert!(names.contains(&"from_tar_2.xml"));
        assert!(names.contains(&"from_zip.xml"));

        // Le fichier direct n'a pas de source_archive
        let direct = exchanges.iter().find(|e| e.source_filename.as_deref() == Some("direct.xml")).unwrap();
        assert!(direct.get_property("source_archive").is_none());

        // Les fichiers extraits ont source_archive
        let from_tar = exchanges.iter().find(|e| e.source_filename.as_deref() == Some("from_tar_1.xml")).unwrap();
        assert_eq!(from_tar.get_property("source_archive").map(|s| s.as_str()), Some("batch.tar.gz"));
    }

    #[tokio::test]
    async fn test_file_endpoint_archives_consumed_file() {
        // Le poll par défaut doit déplacer le fichier vers `.processed/<date>/`
        // pour éviter qu'un cycle de polling suivant ne le re-traite et
        // déclenche un CDV 213 (BR-FR-12/13) à cause de la dédup.
        let dir = tempfile::tempdir().unwrap();
        let dir_path = dir.path().to_str().unwrap();

        std::fs::write(dir.path().join("facture.xml"), b"<Invoice/>").unwrap();

        let endpoint = FileEndpoint::input("test", dir_path).with_stable_delay(0);
        let exchanges = endpoint.poll().await.unwrap();
        assert_eq!(exchanges.len(), 1);

        // Le fichier d'origine n'existe plus
        assert!(!dir.path().join("facture.xml").exists(),
            "Le fichier consommé doit être déplacé hors de la racine du dossier d'entrée");

        // Il est dans .processed/<YYYYMMDD>/
        let date = chrono::Utc::now().format("%Y%m%d").to_string();
        let archived = dir.path().join(".processed").join(&date).join("facture.xml");
        assert!(archived.exists(), "Le fichier doit être archivé sous .processed/{}/", date);
        assert_eq!(std::fs::read(&archived).unwrap(), b"<Invoice/>");
    }

    #[tokio::test]
    async fn test_file_endpoint_repoll_returns_empty_after_archive() {
        // Régression du bug "tous les CDV en 213" : un 2e poll sur le même
        // dossier ne doit rien renvoyer une fois les fichiers archivés.
        let dir = tempfile::tempdir().unwrap();
        let dir_path = dir.path().to_str().unwrap();

        std::fs::write(dir.path().join("a.xml"), b"<A/>").unwrap();
        std::fs::write(dir.path().join("b.xml"), b"<B/>").unwrap();

        let endpoint = FileEndpoint::input("test", dir_path).with_stable_delay(0);

        let first = endpoint.poll().await.unwrap();
        assert_eq!(first.len(), 2);

        let second = endpoint.poll().await.unwrap();
        assert_eq!(second.len(), 0, "Un 2e poll ne doit pas re-renvoyer les fichiers déjà archivés");
    }

    #[tokio::test]
    async fn test_file_endpoint_with_archive_dir_none_keeps_files() {
        // Opt-out explicite : `with_archive_dir(None)` conserve les fichiers
        // dans le dossier d'entrée (comportement legacy, utilisé par les
        // benches pour pouvoir re-poll le même dossier).
        let dir = tempfile::tempdir().unwrap();
        let dir_path = dir.path().to_str().unwrap();

        std::fs::write(dir.path().join("facture.xml"), b"<Invoice/>").unwrap();

        let endpoint = FileEndpoint::input("test", dir_path)
            .with_stable_delay(0)
            .with_archive_dir(None);

        let first = endpoint.poll().await.unwrap();
        assert_eq!(first.len(), 1);
        assert!(dir.path().join("facture.xml").exists(),
            "Sans archivage, le fichier d'origine doit rester en place");

        let second = endpoint.poll().await.unwrap();
        assert_eq!(second.len(), 1, "Sans archivage, un 2e poll re-renvoie le même fichier");
    }

    #[tokio::test]
    async fn test_file_endpoint_archive_handles_duplicate_filename() {
        // Si le même nom de fichier est re-soumis le même jour, l'archivage
        // doit suffixer pour ne pas écraser l'archive précédente.
        let dir = tempfile::tempdir().unwrap();
        let dir_path = dir.path().to_str().unwrap();

        let endpoint = FileEndpoint::input("test", dir_path).with_stable_delay(0);

        std::fs::write(dir.path().join("dup.xml"), b"<v1/>").unwrap();
        endpoint.poll().await.unwrap();

        // Re-soumission du même nom
        std::fs::write(dir.path().join("dup.xml"), b"<v2/>").unwrap();
        endpoint.poll().await.unwrap();

        let date = chrono::Utc::now().format("%Y%m%d").to_string();
        let archive_day = dir.path().join(".processed").join(&date);

        let archived: Vec<_> = std::fs::read_dir(&archive_day)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();

        assert_eq!(archived.len(), 2, "Les deux versions doivent coexister : {:?}", archived);
        assert!(archived.iter().any(|n| n == "dup.xml"));
        assert!(archived.iter().any(|n| n != "dup.xml" && n.starts_with("dup-") && n.ends_with(".xml")));
    }

    #[tokio::test]
    async fn test_file_endpoint_skips_archive_dir() {
        // Le sous-répertoire `.processed/` est ignoré par le poll même s'il
        // contient des fichiers (issus d'archivages précédents).
        let dir = tempfile::tempdir().unwrap();
        let dir_path = dir.path().to_str().unwrap();

        std::fs::create_dir_all(dir.path().join(".processed/20260101")).unwrap();
        std::fs::write(
            dir.path().join(".processed/20260101/ancien.xml"),
            b"<Ancien/>",
        ).unwrap();
        std::fs::write(dir.path().join("nouveau.xml"), b"<Nouveau/>").unwrap();

        let endpoint = FileEndpoint::input("test", dir_path).with_stable_delay(0);
        let exchanges = endpoint.poll().await.unwrap();

        assert_eq!(exchanges.len(), 1);
        assert_eq!(exchanges[0].source_filename.as_deref(), Some("nouveau.xml"));
    }

    #[tokio::test]
    async fn test_file_endpoint_poll_tar_gz_ignores_directories() {
        let dir = tempfile::tempdir().unwrap();
        let dir_path = dir.path().to_str().unwrap();

        // Créer un tar.gz avec un répertoire et un fichier
        use flate2::write::GzEncoder;
        use flate2::Compression;
        let buf = Vec::new();
        let enc = GzEncoder::new(buf, Compression::default());
        let mut tar = tar::Builder::new(enc);

        // Ajouter un répertoire
        let mut dir_header = tar::Header::new_gnu();
        dir_header.set_entry_type(tar::EntryType::Directory);
        dir_header.set_size(0);
        dir_header.set_mode(0o755);
        dir_header.set_cksum();
        tar.append_data(&mut dir_header, "subdir/", &[] as &[u8]).unwrap();

        // Ajouter un fichier dans le répertoire
        let content = b"<Invoice/>";
        let mut file_header = tar::Header::new_gnu();
        file_header.set_size(content.len() as u64);
        file_header.set_mode(0o644);
        file_header.set_cksum();
        tar.append_data(&mut file_header, "subdir/facture.xml", &content[..]).unwrap();

        let enc = tar.into_inner().unwrap();
        let tar_gz = enc.finish().unwrap();
        std::fs::write(dir.path().join("with_dir.tar.gz"), &tar_gz).unwrap();

        let endpoint = FileEndpoint::input("test", dir_path).with_stable_delay(0);
        let exchanges = endpoint.poll().await.unwrap();

        // Seul le fichier est extrait, pas le répertoire
        assert_eq!(exchanges.len(), 1);
        assert_eq!(exchanges[0].source_filename.as_deref(), Some("facture.xml"));
        assert_eq!(exchanges[0].body, b"<Invoice/>");
    }
}
