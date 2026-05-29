//! Murmur relay.
//!
//! A WebSocket relay that routes **opaque** MLS ciphertext between clients. The
//! core invariant: the server holds no key material and never inspects payloads —
//! it routes purely on group id (sealed sender). It moves bytes; it cannot read
//! them.
//!
//! Phase 3 MVP: in-memory group → subscriber routing, no persistence. Postgres
//! (durable history) and Redis (multi-node fanout) are a later hardening step;
//! the wire protocol below does not change when they arrive.

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    extract::State,
    response::Response,
    routing::get,
    Router,
};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};

type ClientId = u64;
type GroupId = String;
type Outbound = UnboundedSender<Message>;

/// Shared relay state: for each group, the set of currently-connected subscribers.
#[derive(Clone, Default)]
pub struct AppState {
    inner: Arc<Mutex<HashMap<GroupId, HashMap<ClientId, Outbound>>>>,
}

impl AppState {
    fn subscribe(&self, group: &str, id: ClientId, tx: Outbound) {
        self.inner
            .lock()
            .unwrap()
            .entry(group.to_string())
            .or_default()
            .insert(id, tx);
    }

    /// Fan a payload out to every *other* subscriber of `group`. The body is
    /// opaque ciphertext — never parsed here.
    fn relay(&self, group: &str, from: ClientId, body: &str) {
        let map = self.inner.lock().unwrap();
        if let Some(subs) = map.get(group) {
            let event = serde_json::to_string(&ServerMsg::Msg {
                group: group.to_string(),
                body: body.to_string(),
            })
            .unwrap();
            for (cid, tx) in subs.iter() {
                if *cid != from {
                    let _ = tx.send(Message::Text(event.clone()));
                }
            }
        }
    }

    fn drop_client(&self, id: ClientId, groups: &HashSet<GroupId>) {
        let mut map = self.inner.lock().unwrap();
        for g in groups {
            if let Some(subs) = map.get_mut(g) {
                subs.remove(&id);
                if subs.is_empty() {
                    map.remove(g);
                }
            }
        }
    }
}

/// Client → server control frames (JSON text).
#[derive(Deserialize)]
#[serde(tag = "op", rename_all = "lowercase")]
enum ClientMsg {
    /// Subscribe to a group's traffic.
    Sub { group: String },
    /// Relay opaque ciphertext to the group.
    Send { group: String, body: String },
}

/// Server → client event frames (JSON text).
#[derive(Serialize)]
#[serde(tag = "evt", rename_all = "lowercase")]
enum ServerMsg {
    SubOk { group: String },
    Msg { group: String, body: String },
}

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

/// Build the relay router.
pub fn app() -> Router {
    Router::new()
        .route("/healthz", get(|| async { "ok" }))
        .route("/ws", get(ws_handler))
        .with_state(AppState::default())
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: AppState) {
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    let (mut sink, mut stream) = socket.split();
    let (tx, mut rx) = unbounded_channel::<Message>();

    // Pump queued outbound frames to the socket.
    let writer = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if sink.send(msg).await.is_err() {
                break;
            }
        }
    });

    let mut subscribed: HashSet<GroupId> = HashSet::new();

    while let Some(Ok(msg)) = stream.next().await {
        match msg {
            Message::Text(text) => {
                match serde_json::from_str::<ClientMsg>(&text) {
                    Ok(ClientMsg::Sub { group }) => {
                        state.subscribe(&group, id, tx.clone());
                        subscribed.insert(group.clone());
                        let ack = serde_json::to_string(&ServerMsg::SubOk { group }).unwrap();
                        let _ = tx.send(Message::Text(ack));
                    }
                    Ok(ClientMsg::Send { group, body }) => {
                        // Opaque relay — never inspect `body`.
                        state.relay(&group, id, &body);
                    }
                    Err(_) => { /* ignore malformed control frames */ }
                }
            }
            Message::Close(_) => break,
            _ => {}
        }
    }

    state.drop_client(id, &subscribed);
    writer.abort();
}
