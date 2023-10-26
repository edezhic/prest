This is a hobby project, there are no solid long-term plans and api is unstable. However, [the overall idea](https://prest.blog/motivation) seems quite promising to me so there are things I'll likely build next:

- update default logo+favicon and how they are used with PWAOptions
- logs for the SW: simple `console_log`s under cfg(debug_assrts) or maybe tracing ([1](https://github.com/old-storyai/tracing-wasm), [2](https://docs.rs/tracing-chrome/latest/tracing_chrome/))
- dev hyperscript/wasm ui tools
- GlueSQL on the client - [doc](https://gluesql.org/docs/0.14/getting-started/javascript-web)

### Publish
It's not on [crates.io](https://crates.io/crates/prest) yet because it depends on the latest unpublished [axum](https://github.com/tokio-rs/axum) and related packages. Awaiting them to upload the first alpha version.
