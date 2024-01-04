**P**rogressive **REST**ful framework that simplifies app development. Its still in early active development and recommended only for personal projects, experimentation and rust/web dev learning. The docs are in the [blog](https://prest.blog/) which is also [made with prest](https://prest.blog/about). Quick peek into prest's hello world:

```rust
use prest::*;
fn main() {
    route("/", get("Hello world")).run()
}
```

### yet another framework?

Yes. Initial motivation came from [Rust](https://www.rust-lang.org/) - arguably the most [reliable practical language](https://edezhic.medium.com/reliable-software-engineering-with-rust-5bb4553b5d54) with an [amazingly wide ecosystem](https://github.com/rust-unofficial/awesome-rust). It's adoption is growing rapidly, but many newcomers stumble upon onboarding pains and getting lost in the myriads of libraries. So, I decided to build prest in attempts to _make application development simple again_.

**Prest allows building full-stack cross-platform apps for the development cost of writing HTML**. Deployment? Just compile and you'll get a single all-included binary. Database? Already embedded one with a query builder for you. Authentication? Session and user management are built-in as well. The fullstack [todo app example](https://prest.blog/app-todo) is just about 50 lines of code total. And there are plenty of examples how to include other tech into your prest app.

It's based on mature web standards instead of custom solutions like [React Native](https://reactnative.dev/) or [Flutter](https://flutter.dev/), and built with full-stack in mind from the very beginning. Modern web capabilities have more than enough for most apps - you can even build games & AI with near-native performance using wasm and [WebGPU](https://developer.chrome.com/blog/webgpu-io2023/), and on the other hand you can relatively easily support old platforms and hardware. 

However, rust requires some understanding of low-level details which are *somewhat* hidden in languages like Javascript and Dart. Prest attempts to keep them as far as possible, but you won't be able to dodge them if you want top performance. Also, web apis aren't as all-powerful as native ones, so if you rely on direct access to OS apis then building with prest is probably not worth it. Anyway, rust has plenty of libraries to work with modern platforms so check it out even without prest!

### getting started

Prest docs assume that you're familiar with rust. If you aren't yet check out [The Rust Book](https://doc.rust-lang.org/book/) - definitely the best guide with interactive examples (available in dozens of languages!). Also, I strongly recommend skimming through the first three chapters of the [async book](https://rust-lang.github.io/async-book/) to get an overall understanding how concurrency works in rust. 

Here are the onboarding examples to the core features of prest:

1. [Hello Host](https://prest.blog/hello-host) - setting up the server
2. [Hello HTML](https://prest.blog/hello-html) - adding an interface
3. [Hello PWA](https://prest.blog/hello-pwa) - making UI [installable](https://developer.mozilla.org/en-US/docs/Web/Progressive_web_apps/Guides/Making_PWAs_installable)

To run locally you'll need the latest stable [rust toolchain](https://rustup.rs/). Many examples are also supported on [replit](https://replit.com/) so you can [fork it there](https://replit.com/@eDezhic/prest-app) and run in the cloud without any setup. It runs [rust-analyzer](https://rust-analyzer.github.io/) and I recommend it for local development as well. To build & start any example from the forked prest repo use `cargo run -p EXAMPLE-NAME`, or just copy the selected example's code from the blog and `cargo run` it. Some examples require additional setup which is described in their docs.

Some of the examples that showcase how to use prest with other things:

* different databases - postgres through [seaorm](https://prest.blog/with-seaorm-postgres) or [diesel](https://prest.blog/with-diesel-postgres), sqlite through [sqlx](https://prest.blog/with-sqlx-sqlite) or [turbosql](https://prest.blog/with-turbosql-sqlite), [mongo](https://prest.blog/with-mongo-driver), [redis](https://prest.blog/with-redis-driver)
* authentication, user and session management with [Google OpenID/OAuth](https://prest.blog/with-openid-google)
* compilation and bundling of [SASS/SCSS](https://prest.blog/with-grass-scss), [TypeScript](https://prest.blog/with-swc-typescript) and other sources in the build pipeline
* other templating engines like [Askama](https://prest.blog/with-jinja-askama) which provides Jinja-like syntax
* even [Large Language Models](https://prest.blog/with-candle-mistral) and [blockchain Smart Contracts](https://prest.blog/with-substrate-contract)!

You can also compile prest apps [into native binaries](https://prest.blog/into-native) if you need access to system APIs or want to distribute as a file, and you can compile the host [into WebAssembly with a system interface](https://prest.blog/into-wasi). You can even combine the best of both worlds and [create portable wasi binaries](https://github.com/dylibso/hermit). The range of possibilities is so wide that only C and C++ can exceed it, but rust provides much better development and maintenance experience in most cases. To be fair the rust ecosystem is relatively young, but it's growing fast and already has a suprisingly large and diverse set of stable libraries.

### under the hood
Prest itself is a relatively thin wrapper around a whole bunch of rust libs, and it is intended to stay that way for the foreseeable future to enable frequent major changes. The initial goal is to come up with a simple interface over an extendable foundation with reasonable defaults. So, its existance is only possible thanks to a number of brilliant projects which you can find among the [prest's dependencies](https://github.com/edezhic/prest/blob/main/Cargo.toml) and mentions in the docs.

Architectural inspiration came from [this proof-of-concept](https://github.com/richardanaya/wasm-service) - combination of a rust-based [Service Worker](https://developer.mozilla.org/en-US/docs/Web/API/Service_Worker_API) compiled into [WebAssembly](https://webassembly.org/) with [HTMX](https://htmx.org/) library. This will likely sound pretty wild if you haven't worked with these technologies, but the underlying idea is simple - extend the regular [REST architecture](https://htmx.org/essays/rest-explained/) with a client-side worker that can respond to some of the requests. Thanks to the rich wasm support in rust you can easily cross-compile rendering code for both server and the service worker. Thanks to HTMX you can easily build dynamic UIs without writing a single line of javascript. And thanks to [progressive web capabilities](https://web.dev/what-are-pwas/) this combo easily becomes a native-like installable application.

While rust allows working with native bindings on any platform, prest is mostly focused on web apis and standards to be as cross-platform and easily distributable as it gets. Nowadays there are plenty of them for all kinds of use cases, check [chromium's](https://fugu-tracker.web.app/) for example. In many cases a bit of js or hyperscript would be easier to make and use on the client side, but you can also work with web apis in rust using [wasm-bindgen](https://github.com/rustwasm/wasm-bindgen) if you want more reliability or aren't comfortable with these languages.

### what's next
This is a hobby project and plans change frequently, but there are things I'd likely work on or consider next:
+ docs updates
+ auth integration (user extractor - https://github.com/maxcountryman/axum-login/commit/1717ead4ba1228807d27191e342e39f549e2ae9c etc)
+ turn tutorials into 2 parts: Backend and Frontend for a Todo app. How to fit state and PWA into it?
