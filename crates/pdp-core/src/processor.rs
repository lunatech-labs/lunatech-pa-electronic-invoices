use async_trait::async_trait;
use crate::exchange::Exchange;
use crate::error::PdpResult;

/// Un Processor transforme un Exchange.
/// C'est l'équivalent du Processor dans Apache Camel.
/// Chaque étape du pipeline (parsing, validation, transformation, etc.)
/// est un Processor.
#[async_trait]
pub trait Processor: Send + Sync {
    /// Nom du processor (pour la traçabilité)
    fn name(&self) -> &str;

    /// Traite l'exchange et retourne l'exchange modifié
    async fn process(&self, exchange: Exchange) -> PdpResult<Exchange>;
}

/// Processor qui enchaîne plusieurs processors
pub struct ProcessorChain {
    name: String,
    processors: Vec<Box<dyn Processor>>,
}

impl ProcessorChain {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            processors: Vec::new(),
        }
    }

    pub fn add(mut self, processor: Box<dyn Processor>) -> Self {
        self.processors.push(processor);
        self
    }
}

#[async_trait]
impl Processor for ProcessorChain {
    fn name(&self) -> &str {
        &self.name
    }

    async fn process(&self, mut exchange: Exchange) -> PdpResult<Exchange> {
        for processor in &self.processors {
            tracing::debug!(
                processor = processor.name(),
                exchange_id = %exchange.id,
                "Exécution du processor"
            );
            exchange = processor.process(exchange).await?;
        }
        Ok(exchange)
    }
}

/// Processor simple basé sur une closure (pour les cas simples)
pub struct FnProcessor<F>
where
    F: Fn(Exchange) -> PdpResult<Exchange> + Send + Sync,
{
    name: String,
    func: F,
}

impl<F> FnProcessor<F>
where
    F: Fn(Exchange) -> PdpResult<Exchange> + Send + Sync,
{
    pub fn new(name: &str, func: F) -> Self {
        Self {
            name: name.to_string(),
            func,
        }
    }
}

#[async_trait]
impl<F> Processor for FnProcessor<F>
where
    F: Fn(Exchange) -> PdpResult<Exchange> + Send + Sync,
{
    fn name(&self) -> &str {
        &self.name
    }

    async fn process(&self, exchange: Exchange) -> PdpResult<Exchange> {
        (self.func)(exchange)
    }
}

/// Processor de logging (utile pour le debug)
pub struct LogProcessor {
    name: String,
    level: LogLevel,
}

#[derive(Debug, Clone)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl LogProcessor {
    pub fn new(name: &str, level: LogLevel) -> Self {
        Self {
            name: name.to_string(),
            level,
        }
    }

    pub fn info(name: &str) -> Self {
        Self::new(name, LogLevel::Info)
    }
}

/// Processor qui tag chaque exchange avec le SIREN du tenant.
/// Insere en tete de chaque pipeline tenant pour la tracabilite.
pub struct TenantTagProcessor {
    siren: String,
}

impl TenantTagProcessor {
    pub fn new(siren: &str) -> Self {
        Self {
            siren: siren.to_string(),
        }
    }
}

#[async_trait]
impl Processor for LogProcessor {
    fn name(&self) -> &str {
        &self.name
    }

    async fn process(&self, exchange: Exchange) -> PdpResult<Exchange> {
        let msg = format!(
            "[{}] Exchange {} | flow={} | status={} | fichier={} | erreurs={}",
            self.name,
            exchange.id,
            exchange.flow_id,
            exchange.status,
            exchange.source_filename.as_deref().unwrap_or("N/A"),
            exchange.errors.len(),
        );

        match self.level {
            LogLevel::Debug => tracing::debug!("{}", msg),
            LogLevel::Info => tracing::info!("{}", msg),
            LogLevel::Warn => tracing::warn!("{}", msg),
            LogLevel::Error => tracing::error!("{}", msg),
        }

        Ok(exchange)
    }
}

#[async_trait]
impl Processor for TenantTagProcessor {
    fn name(&self) -> &str {
        "tenant-tag"
    }

    async fn process(&self, mut exchange: Exchange) -> PdpResult<Exchange> {
        exchange.set_tenant_siren(&self.siren);
        Ok(exchange)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tenant_tag_processor() {
        let processor = TenantTagProcessor::new("123456789");
        assert_eq!(processor.name(), "tenant-tag");

        let exchange = Exchange::new(b"test".to_vec());
        assert!(exchange.tenant_siren().is_none());

        let exchange = processor.process(exchange).await.unwrap();
        assert_eq!(exchange.tenant_siren(), Some("123456789"));
    }

    #[tokio::test]
    async fn test_tenant_tag_processor_in_chain() {
        let chain = ProcessorChain::new("test-chain")
            .add(Box::new(TenantTagProcessor::new("999888777")))
            .add(Box::new(LogProcessor::info("after-tag")));

        let exchange = Exchange::new(b"data".to_vec());
        let exchange = chain.process(exchange).await.unwrap();
        assert_eq!(exchange.tenant_siren(), Some("999888777"));
    }
}
