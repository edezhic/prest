This is a hobby project and there are no solid long-term plans. However, [the overall idea](https://prest.blog/motivation) seems quite promising to me, so there are things I'll likely build next:

- move substrate contract build and test commands into the build.rs
- simplify gluesql example. Add some gluesql/block_on utils into prest?

### More examples

- `with-seaorm-postgres`
- [with-askama](https://github.com/djc/askama)
- [with-tinytemplate](https://crates.io/crates/tinytemplate)
- [with-tauri](https://beta.tauri.app/)
- `with-mongo`
- `with-hyperscript`
- `with-sass`
- `with-webrtc`? with UI wasm?
- `with-openapi`
- `into-container-scratch`
- `into-wasi`

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
