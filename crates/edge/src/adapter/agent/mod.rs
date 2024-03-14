pub mod agent_mqtt_adapter;

/// Trait, representing the Agent Gateway interface.
/// All agent gateway adapters must implement these methods.
pub trait AgentGateway {
    type Message;
    
    /// Method to handle incoming messages from the agent.
    /// Parameters:
    /// - `client`: MQTT client instance.
    /// - `userdata`: Any additional user data passed to the MQTT client.
    /// - `msg`: The MQTT message received from the agent.
    fn on_message(&mut self, message: Self::Message);
}