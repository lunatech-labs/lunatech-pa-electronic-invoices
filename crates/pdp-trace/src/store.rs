use base64::Engine as _;
use pdp_core::error::{PdpError, PdpResult};
use pdp_core::exchange::Exchange;
use pdp_core::model::{FlowEvent, FlowStatus};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Préfixe des index Elasticsearch (un index par SIREN)
const INDEX_PREFIX: &str = "pdp";

/// Index par défaut pour les flux sans SIREN identifié
const DEFAULT_INDEX: &str = "pdp-unknown";

/// Store de traçabilité : persiste les factures, PDF et événements dans Elasticsearch.
///
/// Architecture : un index par numéro SIREN (endpoint = client = SIREN).
/// - Index `pdp-{siren}` contient tous les documents de ce client
/// - Chaque document contient : métadonnées facture + XML brut + PDF base64 + événements
pub struct TraceStore {
    client: Client,
    base_url: String,
}

/// Document Elasticsearch pour un exchange (facture traitée)
#[derive(Debug, Serialize, Deserialize)]
pub struct ExchangeDocument {
    pub exchange_id: String,
    pub flow_id: String,
    pub source_filename: Option<String>,
    pub invoice_number: Option<String>,
    pub invoice_key: Option<String>,
    pub seller_name: Option<String>,
    pub buyer_name: Option<String>,
    pub seller_siret: Option<String>,
    pub buyer_siret: Option<String>,
    pub seller_siren: Option<String>,
    pub buyer_siren: Option<String>,
    pub source_format: Option<String>,
    pub total_ht: Option<f64>,
    pub total_ttc: Option<f64>,
    pub total_tax: Option<f64>,
    pub currency: Option<String>,
    pub issue_date: Option<String>,
    pub status: String,
    pub error_count: i32,
    /// XML brut de la facture (stocké tel quel, searchable)
    pub raw_xml: Option<String>,
    /// PDF en base64 (Factur-X ou PDF visuel)
    pub raw_pdf_base64: Option<String>,
    pub attachment_count: usize,
    pub attachment_filenames: Vec<String>,
    pub events: Vec<EventEntry>,
    pub errors: Vec<ErrorEntry>,
    pub validation_warnings: Vec<WarningEntry>,
    pub created_at: String,
    pub updated_at: String,
}

/// Entrée d'événement dans le document ES
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EventEntry {
    pub id: String,
    pub route_id: String,
    pub status: String,
    pub message: String,
    pub error_detail: Option<String>,
    pub timestamp: String,
}

/// Entrée d'erreur dans le document ES
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ErrorEntry {
    pub step: String,
    pub message: String,
    pub detail: Option<String>,
    pub timestamp: String,
}

/// Entrée de warning de validation dans le document ES (pour audit)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WarningEntry {
    pub rule_id: String,
    pub level: String,
    pub message: String,
    pub source: String,
}

/// Statistiques globales
#[derive(Debug)]
pub struct TraceStats {
    pub total_exchanges: i64,
    pub total_errors: i64,
    pub total_distributed: i64,
}

/// Résumé d'un exchange (pour les listes)
#[derive(Debug, Serialize, Deserialize)]
pub struct ExchangeSummary {
    pub exchange_id: String,
    pub flow_id: String,
    pub source_filename: Option<String>,
    pub invoice_number: Option<String>,
    pub seller_name: Option<String>,
    pub buyer_name: Option<String>,
    pub status: String,
    pub error_count: i32,
    pub created_at: String,
}

impl TraceStore {
    /// Crée un nouveau store connecté à Elasticsearch
    pub async fn new(elasticsearch_url: &str) -> PdpResult<Self> {
        let client = Client::new();
        let base_url = elasticsearch_url.trim_end_matches('/').to_string();

        // Vérifier la connexion
        client.get(&base_url).send().await
            .map_err(|e| PdpError::TraceError(format!("Ping Elasticsearch échoué: {}", e)))?;

        Ok(Self { client, base_url })
    }

    /// Crée un store pour les tests
    pub async fn for_test() -> PdpResult<Self> {
        let url = std::env::var("ELASTICSEARCH_URL")
            .unwrap_or_else(|_| "http://localhost:9200".to_string());
        Self::new(&url).await
    }

    /// Retourne le nom d'index pour un SIREN donné
    pub fn index_name(siren: &str) -> String {
        let clean = siren.chars().filter(|c| c.is_ascii_digit()).collect::<String>();
        if clean.len() >= 9 {
            format!("{}-{}", INDEX_PREFIX, &clean[..9])
        } else if !clean.is_empty() {
            format!("{}-{}", INDEX_PREFIX, clean)
        } else {
            DEFAULT_INDEX.to_string()
        }
    }

    /// Extrait le SIREN depuis un SIRET (9 premiers chiffres)
    pub fn siren_from_siret(siret: &str) -> Option<String> {
        let digits: String = siret.chars().filter(|c| c.is_ascii_digit()).collect();
        if digits.len() >= 9 {
            Some(digits[..9].to_string())
        } else {
            None
        }
    }

    /// Détermine l'index cible pour un exchange (basé sur le SIREN vendeur)
    fn index_for_exchange(exchange: &Exchange) -> String {
        exchange.invoice.as_ref()
            .and_then(|i| i.seller_siret.as_deref())
            .and_then(Self::siren_from_siret)
            .map(|s| Self::index_name(&s))
            .unwrap_or_else(|| DEFAULT_INDEX.to_string())
    }

    /// Crée l'index avec le mapping si nécessaire
    async fn ensure_index(&self, index: &str) -> PdpResult<()> {
        let resp = self.client
            .head(&format!("{}/{}", self.base_url, index))
            .send()
            .await
            .map_err(|e| PdpError::TraceError(format!("Vérification index échouée: {}", e)))?;

        if resp.status().is_success() {
            return Ok(());
        }

        let mapping = serde_json::json!({
            "settings": {
                "number_of_shards": 1,
                "number_of_replicas": 0
            },
            "mappings": {
                "properties": {
                    "exchange_id": { "type": "keyword" },
                    "flow_id": { "type": "keyword" },
                    "source_filename": { "type": "keyword" },
                    "invoice_number": { "type": "keyword" },
                    "invoice_key": { "type": "keyword" },
                    "seller_name": { "type": "text", "fields": { "keyword": { "type": "keyword" } } },
                    "buyer_name": { "type": "text", "fields": { "keyword": { "type": "keyword" } } },
                    "seller_siret": { "type": "keyword" },
                    "buyer_siret": { "type": "keyword" },
                    "seller_siren": { "type": "keyword" },
                    "buyer_siren": { "type": "keyword" },
                    "source_format": { "type": "keyword" },
                    "total_ht": { "type": "double" },
                    "total_ttc": { "type": "double" },
                    "total_tax": { "type": "double" },
                    "currency": { "type": "keyword" },
                    "issue_date": { "type": "date", "format": "yyyy-MM-dd||strict_date_optional_time" },
                    "status": { "type": "keyword" },
                    "error_count": { "type": "integer" },
                    "raw_xml": { "type": "text", "index": true },
                    "raw_pdf_base64": { "type": "binary" },
                    "attachment_count": { "type": "integer" },
                    "attachment_filenames": { "type": "keyword" },
                    "events": {
                        "type": "nested",
                        "properties": {
                            "id": { "type": "keyword" },
                            "route_id": { "type": "keyword" },
                            "status": { "type": "keyword" },
                            "message": { "type": "text" },
                            "error_detail": { "type": "text" },
                            "timestamp": { "type": "date" }
                        }
                    },
                    "errors": {
                        "type": "nested",
                        "properties": {
                            "step": { "type": "keyword" },
                            "message": { "type": "text" },
                            "detail": { "type": "text" },
                            "timestamp": { "type": "date" }
                        }
                    },
                    "validation_warnings": {
                        "type": "nested",
                        "properties": {
                            "rule_id": { "type": "keyword" },
                            "level": { "type": "keyword" },
                            "message": { "type": "text" },
                            "source": { "type": "keyword" }
                        }
                    },
                    "created_at": { "type": "date" },
                    "updated_at": { "type": "date" }
                }
            }
        });

        let resp = self.client
            .put(&format!("{}/{}", self.base_url, index))
            .json(&mapping)
            .send()
            .await
            .map_err(|e| PdpError::TraceError(format!("Création index '{}' échouée: {}", index, e)))?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            // Ignore "resource_already_exists_exception" (race condition)
            if !body.contains("resource_already_exists") {
                return Err(PdpError::TraceError(format!("Création index '{}' échouée: {}", index, body)));
            }
        }

        tracing::info!(index = index, "Index Elasticsearch créé");
        Ok(())
    }

    /// Construit un ExchangeDocument depuis un Exchange
    fn build_document(exchange: &Exchange) -> ExchangeDocument {
        let invoice = exchange.invoice.as_ref();
        let seller_siren = invoice
            .and_then(|i| i.seller_siret.as_deref())
            .and_then(Self::siren_from_siret);
        let buyer_siren = invoice
            .and_then(|i| i.buyer_siret.as_deref())
            .and_then(Self::siren_from_siret);

        let raw_xml = invoice.and_then(|i| i.raw_xml.clone());
        let raw_pdf_base64 = invoice
            .and_then(|i| i.raw_pdf.as_ref())
            .map(|pdf| base64::engine::general_purpose::STANDARD.encode(pdf));

        let attachment_filenames: Vec<String> = invoice
            .map(|i| {
                i.attachments.iter()
                    .filter_map(|a| a.filename.clone())
                    .collect()
            })
            .unwrap_or_default();

        let errors: Vec<ErrorEntry> = exchange.errors.iter().map(|e| ErrorEntry {
            step: e.step.clone(),
            message: e.message.clone(),
            detail: e.detail.clone(),
            timestamp: e.timestamp.to_rfc3339(),
        }).collect();

        // Extraire les warnings de validation depuis la property JSON pour audit
        let validation_warnings: Vec<WarningEntry> = exchange
            .get_property("validation.xml.issues")
            .and_then(|json| serde_json::from_str::<Vec<serde_json::Value>>(&json).ok())
            .map(|issues| {
                issues.iter()
                    .filter(|i| {
                        let level = i.get("level").and_then(|l| l.as_str()).unwrap_or("");
                        level == "Warning" || level == "Info"
                    })
                    .map(|i| WarningEntry {
                        rule_id: i.get("rule_id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                        level: i.get("level").and_then(|v| v.as_str()).unwrap_or("Warning").to_string(),
                        message: i.get("message").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                        source: i.get("source").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    })
                    .collect()
            })
            .unwrap_or_default();

        ExchangeDocument {
            exchange_id: exchange.id.to_string(),
            flow_id: exchange.flow_id.to_string(),
            source_filename: exchange.source_filename.clone(),
            invoice_number: invoice.map(|i| i.invoice_number.clone()),
            invoice_key: invoice.map(|i| i.key_string()),
            seller_name: invoice.and_then(|i| i.seller_name.clone()),
            buyer_name: invoice.and_then(|i| i.buyer_name.clone()),
            seller_siret: invoice.and_then(|i| i.seller_siret.clone()),
            buyer_siret: invoice.and_then(|i| i.buyer_siret.clone()),
            seller_siren,
            buyer_siren,
            source_format: invoice.map(|i| i.source_format.to_string()),
            total_ht: invoice.and_then(|i| i.total_ht),
            total_ttc: invoice.and_then(|i| i.total_ttc),
            total_tax: invoice.and_then(|i| i.total_tax),
            currency: invoice.and_then(|i| i.currency.clone()),
            issue_date: invoice.and_then(|i| i.issue_date.clone()),
            status: exchange.status.to_string(),
            error_count: exchange.errors.len() as i32,
            raw_xml,
            raw_pdf_base64,
            attachment_count: attachment_filenames.len(),
            attachment_filenames,
            events: Vec::new(),
            errors,
            validation_warnings,
            created_at: exchange.created_at.to_rfc3339(),
            updated_at: exchange.updated_at.to_rfc3339(),
        }
    }

    /// Enregistre un exchange complet (facture + XML + PDF + métadonnées)
    pub async fn record_exchange(&self, exchange: &Exchange) -> PdpResult<()> {
        let index = Self::index_for_exchange(exchange);
        self.ensure_index(&index).await?;

        let doc = Self::build_document(exchange);
        let doc_id = doc.exchange_id.clone();

        let resp = self.client
            .put(&format!("{}/{}/_doc/{}", self.base_url, index, doc_id))
            .json(&doc)
            .send()
            .await
            .map_err(|e| PdpError::TraceError(format!("Indexation exchange échouée: {}", e)))?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(PdpError::TraceError(format!("Indexation exchange échouée: {}", body)));
        }

        tracing::debug!(
            exchange_id = %exchange.id,
            index = %index,
            "Exchange indexé dans Elasticsearch"
        );

        Ok(())
    }

    /// Enregistre un événement de flux (ajouté au document exchange existant via update)
    pub async fn record_event(&self, event: &FlowEvent) -> PdpResult<()> {
        let entry = EventEntry {
            id: event.id.to_string(),
            route_id: event.route_id.clone(),
            status: event.status.to_string(),
            message: event.message.clone(),
            error_detail: event.error_detail.clone(),
            timestamp: event.timestamp.to_rfc3339(),
        };

        // Chercher le document par flow_id dans tous les index pdp-*
        let search_body = serde_json::json!({
            "query": { "term": { "flow_id": event.flow_id.to_string() } },
            "size": 1,
            "_source": false
        });

        let search_resp = self.client
            .post(&format!("{}/{}-*/_search", self.base_url, INDEX_PREFIX))
            .json(&search_body)
            .send()
            .await;

        if let Ok(resp) = search_resp {
            if let Ok(body) = resp.json::<serde_json::Value>().await {
                if let Some(hit) = body["hits"]["hits"].as_array().and_then(|a| a.first()) {
                    let index = hit["_index"].as_str().unwrap_or(DEFAULT_INDEX);
                    let doc_id = hit["_id"].as_str().unwrap_or("");

                    if !doc_id.is_empty() {
                        let update_body = serde_json::json!({
                            "script": {
                                "source": "ctx._source.events.add(params.event); ctx._source.status = params.status; ctx._source.updated_at = params.now",
                                "params": {
                                    "event": entry,
                                    "status": event.status.to_string(),
                                    "now": chrono::Utc::now().to_rfc3339()
                                }
                            }
                        });

                        self.client
                            .post(&format!("{}/{}/_update/{}", self.base_url, index, doc_id))
                            .json(&update_body)
                            .send()
                            .await
                            .map_err(|e| PdpError::TraceError(
                                format!("Mise à jour événement échouée: {}", e)
                            ))?;

                        return Ok(());
                    }
                }
            }
        }

        // Si pas de document trouvé, créer un document minimal dans l'index par défaut
        self.ensure_index(DEFAULT_INDEX).await?;
        let doc = serde_json::json!({
            "exchange_id": Uuid::new_v4().to_string(),
            "flow_id": event.flow_id.to_string(),
            "status": event.status.to_string(),
            "error_count": 0,
            "events": [entry],
            "errors": [],
            "attachment_filenames": [],
            "attachment_count": 0,
            "created_at": chrono::Utc::now().to_rfc3339(),
            "updated_at": chrono::Utc::now().to_rfc3339()
        });

        self.client
            .post(&format!("{}/{}/_doc", self.base_url, DEFAULT_INDEX))
            .json(&doc)
            .send()
            .await
            .map_err(|e| PdpError::TraceError(format!("Indexation événement échouée: {}", e)))?;

        Ok(())
    }

    /// Récupère tous les événements d'un flux
    pub async fn get_flow_events(&self, flow_id: Uuid) -> PdpResult<Vec<FlowEvent>> {
        let search_body = serde_json::json!({
            "query": { "term": { "flow_id": flow_id.to_string() } },
            "size": 1
        });

        let resp = self.client
            .post(&format!("{}/{}-*/_search", self.base_url, INDEX_PREFIX))
            .json(&search_body)
            .send()
            .await
            .map_err(|e| PdpError::TraceError(format!("Recherche événements échouée: {}", e)))?;

        let body: serde_json::Value = resp.json().await
            .map_err(|e| PdpError::TraceError(format!("Parse réponse ES échouée: {}", e)))?;

        let mut events = Vec::new();
        if let Some(hits) = body["hits"]["hits"].as_array() {
            for hit in hits {
                if let Some(source) = hit.get("_source") {
                    if let Some(entries) = source["events"].as_array() {
                        for entry in entries {
                            if let Ok(e) = serde_json::from_value::<EventEntry>(entry.clone()) {
                                events.push(FlowEvent {
                                    id: e.id.parse().unwrap_or_else(|_| Uuid::new_v4()),
                                    flow_id,
                                    invoice_key: source["invoice_key"].as_str().map(|s| s.to_string()),
                                    route_id: e.route_id,
                                    status: parse_status(&e.status),
                                    message: e.message,
                                    error_detail: e.error_detail,
                                    timestamp: chrono::DateTime::parse_from_rfc3339(&e.timestamp)
                                        .map(|dt| dt.with_timezone(&chrono::Utc))
                                        .unwrap_or_else(|_| chrono::Utc::now()),
                                });
                            }
                        }
                    }
                }
            }
        }

        events.sort_by_key(|e| e.timestamp);
        Ok(events)
    }

    /// Récupère les flux en erreur (tous les index)
    pub async fn get_error_flows(&self) -> PdpResult<Vec<ExchangeSummary>> {
        let search_body = serde_json::json!({
            "query": {
                "bool": {
                    "should": [
                        { "term": { "status": "ERREUR" } },
                        { "range": { "error_count": { "gt": 0 } } }
                    ],
                    "minimum_should_match": 1
                }
            },
            "sort": [{ "created_at": "desc" }],
            "size": 100
        });

        let resp = self.client
            .post(&format!("{}/{}-*/_search", self.base_url, INDEX_PREFIX))
            .json(&search_body)
            .send()
            .await
            .map_err(|e| PdpError::TraceError(format!("Recherche erreurs échouée: {}", e)))?;

        let body: serde_json::Value = resp.json().await
            .map_err(|e| PdpError::TraceError(format!("Parse réponse ES échouée: {}", e)))?;

        Ok(Self::parse_summaries(&body))
    }

    /// Statistiques globales (tous les index pdp-*)
    pub async fn get_stats(&self) -> PdpResult<TraceStats> {
        let pattern = format!("{}-*", INDEX_PREFIX);

        let total = self.count_query(&pattern, serde_json::json!({ "match_all": {} })).await?;
        let errors = self.count_query(&pattern, serde_json::json!({
            "range": { "error_count": { "gt": 0 } }
        })).await?;
        let distributed = self.count_query(&pattern, serde_json::json!({
            "term": { "status": "DISTRIBUÉ" }
        })).await?;

        Ok(TraceStats {
            total_exchanges: total,
            total_errors: errors,
            total_distributed: distributed,
        })
    }

    /// Compte les documents matchant une query
    async fn count_query(&self, index_pattern: &str, query: serde_json::Value) -> PdpResult<i64> {
        let resp = self.client
            .post(&format!("{}/{}/_count", self.base_url, index_pattern))
            .json(&serde_json::json!({ "query": query }))
            .send()
            .await;

        match resp {
            Ok(r) => {
                let body: serde_json::Value = r.json().await
                    .map_err(|e| PdpError::TraceError(format!("Parse count échouée: {}", e)))?;
                Ok(body["count"].as_i64().unwrap_or(0))
            }
            Err(_) => Ok(0), // Index n'existe pas encore
        }
    }

    /// Recherche full-text dans les XML (tous les index ou un SIREN spécifique)
    pub async fn search_xml(&self, query: &str, siren: Option<&str>) -> PdpResult<Vec<ExchangeSummary>> {
        let index = siren
            .map(|s| Self::index_name(s))
            .unwrap_or_else(|| format!("{}-*", INDEX_PREFIX));

        let search_body = serde_json::json!({
            "query": {
                "match": { "raw_xml": query }
            },
            "sort": [{ "created_at": "desc" }],
            "size": 50,
            "_source": ["exchange_id", "flow_id", "source_filename", "invoice_number",
                        "seller_name", "buyer_name", "status", "error_count", "created_at"]
        });

        let resp = self.client
            .post(&format!("{}/{}/_search", self.base_url, index))
            .json(&search_body)
            .send()
            .await
            .map_err(|e| PdpError::TraceError(format!("Recherche XML échouée: {}", e)))?;

        let body: serde_json::Value = resp.json().await
            .map_err(|e| PdpError::TraceError(format!("Parse réponse ES échouée: {}", e)))?;

        Ok(Self::parse_summaries(&body))
    }

    /// Récupère un document complet par exchange_id
    pub async fn get_exchange(&self, exchange_id: &str, siren: Option<&str>) -> PdpResult<Option<ExchangeDocument>> {
        let index = siren
            .map(|s| Self::index_name(s))
            .unwrap_or_else(|| format!("{}-*", INDEX_PREFIX));

        let search_body = serde_json::json!({
            "query": { "term": { "exchange_id": exchange_id } },
            "size": 1
        });

        let resp = self.client
            .post(&format!("{}/{}/_search", self.base_url, index))
            .json(&search_body)
            .send()
            .await
            .map_err(|e| PdpError::TraceError(format!("Recherche exchange échouée: {}", e)))?;

        let body: serde_json::Value = resp.json().await
            .map_err(|e| PdpError::TraceError(format!("Parse réponse ES échouée: {}", e)))?;

        if let Some(hit) = body["hits"]["hits"].as_array().and_then(|a| a.first()) {
            if let Some(source) = hit.get("_source") {
                let doc: ExchangeDocument = serde_json::from_value(source.clone())
                    .map_err(|e| PdpError::TraceError(format!("Désérialisation exchange échouée: {}", e)))?;
                return Ok(Some(doc));
            }
        }

        Ok(None)
    }

    /// Liste tous les index (= tous les SIREN connus)
    pub async fn list_sirens(&self) -> PdpResult<Vec<String>> {
        let resp = self.client
            .get(&format!("{}/_cat/indices/{}-*?format=json", self.base_url, INDEX_PREFIX))
            .send()
            .await
            .map_err(|e| PdpError::TraceError(format!("Liste index échouée: {}", e)))?;

        let body: serde_json::Value = resp.json().await
            .map_err(|e| PdpError::TraceError(format!("Parse liste index échouée: {}", e)))?;

        let mut sirens = Vec::new();
        if let Some(indices) = body.as_array() {
            let prefix = format!("{}-", INDEX_PREFIX);
            for idx in indices {
                if let Some(name) = idx["index"].as_str() {
                    if let Some(siren) = name.strip_prefix(&prefix) {
                        if siren != "unknown" {
                            sirens.push(siren.to_string());
                        }
                    }
                }
            }
        }

        Ok(sirens)
    }

    /// Supprime tous les index pdp-* (pour les tests)
    pub async fn cleanup(&self) -> PdpResult<()> {
        let _ = self.client
            .delete(&format!("{}/{}-*", self.base_url, INDEX_PREFIX))
            .send()
            .await;
        Ok(())
    }

    /// Parse les hits ES en ExchangeSummary
    fn parse_summaries(body: &serde_json::Value) -> Vec<ExchangeSummary> {
        let mut summaries = Vec::new();
        if let Some(hits) = body["hits"]["hits"].as_array() {
            for hit in hits {
                if let Some(source) = hit.get("_source") {
                    summaries.push(ExchangeSummary {
                        exchange_id: source["exchange_id"].as_str().unwrap_or("").to_string(),
                        flow_id: source["flow_id"].as_str().unwrap_or("").to_string(),
                        source_filename: source["source_filename"].as_str().map(|s| s.to_string()),
                        invoice_number: source["invoice_number"].as_str().map(|s| s.to_string()),
                        seller_name: source["seller_name"].as_str().map(|s| s.to_string()),
                        buyer_name: source["buyer_name"].as_str().map(|s| s.to_string()),
                        status: source["status"].as_str().unwrap_or("INCONNU").to_string(),
                        error_count: source["error_count"].as_i64().unwrap_or(0) as i32,
                        created_at: source["created_at"].as_str().unwrap_or("").to_string(),
                    });
                }
            }
        }
        summaries
    }
}

fn parse_status(s: &str) -> FlowStatus {
    match s {
        "REÇU" => FlowStatus::Received,
        "PARSING" => FlowStatus::Parsing,
        "PARSÉ" => FlowStatus::Parsed,
        "VALIDATION" => FlowStatus::Validating,
        "VALIDÉ" => FlowStatus::Validated,
        "TRANSFORMATION" => FlowStatus::Transforming,
        "TRANSFORMÉ" => FlowStatus::Transformed,
        "DISTRIBUTION" => FlowStatus::Distributing,
        "DISTRIBUÉ" => FlowStatus::Distributed,
        "ATTENTE_ACK" => FlowStatus::WaitingAck,
        "ACQUITTÉ" => FlowStatus::Acknowledged,
        "REJETÉ" => FlowStatus::Rejected,
        "ANNULÉ" => FlowStatus::Cancelled,
        _ => FlowStatus::Error,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pdp_core::model::FlowStatus;

    /// Les tests nécessitent une instance Elasticsearch.
    /// Lancer : docker run -d --name pdp-es -p 9200:9200 -e "discovery.type=single-node" -e "xpack.security.enabled=false" elasticsearch:8.15.0
    /// Ou définir ELASTICSEARCH_URL dans l'environnement.

    async fn setup_store() -> Option<TraceStore> {
        match TraceStore::for_test().await {
            Ok(store) => {
                store.cleanup().await.ok();
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                Some(store)
            }
            Err(e) => {
                eprintln!("Elasticsearch non disponible, test ignoré: {}", e);
                None
            }
        }
    }

    #[test]
    fn test_index_name() {
        assert_eq!(TraceStore::index_name("123456789"), "pdp-123456789");
        assert_eq!(TraceStore::index_name("12345678901234"), "pdp-123456789");
        assert_eq!(TraceStore::index_name("123"), "pdp-123");
        assert_eq!(TraceStore::index_name(""), "pdp-unknown");
    }

    #[test]
    fn test_siren_from_siret() {
        assert_eq!(TraceStore::siren_from_siret("12345678901234"), Some("123456789".to_string()));
        assert_eq!(TraceStore::siren_from_siret("123456789"), Some("123456789".to_string()));
        assert_eq!(TraceStore::siren_from_siret("123"), None);
    }

    #[tokio::test]
    async fn test_trace_store_exchange() {
        let Some(store) = setup_store().await else { return };

        let exchange = Exchange::new(b"<Invoice>test</Invoice>".to_vec())
            .with_filename("test.xml");

        store.record_exchange(&exchange).await.expect("Record exchange failed");

        // Attendre l'indexation (ES est near-realtime)
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        let stats = store.get_stats().await.expect("Get stats failed");
        assert!(stats.total_exchanges >= 1, "Doit avoir au moins 1 exchange");
    }

    #[tokio::test]
    async fn test_trace_store_event() {
        let Some(store) = setup_store().await else { return };

        let exchange = Exchange::new(b"<Invoice>test event</Invoice>".to_vec())
            .with_filename("test_event.xml");
        store.record_exchange(&exchange).await.expect("Record exchange failed");

        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        let event = FlowEvent::new(
            exchange.flow_id,
            "test-route",
            FlowStatus::Received,
            "Facture reçue",
        );
        store.record_event(&event).await.expect("Record event failed");

        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        let events = store.get_flow_events(exchange.flow_id).await.expect("Get events failed");
        assert!(!events.is_empty(), "Doit avoir au moins 1 événement");
    }

    #[tokio::test]
    async fn test_trace_store_stats() {
        let Some(store) = setup_store().await else { return };
        let stats = store.get_stats().await.expect("Get stats failed");
        assert_eq!(stats.total_exchanges, 0);
        assert_eq!(stats.total_errors, 0);
        assert_eq!(stats.total_distributed, 0);
    }
}
