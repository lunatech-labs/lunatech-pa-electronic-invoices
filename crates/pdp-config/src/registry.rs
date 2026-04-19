//! # TenantRegistry -- Registre multi-tenant
//!
//! Registre central des tenants charges en memoire.
//! Fournit la resolution tenant par SIREN, par token Bearer,
//! ou par les proprietes d'un exchange.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::model::{PdpConfig, TenantConfig};

/// Entree tenant dans le registre (runtime)
#[derive(Debug)]
pub struct TenantRegistryEntry {
    pub siren: String,
    pub tenant_dir: PathBuf,
    pub config: TenantConfig,
    pub initial_sequence: u64,
}

impl TenantRegistryEntry {
    /// Répertoire d'entrée : le client y dépose ses factures
    pub fn in_dir(&self) -> PathBuf {
        self.tenant_dir.join("in")
    }

    /// Répertoire de sortie : la PDP y dépose les factures reçues et CDV
    pub fn out_dir(&self) -> PathBuf {
        self.tenant_dir.join("out")
    }
}

/// Registre central multi-tenant
#[derive(Debug)]
pub struct TenantRegistry {
    /// Tenants indexes par SIREN
    tenants: HashMap<String, TenantRegistryEntry>,
    /// Mapping token Bearer -> SIREN (pour resolution HTTP)
    token_map: HashMap<String, String>,
    /// SIREN par defaut (mode mono-tenant)
    default_siren: Option<String>,
}

impl TenantRegistry {
    /// Charge le registre depuis la configuration.
    /// - Si `tenants_dir` est configure, decouvre les tenants depuis le disque
    /// - Sinon, cree un tenant synthetique a partir de la config racine
    pub fn load(config: &PdpConfig, base_dir: &Path) -> Result<Self, String> {
        let mut tenants = HashMap::new();
        let mut default_siren = None;

        if let Some(ref tenants_dir_name) = config.tenants_dir {
            let tenants_path = base_dir.join(tenants_dir_name);
            let entries = crate::tenant::discover_tenants(&tenants_path)?;
            for entry in entries {
                tenants.insert(
                    entry.siren.clone(),
                    TenantRegistryEntry {
                        siren: entry.siren.clone(),
                        tenant_dir: entry.tenant_dir,
                        config: entry.config,
                        initial_sequence: entry.initial_sequence,
                    },
                );
            }
            if tenants.len() == 1 {
                default_siren = tenants.keys().next().cloned();
            }
            tracing::info!(
                tenant_count = tenants.len(),
                "TenantRegistry charge (multi-tenant)"
            );
        } else {
            // Mode mono-tenant : synthetiser depuis la config racine
            let entry = crate::tenant::synthetic_tenant(config);
            default_siren = Some(entry.siren.clone());
            tenants.insert(
                entry.siren.clone(),
                TenantRegistryEntry {
                    siren: entry.siren.clone(),
                    tenant_dir: entry.tenant_dir,
                    config: entry.config,
                    initial_sequence: entry.initial_sequence,
                },
            );
            tracing::info!(
                siren = ?default_siren,
                "TenantRegistry charge (mono-tenant)"
            );
        }

        let token_map = config.token_tenant_map.clone();

        Ok(Self {
            tenants,
            token_map,
            default_siren,
        })
    }

    /// Retourne un tenant par SIREN
    pub fn get(&self, siren: &str) -> Option<&TenantRegistryEntry> {
        self.tenants.get(siren)
    }

    /// Resout le tenant a partir d'un token Bearer
    pub fn resolve_from_token(&self, token: &str) -> Option<&TenantRegistryEntry> {
        self.token_map
            .get(token)
            .and_then(|siren| self.tenants.get(siren))
    }

    /// Retourne le tenant par defaut (mode mono-tenant)
    pub fn default_tenant(&self) -> Option<&TenantRegistryEntry> {
        self.default_siren
            .as_ref()
            .and_then(|siren| self.tenants.get(siren))
    }

    /// Liste tous les SIRENs enregistres
    pub fn list_sirens(&self) -> Vec<&str> {
        let mut sirens: Vec<&str> = self.tenants.keys().map(|s| s.as_str()).collect();
        sirens.sort();
        sirens
    }

    /// Nombre de tenants
    pub fn len(&self) -> usize {
        self.tenants.len()
    }

    /// Aucun tenant ?
    pub fn is_empty(&self) -> bool {
        self.tenants.is_empty()
    }

    /// Itere sur tous les tenants
    pub fn iter(&self) -> impl Iterator<Item = (&str, &TenantRegistryEntry)> {
        self.tenants.iter().map(|(k, v)| (k.as_str(), v))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::*;
    use std::collections::HashMap;
    use tempfile::TempDir;

    fn minimal_config() -> PdpConfig {
        PdpConfig {
            pdp: PdpIdentity {
                id: "PDP-001".to_string(),
                name: "Test PDP".to_string(),
                siret: Some("12345678901234".to_string()),
                siren: Some("123456789".to_string()),
                matricule: Some("0001".to_string()),
            },
            elasticsearch: ElasticsearchConfig::default(),
            database: None,
            routes: vec![],
            validation: ValidationConfig::default(),
            polling: PollingConfig::default(),
            logging: LoggingConfig::default(),
            ppf: None,
            afnor: None,
            http_server: None,
            tenants_dir: None,
            token_tenant_map: HashMap::new(),
            alerts: None,
        }
    }

    #[test]
    fn test_mono_tenant_from_root_config() {
        let config = minimal_config();
        let registry = TenantRegistry::load(&config, Path::new(".")).unwrap();

        assert_eq!(registry.len(), 1);
        assert!(!registry.is_empty());

        let tenant = registry.get("123456789").unwrap();
        assert_eq!(tenant.siren, "123456789");
        assert_eq!(tenant.config.pdp.name, "Test PDP");
    }

    #[test]
    fn test_default_tenant() {
        let config = minimal_config();
        let registry = TenantRegistry::load(&config, Path::new(".")).unwrap();

        let default = registry.default_tenant().unwrap();
        assert_eq!(default.siren, "123456789");
    }

    #[test]
    fn test_list_sirens() {
        let config = minimal_config();
        let registry = TenantRegistry::load(&config, Path::new(".")).unwrap();

        let sirens = registry.list_sirens();
        assert_eq!(sirens, vec!["123456789"]);
    }

    #[test]
    fn test_resolve_from_token() {
        let mut config = minimal_config();
        config.token_tenant_map.insert(
            "secret-token-abc".to_string(),
            "123456789".to_string(),
        );

        let registry = TenantRegistry::load(&config, Path::new(".")).unwrap();

        let tenant = registry.resolve_from_token("secret-token-abc").unwrap();
        assert_eq!(tenant.siren, "123456789");

        assert!(registry.resolve_from_token("unknown-token").is_none());
    }

    fn write_tenant_config(dir: &Path, siren: &str) {
        let yaml = format!(r#"
pdp:
  id: "PDP-{siren}"
  name: "Tenant {siren}"
  siren: "{siren}"
  siret: "{siren}01234"
  matricule: "0001"
routes: []
"#);
        std::fs::write(dir.join("config.yaml"), yaml).unwrap();
    }

    #[test]
    fn test_multi_tenant_from_directory() {
        let tmp = TempDir::new().unwrap();
        let tenants_dir = tmp.path().join("tenants");
        std::fs::create_dir(&tenants_dir).unwrap();
        let d1 = tenants_dir.join("111111111");
        let d2 = tenants_dir.join("222222222");
        std::fs::create_dir(&d1).unwrap();
        std::fs::create_dir(&d2).unwrap();
        write_tenant_config(&d1, "111111111");
        write_tenant_config(&d2, "222222222");

        let mut config = minimal_config();
        config.tenants_dir = Some("tenants".to_string());

        let registry = TenantRegistry::load(&config, tmp.path()).unwrap();

        assert_eq!(registry.len(), 2);
        // With 2 tenants, no default
        assert!(registry.default_tenant().is_none());

        assert!(registry.get("111111111").is_some());
        assert!(registry.get("222222222").is_some());

        let sirens = registry.list_sirens();
        assert_eq!(sirens, vec!["111111111", "222222222"]);
    }

    #[test]
    fn test_single_multi_tenant_has_default() {
        let tmp = TempDir::new().unwrap();
        let tenants_dir = tmp.path().join("tenants");
        std::fs::create_dir(&tenants_dir).unwrap();
        let d = tenants_dir.join("999888777");
        std::fs::create_dir(&d).unwrap();
        write_tenant_config(&d, "999888777");

        let mut config = minimal_config();
        config.tenants_dir = Some("tenants".to_string());

        let registry = TenantRegistry::load(&config, tmp.path()).unwrap();

        assert_eq!(registry.len(), 1);
        let default = registry.default_tenant().unwrap();
        assert_eq!(default.siren, "999888777");
    }

    #[test]
    fn test_iter() {
        let config = minimal_config();
        let registry = TenantRegistry::load(&config, Path::new(".")).unwrap();

        let entries: Vec<_> = registry.iter().collect();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].0, "123456789");
    }
}
