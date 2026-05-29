//! Prove the relay routes opaque payloads between two real WebSocket clients on
//! group id alone, and never echoes to the sender.

use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

type Ws = tokio_tungstenite::WebSocketStream<
    tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
>;

async fn next_text(ws: &mut Ws) -> String {
    loop {
        let msg = tokio::time::timeout(Duration::from_secs(5), ws.next())
            .await
            .expect("timed out waiting for frame")
            .expect("stream ended")
            .expect("ws error");
        if let Message::Text(t) = msg {
            return t;
        }
    }
}

#[tokio::test]
async fn relays_opaque_payload_between_clients() {
    // Spawn the relay on an ephemeral port.
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, murmur_server::app()).await.unwrap();
    });

    let url = format!("ws://{addr}/ws");
    let (mut bob, _) = connect_async(&url).await.expect("bob connect");
    let (mut alice, _) = connect_async(&url).await.expect("alice connect");

    // Both subscribe to the same group; wait for the acks so routing is in place.
    bob.send(Message::Text(r#"{"op":"sub","group":"channel-1"}"#.into()))
        .await
        .unwrap();
    assert!(next_text(&mut bob).await.contains("subok"));

    alice
        .send(Message::Text(r#"{"op":"sub","group":"channel-1"}"#.into()))
        .await
        .unwrap();
    assert!(next_text(&mut alice).await.contains("subok"));

    // Alice relays an opaque ciphertext blob.
    alice
        .send(Message::Text(
            r#"{"op":"send","group":"channel-1","body":"OPAQUE-CIPHERTEXT-7f3a"}"#.into(),
        ))
        .await
        .unwrap();

    // Bob receives it; the server routed on group id without reading it.
    let received = next_text(&mut bob).await;
    assert!(received.contains("\"evt\":\"msg\""));
    assert!(received.contains("OPAQUE-CIPHERTEXT-7f3a"));
    assert!(received.contains("channel-1"));

    // Alice must NOT receive her own message back.
    let echo = tokio::time::timeout(Duration::from_millis(400), alice.next()).await;
    assert!(echo.is_err(), "sender should not get an echo");
}

#[tokio::test]
async fn directory_publish_fetch_and_welcome_inbox() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, murmur_server::app()).await.unwrap();
    });
    let url = format!("ws://{addr}/ws");

    let (mut bob, _) = connect_async(&url).await.unwrap();
    let (mut alice, _) = connect_async(&url).await.unwrap();

    // Bob registers and publishes a key package.
    bob.send(Message::Text(r#"{"op":"register","user":"bob"}"#.into()))
        .await
        .unwrap();
    assert!(next_text(&mut bob).await.contains("registered"));
    bob.send(Message::Text(
        r#"{"op":"publishkp","user":"bob","kp":"BOB_KEY_PACKAGE"}"#.into(),
    ))
    .await
    .unwrap();

    // Alice registers, then fetches Bob's key package from the directory.
    alice
        .send(Message::Text(r#"{"op":"register","user":"alice"}"#.into()))
        .await
        .unwrap();
    assert!(next_text(&mut alice).await.contains("registered"));
    alice
        .send(Message::Text(r#"{"op":"fetchkp","user":"bob"}"#.into()))
        .await
        .unwrap();
    let kp_reply = next_text(&mut alice).await;
    assert!(kp_reply.contains("\"evt\":\"kp\"") && kp_reply.contains("BOB_KEY_PACKAGE"));

    // Alice delivers a Welcome to Bob's inbox; Bob receives it tagged with sender.
    alice
        .send(Message::Text(
            r#"{"op":"welcome","to":"bob","body":"WELCOME_BUNDLE_42"}"#.into(),
        ))
        .await
        .unwrap();
    let welcome = next_text(&mut bob).await;
    assert!(welcome.contains("\"evt\":\"welcome\""));
    assert!(welcome.contains("WELCOME_BUNDLE_42"));
    assert!(welcome.contains("\"from\":\"alice\""));
}
