use std::time::Duration;

use iot_system::config::Mqtt;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Configuration {
    mqtt: Mqtt,
    delay: f64,
}

impl Configuration {
    pub fn mqtt(&self) -> &Mqtt {
        &self.mqtt
    }

    pub fn delay(&self) -> Duration {
        Duration::from_secs_f64(self.delay)
    }
}

impl iot_system::config::TryRead<'_> for Configuration {}
