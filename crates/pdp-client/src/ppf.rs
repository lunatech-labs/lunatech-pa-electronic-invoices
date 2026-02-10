use flate2::write::GzEncoder;
use flate2::Compression;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::error::{ClientError, ClientResult};

// ============================================================
// Codes interfaces PPF — Tableau 4 des specs externes v3.1
// ============================================================

/// Code interface PPF identifiant la nature et le format d'un flux.
/// Specs externes v3.1, chapitre 3.4.6 « Le nommage des flux ».
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CodeInterface {
    /// F1 — Données réglementaires au format UBL
    #[serde(rename = "FFE0111A")]
    F1Ubl,
    /// F1 — Données réglementaires au format CII
    #[serde(rename = "FFE0112A")]
    F1Cii,
    /// F6 — Cycle de vie de factures (CDAR)
    #[serde(rename = "FFE0614A")]
    F6Facture,
    /// F6 — Cycle de vie de données réglementaires (CDAR)
    #[serde(rename = "FFE0604A")]
    F6DonneesReglementaires,
    /// F6 — Cycle de vie de statuts obligatoires (CDAR)
    #[serde(rename = "FFE0654A")]
    F6StatutsObligatoires,
    /// F6 — Cycle de vie de données de transaction et de paiement (CDAR)
    #[serde(rename = "FFE0624A")]
    F6TransactionPaiement,
    /// F6 — Cycle de vie d'actualisation de l'annuaire (CDAR)
    #[serde(rename = "FFE0634A")]
    F6Annuaire,
    /// F10 — Données de transaction et de paiement (format spécifique)
    #[serde(rename = "FFE1025A")]
    F10TransactionPaiement,
    /// F13 — Actualisation de l'annuaire (format spécifique)
    #[serde(rename = "FFE1235A")]
    F13Annuaire,
    /// F14 — Export de l'annuaire (format spécifique)
    #[serde(rename = "FFE1435A")]
    F14ExportAnnuaire,
}

impl CodeInterface {
    /// Retourne le code interface sous forme de chaîne (8 caractères).
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::F1Ubl => "FFE0111A",
            Self::F1Cii => "FFE0112A",
            Self::F6Facture => "FFE0614A",
            Self::F6DonneesReglementaires => "FFE0604A",
            Self::F6StatutsObligatoires => "FFE0654A",
            Self::F6TransactionPaiement => "FFE0624A",
            Self::F6Annuaire => "FFE0634A",
            Self::F10TransactionPaiement => "FFE1025A",
            Self::F13Annuaire => "FFE1235A",
            Self::F14ExportAnnuaire => "FFE1435A",
        }
    }

    /// Retourne les 4 chiffres du code interface (partie IIII).
    pub fn digits(&self) -> &'static str {
        match self {
            Self::F1Ubl => "0111",
            Self::F1Cii => "0112",
            Self::F6Facture => "0614",
            Self::F6DonneesReglementaires => "0604",
            Self::F6StatutsObligatoires => "0654",
            Self::F6TransactionPaiement => "0624",
            Self::F6Annuaire => "0634",
            Self::F10TransactionPaiement => "1025",
            Self::F13Annuaire => "1235",
            Self::F14ExportAnnuaire => "1435",
        }
    }
}

impl std::fmt::Display for CodeInterface {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// ============================================================
// Profil sémantique des fichiers F1
// ============================================================

/// Profil sémantique d'un fichier F1 (données réglementaires).
/// Détermine la trajectoire de traitement par le PPF.
/// Specs externes v3.1, note 74 : « <profil>_<nom_de_fichier>.xml »
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProfilF1 {
    /// Profil Base — socle de données réglementaires
    Base,
    /// Profil Full — données réglementaires complètes
    Full,
}

impl ProfilF1 {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Base => "Base",
            Self::Full => "Full",
        }
    }
}

impl std::fmt::Display for ProfilF1 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// ============================================================
// Nommage des flux PPF — chapitre 3.4.6
// ============================================================

/// Configuration du nommage des flux PPF.
/// Specs externes v3.1, chapitre 3.4.6 « Le nommage des flux ».
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PpfFluxConfig {
    /// Code application du partenaire (6 caractères alphanumériques).
    /// Attribué lors du raccordement au PPF.
    pub code_application: String,
}

impl PpfFluxConfig {
    pub fn new(code_application: &str) -> ClientResult<Self> {
        if code_application.len() != 6 || !code_application.chars().all(|c| c.is_ascii_alphanumeric()) {
            return Err(ClientError::ConfigError(format!(
                "Code application PPF invalide '{}': doit être exactement 6 caractères alphanumériques",
                code_application
            )));
        }
        Ok(Self {
            code_application: code_application.to_string(),
        })
    }
}

/// Génère le nom de l'enveloppe d'un flux PPF (fichier tar.gz).
///
/// Format : `{CODE_INTERFACE}_{CODE_APP}_{IDENTIFIANT_FLUX}.tar.gz`
///
/// L'identifiant du flux fait 25 caractères :
/// - 6 premiers = code application de l'émetteur
/// - 19 suivants = numéro de séquence (chiffres ou lettres majuscules)
///
/// Exemple : `FFE0111A_AAA123_AAA1230111000000000000001.tar.gz`
pub fn flux_envelope_name(
    code_interface: CodeInterface,
    code_application: &str,
    sequence_number: &str,
) -> ClientResult<String> {
    // Valider le code application (6 chars alphanum)
    if code_application.len() != 6 || !code_application.chars().all(|c| c.is_ascii_alphanumeric()) {
        return Err(ClientError::ConfigError(format!(
            "Code application invalide '{}': 6 caractères alphanumériques requis",
            code_application
        )));
    }

    // Valider le numéro de séquence (19 chars, chiffres ou lettres majuscules)
    if sequence_number.len() != 19
        || !sequence_number.chars().all(|c| c.is_ascii_digit() || c.is_ascii_uppercase())
    {
        return Err(ClientError::ConfigError(format!(
            "Numéro de séquence invalide '{}': 19 caractères (chiffres ou lettres majuscules) requis",
            sequence_number
        )));
    }

    // Identifiant flux = code_app (6) + séquence (19) = 25 chars
    let identifiant_flux = format!("{}{}", code_application, sequence_number);

    Ok(format!(
        "{}_{}_{}.tar.gz",
        code_interface.as_str(),
        code_application,
        identifiant_flux
    ))
}

/// Génère un numéro de séquence recommandé par l'AIFE.
///
/// Format recommandé : code_interface_digits (4) + compteur (15 chiffres)
/// Total = 19 caractères
pub fn recommended_sequence(code_interface: CodeInterface, counter: u64) -> String {
    format!("{}{:015}", code_interface.digits(), counter)
}

/// Génère le nom d'un fichier F1 à l'intérieur de l'archive tar.gz.
///
/// Format : `{profil}_{nom_de_fichier}.xml`
/// Specs externes v3.1, note 74.
pub fn f1_inner_filename(profil: ProfilF1, base_name: &str) -> String {
    let name = base_name.trim_end_matches(".xml");
    format!("{}_{}.xml", profil.as_str(), name)
}

// ============================================================
// Construction d'archives tar.gz pour le PPF
// ============================================================

/// Fichier à inclure dans une archive tar.gz PPF.
#[derive(Debug, Clone)]
pub struct FluxFile {
    /// Nom du fichier dans l'archive
    pub filename: String,
    /// Contenu du fichier (XML UTF-8)
    pub content: Vec<u8>,
}

/// Construit une archive tar.gz contenant les fichiers donnés.
///
/// Tous les fichiers dans un flux doivent être de même nature et même format.
/// Taille max : 1 Go pour le flux, 120 Mo par fichier.
pub fn build_tar_gz(files: &[FluxFile]) -> ClientResult<Vec<u8>> {
    if files.is_empty() {
        return Err(ClientError::ConfigError(
            "Impossible de créer un flux vide (IRR_VIDE)".to_string(),
        ));
    }

    // Vérifier la taille de chaque fichier (120 Mo max)
    const MAX_FILE_SIZE: usize = 120 * 1024 * 1024;
    for file in files {
        if file.content.len() > MAX_FILE_SIZE {
            return Err(ClientError::ConfigError(format!(
                "Fichier '{}' dépasse la taille maximale de 120 Mo ({} octets)",
                file.filename,
                file.content.len()
            )));
        }
        if file.content.is_empty() {
            return Err(ClientError::ConfigError(format!(
                "Fichier '{}' est vide (IRR_VIDE_F)",
                file.filename
            )));
        }
    }

    let buf = Vec::new();
    let encoder = GzEncoder::new(buf, Compression::default());
    let mut archive = tar::Builder::new(encoder);

    for file in files {
        let mut header = tar::Header::new_gnu();
        header.set_size(file.content.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();

        archive
            .append_data(&mut header, &file.filename, file.content.as_slice())
            .map_err(|e| {
                ClientError::IoError(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Erreur ajout fichier '{}' dans tar.gz: {}", file.filename, e),
                ))
            })?;
    }

    let encoder = archive.into_inner().map_err(|e| {
        ClientError::IoError(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Erreur finalisation archive tar: {}", e),
        ))
    })?;

    let compressed = encoder.finish().map_err(|e| {
        ClientError::IoError(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Erreur finalisation compression gzip: {}", e),
        ))
    })?;

    // Vérifier la taille totale du flux (1 Go max)
    const MAX_FLUX_SIZE: usize = 1024 * 1024 * 1024;
    if compressed.len() > MAX_FLUX_SIZE {
        return Err(ClientError::ConfigError(format!(
            "Flux dépasse la taille maximale de 1 Go ({} octets)",
            compressed.len()
        )));
    }

    Ok(compressed)
}

/// Calcule le SHA-256 d'un contenu (utile pour les checksums de flux)
pub fn sha256_hex(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

// ============================================================
// Helpers de construction de flux complets
// ============================================================

/// Construit un flux F1 complet (enveloppe tar.gz nommée + fichiers XML).
///
/// Retourne le nom du fichier tar.gz et son contenu binaire.
pub fn build_f1_flux(
    config: &PpfFluxConfig,
    code_interface: CodeInterface,
    sequence_number: &str,
    files: &[(ProfilF1, &str, &[u8])],
) -> ClientResult<(String, Vec<u8>)> {
    // Valider que le code interface est bien un F1
    match code_interface {
        CodeInterface::F1Ubl | CodeInterface::F1Cii => {}
        _ => {
            return Err(ClientError::ConfigError(format!(
                "Code interface {} n'est pas un flux F1",
                code_interface
            )));
        }
    }

    let flux_files: Vec<FluxFile> = files
        .iter()
        .map(|(profil, name, content)| FluxFile {
            filename: f1_inner_filename(*profil, name),
            content: content.to_vec(),
        })
        .collect();

    let envelope_name = flux_envelope_name(
        code_interface,
        &config.code_application,
        sequence_number,
    )?;

    let tar_gz = build_tar_gz(&flux_files)?;

    Ok((envelope_name, tar_gz))
}

/// Construit un flux F6 (cycle de vie CDAR) complet.
pub fn build_f6_flux(
    config: &PpfFluxConfig,
    code_interface: CodeInterface,
    sequence_number: &str,
    cdar_files: &[(&str, &[u8])],
) -> ClientResult<(String, Vec<u8>)> {
    // Valider que le code interface est bien un F6
    match code_interface {
        CodeInterface::F6Facture
        | CodeInterface::F6DonneesReglementaires
        | CodeInterface::F6StatutsObligatoires
        | CodeInterface::F6TransactionPaiement
        | CodeInterface::F6Annuaire => {}
        _ => {
            return Err(ClientError::ConfigError(format!(
                "Code interface {} n'est pas un flux F6",
                code_interface
            )));
        }
    }

    let flux_files: Vec<FluxFile> = cdar_files
        .iter()
        .map(|(name, content)| FluxFile {
            filename: name.to_string(),
            content: content.to_vec(),
        })
        .collect();

    let envelope_name = flux_envelope_name(
        code_interface,
        &config.code_application,
        sequence_number,
    )?;

    let tar_gz = build_tar_gz(&flux_files)?;

    Ok((envelope_name, tar_gz))
}

/// Construit un flux F10 (e-reporting transactions/paiements) complet.
pub fn build_f10_flux(
    config: &PpfFluxConfig,
    sequence_number: &str,
    report_files: &[(&str, &[u8])],
) -> ClientResult<(String, Vec<u8>)> {
    let flux_files: Vec<FluxFile> = report_files
        .iter()
        .map(|(name, content)| FluxFile {
            filename: name.to_string(),
            content: content.to_vec(),
        })
        .collect();

    let envelope_name = flux_envelope_name(
        CodeInterface::F10TransactionPaiement,
        &config.code_application,
        sequence_number,
    )?;

    let tar_gz = build_tar_gz(&flux_files)?;

    Ok((envelope_name, tar_gz))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code_interface_as_str() {
        assert_eq!(CodeInterface::F1Ubl.as_str(), "FFE0111A");
        assert_eq!(CodeInterface::F1Cii.as_str(), "FFE0112A");
        assert_eq!(CodeInterface::F6Facture.as_str(), "FFE0614A");
        assert_eq!(CodeInterface::F6StatutsObligatoires.as_str(), "FFE0654A");
        assert_eq!(CodeInterface::F10TransactionPaiement.as_str(), "FFE1025A");
        assert_eq!(CodeInterface::F13Annuaire.as_str(), "FFE1235A");
    }

    #[test]
    fn test_code_interface_digits() {
        assert_eq!(CodeInterface::F1Ubl.digits(), "0111");
        assert_eq!(CodeInterface::F1Cii.digits(), "0112");
        assert_eq!(CodeInterface::F6Facture.digits(), "0614");
    }

    #[test]
    fn test_recommended_sequence() {
        let seq = recommended_sequence(CodeInterface::F1Ubl, 1);
        assert_eq!(seq, "0111000000000000001");
        assert_eq!(seq.len(), 19);

        let seq = recommended_sequence(CodeInterface::F1Cii, 123456789);
        assert_eq!(seq, "0112000000123456789");
    }

    #[test]
    fn test_flux_envelope_name() {
        let name = flux_envelope_name(
            CodeInterface::F1Ubl,
            "AAA123",
            "0111000000000000001",
        )
        .unwrap();
        assert_eq!(name, "FFE0111A_AAA123_AAA1230111000000000000001.tar.gz");
    }

    #[test]
    fn test_flux_envelope_name_example_from_specs() {
        // Exemple des specs : FFE0111A_AAA123_AAA1230111000000000000001
        let seq = recommended_sequence(CodeInterface::F1Ubl, 1);
        let name = flux_envelope_name(CodeInterface::F1Ubl, "AAA123", &seq).unwrap();
        assert_eq!(name, "FFE0111A_AAA123_AAA1230111000000000000001.tar.gz");
    }

    #[test]
    fn test_flux_envelope_name_invalid_code_app() {
        let result = flux_envelope_name(CodeInterface::F1Ubl, "AB", "0111000000000000001");
        assert!(result.is_err());

        let result = flux_envelope_name(CodeInterface::F1Ubl, "ABC12!", "0111000000000000001");
        assert!(result.is_err());
    }

    #[test]
    fn test_flux_envelope_name_invalid_sequence() {
        let result = flux_envelope_name(CodeInterface::F1Ubl, "AAA123", "0111");
        assert!(result.is_err());

        let result = flux_envelope_name(CodeInterface::F1Ubl, "AAA123", "011100000000000000!");
        assert!(result.is_err());
    }

    #[test]
    fn test_f1_inner_filename() {
        assert_eq!(
            f1_inner_filename(ProfilF1::Base, "facture_001"),
            "Base_facture_001.xml"
        );
        assert_eq!(
            f1_inner_filename(ProfilF1::Full, "facture_001.xml"),
            "Full_facture_001.xml"
        );
    }

    #[test]
    fn test_ppf_flux_config_valid() {
        let config = PpfFluxConfig::new("AAA123").unwrap();
        assert_eq!(config.code_application, "AAA123");
    }

    #[test]
    fn test_ppf_flux_config_invalid() {
        assert!(PpfFluxConfig::new("AB").is_err());
        assert!(PpfFluxConfig::new("ABC12!").is_err());
        assert!(PpfFluxConfig::new("ABCDEFG").is_err());
    }

    #[test]
    fn test_build_tar_gz_single_file() {
        let files = vec![FluxFile {
            filename: "Base_facture_001.xml".to_string(),
            content: b"<Invoice/>".to_vec(),
        }];
        let result = build_tar_gz(&files);
        assert!(result.is_ok());
        let tar_gz = result.unwrap();
        assert!(!tar_gz.is_empty());

        // Vérifier qu'on peut décompresser
        let decoder = flate2::read::GzDecoder::new(tar_gz.as_slice());
        let mut archive = tar::Archive::new(decoder);
        let entries: Vec<_> = archive.entries().unwrap().collect();
        assert_eq!(entries.len(), 1);
    }

    #[test]
    fn test_build_tar_gz_multiple_files() {
        let files = vec![
            FluxFile {
                filename: "Base_facture_001.xml".to_string(),
                content: b"<Invoice>1</Invoice>".to_vec(),
            },
            FluxFile {
                filename: "Full_facture_002.xml".to_string(),
                content: b"<Invoice>2</Invoice>".to_vec(),
            },
        ];
        let result = build_tar_gz(&files);
        assert!(result.is_ok());

        let tar_gz = result.unwrap();
        let decoder = flate2::read::GzDecoder::new(tar_gz.as_slice());
        let mut archive = tar::Archive::new(decoder);
        let entries: Vec<_> = archive.entries().unwrap().collect();
        assert_eq!(entries.len(), 2);
    }

    #[test]
    fn test_build_tar_gz_empty_fails() {
        let result = build_tar_gz(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_build_tar_gz_empty_file_fails() {
        let files = vec![FluxFile {
            filename: "empty.xml".to_string(),
            content: vec![],
        }];
        let result = build_tar_gz(&files);
        assert!(result.is_err());
    }

    #[test]
    fn test_build_tar_gz_content_preserved() {
        use std::io::Read;

        let xml_content = b"<?xml version=\"1.0\"?><Invoice><ID>FA-001</ID></Invoice>";
        let files = vec![FluxFile {
            filename: "Base_FA001.xml".to_string(),
            content: xml_content.to_vec(),
        }];

        let tar_gz = build_tar_gz(&files).unwrap();
        let decoder = flate2::read::GzDecoder::new(tar_gz.as_slice());
        let mut archive = tar::Archive::new(decoder);

        for entry in archive.entries().unwrap() {
            let mut entry = entry.unwrap();
            let path = entry.path().unwrap().to_string_lossy().to_string();
            assert_eq!(path, "Base_FA001.xml");

            let mut content = Vec::new();
            entry.read_to_end(&mut content).unwrap();
            assert_eq!(content, xml_content);
        }
    }

    #[test]
    fn test_build_f1_flux() {
        let config = PpfFluxConfig::new("AAA123").unwrap();
        let seq = recommended_sequence(CodeInterface::F1Ubl, 42);

        let (name, tar_gz) = build_f1_flux(
            &config,
            CodeInterface::F1Ubl,
            &seq,
            &[(ProfilF1::Base, "facture_001", b"<Invoice/>")],
        )
        .unwrap();

        assert!(name.starts_with("FFE0111A_AAA123_"));
        assert!(name.ends_with(".tar.gz"));
        assert!(!tar_gz.is_empty());
    }

    #[test]
    fn test_build_f1_flux_wrong_code_interface() {
        let config = PpfFluxConfig::new("AAA123").unwrap();
        let seq = recommended_sequence(CodeInterface::F6Facture, 1);

        let result = build_f1_flux(
            &config,
            CodeInterface::F6Facture,
            &seq,
            &[(ProfilF1::Base, "facture_001", b"<Invoice/>")],
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_build_f6_flux() {
        let config = PpfFluxConfig::new("AAA123").unwrap();
        let seq = recommended_sequence(CodeInterface::F6StatutsObligatoires, 1);

        let (name, tar_gz) = build_f6_flux(
            &config,
            CodeInterface::F6StatutsObligatoires,
            &seq,
            &[("cdar_001.xml", b"<CDAR/>")],
        )
        .unwrap();

        assert!(name.starts_with("FFE0654A_AAA123_"));
        assert!(name.ends_with(".tar.gz"));
        assert!(!tar_gz.is_empty());
    }

    #[test]
    fn test_build_f10_flux() {
        let config = PpfFluxConfig::new("AAA123").unwrap();
        let seq = recommended_sequence(CodeInterface::F10TransactionPaiement, 1);

        let (name, tar_gz) = build_f10_flux(
            &config,
            &seq,
            &[("report_001.xml", b"<Report/>")],
        )
        .unwrap();

        assert!(name.starts_with("FFE1025A_AAA123_"));
        assert!(name.ends_with(".tar.gz"));
        assert!(!tar_gz.is_empty());
    }

    #[test]
    fn test_sha256_hex() {
        let hash = sha256_hex(b"hello world");
        assert_eq!(
            hash,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }
}
