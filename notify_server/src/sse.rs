use crate::{AppEvent, AppState};
use axum::{
    extract::State,
    response::{sse::Event, Sse},
    Extension,
};
use chat_core::User;
use futures::{Stream, StreamExt};
use std::{convert::Infallible, sync::Arc, time::Duration};
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;
use tracing::{info, error, debug};
use dashmap::DashMap;
use async_stream;
use serde_json::to_string;

const CHANNEL_CAPACITY: usize = 256;

pub(crate) async fn sse_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let user_id = user.id as u64;
    let users = state.users.clone();

    let rx = {
        if let Some(tx) = users.get(&user_id) {
            tx.subscribe()
        } else {
            let (tx, rx) = broadcast::channel(CHANNEL_CAPACITY);
            users.insert(user_id, tx);
            rx
        }
    };

    info!("User {} subscribed", user_id);

    let stream = BroadcastStream::new(rx)
        .filter_map(|result| async {
            match result {
                Ok(event) => Some(event),
                Err(err) => {
                    error!("Error in broadcast stream: {:?}", err);
                    None
                }
            }
        })
        .map(|event| {
            let name = match event.as_ref() {
                AppEvent::NewChat(_) => "NewChat",
                AppEvent::AddToChat(_) => "AddToChat",
                AppEvent::RemoveFromChat(_) => "RemoveFromChat",
                AppEvent::NewMessage(_) => "NewMessage",
            };
            let data = to_string(&event).expect("Failed to serialize event");
            debug!("Sending event: {} - {}", name, data);
            Ok(Event::default().data(data).event(name))
        });

    let users_clone = state.users.clone();
    let user_id_clone = user_id;

    let stream = stream.chain(futures::stream::once(async move {
        users_clone.remove(&user_id_clone);
        info!("User {} unsubscribed and cleaned up", user_id_clone);
        if !users_clone.contains_key(&user_id_clone) {
            debug!("Confirmed: User {} removed from DashMap", user_id_clone);
        }
        Ok(Event::default().data("User disconnected").event("disconnect"))
    }));

    let stream = async_stream::stream! {
        let guard = ConnectionDropGuard {
            user_id,
            users: users.clone(),
        };

        for await event in stream {
            yield event;
        }

        drop(guard); // Explicitly drop the guard when the stream ends
    };

    Sse::new(stream)
        .keep_alive(
            axum::response::sse::KeepAlive::new()
                .interval(Duration::from_secs(10))  // Adjusting the interval
                .text("keep-alive-text"),
        )
}

struct ConnectionDropGuard {
    user_id: u64,
    users: Arc<DashMap<u64, broadcast::Sender<Arc<AppEvent>>>>,
}

impl Drop for ConnectionDropGuard {
    fn drop(&mut self) {
        self.users.remove(&self.user_id);
        info!("User {} connection dropped and cleaned up", self.user_id);
        if !self.users.contains_key(&self.user_id) {
            info!("Confirmed: User {} removed from DashMap", self.user_id);
        } else {
            error!("Failed to remove User {} from DashMap", self.user_id);
        }
    }
}