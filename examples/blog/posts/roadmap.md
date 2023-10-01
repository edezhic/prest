This is a hobby project so there are no solid long-term plans, it's still alpha and not even on [crates.io](https://crates.io/crates/prest) yet because the latest [axum](https://github.com/tokio-rs/axum) is not published yet. However, [it seems quite promising to me](https://prest.blog/motivation) so I have some ideas that I'll likely work on next:

### Examples
- [with-candle-mistral](https://github.com/huggingface/candle/tree/main/candle-examples/examples/mistral)
- `with-blockchain` (solana? polkadot? other?)
- `with-diesel` with postgres and sqlite
- [with-askama](https://github.com/djc/askama)
- [with-tinytemplate](https://crates.io/crates/tinytemplate)
- [with-tauri](https://beta.tauri.app/)
- `with-webrtc`?

### Middleware
Include [tower_http](https://docs.rs/tower-http/latest/tower_http/) and probably some shared BE/FE layers like [tracing](https://github.com/old-storyai/tracing-wasm) or smth for [GlueSQL](https://gluesql.org/docs/0.14/getting-started/javascript-web).

### Ergonomics
Less basic boilerplate, more configs with reasonable defaults instead of hardcoded values.
