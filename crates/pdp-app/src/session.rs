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

/// Liste de signatures de cookies révoquées (logout server-side).
///
/// La session étant stateless, on ne peut pas la "supprimer" — mais on
/// peut **mémoriser** sa signature jusqu'à son expiration naturelle, et
/// rejeter toute nouvelle requête qui la présenterait. C'est ce que fait
/// cette liste : un `HashMap<signature_hex, expires_at_unix>` purgé
/// paresseusement à chaque vérification.
///
/// Bornage : `MAX_REVOKED` entrées (5000 par défaut) — au-delà, les
/// entrées les plus anciennes (par expiration) sont évincées. Pour des
/// volumes plus élevés, passer à un store Postgres (Phase B.6).
#[derive(Default)]
pub struct RevocationList {
    entries: std::sync::Mutex<std::collections::HashMap<String, u64>>,
}

const MAX_REVOKED: usize = 5_000;

impl RevocationList {
    pub fn new() -> Self {
        Self::default()
    }

    /// Marque la signature comme révoquée jusqu'à `expires_at`.
    pub fn revoke(&self, signature: &str, expires_at: u64) {
        let mut map = self.entries.lock().unwrap();
        // Purge paresseuse des entrées expirées.
        let now = unix_now();
        map.retain(|_, exp| *exp > now);
        // Bornage : si on dépasse, on jette les plus anciennes.
        if map.len() >= MAX_REVOKED {
            // Heuristique simple : drainer la moitié la plus ancienne.
            let mut by_exp: Vec<(String, u64)> =
                map.iter().map(|(k, v)| (k.clone(), *v)).collect();
            by_exp.sort_by_key(|(_, exp)| *exp);
            for (k, _) in by_exp.into_iter().take(MAX_REVOKED / 2) {
                map.remove(&k);
            }
        }
        map.insert(signature.to_string(), expires_at);
    }

    /// `true` si la signature est dans la liste **et** non expirée.
    pub fn is_revoked(&self, signature: &str) -> bool {
        let now = unix_now();
        let map = self.entries.lock().unwrap();
        map.get(signature).map(|exp| *exp > now).unwrap_or(false)
    }

    /// Taille (utile pour les tests + métriques).
    pub fn len(&self) -> usize {
        self.entries.lock().unwrap().len()
    }
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

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

/// Token de session vérifié — porte le principal et les éléments
/// nécessaires pour révoquer la session (signature + expiration).
#[derive(Debug, Clone)]
pub struct VerifiedSession {
    pub principal: String,
    pub signature: String,
    pub expires_at: u64,
}

/// Vérifie le cookie. Retourne le principal + la signature + l'expiration
/// si tout est OK. La signature peut ensuite être passée à
/// [`RevocationList::revoke`] pour invalider la session côté serveur.
pub fn verify_cookie(secret: &[u8], cookie_value: &str) -> Result<VerifiedSession, SessionError> {
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

    Ok(VerifiedSession {
        principal,
        signature: sig_provided.to_string(),
        expires_at,
    })
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

/// Cherche un user par email + password.
///
/// Le champ `UserConfig.password` peut contenir :
/// - un **hash argon2** (`$argon2id$v=19$...`) — vérification crypto
/// - un mot de passe **en clair** (legacy v1) — comparaison constant-time
///   avec un `tracing::warn!` au démarrage pour pousser à migrer.
///
/// Pour générer un hash : `pdp tools hash-password "monMotDePasse"`.
pub fn authenticate<'a>(
    users: &'a [UserConfig],
    email: &str,
    password: &str,
) -> Option<&'a UserConfig> {
    let candidate = users.iter().find(|u| u.email == email)?;
    if verify_password(&candidate.password, password) {
        Some(candidate)
    } else {
        None
    }
}

/// Vérifie un mot de passe contre la valeur stockée.
///
/// - Si la valeur stockée commence par `$argon2`, on l'interprète comme un
///   hash PHC argon2 et on appelle `argon2::PasswordVerifier`.
/// - Sinon, comparaison constant-time avec le plaintext (mode legacy).
pub fn verify_password(stored: &str, candidate: &str) -> bool {
    if stored.starts_with("$argon2") {
        verify_argon2(stored, candidate).unwrap_or(false)
    } else {
        constant_time_eq(stored.as_bytes(), candidate.as_bytes())
    }
}

/// Hash un mot de passe en argon2id avec un salt aléatoire 16 octets,
/// paramètres par défaut de la lib (memory=19MB, t=2, p=1) — recommandation
/// OWASP 2024 minimale, raisonnable pour un login web.
pub fn hash_password(plaintext: &str) -> Result<String, String> {
    use argon2::password_hash::{PasswordHasher, SaltString};
    use argon2::Argon2;
    let salt = SaltString::generate(&mut argon2::password_hash::rand_core::OsRng);
    let argon = Argon2::default();
    argon
        .hash_password(plaintext.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| format!("argon2 hash error: {e}"))
}

fn verify_argon2(stored: &str, candidate: &str) -> Result<bool, ()> {
    use argon2::password_hash::{PasswordHash, PasswordVerifier};
    use argon2::Argon2;
    let parsed = PasswordHash::new(stored).map_err(|_| ())?;
    Ok(Argon2::default()
        .verify_password(candidate.as_bytes(), &parsed)
        .is_ok())
}

/// Logue un warning pour chaque user dont le password est stocké en clair.
/// À appeler une fois au démarrage du serveur (cf. `main.rs`).
pub fn warn_plaintext_passwords(users: &[UserConfig]) {
    for u in users {
        if !u.password.starts_with("$argon2") {
            tracing::warn!(
                email = %u.email,
                "Mot de passe stocké en clair pour ce user. Utilise \
                 `pdp tools hash-password ...` et remplace par le hash \
                 argon2 dans la config."
            );
        }
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
        let session = verify_cookie(secret, &cookie).expect("valide");
        assert_eq!(session.principal, "alice@tc");
        assert!(!session.signature.is_empty());
        assert!(session.expires_at > 0);
    }

    #[test]
    fn revocation_list_blocks_revoked_signatures() {
        let list = RevocationList::new();
        let sig = "abc-signature";
        let exp = unix_now() + 1000;
        assert!(!list.is_revoked(sig));
        list.revoke(sig, exp);
        assert!(list.is_revoked(sig));
        assert_eq!(list.len(), 1);
    }

    #[test]
    fn revocation_list_drops_expired_entries() {
        let list = RevocationList::new();
        list.revoke("old-sig", unix_now().saturating_sub(10));
        list.revoke("fresh-sig", unix_now() + 1000);
        // is_revoked filtre les expirés
        assert!(!list.is_revoked("old-sig"));
        assert!(list.is_revoked("fresh-sig"));
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
    fn argon2_hash_then_verify_roundtrip() {
        let h = hash_password("hello123").expect("hash ok");
        assert!(h.starts_with("$argon2"));
        assert!(verify_password(&h, "hello123"));
        assert!(!verify_password(&h, "wrong"));
        // Deux hash du même mot de passe doivent différer (salt aléatoire).
        let h2 = hash_password("hello123").expect("hash ok");
        assert_ne!(h, h2);
    }

    #[test]
    fn verify_password_falls_back_to_plaintext_for_legacy() {
        // Pas de prefix $argon2 → comparaison clair (legacy v1).
        assert!(verify_password("hello123", "hello123"));
        assert!(!verify_password("hello123", "wrong"));
    }

    #[test]
    fn authenticate_works_with_argon2_hash() {
        let h = hash_password("secret").expect("hash ok");
        let users = vec![UserConfig {
            email: "a@x".into(),
            password: h,
            principal: "a".into(),
            allowed_sirens: vec!["111".into()],
            role: Role::Tenant,
        }];
        assert!(authenticate(&users, "a@x", "secret").is_some());
        assert!(authenticate(&users, "a@x", "wrong").is_none());
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
