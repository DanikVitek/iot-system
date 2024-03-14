use std::time::Duration;

use color_eyre::Result;
use iot_system::{config::TryRead, domain::Agent, reclone, setup_tracing};
use mqtt::{AsyncClient, ConnectOptionsBuilder};
use tracing::instrument;

use crate::{
    config::Configuration,
    file_datasource::{state, FileDatasource},
};

mod config;
mod file_datasource;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let _guard = setup_tracing("./logs", "lab1.log")?;

    let config = Configuration::try_read()?;
    let client = connect_mqtt(config.mqtt().to_owned()).await?;

    let datasource = FileDatasource::new("./data/accelerometer.csv", "./data/gps.csv");
    let result = publish(client, config.mqtt().topic(), datasource, config.delay()).await;

    result.map_err(Into::into)
}

#[cfg(all(feature = "async-read", not(feature = "sync-read")))]
#[instrument(skip(client, datasource))]
async fn publish(
    client: AsyncClient,
    topic: &str,
    datasource: FileDatasource<state::New>,
    delay: Duration,
) -> Result<()> {
    let mut interval = tokio::time::interval(delay);
    let datasource = datasource.start_reading().await?;

    let (data_reader_sender, mut data_reader_receiver) = tokio::sync::mpsc::channel::<Agent>(7);

    tokio::spawn(read_data(datasource, data_reader_sender));

    while let Some(data) = data_reader_receiver.recv().await {
        tracing::debug!("Data received from the channel. Sending to the broker: {data:#?}");
        let message = mqtt::Message::new(topic, serde_json::to_vec(&data)?, 0);
        if let Err(err) = client.publish(message).await {
            tracing::error!("Failed to send message to topic {topic}: {err}")
        } else {
            tracing::info!("Data sent to the broker");
        }
        interval.tick().await;
    }
    tracing::info!("No more data");

    Ok(())
}

#[cfg(all(not(feature = "async-read"), feature = "sync-read"))]
#[instrument(skip(client, datasource))]
async fn publish(
    client: AsyncClient,
    topic: &str,
    datasource: FileDatasource<state::New>,
    delay: Duration,
) -> Result<()> {
    let mut interval = tokio::time::interval(delay);
    let mut datasource = datasource.start_reading()?;

    tracing::info!("Reading data from the datasource");
    loop {
        interval.tick().await;
        let data: Agent = match datasource.read() {
            Ok(data) => data,
            Err(err) => {
                tracing::error!("Failed to read data from the datasource: {}", err);
                continue;
            }
        };
        tracing::debug!("Sending data to the broker: {data:#?}");
        let message = mqtt::Message::new(topic, serde_json::to_vec(&data)?, 0);
        if let Err(err) = client.publish(message).await {
            tracing::error!("Failed to send message to topic {topic}: {err}")
        };
        tracing::info!("Data sent to the broker");
    }
}

#[cfg(all(not(feature = "async-read"), not(feature = "sync-read")))]
async fn publish(
    _client: AsyncClient,
    _topic: &str,
    _datasource: FileDatasource<state::New>,
    _delay: Duration,
) -> Result<()> {
    panic!("You must enable either the `async-read` or `sync-read` feature to use the `publish` function.")
}

#[cfg(all(feature = "async-read", feature = "sync-read"))]
async fn publish(
    _client: AsyncClient,
    _topic: &str,
    _datasource: FileDatasource<state::New>,
    _delay: Duration,
) -> Result<()> {
    panic!("You must enable only one of the `async-read` or `sync-read` features to use the `publish` function.")
}

#[instrument(skip(datasource, data_reader_sender))]
async fn read_data(
    mut datasource: FileDatasource<state::Reading>,
    data_reader_sender: tokio::sync::mpsc::Sender<Agent>,
) {
    tracing::info!("Reading data from the datasource");
    loop {
        let data: Agent = match datasource.read().await {
            Ok(data) => data,
            Err(err) => {
                tracing::error!("Failed to read data from the datasource: {}", err);
                continue;
            }
        };
        tracing::debug!("Sending data to the channel: {data:#?}");
        if let Err(err) = data_reader_sender.send(data).await {
            tracing::error!("Failed to send data to the receiver: {}", err);
        }
    }
}

#[instrument(skip(config))]
async fn connect_mqtt(config: config::Mqtt) -> Result<AsyncClient> {
    let client = mqtt::AsyncClient::new(config.broker_address())?;

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
                tracing::info!(
                    "Failed to connect to the broker ({}:{}), return code {rc}",
                    config.broker_host(),
                    config.broker_port()
                );
            },
        )
        .await?;

    Ok(client)
}
