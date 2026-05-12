//! Authentification, autorisation et isolation tenant.
//!
//! # Modèle
//!
//! Chaque requête HTTP est associée à un [`SecurityContext`] résolu par le
//! middleware [`crate::server::auth_middleware`] : soit à partir d'un
//! cookie de session (login web), soit à partir du header
//! `Authorization: Bearer <token>` (clients API).
//!
//! Le contexte porte :
//! - un `principal` (libellé logique du porteur, pour audit log)
//! - une liste de `allowed_sirens` (vide si rôle élevé)
//! - un [`Role`] (`Tenant` / `PdpOperator` / `PdpAdmin`)
//!
//! Les handlers UI lisent le contexte via l'extractor [`AuthorizedSiren`]
//! qui combine la query `?siren=...` avec [`SecurityContext::can_access`] :
//! une requête vers un SIREN hors scope est rejetée en 403 **avant** que le
//! handler ne tourne. Les handlers API protégés font la même chose ou
//! appellent directement `ctx.can_access(siren)`.

use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::{FromRequestParts, Query};
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;

// Le type `Role` vit dans `pdp-config` car il est porté par le YAML de
// configuration (TokenConfig). On le ré-exporte ici pour que les handlers
// puissent juste `use crate::security::Role`.
pub use pdp_config::model::{Role, TokenConfig};

/// Contexte d'autorisation porté par chaque requête authentifiée.
///
/// Injecté par `auth_middleware` dans `Request::extensions` (clone via
/// `Extension(ctx)` côté handler) et lu par l'extractor [`AuthorizedSiren`].
#[derive(Debug, Clone)]
pub struct SecurityContext {
    /// Identifiant logique du porteur (label libre, sert pour l'audit log).
    pub principal: String,
    /// SIRENs accessibles **explicitement**. Une liste vide combinée à
    /// `Role::Tenant` = aucun accès (toute requête `?siren=...` retourne 403).
    pub allowed_sirens: Vec<String>,
    pub role: Role,
}

impl SecurityContext {
    /// `true` si le porteur peut accéder aux flux d'un SIREN donné.
    ///
    /// - `Tenant` → uniquement si `siren` est dans `allowed_sirens`
    /// - `PdpOperator` / `PdpAdmin` → toujours
    pub fn can_access(&self, siren: &str) -> bool {
        match self.role {
            Role::PdpAdmin | Role::PdpOperator => true,
            Role::Tenant => self.allowed_sirens.iter().any(|s| s == siren),
        }
    }

    /// `true` si le porteur a accès à des opérations cross-tenant
    /// (statistiques globales, listing toutes erreurs, audit).
    pub fn is_cross_tenant_reader(&self) -> bool {
        matches!(self.role, Role::PdpOperator | Role::PdpAdmin)
    }
}

/// Construit la table de lookup `token → SecurityContext` à partir de la
/// liste `tokens` de la config. Tokens vides ignorés.
pub fn build_token_table(tokens: &[TokenConfig]) -> HashMap<String, SecurityContext> {
    tokens
        .iter()
        .filter(|t| !t.token.is_empty())
        .map(|t| {
            (
                t.token.clone(),
                SecurityContext {
                    principal: t.principal.clone(),
                    allowed_sirens: t.allowed_sirens.clone(),
                    role: t.role,
                },
            )
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Extractor : AuthorizedSiren
// ---------------------------------------------------------------------------

/// Extractor Axum qui fournit un SIREN **déjà validé** au handler.
///
/// Combine deux choses :
/// 1. Lit le `?siren=...` de la query string. Vide ou absent → `400`.
/// 2. Vérifie que le `SecurityContext` (déjà injecté par `auth_middleware`)
///    autorise ce SIREN. Sinon → `403 Forbidden`.
///
/// Si le contexte n'est pas présent (route non passée par le middleware,
/// configuration cassée), retourne `500` — c'est un bug, pas une erreur user.
pub struct AuthorizedSiren(pub String);

impl<S: Send + Sync> FromRequestParts<S> for AuthorizedSiren {
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let ctx = parts
            .extensions
            .get::<Arc<SecurityContext>>()
            .cloned()
            .ok_or_else(|| {
                json_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "MISSING_SECURITY_CONTEXT",
                    "Le middleware d'authentification n'a pas injecté de SecurityContext.",
                )
            })?;

        let q: Query<HashMap<String, String>> = Query::from_request_parts(parts, state)
            .await
            .map_err(|_| {
                json_error(
                    StatusCode::BAD_REQUEST,
                    "INVALID_QUERY",
                    "Query string invalide.",
                )
            })?;

        let siren = q
            .get("siren")
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .ok_or_else(|| {
                json_error(
                    StatusCode::BAD_REQUEST,
                    "SIREN_REQUIRED",
                    "Le paramètre ?siren=... est obligatoire sur cette route.",
                )
            })?
            .to_string();

        if !ctx.can_access(&siren) {
            return Err(json_error(
                StatusCode::FORBIDDEN,
                "SIREN_NOT_AUTHORIZED",
                &format!(
                    "Le porteur '{}' n'a pas accès au SIREN {}.",
                    ctx.principal, siren
                ),
            ));
        }

        Ok(AuthorizedSiren(siren))
    }
}

fn json_error(status: StatusCode, code: &str, message: &str) -> Response {
    let body = Json(serde_json::json!({
        "error": code,
        "message": message,
    }));
    (status, body).into_response()
}

/// Pour les pages UI qui acceptent un `?siren=...` optionnel (la page sans
/// siren affiche un sélecteur). Si le siren est fourni, on vérifie le scope ;
/// sinon on retourne `Ok(None)` (laisser le handler décider d'afficher le
/// picker).
///
/// Retourne :
/// - `Ok(Some(siren))` — siren validé, le porteur peut y accéder
/// - `Ok(None)` — pas de siren dans la query
/// - `Err(response)` — 403 si siren hors scope (le handler propage tel quel)
pub fn authorize_optional_siren(
    ctx: &SecurityContext,
    raw_siren: Option<&str>,
) -> Result<Option<String>, Response> {
    let siren = match raw_siren.map(str::trim).filter(|s| !s.is_empty()) {
        Some(s) => s,
        None => return Ok(None),
    };
    if !ctx.can_access(siren) {
        return Err(json_error(
            StatusCode::FORBIDDEN,
            "SIREN_NOT_AUTHORIZED",
            &format!(
                "Le porteur '{}' n'a pas accès au SIREN {}.",
                ctx.principal, siren
            ),
        ));
    }
    Ok(Some(siren.to_string()))
}

// ---------------------------------------------------------------------------
// Tests unitaires
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx_tenant(sirens: &[&str]) -> SecurityContext {
        SecurityContext {
            principal: "p".into(),
            allowed_sirens: sirens.iter().map(|s| s.to_string()).collect(),
            role: Role::Tenant,
        }
    }

    #[test]
    fn tenant_can_only_access_own_siren() {
        let c = ctx_tenant(&["123456789"]);
        assert!(c.can_access("123456789"));
        assert!(!c.can_access("999999999"));
        assert!(!c.is_cross_tenant_reader());
    }

    #[test]
    fn tenant_with_empty_list_blocks_everything() {
        let c = ctx_tenant(&[]);
        assert!(!c.can_access("123456789"));
        assert!(!c.can_access("0"));
    }

    #[test]
    fn pdp_operator_has_cross_tenant_read() {
        let c = SecurityContext {
            principal: "support".into(),
            allowed_sirens: vec![],
            role: Role::PdpOperator,
        };
        assert!(c.can_access("999999999"));
        assert!(c.is_cross_tenant_reader());
    }

    #[test]
    fn pdp_admin_has_full_access() {
        let c = SecurityContext {
            principal: "admin".into(),
            allowed_sirens: vec![],
            role: Role::PdpAdmin,
        };
        assert!(c.can_access("123456789"));
        assert!(c.is_cross_tenant_reader());
    }

    #[test]
    fn build_token_table_skips_empty_tokens() {
        let tokens = vec![
            TokenConfig {
                token: "t1".into(),
                principal: "p1".into(),
                allowed_sirens: vec!["111".into()],
                role: Role::Tenant,
            },
            TokenConfig {
                token: "".into(),
                principal: "skipped".into(),
                allowed_sirens: vec![],
                role: Role::Tenant,
            },
        ];
        let table = build_token_table(&tokens);
        assert_eq!(table.len(), 1);
        assert!(table.contains_key("t1"));
    }

    #[test]
    fn role_default_is_tenant() {
        assert_eq!(Role::default(), Role::Tenant);
    }
}
