//! Utilitaires de vérification des clés serveur SSH
//!
//! Fournit la lecture d'un fichier known_hosts et la comparaison des clés
//! publiques serveur lors de la connexion SFTP.
//!
//! # Format du fichier known_hosts
//!
//! Format simplifié PDP (une entrée par ligne) :
//! ```text
//! # Commentaire
//! hostname ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAA...
//! hostname ssh-rsa AAAAB3NzaC1yc2EAAAA...
//! ```
//!
//! La clé est au format OpenSSH standard (type + base64).

use std::fs;

/// Convertit une clé publique SSH en chaîne déterministe pour comparaison.
///
/// Utilise le format Debug de la clé publique pour garantir une représentation
/// stable indépendante des traits implémentés par la version de russh_keys.
/// L'administrateur doit stocker la même représentation dans le fichier known_hosts
/// (obtenue via la commande `pdp-sftp --print-server-key <host>`).
pub fn key_to_string(key: &russh_keys::key::PublicKey) -> String {
    format!("{:?}", key)
}

/// Recherche la clé publique d'un hôte dans un fichier known_hosts PDP.
///
/// Format attendu (une entrée par ligne) :
/// ```text
/// # Commentaire
/// hostname clé_publique
/// ```
///
/// La `clé_publique` est tout ce qui suit le premier espace après le hostname.
/// Les lignes commençant par `#` et les lignes vides sont ignorées.
/// Retourne `Some(clé)` si l'hôte est trouvé, `None` sinon.
pub fn lookup_host_key(known_hosts_path: &str, host: &str) -> Option<String> {
    let content = match fs::read_to_string(known_hosts_path) {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!(
                path = %known_hosts_path,
                error = %e,
                "Impossible de lire le fichier known_hosts"
            );
            return None;
        }
    };

    for line in content.lines() {
        let line = line.trim();
        // Ignorer les commentaires et les lignes vides
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Parser la ligne : "hostname clé_publique"
        let parts: Vec<&str> = line.splitn(2, ' ').collect();
        if parts.len() == 2 && parts[0] == host {
            return Some(parts[1].to_string());
        }
    }

    tracing::warn!(
        host = %host,
        path = %known_hosts_path,
        "Hôte non trouvé dans le fichier known_hosts"
    );
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_lookup_host_key_found() {
        let dir = std::env::temp_dir().join("pdp_sftp_test_kh1");
        let _ = fs::create_dir_all(&dir);
        let path = dir.join("known_hosts_test");
        let mut f = fs::File::create(&path).unwrap();
        writeln!(f, "# commentaire").unwrap();
        writeln!(f, "sftp.ppf.finances.gouv.fr ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIExample").unwrap();
        writeln!(f, "other.host.fr ssh-rsa AAAAB3NzaExample").unwrap();

        let result = lookup_host_key(path.to_str().unwrap(), "sftp.ppf.finances.gouv.fr");
        assert_eq!(result, Some("ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIExample".to_string()));

        let result2 = lookup_host_key(path.to_str().unwrap(), "other.host.fr");
        assert_eq!(result2, Some("ssh-rsa AAAAB3NzaExample".to_string()));

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_lookup_host_key_not_found() {
        let dir = std::env::temp_dir().join("pdp_sftp_test_kh2");
        let _ = fs::create_dir_all(&dir);
        let path = dir.join("known_hosts_test2");
        let mut f = fs::File::create(&path).unwrap();
        writeln!(f, "sftp.ppf.finances.gouv.fr ssh-ed25519 AAAA").unwrap();

        let result = lookup_host_key(path.to_str().unwrap(), "unknown.host.fr");
        assert_eq!(result, None);

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_lookup_host_key_file_missing() {
        let result = lookup_host_key("/tmp/nonexistent_known_hosts_file_pdp", "any.host");
        assert_eq!(result, None);
    }

    #[test]
    fn test_lookup_host_key_skips_comments_and_blank() {
        let dir = std::env::temp_dir().join("pdp_sftp_test_kh3");
        let _ = fs::create_dir_all(&dir);
        let path = dir.join("known_hosts_test3");
        let mut f = fs::File::create(&path).unwrap();
        writeln!(f, "# Clés serveur PPF").unwrap();
        writeln!(f, "").unwrap();
        writeln!(f, "  ").unwrap();
        writeln!(f, "ppf.host ssh-ed25519 AAAAC3NzaFingerprint123").unwrap();

        let result = lookup_host_key(path.to_str().unwrap(), "ppf.host");
        assert_eq!(result, Some("ssh-ed25519 AAAAC3NzaFingerprint123".to_string()));

        let _ = fs::remove_file(&path);
    }
}
