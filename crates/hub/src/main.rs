use std::num::NonZeroUsize;

use color_eyre::Result;
use iot_system::{
    config::{Mqtt, TryRead},
    domain::ProcessedAgent,
    proto::{self, store_client::StoreClient},
    setup_tracing,
};
use redis::Commands;
use tokio_stream::StreamExt;
use tonic::transport::Channel;
use tracing::instrument;

use crate::config::Configuration;

mod config;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let _guard = setup_tracing("./logs", "lab3.log")?;

    let Configuration {
        store_api: store_api_config,
        redis: redis_config,
        batch_size,
        mqtt: mqtt_config,
    } = Configuration::try_read()?;

    let redis_client = redis::Client::open(redis_config)?;
    let mqtt_client = iot_system::mqtt::connect(mqtt_config.clone()).await?;
    let store_api_client = StoreClient::connect(store_api_config).await?;

    let Mqtt { topic, .. } = mqtt_config;
    listen_for_topic(mqtt_client, redis_client, store_api_client, batch_size, {
        let t = topic.to_string();
        drop(topic);
        t
    })
    .await?;

    Ok(())
}

macro_rules! try_get {
    ($e:expr, $err:ident => $msg:tt) => {
        match $e {
            Ok(value) => value,
            Err($err) => {
                tracing::error!($msg);
                continue;
            }
        }
    };
}
macro_rules! try_execute {
    ($e:expr, $err:ident => $msg:tt) => {
        if let Err($err) = $e {
            tracing::error!($msg);
            continue;
        }
    };
}

#[instrument(skip(mqtt_client, redis_client, store_api_client))]
async fn listen_for_topic(
    mut mqtt_client: mqtt::AsyncClient,
    mut redis_client: redis::Client,
    store_api_client: StoreClient<Channel>,
    batch_size: NonZeroUsize,
    topic: String,
) -> Result<()> {
    let (data_sender, data_receiver) =
        tokio::sync::mpsc::unbounded_channel::<(Vec<Vec<u8>>, ProcessedAgent)>();

    let handle = tokio::spawn(send_data_to_store_api(store_api_client, data_receiver));

    mqtt_client.subscribe(topic, mqtt::QOS_0).await?;

    let mut messages = mqtt_client.get_stream(None);
    while let Some(message) = messages.next().await.flatten() {
        let payload = message.payload();
        let processed_agent_data: ProcessedAgent = try_get!(
            serde_json::from_slice(payload),
            error => "Failed to decode the payload: {error}"
        );
        tracing::info!("Received message: {processed_agent_data:?}");

        const REDIS_KEY: &str = "processed_agent_data";

        if try_get!(
            redis_client.llen::<_, usize>(REDIS_KEY),
            error => "Failed to get llen for {REDIS_KEY}: {error}"
        ) >= batch_size.get() - 1
        {
            let data: Vec<Vec<u8>> = try_get!(
                redis_client.lpop(REDIS_KEY, NonZeroUsize::new(batch_size.get() - 1)),
                error => "Failed to pop the data from Redis: {error}"
            );
            let Ok(_) = data_sender.send((data, processed_agent_data)) else {
                break;
            };
        } else {
            try_execute!(
                redis_client.lpush::<_, _, ()>(REDIS_KEY, payload),
                error => "Failed to push the data to Redis: {error}"
            );
        }
    }

    handle.await?;

    Ok(())
}

#[instrument(skip_all)]
async fn send_data_to_store_api(
    mut store_api_client: StoreClient<Channel>,
    mut data_receiver: tokio::sync::mpsc::UnboundedReceiver<(Vec<Vec<u8>>, ProcessedAgent)>,
) {
    while let Some((data, processed_agent_data)) = data_receiver.recv().await {
        let data: Vec<proto::ProcessedAgentData> = match data
            .into_iter()
            .map(|data| serde_json::from_slice::<ProcessedAgent>(&data))
            .chain(Some(Ok(processed_agent_data)))
            .map(|result| result.map(Into::into))
            .collect::<Result<_, _>>()
        {
            Ok(value) => value,
            Err(error) => {
                tracing::error!("Failed to decode the data from Redis: {error}");
                continue;
            }
        };
        let request = tonic::Request::new(proto::Input { data });
        let ids = match store_api_client.create_processed_agent_data(request).await {
            Ok(response) => response.into_inner().ids,
            Err(error) => {
                tracing::error!("Failed to send the data to the store API: {error}");
                vec![]
            }
        };
        tracing::info!("Data sent to the store API. Response: {ids:?}");
    }
}
