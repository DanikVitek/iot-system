use std::sync::Arc;

use derive_more::Constructor;
use iot_system::{domain, proto, proto::store_server::Store};
use tonic::{self, async_trait};

use crate::{control::ws::Subscribers, service};

#[derive(Clone, Constructor)]
pub struct StoreService {
    subs: Arc<Subscribers>,
    pool: sqlx::PgPool,
}

#[async_trait]
impl Store for StoreService {
    async fn create_processed_agent_data(
        &self,
        request: tonic::Request<proto::Input>,
    ) -> Result<tonic::Response<proto::ProcessedAgentDataId>, tonic::Status> {
        let data: Vec<domain::ProcessedAgent> = request
            .into_inner()
            .data
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<_, domain::InvalidProcessedAgentDataError>>()
            .map_err(|err| tonic::Status::invalid_argument(err.to_string()))?;
        match <[_; 1]>::try_from(data) {
            Ok([data]) => {
                let id = service::create_processed_agent_data(data, &self.subs, &self.pool)
                    .await
                    .map_err(|err| tonic::Status::internal(err.to_string()))?;
                Ok(tonic::Response::new(proto::ProcessedAgentDataId {
                    ids: vec![id.into()],
                }))
            }
            Err(data) if data.is_empty() => Ok(tonic::Response::new(proto::ProcessedAgentDataId {
                ids: vec![],
            })),
            Err(data) => {
                let ids = service::create_processed_agent_data_list(data, &self.subs, &self.pool)
                    .await
                    .map_err(|err| tonic::Status::internal(err.to_string()))?
                    .into_iter()
                    .map(Into::into)
                    .collect();
                Ok(tonic::Response::new(proto::ProcessedAgentDataId { ids }))
            }
        }
    }
}
