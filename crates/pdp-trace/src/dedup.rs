use async_trait::async_trait;
use pdp_core::error::{PdpError, PdpResult};
use pdp_core::exchange::Exchange;
use pdp_core::processor::Processor;
use std::sync::Arc;

use crate::store::TraceStore;

/// Processeur de détection de doublons persistant via Elasticsearch.
///
/// Vérifie les règles :
/// - **BR-FR-12** : unicité du numéro de facture par SIREN vendeur et année
/// - **BR-FR-13** : doublon exact (même clé de facture = SIREN/NUMERO/ANNEE)
///
/// La clé de facture (`invoice_key`) est calculée par `InvoiceData::key_string()`
/// et stockée dans Elasticsearch comme champ `keyword` pour des recherches exactes.
///
/// Ce processeur complète la détection en mémoire de `ReceptionProcessor` (doublons
/// de noms de fichier dans le même batch) en offrant une détection persistante
/// qui survit aux redémarrages.
pub struct DuplicateCheckProcessor {
    store: Arc<TraceStore>,
}

impl DuplicateCheckProcessor {
    pub fn new(store: Arc<TraceStore>) -> Self {
        Self { store }
    }
}

#[async_trait]
impl Processor for DuplicateCheckProcessor {
    fn name(&self) -> &str {
        "DuplicateCheck"
    }

    async fn process(&self, mut exchange: Exchange) -> PdpResult<Exchange> {
        // Pas de facture parsée => on passe (le parsing n'a pas encore eu lieu ou a échoué)
        let invoice = match &exchange.invoice {
            Some(inv) => inv,
            None => return Ok(exchange),
        };

        let invoice_key = invoice.key_string();
        if invoice_key.is_empty() {
            // Pas assez d'informations pour constituer la clé (SIREN/numéro/année manquant)
            return Ok(exchange);
        }

        let siren = invoice.seller_siren();

        // Rechercher les échanges existants avec la même clé de facture
        let existing = match self.store.search_by_invoice_key(
            &invoice_key,
            siren.as_deref(),
        ).await {
            Ok(results) => results,
            Err(e) => {
                // En cas d'erreur ES (réseau, index inexistant), on log mais on ne bloque pas
                tracing::warn!(
                    error = %e,
                    invoice_key = %invoice_key,
                    "Impossible de vérifier les doublons dans Elasticsearch, traitement continué"
                );
                return Ok(exchange);
            }
        };

        // Exclure l'échange courant des résultats (il peut déjà être indexé
        // si le TraceProcessor::parsed() a été exécuté avant)
        let duplicates: Vec<_> = existing.iter()
            .filter(|e| e.exchange_id != exchange.id.to_string())
            .collect();

        if !duplicates.is_empty() {
            let prev = &duplicates[0];
            exchange.set_property("duplicate.detected", "true");
            exchange.set_property("duplicate.previous_exchange_id", &prev.exchange_id);

            let error = PdpError::ValidationError(format!(
                "BR-FR-12/13 : Facture en doublon (clé: {}). Échange précédent: {} ({})",
                invoice_key,
                prev.exchange_id,
                prev.created_at,
            ));
            exchange.add_error("DuplicateCheck", &error);

            tracing::warn!(
                invoice_key = %invoice_key,
                previous_exchange_id = %prev.exchange_id,
                current_exchange_id = %exchange.id,
                "Doublon détecté — BR-FR-12/13"
            );
        }

        Ok(exchange)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test d'intégration nécessitant Elasticsearch.
    /// Lancer : docker run -d --name pdp-es -p 9200:9200 -e "discovery.type=single-node" -e "xpack.security.enabled=false" elasticsearch:8.15.0

    #[tokio::test]
    async fn test_duplicate_check_no_invoice() {
        // Sans facture parsée, le processor doit passer sans erreur
        let store = match TraceStore::for_test().await {
            Ok(s) => Arc::new(s),
            Err(_) => {
                eprintln!("Elasticsearch non disponible, test ignoré");
                return;
            }
        };

        let processor = DuplicateCheckProcessor::new(store);
        let exchange = Exchange::new(b"<Invoice/>".to_vec());
        // Pas d'invoice => doit passer
        let result = processor.process(exchange).await.unwrap();
        assert!(!result.has_errors());
    }
}
