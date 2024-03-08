use std::fmt::{Debug, Display, Formatter};

use chrono::{DateTime, Utc};
pub use iot_system::domain::{Accelerometer, Agent, Gps, Latitude, Longitude, ProcessedAgent};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToResponse, ToSchema};

#[derive(
    Debug,
    Clone,
    Copy,
    Ord,
    PartialOrd,
    Eq,
    PartialEq,
    Hash,
    Deserialize,
    Serialize,
    sqlx::Type,
    IntoParams,
)]
#[repr(transparent)]
#[serde(transparent)]
#[sqlx(transparent)]
#[into_params(names("id"))]
/// ID of the processed agent to read, update, or delete.
pub struct ProcessedAgentId(i32);

#[derive(Debug, Serialize, ToResponse, ToSchema)]
pub struct ProcessedAgentWithId {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(minimum = 1, value_type = i32, nullable = false)]
    id: Option<ProcessedAgentId>,
    #[serde(flatten)]
    #[schema(inline)]
    data: ProcessedAgent,
}

pub trait Dto {
    type Id<'a>;
}

impl Dto for ProcessedAgent {
    type Id<'a> = ProcessedAgentId;
}

impl Dto for [ProcessedAgent] {
    type Id<'a> = &'a [ProcessedAgentId];
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ProcessedAgentDao {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) id: Option<ProcessedAgentId>,
    pub(super) road_state: String,
    pub(super) x: f64,
    pub(super) y: f64,
    pub(super) z: f64,
    pub(super) latitude: Latitude,
    pub(super) longitude: Longitude,
    pub(super) timestamp: DateTime<Utc>,
}

impl Display for ProcessedAgentId {
    #[inline(always)]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl From<ProcessedAgentWithId> for ProcessedAgentDao {
    fn from(agent: ProcessedAgentWithId) -> Self {
        Self {
            id: agent.id,
            road_state: agent.data.road_state,
            x: agent.data.agent_data.accelerometer().x(),
            y: agent.data.agent_data.accelerometer().y(),
            z: agent.data.agent_data.accelerometer().z(),
            latitude: agent.data.agent_data.gps().latitude(),
            longitude: agent.data.agent_data.gps().longitude(),
            timestamp: agent.data.agent_data.timestamp(),
        }
    }
}

impl From<ProcessedAgentDao> for ProcessedAgent {
    fn from(dao: ProcessedAgentDao) -> Self {
        Self {
            agent_data: Agent::new(
                Accelerometer::new(dao.x, dao.y, dao.z),
                Gps::new(dao.latitude, dao.longitude),
                dao.timestamp,
            ),
            road_state: dao.road_state,
        }
    }
}

impl From<ProcessedAgentDao> for ProcessedAgentWithId {
    fn from(dao: ProcessedAgentDao) -> Self {
        Self {
            id: dao.id,
            data: dao.into(),
        }
    }
}
