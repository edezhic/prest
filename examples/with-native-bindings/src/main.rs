use prest::*;
use tao::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use wry::WebViewBuilder;

fn main() -> Result<()> {
    std::thread::spawn(|| {
        route(
            "/",
            get(html!((Head::with_title("Native")) h1{"Hello world!"})),
        )
        .run()
    });

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Hello world!")
        .build(&event_loop)?;
    let _webview = WebViewBuilder::new(&window)
        .with_url("http://localhost")?
        .build()?;

    event_loop.run(move |event, _, control_flow| {
        println!("{:?}", event);
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
