use adapter::agent::agent_mqtt_adapter;
use color_eyre::Result;
use edge::{
    adapter,
    adapter::{
        agent::agent_mqtt_adapter::AgentMqttAdapter,
        hub::{hub_mqtt_adapter::HubMqttAdapter, HubGateway},
    },
    config::Configuration,
    process_agent_data,
};
use iot_system::{config::TryRead, setup_tracing};

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let _guard = setup_tracing("./logs", "lab4.log")?;

    let config = Configuration::try_read()?;

    let mut hub_adapter = HubMqttAdapter::new(config.hub_mqtt).await?;
    let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel();
    let agent_adapter = AgentMqttAdapter::new(config.agent_mqtt, sender).await?;
    let handle = tokio::spawn(async {
        _ = agent_adapter.listen_for_data().await?;
        Ok::<(), agent_mqtt_adapter::SendError>(())
    });
    let mut prev_data = None;
    while let Some(data) = receiver.recv().await {
        hub_adapter
            .save_data(process_agent_data(
                data.clone(),
                prev_data.replace(data).as_ref(),
            ))
            .await?;
    }
    handle.await??;

    Ok(())
}
