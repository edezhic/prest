mod llm;
use prest::*;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    start_printing_traces();
    /*
    let mut chat = chat::load(Default::default()).unwrap();
    chat.prompt("It goes like this:").unwrap();
    while let Ok(Some(text)) = chat.answering() {
        print!("{text}");
        use std::io::Write;
        std::io::stdout().flush().unwrap();
    }
     */

    let service = Router::new()
        .route("/", get(|| async { "With Mistral inference!" }))
        .route("/model", get(|| async { todo!("WS handler") }))
        .layer(http_tracing());
    serve(service, Default::default()).await.unwrap();
}
/*
use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Router,
};
use axum_extra::TypedHeader;

use std::borrow::Cow;
use std::ops::ControlFlow;
use std::{net::SocketAddr, path::PathBuf};
use tower_http::{
    services::ServeDir,
    trace::{DefaultMakeSpan, TraceLayer},
};

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

//allows to extract the IP of connecting user
use axum::extract::connect_info::ConnectInfo;
use axum::extract::ws::CloseFrame;

//allows to split the websocket stream into separate TX and RX branches
use futures::{sink::SinkExt, stream::StreamExt};
 */