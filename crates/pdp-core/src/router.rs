use std::collections::HashMap;
use tokio::sync::mpsc;
use crate::error::{PdpError, PdpResult};
use crate::exchange::Exchange;
use crate::route::Route;

/// Le Router orchestre l'exécution de toutes les routes.
/// Il gère le cycle de vie des routes et la distribution des exchanges.
pub struct Router {
    routes: HashMap<String, Route>,
    /// Channel pour recevoir les résultats des routes
    result_tx: mpsc::Sender<RouteResult>,
    result_rx: Option<mpsc::Receiver<RouteResult>>,
}

#[derive(Debug)]
pub struct RouteResult {
    pub route_id: String,
    pub exchanges: Vec<Exchange>,
    pub error: Option<PdpError>,
}

impl Router {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel(1000);
        Self {
            routes: HashMap::new(),
            result_tx: tx,
            result_rx: Some(rx),
        }
    }

    /// Ajoute une route au router
    pub fn add_route(&mut self, route: Route) -> PdpResult<()> {
        let id = route.id.clone();
        if self.routes.contains_key(&id) {
            return Err(PdpError::RouteError {
                route_id: id,
                message: "Route déjà enregistrée".to_string(),
            });
        }
        tracing::info!(route_id = %id, desc = %route.description, "Route ajoutée");
        self.routes.insert(id, route);
        Ok(())
    }

    /// Liste les routes enregistrées
    pub fn list_routes(&self) -> Vec<&str> {
        self.routes.keys().map(|k| k.as_str()).collect()
    }

    /// Exécute une route spécifique par son ID
    pub async fn execute_route(&self, route_id: &str) -> PdpResult<Vec<Exchange>> {
        let route = self.routes.get(route_id).ok_or_else(|| PdpError::RouteError {
            route_id: route_id.to_string(),
            message: "Route non trouvée".to_string(),
        })?;

        route.execute().await
    }

    /// Exécute toutes les routes actives une fois (mode one-shot)
    pub async fn execute_all(&self) -> Vec<RouteResult> {
        let mut results = Vec::new();

        for (route_id, route) in &self.routes {
            if !route.enabled {
                tracing::debug!(route_id = %route_id, "Route désactivée, skip");
                continue;
            }

            match route.execute().await {
                Ok(exchanges) => {
                    let success_count = exchanges.iter().filter(|e| !e.has_errors()).count();
                    let error_count = exchanges.iter().filter(|e| e.has_errors()).count();
                    tracing::info!(
                        route_id = %route_id,
                        success = success_count,
                        errors = error_count,
                        "Route exécutée"
                    );
                    results.push(RouteResult {
                        route_id: route_id.clone(),
                        exchanges,
                        error: None,
                    });
                }
                Err(err) => {
                    tracing::error!(route_id = %route_id, error = %err, "Erreur route");
                    results.push(RouteResult {
                        route_id: route_id.clone(),
                        exchanges: Vec::new(),
                        error: Some(err),
                    });
                }
            }
        }

        results
    }

    /// Démarre le router en mode polling (boucle continue)
    pub async fn start_polling(
        &self,
        interval: std::time::Duration,
        mut shutdown: tokio::sync::watch::Receiver<bool>,
    ) -> PdpResult<()> {
        tracing::info!(
            routes = self.routes.len(),
            interval_secs = interval.as_secs(),
            "Démarrage du router en mode polling"
        );

        loop {
            tokio::select! {
                _ = tokio::time::sleep(interval) => {
                    let results = self.execute_all().await;
                    for result in &results {
                        if let Some(ref err) = result.error {
                            tracing::error!(
                                route_id = %result.route_id,
                                error = %err,
                                "Erreur lors du polling"
                            );
                        }
                    }
                    // Envoyer les résultats via le channel
                    for result in results {
                        let _ = self.result_tx.send(result).await;
                    }
                }
                _ = shutdown.changed() => {
                    if *shutdown.borrow() {
                        tracing::info!("Arrêt du router demandé");
                        break;
                    }
                }
            }
        }

        Ok(())
    }

    /// Prend le receiver de résultats (ne peut être appelé qu'une fois)
    pub fn take_result_receiver(&mut self) -> Option<mpsc::Receiver<RouteResult>> {
        self.result_rx.take()
    }
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}
