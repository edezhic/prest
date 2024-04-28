**P**rogressive **REST**ful framework that _makes application development simple again_. Beware that its still alpha and recommended only for personal projects, as well as rust web development training. Tutorials are available in the [blog](https://prest.blog/) which is also built with prest. Quick peek into prest's hello world:

```rust
use prest::*;
fn main() {
    route("/", get("Hello world")).run()
}
```

### yet another framework?

Yes. Initial motivation came from [Rust](https://www.rust-lang.org/) itself - arguably the most [reliable practical language](https://edezhic.medium.com/reliable-software-engineering-with-rust-5bb4553b5d54). It's adoption is growing rapidly, but most newcomers are getting lost in the myriads of libraries and struggle to build their first apps. So, this project is aiming to provide a batteries-included basic setup.

**Prest allows building full-stack cross-platform apps for the development cost of writing HTML**. Database? Already embedded one. Authentication? Built-in as well. UI? Everything necessary for smooth UX is included. Deployment? Just `cargo build` and you'll get a single all-included server binary that can also distribute an [installable PWA](https://web.dev/articles/what-are-pwas). This is possible thanks to a bunch of amazing dependencies described in the [internals](https://prest.blog/internals).

### available features

The core of prest is built around the usual REST components: async http processing with router, middleware, handlers, state and other utilities. Default features include: embedded sql `db` with query builder and orm-like helpers, macros to easily `embed` any files, `html` macro for rust'y templating, `traces` for extensive and convenient logging. All of the above can be compiled both for the server side and into the client's service worker to achieve powerful progressive enchancement and reuse code.

There is also a couple of optional features:

+ `auth` - session and user management based on passwords or openID
+ `https` - TLS integration based on rustls that avoids openssl linking issues and redirects from http to https.
+ `lazy-embed` - to load embedded files from the filesystem even in the release mode
+ `webview` - running host functionality with a webview for client/offline-first apps. Somewhat like Electron but with much smaller and faster binaries.

PWA build process is just a few lines of code thanks to the `prest-build` utility crate that also has a couple of optional features to bundle other sources with your app:

+ `sass` - SASS/SCSS transpilation and bundling 
+ `typescript` - same for TypeScript/JS

Rust ecosystem has plenty of crates for all kinds of tasks and some of them are showcased in prest's examples. You can also disable any of these features (except core) to speed up compilation or to replace with other tools.

### closer look

short code snippets using features above

### getting started

Further docs assume that you're familiar with rust. If you aren't yet check out [The Rust Book](https://doc.rust-lang.org/book/) - definitely the best guide with interactive examples (available in dozens of languages!). Also, I strongly recommend skimming through the first three chapters of the [async book](https://rust-lang.github.io/async-book/) to get an overall understanding how concurrency works in rust. 

Prest tutorials are designed to start from basics and then add more and more features on top:

1. [Todo](https://prest.blog/todo) = basic full-stack todo app in just about 50 lines of code
2. [PWA](https://prest.blog/todo-pwa) = 1 + PWA capabilities and an offline view, ~80 LoC
3. [Auth](https://prest.blog/todo-pwa-auth) = 2 + username+password and Google auth, ~110 LoC
4. [Sync](https://prest.blog/todo-pwa-auth-sync) = 3 + synchronization between clients, ~160 LoC

There are also todo examples with different databases - postgres through [seaorm](https://prest.blog/postgres-seaorm) or [diesel](https://prest.blog/postgres-diesel), sqlite through [sqlx](https://prest.blog/sqlite-sqlx) or [turbosql](https://prest.blog/sqlite-turbosql), [mongo](https://prest.blog/mongo-driver), [redis](https://prest.blog/redis-driver). Also, there is a couple of examples that showcase how one might use prest with uncommon for web development tech: [web scraper](https://prest.blog/scraper), [Large Language Model](https://prest.blog/llm-mistral) and a [blockchain Smart Contract](https://prest.blog/smart-contract).

To run locally you'll need the latest stable [rust toolchain](https://rustup.rs/). I also recommend setting up the [rust-analyzer](https://rust-analyzer.github.io/) for your favourite IDE right away. To build & start any example from the cloned prest repo use `cargo run -p EXAMPLE-NAME`. Or just copy the selected example's code from the tutorials into local files and `cargo run` it. Some examples require additional setup and credentials which are mentioned in their docs.

### what's next

This is a hobby project and plans change on the fly, but there are things I'd likely work on or consider next:
* db editor
* admin panel docs
+ file storage - https://github.com/dirs-dev/directories-rs
+ i18n - https://github.com/longbridgeapp/rust-i18n
+ register templates for different response codes? Or just use htmx-based error handling (and maybe redirects) in these cases?
+ htmx! macro 
+ sql escaping?
+ does SW DB make sense? Too exotic to make sense?
+ rewrite scraping example
+ rewrite blockchain example
+ add something storybook-like
+ self-update mechanism based on [self-replace](https://github.com/mitsuhiko/self-replace) or [shellflip](https://github.com/cloudflare/shellflip) or smth like that
+ Host build utils? Currently blog is built using `CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER=rust-lld CARGO_PROFILE_RELEASE_LTO=fat cargo build -p blog --release --target=x86_64-unknown-linux-musl` and simply executed 
+ emails https://github.com/stalwartlabs/mail-server

