use core::fmt;
use std::str::FromStr;

use serde::Deserialize;

pub trait TryRead<'de>: Deserialize<'de> {
    fn try_read() -> Result<Self, ConfigurationError> {
        let base_path = std::env::current_dir().map_err(ConfigurationError::CurrentDir)?;
        let config_dir = base_path.join("configuration");

        let config = config::Config::builder()
            .add_source(config::File::from(config_dir.join("base")).required(true))
            .add_source({
                let environment: Environment = std::env::var("APP_ENVIRONMENT")
                    .map_err(EnvironmentError::VarError)?
                    .parse()
                    .map_err(EnvironmentError::ParseError)?;
                config::File::from(config_dir.join(environment.as_str())).required(true)
            })
            .add_source(config::Environment::with_prefix("APP").separator("__"))
            .build()?;
        config.try_deserialize().map_err(Into::into)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigurationError {
    #[error("Failed to determine the current directory")]
    CurrentDir(#[source] std::io::Error),
    #[error("{0}")]
    Environment(
        #[from]
        #[source]
        EnvironmentError,
    ),
    #[error("{0}")]
    ConfigError(
        #[from]
        #[source]
        config::ConfigError,
    ),
}

#[derive(Debug, thiserror::Error)]
pub enum EnvironmentError {
    #[error("Failed to read APP_ENVIRONMENT")]
    VarError(
        #[from]
        #[source]
        std::env::VarError,
    ),
    #[error("{0}")]
    ParseError(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Environment {
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
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "local" => Ok(Self::Local),
            "production" => Ok(Self::Production),
            _ => Err(format!("Unknown environment: {s}")),
        }
    }
}

impl fmt::Display for Environment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
