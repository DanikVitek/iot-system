use mqtt::ConnectOptionsBuilder;
use tracing::instrument;

use crate::{config::Mqtt, reclone};

#[instrument]
pub async fn connect(config: Mqtt) -> mqtt::Result<mqtt::AsyncClient> {
    let client = mqtt::AsyncClient::new(&config)?;

    client
        .connect_with_callbacks(
            ConnectOptionsBuilder::new().finalize(),
            {
                reclone!(config);
                move |_, _| {
                    tracing::info!(
                        "Connected to the broker ({}:{})",
                        config.broker_host(),
                        config.broker_port()
                    );
                }
            },
            move |_, _, rc| {
                tracing::error!(
                    "Failed to connect to the broker ({}:{}), return code {rc}",
                    config.broker_host(),
                    config.broker_port()
                );
            },
        )
        .await?;

    Ok(client)
}
