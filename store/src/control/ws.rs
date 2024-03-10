use std::{
    collections::HashMap,
    sync::{
        atomic::{self, AtomicU64},
        Arc,
    },
};

use actix_web::{
    get,
    web::{self, Bytes},
    HttpRequest, HttpResponse,
};
use actix_ws::{CloseCode, CloseReason};
use iot_system::reclone;
use serde::{ser::SerializeStruct, Serialize, Serializer};
use tokio::{
    runtime::Handle,
    sync::{Mutex, RwLock},
};
use tokio_stream::StreamExt;
use tracing::instrument;

use crate::{data::Dto, error::AppResult};

/// Websocket endpoint for subscribing to processed agent data
#[get("/ws")]
#[instrument(skip_all)]
pub async fn ws_endpoint(
    req: HttpRequest,
    body: web::Payload,
    subscribers: web::Data<Subscribers>,
) -> actix_web::Result<HttpResponse> {
    let (response, session, msg_stream) = actix_ws::handle(&req, body)?;

    actix_web::rt::spawn(ws_handler(
        session,
        msg_stream,
        web::Data::into_inner(subscribers),
    ));

    Ok(response)
}

#[instrument(skip_all)]
async fn ws_handler(
    session: actix_ws::Session,
    mut msg_stream: actix_ws::MessageStream,
    subscribers: Arc<Subscribers>,
) {
    let _id = subscribers.add(session.clone()).await;
    while let Some(msg) = msg_stream.next().await {
        reclone!(mut session);
        match msg {
            Ok(msg) => {
                if let actix_ws::Message::Ping(bytes) = msg {
                    if session.pong(&bytes).await.is_err() {
                        return; // session closed
                    }
                }
            }
            // <editor-fold desc="Error handling" defaultstate="collapsed">
            Err(err) => match err {
                actix_ws::ProtocolError::UnmaskedFrame => {
                    let reason = Some(CloseReason {
                        code: CloseCode::Protocol,
                        description: Some("Received unmasked frame".into()),
                    });
                    if session.close(reason).await.is_err() {
                        return; // session closed
                    }
                }
                actix_ws::ProtocolError::MaskedFrame => {
                    let reason = Some(CloseReason {
                        code: CloseCode::Protocol,
                        description: Some("Received masked frame".into()),
                    });
                    if session.close(reason).await.is_err() {
                        return; // session closed
                    }
                }
                actix_ws::ProtocolError::InvalidOpcode(opcode) => {
                    let reason = Some(CloseReason {
                        code: CloseCode::Protocol,
                        description: Some(format!("Received invalid opcode: {}", opcode)),
                    });
                    if session.close(reason).await.is_err() {
                        return; // session closed
                    }
                }
                actix_ws::ProtocolError::InvalidLength(len) => {
                    let reason = Some(CloseReason {
                        code: CloseCode::Protocol,
                        description: Some(format!("Received invalid length: {}", len)),
                    });
                    if session.close(reason).await.is_err() {
                        return; // session closed
                    }
                }
                actix_ws::ProtocolError::BadOpCode => {
                    let reason = Some(CloseReason {
                        code: CloseCode::Protocol,
                        description: Some("Received bad opcode".into()),
                    });
                    if session.close(reason).await.is_err() {
                        return; // session closed
                    }
                }
                actix_ws::ProtocolError::Overflow => {
                    let reason = Some(CloseReason {
                        code: CloseCode::Size,
                        description: Some("Received message too big".into()),
                    });
                    if session.close(reason).await.is_err() {
                        return; // session closed
                    }
                }
                actix_ws::ProtocolError::ContinuationNotStarted => {
                    let reason = Some(CloseReason {
                        code: CloseCode::Protocol,
                        description: Some("Received continuation frame before start".into()),
                    });
                    if session.close(reason).await.is_err() {
                        return; // session closed
                    }
                }
                actix_ws::ProtocolError::ContinuationStarted => {
                    let reason = Some(CloseReason {
                        code: CloseCode::Protocol,
                        description: Some("Received start frame during continuation".into()),
                    });
                    if session.close(reason).await.is_err() {
                        return; // session closed
                    }
                }
                actix_ws::ProtocolError::ContinuationFragment(opcode) => {
                    let description =
                        format!("Received continuation fragment with opcode: {}", opcode);
                    tracing::error!("{}", description);
                    let reason = Some(CloseReason {
                        code: CloseCode::Protocol,
                        description: Some(description),
                    });
                    if session.close(reason).await.is_err() {
                        return; // session closed
                    }
                }
                actix_ws::ProtocolError::Io(err) => {
                    tracing::error!("I/O error: {}", err);
                    let reason = Some(CloseReason {
                        code: CloseCode::Error,
                        description: Some("I/O error".into()),
                    });
                    if session.close(reason).await.is_err() {
                        return; // session closed
                    }
                }
            },
            // </editor-fold>
        }
    }

    _ = session.close(None).await
}

pub struct Subscribers(RwLock<HashMap<u64, Mutex<actix_ws::Session>>>);

#[derive(Debug)]
pub enum Message<'a, 'b, T: Dto + ?Sized> {
    New { id: T::Id<'b>, data: &'a T },
    Update { id: T::Id<'b>, data: &'a T },
    Delete { id: T::Id<'b> },
}

struct SubscriberId {
    value: u64,
    subscribers: Arc<Subscribers>,
}

impl Subscribers {
    pub fn new() -> Self {
        Subscribers(RwLock::new(HashMap::new()))
    }

    async fn add(self: Arc<Self>, session: actix_ws::Session) -> SubscriberId {
        let mut subscribers = self.0.write().await;

        let id = self.next_id();
        subscribers.insert(id, Mutex::new(session));

        SubscriberId {
            value: id,
            subscribers: Arc::clone(&self),
        }
    }

    fn next_id(&self) -> u64 {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        NEXT_ID.fetch_add(1, atomic::Ordering::Relaxed)
    }

    pub async fn broadcast<'a, 'b, T>(&self, msg: Message<'a, 'b, T>) -> AppResult<()>
    where
        T: Serialize + Dto + ?Sized,
        <T as Dto>::Id<'b>: Serialize,
    {
        let data: Bytes = serde_json::to_vec(&msg)?.into();

        let subscribers = self.0.read().await;
        let mut to_remove = Vec::new();
        for (&id, subscriber) in subscribers.iter() {
            let mut subscriber = subscriber.lock().await;
            let result = subscriber.binary(Bytes::clone(&data)).await;
            if result.is_err() {
                to_remove.push(id);
            }
        }
        drop(subscribers);

        if !to_remove.is_empty() {
            let mut subscribers = self.0.write().await;
            for id in to_remove {
                subscribers.remove(&id);
            }
        }

        Ok(())
    }
}

impl Default for Subscribers {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for SubscriberId {
    fn drop(&mut self) {
        Handle::current().block_on(async move {
            let mut subscribers = self.subscribers.0.write().await;
            subscribers.remove(&self.value);
        });
    }
}

impl<'a, 'b, T: Dto + ?Sized> Serialize for Message<'a, 'b, T>
where
    T: Serialize,
    T::Id<'b>: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        const KIND_FIELD: &str = "kind";
        const DATA_TYPE_FIELD: &str = "data_type";

        match self {
            Message::New { id, data } => {
                let mut state = serializer.serialize_struct("Message", 3)?;
                state.serialize_field(KIND_FIELD, "new")?;
                state.serialize_field("id", id)?;
                state.serialize_field("data", data)?;
                state.end()
            }
            Message::Update { id, data } => {
                let mut state = serializer.serialize_struct("Message", 3)?;
                state.serialize_field(KIND_FIELD, "update")?;
                state.serialize_field("id", id)?;
                state.serialize_field("data", data)?;
                state.end()
            }
            Message::Delete { id } => {
                let mut state = serializer.serialize_struct("Message", 3)?;
                state.serialize_field(KIND_FIELD, "delete")?;
                state.serialize_field("id", id)?;
                state.serialize_field(DATA_TYPE_FIELD, std::any::type_name::<T>())?;
                state.end()
            }
        }
    }
}
