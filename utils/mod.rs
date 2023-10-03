#[cfg(feature = "maud")]
mod maud_support;
#[cfg(feature = "maud")]
pub use maud_support::*;

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

#[cfg(feature = "auth")]
pub mod auth;

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
