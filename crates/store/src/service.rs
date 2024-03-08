use std::num::{NonZeroU32, NonZeroU8};

use sqlx::PgPool;
use tracing::instrument;

use crate::{
    control::ws::{Message, Subscribers},
    data::{repo, ProcessedAgent, ProcessedAgentId, ProcessedAgentWithId},
    error::AppResult,
};

#[instrument(skip(subs, pool))]
pub async fn create_processed_agent_data(
    data: ProcessedAgent,
    subs: &Subscribers,
    pool: &PgPool,
) -> AppResult<ProcessedAgentId> {
    let id = repo::insert_processed_agent_data(&data, pool).await?;
    subs.broadcast(Message::New { id, data: &data }).await?;

    Ok(id)
}

#[instrument(skip(subs, pool))]
pub async fn create_processed_agent_data_list(
    data: Vec<ProcessedAgent>,
    subs: &Subscribers,
    pool: &PgPool,
) -> AppResult<Vec<ProcessedAgentId>> {
    let ids = repo::insert_processed_agent_data_list(&data, pool).await?;
    subs.broadcast(Message::New {
        id: ids.as_slice(),
        data: data.as_slice(),
    })
    .await?;

    Ok(ids)
}

#[instrument(skip(pool))]
pub async fn fetch_processed_agent_data(
    id: ProcessedAgentId,
    pool: &PgPool,
) -> AppResult<Option<ProcessedAgent>> {
    Ok(repo::select_processed_agent_data(id, pool).await?)
}

#[instrument(skip(pool))]
pub async fn fetch_processed_agent_data_list(
    page: NonZeroU32,
    size: NonZeroU8,
    pool: &PgPool,
) -> AppResult<Vec<ProcessedAgentWithId>> {
    Ok(repo::select_processed_agent_data_list(page, size, pool).await?)
}

#[instrument(skip(pool, subs))]
pub async fn update_processed_agent_data(
    id: ProcessedAgentId,
    data: ProcessedAgent,
    pool: &PgPool,
    subs: &Subscribers,
) -> AppResult<bool> {
    let updated = repo::update_processed_agent_data(id, &data, pool).await?;
    if updated {
        subs.broadcast(Message::Update { id, data: &data }).await?;
    }

    Ok(updated)
}

#[instrument(skip(pool, subs))]
pub async fn delete_processed_agent_data(
    id: ProcessedAgentId,
    pool: &PgPool,
    subs: &Subscribers,
) -> AppResult<()> {
    let deleted = repo::delete_processed_agent_data(id, pool).await?;
    if deleted {
        subs.broadcast::<ProcessedAgent>(Message::Delete { id })
            .await?;
    }

    Ok(())
}
