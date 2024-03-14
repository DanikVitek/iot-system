use core::fmt;

use chrono::{DateTime, Utc};
use derive_more::{Constructor, Into};
use serde::{de, Deserialize, Deserializer, Serialize};
#[cfg(feature = "utoipa")]
use utoipa::{ToResponse, ToSchema};

#[cfg(feature = "tonic")]
use crate::proto;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize, Constructor)]
#[cfg_attr(feature = "utoipa", derive(ToResponse, ToSchema))]
pub struct Accelerometer {
    x: f64,
    y: f64,
    z: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize, Constructor)]
#[cfg_attr(feature = "utoipa", derive(ToResponse, ToSchema))]
pub struct Gps {
    #[cfg_attr(feature = "utoipa", schema(minimum = -90.0, maximum = 90.0, value_type = f64))]
    latitude: Latitude,
    #[cfg_attr(feature = "utoipa", schema(minimum = -180.0, maximum = 180.0, value_type = f64))]
    longitude: Longitude,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Into)]
#[repr(transparent)]
#[serde(transparent)]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type), sqlx(transparent))]
pub struct Latitude(f64);

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Into)]
#[repr(transparent)]
#[serde(transparent)]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type), sqlx(transparent))]
pub struct Longitude(f64);

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize, Constructor)]
#[cfg_attr(feature = "utoipa", derive(ToResponse, ToSchema))]
pub struct Agent {
    accelerometer: Accelerometer,
    gps: Gps,
    timestamp: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Serialize, Constructor)]
#[cfg_attr(feature = "utoipa", derive(ToResponse, ToSchema))]
pub struct ProcessedAgent {
    #[serde(flatten)]
    agent_data: Agent,
    road_state: RoadState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(ToResponse, ToSchema))]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
#[serde(rename_all = "UPPERCASE")]
pub enum RoadState {
    #[default]
    Smooth,
    Rough,
}

impl ProcessedAgent {
    pub fn agent_data(&self) -> &Agent {
        &self.agent_data
    }
    pub fn road_state(&self) -> RoadState {
        self.road_state
    }
}

impl Accelerometer {
    pub fn x(&self) -> f64 {
        self.x
    }

    pub fn y(&self) -> f64 {
        self.y
    }

    pub fn z(&self) -> f64 {
        self.z
    }
}

impl Gps {
    pub fn longitude(&self) -> Longitude {
        self.longitude
    }

    pub fn latitude(&self) -> Latitude {
        self.latitude
    }
}

impl Agent {
    pub fn accelerometer(&self) -> Accelerometer {
        self.accelerometer
    }

    pub fn gps(&self) -> Gps {
        self.gps
    }

    pub fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }
}

impl<'de> Deserialize<'de> for Latitude {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        try_from_deserialize::<_, _, f64>(deserializer)
    }
}

impl<'de> Deserialize<'de> for Longitude {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        try_from_deserialize::<_, _, f64>(deserializer)
    }
}

#[inline(always)]
fn try_from_deserialize<'de, D, T, U>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    U: Deserialize<'de>,
    T: TryFrom<U>,
    <T as TryFrom<U>>::Error: fmt::Display,
{
    U::deserialize(deserializer).and_then(|v| T::try_from(v).map_err(de::Error::custom))
}
impl TryFrom<f64> for Latitude {
    type Error = InvalidLatitudeError;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        if (-90.0..=90.0).contains(&value) {
            Ok(Latitude(value))
        } else {
            Err(InvalidLatitudeError)
        }
    }
}

#[derive(Debug, Clone, Copy, thiserror::Error)]
#[error("latitude must be in range -90..90")]
pub struct InvalidLatitudeError;

impl TryFrom<f64> for Longitude {
    type Error = InvalidLongitudeError;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        if (-180.0..=180.0).contains(&value) {
            Ok(Longitude(value))
        } else {
            Err(InvalidLongitudeError)
        }
    }
}

#[derive(Debug, Clone, Copy, thiserror::Error)]
#[error("longitude must be in range -180..180")]
pub struct InvalidLongitudeError;

#[cfg(feature = "tonic")]
impl From<proto::AccelerometerData> for Accelerometer {
    fn from(data: proto::AccelerometerData) -> Self {
        Self::new(data.x, data.y, data.z)
    }
}

#[cfg(feature = "tonic")]
impl TryFrom<proto::GpsData> for Gps {
    type Error = InvalidGpsDataError;

    fn try_from(value: proto::GpsData) -> Result<Self, Self::Error> {
        match (
            Latitude::try_from(value.latitude),
            Longitude::try_from(value.longitude),
        ) {
            (Ok(latitude), Ok(longitude)) => Ok(Self::new(latitude, longitude)),
            (Err(err), Ok(_)) => Err(err.into()),
            (Ok(_), Err(err)) => Err(err.into()),
            (Err(_), Err(_)) => Err(InvalidGpsDataError::Both(
                InvalidLatitudeError,
                InvalidLongitudeError,
            )),
        }
    }
}

#[cfg(feature = "tonic")]
#[derive(Debug, thiserror::Error)]
pub enum InvalidGpsDataError {
    #[error("Invalid latitude: {0}")]
    InvalidLatitude(
        #[from]
        #[source]
        InvalidLatitudeError,
    ),
    #[error("Invalid longitude: {0}")]
    InvalidLongitude(
        #[from]
        #[source]
        InvalidLongitudeError,
    ),
    #[error("Invalid latitude and longitude: {0}, {1}")]
    Both(InvalidLatitudeError, InvalidLongitudeError),
}

#[cfg(feature = "tonic")]
impl TryFrom<proto::AgentData> for Agent {
    type Error = InvalidAgentDataError;

    fn try_from(value: proto::AgentData) -> Result<Self, Self::Error> {
        let accelerometer = value
            .accelerometer
            .ok_or(InvalidAgentDataError::MissingAccelerometer)?
            .into();
        let gps = value
            .gps
            .ok_or(InvalidAgentDataError::MissingGps)?
            .try_into()?;
        let timestamp = value
            .timestamp
            .ok_or(InvalidAgentDataError::MissingTimestamp)?
            .try_into()?;
        Ok(Self::new(accelerometer, gps, timestamp))
    }
}

#[cfg(feature = "tonic")]
#[derive(Debug, thiserror::Error)]
pub enum InvalidAgentDataError {
    #[error("Missing GPS data")]
    MissingGps,
    #[error("Missing accelerometer data")]
    MissingAccelerometer,
    #[error("Missing timestamp")]
    MissingTimestamp,
    #[error("Invalid timestamp: {0}")]
    InvalidTimestamp(
        #[from]
        #[source]
        InvalidDateTimeError,
    ),
    #[error("Invalid GPS data: {0}")]
    InvalidGpsData(
        #[from]
        #[source]
        InvalidGpsDataError,
    ),
}

#[cfg(feature = "tonic")]
impl From<proto::RoadState> for RoadState {
    fn from(value: proto::RoadState) -> Self {
        match value {
            proto::RoadState::Smooth => Self::Smooth,
            proto::RoadState::Rough => Self::Rough,
        }
    }
}

#[cfg(feature = "tonic")]
impl From<RoadState> for proto::RoadState {
    fn from(value: RoadState) -> Self {
        match value {
            RoadState::Smooth => Self::Smooth,
            RoadState::Rough => Self::Rough,
        }
    }
}

#[cfg(feature = "tonic")]
impl TryFrom<proto::ProcessedAgentData> for ProcessedAgent {
    type Error = InvalidProcessedAgentDataError;

    fn try_from(value: proto::ProcessedAgentData) -> Result<Self, Self::Error> {
        let road_state = value.road_state().into();
        let agent_data = value
            .agent
            .ok_or(InvalidProcessedAgentDataError::MissingAgentData)?;
        let agent_data = Agent::try_from(agent_data)?;
        Ok(Self::new(agent_data, road_state))
    }
}

#[cfg(feature = "tonic")]
#[derive(Debug, thiserror::Error)]
pub enum InvalidProcessedAgentDataError {
    #[error("Missing agent data")]
    MissingAgentData,
    #[error("Invalid agent data: {0}")]
    InvalidAgentData(
        #[from]
        #[source]
        InvalidAgentDataError,
    ),
}

#[cfg(feature = "tonic")]
impl TryFrom<proto::DateTimeUtc> for DateTime<Utc> {
    type Error = InvalidDateTimeError;

    fn try_from(value: proto::DateTimeUtc) -> Result<Self, Self::Error> {
        DateTime::from_timestamp(value.seconds, value.nanos).ok_or(InvalidDateTimeError)
    }
}

#[cfg(feature = "tonic")]
#[derive(Debug, thiserror::Error)]
#[error("Out-of-range number of seconds and/or invalid nanosecond")]
pub struct InvalidDateTimeError;

#[cfg(feature = "tonic")]
impl From<ProcessedAgent> for proto::ProcessedAgentData {
    fn from(value: ProcessedAgent) -> Self {
        Self {
            agent: Some(value.agent_data.into()),
            road_state: value.road_state as i32,
        }
    }
}

#[cfg(feature = "tonic")]
impl From<Agent> for proto::AgentData {
    fn from(value: Agent) -> Self {
        Self {
            accelerometer: Some(value.accelerometer.into()),
            gps: Some(value.gps.into()),
            timestamp: Some(value.timestamp.into()),
        }
    }
}

#[cfg(feature = "tonic")]
impl From<Accelerometer> for proto::AccelerometerData {
    fn from(Accelerometer { x, y, z }: Accelerometer) -> Self {
        Self { x, y, z }
    }
}

#[cfg(feature = "tonic")]
impl From<Gps> for proto::GpsData {
    fn from(
        Gps {
            latitude,
            longitude,
        }: Gps,
    ) -> Self {
        Self {
            latitude: latitude.into(),
            longitude: longitude.into(),
        }
    }
}

#[cfg(feature = "tonic")]
impl From<DateTime<Utc>> for proto::DateTimeUtc {
    fn from(value: DateTime<Utc>) -> Self {
        Self {
            seconds: value.timestamp(),
            nanos: value.timestamp_subsec_nanos(),
        }
    }
}
