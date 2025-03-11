# prest

**P**rogressive **REST**ful framework that aims to _make application development simple again_. Even if you are not familiar with Rust yet you might be interested because it's designed to be as beginner-friendly as possible. Tutorials are available in the [blog](https://prest.blog/) which is also built with prest. Beware that its still alpha and can only be recommended for pet projects and training because many breaking changes are expected. 

It ain't easy to compete with laravel, rails, nextjs and many others, but I always wanted a framework which enables simplicity in common development needs and allows **any** customizations/optimizations without switching languages. Rust provides ways to build servers, clients, AIs, blockchains, OS kernels and whatever else you might need, while also being arguably the most [reliable practical language](https://prest.blog/rust). Thanks to a lot of [amazing libraries](https://prest.blog/internals) in the rust ecosystem prest re-exports a comprehensive development toolkit and adds a bunch of integrations and features on top of them for simplicity:

#### Easy start
Create a default rust project, add `prest` dependency, bulk `use` everything from it, add `init` macro and make your `main` async. No massive boilerplates, no custom required CLI tools.

`Cargo.toml`
```toml
[dependencies]
prest = "0.5"
```

`src/main.rs`
```rust
use prest::*;
#[init]
async fn main() {
    ...
}
```

By default it reads the env variables, initializes the runtime, logging, database and other prest subsystems.

#### Server
High-performance, concurrent, intuitively routed. Based on [axum](https://github.com/tokio-rs/axum) so it includes powerful middleware api, simple extractors to get information handlers need from requests and flexible return types. But prest also enchances it with a bunch of additional utilities to get started quickly - just `run().await` your router and everything else will be configured automatically.

```rust
route("/", get("Hello world")).run().await
```

You can also add logic after the `await` in case you want to run smth during the shutdown. 

For deserialization of incoming data there is a small utility extractor `Vals<T>` which extracts fields from the query in GET requests and expects json bodies for other methods, for example:

```rust 
route("/send_data", post(|Vals(data): Vals<Data>| async {/* data will be of type Data */}))
```

Router is built by composing routes and other routers like this:

```rust
let router = route("/path", /* handler */)
    .route("/:path_param", /* handler */)
    .layer(/* middleware */)
    .nest(/* another router */);
```

For more details about routing, handlers and layers check out [axum's docs](https://docs.rs/axum/latest/axum/). Under the hood it's powered by [tokio](https://tokio.rs/) stack for extreme speed and reliability. Also, if you need to access runtime directly prest exports `RT` static which you can use. `Serialize` and `Deserialize` macros (check out [`serde`](https://serde.rs/) for details) are re-exported from prest as well.

#### State
While axum provides a way to manage shared state it's relatively verbose so prest provides a bit easier solution - `state!` macro:

```rust
state!(GLOBAL: String = { sync_function()? });
state!(GLOBAL: String = async { async_function().await? });
```

It works with both sync and async initialization code, and supports `?` operator inside. Under the hood it's using standard [`LazyLock`](https://doc.rust-lang.org/stable/std/sync/struct.LazyLock.html), [`anyhow`](https://docs.rs/anyhow)-based return type to support `?` (but beware that errors in state initialization will panic and stop the app), and tokio's runtime when execution of async code is needed. Variables themselves are statics which can be used anywhere in your codebase, just keep in mind that if you'll need mutable globals you'll have to use `RwLock`, `Mutex` or smth alike.

#### UI
`html!` macro for rust'y templating forked from [maud](https://maud.lambda.xyz/), easy styling with built-in [tailwind](https://tailwindcss.com) classes which are compiled into css inside html, simple client-server interactions with [htmx](https://htmx.org) and it's aliases, unlimited flexibility with [hyperscript](https://hyperscript.org). Smooth UX without separate front-end stacks:

```rust
html!{ 
    nav $"w-full bg-gray-900 rounded-full" {
        input $"text-xs lg:text-md" 
            name="query" 
            post="/search" 
            into="#search-results" {} 
    }
    ...
    div #search-results {"response will be here!"}
}
```

For more details about these tools I suggest checking out their docs, overall they are pretty simple and intuitive by themselves but powerful enough for the vast majority of apps. Default prest bundle includes tailwind's presets and patched htmx version which sends non-GET request payloads in json format to easily use with `Vals` and includes a few other tweaks for better compatability.

#### Database
Embedded DB that works without running separate services based on [GlueSQL](https://gluesql.org/docs) for compatibility with SQL and [sled](https://github.com/spacejam/sled) for high-performance storage. Prest enchances them with the `Storage` macro to automatically derive schema based on usual rust structs, and some helper functions to easily interact with the underlying tables without worrying about SQL injections:

```rust
#[derive(Storage, Deserialize)]
struct Todo {
    id: Uuid,
    task: String,
    done: bool,
}
...
Todo::get_all().await?;
Todo::select_by_task("Buy milk").await?;
Todo::select()
    .filter(col("done").eq(true))
    .order_by("task")
    .rows()
    .await?;

let todo = Todo {
    id: Uuid::now_v7(),
    task: "Buy milk".into(),
    done: false,
};
todo.save().await?;
todo.update_done(true).await?;
assert!(todo.check_done(true).await?);
todo.remove().await?;
```

It's aimed to support all the basic types supported by GlueSQL, `Option`, `Vec`, as well as custom ones which can be serialized/deserialized. As of now `Storage` also requires derived `Deserialize` trait for the DB editor in the...

#### Admin panel
Monitors host system's resources, collects filtered stats for requests/responses with their timings, high-level info and detailed traces, provides read/write GUI to tables, tracks scheduled tasks, and provide controls over remote host in local builds. While blog intentionally exposes access to it for demo purposes (cog in the menu), by default it is protected by...

#### Auth
Session and user management using passwords and OAuth/openID protocols. Based on the built-in DB, [openidconnect-rs](https://github.com/ramosbugs/openidconnect-rs), [axum-login](https://github.com/maxcountryman/axum-login) and [password-auth](https://crates.io/crates/password-auth). Persisted in the built-in DB, can be initiated by leading users to the predefined routes, and can retrieve current auth/user info using extractors:

```rust
html!{ 
    // for username/password flow
    form method="POST" action=(LOGIN_ROUTE) { ... }
    // for oauth flow
    a href=(GOOGLE_LOGIN_ROUTE) {"Login with Google"}
}
...
route("/authorized-only", get(|user: User| async {"Hello world"}));
route("/optional", get(|auth: Auth| async {"auth.user is Option<User>"}));
```

To enable it you'll need the `auth` feature of prest:

```toml
prest = { version = "0.5", features = ["auth"] }
```

If someone requests a route which requires authorization `401 Unauthorized` error will be returned.

#### Schedule
There is also a rust-based cron alternative for background tasks based on [tokio-schedule](https://docs.rs/tokio_schedule) and enchanced with some utilities and integrations. They can be spawned as easy as:

```rust 
// can return either `()` or `Result<(), E: Display>` to enable `?`
RT.every(5).minutes().spawn(|| async { do_smth().await })
RT.every(1).day().at(hour, minute, second).spawn(...) 
```

You can also give names to your scheduled tasks and prest will collect additional stats over their timings and execution results:

```rust
RT.every(3).hours().schedule("my regular task", || async { ... })
RT.every(2).days().at(hour, minute, second).schedule(...) 
```

#### Logs
Logging is powered by [tracing](https://docs.rs/tracing) ecosystem with `trace!`, `debug!`, `info!`, `warn!` and `error!` macros: 

```rust
info!("My variable value is {}", x); // supports same formatting as `format!` macro
```

Prest initializes a subscriber which collects these records into several streams: high-level `INFO`+ level logs are written in html format to be observed in the main admin panel page (and the shell in debug builds), and traces of all levels are also written in a non-blocking fashion to files split by days in the `json` format, which can be also explored through special page in the admin panel. By default prest filters low-level logs of its dependencies to avoid spamming your feeds, and you can also add more filters in the argument to the `init` macro:

```rust
// like in the `scraping` example
#[init(log_filters=[("html5ever", "info"), ("selectors", "info")])]
```

#### Build utils
There is also another prest crate `prest-build`. Unlike usual dependencies it goes into build deps section:

```toml
[build-dependencies]
prest-build = "0.3"
```

It includes a couple of optional features - `sass` and `typescript` which allow transpilation and bundling for typescript/js and sass/scss/css respectfully:

```rust
// paths relative to the build script
bundle_ts("path to main ts/js file");
bundle_sass("path to main sass/scss/css file");
```

And their compiled versions can be embedded with `embed_build_output_as!` macro:

```rust
embed_build_output_as!(BuiltAssets);
...
router.embed(BuiltAssets)
```

They can be requested with the same name as originals but `ts`/`tsx` and `scss`/`sass` extensions will be replaced with `js` and `css` accordingly.

Also, there is a similar and more flexible macro `embed_as!` which can be used with arbitrary folders and files, and this macro is designed to read files from the hard drive as needed in debug builds to avoid slowing down compilation, but in release builds it will embed their contents into the binary and you'll get single-file binary with your whole app in it for convenience and faster file access. These macros generate rust structures which provide access for files' contents and metadata. Here is a snippet from the blogs internals which embeds projects files to render on its pages:

```rust
embed_as!(ExamplesDocs from "../" only "*.md");
embed_as!(ExamplesCode from "../" except "*.md");
```

General syntax:

```rust
embed_as!(StructName from "path" only "path or wildcard"/*, "..."*/ except "path or wildcard"/*, ...*/)
```

Such structs can be embedded into the router like this: `.embed(StructName)`.

#### Deployment
Prest supports 1 click build-upload-start deploy script to linux servers through ssh and sftp, based on docker for cross-platform compilation, and comes with automatically configured TLS based on LetsEncrypt. To make it work you'll need to have docker engine installed, specify the domain in the `Cargo.toml` and provide credentials:

```toml
[package.metadata]
domain = "prest.blog"
```
```sh
# add when starting app locally in the shell or in the .env file
SSH_ADDR=123.232.111.222
SSH_USER=root
SSH_PASSWORD=verystrongpassword
```

And just click the `Deploy` button in the local admin panel! You can also manage deployments there: stop the current one, start a previous one, or cleanup old builds. Note that without domain it will not configure TLS.

Anyway, even with the fastest framework and server users will still have to face network delays and you may want to provide more native-app-like experience so...

#### [PWA](https://web.dev/articles/what-are-pwas)
Core `prest-build` function is to build some of your routes into a WASM-based Service Worker and compose a Progressive Web Application so that your users can install it and access these routes offline. To make it work you'll need to add `wasm-bindgen` dependency and `prest-build` build-dependency, separate host-only from shared host+client code and initialize shared routes in the SW, and add a lil build script:

```toml
[dependencies]
wasm-bindgen = "0.2"
```

```rust
#[wasm_bindgen(start)]
pub fn main() {
    shared_routes().handle_fetch_events()
}
```

```rust
use prest_build::*;
fn main() {
    build_pwa(PWAOptions::new()).unwrap();
}
```

To embed the compiled assets into the host you can use the same `embed_build_output_as!` macro. By default it will only run full PWA build in the `--release` mode to avoid slowing down usual development, but you can use `PWA=debug` env variable to enforce full builds. The general idea is to render some of the templates on the client-side to provide extremely fast responses while also supporting server-side renders as a fallback and indexing mechanism. And get it all with just a few lines of boilerplate code. If PWA experience is not enough for you there is another available option...

#### Native
Running host functionality with a webview for offline-first apps. Somewhat like Electron but with much smaller and faster binaries. Based on the same libraries as [Tauri](https://tauri.app/) but for rust-first apps. To build for desktops just enable webview feature like this:

```toml
prest = { version = "0.5", features = ["webview"] }
```

This is quite different from server-first or PWA apps and require quite different architecture, especially around auth and similar components. For mobile platforms you'll need to do [some work](https://github.com/tauri-apps/wry/blob/dev/MOBILE.md) as of now, but hopefully this will be mostly automated as well.

#### Others
And the story doesn't end here. Prest host includes a graceful shutdown mechanism which awaits currently processing requests and in-progress scheduled tasks before exiting, [`Server Sent Events`](https://developer.mozilla.org/en-US/docs/Web/API/Server-sent_events) utils to easily stream data to the clients, a whole bunch of small utils like `Vals` extractor and `ok()` function which can wrap return values of handler closures to provide to allow using `?` operator inside of them. If you think that prest is missing some feature which may be useful for you or for modern app development in general - please add an issue in [the repo](https://github.com/edezhic/prest)!

### getting started

If you aren't familiar with rust yet I strongly recommend to check out [The Rust Book](https://doc.rust-lang.org/book/) - definitely the best guide with interactive examples available in dozens of languages! Also, I suggest skimming through the first three chapters of the [async book](https://rust-lang.github.io/async-book/) to get an overall understanding how concurrency works in rust. 

Prest tutorials are designed to start from basics and then add more and more features on top:

1. [Todo](https://prest.blog/todo) = basic full-stack todo app in just about 50 lines of code
2. [PWA](https://prest.blog/todo-pwa) = 1 + PWA capabilities and an offline view, ~80 LoC
3. [Auth](https://prest.blog/todo-pwa-auth) = 2 + username+password and Google auth, ~110 LoC
4. [Sync](https://prest.blog/todo-pwa-auth-sync) = 3 + synchronization between clients, ~130 LoC

There are also todo examples with alternative databases - postgres through [seaorm](https://prest.blog/postgres-seaorm) or [diesel](https://prest.blog/postgres-diesel), sqlite through [sqlx](https://prest.blog/sqlite-sqlx) or [turbosql](https://prest.blog/sqlite-turbosql), [mongo](https://prest.blog/mongo-driver), [redis](https://prest.blog/redis-driver). Also, there is a couple of examples that showcase how one might use prest with uncommon for web development tech: [web scraper](https://prest.blog/scraper), [Large Language Model](https://prest.blog/llm-mistral) and [Solana blockchain program](https://prest.blog/solana).

To run locally you'll need the latest stable [rust toolchain](https://rustup.rs/). I also recommend setting up the [rust-analyzer](https://rust-analyzer.github.io/) for your favourite IDE right away. To build & start any example from the cloned prest repo use `cargo run -p EXAMPLE-NAME`. Or just copy the selected example's code from the tutorials into local files and `cargo run` it. Some examples require additional setup and credentials which are mentioned in their docs.