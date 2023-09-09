# Prest

**Prest** is a **P**rogressive **REST**ful framework designed to quickly start your cross-platform apps. Beware that it's an early WIP - I've verified that rust with the dependencies used here provide a nice development experience, so I've decided to try to compose them in a easy-to-start way: basic PWA is just ~30 lines of code and ~30 lines of cargo config.

This repo contains a bunch of examples which can be run with `cargo run -p NAME`:

- `hello-world` is the simplest example with just 5 LoC
- `pwa` is a basic Progressive RESTful app
- `with-htmx` includes ~fancy UI
- `with-oauth` includes Google OAuth flow and basic authz/authn boilerplate
- `with-tls` includes HTTPS server
- `with-scraping` includes parser for a news website
- `with-gluesql` includes embedded sled-powered GlueSQL DB (TODO try adding into SW as well)
- `with-tracing` includes tracing on the host (TODO for the SW - https://github.com/old-storyai/tracing-wasm)

To build and run them you'll need the nightly [rust toolchain](https://rustup.rs/)

TODO:
- `with-language-model`
- `with-diesel`
- `with-blockchain`
- `with-webrtc`?
- `docs`/``
- `native` wry/tauri stuff
- -host: catch panic and compression layers, other tower-http middleware?

Notes about architectural choices:
* WRY - awesome but I decided to focus on PWA thing, seems to have better platform support and easier to use. 
* WASI - awesome but early. Need wider library support and more system APIs(at least full TLS) to get real.
* Maud - questionable but I love the rusty minimalistic syntax.
* Grass(SCSS) - simple to start and scalable for complex projects, does not enforce anything. 
* TypeScript - type and memory safety all the way down, writing browser code in Rust is painful DX
* Axum - elegance and possibility to use without runtime for the SW.
* Tokio - currently the most popular async runtime
* GlueSQL - uniform and familiar interface over any storage even on the client.
* Rustls - rust all the way down + potentially improved security due to cleaner code.
