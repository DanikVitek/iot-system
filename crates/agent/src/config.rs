use std::{sync::Arc, time::Duration};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Configuration {
    mqtt: Mqtt,
    delay: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Mqtt {
    broker_host: Arc<str>,
    broker_port: u16,
    topic: Arc<str>,
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

impl Mqtt {
    pub fn broker_host(&self) -> &str {
        &self.broker_host
    }

    pub fn broker_port(&self) -> u16 {
        self.broker_port
    }

    pub fn topic(&self) -> &str {
        &self.topic
    }

    pub fn broker_address(&self) -> String {
        format!("tcp://{}:{}", self.broker_host, self.broker_port)
    }
}
