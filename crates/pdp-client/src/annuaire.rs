use serde::{Deserialize, Serialize};
use tracing;

use crate::auth::PisteAuth;
use crate::error::{ClientError, ClientResult};
use crate::model::PpfEnvironment;

// ============================================================
// PPF Annuaire — Modèles
// ============================================================

/// Entreprise (unité légale) identifiée par SIREN
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Siren {
    pub siren: Option<String>,
    pub raison_sociale: Option<String>,
    pub type_entite: Option<String>,
    pub statut_administratif: Option<String>,
    /// Identifiant de la PDP rattachée
    pub id_pdp: Option<String>,
    /// Nom de la PDP rattachée
    pub nom_pdp: Option<String>,
}

/// Établissement identifié par SIRET
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Siret {
    pub siret: Option<String>,
    pub siren: Option<String>,
    pub nic: Option<String>,
    pub raison_sociale: Option<String>,
    pub adresse: Option<String>,
    pub code_postal: Option<String>,
    pub ville: Option<String>,
    pub statut_administratif: Option<String>,
}

/// Code de routage (pour déterminer la PDP destinataire)
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoutingCode {
    pub id: Option<String>,
    pub siren: Option<String>,
    pub siret: Option<String>,
    /// Matricule de la PDP (schemeID 0238)
    pub id_pdp: Option<String>,
    pub nom_pdp: Option<String>,
    /// Code de routage spécifique
    pub code_routage: Option<String>,
    pub date_debut: Option<String>,
    pub date_fin: Option<String>,
}

/// Ligne d'annuaire
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectoryLine {
    pub id: Option<String>,
    pub siren: Option<String>,
    pub siret: Option<String>,
    pub id_pdp: Option<String>,
    pub nom_pdp: Option<String>,
    pub statut: Option<String>,
}

/// Résultat de recherche paginé
#[derive(Debug, Clone, Deserialize)]
pub struct SearchResponse<T> {
    pub items: Option<Vec<T>>,
    pub total: Option<u64>,
}

/// Résultat de résolution de routage
#[derive(Debug, Clone)]
pub struct RoutingResolution {
    /// Matricule de la PDP destinataire (0238)
    pub pdp_matricule: String,
    /// Nom de la PDP destinataire
    pub pdp_name: String,
    /// URL du Flow Service de la PDP destinataire (si connue)
    pub flow_service_url: Option<String>,
    /// Le destinataire est-il sur le PPF directement ?
    pub is_ppf: bool,
}

// ============================================================
// PPF Annuaire Client
// ============================================================

/// Configuration du client Annuaire PPF
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnuaireConfig {
    pub environment: PpfEnvironment,
}

impl AnnuaireConfig {
    fn base_url(&self) -> String {
        format!("{}/ppf/annuaire/v1.9.0", self.environment.base_url())
    }
}

/// Client pour l'API Annuaire du PPF
pub struct AnnuaireClient {
    config: AnnuaireConfig,
    auth: PisteAuth,
    http: reqwest::Client,
}

impl AnnuaireClient {
    pub fn new(config: AnnuaireConfig, auth: PisteAuth) -> Self {
        Self {
            config,
            auth,
            http: reqwest::Client::new(),
        }
    }

    async fn auth_header(&self) -> ClientResult<String> {
        let token = self.auth.get_token().await?;
        Ok(format!("Bearer {}", token))
    }

    async fn handle_response<T: serde::de::DeserializeOwned>(
        &self,
        response: reqwest::Response,
        operation: &str,
    ) -> ClientResult<T> {
        let status = response.status();
        if status.is_success() {
            return Ok(response.json::<T>().await?);
        }
        if status.as_u16() == 401 {
            self.auth.invalidate().await;
            return Err(ClientError::TokenExpired);
        }
        let body = response.text().await.unwrap_or_default();
        Err(ClientError::HttpError {
            status: status.as_u16(),
            message: format!("{}: {}", operation, body),
        })
    }

    /// Consulte une entreprise par SIREN
    pub async fn get_siren(&self, siren: &str) -> ClientResult<Siren> {
        let url = format!("{}/siren/{}", self.config.base_url(), siren);
        let auth = self.auth_header().await?;

        tracing::debug!(siren = %siren, "Consultation annuaire SIREN");

        let response = self
            .http
            .get(&url)
            .header("Authorization", auth)
            .send()
            .await?;

        self.handle_response(response, "get_siren").await
    }

    /// Consulte un établissement par SIRET
    pub async fn get_siret(&self, siret: &str) -> ClientResult<Siret> {
        let url = format!("{}/siret/{}", self.config.base_url(), siret);
        let auth = self.auth_header().await?;

        tracing::debug!(siret = %siret, "Consultation annuaire SIRET");

        let response = self
            .http
            .get(&url)
            .header("Authorization", auth)
            .send()
            .await?;

        self.handle_response(response, "get_siret").await
    }

    /// Recherche de codes de routage pour un SIREN/SIRET donné.
    /// Permet de déterminer quelle PDP gère le destinataire.
    pub async fn rechercher_routage(
        &self,
        siren: Option<&str>,
        siret: Option<&str>,
    ) -> ClientResult<SearchResponse<RoutingCode>> {
        let url = format!("{}/routage/recherche", self.config.base_url());
        let auth = self.auth_header().await?;

        let mut criteria = serde_json::Map::new();
        if let Some(s) = siren {
            criteria.insert("siren".to_string(), serde_json::Value::String(s.to_string()));
        }
        if let Some(s) = siret {
            criteria.insert("siret".to_string(), serde_json::Value::String(s.to_string()));
        }

        tracing::debug!(
            siren = siren.unwrap_or("N/A"),
            siret = siret.unwrap_or("N/A"),
            "Recherche routage annuaire"
        );

        let response = self
            .http
            .post(&url)
            .header("Authorization", auth)
            .json(&serde_json::Value::Object(criteria))
            .send()
            .await?;

        self.handle_response(response, "rechercher_routage").await
    }

    /// Résout le routage pour un destinataire donné (SIREN ou SIRET).
    /// Retourne la PDP destinataire ou indique que c'est le PPF.
    pub async fn resoudre_routage(
        &self,
        buyer_siren: Option<&str>,
        buyer_siret: Option<&str>,
    ) -> ClientResult<RoutingResolution> {
        let result = self
            .rechercher_routage(buyer_siren, buyer_siret)
            .await?;

        if let Some(items) = &result.items {
            if let Some(routing) = items.first() {
                if let Some(ref pdp_id) = routing.id_pdp {
                    // Le matricule "0000" = PPF
                    let is_ppf = pdp_id == "0000";

                    return Ok(RoutingResolution {
                        pdp_matricule: pdp_id.clone(),
                        pdp_name: routing.nom_pdp.clone().unwrap_or_default(),
                        flow_service_url: None,
                        is_ppf,
                    });
                }
            }
        }

        // Par défaut, envoyer au PPF
        tracing::warn!(
            buyer_siren = buyer_siren.unwrap_or("N/A"),
            buyer_siret = buyer_siret.unwrap_or("N/A"),
            "Aucun routage trouvé, envoi au PPF par défaut"
        );

        Ok(RoutingResolution {
            pdp_matricule: "0000".to_string(),
            pdp_name: "PPF".to_string(),
            flow_service_url: None,
            is_ppf: true,
        })
    }
}

// ============================================================
// AFNOR Directory Service Client (PDP↔PDP)
// ============================================================

/// Configuration du client AFNOR Directory Service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AfnorDirectoryConfig {
    /// URL de base du Directory Service
    pub base_url: String,
}

/// Entreprise AFNOR Directory
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AfnorSiren {
    pub siren: Option<String>,
    pub business_name: Option<String>,
    pub entity_type: Option<String>,
    pub administrative_status: Option<String>,
    pub id_instance: Option<String>,
}

/// Établissement AFNOR Directory
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AfnorSiret {
    pub siret: Option<String>,
    pub siren: Option<String>,
    pub nic: Option<String>,
    pub business_name: Option<String>,
    pub id_instance: Option<String>,
}

/// Code de routage AFNOR
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AfnorRoutingCode {
    pub id: Option<String>,
    pub code: Option<String>,
    pub description: Option<String>,
    pub siret: Option<String>,
    pub siren: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

/// Ligne d'annuaire AFNOR
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AfnorDirectoryLine {
    pub id: Option<String>,
    pub siren: Option<String>,
    pub siret: Option<String>,
    pub routing_code: Option<String>,
    pub status: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

/// Client pour l'API AFNOR Directory Service (PDP↔PDP)
pub struct AfnorDirectoryClient {
    config: AfnorDirectoryConfig,
    auth: PisteAuth,
    http: reqwest::Client,
}

impl AfnorDirectoryClient {
    pub fn new(config: AfnorDirectoryConfig, auth: PisteAuth) -> Self {
        Self {
            config,
            auth,
            http: reqwest::Client::new(),
        }
    }

    async fn auth_header(&self) -> ClientResult<String> {
        let token = self.auth.get_token().await?;
        Ok(format!("Bearer {}", token))
    }

    async fn handle_response<T: serde::de::DeserializeOwned>(
        &self,
        response: reqwest::Response,
        operation: &str,
    ) -> ClientResult<T> {
        let status = response.status();
        if status.is_success() {
            return Ok(response.json::<T>().await?);
        }
        if status.as_u16() == 401 {
            self.auth.invalidate().await;
            return Err(ClientError::TokenExpired);
        }
        let body = response.text().await.unwrap_or_default();
        Err(ClientError::HttpError {
            status: status.as_u16(),
            message: format!("{}: {}", operation, body),
        })
    }

    /// Consulte une entreprise par SIREN dans l'annuaire AFNOR
    pub async fn get_siren(&self, siren: &str) -> ClientResult<AfnorSiren> {
        let url = format!(
            "{}/v1/siren/code-insee:{}",
            self.config.base_url, siren
        );
        let auth = self.auth_header().await?;

        let response = self
            .http
            .get(&url)
            .header("Authorization", auth)
            .header("Accept-Language", "fr")
            .send()
            .await?;

        self.handle_response(response, "afnor_get_siren").await
    }

    /// Consulte un établissement par SIRET dans l'annuaire AFNOR
    pub async fn get_siret(&self, siret: &str) -> ClientResult<AfnorSiret> {
        let url = format!(
            "{}/v1/siret/code-insee:{}",
            self.config.base_url, siret
        );
        let auth = self.auth_header().await?;

        let response = self
            .http
            .get(&url)
            .header("Authorization", auth)
            .header("Accept-Language", "fr")
            .send()
            .await?;

        self.handle_response(response, "afnor_get_siret").await
    }

    /// Recherche de codes de routage AFNOR
    pub async fn search_routing_codes(
        &self,
        criteria: &serde_json::Value,
    ) -> ClientResult<SearchResponse<AfnorRoutingCode>> {
        let url = format!("{}/v1/routing-code/search", self.config.base_url);
        let auth = self.auth_header().await?;

        let response = self
            .http
            .post(&url)
            .header("Authorization", auth)
            .json(criteria)
            .send()
            .await?;

        self.handle_response(response, "afnor_search_routing_codes")
            .await
    }

    /// Crée un code de routage AFNOR
    pub async fn create_routing_code(
        &self,
        routing_code: &AfnorRoutingCode,
    ) -> ClientResult<AfnorRoutingCode> {
        let url = format!("{}/v1/routing-code", self.config.base_url);
        let auth = self.auth_header().await?;

        let response = self
            .http
            .post(&url)
            .header("Authorization", auth)
            .json(routing_code)
            .send()
            .await?;

        self.handle_response(response, "afnor_create_routing_code")
            .await
    }

    /// Recherche de lignes d'annuaire AFNOR
    pub async fn search_directory_lines(
        &self,
        criteria: &serde_json::Value,
    ) -> ClientResult<SearchResponse<AfnorDirectoryLine>> {
        let url = format!("{}/v1/directory-line/search", self.config.base_url);
        let auth = self.auth_header().await?;

        let response = self
            .http
            .post(&url)
            .header("Authorization", auth)
            .json(criteria)
            .send()
            .await?;

        self.handle_response(response, "afnor_search_directory_lines")
            .await
    }

    /// Crée une ligne d'annuaire AFNOR
    pub async fn create_directory_line(
        &self,
        line: &AfnorDirectoryLine,
    ) -> ClientResult<AfnorDirectoryLine> {
        let url = format!("{}/v1/directory-line", self.config.base_url);
        let auth = self.auth_header().await?;

        let response = self
            .http
            .post(&url)
            .header("Authorization", auth)
            .json(line)
            .send()
            .await?;

        self.handle_response(response, "afnor_create_directory_line")
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_annuaire_config_url() {
        let config = AnnuaireConfig {
            environment: PpfEnvironment::Rec,
        };
        assert!(config.base_url().contains("env.rec."));
        assert!(config.base_url().ends_with("/ppf/annuaire/v1.9.0"));
    }

    #[test]
    fn test_routing_resolution_ppf() {
        let resolution = RoutingResolution {
            pdp_matricule: "0000".to_string(),
            pdp_name: "PPF".to_string(),
            flow_service_url: None,
            is_ppf: true,
        };
        assert!(resolution.is_ppf);
    }

    #[test]
    fn test_routing_resolution_pdp() {
        let resolution = RoutingResolution {
            pdp_matricule: "1111".to_string(),
            pdp_name: "UNO".to_string(),
            flow_service_url: Some("https://api.flow.uno.fr/flow-service".to_string()),
            is_ppf: false,
        };
        assert!(!resolution.is_ppf);
        assert_eq!(resolution.pdp_matricule, "1111");
    }
}
