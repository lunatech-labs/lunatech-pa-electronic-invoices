use pdp_core::error::{PdpError, PdpResult};
use crate::model::PdpConfig;

/// Charge la configuration depuis un fichier YAML
pub fn load_config(path: &str) -> PdpResult<PdpConfig> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| PdpError::ConfigError(format!("Impossible de lire {}: {}", path, e)))?;

    let config: PdpConfig = serde_yaml::from_str(&content)
        .map_err(|e| PdpError::ConfigError(format!("YAML invalide dans {}: {}", path, e)))?;

    tracing::info!(
        pdp_id = %config.pdp.id,
        pdp_name = %config.pdp.name,
        routes = config.routes.len(),
        "Configuration chargée depuis {}",
        path
    );

    Ok(config)
}

/// Charge la configuration depuis une chaîne YAML (utile pour les tests)
pub fn load_config_from_str(yaml: &str) -> PdpResult<PdpConfig> {
    let config: PdpConfig = serde_yaml::from_str(yaml)
        .map_err(|e| PdpError::ConfigError(format!("YAML invalide: {}", e)))?;
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_config_from_str() {
        let yaml = r#"
pdp:
  id: PDP-TEST-001
  name: PDP de Test
  siret: "12345678901234"

elasticsearch:
  url: "http://localhost:9200"

routes:
  - id: route-ubl-in
    description: "Réception factures UBL"
    enabled: true
    source:
      type: file
      path: /tmp/pdp/in
      file_pattern: "*.xml"
    destination:
      type: file
      path: /tmp/pdp/out
    validate: true
    generate_cdar: true
    cdar_receiver:
      pdp_id: PPF
      pdp_name: Portail Public de Facturation

polling:
  interval_secs: 30

logging:
  format: text
  level: debug
"#;

        let config = load_config_from_str(yaml).expect("Config parsing failed");
        assert_eq!(config.pdp.id, "PDP-TEST-001");
        assert_eq!(config.pdp.name, "PDP de Test");
        assert_eq!(config.routes.len(), 1);
        assert_eq!(config.routes[0].id, "route-ubl-in");
        assert!(config.routes[0].validate);
        assert!(config.routes[0].generate_cdar);
        assert_eq!(config.polling.interval_secs, 30);
    }

    #[test]
    fn test_load_config_invalid_yaml() {
        let result = load_config_from_str("not: [valid: yaml: {{");
        assert!(result.is_err());
    }

    #[test]
    fn test_load_config_file_not_found() {
        let result = load_config("/nonexistent/path/config.yaml");
        assert!(result.is_err());
    }
}
