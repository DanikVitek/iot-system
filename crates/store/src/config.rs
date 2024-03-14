use iot_system::config::Server;
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

impl Configuration {
    pub fn database(&self) -> &Database {
        &self.database
    }

    pub fn http_server(&self) -> &Server {
        &self.http_server
    }

    pub fn grpc_server(&self) -> &Server {
        &self.grpc_server
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
