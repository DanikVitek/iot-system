use std::num::NonZeroUsize;

use iot_system::config::{Mqtt, Server};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Configuration {
    pub store_api: Server,
    pub redis: Server,
    pub batch_size: NonZeroUsize,
    pub mqtt: Mqtt,
}

impl iot_system::config::TryRead<'_> for Configuration {}
