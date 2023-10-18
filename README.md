**prest** is a **P**rogressive **REST**ful framework designed to simplify web and cross-platform app development. Beware that [it's still very early](https://prest.blog/roadmap), but I think that the [overall idea](https://prest.blog/motivation) has potential. To get started you'll need the latest stable [rust toolchain](https://rustup.rs/). Examples currently serve as both tests and docs, you can run them with `cargo run -p NAME`:

- `blog` describes the project at [prest.blog](https://prest.blog)
- `hello-world` is the simplest server
- `hello-pwa` is the simplest [Progressive Web App](https://web.dev/what-are-pwas/)
- `with-sqlx-sqlite` includes [SQLx](https://github.com/launchbadge/sqlx)-based connection to [SQLite](https://www.sqlite.org/index.html) DB
- `with-gluesql-sled` includes embedded [sled](http://sled.rs/)-powered [GlueSQL](https://gluesql.org/docs/) DB
- `with-oauth-google` includes [Google OAuth](https://developers.google.com/identity/protocols/oauth2) flow and in-memory session + user management
- `with-candle-mistral` includes [Mistral](https://mistral.ai/news/announcing-mistral-7b/) LLM using [candle](https://github.com/huggingface/candle) ML framework
- `with-substrate-contract` includes an [ink!](https://use.ink/)-based contract for [Substrate](https://substrate.io/)-based blockchains like [Polkadot](https://www.polkadot.network/)
- `with-scraper` includes [scraper](https://github.com/causal-agent/scraper-based)-based parser for a news website
- `with-tracing` includes tracing on the host
- `with-https` includes [Rustls](https://github.com/rustls/rustls)-based HTTPS server
- `with-typescript` includes TypeScript bundle

Most examples are supported on [replit](https://replit.com/) and you can [fork it there](https://replit.com/@eDezhic/prest) to quickly run in the cloud. It includes [rust-analyzer](https://rust-analyzer.github.io/) and I suggest to use it in local development as well. If you also want to include Typescript then [VS Code](https://code.visualstudio.com/) might be a great choice.

Some examples require [nightly](https://rust-lang.github.io/rustup/concepts/channels.html#working-with-nightly-rust) toolchain, [WebAssembly](https://webassembly.org/) target (`rustup target add wasm32-unknown-unknown`), env vars (check out `.env.example`) or other setup.
