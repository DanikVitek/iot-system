use std::net::{IpAddr, SocketAddr, ToSocketAddrs};

use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;
use sqlx::postgres::PgConnectOptions;

#[derive(Debug, Deserialize)]
pub struct Configuration {
    database: Database,
    http_server: Server,
    grpc_server: Server,
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
    host: IpAddr,
    port: u16,
}

impl Configuration {
    pub fn database(&self) -> &Database {
        &self.database
    }

    pub fn http_server(&self) -> Server {
        self.http_server
    }

    pub fn grpc_server(&self) -> Server {
        self.grpc_server
    }
}

impl iot_system::config::TryRead<'_> for Configuration {}

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

impl From<Server> for SocketAddr {
    fn from(value: Server) -> Self {
        SocketAddr::new(value.host, value.port)
    }
}

impl ToSocketAddrs for Server {
    type Iter = <SocketAddr as ToSocketAddrs>::Iter;

    fn to_socket_addrs(&self) -> std::io::Result<Self::Iter> {
        SocketAddr::from(*self).to_socket_addrs()
    }
}
