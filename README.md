# prest

**P**rogressive **REST**ful framework that _makes application development simple again_. Even if you are not familiar with Rust yet you might be interested because it's designed to be as beginner-friendly as possible. Tutorials are available in the [blog](https://prest.blog/) which is also built with prest. Beware that its still alpha so can be recommended only for pet projects and training because many breaking changes are expected. 

It ain't easy to compete with Laravel, Rails, Nextjs and many others, but I always wanted a framework which enables simplicity in common development needs and allows **any** customizations/optimizations without switching languages. Rust provides ways to build servers, clients, AIs, blockchains, OS kernels and whatever else you might need, while also being arguably the most [reliable practical language](https://prest.blog/rust). Thanks to a lot of [amazing dependencies](https://prest.blog/internals) under the hood prest already provides a whole bunch of features:

**Easy start** - create a default rust project, add `prest` dependency, bulk `use` everything from it, invoke `init!` macro and add your app's logic. No massive boilerplate projects, no custom required CLI tools.

```rust
use prest::*;
fn main() {
    init!();
    ...
}
```

**Server** - high-performance, concurrent, intuitively routed. Includes powerful middleware api, simple extractors to get information handlers need from requests and flexible return types. Just `run` your router and everything else will be configured automatically.

```rust
route("/", get("Hello world")).run()
```

**UI** - `html!` macro for rust'y templating, easy inline styling with built-in [tailwind](https://tailwindcss.com) classes, simple client-server interactions with [htmx](https://htmx.org), unlimited flexibility with [hyperscript](https://hyperscript.org). Smooth UX without separate front-end stacks:

```rust
html!{ 
    nav $"w-full bg-gray-900 rounded-full" {
        input $"mx-auto text-xs lg:text-md" name="search" 
            hx-post="/search" hx-target="#search-results" {} 
    }
    ...
    main {
        div#"search-results" {"Response will be placed here!"}
    }
}
```

**Database** - embedded SQL DB that works without running separate services. Auto-derived schema based on usual rust structs with query builder and helper functions. Just add it into the `init!` macro to make sure it's initialized.

```rust
#[derive(Table, Deserialize)]
struct Todo {
    id: Uuid,
    task: String,
    done: bool,
}
...
init!(tables Todo/*, OtherTable, ... */)
...
Todo::find_all()
Todo::find_by_task("Buy milk")
Todo::select().filter(col("done").eq(true)).order_by("task").values()
todo.save()
todo.update_task("Buy milk and bread")
todo.check_task("Buy milk and bread")
todo.remove()
```

As of now `Table` also requires derived `Deserialize` to enable DB editor in the...

**Admin panel** - collects filtered stats for requests/responses, logs, and provides read/write GUI to all initialized tables. While blog intentionally exposes access to it for demo purposes, by default all built-in and apps routes starting with `/admin` are protected by...

**Auth** - session and user management based on passwords and OAuth/openID. Persisted in the built-in DB, can be initiated by leading users to the predefined routes, and can retrieve current auth/user info using an extractor:

```rust
html!{ 
    // for username/password flow
    form method="POST" action=(LOGIN_ROUTE) { ... }
    // for oauth flow
    a href=(GOOGLE_LOGIN_ROUTE) {"Login with Google"}
}
...
route("/authorized-only", get(|user: User| async {"Hello world"}))
route("/optional", get(|auth: Auth| async {"auth.user is Option<User>"}))
```

To enable it you'll need the `auth` feature of prest:

```toml
prest = { version = "0.4", features = ["auth"] }
```

Note that currently without this feature panel will be public, so you can take a look in the [blog's](https://prest.blog/admin).

**Deployment** - prest supports 1 click build-upload-start deploy pipeline based on docker for cross-platform compilation, and comes with automatically configured TLS based on LetsEncrypt. To make it work you'll need to specify the domain in the `Cargo.toml` and provide credentials:

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

And just click the `Deploy` button in admin panel! It's quite likely that you'll want to provide more native-app-like experience for users so...

**[PWA](https://web.dev/articles/what-are-pwas)** - you can build some of your server and UI code into a WASM-based Service Worker and compose a Progressive Web Application so that your users can install it and use some routes offline. To make it work you'll need to separate host-only from shared host+client code and initialize shared routes in the SW, add `wasm-bindgen` and `prest-build` dependencies, add a lil build script and embed the compiled assets into the host:

```rust
#[wasm_bindgen(start)]
pub fn main() {
    shared_routes().handle_fetch_events()
}
```

```toml
wasm-bindgen = "0.2"
[build-dependencies]
prest-build = "0.3"
```

```rust
use prest_build::*;
fn main() {
    build_pwa(PWAOptions::default()).unwrap();
}
```

```rust
embed_build_output_as!(BuiltAssets);
...
router.embed(BuiltAssets)
```

By default it will only run full PWA build in the `--release` mode to avoid slowing down usual development, but you can use `PWA=debug` env variable to enforce full builds. If PWA experience is not enough for you there is another available option...

**Native** - running host functionality with a webview for offline-first apps. Somewhat like Electron but with much smaller and faster binaries. Based on the same libraries as Tauri but for rust-first apps. To build for desktops just enable webview feature like this:

```toml
prest = { version = "0.4", features = ["webview"] }
```

But for mobile platforms you'll need to do [some work](https://github.com/tauri-apps/wry/blob/dev/MOBILE.md) as of now.

**Build utils** - besides PWA `prest-build` includes a couple of optional features - `sass` and `typescript` which allow transpilation and bundling for typescript/js and sass/scss respectfully:

```rust
// paths relative to the build script
bundle_sass("path to main css/scss/sass file")
bundle_ts("path to main ts/js file")
```

And their compiled versions can be embedded with `embed_build_output_as!` just like PWA assets. Also, there is a similar and more flexible macro `embed_as!` which can be used with arbitrary folders and files, and this macro is designed to read files from the drive as needed in debug builds to avoid slowing down compilation, but in release builds it will embed their contents into the binary and you'll get 1 file with your whole app in it for convenience and faster reading. These macros generate rust structures which provide access for files' contents and metadata like blog is processing to render docs:

```rust
embed_as!(ExamplesDocs from "../" only "*.md");
embed_as!(ExamplesCode from "../" except "*.md");
```

or they can be easily embedded into the router with `.embed(StructName)`.

There is also a rust-based cron alternative for background tasks spawned as easy as:

```rust 
RT.every(5).seconds().spawn(|| async { do_smth().await })
RT.every(1).day().at(hour, minute, second).spawn(...) 
```

Logging with `trace!`, `debug!`, `info!`, `warn!` and `error!` macros, graceful shutdown mechanism, and many other utils.

### getting started

If you aren't familiar with rust yet I strongly recommend to check out [The Rust Book](https://doc.rust-lang.org/book/) - definitely the best guide with interactive examples available in dozens of languages! Also, I suggest skimming through the first three chapters of the [async book](https://rust-lang.github.io/async-book/) to get an overall understanding how concurrency works in rust. 

Prest tutorials are designed to start from basics and then add more and more features on top:

1. [Todo](https://prest.blog/todo) = basic full-stack todo app in just about 50 lines of code
2. [PWA](https://prest.blog/todo-pwa) = 1 + PWA capabilities and an offline view, ~80 LoC
3. [Auth](https://prest.blog/todo-pwa-auth) = 2 + username+password and Google auth, ~110 LoC
4. [Sync](https://prest.blog/todo-pwa-auth-sync) = 3 + synchronization between clients, ~160 LoC

There are also todo examples with alternative databases - postgres through [seaorm](https://prest.blog/postgres-seaorm) or [diesel](https://prest.blog/postgres-diesel), sqlite through [sqlx](https://prest.blog/sqlite-sqlx) or [turbosql](https://prest.blog/sqlite-turbosql), [mongo](https://prest.blog/mongo-driver), [redis](https://prest.blog/redis-driver). Also, there is a couple of examples that showcase how one might use prest with uncommon for web development tech: [web scraper](https://prest.blog/scraper), [Large Language Model](https://prest.blog/llm-mistral), [Solana blockchain program](https://prest.blog/solana) and [Polkadot smart contract](https://prest.blog/polkadot).

To run locally you'll need the latest stable [rust toolchain](https://rustup.rs/). I also recommend setting up the [rust-analyzer](https://rust-analyzer.github.io/) for your favourite IDE right away. To build & start any example from the cloned prest repo use `cargo run -p EXAMPLE-NAME`. Or just copy the selected example's code from the tutorials into local files and `cargo run` it. Some examples require additional setup and credentials which are mentioned in their docs.

### what's next

This is a hobby project and plans change on the fly, but there are things I'd likely work on or consider next:
+ upgrade scraping and polkadot examples, maybe also the llm example
+ web3 and more interactive frontend tools
+ sql escaping?

There are also longer term things which will be needed or nice to have before the release of prest:
* await stable releases of most important dependencies like axum and sled 
* parallel frontend and cranelift backend of the rust compiler for faster builds
* stabilization and support of async iterator and other fundamental concurrent std apis
* more optional configs all around for flexibility
* find a way to include/re-export wasm-bindgen into the prest to avoid need for other deps 
* better Service Worker DX in Rust
* wider range of new examples like [i18n](https://github.com/longbridgeapp/rust-i18n), highly interactive UIs, native mobile builds, webgpu-based modern language model, and others
