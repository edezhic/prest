**Prest** is a **P**rogressive **REST**ful framework designed to quickly start your cross-platform apps. Beware that [it's still alpha](https://prest.blog/roadmap), but I believe that the [overall idea](https://prest.blog/motivation) has a lot of potential.

Project's [repo](https://github.com/edezhic/prest) contains a bunch of examples which can be run with `cargo run -p NAME`:

- `blog` includes description of the project, available at [prest.blog](https://prest.blog)
- `hello-world` is the simplest example in just 5 LoC
- `hello-world-pwa` is a basic Progressive RESTful app in 30 LoC
- `with-oauth` includes Google OAuth flow and basic user management
- `with-candle-mistral` includes [Mistral](https://mistral.ai/news/announcing-mistral-7b/) LLM using [candle](https://github.com/huggingface/candle) ML framework
- `with-tls` includes HTTPS server
- `with-scraping` includes parser for a news website
- `with-gluesql` includes embedded [sled](http://sled.rs/)-powered [GlueSQL](https://gluesql.org/docs/) DB
- `with-tracing` includes tracing on the host
- `with-ui-typescript` includes TypeScript transpiled into JS, requires [nightly](https://rust-lang.github.io/rustup/concepts/channels.html#working-with-nightly-rust)

To build and run them you'll need the [rust toolchain](https://rustup.rs/). I would also recommend [rust-analyzer](https://rust-analyzer.github.io/) extension for your IDE, and [VS Code](https://code.visualstudio.com/), especially if you want to include TypeScript in your project.
