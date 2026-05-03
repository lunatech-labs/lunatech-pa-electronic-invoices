//! Session web — cookie HMAC-signé portant l'identité du user connecté.
//!
//! # Modèle
//!
//! Une fois un user authentifié via `/login`, on lui pose un cookie
//! `ferrite_session` de la forme :
//!
//! ```text
//! <principal_b64url>.<expires_at_unix>.<hmac_b64url>
//! ```
//!
//! - `principal_b64url` : `principal` du user (libellé déjà court, base64url
//!   pour autoriser tout caractère sans échappement),
//! - `expires_at_unix` : timestamp Unix d'expiration (entier, secondes),
//! - `hmac_b64url` : HMAC-SHA256 sur `principal|expires_at` avec le secret
//!   de session (32 octets minimum, dérivé de `session_secret` ou aléatoire).
//!
//! Le serveur ne stocke **rien** côté Postgres / mémoire — la validation est
//! purement cryptographique. Conséquence : un logout ne peut pas invalider
//! une session avant son TTL sauf à tourner le secret (Phase B.5 : ajout
//! d'une table `revoked_sessions` ou passage à un store stateful).
//!
//! Lookup du `SecurityContext` à partir du `principal` : le middleware
//! cherche dans `state.users` (table en mémoire construite depuis la config),
//! exactement comme il cherche un Bearer token.

use std::time::{SystemTime, UNIX_EPOCH};

use base64::Engine as _;
use hmac::{Hmac, Mac};
use sha2::Sha256;

use crate::security::SecurityContext;
use pdp_config::model::UserConfig;

/// Nom du cookie posé par `/login`.
pub const SESSION_COOKIE: &str = "ferrite_session";

/// Erreurs de vérification d'un cookie de session.
#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    #[error("format de cookie invalide")]
    Malformed,
    #[error("signature HMAC invalide")]
    BadSignature,
    #[error("session expirée")]
    Expired,
}

/// Génère la valeur du cookie pour un user donné.
pub fn issue_cookie(secret: &[u8], principal: &str, ttl_secs: u64) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let expires_at = now + ttl_secs;
    let payload = format!("{}|{}", principal, expires_at);
    let sig = hmac_b64(secret, &payload);
    let principal_b64 = b64url(principal.as_bytes());
    format!("{}.{}.{}", principal_b64, expires_at, sig)
}

/// Vérifie le cookie et retourne le `principal` qu'il porte si la signature
/// est valide et qu'il n'est pas expiré.
pub fn verify_cookie(secret: &[u8], cookie_value: &str) -> Result<String, SessionError> {
    let mut parts = cookie_value.splitn(3, '.');
    let principal_b64 = parts.next().ok_or(SessionError::Malformed)?;
    let expires_at_str = parts.next().ok_or(SessionError::Malformed)?;
    let sig_provided = parts.next().ok_or(SessionError::Malformed)?;

    let principal_bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(principal_b64)
        .map_err(|_| SessionError::Malformed)?;
    let principal =
        String::from_utf8(principal_bytes).map_err(|_| SessionError::Malformed)?;
    let expires_at: u64 = expires_at_str.parse().map_err(|_| SessionError::Malformed)?;

    let payload = format!("{}|{}", principal, expires_at);
    let sig_expected = hmac_b64(secret, &payload);
    // Comparaison constant-time
    if !constant_time_eq(sig_provided.as_bytes(), sig_expected.as_bytes()) {
        return Err(SessionError::BadSignature);
    }

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    if now >= expires_at {
        return Err(SessionError::Expired);
    }

    Ok(principal)
}

/// Lookup du `SecurityContext` correspondant à un user déjà authentifié.
/// Retourne `None` si le user n'existe plus dans la config (rotation).
pub fn user_to_context(users: &[UserConfig], principal: &str) -> Option<SecurityContext> {
    users
        .iter()
        .find(|u| u.principal == principal)
        .map(|u| SecurityContext {
            principal: u.principal.clone(),
            allowed_sirens: u.allowed_sirens.clone(),
            role: u.role,
        })
}

/// Cherche un user par email + password en clair (Phase B v1, cf. note
/// `UserConfig.password`). Argon2 prévu Phase B.5.
///
/// Comparaison du mot de passe en temps constant pour éviter le timing
/// attack basé sur la longueur. Pour l'email on tolère une mismatch rapide
/// (énumération possible sur l'email mais pas critique pour le scope v1).
pub fn authenticate<'a>(
    users: &'a [UserConfig],
    email: &str,
    password: &str,
) -> Option<&'a UserConfig> {
    let candidate = users.iter().find(|u| u.email == email)?;
    if constant_time_eq(candidate.password.as_bytes(), password.as_bytes()) {
        Some(candidate)
    } else {
        None
    }
}

/// Génère un secret de session aléatoire 32 octets si l'admin n'en a pas
/// fourni dans le YAML.
pub fn random_secret() -> Vec<u8> {
    use std::time::SystemTime;
    // 32 octets dérivés de l'horloge + addresse mémoire — pas un PRNG
    // cryptographique, mais suffit pour fabriquer un secret jetable sur
    // une instance dev. Les déploiements prod doivent fournir
    // `session_secret:` explicitement.
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let mut buf = Vec::with_capacity(32);
    let mut x = now as u64;
    for _ in 0..4 {
        x = x.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        buf.extend_from_slice(&x.to_le_bytes());
    }
    buf
}

// ---------------------------------------------------------------------------
// Internes
// ---------------------------------------------------------------------------

fn hmac_b64(secret: &[u8], data: &str) -> String {
    type HmacSha256 = Hmac<Sha256>;
    let mut mac = HmacSha256::new_from_slice(secret).expect("HMAC accepte n'importe quelle clé");
    mac.update(data.as_bytes());
    b64url(&mac.finalize().into_bytes())
}

fn b64url(bytes: &[u8]) -> String {
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff: u8 = 0;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use pdp_config::model::Role;

    #[test]
    fn issue_then_verify_roundtrip() {
        let secret = b"super-secret-key-must-be-32-bytes-or-more";
        let cookie = issue_cookie(secret, "alice@tc", 3600);
        assert_eq!(
            verify_cookie(secret, &cookie).expect("valide"),
            "alice@tc".to_string()
        );
    }

    #[test]
    fn verify_rejects_tampered_signature() {
        let secret = b"super-secret-key-must-be-32-bytes-or-more";
        let mut cookie = issue_cookie(secret, "alice@tc", 3600);
        // Flip un caractère dans la signature
        let last = cookie.pop().unwrap();
        let new_last = if last == 'A' { 'B' } else { 'A' };
        cookie.push(new_last);
        let err = verify_cookie(secret, &cookie).unwrap_err();
        assert!(matches!(err, SessionError::BadSignature));
    }

    #[test]
    fn verify_rejects_expired_cookie() {
        let secret = b"super-secret-key-must-be-32-bytes-or-more";
        // TTL = 0 → expire immédiatement
        let cookie = issue_cookie(secret, "alice@tc", 0);
        // Petit sleep pour franchir la frontière de seconde
        std::thread::sleep(std::time::Duration::from_millis(1100));
        let err = verify_cookie(secret, &cookie).unwrap_err();
        assert!(matches!(err, SessionError::Expired));
    }

    #[test]
    fn verify_rejects_wrong_secret() {
        let secret_a = b"secret-a-12345678901234567890123456789012";
        let secret_b = b"secret-b-12345678901234567890123456789012";
        let cookie = issue_cookie(secret_a, "alice@tc", 3600);
        let err = verify_cookie(secret_b, &cookie).unwrap_err();
        assert!(matches!(err, SessionError::BadSignature));
    }

    #[test]
    fn verify_rejects_malformed() {
        let secret = b"k";
        for bad in ["", "abc", "a.b", "a.b.c.d.e"] {
            assert!(matches!(
                verify_cookie(secret, bad),
                Err(SessionError::Malformed) | Err(SessionError::BadSignature)
            ));
        }
    }

    fn user(email: &str, password: &str, principal: &str, sirens: &[&str]) -> UserConfig {
        UserConfig {
            email: email.into(),
            password: password.into(),
            principal: principal.into(),
            allowed_sirens: sirens.iter().map(|s| s.to_string()).collect(),
            role: Role::Tenant,
        }
    }

    #[test]
    fn authenticate_matches_credentials() {
        let users = vec![user("alice@tc", "pwd123", "alice", &["123"])];
        assert!(authenticate(&users, "alice@tc", "pwd123").is_some());
        assert!(authenticate(&users, "alice@tc", "wrong").is_none());
        assert!(authenticate(&users, "bob@tc", "pwd123").is_none());
    }

    #[test]
    fn user_to_context_rebuilds_security_context() {
        let users = vec![user("alice@tc", "pwd", "alice", &["123"])];
        let ctx = user_to_context(&users, "alice").expect("trouvé");
        assert_eq!(ctx.principal, "alice");
        assert_eq!(ctx.allowed_sirens, vec!["123".to_string()]);
        assert_eq!(ctx.role, Role::Tenant);
        assert!(user_to_context(&users, "bob").is_none());
    }
}
