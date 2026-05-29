//! Murmur relay.
//!
//! A WebSocket relay that (a) routes **opaque** MLS ciphertext between clients on
//! group id alone, (b) hosts a **key-package directory** so a client can fetch a
//! peer's key package to add them, and (c) delivers **Welcome** bundles to a named
//! user's inbox so they can join. The server holds no key material and never
//! inspects ciphertext — it moves bytes; it cannot read them.
//!
//! Phase 3 MVP: in-memory state, no persistence/auth. Postgres (durable history),
//! Redis (multi-node fanout), and real auth are later hardening steps; the wire
//! protocol below does not change when they arrive.

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

#[derive(Default)]
struct Inner {
    /// Every connected client's outbound channel.
    clients: HashMap<ClientId, Outbound>,
    /// Online users → their connection.
    user_to_client: HashMap<String, ClientId>,
    /// Key-package directory: user → published key package (opaque string).
    directory: HashMap<String, String>,
    /// Group id → subscriber connections.
    groups: HashMap<GroupId, HashSet<ClientId>>,
}

#[derive(Clone, Default)]
pub struct AppState {
    inner: Arc<Mutex<Inner>>,
}

impl AppState {
    fn add_client(&self, id: ClientId, tx: Outbound) {
        self.inner.lock().unwrap().clients.insert(id, tx);
    }

    fn register(&self, id: ClientId, user: &str) {
        self.inner
            .lock()
            .unwrap()
            .user_to_client
            .insert(user.to_string(), id);
    }

    fn publish_kp(&self, user: &str, kp: &str) {
        self.inner
            .lock()
            .unwrap()
            .directory
            .insert(user.to_string(), kp.to_string());
    }

    fn fetch_kp(&self, user: &str) -> Option<String> {
        self.inner.lock().unwrap().directory.get(user).cloned()
    }

    /// Deliver a Welcome to `to`'s connection, if online.
    fn deliver_welcome(&self, from: &str, to: &str, body: &str) {
        let g = self.inner.lock().unwrap();
        if let Some(cid) = g.user_to_client.get(to) {
            if let Some(tx) = g.clients.get(cid) {
                let event = serde_json::to_string(&ServerMsg::Welcome {
                    from: from.to_string(),
                    body: body.to_string(),
                })
                .unwrap();
                let _ = tx.send(Message::Text(event));
            }
        }
    }

    fn subscribe(&self, group: &str, id: ClientId) {
        self.inner
            .lock()
            .unwrap()
            .groups
            .entry(group.to_string())
            .or_default()
            .insert(id);
    }

    /// Fan opaque ciphertext to every *other* subscriber of `group`.
    fn relay(&self, group: &str, from: ClientId, body: &str) {
        let g = self.inner.lock().unwrap();
        let Some(subs) = g.groups.get(group) else { return };
        let event = serde_json::to_string(&ServerMsg::Msg {
            group: group.to_string(),
            body: body.to_string(),
        })
        .unwrap();
        for cid in subs.iter() {
            if *cid != from {
                if let Some(tx) = g.clients.get(cid) {
                    let _ = tx.send(Message::Text(event.clone()));
                }
            }
        }
    }

    fn drop_client(&self, id: ClientId, user: &Option<String>, groups: &HashSet<GroupId>) {
        let mut g = self.inner.lock().unwrap();
        g.clients.remove(&id);
        if let Some(u) = user {
            if g.user_to_client.get(u) == Some(&id) {
                g.user_to_client.remove(u);
            }
        }
        for grp in groups {
            if let Some(subs) = g.groups.get_mut(grp) {
                subs.remove(&id);
                if subs.is_empty() {
                    g.groups.remove(grp);
                }
            }
        }
    }
}

/// Client → server control frames (JSON text).
#[derive(Deserialize)]
#[serde(tag = "op", rename_all = "lowercase")]
enum ClientMsg {
    /// Claim a username for this connection.
    Register { user: String },
    /// Publish a key package to the directory under a username.
    Publishkp { user: String, kp: String },
    /// Fetch a user's published key package.
    Fetchkp { user: String },
    /// Deliver a Welcome bundle to a named user's inbox.
    Welcome { to: String, body: String },
    /// Subscribe to a group's traffic.
    Sub { group: String },
    /// Relay opaque ciphertext to a group.
    Send { group: String, body: String },
}

/// Server → client event frames (JSON text).
#[derive(Serialize)]
#[serde(tag = "evt", rename_all = "lowercase")]
enum ServerMsg {
    Registered { user: String },
    /// Key-package lookup result. `kp` is empty if the user is unknown.
    Kp { user: String, kp: String },
    Welcome { from: String, body: String },
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
    state.add_client(id, tx.clone());

    let writer = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if sink.send(msg).await.is_err() {
                break;
            }
        }
    });

    let mut me: Option<String> = None;
    let mut subscribed: HashSet<GroupId> = HashSet::new();

    while let Some(Ok(msg)) = stream.next().await {
        match msg {
            Message::Text(text) => match serde_json::from_str::<ClientMsg>(&text) {
                Ok(ClientMsg::Register { user }) => {
                    state.register(id, &user);
                    me = Some(user.clone());
                    let _ = tx.send(Message::Text(
                        serde_json::to_string(&ServerMsg::Registered { user }).unwrap(),
                    ));
                }
                Ok(ClientMsg::Publishkp { user, kp }) => state.publish_kp(&user, &kp),
                Ok(ClientMsg::Fetchkp { user }) => {
                    let kp = state.fetch_kp(&user).unwrap_or_default();
                    let _ = tx.send(Message::Text(
                        serde_json::to_string(&ServerMsg::Kp { user, kp }).unwrap(),
                    ));
                }
                Ok(ClientMsg::Welcome { to, body }) => {
                    let from = me.clone().unwrap_or_default();
                    state.deliver_welcome(&from, &to, &body);
                }
                Ok(ClientMsg::Sub { group }) => {
                    state.subscribe(&group, id);
                    subscribed.insert(group.clone());
                    let _ = tx.send(Message::Text(
                        serde_json::to_string(&ServerMsg::SubOk { group }).unwrap(),
                    ));
                }
                Ok(ClientMsg::Send { group, body }) => state.relay(&group, id, &body),
                Err(_) => { /* ignore malformed control frames */ }
            },
            Message::Close(_) => break,
            _ => {}
        }
    }

    state.drop_client(id, &me, &subscribed);
    writer.abort();
}
