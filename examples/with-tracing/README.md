## Minimal PWRS app with tracing

Basic app + a couple of lines in `host.rs` which provide tracing:
```rust
pwrs_host::init_logging();
...
    .layer(pwrs_host::http_tracing());
```

TODO:
* sw tracing: https://github.com/old-storyai/tracing-wasm
* configurable