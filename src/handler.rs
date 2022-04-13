use crate::message::Message;
use crate::state::State;
use axum::{
    extract::ws::{WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    Extension,
};
use futures::{
    sink::SinkExt,
    stream::{SplitSink, SplitStream, StreamExt},
};
use std::{convert::From, sync::Arc};
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};

pub async fn handler(
    ws: WebSocketUpgrade,
    Extension(state): Extension<Arc<State>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: Arc<State>) {
    let (this_sender, this_receiver) = socket.split();

    let (destination_socket, _) = connect_async(state.destination_url.clone())
        .await
        .expect("Failed.");
    let (destination_sender, destination_reader) = destination_socket.split();

    tokio::spawn(handle_to_client(this_sender, destination_reader));
    tokio::spawn(handle_from_client(this_receiver, destination_sender));
}

async fn handle_from_client(
    mut this_receiver: SplitStream<WebSocket>,
    mut destination_sender: SplitSink<
        WebSocketStream<MaybeTlsStream<TcpStream>>,
        tokio_tungstenite::tungstenite::Message,
    >,
) {
    while let Some(msg) = this_receiver.next().await {
        let msg = if let Ok(msg) = msg {
            msg
        } else {
            return;
        };

        if destination_sender
            .send(Message::from(msg).into())
            .await
            .is_err()
        {
            return;
        }
    }
}

async fn handle_to_client(
    mut this_sender: SplitSink<WebSocket, axum::extract::ws::Message>,
    mut destination_receiver: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
) {
    while let Some(msg) = destination_receiver.next().await {
        let msg = if let Ok(msg) = msg {
            msg
        } else {
            return;
        };

        if this_sender.send(Message::from(msg).into()).await.is_err() {
            return;
        }
    }
}
