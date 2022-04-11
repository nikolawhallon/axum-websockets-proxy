use axum::{
    extract::ws::{WebSocket, WebSocketUpgrade},
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
use std::convert::From;

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

        if destination_sender.send(Message::from(msg).into()).await.is_err() {
            // client disconnected
            return;
        }
    }
}

async fn to_client(
    mut this_sender: SplitSink<WebSocket, axum::extract::ws::Message>,
    mut destination_receiver: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
) {
    while let Some(msg) = destination_receiver.next().await {
        let msg = if let Ok(msg) = msg {
            msg
        } else {
            // client disconnected
            return;
        };

        if this_sender.send(Message::from(msg).into()).await.is_err() {
            // client disconnected
            return;
        }
    }
}

pub enum Message {
    Text(String),
    Binary(Vec<u8>),
    Ping(Vec<u8>),
    Pong(Vec<u8>),
    Close(Option<tungstenite::protocol::CloseFrame<'static>>),
}

impl From<axum::extract::ws::Message> for Message {
    fn from(item: axum::extract::ws::Message) -> Self {
        match item {
            axum::extract::ws::Message::Text(text) => Message::Text(text),
            axum::extract::ws::Message::Binary(binary) => Message::Binary(binary),
            axum::extract::ws::Message::Ping(ping) => Message::Ping(ping),
            axum::extract::ws::Message::Pong(pong) => Message::Pong(pong),
            // will deal with this later
            axum::extract::ws::Message::Close(close) => Message::Close(None),
        }
    }
}

impl From<tokio_tungstenite::tungstenite::Message> for Message {
    fn from(item: tokio_tungstenite::tungstenite::Message) -> Self {
        match item {
            tokio_tungstenite::tungstenite::Message::Text(text) => Message::Text(text),
            tokio_tungstenite::tungstenite::Message::Binary(binary) => Message::Binary(binary),
            tokio_tungstenite::tungstenite::Message::Ping(ping) => Message::Ping(ping),
            tokio_tungstenite::tungstenite::Message::Pong(pong) => Message::Pong(pong),
            // will deal with this later
            tokio_tungstenite::tungstenite::Message::Close(close) => Message::Close(None),
        }
    }
}

// why the hell is this needed
impl From<Message> for axum::extract::ws::Message {
    fn from(item: Message) -> axum::extract::ws::Message {
        match item {
            Message::Text(text) => axum::extract::ws::Message::Text(text),
            Message::Binary(binary) => axum::extract::ws::Message::Binary(binary),
            Message::Ping(ping) => axum::extract::ws::Message::Ping(ping),
            Message::Pong(pong) => axum::extract::ws::Message::Pong(pong),
            // will deal with this later
            Message::Close(close) => axum::extract::ws::Message::Close(None),
        }
    }
}

// why the hell is this needed
impl From<Message> for tokio_tungstenite::tungstenite::Message {
    fn from(item: Message) -> tokio_tungstenite::tungstenite::Message {
        match item {
            Message::Text(text) => tokio_tungstenite::tungstenite::Message::Text(text),
            Message::Binary(binary) => tokio_tungstenite::tungstenite::Message::Binary(binary),
            Message::Ping(ping) => tokio_tungstenite::tungstenite::Message::Ping(ping),
            Message::Pong(pong) => tokio_tungstenite::tungstenite::Message::Pong(pong),
            // will deal with this later
            Message::Close(close) => tokio_tungstenite::tungstenite::Message::Close(None),
        }
    }
}
