use crate::endpoint::{Consumer, Producer};
use crate::error::PdpResult;
use crate::exchange::Exchange;
use crate::model::FlowStatus;
use crate::processor::Processor;

/// Une Route définit un chemin de traitement : source -> processors -> destination.
/// Équivalent d'une Route dans Apache Camel.
pub struct Route {
    /// Identifiant unique de la route
    pub id: String,
    /// Description de la route
    pub description: String,
    /// Source (Consumer)
    pub consumer: Box<dyn Consumer>,
    /// Chaîne de processors
    pub processors: Vec<Box<dyn Processor>>,
    /// Destination (Producer)
    pub producer: Box<dyn Producer>,
    /// Route d'erreur (optionnelle)
    pub error_handler: Option<Box<dyn Producer>>,
    /// Active ou non
    pub enabled: bool,
}

impl Route {
    /// Exécute la route : poll source -> process -> send destination
    pub async fn execute(&self) -> PdpResult<Vec<Exchange>> {
        if !self.enabled {
            tracing::debug!(route_id = %self.id, "Route désactivée, skip");
            return Ok(Vec::new());
        }

        tracing::info!(route_id = %self.id, desc = %self.description, "Exécution de la route");

        // 1. Poll la source
        let exchanges = self.consumer.poll().await?;
        tracing::info!(
            route_id = %self.id,
            count = exchanges.len(),
            "Exchanges récupérés depuis la source"
        );

        let mut results = Vec::new();

        for mut exchange in exchanges {
            exchange.set_header("route.id", &self.id);

            // 2. Passer dans chaque processor
            let mut current = exchange;
            let mut had_error = false;

            for processor in &self.processors {
                tracing::debug!(
                    route_id = %self.id,
                    processor = processor.name(),
                    exchange_id = %current.id,
                    "Processor en cours"
                );

                match processor.process(current.clone()).await {
                    Ok(processed) => {
                        current = processed;
                    }
                    Err(err) => {
                        tracing::error!(
                            route_id = %self.id,
                            processor = processor.name(),
                            exchange_id = %current.id,
                            error = %err,
                            "Erreur dans le processor"
                        );
                        current.add_error(processor.name(), &err);
                        had_error = true;

                        // Envoyer vers le error handler si configuré
                        if let Some(ref error_handler) = self.error_handler {
                            let _ = error_handler.send(current.clone()).await;
                        }
                        break;
                    }
                }
            }

            // 3. Envoyer vers la destination (sauf si erreur)
            if !had_error {
                current.set_status(FlowStatus::Distributing);
                match self.producer.send(current.clone()).await {
                    Ok(sent) => {
                        let mut final_exchange = sent;
                        final_exchange.set_status(FlowStatus::Distributed);
                        results.push(final_exchange);
                    }
                    Err(err) => {
                        tracing::error!(
                            route_id = %self.id,
                            exchange_id = %current.id,
                            error = %err,
                            "Erreur lors de l'envoi vers la destination"
                        );
                        current.add_error("producer", &err);
                        if let Some(ref error_handler) = self.error_handler {
                            let _ = error_handler.send(current.clone()).await;
                        }
                        results.push(current);
                    }
                }
            } else {
                results.push(current);
            }
        }

        tracing::info!(
            route_id = %self.id,
            total = results.len(),
            errors = results.iter().filter(|e| e.has_errors()).count(),
            "Route terminée"
        );

        Ok(results)
    }
}

/// Builder fluent pour construire une Route
pub struct RouteBuilder {
    id: String,
    description: String,
    consumer: Option<Box<dyn Consumer>>,
    processors: Vec<Box<dyn Processor>>,
    producer: Option<Box<dyn Producer>>,
    error_handler: Option<Box<dyn Producer>>,
    enabled: bool,
}

impl RouteBuilder {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            description: String::new(),
            consumer: None,
            processors: Vec::new(),
            producer: None,
            error_handler: None,
            enabled: true,
        }
    }

    pub fn description(mut self, desc: &str) -> Self {
        self.description = desc.to_string();
        self
    }

    pub fn from_source(mut self, consumer: Box<dyn Consumer>) -> Self {
        self.consumer = Some(consumer);
        self
    }

    pub fn process(mut self, processor: Box<dyn Processor>) -> Self {
        self.processors.push(processor);
        self
    }

    pub fn to_destination(mut self, producer: Box<dyn Producer>) -> Self {
        self.producer = Some(producer);
        self
    }

    pub fn on_error(mut self, error_handler: Box<dyn Producer>) -> Self {
        self.error_handler = Some(error_handler);
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn build(self) -> PdpResult<Route> {
        let consumer = self.consumer.ok_or_else(|| {
            crate::error::PdpError::RouteError {
                route_id: self.id.clone(),
                message: "Consumer (source) non défini".to_string(),
            }
        })?;

        let producer = self.producer.ok_or_else(|| {
            crate::error::PdpError::RouteError {
                route_id: self.id.clone(),
                message: "Producer (destination) non défini".to_string(),
            }
        })?;

        Ok(Route {
            id: self.id,
            description: self.description,
            consumer,
            processors: self.processors,
            producer,
            error_handler: self.error_handler,
            enabled: self.enabled,
        })
    }
}
