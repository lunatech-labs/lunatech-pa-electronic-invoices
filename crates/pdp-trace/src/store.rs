use base64::Engine as _;
use pdp_core::error::{PdpError, PdpResult};
use pdp_core::exchange::Exchange;
use pdp_core::model::{FlowEvent, FlowStatus};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Préfixe des index Elasticsearch (un index par SIREN)
const INDEX_PREFIX: &str = "pdp";

/// Suffixe de l'index par défaut pour les flux sans SIREN identifié
const UNKNOWN_SUFFIX: &str = "unknown";

/// Statuts considérés comme **terminaux réussis** : un flux qui les atteint
/// avec `error_count = 0` est considéré "OK" pour le dashboard et le filtre UI.
/// Les flux démo ne dépassent souvent pas `VALIDÉ` (pas de PDP destinataire
/// qui acquitte), il serait donc trompeur de ne compter que `DISTRIBUÉ`.
pub const TERMINAL_OK_STATUSES: &[&str] = &[
    "VALIDÉ",
    "TRANSFORMÉ",
    "DISTRIBUTION",
    "DISTRIBUÉ",
    "ATTENTE_ACK",
    "ACQUITTÉ",
];

/// Statuts considérés comme **terminaux d'échec** : combinés avec
/// `error_count > 0` pour la catégorie "Erreur" du dashboard et de la liste.
pub const TERMINAL_FAIL_STATUSES: &[&str] = &["REJETÉ", "ANNULÉ", "ERREUR"];

/// Store de traçabilité : persiste les factures, PDF et événements dans Elasticsearch.
///
/// Architecture : un index par numéro SIREN (endpoint = client = SIREN).
/// - Index `{prefix}-{siren}` contient tous les documents de ce client
/// - Chaque document contient : métadonnées facture + XML brut + PDF base64 + événements
///
/// Le préfixe est `pdp` en production. Les tests peuvent utiliser un préfixe
/// distinct (via [`TraceStore::for_test`]) pour ne pas polluer / wiper les
/// indices de la démo qui partagent le même Elasticsearch.
pub struct TraceStore {
    client: Client,
    base_url: String,
    index_prefix: String,
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
    /// Code statut AFNOR du cycle de vie facture (XP Z12-012, codes 200-501)
    /// porté par le dernier CDV reçu pour cette facture. Plus granulaire
    /// que `status` (FlowStatus) qui collapse 204/205/212 sur Acknowledged.
    /// Présent uniquement après réception d'un CDV traitement (TypeCode 23).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cdv_status_code: Option<u16>,
    /// Horodatage RFC3339 du moment où la PDP a reçu/enregistré le
    /// `cdv_status_code`. Renseigné par `CdvReceptionProcessor` (CDV reçus
    /// d'acteurs externes) ou par le seed démo (statuts simulés via
    /// `_update_by_query`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cdv_received_at: Option<String>,
    /// XML brut de la facture (stocké tel quel, searchable).
    /// Contient toujours le document **original** tel que reçu, indépendamment
    /// d'une éventuelle transformation effectuée par la PDP.
    pub raw_xml: Option<String>,
    /// PDF en base64 (Factur-X ou PDF visuel)
    pub raw_pdf_base64: Option<String>,
    /// XML **converti** par la PDP réceptrice (si une transformation UBL↔CII a eu lieu).
    /// Renseigné automatiquement quand l'exchange porte le header `transform.target`.
    /// Permet de servir `GET /v1/flows/{flowId}?docType=Converted` (XP Z12-013 §6.1.3).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub converted_xml: Option<String>,
    /// Format de la conversion (`UBL`, `CII`, `Factur-X`) si `converted_xml` est présent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub converted_format: Option<String>,
    /// XML du CDV (Compte-rendu De Vie) **généré par notre PDP** lors du dépôt :
    /// CDV 200 Déposée, 213 Rejetée, 221 ERREUR_ROUTAGE, 202 Reçue ou 501 Irrecevable.
    /// Distinct de `cdv_status_code` (qui ne porte que les CDV REÇUS d'acteurs
    /// externes : 204/205/210/212…). Ce champ permet de servir le CDV via l'UI
    /// et d'avoir l'XML auditable (XP Z12-012 §A.1).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub generated_cdv_xml: Option<String>,
    /// Code statut AFNOR du CDV généré par notre PDP (200/202/213/221/501).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub generated_cdv_status_code: Option<u16>,
    /// Horodatage RFC3339 du moment où le CDV "generated" a été produit
    /// (capté par CdarProcessor / IrrecevabiliteProcessor via la propriété
    /// `cdv.generated_at`). Affiché dans la timeline pour montrer quand
    /// la PDP a émis le CDV 200/202/213/221/501.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub generated_cdv_at: Option<String>,
    /// XML du CDV 203 (Mise à disposition) — émis APRÈS un 202 Reçue par
    /// `CdvDispositionProcessor`, quand la facture a été écrite vers la
    /// destination buyer. Slot séparé pour préserver le 202 d'origine
    /// dans `generated_cdv_xml`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disposition_cdv_xml: Option<String>,
    /// Code statut du CDV de mise à disposition (toujours 203 si présent).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disposition_cdv_status_code: Option<u16>,
    /// Horodatage RFC3339 du CDV 203 Mise à disposition (capté par
    /// `CdvDispositionProcessor` via la propriété `cdv.disposition.generated_at`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disposition_cdv_at: Option<String>,
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

/// Séries temporelles quotidiennes utilisées par les sparklines du dashboard.
/// Chaque Vec<i64> est de longueur `days` (alignée à droite : index 0 = jour le
/// plus ancien, dernier index = aujourd'hui).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyBreakdown {
    pub total: Vec<i64>,
    pub distributed: Vec<i64>,
    pub pending: Vec<i64>,
    pub errors: Vec<i64>,
}

impl DailyBreakdown {
    /// Constructeur "vide" — utile en fallback quand ES ne répond pas.
    pub fn zeros(days: usize) -> Self {
        Self {
            total: vec![0; days],
            distributed: vec![0; days],
            pending: vec![0; days],
            errors: vec![0; days],
        }
    }
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
    #[serde(default)]
    pub seller_siret: Option<String>,
    #[serde(default)]
    pub buyer_siret: Option<String>,
    #[serde(default)]
    pub seller_siren: Option<String>,
    #[serde(default)]
    pub buyer_siren: Option<String>,
    pub status: String,
    pub error_count: i32,
    pub created_at: String,
    /// Nombre de pièces jointes (BG-24) — utile pour les listes UI sans
    /// avoir à re-charger le document complet.
    #[serde(default)]
    pub attachment_count: usize,
    /// Code statut AFNOR du cycle de vie facture (XP Z12-012, codes 200-501)
    /// si un CDV a été reçu — sinon `None` (le statut affiché retombe sur
    /// `status` mappé vers AFNOR via `FlowStatus`).
    #[serde(default)]
    pub cdv_status_code: Option<u16>,
}

impl TraceStore {
    /// Crée un nouveau store connecté à Elasticsearch
    pub async fn new(elasticsearch_url: &str) -> PdpResult<Self> {
        Self::new_with_prefix(elasticsearch_url, INDEX_PREFIX).await
    }

    /// Crée un store avec un préfixe d'index personnalisé.
    /// Utilisé par les tests pour s'isoler des indices de production.
    pub async fn new_with_prefix(elasticsearch_url: &str, prefix: &str) -> PdpResult<Self> {
        let client = Client::new();
        let base_url = elasticsearch_url.trim_end_matches('/').to_string();

        // Vérifier la connexion
        client.get(&base_url).send().await
            .map_err(|e| PdpError::TraceError(format!("Ping Elasticsearch échoué: {}", e)))?;

        Ok(Self { client, base_url, index_prefix: prefix.to_string() })
    }

    /// Crée un store no-op (Elasticsearch indisponible).
    /// Les appels d'écriture sont silencieusement ignorés.
    pub fn noop() -> Self {
        Self {
            client: Client::new(),
            base_url: String::new(),
            index_prefix: INDEX_PREFIX.to_string(),
        }
    }

    /// Retourne true si le store est connecté (pas no-op)
    pub fn is_connected(&self) -> bool {
        !self.base_url.is_empty()
    }

    /// Crée un store pour les tests, avec un préfixe d'index unique
    /// (`pdp-itest-{uuid}-`) pour ne JAMAIS interférer avec les indices
    /// de la démo (`pdp-{siren}`) qui partagent souvent le même cluster ES.
    ///
    /// Le préfixe `PDP_TEST_INDEX_PREFIX` peut être défini dans l'environnement
    /// pour réutiliser un préfixe stable entre runs (utile pour debug).
    pub async fn for_test() -> PdpResult<Self> {
        let url = std::env::var("ELASTICSEARCH_URL")
            .unwrap_or_else(|_| "http://localhost:9200".to_string());
        let prefix = std::env::var("PDP_TEST_INDEX_PREFIX")
            .unwrap_or_else(|_| format!("pdp-itest-{}", &Uuid::new_v4().to_string()[..8]));
        Self::new_with_prefix(&url, &prefix).await
    }

    /// Retourne le nom d'index pour un SIREN donné
    pub fn index_name(&self, siren: &str) -> String {
        let clean = siren.chars().filter(|c| c.is_ascii_digit()).collect::<String>();
        if clean.len() >= 9 {
            format!("{}-{}", self.index_prefix, &clean[..9])
        } else if !clean.is_empty() {
            format!("{}-{}", self.index_prefix, clean)
        } else {
            self.default_index()
        }
    }

    /// Index "fourre-tout" pour les flux sans SIREN identifié.
    fn default_index(&self) -> String {
        format!("{}-{}", self.index_prefix, UNKNOWN_SUFFIX)
    }

    /// Wildcard couvrant tous les index de ce store (`{prefix}-*`).
    fn index_pattern(&self) -> String {
        format!("{}-*", self.index_prefix)
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
    fn index_for_exchange(&self, exchange: &Exchange) -> String {
        // Pour les exchanges intra-PDP côté réception, on indexe sous le
        // buyer (pas le seller) — la pipeline réception archive la "copie
        // reçue" du flow, qui appartient au tenant acheteur. Cela évite que
        // le snapshot reception écrase celui de l'émission (sous seller),
        // et permet à la liste "Factures reçues" du buyer de pointer
        // directement sur pdp-{buyer_siren}.
        let is_intra_reception = exchange
            .get_header("source.protocol")
            .map(|s| s.as_str()) == Some("intra-pdp");
        let primary_siret = if is_intra_reception {
            exchange.invoice.as_ref().and_then(|i| i.buyer_siret.as_deref())
        } else {
            exchange.invoice.as_ref().and_then(|i| i.seller_siret.as_deref())
        };
        primary_siret
            .and_then(Self::siren_from_siret)
            .map(|s| self.index_name(&s))
            .unwrap_or_else(|| self.default_index())
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
                    "cdv_status_code": { "type": "short" },
                    "raw_xml": { "type": "text", "index": true },
                    "raw_pdf_base64": { "type": "binary" },
                    "converted_xml": { "type": "text", "index": true },
                    "converted_format": { "type": "keyword" },
                    "generated_cdv_xml": { "type": "text", "index": false },
                    "generated_cdv_status_code": { "type": "short" },
                    "generated_cdv_at": { "type": "date" },
                    "cdv_received_at": { "type": "date" },
                    "disposition_cdv_xml": { "type": "text", "index": false },
                    "disposition_cdv_status_code": { "type": "short" },
                    "disposition_cdv_at": { "type": "date" },
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

        // Si une transformation a eu lieu (TransformProcessor pose le header
        // `transform.target`), capturer le contenu converti depuis `exchange.body`
        // pour pouvoir le servir via `GET /v1/flows/{flowId}?docType=Converted`.
        let (converted_xml, converted_format) = match (
            exchange.get_header("transform.target"),
            std::str::from_utf8(&exchange.body),
        ) {
            (Some(target), Ok(body_str)) if !body_str.is_empty() => {
                // Ne stocker que si le body est différent du raw_xml original
                // (sinon TransformProcessor a court-circuité car formats identiques).
                let is_distinct = raw_xml.as_deref() != Some(body_str);
                if is_distinct {
                    (Some(body_str.to_string()), Some(target.clone()))
                } else {
                    (None, None)
                }
            }
            _ => (None, None),
        };

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
            // `cdv.status_code` est posé par CdarProcessor avec le code du CDV
            // que LA PDP vient de générer (200/202 systématiquement) — ce
            // n'est PAS le statut métier réel de la facture. On capture
            // uniquement les CDV reçus d'acteurs externes (acheteur via
            // CdvReceptionProcessor, qui pose `cdv.received=true`) ou les
            // statuts simulés par la démo (POST direct via _update_by_query).
            cdv_status_code: if exchange.get_property("cdv.received").is_some() {
                exchange
                    .get_property("cdv.status_code")
                    .and_then(|s| s.parse::<u16>().ok())
            } else {
                None
            },
            cdv_received_at: if exchange.get_property("cdv.received").is_some() {
                // Préfère la valeur posée par CdvReceptionProcessor, sinon
                // tombe sur le moment d'indexation actuel.
                exchange
                    .get_property("cdv.received_at")
                    .cloned()
                    .or_else(|| Some(chrono::Utc::now().to_rfc3339()))
            } else {
                None
            },
            // CDV généré par notre PDP (200/202/213/221/501) — capté UNIQUEMENT
            // s'il n'a pas été reçu d'un acteur externe. Le `cdv.generated`
            // header (posé par CdarProcessor) atteste qu'on est la source.
            generated_cdv_xml: if exchange.get_header("cdv.generated").is_some()
                && exchange.get_property("cdv.received").is_none()
            {
                exchange.get_property("cdv.xml").cloned()
            } else {
                None
            },
            generated_cdv_status_code: if exchange.get_header("cdv.generated").is_some()
                && exchange.get_property("cdv.received").is_none()
            {
                exchange
                    .get_property("cdv.status_code")
                    .and_then(|s| s.parse::<u16>().ok())
            } else {
                None
            },
            generated_cdv_at: if exchange.get_header("cdv.generated").is_some()
                && exchange.get_property("cdv.received").is_none()
            {
                exchange.get_property("cdv.generated_at").cloned()
            } else {
                None
            },
            // CDV 203 Mise à disposition — capté quand
            // `CdvDispositionProcessor` a posé `cdv.disposition.generated`.
            disposition_cdv_xml: if exchange.get_header("cdv.disposition.generated").is_some() {
                exchange.get_property("cdv.disposition.xml").cloned()
            } else {
                None
            },
            disposition_cdv_status_code: if exchange.get_header("cdv.disposition.generated").is_some() {
                exchange
                    .get_property("cdv.disposition.status_code")
                    .and_then(|s| s.parse::<u16>().ok())
            } else {
                None
            },
            disposition_cdv_at: if exchange.get_header("cdv.disposition.generated").is_some() {
                exchange.get_property("cdv.disposition.generated_at").cloned()
            } else {
                None
            },
            raw_xml,
            raw_pdf_base64,
            converted_xml,
            converted_format,
            attachment_count: attachment_filenames.len(),
            attachment_filenames,
            events: Vec::new(),
            errors,
            validation_warnings,
            created_at: exchange.created_at.to_rfc3339(),
            updated_at: exchange.updated_at.to_rfc3339(),
        }
    }

    /// Enregistre un exchange complet (facture + XML + PDF + métadonnées).
    ///
    /// Deux subtilités importantes :
    ///
    /// **1. Cohérence multi-index** — un même `exchange_id` traverse plusieurs
    /// `record_exchange` dans le pipeline. La première (TraceProcessor::received)
    /// arrive avant le parsing — `seller_siret` est inconnu et le doc est rangé
    /// dans `pdp-unknown`. La seconde (TraceProcessor::parsed) connaît le
    /// vendeur et écrit dans `pdp-{seller_siren}`. Sans nettoyage, on garderait
    /// **deux copies** du même exchange. On supprime donc les copies stale
    /// dans les autres index avant l'upsert (en récupérant d'abord leurs
    /// events/errors pour ne pas perdre la timeline déjà commencée).
    ///
    /// **2. Préservation des arrays `events` / `errors` / `validation_warnings`** —
    /// un PUT complet écraserait les events ajoutés par `record_event`. On
    /// passe donc en `_update` avec `doc_as_upsert` : à la première création,
    /// `upsert` initialise les arrays ; aux mises à jour, `doc` ne contient
    /// que les métadonnées (les arrays restent intactes).
    pub async fn record_exchange(&self, exchange: &Exchange) -> PdpResult<()> {
        let index = self.index_for_exchange(exchange);
        self.ensure_index(&index).await?;

        let doc = Self::build_document(exchange);
        let doc_id = doc.exchange_id.clone();

        // Récupère les events/errors/warnings d'éventuelles copies stale dans
        // d'autres index, puis les supprime. Les arrays récupérées sont
        // injectées dans l'upsert pour conserver la timeline.
        let mut carried_events: Vec<EventEntry> = Vec::new();
        let mut carried_errors: Vec<ErrorEntry> = Vec::new();
        let mut carried_warnings: Vec<WarningEntry> = Vec::new();
        if let Ok(resp) = self
            .client
            .post(&format!(
                "{}/{}/_search",
                self.base_url,
                self.index_pattern()
            ))
            .json(&serde_json::json!({
                "query": {
                    "bool": {
                        "must": [{ "term": { "exchange_id": &doc_id } }],
                        "must_not": [{ "term": { "_index": &index } }]
                    }
                },
                "size": 5,
                "_source": ["events", "errors", "validation_warnings"]
            }))
            .send()
            .await
        {
            if let Ok(body) = resp.json::<serde_json::Value>().await {
                if let Some(hits) = body["hits"]["hits"].as_array() {
                    for hit in hits {
                        if let Some(src) = hit.get("_source") {
                            if let Some(arr) = src.get("events").and_then(|v| v.as_array()) {
                                for e in arr {
                                    if let Ok(ev) = serde_json::from_value::<EventEntry>(e.clone()) {
                                        carried_events.push(ev);
                                    }
                                }
                            }
                            if let Some(arr) = src.get("errors").and_then(|v| v.as_array()) {
                                for e in arr {
                                    if let Ok(er) = serde_json::from_value::<ErrorEntry>(e.clone()) {
                                        carried_errors.push(er);
                                    }
                                }
                            }
                            if let Some(arr) = src
                                .get("validation_warnings")
                                .and_then(|v| v.as_array())
                            {
                                for w in arr {
                                    if let Ok(wn) = serde_json::from_value::<WarningEntry>(w.clone())
                                    {
                                        carried_warnings.push(wn);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        // Supprime les copies stale (autres index)
        let _ = self
            .client
            .post(&format!(
                "{}/{}/_delete_by_query?refresh=true",
                self.base_url,
                self.index_pattern()
            ))
            .json(&serde_json::json!({
                "query": {
                    "bool": {
                        "must": [{ "term": { "exchange_id": &doc_id } }],
                        "must_not": [{ "term": { "_index": &index } }]
                    }
                }
            }))
            .send()
            .await;

        // `doc` (mises à jour) — on retire SEULEMENT `events`, géré par
        // `record_event` via append scripted. `errors` et
        // `validation_warnings` sont alimentés par les processors (via
        // `Exchange::add_error` et la property `validation.xml.issues`)
        // et doivent donc être mis à jour à chaque `record_exchange`,
        // sinon une erreur ajoutée après le premier upsert (ex.
        // AnnuaireValidationProcessor en aval) ne serait jamais reflétée.
        let mut update_doc = serde_json::to_value(&doc)
            .map_err(|e| PdpError::TraceError(format!("Sérialisation doc échouée: {}", e)))?;
        if let Some(obj) = update_doc.as_object_mut() {
            obj.remove("events");
        }
        // `upsert` (première création) avec les arrays initialisées et les
        // events/errors recueillis depuis l'ancien index si applicable.
        let mut upsert_doc = serde_json::to_value(&doc)
            .map_err(|e| PdpError::TraceError(format!("Sérialisation upsert échouée: {}", e)))?;
        if let Some(obj) = upsert_doc.as_object_mut() {
            obj.insert(
                "events".into(),
                serde_json::to_value(&carried_events).unwrap_or_else(|_| serde_json::json!([])),
            );
            obj.insert(
                "errors".into(),
                serde_json::to_value(&carried_errors).unwrap_or_else(|_| serde_json::json!([])),
            );
            obj.insert(
                "validation_warnings".into(),
                serde_json::to_value(&carried_warnings).unwrap_or_else(|_| serde_json::json!([])),
            );
        }
        let body = serde_json::json!({
            "doc": update_doc,
            "upsert": upsert_doc,
        });

        // `refresh=true` : le document est immédiatement searchable, indispensable
        // car `record_event` (appelé juste après dans le pipeline) le recherche
        // par `flow_id` pour append l'événement à `events`. Sans refresh, ES ne
        // le trouve pas pendant ~1s et `record_event` crée un fallback orphelin.
        let resp = self.client
            .post(&format!("{}/{}/_update/{}?refresh=true", self.base_url, index, doc_id))
            .json(&body)
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

        let pattern = self.index_pattern();
        let default_idx = self.default_index();
        let search_resp = self.client
            .post(&format!("{}/{}/_search", self.base_url, pattern))
            .json(&search_body)
            .send()
            .await;

        if let Ok(resp) = search_resp {
            if let Ok(body) = resp.json::<serde_json::Value>().await {
                if let Some(hit) = body["hits"]["hits"].as_array().and_then(|a| a.first()) {
                    let index = hit["_index"].as_str().unwrap_or(&default_idx);
                    let doc_id = hit["_id"].as_str().unwrap_or("");

                    if !doc_id.is_empty() {
                        // Idempotence : on n'ajoute l'événement que si son id
                        // n'est pas déjà présent. Le bus pdp-events garantit
                        // l'unicité d'event.id ; un rejouage at-least-once
                        // ne crée donc pas de doublon dans `events`.
                        let update_body = serde_json::json!({
                            "script": {
                                "source": "if (ctx._source.events == null) { ctx._source.events = []; } \
                                          boolean exists = false; \
                                          for (int i = 0; i < ctx._source.events.size(); i++) { \
                                              if (ctx._source.events[i].id == params.event.id) { exists = true; break; } \
                                          } \
                                          if (!exists) { \
                                              ctx._source.events.add(params.event); \
                                              ctx._source.status = params.status; \
                                              ctx._source.updated_at = params.now; \
                                          }",
                                "params": {
                                    "event": entry,
                                    "status": event.status.to_string(),
                                    "now": chrono::Utc::now().to_rfc3339()
                                }
                            }
                        });

                        self.client
                            .post(&format!("{}/{}/_update/{}?refresh=true", self.base_url, index, doc_id))
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
        self.ensure_index(&default_idx).await?;
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
            .post(&format!("{}/{}/_doc", self.base_url, default_idx))
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
            .post(&format!("{}/{}/_search", self.base_url, self.index_pattern()))
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
            .post(&format!("{}/{}/_search", self.base_url, self.index_pattern()))
            .json(&search_body)
            .send()
            .await
            .map_err(|e| PdpError::TraceError(format!("Recherche erreurs échouée: {}", e)))?;

        let body: serde_json::Value = resp.json().await
            .map_err(|e| PdpError::TraceError(format!("Parse réponse ES échouée: {}", e)))?;

        Ok(Self::parse_summaries(&body))
    }

    /// Statistiques globales (tous les index `{prefix}-*`)
    pub async fn get_stats(&self) -> PdpResult<TraceStats> {
        let pattern = self.index_pattern();

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
            .map(|s| self.index_name(s))
            .unwrap_or_else(|| self.index_pattern());

        let search_body = serde_json::json!({
            "query": {
                "match": { "raw_xml": query }
            },
            "sort": [{ "created_at": "desc" }],
            "size": 50,
            "_source": ["exchange_id", "flow_id", "source_filename", "invoice_number", "attachment_count", "seller_siret", "buyer_siret", "seller_siren", "buyer_siren",
                        "seller_name", "buyer_name", "status", "error_count", "created_at", "cdv_status_code"]
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

    /// Recherche des échanges par clé de facture (SIREN/NUMERO/ANNEE) pour la détection de doublons.
    /// Utilisé par le DuplicateCheckProcessor pour vérifier BR-FR-12 et BR-FR-13.
    pub async fn search_by_invoice_key(&self, invoice_key: &str, siren: Option<&str>) -> PdpResult<Vec<ExchangeSummary>> {
        let index = siren
            .map(|s| self.index_name(s))
            .unwrap_or_else(|| self.index_pattern());

        let search_body = serde_json::json!({
            "query": { "term": { "invoice_key": invoice_key } },
            "sort": [{ "created_at": "desc" }],
            "size": 5,
            "_source": ["exchange_id", "flow_id", "source_filename", "invoice_number", "attachment_count", "seller_siret", "buyer_siret", "seller_siren", "buyer_siren",
                        "seller_name", "buyer_name", "status", "error_count", "created_at", "cdv_status_code"]
        });

        let resp = self.client
            .post(&format!("{}/{}/_search", self.base_url, index))
            .json(&search_body)
            .send()
            .await
            .map_err(|e| PdpError::TraceError(format!("Recherche par invoice_key échouée: {}", e)))?;

        if !resp.status().is_success() {
            // Index n'existe pas encore ou erreur ES — pas de doublon
            return Ok(Vec::new());
        }

        let body: serde_json::Value = resp.json().await
            .map_err(|e| PdpError::TraceError(format!("Parse réponse ES échouée: {}", e)))?;

        Ok(Self::parse_summaries(&body))
    }

    /// Récupère un document complet par exchange_id, en garantissant que le
    /// SIREN demandé correspond effectivement à une partie du flux (vendeur ou
    /// acheteur). Sans ça, un client connaissant un `exchange_id` arbitraire
    /// pourrait lire les factures de n'importe quel tenant.
    ///
    /// Si `siren` est `None` (cas admin / opérateur PDP cross-tenant), la
    /// requête se fait sur tous les index sans filtre. Les handlers HTTP
    /// publics ne doivent **jamais** passer `None` — uniquement les CLI
    /// admin et les tests.
    pub async fn get_exchange(&self, exchange_id: &str, siren: Option<&str>) -> PdpResult<Option<ExchangeDocument>> {
        let index = self.index_pattern();

        let mut must: Vec<serde_json::Value> = vec![
            serde_json::json!({ "term": { "exchange_id": exchange_id } }),
        ];
        if let Some(s) = siren {
            must.push(serde_json::json!({
                "bool": {
                    "should": [
                        { "term": { "seller_siren": s } },
                        { "term": { "buyer_siren": s } }
                    ],
                    "minimum_should_match": 1
                }
            }));
        }
        let search_body = serde_json::json!({
            "query": { "bool": { "must": must } },
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

    /// Construit la query ES utilisée par `list_exchanges` et `count_exchanges`,
    /// en fonction des filtres UI (siren, status, plage de dates, direction).
    ///
    /// `direction` :
    /// - `Some("emises")` → tenant vendeur uniquement (`seller_siren = X`)
    /// - `Some("recues")` → tenant acheteur uniquement (`buyer_siren = X`)
    /// - `None` → les deux (vendeur OU acheteur — comportement legacy)
    fn build_tenant_filter_query(
        &self,
        siren: &str,
        status: Option<&str>,
        from_date: Option<&str>,
        to_date: Option<&str>,
        direction: Option<&str>,
    ) -> serde_json::Value {
        // L'index ES est keyé par seller_siren ; pour qu'un tenant voit *aussi*
        // les factures où il est acheteur (réception), on requête le wildcard
        // `pdp-*` et on précise le rôle attendu côté must.
        let tenant_match = match direction {
            Some("emises") => serde_json::json!({ "term": { "seller_siren": siren } }),
            Some("recues") => serde_json::json!({ "term": { "buyer_siren": siren } }),
            _ => serde_json::json!({
                "bool": {
                    "should": [
                        { "term": { "seller_siren": siren } },
                        { "term": { "buyer_siren": siren } }
                    ],
                    "minimum_should_match": 1
                }
            }),
        };
        let mut must: Vec<serde_json::Value> = vec![tenant_match];
        let mut must_not: Vec<serde_json::Value> = Vec::new();
        // Filtre status logique. Les statuts ES présents sur un flux passent
        // typiquement REÇU → PARSÉ → VALIDÉ → TRANSFORMÉ → DISTRIBUÉ → ACQUITTÉ.
        // Le filtre UI regroupe ces statuts en 3 grandes catégories :
        //  - "ok" : aucun erreur, status terminal réussi (VALIDÉ et au-delà)
        //  - "erreur" : error_count > 0 OU rejetté/annulé
        //  - "attente" : pas encore arrivé à un état terminal et pas en erreur
        //  - autre : term match exact (compatibilité API)
        let terminal_ok = TERMINAL_OK_STATUSES;
        let terminal_fail = TERMINAL_FAIL_STATUSES;
        if let Some(s) = status {
            match s.to_uppercase().as_str() {
                "OK" | "DISTRIBUÉ" | "DISTRIBUE" | "VALIDÉ" | "VALIDE" => {
                    must.push(serde_json::json!({ "term": { "error_count": 0 } }));
                    must.push(serde_json::json!({ "terms": { "status": terminal_ok } }));
                }
                "ERREUR" | "ERROR" => {
                    must.push(serde_json::json!({
                        "bool": {
                            "should": [
                                { "range": { "error_count": { "gt": 0 } } },
                                { "terms": { "status": terminal_fail } }
                            ],
                            "minimum_should_match": 1
                        }
                    }));
                }
                "EN_ATTENTE" | "ATTENTE" | "PENDING" => {
                    must.push(serde_json::json!({ "term": { "error_count": 0 } }));
                    let mut blocked: Vec<&str> = terminal_ok.to_vec();
                    blocked.extend(terminal_fail);
                    must_not.push(serde_json::json!({ "terms": { "status": blocked } }));
                }
                other => {
                    must.push(serde_json::json!({ "term": { "status": other } }));
                }
            }
        }
        let mut range = serde_json::Map::new();
        if let Some(f) = from_date { range.insert("gte".into(), serde_json::Value::String(f.into())); }
        if let Some(t) = to_date { range.insert("lte".into(), serde_json::Value::String(t.into())); }
        if !range.is_empty() {
            must.push(serde_json::json!({ "range": { "issue_date": range } }));
        }
        let mut bool_q = serde_json::Map::new();
        bool_q.insert("must".into(), serde_json::Value::Array(must));
        if !must_not.is_empty() {
            bool_q.insert("must_not".into(), serde_json::Value::Array(must_not));
        }
        serde_json::json!({ "bool": bool_q })
    }

    /// Compte le nombre total d'exchanges d'un tenant, avec les mêmes filtres
    /// que `list_exchanges` mais sans pagination. Utilisé par l'UI pour
    /// afficher le total et le nombre de pages.
    ///
    /// Si `dedup_by_invoice` est `true`, retourne le **nombre de factures
    /// uniques** (cardinalité sur `invoice_number`). Une facture re-soumise
    /// crée plusieurs exchanges (BR-FR-12/13), mais ne doit compter que pour
    /// une seule entrée d'UI. Sinon, retourne le total brut d'exchanges.
    pub async fn count_exchanges(
        &self,
        siren: &str,
        status: Option<&str>,
        from_date: Option<&str>,
        to_date: Option<&str>,
        direction: Option<&str>,
    ) -> PdpResult<i64> {
        self.count_exchanges_with_dedup(siren, status, from_date, to_date, direction, false)
            .await
    }

    /// Variante avec contrôle explicite de la déduplication par
    /// `invoice_number`. Voir [`count_exchanges`].
    pub async fn count_exchanges_with_dedup(
        &self,
        siren: &str,
        status: Option<&str>,
        from_date: Option<&str>,
        to_date: Option<&str>,
        direction: Option<&str>,
        dedup_by_invoice: bool,
    ) -> PdpResult<i64> {
        let index = self.index_pattern();
        let query =
            self.build_tenant_filter_query(siren, status, from_date, to_date, direction);

        if !dedup_by_invoice {
            return self.count_query(&index, query).await;
        }

        // Cardinalité sur invoice_number : compte des numéros distincts.
        // `precision_threshold` à 40000 garantit un compte exact jusqu'à 40k
        // factures uniques par tenant (largement suffisant pour les volumes
        // de la démo et la plupart des PDPs).
        let body = serde_json::json!({
            "query": query,
            "size": 0,
            "aggs": {
                "unique_invoices": {
                    "cardinality": {
                        "field": "invoice_number",
                        "precision_threshold": 40000
                    }
                }
            }
        });
        let resp = self.client
            .post(&format!("{}/{}/_search", self.base_url, index))
            .json(&body)
            .send()
            .await
            .map_err(|e| PdpError::TraceError(format!("Count cardinality échoué: {}", e)))?;
        if !resp.status().is_success() {
            return Ok(0);
        }
        let body: serde_json::Value = resp.json().await
            .map_err(|e| PdpError::TraceError(format!("Parse réponse ES échouée: {}", e)))?;
        Ok(body["aggregations"]["unique_invoices"]["value"]
            .as_i64()
            .unwrap_or(0))
    }

    /// Liste paginée d'exchanges pour un tenant, avec filtres optionnels.
    ///
    /// Filtres :
    /// - `status` : valeur exacte du champ `status` (`DISTRIBUÉ`, `ERREUR`, `REJETE`, ...)
    /// - `from_date` / `to_date` : bornes sur `issue_date` (format `YYYY-MM-DD`)
    ///
    /// Pagination :
    /// - `page` (0-indexed) × `page_size` → offset ES
    pub async fn list_exchanges(
        &self,
        siren: &str,
        status: Option<&str>,
        from_date: Option<&str>,
        to_date: Option<&str>,
        page: usize,
        page_size: usize,
        direction: Option<&str>,
    ) -> PdpResult<Vec<ExchangeSummary>> {
        self.list_exchanges_with_dedup(
            siren, status, from_date, to_date, page, page_size, direction, false,
        )
        .await
    }

    /// Variante avec déduplication par `invoice_number` (collapse ES).
    ///
    /// Quand `dedup_by_invoice=true`, ES regroupe les documents par
    /// `invoice_number` et ne renvoie que le plus récent de chaque groupe.
    /// La pagination opère alors sur les factures uniques, pas sur les
    /// exchanges bruts — ce qui aligne le total (cardinalité) avec le
    /// contenu effectivement affiché.
    ///
    /// Sans dedup, on pagine sur les exchanges bruts (toutes soumissions
    /// incluses) — utile pour `?show_duplicates=true`.
    pub async fn list_exchanges_with_dedup(
        &self,
        siren: &str,
        status: Option<&str>,
        from_date: Option<&str>,
        to_date: Option<&str>,
        page: usize,
        page_size: usize,
        direction: Option<&str>,
        dedup_by_invoice: bool,
    ) -> PdpResult<Vec<ExchangeSummary>> {
        let index = self.index_pattern();
        let query =
            self.build_tenant_filter_query(siren, status, from_date, to_date, direction);

        let from = page * page_size;
        let mut body = serde_json::json!({
            "query": query,
            "from": from,
            "size": page_size,
            "sort": [{ "created_at": "desc" }],
            "_source": ["exchange_id", "flow_id", "source_filename", "invoice_number", "attachment_count", "seller_siret", "buyer_siret", "seller_siren", "buyer_siren",
                        "seller_name", "buyer_name", "status", "error_count", "created_at", "cdv_status_code"]
        });

        if dedup_by_invoice {
            // ES `collapse` : un seul hit par invoice_number (le plus récent
            // grâce au sort created_at desc). Les exchanges sans
            // invoice_number (rare, mais possible si parsing échoué) sont
            // retournés tels quels — ils n'ont pas de groupe à collapser.
            body["collapse"] = serde_json::json!({ "field": "invoice_number" });
        }

        let resp = self.client
            .post(&format!("{}/{}/_search", self.base_url, index))
            .json(&body)
            .send()
            .await
            .map_err(|e| PdpError::TraceError(format!("Liste exchanges échouée: {}", e)))?;

        if !resp.status().is_success() {
            return Ok(Vec::new());
        }
        let body: serde_json::Value = resp.json().await
            .map_err(|e| PdpError::TraceError(format!("Parse réponse ES échouée: {}", e)))?;
        Ok(Self::parse_summaries(&body))
    }

    /// Récupère la raison sociale d'un tenant. On cherche d'abord un document où
    /// le SIREN apparaît comme **vendeur** (le `seller_name` est alors la raison
    /// sociale du tenant) ; sinon on retombe sur un document où il est acheteur,
    /// auquel cas on retourne le `buyer_name`.
    pub async fn get_tenant_name(&self, siren: &str) -> Option<String> {
        let index = self.index_pattern();
        // Tentative 1 : tenant en tant que vendeur → seller_name
        let body = serde_json::json!({
            "query": { "term": { "seller_siren": siren } },
            "size": 1,
            "_source": ["seller_name"],
        });
        let resp = self
            .client
            .post(&format!("{}/{}/_search", self.base_url, index))
            .json(&body)
            .send()
            .await
            .ok()?;
        if resp.status().is_success() {
            if let Ok(body) = resp.json::<serde_json::Value>().await {
                if let Some(name) = body["hits"]["hits"]
                    .as_array()
                    .and_then(|a| a.first())
                    .and_then(|h| h.get("_source"))
                    .and_then(|s| s.get("seller_name"))
                    .and_then(|s| s.as_str())
                {
                    return Some(name.to_string());
                }
            }
        }
        // Tentative 2 : tenant en tant qu'acheteur → buyer_name
        let body = serde_json::json!({
            "query": { "term": { "buyer_siren": siren } },
            "size": 1,
            "_source": ["buyer_name"],
        });
        let resp = self
            .client
            .post(&format!("{}/{}/_search", self.base_url, index))
            .json(&body)
            .send()
            .await
            .ok()?;
        if !resp.status().is_success() {
            return None;
        }
        let body: serde_json::Value = resp.json().await.ok()?;
        body["hits"]["hits"]
            .as_array()?
            .first()?
            .get("_source")?
            .get("buyer_name")?
            .as_str()
            .map(String::from)
    }

    /// Stats par tenant : compte les flux où le tenant est **vendeur OU acheteur**
    /// (donc émissions + réceptions). Requête sur le wildcard `pdp-*` car les
    /// flux reçus sont indexés dans l'index du fournisseur.
    ///
    /// La sémantique des compteurs est alignée sur celle du filtre UI
    /// `list_exchanges` (cf. [`TERMINAL_OK_STATUSES`] / [`TERMINAL_FAIL_STATUSES`])
    /// pour qu'un dashboard "Distribués: 14" corresponde *exactement* à la
    /// liste retournée par `?status=OK`.
    pub async fn get_stats_for_siren(&self, siren: &str) -> PdpResult<TraceStats> {
        let index = self.index_pattern();
        let tenant_match = |extra: serde_json::Value| -> serde_json::Value {
            serde_json::json!({
                "bool": {
                    "must": [
                        {
                            "bool": {
                                "should": [
                                    { "term": { "seller_siren": siren } },
                                    { "term": { "buyer_siren": siren } }
                                ],
                                "minimum_should_match": 1
                            }
                        },
                        extra
                    ]
                }
            })
        };
        let total = self
            .count_query(&index, tenant_match(serde_json::json!({ "match_all": {} })))
            .await?;
        // "Erreurs" = error_count > 0 OU status terminal d'échec
        let errors = self
            .count_query(
                &index,
                tenant_match(serde_json::json!({
                    "bool": {
                        "should": [
                            { "range": { "error_count": { "gt": 0 } } },
                            { "terms": { "status": TERMINAL_FAIL_STATUSES } }
                        ],
                        "minimum_should_match": 1
                    }
                })),
            )
            .await?;
        // "Distribués" (au sens UI/filtre OK) = error_count = 0 ET status terminal OK
        let distributed = self
            .count_query(
                &index,
                tenant_match(serde_json::json!({
                    "bool": {
                        "must": [
                            { "term": { "error_count": 0 } },
                            { "terms": { "status": TERMINAL_OK_STATUSES } }
                        ]
                    }
                })),
            )
            .await?;
        Ok(TraceStats {
            total_exchanges: total,
            total_errors: errors,
            total_distributed: distributed,
        })
    }

    /// Renvoie 4 séries quotidiennes (total / distribués / erreurs / pending)
    /// pour un tenant sur les `days` derniers jours, dans l'ordre chronologique.
    /// Utilisé par les sparklines des 4 KPI du dashboard.
    ///
    /// Une seule requête ES avec un `date_histogram` + 2 `filter` sub-aggregations.
    /// Pending est dérivé côté Rust (`total - distribués - erreurs`).
    pub async fn daily_breakdown_for_siren(
        &self,
        siren: &str,
        days: u32,
    ) -> PdpResult<DailyBreakdown> {
        let index = self.index_pattern();
        let now = chrono::Utc::now();
        let from = now - chrono::Duration::days(days as i64 - 1);
        let body = serde_json::json!({
            "size": 0,
            "query": {
                "bool": {
                    "should": [
                        { "term": { "seller_siren": siren } },
                        { "term": { "buyer_siren": siren } }
                    ],
                    "minimum_should_match": 1
                }
            },
            "aggs": {
                "by_day": {
                    "date_histogram": {
                        "field": "created_at",
                        "calendar_interval": "1d",
                        "min_doc_count": 0,
                        "extended_bounds": {
                            "min": from.format("%Y-%m-%d").to_string(),
                            "max": now.format("%Y-%m-%d").to_string(),
                        },
                        "time_zone": "UTC",
                    },
                    "aggs": {
                        "distributed": {
                            "filter": {
                                "bool": {
                                    "must": [
                                        { "term": { "error_count": 0 } },
                                        { "terms": { "status": TERMINAL_OK_STATUSES } }
                                    ]
                                }
                            }
                        },
                        "errors": {
                            "filter": {
                                "bool": {
                                    "should": [
                                        { "range": { "error_count": { "gt": 0 } } },
                                        { "terms": { "status": TERMINAL_FAIL_STATUSES } }
                                    ],
                                    "minimum_should_match": 1
                                }
                            }
                        }
                    }
                }
            }
        });
        let n = days as usize;
        let resp = self
            .client
            .post(&format!("{}/{}/_search", self.base_url, index))
            .json(&body)
            .send()
            .await
            .map_err(|e| PdpError::TraceError(format!("Aggregation daily_breakdown échouée: {}", e)))?;
        if !resp.status().is_success() {
            return Ok(DailyBreakdown::zeros(n));
        }
        let json: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| PdpError::TraceError(format!("Parse réponse ES échouée: {}", e)))?;
        let buckets = json["aggregations"]["by_day"]["buckets"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        let mut total = vec![0i64; n];
        let mut distributed = vec![0i64; n];
        let mut errors = vec![0i64; n];
        let start = n.saturating_sub(buckets.len());
        for (i, b) in buckets.iter().enumerate() {
            let idx = start + i;
            if idx >= n { break; }
            total[idx] = b["doc_count"].as_i64().unwrap_or(0);
            distributed[idx] = b["distributed"]["doc_count"].as_i64().unwrap_or(0);
            errors[idx] = b["errors"]["doc_count"].as_i64().unwrap_or(0);
        }
        let pending: Vec<i64> = total
            .iter()
            .zip(distributed.iter())
            .zip(errors.iter())
            .map(|((t, d), e)| (t - d - e).max(0))
            .collect();
        Ok(DailyBreakdown { total, distributed, pending, errors })
    }

    /// Variante legacy : ne retourne que la série `total`. Conservée pour
    /// compatibilité, redirige vers `daily_breakdown_for_siren`.
    pub async fn daily_counts_for_siren(
        &self,
        siren: &str,
        days: u32,
    ) -> PdpResult<Vec<i64>> {
        let index = self.index_pattern();
        let now = chrono::Utc::now();
        let from = now - chrono::Duration::days(days as i64 - 1);
        let body = serde_json::json!({
            "size": 0,
            "query": {
                "bool": {
                    "should": [
                        { "term": { "seller_siren": siren } },
                        { "term": { "buyer_siren": siren } }
                    ],
                    "minimum_should_match": 1
                }
            },
            "aggs": {
                "by_day": {
                    "date_histogram": {
                        "field": "created_at",
                        "calendar_interval": "1d",
                        "min_doc_count": 0,
                        "extended_bounds": {
                            "min": from.format("%Y-%m-%d").to_string(),
                            "max": now.format("%Y-%m-%d").to_string(),
                        },
                        "time_zone": "UTC",
                    }
                }
            }
        });
        let resp = self
            .client
            .post(&format!("{}/{}/_search", self.base_url, index))
            .json(&body)
            .send()
            .await
            .map_err(|e| PdpError::TraceError(format!("Aggregation daily_counts échouée: {}", e)))?;
        if !resp.status().is_success() {
            return Ok(vec![0; days as usize]);
        }
        let json: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| PdpError::TraceError(format!("Parse réponse ES échouée: {}", e)))?;
        let buckets = json["aggregations"]["by_day"]["buckets"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        let counts: Vec<i64> = buckets
            .iter()
            .map(|b| b["doc_count"].as_i64().unwrap_or(0))
            .collect();
        // L'agrégation peut retourner moins que `days` buckets si la fenêtre
        // est partielle ; on aligne à droite (jours les plus récents en queue).
        let mut padded = vec![0i64; days as usize];
        let start = (days as usize).saturating_sub(counts.len());
        for (i, c) in counts.iter().enumerate() {
            if start + i < padded.len() {
                padded[start + i] = *c;
            }
        }
        Ok(padded)
    }

    /// Récupère toutes les factures émises (status DISTRIBUÉ) d'un tenant
    /// sur une période donnée. Utilisé par l'e-reporting Flux 10.
    ///
    /// - `siren` : SIREN du tenant (un index par SIREN : `pdp-{siren}`)
    /// - `from_date`, `to_date` : bornes au format `YYYY-MM-DD` (inclusives)
    ///
    /// Retourne les `ExchangeDocument` complets (incluant le `raw_xml`)
    /// triés par date d'émission croissante.
    pub async fn get_invoices_by_period(
        &self,
        siren: &str,
        from_date: &str,
        to_date: &str,
    ) -> PdpResult<Vec<ExchangeDocument>> {
        let index = self.index_name(siren);

        let search_body = serde_json::json!({
            "query": {
                "bool": {
                    "must": [
                        { "range": {
                            "issue_date": {
                                "gte": from_date,
                                "lte": to_date,
                            }
                        }},
                        { "term": { "status": "DISTRIBUÉ" } }
                    ]
                }
            },
            "sort": [{ "issue_date": "asc" }],
            "size": 10_000,
        });

        let resp = self.client
            .post(&format!("{}/{}/_search", self.base_url, index))
            .json(&search_body)
            .send()
            .await
            .map_err(|e| PdpError::TraceError(format!("Recherche factures par période échouée: {}", e)))?;

        if !resp.status().is_success() {
            // Index n'existe pas encore (tenant sans facture)
            return Ok(Vec::new());
        }

        let body: serde_json::Value = resp.json().await
            .map_err(|e| PdpError::TraceError(format!("Parse réponse ES échouée: {}", e)))?;

        let mut docs = Vec::new();
        if let Some(hits) = body["hits"]["hits"].as_array() {
            for hit in hits {
                if let Some(source) = hit.get("_source") {
                    if let Ok(doc) = serde_json::from_value::<ExchangeDocument>(source.clone()) {
                        docs.push(doc);
                    }
                }
            }
        }
        Ok(docs)
    }

    /// Liste tous les index (= tous les SIREN connus)
    pub async fn list_sirens(&self) -> PdpResult<Vec<String>> {
        let resp = self.client
            .get(&format!("{}/_cat/indices/{}?format=json", self.base_url, self.index_pattern()))
            .send()
            .await
            .map_err(|e| PdpError::TraceError(format!("Liste index échouée: {}", e)))?;

        let body: serde_json::Value = resp.json().await
            .map_err(|e| PdpError::TraceError(format!("Parse liste index échouée: {}", e)))?;

        let mut sirens = Vec::new();
        if let Some(indices) = body.as_array() {
            let prefix = format!("{}-", self.index_prefix);
            for idx in indices {
                if let Some(name) = idx["index"].as_str() {
                    if let Some(siren) = name.strip_prefix(&prefix) {
                        if siren != UNKNOWN_SUFFIX {
                            sirens.push(siren.to_string());
                        }
                    }
                }
            }
        }

        Ok(sirens)
    }

    /// Supprime tous les indices `{prefix}-*` de **ce store**.
    ///
    /// **Double garde-fou contre la perte de données démo** :
    ///
    /// 1. Le préfixe est par instance : un store de test (`for_test()`)
    ///    a un préfixe unique (`pdp-itest-{uuid}-*`) et ne peut donc PAS
    ///    supprimer les indices `pdp-{siren}` de la démo.
    /// 2. Si le préfixe est le préfixe de production (`pdp`), la variable
    ///    d'env `PDP_TRACE_ALLOW_CLEANUP=1` est requise. Sinon le cleanup
    ///    est ignoré.
    ///
    /// Les tests internes utilisent [`Self::force_cleanup`] qui ignore
    /// uniquement le garde-fou env var (mais reste scoped au préfixe du store).
    pub async fn cleanup(&self) -> PdpResult<()> {
        if self.index_prefix == INDEX_PREFIX
            && std::env::var("PDP_TRACE_ALLOW_CLEANUP").as_deref() != Ok("1")
        {
            tracing::debug!(
                "TraceStore::cleanup() ignoré sur préfixe production — set PDP_TRACE_ALLOW_CLEANUP=1 pour activer"
            );
            return Ok(());
        }
        self.force_cleanup().await
    }

    /// Cleanup inconditionnel — usage interne tests uniquement.
    /// Supprime tous les indices `{prefix}-*` (préfixe de **ce** store) sans
    /// vérifier l'env var. **Ne touche jamais** les indices d'autres préfixes.
    #[doc(hidden)]
    pub async fn force_cleanup(&self) -> PdpResult<()> {
        let _ = self.client
            .delete(&format!("{}/{}", self.base_url, self.index_pattern()))
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
                        seller_siret: source["seller_siret"].as_str().map(|s| s.to_string()),
                        buyer_siret: source["buyer_siret"].as_str().map(|s| s.to_string()),
                        seller_siren: source["seller_siren"].as_str().map(|s| s.to_string()),
                        buyer_siren: source["buyer_siren"].as_str().map(|s| s.to_string()),
                        status: source["status"].as_str().unwrap_or("INCONNU").to_string(),
                        error_count: source["error_count"].as_i64().unwrap_or(0) as i32,
                        created_at: source["created_at"].as_str().unwrap_or("").to_string(),
                        attachment_count: source["attachment_count"].as_u64().unwrap_or(0) as usize,
                        cdv_status_code: source["cdv_status_code"].as_u64().map(|v| v as u16),
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
                // force_cleanup() ignore le garde-fou ENV — réservé aux tests
                store.force_cleanup().await.ok();
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
        // Utilise un store noop (préfixe par défaut "pdp") pour tester index_name
        let store = TraceStore::noop();
        assert_eq!(store.index_name("123456789"), "pdp-123456789");
        assert_eq!(store.index_name("12345678901234"), "pdp-123456789");
        assert_eq!(store.index_name("123"), "pdp-123");
        assert_eq!(store.index_name(""), "pdp-unknown");
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
