use prest::*;
use wry::{
    application::{
      event::{Event, WindowEvent},
      event_loop::{ControlFlow, EventLoop},
      window::WindowBuilder,
    },
    webview::WebViewBuilder,
  };

fn main() -> Result<()> {
    std::thread::spawn({
        let router = Router::new().route("/", get(html!((Head::default()) h1{"Hello world!"})));
        serve(router, Default::default())
    });
    
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Hello world!")
        .build(&event_loop)?;
    let _webview = WebViewBuilder::new(window)?
        .with_url("http://localhost")?
        .build()?;

    event_loop.run(move |event, _, control_flow| {
        println!("{:?}", event);
        *control_flow = ControlFlow::Wait;
        if let Event::WindowEvent { event: WindowEvent::CloseRequested, .. } = event {
            *control_flow = ControlFlow::Exit
        }
    });
}
