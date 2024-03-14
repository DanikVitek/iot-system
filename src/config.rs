use core::fmt;
use std::{
    net::{Ipv6Addr, SocketAddr, ToSocketAddrs},
    str::FromStr,
    sync::Arc,
};

#[cfg(feature = "redis")]
use redis::{ConnectionInfo, IntoConnectionInfo, RedisResult};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Server {
    host: Arc<str>,
    port: u16,
}

#[cfg(feature = "mqtt")]
#[derive(Debug, Clone, Deserialize)]
pub struct Mqtt {
    #[serde(flatten)]
    server: Server,
    pub topic: Arc<str>,
}

impl Server {
    pub fn host(&self) -> Arc<str> {
        Arc::clone(&self.host)
    }

    pub fn port(&self) -> u16 {
        self.port
    }
}

#[cfg(feature = "mqtt")]
impl Mqtt {
    pub fn broker_host(&self) -> Arc<str> {
        self.server.host()
    }

    pub fn broker_port(&self) -> u16 {
        self.server.port
    }

    pub fn topic(&self) -> Arc<str> {
        Arc::clone(&self.topic)
    }

    pub fn broker_address(&self) -> String {
        format!("tcp://{}:{}", self.broker_host(), self.broker_port())
    }
}

impl ToSocketAddrs for Server {
    type Iter = <(&'static str, u16) as ToSocketAddrs>::Iter;

    fn to_socket_addrs(&self) -> std::io::Result<Self::Iter> {
        (&*self.host, self.port).to_socket_addrs()
    }
}

impl TryFrom<&Server> for SocketAddr {
    type Error = std::io::Error;

    fn try_from(value: &Server) -> Result<Self, Self::Error> {
        Ok(value.to_socket_addrs()?.next().unwrap())
    }
}

#[cfg(feature = "mqtt")]
impl From<&Mqtt> for mqtt::CreateOptions {
    fn from(value: &Mqtt) -> Self {
        format!("tcp://{}:{}", value.server.host, value.server.port).into()
    }
}

#[cfg(feature = "redis")]
impl IntoConnectionInfo for &Server {
    fn into_connection_info(self) -> RedisResult<ConnectionInfo> {
        (&*self.host, self.port).into_connection_info()
    }
}

#[cfg(feature = "redis")]
impl IntoConnectionInfo for Server {
    #[inline(always)]
    fn into_connection_info(self) -> RedisResult<ConnectionInfo> {
        (&self).into_connection_info()
    }
}

#[cfg(feature = "tonic")]
impl TryFrom<Server> for tonic::transport::Endpoint {
    type Error = <tonic::transport::Endpoint as TryFrom<String>>::Error;

    //noinspection HttpUrlsUsage
    fn try_from(value: Server) -> Result<Self, Self::Error> {
        if let Ok(addr) = Ipv6Addr::from_str(&value.host) {
            format!("http://[{}]:{}", addr, value.port).try_into()
        } else {
            format!("http://{}:{}", value.host, value.port).try_into()
        }
    }
}

pub trait TryRead<'de>: Deserialize<'de> {
    fn try_read() -> Result<Self, ConfigurationError> {
        let base_path = std::env::current_dir().map_err(ConfigurationError::CurrentDir)?;
        let config_dir = base_path.join("configuration");

        let config = config::Config::builder()
            .add_source(config::File::from(config_dir.join("base")).required(true))
            .add_source({
                let environment: Environment = std::env::var("APP_ENVIRONMENT")?
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

impl From<std::env::VarError> for ConfigurationError {
    fn from(err: std::env::VarError) -> Self {
        Self::Environment(EnvironmentError::VarError(err))
    }
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
