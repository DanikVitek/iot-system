use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Configuration {
    store_api: StoreApi,
    redis: Redis,
    batch_size: u32,
    mqtt_broker: MQTTBroker,
}

#[derive(Debug, Deserialize)]
pub struct StoreApi {
    host: String,
    port: u16,
}

#[derive(Debug, Deserialize)]
pub struct Redis {
    host: String,
    port: u16,
}

#[derive(Debug, Deserialize)]
pub struct MQTTBroker {
    host: String,
    port: u16,
    topic: String,
}

impl Configuration {
    pub fn store_api(&self) -> &StoreApi {
        &self.store_api
    }
    pub fn redis(&self) -> &Redis {
        &self.redis
    }
    pub fn batch_size(&self) -> u32 {
        self.batch_size
    }
    pub fn mqtt_broker(&self) -> &MQTTBroker {
        &self.mqtt_broker
    }
}

impl iot_system::config::TryRead<'_> for Configuration {}

impl StoreApi {
    pub fn host(&self) -> &str {
        &self.host
    }
    pub fn port(&self) -> u16 {
        self.port
    }

    //noinspection HttpUrlsUsage
    pub fn base_url(&self) -> String {
        format!("http://{}:{}", self.host, self.port)
    }
}

impl Redis {
    pub fn host(&self) -> &str {
        &self.host
    }
    pub fn port(&self) -> u16 {
        self.port
    }
}

impl MQTTBroker {
    pub fn host(&self) -> &str {
        &self.host
    }
    pub fn port(&self) -> u16 {
        self.port
    }
    pub fn topic(&self) -> &str {
        &self.topic
    }
}
