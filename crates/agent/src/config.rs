use std::{str::FromStr, sync::Arc, time::Duration};

use color_eyre::eyre::{eyre, Context};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Configuration {
    mqtt: Mqtt,
    delay: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Mqtt {
    broker_host: Arc<str>,
    broker_port: u16,
    topic: Arc<str>,
}

impl Configuration {
    pub fn try_read() -> color_eyre::Result<Self> {
        let base_path =
            std::env::current_dir().context("Failed to determine the current directory")?;
        let config_dir = base_path.join("configuration");

        let config = config::Config::builder()
            .add_source(config::File::from(config_dir.join("base")).required(true))
            .add_source({
                let environment: Environment = std::env::var("APP_ENVIRONMENT")?.parse()?;
                config::File::from(config_dir.join(environment.as_str())).required(true)
            })
            .build()?;
        config.try_deserialize().map_err(Into::into)
    }

    pub fn mqtt(&self) -> &Mqtt {
        &self.mqtt
    }

    pub fn delay(&self) -> Duration {
        Duration::from_secs_f64(self.delay)
    }
}

impl Mqtt {
    pub fn broker_host(&self) -> &str {
        &self.broker_host
    }

    pub fn broker_port(&self) -> u16 {
        self.broker_port
    }

    pub fn topic(&self) -> &str {
        &self.topic
    }

    pub fn broker_address(&self) -> String {
        format!("tcp://{}:{}", self.broker_host, self.broker_port)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Environment {
    Local,
    Production,
}

impl Environment {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::Production => "production",
        }
    }
}

impl FromStr for Environment {
    type Err = color_eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "local" => Ok(Self::Local),
            "production" => Ok(Self::Production),
            _ => Err(eyre!("Unknown environment: {}", s)),
        }
    }
}
