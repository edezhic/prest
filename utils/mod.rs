use crate::*;

#[macro_export]
macro_rules! template {
    ($($markup:tt)*) => { get(|| async { html!($($markup)*) })};
}

#[macro_export]
macro_rules! redirect {
    ($path:literal) => { all(|| async { Redirect::to($path) })};
}

#[cfg(feature = "sw-bindings")]
mod sw_bindings;
#[cfg(feature = "sw-bindings")]
pub use sw_bindings::*;

#[cfg(feature = "serve")]
mod serve;
#[cfg(feature = "serve")]
pub use serve::*;

#[cfg(feature = "print-traces")]
pub fn start_printing_traces() {
    use tracing_subscriber::{filter::LevelFilter, fmt, prelude::*, Layer};
    let fmt_layer = fmt::Layer::default().with_filter(LevelFilter::DEBUG);
    tracing_subscriber::registry().with(fmt_layer).init();
}

#[cfg(feature = "oauth")]
pub mod oauth;

#[cfg(feature = "dot_env")]
pub fn set_dot_env_variables() {
    dotenv::dotenv().unwrap();
}

#[cfg(feature = "random")]
pub fn generate_secret<T>() -> T 
    where rand::distributions::Standard: rand::prelude::Distribution<T>
{
    rand::Rng::gen::<T>(&mut rand::thread_rng())
}
