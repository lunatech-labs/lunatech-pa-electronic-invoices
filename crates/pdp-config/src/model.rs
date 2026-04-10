use serde::{Deserialize, Serialize};

/// Configuration globale de la PDP
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdpConfig {
    pub pdp: PdpIdentity,
    pub elasticsearch: ElasticsearchConfig,
    pub routes: Vec<RouteConfig>,
    #[serde(default)]
    pub validation: ValidationConfig,
    #[serde(default)]
    pub polling: PollingConfig,
    #[serde(default)]
    pub logging: LoggingConfig,
    /// Configuration de connexion au PPF (optionnelle en dev)
    #[serde(default)]
    pub ppf: Option<PpfConfig>,
    /// Configuration AFNOR Flow Service PDP↔PDP (optionnelle)
    #[serde(default)]
    pub afnor: Option<AfnorConfig>,
    /// Configuration du serveur HTTP API (optionnelle — si absent, pas de serveur HTTP)
    #[serde(default)]
    pub http_server: Option<HttpServerConfig>,
}

/// Configuration du serveur HTTP API AFNOR
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpServerConfig {
    /// Adresse d'écoute (défaut: "0.0.0.0")
    #[serde(default = "default_http_host")]
    pub host: String,
    /// Port d'écoute (défaut: 8080)
    #[serde(default = "default_http_port")]
    pub port: u16,
    /// Secret HMAC pour la vérification des signatures webhook
    #[serde(default)]
    pub webhook_secret: Option<String>,
    /// Tokens Bearer autorisés pour l'authentification API
    /// Si absent ou vide, l'authentification est désactivée (mode développement)
    #[serde(default)]
    pub bearer_tokens: Option<Vec<String>>,
}

fn default_http_host() -> String {
    "0.0.0.0".to_string()
}

fn default_http_port() -> u16 {
    8080
}

/// Identité de la PDP
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdpIdentity {
    /// Identifiant PDP (ex: "PDP-001")
    pub id: String,
    /// Nom de la PDP
    pub name: String,
    /// SIRET de la PDP
    pub siret: Option<String>,
    /// SIREN de la PDP
    pub siren: Option<String>,
    /// Matricule PDP attribué par la DGFiP (schemeID 0238)
    /// Ex: "1111" pour la PDP UNO dans les exemples v3.0
    pub matricule: Option<String>,
}

/// Configuration Elasticsearch (traçabilité + archivage)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElasticsearchConfig {
    #[serde(default = "default_es_url")]
    pub url: String,
}

fn default_es_url() -> String {
    std::env::var("ELASTICSEARCH_URL")
        .unwrap_or_else(|_| "http://localhost:9200".to_string())
}

impl Default for ElasticsearchConfig {
    fn default() -> Self {
        Self {
            url: default_es_url(),
        }
    }
}

/// Configuration d'une route
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteConfig {
    pub id: String,
    pub description: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    pub source: EndpointConfig,
    pub destination: EndpointConfig,
    #[serde(default)]
    pub error_destination: Option<EndpointConfig>,
    #[serde(default)]
    pub transform_to: Option<String>,
    #[serde(default = "default_true")]
    pub validate: bool,
    #[serde(default = "default_true")]
    pub generate_cdar: bool,
    #[serde(default)]
    pub cdar_receiver: Option<CdarReceiverConfig>,
}

fn default_true() -> bool {
    true
}

/// Configuration d'un endpoint (source ou destination)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointConfig {
    #[serde(rename = "type")]
    pub endpoint_type: String,
    pub path: String,
    #[serde(default)]
    pub host: Option<String>,
    #[serde(default)]
    pub port: Option<u16>,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub password: Option<String>,
    #[serde(default)]
    pub private_key_path: Option<String>,
    #[serde(default)]
    pub file_pattern: Option<String>,
    #[serde(default)]
    pub archive_path: Option<String>,
    #[serde(default)]
    pub delete_after_read: Option<bool>,
    /// Chemin vers le fichier known_hosts pour vérification des clés serveur SSH (SFTP)
    #[serde(default)]
    pub known_hosts_path: Option<String>,
}

/// Configuration du destinataire CDAR
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CdarReceiverConfig {
    pub pdp_id: String,
    pub pdp_name: String,
}

/// Configuration de la validation XSD/Schematron
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationConfig {
    #[serde(default = "default_specs_dir")]
    pub specs_dir: String,
    #[serde(default = "default_true")]
    pub xsd_enabled: bool,
    #[serde(default = "default_true")]
    pub en16931_enabled: bool,
    #[serde(default = "default_true")]
    pub br_fr_enabled: bool,
}

fn default_specs_dir() -> String {
    std::env::var("PDP_SPECS_DIR").unwrap_or_else(|_| "./specs".to_string())
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            specs_dir: default_specs_dir(),
            xsd_enabled: true,
            en16931_enabled: true,
            br_fr_enabled: true,
        }
    }
}

/// Configuration du polling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PollingConfig {
    #[serde(default = "default_interval")]
    pub interval_secs: u64,
}

fn default_interval() -> u64 {
    60
}

impl Default for PollingConfig {
    fn default() -> Self {
        Self {
            interval_secs: default_interval(),
        }
    }
}

/// Configuration du logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    #[serde(default = "default_log_format")]
    pub format: String,
    #[serde(default = "default_log_level")]
    pub level: String,
}

fn default_log_format() -> String {
    "text".to_string()
}

fn default_log_level() -> String {
    "info".to_string()
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            format: default_log_format(),
            level: default_log_level(),
        }
    }
}

/// Configuration de connexion au PPF
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PpfConfig {
    /// Environnement PPF : dev, int, rec, preprod, prod
    #[serde(default = "default_ppf_env")]
    pub environment: String,
    /// Code interface pour le Système d'Échange (pattern ^[A-Z]{3}[0-9]{4}[A-Z]{1}$)
    pub code_interface: String,
    /// Code application PISTE
    pub code_application_piste: String,
    /// Répertoire de sortie pour les fichiers Flux 1 (données réglementaires)
    /// Ces fichiers seront ensuite archivés en tar.gz et envoyés via SFTP au PPF
    #[serde(default = "default_flux1_output_dir")]
    pub flux1_output_dir: String,
    /// Stratégie de profil Flux 1 : "auto" (défaut), "base", ou "full"
    /// - auto : lignes présentes → Full, sinon Base
    /// - base : toujours Base (sans lignes)
    /// - full : toujours Full (fallback Base si pas de lignes dans la source)
    #[serde(default = "default_flux1_profile")]
    pub flux1_profile: String,
    /// Authentification PISTE (OAuth2 client_credentials)
    pub auth: PisteAuthConfigYaml,
    /// Configuration SFTP pour le dépôt des flux vers le PPF
    /// Si absent, les flux sont écrits localement dans flux1_output_dir
    #[serde(default)]
    pub sftp: Option<PpfSftpConfigYaml>,
    /// Séquence initiale pour le nommage des flux (compteur atomique)
    #[serde(default)]
    pub initial_sequence: Option<u64>,
}

/// Configuration SFTP du PPF (Système d'Échange)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PpfSftpConfigYaml {
    /// Nom d'hôte du serveur SFTP PPF
    pub host: String,
    /// Port SFTP (défaut: 22)
    #[serde(default = "default_sftp_port")]
    pub port: u16,
    /// Nom d'utilisateur SFTP
    pub username: String,
    /// Chemin vers la clé privée RSA X509v3
    pub private_key_path: String,
    /// Répertoire distant de dépôt
    #[serde(default = "default_sftp_remote_path")]
    pub remote_path: String,
    /// Chemin vers le fichier known_hosts (optionnel)
    #[serde(default)]
    pub known_hosts_path: Option<String>,
    /// Chemin vers le fichier de persistance du numéro de séquence (optionnel)
    /// Si absent, le compteur repart de initial_sequence à chaque redémarrage
    #[serde(default)]
    pub sequence_file: Option<String>,
}

fn default_sftp_port() -> u16 {
    22
}

fn default_sftp_remote_path() -> String {
    "/upload".to_string()
}

/// Configuration d'authentification PISTE dans le YAML
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PisteAuthConfigYaml {
    /// URL du token endpoint PISTE
    /// Ex: "https://oauth.piste.gouv.fr/api/oauth/token"
    pub token_url: String,
    /// Client ID (ou variable d'env $PISTE_CLIENT_ID)
    pub client_id: String,
    /// Client Secret (ou variable d'env $PISTE_CLIENT_SECRET)
    pub client_secret: String,
    /// Scopes demandés
    #[serde(default = "default_piste_scope")]
    pub scope: String,
}

fn default_ppf_env() -> String {
    "dev".to_string()
}

fn default_flux1_output_dir() -> String {
    std::env::var("PDP_FLUX1_OUTPUT_DIR").unwrap_or_else(|_| "./output/flux1".to_string())
}

fn default_flux1_profile() -> String {
    std::env::var("PDP_FLUX1_PROFILE").unwrap_or_else(|_| "auto".to_string())
}

fn default_piste_scope() -> String {
    "openid".to_string()
}

/// Configuration AFNOR Flow Service (PDP↔PDP)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AfnorConfig {
    /// URL de base du Flow Service local (notre API exposée)
    /// Ex: "https://api.flow.notre-pdp.fr/flow-service"
    pub flow_service_url: Option<String>,
    /// URL de base du Directory Service
    pub directory_service_url: Option<String>,
    /// Authentification PISTE pour les appels PDP↔PDP
    pub auth: Option<PisteAuthConfigYaml>,
    /// Liste des PDP partenaires connues
    #[serde(default)]
    pub partners: Vec<PdpPartnerConfig>,
}

/// Configuration d'une PDP partenaire
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdpPartnerConfig {
    /// Matricule de la PDP partenaire (schemeID 0238)
    pub matricule: String,
    /// Nom de la PDP partenaire
    pub name: String,
    /// URL du Flow Service de la PDP partenaire
    pub flow_service_url: String,
}
