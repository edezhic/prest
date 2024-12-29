use crate::*;
use async_broadcast::{broadcast, Receiver, Sender};
pub use axum::response::sse::{Event as SseEvent, KeepAlive as SseKeepAlive, Sse};
/// Alias for Server Sent Events event
pub type SseItem = Result<SseEvent, std::convert::Infallible>;

// use stream::{Map, TryStream};

/// SseEvent wrapper which holds
#[derive(Clone)]
pub(crate) struct SseEventWrapper<T: Clone + Send> {
    pub event_name: String,
    pub data: T,
}

// unsafe impl<T: Clone + Send> Send for SseEventWrapper<T> {}
/// Broadcasting singleton for SSE (check out todo sync example)
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
    // pub(crate) fn stream(&self) -> Receiver<SseEventWrapper<T>> {
    //     self.receiver.new_receiver()
    // }

    pub async fn send<E: Into<String>>(&self, event_name: E, data: T) -> Result {
        self.sender
            .broadcast_direct(SseEventWrapper {
                event_name: event_name.into(),
                data,
            })
            .await
            .somehow()?;
        Ok(())
    }
}

/// Utility to `stream_and_render` [`SseBroadcast`]s
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

        Sse::new(stream.map(Ok::<axum::response::sse::Event, std::convert::Infallible>))
            .keep_alive(SseKeepAlive::default())
            .into_response()
    }

    // fn subscribe<F, S>(&self, f: F) -> MethodRouter<S> where
    // // S:
    // F: FnMut(&String, T) -> Markup + std::marker::Send + 'static {
    //     get(|| async {})
    // }
}
