use serde::{Deserialize, Serialize};
use tracing;

use crate::auth::PisteAuth;
use crate::error::{ClientError, ClientResult};
use crate::model::{
    AfnorRequestHeaders, HealthCheckResponse, PpfEnvironment,
    SearchSirenParams, SearchSiretParams,
};

// ============================================================
// PPF Annuaire — Modeles
// ============================================================

/// Entreprise (unite legale) identifiee par SIREN
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Siren {
    pub siren: Option<String>,
    pub raison_sociale: Option<String>,
    pub type_entite: Option<String>,
    pub statut_administratif: Option<String>,
    /// Identifiant de la PDP rattachee
    pub id_pdp: Option<String>,
    /// Nom de la PDP rattachee
    pub nom_pdp: Option<String>,
}

/// Etablissement identifie par SIRET
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

/// Code de routage (pour determiner la PDP destinataire)
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoutingCode {
    pub id: Option<String>,
    pub siren: Option<String>,
    pub siret: Option<String>,
    /// Matricule de la PDP (schemeID 0238)
    pub id_pdp: Option<String>,
    pub nom_pdp: Option<String>,
    /// Code de routage specifique
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

/// Resultat de recherche pagine
#[derive(Debug, Clone, Deserialize)]
pub struct SearchResponse<T> {
    pub items: Option<Vec<T>>,
    pub total: Option<u64>,
}

/// Resultat de resolution de routage
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
        }

        let retry_after = response
            .headers()
            .get("Retry-After")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u64>().ok());

        let body = response.text().await.unwrap_or_default();
        Err(ClientError::from_http_response(
            status.as_u16(),
            &body,
            operation,
            retry_after,
        ))
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

    /// Consulte un etablissement par SIRET
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

    /// Recherche de codes de routage pour un SIREN/SIRET donne.
    /// Permet de determiner quelle PDP gere le destinataire.
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

    /// Resout le routage pour un destinataire donne (SIREN ou SIRET).
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

        // Par defaut, envoyer au PPF
        tracing::warn!(
            buyer_siren = buyer_siren.unwrap_or("N/A"),
            buyer_siret = buyer_siret.unwrap_or("N/A"),
            "Aucun routage trouve, envoi au PPF par defaut"
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
// AFNOR Directory Service Client (PDP<>PDP) — XP Z12-013 Annexe B
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

/// Etablissement AFNOR Directory
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

/// Client pour l'API AFNOR Directory Service v1.2.0 (PDP<>PDP)
///
/// Implemente l'ensemble des endpoints definis dans l'Annexe B de la norme XP Z12-013 :
/// - SIREN : GET par code-insee, POST search
/// - SIRET : GET par code-insee, POST search
/// - Routing codes : GET par siret+code, POST search, POST create
/// - Directory lines : GET par addressing-id, POST search, POST create
/// - Health check : GET /v1/healthcheck
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

    /// Applique les headers optionnels AFNOR
    fn apply_headers(
        &self,
        mut builder: reqwest::RequestBuilder,
        headers: Option<&AfnorRequestHeaders>,
    ) -> reqwest::RequestBuilder {
        if let Some(h) = headers {
            if let Some(ref id) = h.request_id {
                builder = builder.header("Request-Id", id.as_str());
            }
            if let Some(ref org) = h.organization_id {
                builder = builder.header("Organization-Id", org.as_str());
            }
            if let Some(ref lang) = h.accept_language {
                builder = builder.header("Accept-Language", lang.as_str());
            }
        } else {
            // Default Accept-Language pour le Directory Service
            builder = builder.header("Accept-Language", "fr");
        }
        builder
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
        }

        let retry_after = response
            .headers()
            .get("Retry-After")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u64>().ok());

        let body = response.text().await.unwrap_or_default();
        Err(ClientError::from_http_response(
            status.as_u16(),
            &body,
            operation,
            retry_after,
        ))
    }

    // ============================================================
    // SIREN — GET + POST search
    // ============================================================

    /// Consulte une entreprise par SIREN dans l'annuaire AFNOR.
    /// Correspond a `GET /v1/siren/code-insee:{siren}`
    ///
    /// Le parametre `fields` permet de selectionner les champs retournes.
    pub async fn get_siren(
        &self,
        siren: &str,
        fields: Option<&[&str]>,
        headers: Option<&AfnorRequestHeaders>,
    ) -> ClientResult<AfnorSiren> {
        let mut url = format!(
            "{}/v1/siren/code-insee:{}",
            self.config.base_url, siren
        );
        if let Some(f) = fields {
            let fields_str = f.join(",");
            url = format!("{}?fields={}", url, fields_str);
        }
        let auth = self.auth_header().await?;

        let builder = self
            .http
            .get(&url)
            .header("Authorization", auth);

        let builder = self.apply_headers(builder, headers);

        let response = builder.send().await?;

        self.handle_response(response, "afnor_get_siren").await
    }

    /// Recherche d'entreprises par SIREN avec criteres.
    /// Correspond a `POST /v1/siren/search`
    pub async fn search_siren(
        &self,
        params: &SearchSirenParams,
        headers: Option<&AfnorRequestHeaders>,
    ) -> ClientResult<SearchResponse<AfnorSiren>> {
        let url = format!("{}/v1/siren/search", self.config.base_url);
        let auth = self.auth_header().await?;

        let builder = self
            .http
            .post(&url)
            .header("Authorization", auth)
            .json(params);

        let builder = self.apply_headers(builder, headers);

        let response = builder.send().await?;

        self.handle_response(response, "afnor_search_siren").await
    }

    // ============================================================
    // SIRET — GET + POST search
    // ============================================================

    /// Consulte un etablissement par SIRET dans l'annuaire AFNOR.
    /// Correspond a `GET /v1/siret/code-insee:{siret}`
    pub async fn get_siret(
        &self,
        siret: &str,
        fields: Option<&[&str]>,
        headers: Option<&AfnorRequestHeaders>,
    ) -> ClientResult<AfnorSiret> {
        let mut url = format!(
            "{}/v1/siret/code-insee:{}",
            self.config.base_url, siret
        );
        if let Some(f) = fields {
            let fields_str = f.join(",");
            url = format!("{}?fields={}", url, fields_str);
        }
        let auth = self.auth_header().await?;

        let builder = self
            .http
            .get(&url)
            .header("Authorization", auth);

        let builder = self.apply_headers(builder, headers);

        let response = builder.send().await?;

        self.handle_response(response, "afnor_get_siret").await
    }

    /// Recherche d'etablissements par SIRET avec criteres.
    /// Correspond a `POST /v1/siret/search`
    pub async fn search_siret(
        &self,
        params: &SearchSiretParams,
        headers: Option<&AfnorRequestHeaders>,
    ) -> ClientResult<SearchResponse<AfnorSiret>> {
        let url = format!("{}/v1/siret/search", self.config.base_url);
        let auth = self.auth_header().await?;

        let builder = self
            .http
            .post(&url)
            .header("Authorization", auth)
            .json(params);

        let builder = self.apply_headers(builder, headers);

        let response = builder.send().await?;

        self.handle_response(response, "afnor_search_siret").await
    }

    // ============================================================
    // Routing codes — GET + POST search + POST create
    // ============================================================

    /// Recupere un code de routage specifique par SIRET et identifiant.
    /// Correspond a `GET /v1/routing-code/siret:{siret}/code:{routing_id}`
    pub async fn get_routing_code(
        &self,
        siret: &str,
        routing_id: &str,
        headers: Option<&AfnorRequestHeaders>,
    ) -> ClientResult<AfnorRoutingCode> {
        let url = format!(
            "{}/v1/routing-code/siret:{}/code:{}",
            self.config.base_url, siret, routing_id
        );
        let auth = self.auth_header().await?;

        let builder = self
            .http
            .get(&url)
            .header("Authorization", auth);

        let builder = self.apply_headers(builder, headers);

        let response = builder.send().await?;

        self.handle_response(response, "afnor_get_routing_code").await
    }

    /// Recherche de codes de routage AFNOR.
    /// Correspond a `POST /v1/routing-code/search`
    pub async fn search_routing_codes(
        &self,
        criteria: &serde_json::Value,
        headers: Option<&AfnorRequestHeaders>,
    ) -> ClientResult<SearchResponse<AfnorRoutingCode>> {
        let url = format!("{}/v1/routing-code/search", self.config.base_url);
        let auth = self.auth_header().await?;

        let builder = self
            .http
            .post(&url)
            .header("Authorization", auth)
            .json(criteria);

        let builder = self.apply_headers(builder, headers);

        let response = builder.send().await?;

        self.handle_response(response, "afnor_search_routing_codes")
            .await
    }

    /// Cree un code de routage AFNOR.
    /// Correspond a `POST /v1/routing-code`
    pub async fn create_routing_code(
        &self,
        routing_code: &AfnorRoutingCode,
        headers: Option<&AfnorRequestHeaders>,
    ) -> ClientResult<AfnorRoutingCode> {
        let url = format!("{}/v1/routing-code", self.config.base_url);
        let auth = self.auth_header().await?;

        let builder = self
            .http
            .post(&url)
            .header("Authorization", auth)
            .json(routing_code);

        let builder = self.apply_headers(builder, headers);

        let response = builder.send().await?;

        self.handle_response(response, "afnor_create_routing_code")
            .await
    }

    // ============================================================
    // Directory lines — GET + POST search + POST create
    // ============================================================

    /// Recupere une ligne d'annuaire par son identifiant d'adressage.
    /// Correspond a `GET /v1/directory-line/code:{addressing_id}`
    pub async fn get_directory_line(
        &self,
        addressing_id: &str,
        headers: Option<&AfnorRequestHeaders>,
    ) -> ClientResult<AfnorDirectoryLine> {
        let url = format!(
            "{}/v1/directory-line/code:{}",
            self.config.base_url, addressing_id
        );
        let auth = self.auth_header().await?;

        let builder = self
            .http
            .get(&url)
            .header("Authorization", auth);

        let builder = self.apply_headers(builder, headers);

        let response = builder.send().await?;

        self.handle_response(response, "afnor_get_directory_line")
            .await
    }

    /// Recherche de lignes d'annuaire AFNOR.
    /// Correspond a `POST /v1/directory-line/search`
    pub async fn search_directory_lines(
        &self,
        criteria: &serde_json::Value,
        headers: Option<&AfnorRequestHeaders>,
    ) -> ClientResult<SearchResponse<AfnorDirectoryLine>> {
        let url = format!("{}/v1/directory-line/search", self.config.base_url);
        let auth = self.auth_header().await?;

        let builder = self
            .http
            .post(&url)
            .header("Authorization", auth)
            .json(criteria);

        let builder = self.apply_headers(builder, headers);

        let response = builder.send().await?;

        self.handle_response(response, "afnor_search_directory_lines")
            .await
    }

    /// Cree une ligne d'annuaire AFNOR.
    /// Correspond a `POST /v1/directory-line`
    pub async fn create_directory_line(
        &self,
        line: &AfnorDirectoryLine,
        headers: Option<&AfnorRequestHeaders>,
    ) -> ClientResult<AfnorDirectoryLine> {
        let url = format!("{}/v1/directory-line", self.config.base_url);
        let auth = self.auth_header().await?;

        let builder = self
            .http
            .post(&url)
            .header("Authorization", auth)
            .json(line);

        let builder = self.apply_headers(builder, headers);

        let response = builder.send().await?;

        self.handle_response(response, "afnor_create_directory_line")
            .await
    }

    // ============================================================
    // Health check — GET /v1/healthcheck
    // ============================================================

    /// Verifie l'etat du Directory Service.
    /// Correspond a `GET /v1/healthcheck`
    pub async fn healthcheck(&self) -> ClientResult<HealthCheckResponse> {
        let url = format!("{}/v1/healthcheck", self.config.base_url);

        let response = self.http.get(&url).send().await?;

        self.handle_response(response, "directory_healthcheck").await
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
