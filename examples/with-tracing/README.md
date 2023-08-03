## Minimal PWRS app with tracing

Basic app + a couple of lines in `host.rs` which provide tracing:
```rust
pwrs::host::init_logging();
...
    .layer(pwrs::host::http_tracing());
```

TODO:
* sw tracing: https://github.com/old-storyai/tracing-wasm
* configurable