**prest** is a **P**rogressive **REST**ful framework designed to quickly start your cross-platform apps. Beware that [it's still alpha](https://prest.blog/roadmap), but I believe that the [overall idea](https://prest.blog/motivation) has a lot of potential. To get started you'll need the [rust toolchain](https://rustup.rs/). There is a bunch of examples which can be run with `cargo run -p NAME`:

- `blog` self-description of the project, available at [prest.blog](https://prest.blog)
- `hello-world` is the simplest app
- `hello-world-pwa` is the simplest progressive restful app
- `with-oauth-google` includes Google OAuth flow and in-memory session + user management
- `with-sqlx-sqlite` includes [SQLx](https://github.com/launchbadge/sqlx)-based connection to [SQLite](https://www.sqlite.org/index.html) DB
- `with-gluesql-sled` includes embedded [sled](http://sled.rs/)-powered [GlueSQL](https://gluesql.org/docs/) DB

- `with-candle-mistral` includes [Mistral](https://mistral.ai/news/announcing-mistral-7b/) LLM using [candle](https://github.com/huggingface/candle) ML framework
- `with-substrate-contract` includes an [ink!](https://use.ink/)-based contract for [Substrate](https://substrate.io/)-based blockchains like [Polkadot](https://www.polkadot.network/)
- `with-scraper` includes [scraper](https://github.com/causal-agent/scraper-based)-based parser for a news website
- `with-tracing` includes tracing on the host
- `with-https` includes [Rustls](https://github.com/rustls/rustls)-based HTTPS server
- `with-ui-typescript` includes TypeScript transpiled into JS

Most of the examples are supported on [replit](https://replit.com/) so you can [fork it there](https://replit.com/@eDezhic/prest) to quickly run in the cloud.

For local development I recommend [rust-analyzer](https://rust-analyzer.github.io/) extension for your IDE, and [VS Code](https://code.visualstudio.com/) particularly if you want to include TypeScript in your project. Some of the examples like `with-ui-typescript` require [nightly toolchain](https://rust-lang.github.io/rustup/concepts/channels.html#working-with-nightly-rust).
