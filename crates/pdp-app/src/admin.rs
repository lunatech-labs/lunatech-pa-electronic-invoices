//! Administration : ajout/listing des entreprises (= tenants SIREN).
//!
//! Routes (toutes sous `/ui/admin/*`, protégées par `auth_middleware` + un
//! check explicite `require_admin` qui n'autorise QUE le rôle
//! [`crate::security::Role::PdpAdmin`]) :
//!
//! - `GET  /ui/admin`                    — Tableau de bord + liste entreprises
//! - `GET  /ui/admin/entreprises/new`      — Formulaire de création
//! - `POST /ui/admin/entreprises`          — Création (form-urlencoded)
//!
//! # Modèle
//!
//! Une "entreprise" est l'équivalent d'un tenant `pdp-config` : un dossier
//! `{tenants_dir}/{siren}/` qui contient :
//! - `config.yaml` — [`pdp_config::TenantConfig`] (identité PDP du tenant)
//! - `in/` — Le client y dépose ses factures (émission)
//! - `out/` — La PDP y dépose les factures reçues + CDV
//!
//! Le répertoire `tenants_dir` est résolu depuis `config.tenants_dir`
//! (cf. [`crate::server::AppState::tenants_dir`]). Si non configuré,
//! l'écran affiche un bandeau d'erreur et bloque toute création.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
};
use serde::Deserialize;

use crate::security::{Role, SecurityContext};
use crate::server::AppState;
use crate::ui::{html_escape, html_response, page_shell};

// ============================================================
// Garde d'autorisation
// ============================================================

/// Rejette tout porteur dont le rôle n'est pas `PdpAdmin`.
///
/// Pour les routes UI on retourne un `403 Forbidden` avec une page HTML
/// minimale — pas de redirection vers `/login` car le porteur est bien
/// authentifié, c'est juste qu'il n'a pas les droits.
fn require_admin(ctx: &SecurityContext) -> Result<(), Response> {
    if matches!(ctx.role, Role::PdpAdmin) {
        Ok(())
    } else {
        let body = format!(
            r#"<div class="card">
                <h2>Accès refusé</h2>
                <p>L'administration des entreprises est réservée au rôle <code>pdp_admin</code>.</p>
                <p>Porteur actuel : <code>{}</code> (rôle : <code>{:?}</code>).</p>
                <p><a href="/ui">← Retour au dashboard</a></p>
            </div>"#,
            html_escape(&ctx.principal),
            ctx.role,
        );
        Err((
            StatusCode::FORBIDDEN,
            axum::response::Html(page_shell("Accès refusé", "admin", None, ctx, &body)),
        )
            .into_response())
    }
}

// ============================================================
// Modèle : Entreprise
// ============================================================

/// Vue d'une entreprise telle qu'affichée dans la liste admin.
///
/// Note : `PdpIdentity` porte un champ `matricule` (assigné par la DGFiP,
/// schemeID 0238) qui n'a de sens que pour la PDP elle-même (Ferrite), pas
/// pour les entreprises clientes. On l'ignore ici.
struct Entreprise {
    siren: String,
    name: String,
    siret: Option<String>,
}

/// Scanne `tenants_dir` et retourne la liste des entreprises triées par SIREN.
///
/// Réutilise [`pdp_config::discover_tenants`] pour ne pas dupliquer la
/// logique de chargement de `config.yaml` + auto-config.
fn list_entreprises(tenants_dir: &Path) -> Result<Vec<Entreprise>, String> {
    // `discover_tenants` crée aussi les répertoires `in/` et `out/`
    // manquants. C'est tolérable ici car cette page n'est lue que par un
    // admin et l'effet de bord est exactement celui qu'on souhaite si
    // l'arborescence est partielle.
    let entries = pdp_config::discover_tenants(tenants_dir)?;
    Ok(entries
        .into_iter()
        .map(|e| Entreprise {
            siren: e.siren.clone(),
            name: e.config.pdp.name.clone(),
            siret: e.config.pdp.siret.clone(),
        })
        .collect())
}

// ============================================================
// GET /ui/admin — tableau de bord + liste
// ============================================================

pub async fn handle_admin_dashboard(
    State(state): State<Arc<AppState>>,
    axum::Extension(ctx): axum::Extension<Arc<SecurityContext>>,
) -> Response {
    if let Err(resp) = require_admin(&ctx) {
        return resp;
    }

    let body = match &state.tenants_dir {
        None => no_tenants_dir_banner(),
        Some(dir) => render_entreprises_list(dir),
    };

    html_response(&page_shell(
        "Administration",
        "admin",
        None,
        &ctx,
        &body,
    ))
}

fn render_entreprises_list(tenants_dir: &Path) -> String {
    let entreprises = match list_entreprises(tenants_dir) {
        Ok(c) => c,
        Err(e) => {
            return format!(
                r#"<div class="card"><h2>Erreur</h2><p>Impossible de lire <code>{}</code> : {}</p></div>"#,
                html_escape(&tenants_dir.display().to_string()),
                html_escape(&e),
            );
        }
    };

    let rows = if entreprises.is_empty() {
        r#"<tr><td colspan="4" class="empty">Aucune entreprise enregistrée. Cliquez sur « Ajouter une entreprise » pour en créer une.</td></tr>"#.to_string()
    } else {
        entreprises
            .iter()
            .map(|c| {
                format!(
                    r#"<tr>
                        <td><code>{siren}</code></td>
                        <td>{name}</td>
                        <td><code>{siret}</code></td>
                        <td>
                            <a href="/ui?siren={siren}">Dashboard tenant</a>
                            &nbsp;·&nbsp;
                            <a href="/ui/emises?siren={siren}">Émises</a>
                            &nbsp;·&nbsp;
                            <a href="/ui/recues?siren={siren}">Reçues</a>
                        </td>
                    </tr>"#,
                    siren = html_escape(&c.siren),
                    name = html_escape(&c.name),
                    siret = html_escape(c.siret.as_deref().unwrap_or("—")),
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

    format!(
        r#"<div class="card">
            <h2>Administration des entreprises</h2>
            <p style="color:#666">
                Répertoire racine : <code>{dir}</code>.
                Une entreprise correspond à un sous-dossier <code>{{siren}}/</code> contenant
                <code>config.yaml</code>, <code>in/</code> et <code>out/</code>.
            </p>
            <p style="margin-top:1rem">
                <a href="/ui/admin/entreprises/new" class="dl-btn">+ Ajouter une entreprise</a>
            </p>
        </div>

        <div class="card">
            <h2>Entreprises enregistrées ({count})</h2>
            <table>
                <thead>
                    <tr>
                        <th>SIREN</th>
                        <th>Nom</th>
                        <th>SIRET</th>
                        <th>Actions</th>
                    </tr>
                </thead>
                <tbody>{rows}</tbody>
            </table>
        </div>"#,
        dir = html_escape(&tenants_dir.display().to_string()),
        count = entreprises.len(),
        rows = rows,
    )
}

fn no_tenants_dir_banner() -> String {
    r#"<div class="banner">
        Aucun répertoire <code>tenants_dir</code> n'est configuré dans le YAML
        de la PDP (clé <code>tenants_dir:</code> à la racine de
        <code>config.yaml</code>). L'administration des entreprises est
        désactivée tant que ce répertoire n'est pas défini.
    </div>"#
        .to_string()
}

// ============================================================
// GET /ui/admin/entreprises/new — formulaire
// ============================================================

pub async fn handle_admin_entreprise_new(
    State(state): State<Arc<AppState>>,
    axum::Extension(ctx): axum::Extension<Arc<SecurityContext>>,
) -> Response {
    if let Err(resp) = require_admin(&ctx) {
        return resp;
    }
    let body = match &state.tenants_dir {
        None => no_tenants_dir_banner(),
        Some(_) => render_new_form(None, &NewEntrepriseForm::default()),
    };
    html_response(&page_shell(
        "Nouvelle entreprise",
        "admin",
        None,
        &ctx,
        &body,
    ))
}

fn render_new_form(error: Option<&str>, prefill: &NewEntrepriseForm) -> String {
    let error_html = error
        .map(|e| {
            format!(
                r#"<div class="banner" style="background:#ffebee;color:#b71c1c">⚠️ {}</div>"#,
                html_escape(e),
            )
        })
        .unwrap_or_default();

    format!(
        r#"{error}
        <div class="card">
            <h2>Ajouter une entreprise</h2>
            <p style="color:#666">
                L'entreprise sera créée dans <code>{{tenants_dir}}/{{siren}}/</code> avec
                <code>config.yaml</code>, <code>in/</code> et <code>out/</code>.
                Elle pourra immédiatement émettre et recevoir des factures via les routes
                auto-générées du pipeline (au prochain redémarrage de la PDP).
            </p>
            <form method="post" action="/ui/admin/entreprises" style="margin-top:1.5rem">
                <div style="display:grid;grid-template-columns:200px 1fr;gap:1rem 1.5rem;align-items:center">
                    <label for="siren">SIREN <span style="color:#d32f2f">*</span></label>
                    <input id="siren" name="siren" required pattern="[0-9]{{9}}"
                           maxlength="9" placeholder="9 chiffres, ex: 123456789"
                           value="{siren}" autofocus>

                    <label for="name">Nom de l'entreprise <span style="color:#d32f2f">*</span></label>
                    <input id="name" name="name" required maxlength="200"
                           placeholder="Raison sociale" value="{name}">

                    <label for="siret">SIRET</label>
                    <input id="siret" name="siret" pattern="[0-9]{{14}}" maxlength="14"
                           placeholder="14 chiffres (optionnel)" value="{siret}">
                </div>
                <div style="margin-top:1.5rem;display:flex;gap:0.6rem">
                    <button type="submit" class="dl-btn" style="border:none;cursor:pointer">
                        Créer l'entreprise
                    </button>
                    <a href="/ui/admin" style="padding:0.5rem 1rem">Annuler</a>
                </div>
            </form>
        </div>"#,
        error = error_html,
        siren = html_escape(&prefill.siren),
        name = html_escape(&prefill.name),
        siret = html_escape(prefill.siret.as_deref().unwrap_or("")),
    )
}

// ============================================================
// POST /ui/admin/entreprises — création
// ============================================================

#[derive(Deserialize, Default, Clone)]
pub struct NewEntrepriseForm {
    #[serde(default)]
    pub siren: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub siret: Option<String>,
}

impl NewEntrepriseForm {
    /// Normalise les champs : trim partout, vide → None pour les optionnels.
    fn normalized(&self) -> Self {
        fn norm_opt(s: &Option<String>) -> Option<String> {
            s.as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string)
        }
        Self {
            siren: self.siren.trim().to_string(),
            name: self.name.trim().to_string(),
            siret: norm_opt(&self.siret),
        }
    }
}

pub async fn handle_admin_entreprise_create(
    State(state): State<Arc<AppState>>,
    axum::Extension(ctx): axum::Extension<Arc<SecurityContext>>,
    axum::Form(raw_form): axum::Form<NewEntrepriseForm>,
) -> Response {
    if let Err(resp) = require_admin(&ctx) {
        return resp;
    }

    let tenants_dir = match &state.tenants_dir {
        Some(d) => d.clone(),
        None => {
            return html_response(&page_shell(
                "Nouvelle entreprise",
                "admin",
                None,
                &ctx,
                &no_tenants_dir_banner(),
            ));
        }
    };

    let form = raw_form.normalized();

    // Validation des champs. On ré-affiche le formulaire avec les valeurs
    // saisies (et l'erreur en bandeau) plutôt que de retourner un 400 sec :
    // c'est un formulaire web, l'utilisateur attend de pouvoir corriger.
    if let Err(msg) = validate_form(&form) {
        return render_form_error(&ctx, &form, &msg);
    }

    let target = tenants_dir.join(&form.siren);
    if target.exists() {
        return render_form_error(
            &ctx,
            &form,
            &format!(
                "Le SIREN {} existe déjà ({}). Choisir un autre SIREN ou supprimer le dossier manuellement.",
                form.siren,
                target.display(),
            ),
        );
    }

    if let Err(e) = create_entreprise_on_disk(&tenants_dir, &form) {
        // Best-effort cleanup pour éviter de laisser un répertoire partiel.
        let _ = std::fs::remove_dir_all(&target);
        return render_form_error(
            &ctx,
            &form,
            &format!("Erreur lors de la création de l'entreprise : {}", e),
        );
    }

    tracing::info!(
        siren = %form.siren,
        name = %form.name,
        principal = %ctx.principal,
        "Nouvelle entreprise créée par l'administrateur"
    );

    Redirect::to("/ui/admin").into_response()
}

fn render_form_error(ctx: &SecurityContext, form: &NewEntrepriseForm, msg: &str) -> Response {
    let body = render_new_form(Some(msg), form);
    (
        StatusCode::BAD_REQUEST,
        axum::response::Html(page_shell(
            "Nouvelle entreprise",
            "admin",
            None,
            ctx,
            &body,
        )),
    )
        .into_response()
}

fn validate_form(form: &NewEntrepriseForm) -> Result<(), String> {
    if !pdp_config::is_valid_siren(&form.siren) {
        return Err("Le SIREN doit faire exactement 9 chiffres.".to_string());
    }
    if form.name.is_empty() {
        return Err("Le nom de l'entreprise est obligatoire.".to_string());
    }
    if form.name.len() > 200 {
        return Err("Le nom ne peut pas dépasser 200 caractères.".to_string());
    }
    if let Some(siret) = form.siret.as_deref() {
        if siret.len() != 14 || !siret.chars().all(|c| c.is_ascii_digit()) {
            return Err("Le SIRET doit faire 14 chiffres (ou être laissé vide).".to_string());
        }
        if !siret.starts_with(&form.siren) {
            return Err("Le SIRET doit commencer par le SIREN saisi.".to_string());
        }
    }
    Ok(())
}

/// Crée le dossier entreprise + `in/` + `out/` + `config.yaml`.
/// Le `config.yaml` contient une [`pdp_config::TenantConfig`] minimale
/// (identité PDP + routes vides — les routes sont auto-générées par le
/// runtime à partir de `in/` et `out/`).
fn create_entreprise_on_disk(tenants_dir: &Path, form: &NewEntrepriseForm) -> Result<PathBuf, String> {
    use pdp_config::model::{PdpIdentity, TenantConfig};

    std::fs::create_dir_all(tenants_dir)
        .map_err(|e| format!("création {}: {}", tenants_dir.display(), e))?;

    let target = tenants_dir.join(&form.siren);
    std::fs::create_dir(&target).map_err(|e| format!("création {}: {}", target.display(), e))?;
    std::fs::create_dir(target.join("in"))
        .map_err(|e| format!("création in/: {}", e))?;
    std::fs::create_dir(target.join("out"))
        .map_err(|e| format!("création out/: {}", e))?;

    // Note : `matricule` reste à `None`. Ce champ identifie la PDP (Ferrite,
    // schemeID 0238 DGFiP) — il ne s'applique pas à une entreprise cliente.
    let config = TenantConfig {
        pdp: PdpIdentity {
            id: format!("TENANT-{}", form.siren),
            name: form.name.clone(),
            siren: Some(form.siren.clone()),
            siret: form.siret.clone(),
            matricule: None,
        },
        routes: Vec::new(),
        ppf: None,
        afnor: None,
    };

    let yaml = serde_yaml::to_string(&config)
        .map_err(|e| format!("sérialisation config.yaml: {}", e))?;
    let config_path = target.join("config.yaml");
    std::fs::write(&config_path, yaml)
        .map_err(|e| format!("écriture {}: {}", config_path.display(), e))?;

    Ok(target)
}

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn admin_ctx() -> SecurityContext {
        SecurityContext {
            principal: "admin@test".into(),
            allowed_sirens: vec![],
            role: Role::PdpAdmin,
        }
    }

    fn tenant_ctx() -> SecurityContext {
        SecurityContext {
            principal: "tenant".into(),
            allowed_sirens: vec!["123456789".into()],
            role: Role::Tenant,
        }
    }

    #[test]
    fn require_admin_accepts_pdp_admin() {
        assert!(require_admin(&admin_ctx()).is_ok());
    }

    #[test]
    fn require_admin_rejects_tenant() {
        assert!(require_admin(&tenant_ctx()).is_err());
    }

    #[test]
    fn require_admin_rejects_operator() {
        let ctx = SecurityContext {
            principal: "support".into(),
            allowed_sirens: vec![],
            role: Role::PdpOperator,
        };
        assert!(require_admin(&ctx).is_err());
    }

    #[test]
    fn validate_form_accepts_minimum_valid() {
        let form = NewEntrepriseForm {
            siren: "123456789".into(),
            name: "Acme Corp".into(),
            siret: None,
        };
        assert!(validate_form(&form).is_ok());
    }

    #[test]
    fn validate_form_rejects_bad_siren() {
        let form = NewEntrepriseForm {
            siren: "12345".into(),
            name: "Acme".into(),
            ..Default::default()
        };
        assert!(validate_form(&form).is_err());
    }

    #[test]
    fn validate_form_rejects_empty_name() {
        let form = NewEntrepriseForm {
            siren: "123456789".into(),
            name: "".into(),
            ..Default::default()
        };
        assert!(validate_form(&form).is_err());
    }

    #[test]
    fn validate_form_rejects_siret_not_matching_siren() {
        let form = NewEntrepriseForm {
            siren: "123456789".into(),
            name: "Acme".into(),
            siret: Some("98765432101234".into()),
            ..Default::default()
        };
        assert!(validate_form(&form).is_err());
    }

    #[test]
    fn validate_form_accepts_siret_starting_with_siren() {
        let form = NewEntrepriseForm {
            siren: "123456789".into(),
            name: "Acme".into(),
            siret: Some("12345678901234".into()),
            ..Default::default()
        };
        assert!(validate_form(&form).is_ok());
    }

    #[test]
    fn create_entreprise_writes_full_layout() {
        let tmp = TempDir::new().unwrap();
        let form = NewEntrepriseForm {
            siren: "111222333".into(),
            name: "Test Co".into(),
            siret: Some("11122233300001".into()),
        };
        let target = create_entreprise_on_disk(tmp.path(), &form).unwrap();
        assert!(target.join("in").is_dir());
        assert!(target.join("out").is_dir());
        assert!(target.join("config.yaml").is_file());

        let entries = pdp_config::discover_tenants(tmp.path()).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].siren, "111222333");
        assert_eq!(entries[0].config.pdp.name, "Test Co");
        assert_eq!(entries[0].config.pdp.siret.as_deref(), Some("11122233300001"));
        // Le matricule est volontairement laissé vide : il identifie la PDP,
        // pas l'entreprise cliente.
        assert!(entries[0].config.pdp.matricule.is_none());
    }

    #[test]
    fn list_entreprises_returns_sorted_by_siren() {
        let tmp = TempDir::new().unwrap();
        for (siren, name) in [("999111222", "C"), ("111222333", "A"), ("555666777", "B")] {
            create_entreprise_on_disk(
                tmp.path(),
                &NewEntrepriseForm {
                    siren: siren.into(),
                    name: name.into(),
                    ..Default::default()
                },
            )
            .unwrap();
        }
        let entreprises = list_entreprises(tmp.path()).unwrap();
        assert_eq!(entreprises.len(), 3);
        assert_eq!(entreprises[0].siren, "111222333");
        assert_eq!(entreprises[1].siren, "555666777");
        assert_eq!(entreprises[2].siren, "999111222");
    }

    #[test]
    fn normalized_strips_whitespace_and_empty_optionals() {
        let form = NewEntrepriseForm {
            siren: "  123456789 ".into(),
            name: "  Acme  ".into(),
            siret: Some("   ".into()),
        };
        let n = form.normalized();
        assert_eq!(n.siren, "123456789");
        assert_eq!(n.name, "Acme");
        assert_eq!(n.siret, None);
    }
}
