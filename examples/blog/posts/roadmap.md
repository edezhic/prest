This is a hobby project so there are no solid long-term plans. However, [the overall idea](https://prest.blog/motivation) seems quite promising to me, so there are things I'll likely build next:

- more hyperscript for the blog: fix and htmxify markdown links, ...?
- dev hyperscript/wasm ui tools
- move simplest examples into docs?

### More examples

- [with-askama](https://github.com/djc/askama) + `.sass`?
- [with-tinytemplate](https://crates.io/crates/tinytemplate)
- [with-tauri](https://beta.tauri.app/)
- `with-mongo`
- `with-diesel-postgres` on replit?
- `with-openapi`
- `with-ui-wasm`
- `into-container-scratch`
- `into-wasi` [tokio partially supports WASI](https://docs.rs/tokio/latest/tokio/#wasm-support)

### Middleware

Feature? Default? Opt?

Include more of [tower_http](https://docs.rs/tower-http/latest/tower_http/), probably some shared BE/FE layers like tracing ([1](https://github.com/old-storyai/tracing-wasm), [2](https://docs.rs/tracing-chrome/latest/tracing_chrome/)) or smth for [GlueSQL](https://gluesql.org/docs/0.14/getting-started/javascript-web) and other middleware.

### Ergonomics
Less basic boilerplate, more configs with reasonable defaults instead of hardcoded values.

Shorthand for `static X: Lazy<Arc<RwLock<T>>> = ...`
Dep tokio/sync by default for async RwLock?

Include a patched version of rust-embed to avoid having this dep everywhere.

### Publish
It's not on [crates.io](https://crates.io/crates/prest) yet because it depends on the latest unpublished [axum](https://github.com/tokio-rs/axum) and related packages. Awaiting them to upload the first alpha version.
