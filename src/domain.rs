use core::fmt;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize};
#[cfg(feature = "utoipa")]
use utoipa::{ToResponse, ToSchema};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(ToResponse, ToSchema))]
pub struct Accelerometer {
    x: f64,
    y: f64,
    z: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(ToResponse, ToSchema))]
pub struct Gps {
    #[cfg_attr(feature = "utoipa", schema(minimum = -90.0, maximum = 90.0, value_type = f64))]
    latitude: Latitude,
    #[cfg_attr(feature = "utoipa", schema(minimum = -180.0, maximum = 180.0, value_type = f64))]
    longitude: Longitude,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize)]
#[repr(transparent)]
#[serde(transparent)]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type), sqlx(transparent))]
pub struct Latitude(f64);

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize)]
#[repr(transparent)]
#[serde(transparent)]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type), sqlx(transparent))]
pub struct Longitude(f64);

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(ToResponse, ToSchema))]
pub struct Agent {
    accelerometer: Accelerometer,
    gps: Gps,
    timestamp: DateTime<Utc>,
}

impl Accelerometer {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

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
    pub fn new(latitude: Latitude, longitude: Longitude) -> Self {
        Self {
            latitude,
            longitude,
        }
    }

    pub fn longitude(&self) -> Longitude {
        self.longitude
    }

    pub fn latitude(&self) -> Latitude {
        self.latitude
    }
}

impl Agent {
    pub fn new(accelerometer: Accelerometer, gps: Gps, timestamp: DateTime<Utc>) -> Self {
        Self {
            accelerometer,
            gps,
            timestamp,
        }
    }

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
        deserialize::<_, f64, _>(deserializer)
    }
}

impl<'de> Deserialize<'de> for Longitude {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserialize::<_, f64, _>(deserializer)
    }
}

fn deserialize<'de, T, U, D>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: TryFrom<U>,
    U: Deserialize<'de>,
    <T as TryFrom<U>>::Error: fmt::Display,
{
    Deserialize::deserialize(deserializer)
        .and_then(|v| T::try_from(v).map_err(serde::de::Error::custom))
}

impl From<Latitude> for f64 {
    fn from(Latitude(value): Latitude) -> f64 {
        value
    }
}

impl From<Longitude> for f64 {
    fn from(Longitude(value): Longitude) -> f64 {
        value
    }
}

impl TryFrom<f64> for Latitude {
    type Error = &'static str;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        if (-90.0..=90.0).contains(&value) {
            Ok(Latitude(value))
        } else {
            Err("latitude must be in range -90..90")
        }
    }
}

impl TryFrom<f64> for Longitude {
    type Error = &'static str;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        if (-180.0..=180.0).contains(&value) {
            Ok(Longitude(value))
        } else {
            Err("longitude must be in range -180..180")
        }
    }
}
