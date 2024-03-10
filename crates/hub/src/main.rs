mod config;

use iot_system::{config::TryRead, setup_tracing};

use crate::config::Configuration;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let _guard = setup_tracing("./log", "lab3.log")?;

    let _config = Configuration::try_read()?;
    // let redis_client = redis::Client::open("");

    Ok(())
}
