use iot_system::config::{Mqtt, Server};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Configuration {
    pub mqtt: Mqtt,
    pub hub_mqtt: Mqtt,
    pub hub_grpc: Server,
}

impl iot_system::config::TryRead<'_> for Configuration {}
