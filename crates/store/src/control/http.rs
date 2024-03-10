use std::{
    fmt,
    num::{NonZeroU32, NonZeroU8},
};

use actix_web::{
    delete, get,
    http::header,
    post, put,
    web::{Data, Json, Path, Query},
    Either, HttpResponse,
};
use serde::{Deserialize, Deserializer};
use tracing::instrument;
use utoipa::IntoParams;

use crate::{
    control::ws,
    data::{ProcessedAgent, ProcessedAgentId, ProcessedAgentWithId},
    service,
};

/// Post a single/list of processed agent data and notify ws subscribers
#[utoipa::path(
    path = "/api/processed-agent-data",
    request_body(
        content = Vec<ProcessedAgent>,
        description = "Processed agent(-s) data to post and notify ws subscribers about",
        examples(
            ("Single" = (
                value = json!({
                    "road_state": "NORMAL",
                    "accelerometer": {
                        "x": 0.0,
                        "y": 0.0,
                        "z": 0.0
                    },
                    "gps": {
                        "latitude": 0.0,
                        "longitude": 0.0
                    },
                    "timestamp": "2023-10-01T00:00:00Z"
                })
            )),
            ("List" = (
                value = json!([{
                    "road_state": "NORMAL",
                    "accelerometer": {
                        "x": 0.0,
                        "y": 0.0,
                        "z": 0.0
                    },
                    "gps": {
                        "latitude": 0.0,
                        "longitude": 0.0
                    },
                    "timestamp": "2023-10-01T00:00:00Z"
                }])
            )),
        )
    ),
    responses(
        (status = 200, description = "List is empty"),
        (
            status = 201,
            headers(("Location" = Vec<String>, description = "Locations of the created resources")),
        ),
        (status = 400, description = "Invalid request body"),
        (status = "5XX", description = "Internal server error")
    )
)]
#[post("/processed-agent-data")]
#[instrument(skip(subs, pool))]
pub async fn create_processed_agent_data(
    data: Either<Json<ProcessedAgent>, Json<Vec<ProcessedAgent>>>,
    subs: Data<ws::Subscribers>,
    pool: Data<sqlx::PgPool>,
) -> actix_web::Result<HttpResponse> {
    let result = match data {
        Either::Right(Json(data)) if data.is_empty() => HttpResponse::Ok().finish(),
        Either::Right(Json(data)) if data.len() == 1 => {
            let [data] = unsafe { <[_; 1] as TryFrom<Vec<_>>>::try_from(data).unwrap_unchecked() };
            let id = service::create_processed_agent_data(data, &subs, &pool).await?;
            HttpResponse::Created()
                .append_header((
                    header::LOCATION,
                    format!(r#"["/api/processed-agent-data/{id}"]"#),
                ))
                .finish()
        }
        Either::Left(Json(data)) => {
            let id = service::create_processed_agent_data(data, &subs, &pool).await?;
            HttpResponse::Created()
                .append_header((header::LOCATION, format!("/api/processed-agent-data/{id}")))
                .finish()
        }
        Either::Right(Json(data)) => {
            let ids = service::create_processed_agent_data_list(data, &subs, &pool).await?;
            let mut response = HttpResponse::Created();
            response.append_header((
                header::LOCATION,
                serde_json::to_string(
                    &ids.into_iter()
                        .map(|id| format!("/api/processed-agent-data/{id}"))
                        .collect::<Vec<_>>(),
                )?,
            ));
            response.finish()
        }
    };
    Ok(result)
}

/// Read a single processed agent data by ID
#[utoipa::path(
    path = "/api/processed-agent-data/{id}",
    params(ProcessedAgentId),
    responses(
        (
            status = 200,
            body = ProcessedAgent,
            description = "A single processed agent data, corresponding to the given id",
            example = json!({
                "road_state": "NORMAL",
                "accelerometer": {
                    "x": 0.0,
                    "y": 0.0,
                    "z": 0.0
                },
                "gps": {
                    "latitude": 0.0,
                    "longitude": 0.0
                },
                "timestamp": "2023-10-01T00:00:00Z"
            }),
        ),
        (status = 400, description = "Invalid ID"),
        (status = 404, description = "Processed agent data not found"),
        (status = "5XX", description = "Internal server error")
    )
)]
#[get("/processed-agent-data/{id}")]
#[instrument(skip(pool))]
pub async fn read_processed_agent_data(
    id: Path<ProcessedAgentId>,
    pool: Data<sqlx::PgPool>,
) -> actix_web::Result<Option<Json<ProcessedAgent>>> {
    let result = service::fetch_processed_agent_data(id.into_inner(), &pool).await?;
    Ok(result.map(Json))
}

/// Read a list of processed agent data
#[utoipa::path(
    path = "/api/processed-agent-data",
    params(Pagination),
    responses(
        (
            status = 200,
            body = Vec<ProcessedAgentWithId>,
            description = "List of processed agent data"
        ),
        (status = 400, description = "Invalid pagination parameters"),
        (status = "5XX", description = "Internal server error")
    )
)]
#[get("/processed-agent-data")]
#[instrument(skip(pool))]
pub async fn read_processed_agent_data_list(
    pagination: Query<Pagination>,
    pool: Data<sqlx::PgPool>,
) -> actix_web::Result<Json<Vec<ProcessedAgentWithId>>> {
    let result =
        service::fetch_processed_agent_data_list(pagination.page.0, pagination.size.0, &pool)
            .await?;
    Ok(Json(result))
}

#[derive(Debug, Default, Deserialize, IntoParams)]
struct Pagination {
    /// The page number, starting from 1
    #[serde(default)]
    #[param(minimum = 1, value_type = u32, default = 1)]
    page: PageNumber,
    /// The number of items per page, between 1 and 20
    #[serde(default)]
    #[param(minimum = 1, maximum = 20, value_type = u8, default = 5)]
    size: PageSize,
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Deserialize)]
#[repr(transparent)]
#[serde(transparent)]
struct PageNumber(NonZeroU32);

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)] // `Deserialize` is derived manually
#[repr(transparent)]
struct PageSize(NonZeroU8);

/// Update a single processed agent data and notify ws subscribers
#[utoipa::path(
    path = "/api/processed-agent-data/{id}",
    params(ProcessedAgentId),
    request_body(
        content = ProcessedAgent,
        description = "New processed agent data to replace the existing one",
        example = json!({
            "road_state": "NORMAL",
            "accelerometer": {
                "x": 0.0,
                "y": 0.0,
                "z": 0.0
            },
            "gps": {
                "latitude": 0.0,
                "longitude": 0.0
            },
            "timestamp": "2023-10-01T00:00:00Z"
        }),
    ),
    responses(
        (status = 204, description = "Processed agent data updated"),
        (status = 400, description = "Invalid ID or request body"),
        (status = 404, description = "Processed agent data for the given ID was not found"),
        (status = "5XX", description = "Internal server error")
    )
)]
#[put("/processed-agent-data/{id}")]
#[instrument(skip(pool, subs))]
pub async fn update_processed_agent_data(
    id: Path<ProcessedAgentId>,
    data: Json<ProcessedAgent>,
    subs: Data<ws::Subscribers>,
    pool: Data<sqlx::PgPool>,
) -> actix_web::Result<HttpResponse> {
    let id = id.into_inner();
    let data = data.into_inner();
    let updated = service::update_processed_agent_data(id, data, &pool, &subs).await?;
    Ok(if updated {
        HttpResponse::NoContent().finish()
    } else {
        HttpResponse::NotFound().finish()
    })
}

/// Delete a single processed agent data and notify ws subscribers
#[utoipa::path(
    path = "/api/processed-agent-data/{id}",
    params(ProcessedAgentId),
    responses(
        (status = 204, description = "Processed agent data deleted or was not present in the first place"),
        (status = 400, description = "Invalid ID"),
        (status = "5XX", description = "Internal server error")
    )
)]
#[delete("/processed-agent-data/{id}")]
#[instrument(skip(pool, subs))]
pub async fn delete_processed_agent_data(
    id: Path<ProcessedAgentId>,
    subs: Data<ws::Subscribers>,
    pool: Data<sqlx::PgPool>,
) -> actix_web::Result<HttpResponse> {
    let id = id.into_inner();
    service::delete_processed_agent_data(id, &pool, &subs).await?;
    Ok(HttpResponse::NoContent().finish())
}

impl Default for PageNumber {
    #[inline(always)]
    fn default() -> Self {
        PageNumber(NonZeroU32::MIN)
    }
}

impl Default for PageSize {
    #[inline(always)]
    fn default() -> Self {
        PageSize(unsafe { NonZeroU8::new(5).unwrap_unchecked() })
    }
}

impl fmt::Display for PageSize {
    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {}",
            self.0,
            if self.0 == NonZeroU8::MIN {
                "item"
            } else {
                "items"
            }
        )
    }
}

impl From<PageSize> for NonZeroU8 {
    #[inline(always)]
    fn from(PageSize(value): PageSize) -> Self {
        value
    }
}

impl<'de> Deserialize<'de> for PageSize {
    fn deserialize<D>(deserializer: D) -> Result<PageSize, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = NonZeroU8::deserialize(deserializer)?;
        match value.get() {
            ..=20 => Ok(PageSize(value)),
            _ => Err(serde::de::Error::custom(
                "page size must be between 1 and 20",
            )),
        }
    }
}
