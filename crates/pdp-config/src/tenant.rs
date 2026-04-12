//! # Tenant Discovery & Loading
//!
//! Découverte et chargement des tenants depuis le répertoire `tenants/{siren}/`.
//!
//! # Convention de répertoires
//!
//! ```text
//! tenants/
//!   456789012/              # SIREN du client
//!     in/                   # Le client dépose ses factures ici (émission)
//!     out/                  # La PDP dépose les factures reçues + CDV ici
//!     config.yaml           # Configuration spécifique (optionnel)
//!     sequence.txt          # Compteur séquence PPF (auto-généré)
//!     certs/                # Certificats (optionnel)
//! ```
//!
//! Si `config.yaml` est absent, un tenant minimal est créé automatiquement
//! avec les répertoires `in/` et `out/` comme source et destination.

use std::path::{Path, PathBuf};
use crate::model::{PdpIdentity, TenantConfig};

/// Entrée tenant chargée depuis le disque
#[derive(Debug, Clone)]
pub struct TenantEntry {
    /// SIREN du tenant (9 chiffres)
    pub siren: String,
    /// Répertoire racine du tenant
    pub tenant_dir: PathBuf,
    /// Configuration chargée ou auto-générée
    pub config: TenantConfig,
    /// Séquence PPF initiale (chargée depuis sequence.txt)
    pub initial_sequence: u64,
}

impl TenantEntry {
    /// Répertoire d'entrée : le client y dépose ses factures
    pub fn in_dir(&self) -> PathBuf {
        self.tenant_dir.join("in")
    }

    /// Répertoire de sortie : la PDP y dépose les factures reçues et CDV
    pub fn out_dir(&self) -> PathBuf {
        self.tenant_dir.join("out")
    }

    /// Crée les répertoires in/ et out/ s'ils n'existent pas
    pub fn ensure_dirs(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(self.in_dir())?;
        std::fs::create_dir_all(self.out_dir())?;
        Ok(())
    }
}

/// Vérifie qu'un nom de répertoire est un SIREN valide (9 chiffres)
pub fn is_valid_siren(name: &str) -> bool {
    name.len() == 9 && name.chars().all(|c| c.is_ascii_digit())
}

/// Charge le numéro de séquence depuis `sequence.txt` dans le répertoire tenant
pub fn load_sequence(tenant_dir: &Path) -> u64 {
    let path = tenant_dir.join("sequence.txt");
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(0)
}

/// Persiste le numéro de séquence dans `sequence.txt`
pub fn save_sequence(tenant_dir: &Path, seq: u64) -> std::io::Result<()> {
    let path = tenant_dir.join("sequence.txt");
    std::fs::write(&path, seq.to_string())
}

/// Charge la configuration d'un tenant depuis son répertoire.
/// Si `config.yaml` est absent, retourne None (le tenant sera auto-configuré).
pub fn load_tenant_config(tenant_dir: &Path) -> Result<Option<TenantConfig>, String> {
    let config_path = tenant_dir.join("config.yaml");
    if !config_path.exists() {
        return Ok(None);
    }
    let content = std::fs::read_to_string(&config_path)
        .map_err(|e| format!("Lecture {}: {}", config_path.display(), e))?;
    let config: TenantConfig = serde_yaml::from_str(&content)
        .map_err(|e| format!("Parse YAML {}: {}", config_path.display(), e))?;
    Ok(Some(config))
}

/// Crée une configuration minimale auto-générée pour un tenant sans config.yaml.
/// Les répertoires `in/` et `out/` sont utilisés comme source et destination.
fn auto_config(siren: &str) -> TenantConfig {
    TenantConfig {
        pdp: PdpIdentity {
            id: format!("PDP-{}", siren),
            name: format!("Tenant {}", siren),
            siren: Some(siren.to_string()),
            siret: None,
            matricule: None,
        },
        routes: Vec::new(), // Routes auto-générées par le runtime
        ppf: None,
        afnor: None,
    }
}

/// Découvre et charge tous les tenants depuis le répertoire parent.
///
/// Chaque sous-répertoire dont le nom est un SIREN valide (9 chiffres)
/// est chargé comme un tenant. Si `config.yaml` est absent, une config
/// minimale est auto-générée.
///
/// Les répertoires `in/` et `out/` sont créés automatiquement.
pub fn discover_tenants(tenants_dir: &Path) -> Result<Vec<TenantEntry>, String> {
    if !tenants_dir.exists() {
        return Err(format!("Répertoire tenants introuvable: {}", tenants_dir.display()));
    }

    let mut entries = Vec::new();
    let read_dir = std::fs::read_dir(tenants_dir)
        .map_err(|e| format!("Lecture répertoire {}: {}", tenants_dir.display(), e))?;

    for entry in read_dir {
        let entry = entry.map_err(|e| format!("Lecture entrée: {}", e))?;
        let path = entry.path();

        if !path.is_dir() {
            continue;
        }

        let dir_name = match path.file_name().and_then(|n| n.to_str()) {
            Some(name) => name.to_string(),
            None => continue,
        };

        if !is_valid_siren(&dir_name) {
            tracing::warn!(
                dir = %dir_name,
                "Répertoire ignoré dans tenants/ : nom n'est pas un SIREN valide (9 chiffres)"
            );
            continue;
        }

        let config = match load_tenant_config(&path) {
            Ok(Some(cfg)) => {
                tracing::info!(
                    siren = %dir_name,
                    pdp_name = %cfg.pdp.name,
                    "Tenant chargé depuis config.yaml"
                );
                cfg
            }
            Ok(None) => {
                tracing::info!(
                    siren = %dir_name,
                    "Tenant auto-configuré (pas de config.yaml)"
                );
                auto_config(&dir_name)
            }
            Err(e) => {
                tracing::error!(siren = %dir_name, error = %e, "Erreur chargement tenant");
                continue;
            }
        };

        let initial_sequence = load_sequence(&path);

        let tenant = TenantEntry {
            siren: dir_name,
            tenant_dir: path,
            config,
            initial_sequence,
        };

        // Créer in/ et out/ s'ils n'existent pas
        if let Err(e) = tenant.ensure_dirs() {
            tracing::error!(
                siren = %tenant.siren,
                error = %e,
                "Impossible de créer les répertoires in/ et out/"
            );
            continue;
        }

        entries.push(tenant);
    }

    entries.sort_by(|a, b| a.siren.cmp(&b.siren));
    Ok(entries)
}

/// Crée un TenantEntry synthétique à partir de la config racine (mode mono-tenant)
pub fn synthetic_tenant(config: &crate::model::PdpConfig) -> TenantEntry {
    let siren = config.pdp.siren.clone().unwrap_or_else(|| "000000000".to_string());
    TenantEntry {
        siren,
        tenant_dir: PathBuf::from("."),
        config: TenantConfig {
            pdp: config.pdp.clone(),
            routes: config.routes.clone(),
            ppf: config.ppf.clone(),
            afnor: config.afnor.clone(),
        },
        initial_sequence: config.ppf.as_ref()
            .and_then(|p| p.initial_sequence)
            .unwrap_or(0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_is_valid_siren() {
        assert!(is_valid_siren("123456789"));
        assert!(is_valid_siren("000000000"));
        assert!(!is_valid_siren("12345678"));  // too short
        assert!(!is_valid_siren("1234567890")); // too long
        assert!(!is_valid_siren("12345678a")); // non-digit
        assert!(!is_valid_siren(""));
    }

    #[test]
    fn test_load_sequence_missing_file() {
        let dir = TempDir::new().unwrap();
        assert_eq!(load_sequence(dir.path()), 0);
    }

    #[test]
    fn test_load_save_sequence() {
        let dir = TempDir::new().unwrap();
        save_sequence(dir.path(), 42).unwrap();
        assert_eq!(load_sequence(dir.path()), 42);
        save_sequence(dir.path(), 999).unwrap();
        assert_eq!(load_sequence(dir.path()), 999);
    }

    #[test]
    fn test_in_out_dirs() {
        let dir = TempDir::new().unwrap();
        let entry = TenantEntry {
            siren: "123456789".to_string(),
            tenant_dir: dir.path().to_path_buf(),
            config: auto_config("123456789"),
            initial_sequence: 0,
        };
        assert_eq!(entry.in_dir(), dir.path().join("in"));
        assert_eq!(entry.out_dir(), dir.path().join("out"));
    }

    #[test]
    fn test_ensure_dirs_creates_in_out() {
        let dir = TempDir::new().unwrap();
        let entry = TenantEntry {
            siren: "123456789".to_string(),
            tenant_dir: dir.path().to_path_buf(),
            config: auto_config("123456789"),
            initial_sequence: 0,
        };

        assert!(!entry.in_dir().exists());
        assert!(!entry.out_dir().exists());

        entry.ensure_dirs().unwrap();

        assert!(entry.in_dir().is_dir());
        assert!(entry.out_dir().is_dir());
    }

    #[test]
    fn test_discover_tenants_empty_dir() {
        let dir = TempDir::new().unwrap();
        let result = discover_tenants(dir.path()).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_discover_tenants_skips_invalid_names() {
        let dir = TempDir::new().unwrap();
        fs::create_dir(dir.path().join("not-a-siren")).unwrap();
        fs::create_dir(dir.path().join("12345")).unwrap();
        let result = discover_tenants(dir.path()).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_discover_tenants_with_config() {
        let dir = TempDir::new().unwrap();
        let tenant_dir = dir.path().join("123456789");
        fs::create_dir(&tenant_dir).unwrap();

        let config_yaml = r#"
pdp:
  id: "PDP-TEST"
  name: "Test Tenant"
  siren: "123456789"
  siret: "12345678901234"
  matricule: "0001"
routes: []
"#;
        fs::write(tenant_dir.join("config.yaml"), config_yaml).unwrap();
        fs::write(tenant_dir.join("sequence.txt"), "42").unwrap();

        let result = discover_tenants(dir.path()).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].siren, "123456789");
        assert_eq!(result[0].config.pdp.name, "Test Tenant");
        assert_eq!(result[0].initial_sequence, 42);
        // in/ et out/ créés automatiquement
        assert!(result[0].in_dir().is_dir());
        assert!(result[0].out_dir().is_dir());
    }

    #[test]
    fn test_discover_tenants_auto_config_without_yaml() {
        let dir = TempDir::new().unwrap();
        let tenant_dir = dir.path().join("987654321");
        fs::create_dir(&tenant_dir).unwrap();
        // Pas de config.yaml → auto-config

        let result = discover_tenants(dir.path()).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].siren, "987654321");
        assert_eq!(result[0].config.pdp.name, "Tenant 987654321");
        assert_eq!(result[0].config.pdp.siren.as_deref(), Some("987654321"));
        assert!(result[0].config.routes.is_empty());
        // in/ et out/ créés
        assert!(result[0].in_dir().is_dir());
        assert!(result[0].out_dir().is_dir());
    }

    #[test]
    fn test_discover_tenants_sorted() {
        let dir = TempDir::new().unwrap();
        for siren in &["999999999", "111111111", "555555555"] {
            let tenant_dir = dir.path().join(siren);
            fs::create_dir(&tenant_dir).unwrap();
            // Pas de config.yaml → auto-config pour les 3
        }
        let result = discover_tenants(dir.path()).unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].siren, "111111111");
        assert_eq!(result[1].siren, "555555555");
        assert_eq!(result[2].siren, "999999999");
    }

    #[test]
    fn test_discover_tenants_mixed_config_and_auto() {
        let dir = TempDir::new().unwrap();

        // Tenant 1 : avec config.yaml
        let t1 = dir.path().join("111111111");
        fs::create_dir(&t1).unwrap();
        fs::write(t1.join("config.yaml"), r#"
pdp:
  id: "PDP-A"
  name: "Entreprise A"
  siren: "111111111"
  siret: "11111111101234"
  matricule: "0001"
routes: []
"#).unwrap();

        // Tenant 2 : sans config.yaml (auto)
        let t2 = dir.path().join("222222222");
        fs::create_dir(&t2).unwrap();

        let result = discover_tenants(dir.path()).unwrap();
        assert_eq!(result.len(), 2);

        // Tenant 1 : config explicite
        assert_eq!(result[0].config.pdp.name, "Entreprise A");
        assert_eq!(result[0].config.pdp.matricule.as_deref(), Some("0001"));

        // Tenant 2 : auto-config
        assert_eq!(result[1].config.pdp.name, "Tenant 222222222");
        assert!(result[1].config.pdp.matricule.is_none());
    }
}
