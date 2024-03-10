use std::num::{NonZeroU32, NonZeroU8};

use iot_system::domain::{Latitude, Longitude};
use sqlx::PgPool;

use super::{ProcessedAgent, ProcessedAgentDao, ProcessedAgentId, ProcessedAgentWithId};

pub async fn insert_processed_agent_data_list(
    agents: &[ProcessedAgent],
    pool: &PgPool,
) -> sqlx::Result<Vec<ProcessedAgentId>> {
    let mut tx = pool.begin().await?;

    let mut ids = Vec::with_capacity(agents.len());
    for agent in agents {
        let record = sqlx::query!(
            r#"
            INSERT INTO processed_agent_data (road_state, x, y, z, latitude, longitude, timestamp)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id as "id!: ProcessedAgentId"
            "#,
            agent.road_state(),
            agent.agent_data().accelerometer().x(),
            agent.agent_data().accelerometer().y(),
            agent.agent_data().accelerometer().z(),
            agent.agent_data().gps().latitude() as Latitude,
            agent.agent_data().gps().longitude() as Longitude,
            agent.agent_data().timestamp()
        )
        .fetch_one(&mut *tx)
        .await?;
        ids.push(record.id);
    }

    tx.commit().await?;

    Ok(ids)
}

pub async fn insert_processed_agent_data(
    agent: &ProcessedAgent,
    pool: &PgPool,
) -> sqlx::Result<ProcessedAgentId> {
    let record = sqlx::query!(
        r#"
        INSERT INTO processed_agent_data (road_state, x, y, z, latitude, longitude, timestamp)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id as "id!: ProcessedAgentId"
        "#,
        agent.road_state(),
        agent.agent_data().accelerometer().x(),
        agent.agent_data().accelerometer().y(),
        agent.agent_data().accelerometer().z(),
        agent.agent_data().gps().latitude() as Latitude,
        agent.agent_data().gps().longitude() as Longitude,
        agent.agent_data().timestamp()
    )
    .fetch_one(pool)
    .await?;

    Ok(record.id)
}

pub async fn select_processed_agent_data(
    id: ProcessedAgentId,
    pool: &PgPool,
) -> sqlx::Result<Option<ProcessedAgent>> {
    let record = sqlx::query_as!(
        ProcessedAgentDao,
        r#"
        SELECT
            NULL as "id?: ProcessedAgentId",
            road_state,
            x, y, z,
            latitude as "latitude!: Latitude",
            longitude as "longitude!: Longitude",
            timestamp
        FROM processed_agent_data
        WHERE id = $1
        "#,
        id as ProcessedAgentId
    )
    .fetch_optional(pool)
    .await?;

    Ok(record.map(Into::into))
}

pub async fn select_processed_agent_data_list(
    page: NonZeroU32,
    size: NonZeroU8,
    pool: &PgPool,
) -> sqlx::Result<Vec<ProcessedAgentWithId>> {
    let offset = (page.get() - 1) * size.get() as u32;

    let records = sqlx::query_as!(
        ProcessedAgentDao,
        r#"
        SELECT
            id as "id!: ProcessedAgentId",
            road_state,
            x, y, z,
            latitude as "latitude!: Latitude",
            longitude as "longitude!: Longitude",
            timestamp
        FROM processed_agent_data
        ORDER BY timestamp DESC
        LIMIT $1 OFFSET $2
        "#,
        size.get() as i32,
        offset as i32
    )
    .fetch_all(pool)
    .await?;

    Ok(records.into_iter().map(Into::into).collect())
}

pub async fn update_processed_agent_data(
    id: ProcessedAgentId,
    data: &ProcessedAgent,
    pool: &PgPool,
) -> sqlx::Result<bool> {
    let result = sqlx::query!(
        r#"
        UPDATE processed_agent_data
        SET road_state = $1, x = $2, y = $3, z = $4, latitude = $5, longitude = $6, timestamp = $7
        WHERE id = $8
        "#,
        data.road_state(),
        data.agent_data().accelerometer().x(),
        data.agent_data().accelerometer().y(),
        data.agent_data().accelerometer().z(),
        data.agent_data().gps().latitude() as Latitude,
        data.agent_data().gps().longitude() as Longitude,
        data.agent_data().timestamp(),
        id as ProcessedAgentId
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected() != 0)
}

pub async fn delete_processed_agent_data(
    id: ProcessedAgentId,
    pool: &PgPool,
) -> sqlx::Result<bool> {
    let result = sqlx::query!(
        r#"
        DELETE FROM processed_agent_data
        WHERE id = $1
        "#,
        id as ProcessedAgentId
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected() != 0)
}
