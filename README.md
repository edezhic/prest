**Prest** is a **P**rogressive **REST**ful framework designed to quickly start your cross-platform apps and develop in any direction. Beware that it's an early WIP! It works but not published yet, pretty much nothing is stable, it lacks documentaion and tests. However, I believe that the [overall idea](https://prest.blog/motivation) has a lot of potential and I have [plans](https://prest.blog/roadmap) to make it work.

Project's [repo](https://github.com/edezhic/prest) contains a bunch of examples which can be run with `cargo run -p NAME`:

- `blog` includes SCSS and description of the project, available at [prest.blog](https://prest.blog)
- `hello-world` is the simplest example in just 5 LoC
- `hello-world-pwa` is a basic Progressive RESTful app
- `with-oauth` includes Google OAuth flow and basic authz/authn boilerplate
- `with-tls` includes HTTPS server
- `with-scraping` includes parser for a news website
- `with-gluesql` includes embedded sled-powered GlueSQL DB
- `with-tracing` includes tracing on the host
- `with-ui-typescript` includes TypeScript transpiled into JS, requires [nightly](https://rust-lang.github.io/rustup/concepts/channels.html#working-with-nightly-rust)

To build and run them you'll need the [rust toolchain](https://rustup.rs/). Also I would recommend to use rust-analyzer extension for your IDE, and VS Code is a nice choice especially if you include TypeScript and/or SCSS in your project.
