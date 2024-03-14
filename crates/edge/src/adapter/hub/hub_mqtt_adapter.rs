use iot_system::{config, domain::ProcessedAgent};

use crate::adapter::hub::HubGateway;

struct HubMqttAdapter {
    client: mqtt::AsyncClient,
}

impl HubMqttAdapter {
    pub fn new(config: config::Mqtt) -> Self {
        let client = mqtt::AsyncClient::new(config.broker_address()).unwrap();
        Self { client }
    }
}

impl HubGateway for HubMqttAdapter {
    fn save_data(&mut self, processed_data: ProcessedAgent) -> bool {
        todo!()
    }
}
