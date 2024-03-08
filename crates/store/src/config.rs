use std::net::{Ipv4Addr, SocketAddr, ToSocketAddrs};
use std::str::FromStr;

use color_eyre::eyre::{eyre, Context};
use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;
use sqlx::postgres::PgConnectOptions;

#[derive(Debug, Deserialize)]
pub struct Configuration {
    database: Database,
    server: Server,
}

#[derive(Debug, Deserialize)]
pub struct Database {
    host: String,
    port: u16,
    username: SecretString,
    password: SecretString,
    #[serde(rename = "dbname")]
    name: SecretString,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct Server {
    host: Ipv4Addr,
    port: u16,
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
            .add_source(config::Environment::with_prefix("APP").separator("__"))
            .build()?;

        config.try_deserialize().map_err(Into::into)
    }

    pub fn database(&self) -> &Database {
        &self.database
    }

    pub fn server(&self) -> Server {
        self.server
    }
}

impl Database {
    pub fn connect_options(&self) -> PgConnectOptions {
        PgConnectOptions::new()
            .host(&self.host)
            .port(self.port)
            .username(self.username.expose_secret())
            .password(self.password.expose_secret())
            .database(self.name.expose_secret())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Environment {
    Local,
    Production,
}

impl From<Server> for SocketAddr {
    fn from(value: Server) -> Self {
        SocketAddr::new(value.host.into(), value.port)
    }
}

impl ToSocketAddrs for Server {
    type Iter = <SocketAddr as ToSocketAddrs>::Iter;

    fn to_socket_addrs(&self) -> std::io::Result<Self::Iter> {
        SocketAddr::from(*self).to_socket_addrs()
    }
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
