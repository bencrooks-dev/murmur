//! Murmur relay binary. Binds the WebSocket relay on `0.0.0.0:8787`.

use murmur_server::app;

#[tokio::main]
async fn main() {
    let addr = "0.0.0.0:8787";
    let listener = tokio::net::TcpListener::bind(addr).await.expect("bind");
    println!("murmur-server relay listening on ws://{addr}/ws");
    axum::serve(listener, app()).await.expect("serve");
}
