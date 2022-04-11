use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Router,
};
use futures::{
    sink::SinkExt,
    stream::{SplitSink, SplitStream, StreamExt},
};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::MaybeTlsStream;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/ws", get(handler));

    axum::Server::bind(&"0.0.0.0:4000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    let (mut this_sender, mut this_receiver) = socket.split();

    let (mut destination_socket, _) = connect_async("ws://localhost:3000/ws")
        .await
        .expect("Failed.");
    let (mut destination_sender, mut destination_reader) = destination_socket.split();

    tokio::spawn(to_client(this_sender, destination_reader));
    tokio::spawn(from_client(this_receiver, destination_sender));
}

async fn from_client(
    mut this_receiver: SplitStream<WebSocket>,
    mut destination_sender: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, tokio_tungstenite::tungstenite::Message>,
) {
    while let Some(msg) = this_receiver.next().await {
        let msg = if let Ok(msg) = msg {
            msg
        } else {
            // client disconnected
            return;
        };

        if destination_sender.send(tokio_tungstenite::tungstenite::Message::Text("test".to_string())).await.is_err() {
            // client disconnected
            return;
        }
    }
}

async fn to_client(
    mut this_sender: SplitSink<WebSocket, Message>,
    mut destination_receiver: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
) {
    while let Some(msg) = destination_receiver.next().await {
        let msg = if let Ok(msg) = msg {
            msg
        } else {
            // client disconnected
            return;
        };

        if this_sender.send(axum::extract::ws::Message::Text("test".to_string())).await.is_err() {
            // client disconnected
            return;
        }
    }
}
