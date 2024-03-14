use color_eyre::Result;
use iot_system::setup_tracing;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let _guard = setup_tracing("./logs", "lab4.log")?;

    Ok(())
}
