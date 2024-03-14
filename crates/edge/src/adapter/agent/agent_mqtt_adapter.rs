use std::sync::Arc;

use iot_system::{config::Mqtt, domain::Agent};

pub struct AgentMqttAdapter {
    client: mqtt::AsyncClient,
    topic: Arc<str>,
    sender: tokio::sync::mpsc::UnboundedSender<Agent>,
}

impl AgentMqttAdapter {
    pub async fn new(
        config: Mqtt,
        sender: tokio::sync::mpsc::UnboundedSender<Agent>,
    ) -> mqtt::Result<Self> {
        let topic = config.topic();
        iot_system::mqtt::connect(config).await.map(|client| Self {
            client,
            topic,
            sender,
        })
    }

    pub async fn listen_for_data(mut self) -> Result<Self, SendError> {
        self.client.subscribe(self.topic.to_string(), 0).await?;
        let messages = self.client.get_stream(None);
        while let Ok(Some(message)) = messages.recv().await {
            if self
                .sender
                .send(serde_json::from_slice(message.payload())?)
                .is_err()
            {
                break;
            };
        }
        self.client.unsubscribe(self.topic.to_string()).await?;
        Ok(self)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SendError {
    #[error("Failed to send message to the hub gateway channel: {0}")]
    Mqtt(
        #[from]
        #[source]
        mqtt::Error,
    ),
    #[error("Failed to deserialize the message: {0}")]
    Serde(
        #[from]
        #[source]
        serde_json::Error,
    ),
}
