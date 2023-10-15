**Prest** is a **P**rogressive **REST**ful framework designed to quickly start your cross-platform apps. Beware that [it's still alpha](https://prest.blog/roadmap), but I believe that the [overall idea](https://prest.blog/motivation) has a lot of potential.

Project's [repo](https://github.com/edezhic/prest) contains a bunch of examples which can be run with `cargo run -p NAME`:

- `hello-world` is the simplest example in just a couple of lines of code
- `hello-world-pwa` is a basic Progressive RESTful app in ~25 LoC
- `blog` includes description of the project, available at [prest.blog](https://prest.blog)
- `with-oauth-google` includes Google OAuth flow and basic session + user management
- `with-candle-mistral` includes [Mistral](https://mistral.ai/news/announcing-mistral-7b/) LLM using [candle](https://github.com/huggingface/candle) ML framework
- `with-substrate-contract` includes an [ink!](https://use.ink/)-based contract for [Substrate](https://substrate.io/)-based blockchains like [Polkadot](https://www.polkadot.network/)
- `with-tls` includes [Rustls](https://github.com/rustls/rustls)-based HTTPS server
- `with-scraping` includes [scraper](https://github.com/causal-agent/scraper-based)-based parser for a news website
- `with-sqlx-sqlite` includes SQLx ... SQLite ...
- `with-gluesql-sled` includes embedded [sled](http://sled.rs/)-powered [GlueSQL](https://gluesql.org/docs/) DB
- `with-tracing` includes tracing on the host
- `with-ui-typescript` includes TypeScript transpiled into JS, requires [nightly](https://rust-lang.github.io/rustup/concepts/channels.html#working-with-nightly-rust)

To build and run them you'll need the [rust toolchain](https://rustup.rs/). I would also recommend [rust-analyzer](https://rust-analyzer.github.io/) extension for your IDE, and [VS Code](https://code.visualstudio.com/), especially if you want to include TypeScript in your project.
