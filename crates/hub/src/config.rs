use std::{
    net::{Ipv6Addr, ToSocketAddrs},
    num::NonZeroUsize,
    str::FromStr,
    sync::Arc,
};

use redis::{ConnectionInfo, IntoConnectionInfo, RedisResult};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Configuration {
    pub store_api: Server,
    pub redis: Server,
    pub batch_size: NonZeroUsize,
    pub mqtt: Mqtt,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Server {
    host: Arc<str>,
    port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Mqtt {
    #[serde(flatten)]
    server: Server,
    pub topic: Arc<str>,
}

impl iot_system::config::TryRead<'_> for Configuration {}

impl Server {
    pub fn host(&self) -> &str {
        &self.host
    }

    pub fn port(&self) -> u16 {
        self.port
    }
}

impl Mqtt {
    pub fn host(&self) -> &str {
        &self.server.host
    }

    pub fn port(&self) -> u16 {
        self.server.port
    }

    pub fn topic(&self) -> &str {
        &self.topic
    }
}

impl ToSocketAddrs for Server {
    type Iter = <(&'static str, u16) as ToSocketAddrs>::Iter;

    fn to_socket_addrs(&self) -> std::io::Result<Self::Iter> {
        (&*self.host, self.port).to_socket_addrs()
    }
}

impl IntoConnectionInfo for &Server {
    fn into_connection_info(self) -> RedisResult<ConnectionInfo> {
        (&*self.host, self.port).into_connection_info()
    }
}

impl IntoConnectionInfo for Server {
    #[inline(always)]
    fn into_connection_info(self) -> RedisResult<ConnectionInfo> {
        (&self).into_connection_info()
    }
}

impl From<&Mqtt> for paho_mqtt::CreateOptions {
    fn from(value: &Mqtt) -> Self {
        format!("tcp://{}:{}", value.server.host, value.server.port).into()
    }
}

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
