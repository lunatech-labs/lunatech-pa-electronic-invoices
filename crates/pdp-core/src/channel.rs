use crate::exchange::Exchange;
use crate::error::PdpResult;
use crate::endpoint::Consumer;
use async_trait::async_trait;
use tokio::sync::mpsc;

/// Consumer qui reçoit des Exchange depuis un canal mpsc.
/// Utilisé pour injecter des flux reçus via HTTP dans le pipeline.
pub struct ChannelConsumer {
    name: String,
    rx: tokio::sync::Mutex<mpsc::Receiver<Exchange>>,
}

impl ChannelConsumer {
    pub fn new(name: &str, rx: mpsc::Receiver<Exchange>) -> Self {
        Self {
            name: name.to_string(),
            rx: tokio::sync::Mutex::new(rx),
        }
    }
}

#[async_trait]
impl Consumer for ChannelConsumer {
    fn name(&self) -> &str {
        &self.name
    }

    async fn poll(&self) -> PdpResult<Vec<Exchange>> {
        let mut rx = self.rx.lock().await;
        let mut exchanges = Vec::new();
        // Drainer tous les exchanges disponibles sans bloquer
        while let Ok(exchange) = rx.try_recv() {
            exchanges.push(exchange);
        }
        Ok(exchanges)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_channel_consumer_empty() {
        let (_tx, rx) = mpsc::channel::<Exchange>(10);
        let consumer = ChannelConsumer::new("test", rx);
        let exchanges = consumer.poll().await.unwrap();
        assert!(exchanges.is_empty());
    }

    #[tokio::test]
    async fn test_channel_consumer_receives_exchanges() {
        let (tx, rx) = mpsc::channel::<Exchange>(10);
        let consumer = ChannelConsumer::new("test", rx);

        // Envoyer 3 exchanges
        for i in 0..3 {
            let exchange = Exchange::new(format!("body-{}", i).into_bytes());
            tx.send(exchange).await.unwrap();
        }

        let exchanges = consumer.poll().await.unwrap();
        assert_eq!(exchanges.len(), 3);
    }

    #[tokio::test]
    async fn test_channel_consumer_drains_all() {
        let (tx, rx) = mpsc::channel::<Exchange>(10);
        let consumer = ChannelConsumer::new("test", rx);

        tx.send(Exchange::new(b"a".to_vec())).await.unwrap();
        tx.send(Exchange::new(b"b".to_vec())).await.unwrap();

        // Premier poll : récupère tout
        let exchanges = consumer.poll().await.unwrap();
        assert_eq!(exchanges.len(), 2);

        // Deuxième poll : vide
        let exchanges = consumer.poll().await.unwrap();
        assert!(exchanges.is_empty());
    }

    #[tokio::test]
    async fn test_channel_consumer_sender_dropped() {
        let (tx, rx) = mpsc::channel::<Exchange>(10);
        let consumer = ChannelConsumer::new("test", rx);

        tx.send(Exchange::new(b"last".to_vec())).await.unwrap();
        drop(tx);

        // Doit encore récupérer le dernier exchange
        let exchanges = consumer.poll().await.unwrap();
        assert_eq!(exchanges.len(), 1);

        // Puis vide
        let exchanges = consumer.poll().await.unwrap();
        assert!(exchanges.is_empty());
    }
}
