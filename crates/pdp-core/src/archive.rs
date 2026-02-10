//! Utilitaires de création d'archives ZIP et tar.gz.
//!
//! Fournit des builders ergonomiques pour créer des archives en mémoire
//! à partir de fichiers (chemins) ou de contenus bruts (bytes).
//!
//! # Exemples
//!
//! ## Créer un tar.gz
//!
//! ```
//! use pdp_core::archive::TarGzBuilder;
//!
//! let tgz = TarGzBuilder::new()
//!     .add("facture_001.xml", b"<Invoice/>")
//!     .add("facture_002.xml", b"<Invoice>2</Invoice>")
//!     .build()
//!     .unwrap();
//!
//! assert!(!tgz.is_empty());
//! ```
//!
//! ## Créer un ZIP
//!
//! ```
//! use pdp_core::archive::ZipBuilder;
//!
//! let zip = ZipBuilder::new()
//!     .add("facture_001.xml", b"<Invoice/>")
//!     .add("rapport.pdf", b"%PDF-1.7 ...")
//!     .build()
//!     .unwrap();
//!
//! assert!(!zip.is_empty());
//! ```

use std::io::{self, Write};
use std::path::Path;

use crate::error::{PdpError, PdpResult};

/// Helper pour créer une erreur IO à partir d'un message.
fn io_err(msg: &str) -> PdpError {
    PdpError::IoError(io::Error::new(io::ErrorKind::Other, msg.to_string()))
}

// ============================================================
// Entrée d'archive (commune aux deux formats)
// ============================================================

/// Fichier à inclure dans une archive.
#[derive(Debug, Clone)]
pub struct ArchiveEntry {
    /// Chemin relatif du fichier dans l'archive (ex: `"factures/fa_001.xml"`)
    pub filename: String,
    /// Contenu binaire du fichier
    pub content: Vec<u8>,
}

// ============================================================
// tar.gz Builder
// ============================================================

/// Builder pour créer des archives tar.gz en mémoire.
///
/// # Exemple
///
/// ```
/// use pdp_core::archive::TarGzBuilder;
///
/// let archive = TarGzBuilder::new()
///     .add("hello.txt", b"Hello, world!")
///     .add("data/config.json", b"{\"key\": \"value\"}")
///     .build()
///     .unwrap();
///
/// // Vérifier le magic number gzip (0x1f 0x8b)
/// assert_eq!(archive[0], 0x1f);
/// assert_eq!(archive[1], 0x8b);
/// ```
pub struct TarGzBuilder {
    entries: Vec<ArchiveEntry>,
    compression_level: u32,
}

impl TarGzBuilder {
    /// Crée un nouveau builder tar.gz.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            compression_level: 6, // défaut flate2
        }
    }

    /// Ajoute un fichier à l'archive à partir de son nom et contenu.
    ///
    /// # Exemple
    ///
    /// ```
    /// use pdp_core::archive::TarGzBuilder;
    ///
    /// let archive = TarGzBuilder::new()
    ///     .add("test.txt", b"contenu")
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn add(mut self, filename: &str, content: &[u8]) -> Self {
        self.entries.push(ArchiveEntry {
            filename: filename.to_string(),
            content: content.to_vec(),
        });
        self
    }

    /// Ajoute un fichier à l'archive depuis un `ArchiveEntry`.
    pub fn add_entry(mut self, entry: ArchiveEntry) -> Self {
        self.entries.push(entry);
        self
    }

    /// Ajoute un fichier à l'archive en lisant son contenu depuis le disque.
    ///
    /// Le nom dans l'archive sera le nom du fichier (sans le chemin parent).
    ///
    /// # Exemple
    ///
    /// ```no_run
    /// use pdp_core::archive::TarGzBuilder;
    ///
    /// let archive = TarGzBuilder::new()
    ///     .add_file("data/facture.xml")
    ///     .unwrap()
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn add_file(self, path: &str) -> PdpResult<Self> {
        let p = Path::new(path);
        let content = std::fs::read(p)?;
        let filename = p
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(path)
            .to_string();
        Ok(self.add(&filename, &content))
    }

    /// Ajoute un fichier en spécifiant le nom dans l'archive et le chemin sur disque.
    pub fn add_file_as(self, archive_name: &str, path: &str) -> PdpResult<Self> {
        let content = std::fs::read(path)?;
        Ok(self.add(archive_name, &content))
    }

    /// Définit le niveau de compression gzip (0-9, défaut: 6).
    pub fn compression_level(mut self, level: u32) -> Self {
        self.compression_level = level.min(9);
        self
    }

    /// Construit l'archive tar.gz et retourne son contenu binaire.
    ///
    /// # Erreurs
    ///
    /// Retourne une erreur si l'archive est vide ou si l'écriture échoue.
    pub fn build(&self) -> PdpResult<Vec<u8>> {
        if self.entries.is_empty() {
            return Err(io_err("Impossible de créer une archive tar.gz vide"));
        }

        let buf = Vec::new();
        let encoder = flate2::write::GzEncoder::new(buf, flate2::Compression::new(self.compression_level));
        let mut tar_builder = tar::Builder::new(encoder);

        for entry in &self.entries {
            let mut header = tar::Header::new_gnu();
            header.set_size(entry.content.len() as u64);
            header.set_mode(0o644);
            header.set_mtime(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
            );
            header.set_cksum();

            tar_builder
                .append_data(&mut header, &entry.filename, entry.content.as_slice())?;
        }

        let encoder = tar_builder.into_inner()?;
        let compressed = encoder.finish()?;

        Ok(compressed)
    }
}

impl Default for TarGzBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================
// ZIP Builder
// ============================================================

/// Builder pour créer des archives ZIP en mémoire.
///
/// # Exemple
///
/// ```
/// use pdp_core::archive::ZipBuilder;
///
/// let archive = ZipBuilder::new()
///     .add("hello.txt", b"Hello!")
///     .add("sous-dossier/data.json", b"{}")
///     .build()
///     .unwrap();
///
/// // Vérifier le magic number ZIP (PK\x03\x04)
/// assert_eq!(&archive[0..2], b"PK");
/// ```
pub struct ZipBuilder {
    entries: Vec<ArchiveEntry>,
    compression: zip::CompressionMethod,
}

impl ZipBuilder {
    /// Crée un nouveau builder ZIP.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            compression: zip::CompressionMethod::Deflated,
        }
    }

    /// Ajoute un fichier à l'archive à partir de son nom et contenu.
    ///
    /// # Exemple
    ///
    /// ```
    /// use pdp_core::archive::ZipBuilder;
    ///
    /// let archive = ZipBuilder::new()
    ///     .add("test.txt", b"contenu")
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn add(mut self, filename: &str, content: &[u8]) -> Self {
        self.entries.push(ArchiveEntry {
            filename: filename.to_string(),
            content: content.to_vec(),
        });
        self
    }

    /// Ajoute un fichier à l'archive depuis un `ArchiveEntry`.
    pub fn add_entry(mut self, entry: ArchiveEntry) -> Self {
        self.entries.push(entry);
        self
    }

    /// Ajoute un fichier à l'archive en lisant son contenu depuis le disque.
    ///
    /// Le nom dans l'archive sera le nom du fichier (sans le chemin parent).
    pub fn add_file(self, path: &str) -> PdpResult<Self> {
        let p = Path::new(path);
        let content = std::fs::read(p)?;
        let filename = p
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(path)
            .to_string();
        Ok(self.add(&filename, &content))
    }

    /// Ajoute un fichier en spécifiant le nom dans l'archive et le chemin sur disque.
    pub fn add_file_as(self, archive_name: &str, path: &str) -> PdpResult<Self> {
        let content = std::fs::read(path)?;
        Ok(self.add(archive_name, &content))
    }

    /// Utilise la compression Stored (pas de compression).
    pub fn no_compression(mut self) -> Self {
        self.compression = zip::CompressionMethod::Stored;
        self
    }

    /// Utilise la compression Deflate (défaut).
    pub fn deflate(mut self) -> Self {
        self.compression = zip::CompressionMethod::Deflated;
        self
    }

    /// Construit l'archive ZIP et retourne son contenu binaire.
    ///
    /// # Erreurs
    ///
    /// Retourne une erreur si l'archive est vide ou si l'écriture échoue.
    pub fn build(&self) -> PdpResult<Vec<u8>> {
        if self.entries.is_empty() {
            return Err(io_err("Impossible de créer une archive ZIP vide"));
        }

        let buf = io::Cursor::new(Vec::new());
        let mut zip_writer = zip::ZipWriter::new(buf);

        let options = zip::write::SimpleFileOptions::default()
            .compression_method(self.compression);

        for entry in &self.entries {
            zip_writer
                .start_file(&entry.filename, options)
                .map_err(|e| io_err(&format!("ZIP '{}': {}", entry.filename, e)))?;

            zip_writer.write_all(&entry.content)?;
        }

        let cursor = zip_writer
            .finish()
            .map_err(|e| io_err(&format!("Finalisation ZIP: {}", e)))?;

        Ok(cursor.into_inner())
    }
}

impl Default for ZipBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================
// Fonctions utilitaires de lecture
// ============================================================

/// Lit et décompresse une archive tar.gz, retourne la liste des fichiers.
///
/// # Exemple
///
/// ```
/// use pdp_core::archive::{TarGzBuilder, read_tar_gz};
///
/// let tgz = TarGzBuilder::new()
///     .add("a.txt", b"hello")
///     .add("b.txt", b"world")
///     .build()
///     .unwrap();
///
/// let entries = read_tar_gz(&tgz).unwrap();
/// assert_eq!(entries.len(), 2);
/// assert_eq!(entries[0].filename, "a.txt");
/// assert_eq!(entries[0].content, b"hello");
/// ```
pub fn read_tar_gz(data: &[u8]) -> PdpResult<Vec<ArchiveEntry>> {
    use std::io::Read;

    let decoder = flate2::read::GzDecoder::new(data);
    let mut archive = tar::Archive::new(decoder);
    let mut entries = Vec::new();

    for entry_result in archive.entries()? {
        let mut entry = entry_result?;
        let filename = entry.path()?.to_string_lossy().to_string();

        let mut content = Vec::new();
        entry.read_to_end(&mut content)?;

        entries.push(ArchiveEntry { filename, content });
    }

    Ok(entries)
}

/// Lit une archive ZIP, retourne la liste des fichiers.
///
/// # Exemple
///
/// ```
/// use pdp_core::archive::{ZipBuilder, read_zip};
///
/// let zip_data = ZipBuilder::new()
///     .add("a.txt", b"hello")
///     .add("b.txt", b"world")
///     .build()
///     .unwrap();
///
/// let entries = read_zip(&zip_data).unwrap();
/// assert_eq!(entries.len(), 2);
/// assert_eq!(entries[0].filename, "a.txt");
/// assert_eq!(entries[0].content, b"hello");
/// ```
pub fn read_zip(data: &[u8]) -> PdpResult<Vec<ArchiveEntry>> {
    use std::io::Read;

    let cursor = io::Cursor::new(data);
    let mut archive = zip::ZipArchive::new(cursor)
        .map_err(|e| io_err(&format!("Lecture ZIP: {}", e)))?;

    let mut entries = Vec::new();

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| io_err(&format!("Entrée ZIP #{}: {}", i, e)))?;

        if file.is_dir() {
            continue;
        }

        let filename = file.name().to_string();
        let mut content = Vec::new();
        file.read_to_end(&mut content)?;

        entries.push(ArchiveEntry { filename, content });
    }

    Ok(entries)
}

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    // --- TarGzBuilder ---

    #[test]
    fn test_tgz_single_file() {
        let tgz = TarGzBuilder::new()
            .add("hello.txt", b"Hello, world!")
            .build()
            .unwrap();

        // Magic number gzip
        assert_eq!(tgz[0], 0x1f);
        assert_eq!(tgz[1], 0x8b);

        let entries = read_tar_gz(&tgz).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].filename, "hello.txt");
        assert_eq!(entries[0].content, b"Hello, world!");
    }

    #[test]
    fn test_tgz_multiple_files() {
        let tgz = TarGzBuilder::new()
            .add("a.xml", b"<a/>")
            .add("b.xml", b"<b/>")
            .add("c.xml", b"<c/>")
            .build()
            .unwrap();

        let entries = read_tar_gz(&tgz).unwrap();
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].filename, "a.xml");
        assert_eq!(entries[1].filename, "b.xml");
        assert_eq!(entries[2].filename, "c.xml");
        assert_eq!(entries[2].content, b"<c/>");
    }

    #[test]
    fn test_tgz_empty_fails() {
        let result = TarGzBuilder::new().build();
        assert!(result.is_err());
    }

    #[test]
    fn test_tgz_subdirectories() {
        let tgz = TarGzBuilder::new()
            .add("factures/fa_001.xml", b"<Invoice>1</Invoice>")
            .add("factures/fa_002.xml", b"<Invoice>2</Invoice>")
            .add("cdv/cdv_200.xml", b"<CDAR/>")
            .build()
            .unwrap();

        let entries = read_tar_gz(&tgz).unwrap();
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].filename, "factures/fa_001.xml");
    }

    #[test]
    fn test_tgz_large_content() {
        let large = vec![0x42u8; 1_000_000]; // 1 Mo
        let tgz = TarGzBuilder::new()
            .add("large.bin", &large)
            .build()
            .unwrap();

        // Compressé doit être bien plus petit (contenu répétitif)
        assert!(tgz.len() < 10_000, "tar.gz devrait être compressé: {} octets", tgz.len());

        let entries = read_tar_gz(&tgz).unwrap();
        assert_eq!(entries[0].content.len(), 1_000_000);
        assert_eq!(entries[0].content, large);
    }

    #[test]
    fn test_tgz_compression_levels() {
        let content = b"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".repeat(100);

        let fast = TarGzBuilder::new()
            .compression_level(1)
            .add("data.txt", &content)
            .build()
            .unwrap();

        let best = TarGzBuilder::new()
            .compression_level(9)
            .add("data.txt", &content)
            .build()
            .unwrap();

        // Les deux doivent être valides
        assert_eq!(read_tar_gz(&fast).unwrap()[0].content, content);
        assert_eq!(read_tar_gz(&best).unwrap()[0].content, content);

        // Meilleure compression = plus petit (ou égal)
        assert!(best.len() <= fast.len());
    }

    #[test]
    fn test_tgz_binary_content() {
        let binary = vec![0u8, 1, 2, 255, 254, 253, 0, 0, 128];
        let tgz = TarGzBuilder::new()
            .add("data.bin", &binary)
            .build()
            .unwrap();

        let entries = read_tar_gz(&tgz).unwrap();
        assert_eq!(entries[0].content, binary);
    }

    #[test]
    fn test_tgz_add_entry() {
        let entry = ArchiveEntry {
            filename: "test.xml".to_string(),
            content: b"<test/>".to_vec(),
        };
        let tgz = TarGzBuilder::new().add_entry(entry).build().unwrap();
        let entries = read_tar_gz(&tgz).unwrap();
        assert_eq!(entries[0].filename, "test.xml");
    }

    #[test]
    fn test_tgz_from_file() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("facture.xml");
        std::fs::write(&file_path, b"<Invoice/>").unwrap();

        let tgz = TarGzBuilder::new()
            .add_file(file_path.to_str().unwrap())
            .unwrap()
            .build()
            .unwrap();

        let entries = read_tar_gz(&tgz).unwrap();
        assert_eq!(entries[0].filename, "facture.xml");
        assert_eq!(entries[0].content, b"<Invoice/>");
    }

    #[test]
    fn test_tgz_from_file_as() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("source.xml");
        std::fs::write(&file_path, b"<Invoice/>").unwrap();

        let tgz = TarGzBuilder::new()
            .add_file_as("Base_facture_001.xml", file_path.to_str().unwrap())
            .unwrap()
            .build()
            .unwrap();

        let entries = read_tar_gz(&tgz).unwrap();
        assert_eq!(entries[0].filename, "Base_facture_001.xml");
    }

    // --- ZipBuilder ---

    #[test]
    fn test_zip_single_file() {
        let zip_data = ZipBuilder::new()
            .add("hello.txt", b"Hello, world!")
            .build()
            .unwrap();

        // Magic number ZIP
        assert_eq!(&zip_data[0..2], b"PK");

        let entries = read_zip(&zip_data).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].filename, "hello.txt");
        assert_eq!(entries[0].content, b"Hello, world!");
    }

    #[test]
    fn test_zip_multiple_files() {
        let zip_data = ZipBuilder::new()
            .add("a.xml", b"<a/>")
            .add("b.xml", b"<b/>")
            .add("c.xml", b"<c/>")
            .build()
            .unwrap();

        let entries = read_zip(&zip_data).unwrap();
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].filename, "a.xml");
        assert_eq!(entries[1].filename, "b.xml");
        assert_eq!(entries[2].filename, "c.xml");
        assert_eq!(entries[2].content, b"<c/>");
    }

    #[test]
    fn test_zip_empty_fails() {
        let result = ZipBuilder::new().build();
        assert!(result.is_err());
    }

    #[test]
    fn test_zip_subdirectories() {
        let zip_data = ZipBuilder::new()
            .add("factures/fa_001.xml", b"<Invoice>1</Invoice>")
            .add("factures/fa_002.xml", b"<Invoice>2</Invoice>")
            .add("cdv/cdv_200.xml", b"<CDAR/>")
            .build()
            .unwrap();

        let entries = read_zip(&zip_data).unwrap();
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].filename, "factures/fa_001.xml");
    }

    #[test]
    fn test_zip_large_content() {
        let large = vec![0x42u8; 1_000_000];
        let zip_data = ZipBuilder::new()
            .add("large.bin", &large)
            .build()
            .unwrap();

        assert!(zip_data.len() < 10_000, "ZIP devrait être compressé: {} octets", zip_data.len());

        let entries = read_zip(&zip_data).unwrap();
        assert_eq!(entries[0].content.len(), 1_000_000);
        assert_eq!(entries[0].content, large);
    }

    #[test]
    fn test_zip_no_compression() {
        let content = b"Hello, world!";
        let zip_data = ZipBuilder::new()
            .no_compression()
            .add("hello.txt", content)
            .build()
            .unwrap();

        let entries = read_zip(&zip_data).unwrap();
        assert_eq!(entries[0].content, content);
        // Sans compression, le ZIP est plus gros que le contenu (headers)
        assert!(zip_data.len() > content.len());
    }

    #[test]
    fn test_zip_binary_content() {
        let binary = vec![0u8, 1, 2, 255, 254, 253, 0, 0, 128];
        let zip_data = ZipBuilder::new()
            .add("data.bin", &binary)
            .build()
            .unwrap();

        let entries = read_zip(&zip_data).unwrap();
        assert_eq!(entries[0].content, binary);
    }

    #[test]
    fn test_zip_add_entry() {
        let entry = ArchiveEntry {
            filename: "test.xml".to_string(),
            content: b"<test/>".to_vec(),
        };
        let zip_data = ZipBuilder::new().add_entry(entry).build().unwrap();
        let entries = read_zip(&zip_data).unwrap();
        assert_eq!(entries[0].filename, "test.xml");
    }

    #[test]
    fn test_zip_from_file() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("facture.xml");
        std::fs::write(&file_path, b"<Invoice/>").unwrap();

        let zip_data = ZipBuilder::new()
            .add_file(file_path.to_str().unwrap())
            .unwrap()
            .build()
            .unwrap();

        let entries = read_zip(&zip_data).unwrap();
        assert_eq!(entries[0].filename, "facture.xml");
        assert_eq!(entries[0].content, b"<Invoice/>");
    }

    #[test]
    fn test_zip_from_file_as() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("source.xml");
        std::fs::write(&file_path, b"<Invoice/>").unwrap();

        let zip_data = ZipBuilder::new()
            .add_file_as("Base_facture_001.xml", file_path.to_str().unwrap())
            .unwrap()
            .build()
            .unwrap();

        let entries = read_zip(&zip_data).unwrap();
        assert_eq!(entries[0].filename, "Base_facture_001.xml");
    }

    // --- Roundtrip mixte ---

    #[test]
    fn test_same_content_both_formats() {
        let files = vec![
            ("facture_001.xml", b"<Invoice><ID>FA-001</ID></Invoice>".as_slice()),
            ("facture_002.xml", b"<Invoice><ID>FA-002</ID></Invoice>".as_slice()),
            ("avoir_001.xml", b"<CreditNote><ID>AV-001</ID></CreditNote>".as_slice()),
        ];

        let mut tgz_builder = TarGzBuilder::new();
        let mut zip_builder = ZipBuilder::new();
        for (name, content) in &files {
            tgz_builder = tgz_builder.add(name, content);
            zip_builder = zip_builder.add(name, content);
        }

        let tgz = tgz_builder.build().unwrap();
        let zip_data = zip_builder.build().unwrap();

        let tgz_entries = read_tar_gz(&tgz).unwrap();
        let zip_entries = read_zip(&zip_data).unwrap();

        assert_eq!(tgz_entries.len(), 3);
        assert_eq!(zip_entries.len(), 3);

        for i in 0..3 {
            assert_eq!(tgz_entries[i].filename, zip_entries[i].filename);
            assert_eq!(tgz_entries[i].content, zip_entries[i].content);
        }
    }

    #[test]
    fn test_file_not_found() {
        let result = TarGzBuilder::new().add_file("/nonexistent/file.xml");
        assert!(result.is_err());

        let result = ZipBuilder::new().add_file("/nonexistent/file.xml");
        assert!(result.is_err());
    }
}
