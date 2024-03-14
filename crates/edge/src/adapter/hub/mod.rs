use iot_system::domain::ProcessedAgent;

pub mod hub_mqtt_adapter;

pub trait HubGateway {
    fn save_data(&mut self, processed_data: ProcessedAgent) -> bool;
}