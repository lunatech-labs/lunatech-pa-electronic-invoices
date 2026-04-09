use serde::{Deserialize, Serialize};

/// Configuration d'une connexion SFTP
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SftpConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    #[serde(default)]
    pub password: Option<String>,
    #[serde(default)]
    pub private_key_path: Option<String>,
    pub remote_path: String,
    /// Pattern de fichiers à récupérer (ex: "*.xml", "*.pdf")
    #[serde(default = "default_pattern")]
    pub file_pattern: String,
    /// Déplacer les fichiers traités dans ce répertoire (sur le serveur SFTP)
    #[serde(default)]
    pub archive_path: Option<String>,
    /// Supprimer les fichiers après traitement
    #[serde(default)]
    pub delete_after_read: bool,
    /// Timeout de connexion en secondes
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
    /// Délai de stabilité en millisecondes : on attend ce délai puis on revérifie
    /// la taille du fichier. Si elle n'a pas changé, le fichier est considéré
    /// comme entièrement écrit et peut être consommé. 0 = pas de vérification.
    #[serde(default = "default_stable_delay")]
    pub stable_delay_ms: u64,
    /// Chemin vers le fichier known_hosts pour vérification des clés serveur SSH.
    /// Si None, la vérification est désactivée (mode développement uniquement).
    /// En production, ce champ DOIT être configuré pour se protéger contre les attaques MITM.
    #[serde(default)]
    pub known_hosts_path: Option<String>,
}

fn default_pattern() -> String {
    "*".to_string()
}

fn default_timeout() -> u64 {
    30
}

fn default_stable_delay() -> u64 {
    1000
}

impl Default for SftpConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 22,
            username: "user".to_string(),
            password: None,
            private_key_path: None,
            remote_path: "/invoices/in".to_string(),
            file_pattern: default_pattern(),
            archive_path: None,
            delete_after_read: false,
            timeout_secs: default_timeout(),
            stable_delay_ms: default_stable_delay(),
            known_hosts_path: None,
        }
    }
}
