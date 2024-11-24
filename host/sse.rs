use crate::*;
pub use axum::response::sse::{Event as SseEvent, KeepAlive as SseKeepAlive, Sse};
use async_broadcast::{broadcast, Receiver, Sender};
/// Alias for Server Sent Events event
pub type SseItem = Result<SseEvent, Infallible>;

// use stream::{Map, TryStream};

#[derive(Clone)]
pub struct SseEventWrapper<T: Clone + Send> {
    pub event_name: String,
    pub data: T,
}

// unsafe impl<T: Clone + Send> Send for SseEventWrapper<T> {}
pub struct SseBroadcast<T: Clone + Send> {
    sender: Sender<SseEventWrapper<T>>,
    receiver: Receiver<SseEventWrapper<T>>,
}

impl<T: Clone + Send> Default for SseBroadcast<T> {
    fn default() -> Self {
        let (sender, receiver) = broadcast(1000);
        Self { sender, receiver }
    }
}

impl<T: Clone + Send> SseBroadcast<T> {
    pub fn stream(&self) -> Receiver<SseEventWrapper<T>> {
        self.receiver.new_receiver()
    }

    pub async fn send<E: Into<String>>(&self, event_name: E, data: T) -> Result {
        self.sender
            .broadcast_direct(SseEventWrapper {
                event_name: event_name.into(),
                data,
            })
            .await
            .map_err(|e| anyhow!("{e}"))?;
        Ok(())
    }
}

pub trait SseBroadcastExt<T: Clone + Send> {
    fn stream_and_render<F>(&self, f: F) -> Response
    where
        F: FnMut(&String, T) -> Markup + std::marker::Send + 'static;

    // fn subscribe<F, S>(&self, f: F) -> MethodRouter<S> where
    // // S:
    // F: FnMut(&String, T) -> Markup + std::marker::Send + 'static;
}

impl<T: Clone + Send + 'static + std::marker::Sync> SseBroadcastExt<T> for SseBroadcast<T> {
    fn stream_and_render<F>(&self, mut f: F) -> Response
    where
        F: (FnMut(&String, T) -> Markup) + std::marker::Send + 'static,
    {
        let stream = self.receiver.new_receiver().map(move |event| {
            let event_name = event.event_name;
            let data = event.data;
            let rendered = f(&event_name, data);
            SseEvent::default().event(event_name).data(rendered.0)
        });

        Sse::new(stream.map(Ok::<axum::response::sse::Event, Infallible>))
            .keep_alive(SseKeepAlive::default())
            .into_response()
    }

    // fn subscribe<F, S>(&self, f: F) -> MethodRouter<S> where
    // // S:
    // F: FnMut(&String, T) -> Markup + std::marker::Send + 'static {
    //     get(|| async {})
    // }
}
