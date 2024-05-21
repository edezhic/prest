use crate::*;

use tao::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    platform::macos::WindowBuilderExtMacOS,
    window::WindowBuilder,
};
use wry::WebViewBuilder;

pub fn init_webview(url: &str) -> Result {
    let size: LogicalSize<f64> = LogicalSize::from((1280., 720.));
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title_hidden(true)
        .with_inner_size(size)
        .build(&event_loop)?;
    let _webview = WebViewBuilder::new(&window)
        .with_devtools(true)
        .with_url(url)?
        .build()?;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        if let Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } = event
        {
            *control_flow = ControlFlow::Exit
        }
    });
}
