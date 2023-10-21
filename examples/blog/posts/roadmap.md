This is a hobby project, there are no solid long-term plans and api is unstable. However, [the overall idea](https://prest.blog/motivation) seems quite promising to me so there are things I'll likely build next:

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

### Publish
It's not on [crates.io](https://crates.io/crates/prest) yet because it depends on the latest unpublished [axum](https://github.com/tokio-rs/axum) and related packages. Awaiting them to upload the first alpha version.

### Other

- logs for the SW: simple `console_log`s under cfg(debug_assrts) or maybe tracing ([1](https://github.com/old-storyai/tracing-wasm), [2](https://docs.rs/tracing-chrome/latest/tracing_chrome/))
- move simplest examples into docs: tracing, TS, https?
- dev hyperscript/wasm ui tools
- GlueSQL on the client - [doc](https://gluesql.org/docs/0.14/getting-started/javascript-web)
