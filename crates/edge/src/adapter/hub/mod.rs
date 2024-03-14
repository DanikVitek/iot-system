use std::error::Error;

use async_trait::async_trait;
use iot_system::domain::ProcessedAgent;

pub mod hub_mqtt_adapter;

#[async_trait]
pub trait HubGateway {
    type Error: Error;

    async fn save_data(&mut self, processed_data: ProcessedAgent) -> Result<(), Self::Error>;
}
