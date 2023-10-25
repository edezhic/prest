**prest** is a **P**rogressive **REST**ful framework designed to simplify web and cross-platform app development. Beware that [it's still very early](https://prest.blog/roadmap), but I think that the [overall idea](https://prest.blog/motivation) has potential. To get started you'll need the latest stable [rust toolchain](https://rustup.rs/). Examples currently serve as both tests and docs, you can run them with `cargo run -p NAME`:

- `blog` describes the project at [prest.blog](https://prest.blog)
- `hello-world` is the simplest server
- `hello-pwa` is the simplest [Progressive Web App](https://web.dev/what-are-pwas/)
- `with-typescript` bundles and transpiles TypeScript for the UI
- `with-scss` includes [grass](https://github.com/connorskees/grass)-based [SASS/SCSS](https://sass-lang.com/) compiler
- `with-tracing` includes tracing on the server
- `with-https` includes [Rustls](https://github.com/rustls/rustls)-based HTTPS server
- `with-mongodb` is a todo app powered by the [official rust mongo driver](https://github.com/mongodb/mongo-rust-driver) 
- `with-sqlx-sqlite` is a todo app powered by [SQLx](https://github.com/launchbadge/sqlx)-based connection to [SQLite](https://www.sqlite.org/index.html) DB
- `with-diesel-postgres` is a todo app powered by [Diesel](https://github.com/launchbadge/sqlx) ORM with [PostgreSQL](https://www.postgresql.org/) DB
- `with-gluesql-sled` is a todo app powered by [GlueSQL](https://gluesql.org/docs/)-wrapped [sled](http://sled.rs/) DB
- `with-redis` is a todo app powered by the [redis client](https://github.com/redis-rs/redis-rs) 
- `with-oauth-google` includes [Google OAuth](https://developers.google.com/identity/protocols/oauth2) flow and in-memory session + user management
- `with-candle-mistral` includes [Mistral](https://mistral.ai/news/announcing-mistral-7b/) LLM using [candle](https://github.com/huggingface/candle) ML framework
- `with-substrate-contract` includes an [ink!](https://use.ink/)-based contract for [Substrate](https://substrate.io/)-based blockchains like [Polkadot](https://www.polkadot.network/)
- `with-scraper` includes [scraper](https://github.com/causal-agent/scraper-based)-based parser for a news website
- `with-askama` includes [Askama](https://github.com/djc/askama) (Jinja-like templates)

Most examples are supported on [replit](https://replit.com/) and you can [fork it there](https://replit.com/@eDezhic/prest) to quickly run in the cloud. It includes [rust-analyzer](https://rust-analyzer.github.io/) and I suggest to use it in local development as well. If you also want to include Typescript then [VS Code](https://code.visualstudio.com/) might be a great choice.

Some examples require [nightly](https://rust-lang.github.io/rustup/concepts/channels.html#working-with-nightly-rust) toolchain, [WebAssembly](https://webassembly.org/) target (`rustup target add wasm32-unknown-unknown`), env vars (check out `.env.example`) or other additional setup which you'll be able to easily discover from the errors.

Temporarily deployed to replit by compiling into the `musl` target and including binary into the repo due to [this issue](https://ask.replit.com/t/deployment-time-outs/73694). To rebuild the binary run `cargo build -p blog --target x86_64-unknown-linux-musl --release` and move `target/.../serve` it into the `_temp_deployment` folder.