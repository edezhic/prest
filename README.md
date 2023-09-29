**Prest** is a **P**rogressive **REST**ful framework designed to quickly start your cross-platform apps. Beware that it's an early WIP - I've verified that Rust with the dependencies used here provide nice results, so I've decided to try to compose them in a easy-to-start way.

This repo contains a bunch of examples which can be run with `cargo run -p NAME`:

- `hello-world` is the simplest example in just 5 LoC
- `hello-world-pwa` is a basic Progressive RESTful app
- `blog` is a bit fancier and describes what this project is about (WIP)
- `with-oauth` includes Google OAuth flow and basic authz/authn boilerplate
- `with-tls` includes HTTPS server
- `with-scraping` includes parser for a news website
- `with-gluesql` includes embedded sled-powered GlueSQL DB (TODO try adding into SW as well)
- `with-tracing` includes tracing on the host (TODO for the SW - https://github.com/old-storyai/tracing-wasm)

To build and run them you'll need the nightly [rust toolchain](https://rustup.rs/). Also I would recommend to use rust-analyzer extension for your IDE, and VS Code is a nice choice especially if you include TypeScript in your project.
