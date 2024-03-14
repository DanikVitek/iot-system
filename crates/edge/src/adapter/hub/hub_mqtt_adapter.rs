use std::sync::Arc;

use async_trait::async_trait;
use iot_system::{config::Mqtt, domain::ProcessedAgent};
use mqtt::Message;
use tracing::instrument;

use crate::adapter::hub::HubGateway;

struct HubMqttAdapter {
    client: mqtt::AsyncClient,
    topic: Arc<str>,
}

impl HubMqttAdapter {
    #[instrument]
    pub async fn new(config: Mqtt) -> mqtt::Result<Self> {
        let topic = config.topic();
        iot_system::mqtt::connect(config)
            .await
            .map(|client| Self { client, topic })
    }
}

#[async_trait]
impl HubGateway for HubMqttAdapter {
    type Error = SendError;

    #[instrument(skip(self))]
    async fn save_data(&mut self, processed_data: ProcessedAgent) -> Result<(), Self::Error> {
        self.client
            .publish(Message::new(
                self.topic.to_string(),
                serde_json::to_vec(&processed_data)?,
                0,
            ))
            .await
            .map_err(Into::into)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SendError {
    #[error("Failed to send message to the broker: {0}")]
    Mqtt(
        #[from]
        #[source]
        mqtt::Error,
    ),
    #[error("Failed to serialize the message: {0}")]
    Serde(
        #[from]
        #[source]
        serde_json::Error,
    ),
}
