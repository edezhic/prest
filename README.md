**P**rogressive **REST**ful framework that simplifies app development. Its still in early active development and recommended only for personal projects, experimentation and rust/web dev learning. The docs are in the [blog](https://prest.blog/) which is also [made with prest](https://prest.blog/about).

### yet another framework?

Yes. Initial motivation comes from [Rust](https://www.rust-lang.org/) itself - arguably the most [reliable practical language](https://edezhic.medium.com/reliable-software-engineering-with-rust-5bb4553b5d54) with an [amazingly wide ecosystem](https://github.com/rust-unofficial/awesome-rust). It's adoption is growing rapidly, but many newcomers stumble upon onboarding pains and getting lost in the myriads of libraries. So, I decided to build prest in attempts to _make application development simple again_.

**Prest allows building full-stack cross-platform apps for the development cost of writing HTML**. It's based on mature web standards and simpler than well established frameworks like [React Native](https://reactnative.dev/), [Flutter](https://flutter.dev/), and even more recent rust alternatives like [Tauri](https://tauri.app/) and [Dioxus](https://dioxuslabs.com/). However, sometimes rust requires understanding of low-level details which are *mostly* hidden in languages like Javascript and Dart. Also, web apis aren't as all-powerful as native ones, so if you need **direct** access to **mobile** OS bindings then project's complexity might increase dramatically.

### getting started

Docs below assume that you're familiar with Rust. If you aren't yet check out [The Rust Book](https://doc.rust-lang.org/book/) - definitely the best guide with interactive examples which is also available in dozens of languages. Also, I strongly recommend checking out the first three chapters of the [async book](https://rust-lang.github.io/async-book/) to understand how async/await work in rust. Then you can proceed with these basic examples:

1. [Host](https://prest.blog/hello-host) - setting up the server
2. [HTML](https://prest.blog/hello-html) - adding an interface
3. [PWA](https://prest.blog/hhelo-pwa) - making UI [installable](https://developer.mozilla.org/en-US/docs/Web/Progressive_web_apps/Guides/Making_PWAs_installable)

To run locally you'll need the latest stable [rust toolchain](https://rustup.rs/). Most examples are supported on [replit](https://replit.com/) so you can [fork it there](https://replit.com/@eDezhic/prest) and run in the cloud. It includes [rust-analyzer](https://rust-analyzer.github.io/) and I recommend it for local development as well. Some examples require additional setup which is described in their readmes. To build & start them use `cargo run -p EXAMPLE-NAME`

There is also a whole bunch of examples that show how you can use other things with prest:

* all kinds of databases - postgres through [seaorm](https://prest.blog/with-seaorm-postgres) or [diesel](https://prest.blog/with-diesel-postgres), sqlite through [sqlx](https://prest.blog/with-sqlx-sqlite) or [turbosql](https://prest.blog/with-turbosql-sqlite), [mongo](https://prest.blog/with-mongo-driver), [redis](https://prest.blog/with-redis-driver) and even a full rust combo [gluesql + sled](https://prest.blog/with-gluesql-sled)
* authentication, authorization, user and session management with [OpenID/OAuth](https://prest.blog/with-openid-google)
* other templating engines like [Askama](https://prest.blog/with-jinja-askama) which provides Jinja-like syntax
* compilation and bundling of [SASS/SCSS](https://prest.blog/with-grass-scss), [TypeScript](https://prest.blog/with-swc-typescript) and other sources in the build pipeline
* extensive and customizable [logging](https://prest.blog/with-tracing-subscriber), efficient concurrent [scraping](https://prest.blog/with-reqwest-scraper), built-in [HTTPS](https://prest.blog/with-rustls-https) encryption
* even [Large Language Models](https://prest.blog/with-candle-mistral) and [blockchain Smart Contracts](https://prest.blog/with-substrate-contract)!

You can also compile your client [into a native binary](https://prest.blog/into-native) if you need access to system APIs, as well as compile the host [into WebAssembly with a system interface](https://prest.blog/into-wasi). You can even combine the best of both worlds and [create portable wasi binaries](https://github.com/dylibso/hermit). The range of possibilities is so wide that only C and C++ can exceed it, but rust provides much better development and maintenance experience in most cases. To be fair the rust ecosystem is relatively young, but it's growing fast and already has a suprisingly large and diverse set of stable libraries.

### under the hood
Prest itself is a relatively thin wrapper around a whole bunch of rust libs, and it is intended to stay that way for the foreseeable future to enable frequent major changes in a pursuit of building a simple interface over an extendable foundation. So, its existance is only possible thanks to a number of brilliant projects which you can find among the [prest's dependencies](https://github.com/edezhic/prest/blob/main/Cargo.toml) and mentions in the docs.

Architectural inspiration came from [this proof-of-concept](https://github.com/richardanaya/wasm-service) - combination of a rust-based [Service Worker](https://developer.mozilla.org/en-US/docs/Web/API/Service_Worker_API) compiled into [WebAssembly](https://webassembly.org/) with [HTMX](https://htmx.org/) library. This will likely sound pretty wild if you haven't worked with these technologies, but the underlying idea is simple - extend the regular [REST architecture](https://htmx.org/essays/rest-explained/) with a client-side worker that can respond to some of the requests. Thanks to the rich wasm support in rust you can easily cross-compile some of your server code into this worker. Thanks to HTMX you can easily build dynamic UIs without writing a single line of javascript. And thanks to [progressive web capabilities](https://web.dev/what-are-pwas/) this combo easily becomes a native-like installable application.

Another somewhat controversial (yet brilliant in my view) piece of the puzzle is [hyperscript](https://hyperscript.org/) - it can solve >99% of UI tasks in an easier and more maintainable way than JS, it keeps related code and markup right next to each other and it pairs exceptionally well with htmx. They are not dependencies of prest but I highly recommend you to try them out together. It might also be a good time to revisit [DOM events](https://en.wikipedia.org/wiki/DOM_event) since they provide a signalling mechanism that can replace functionality of modern JS state-management libraries. Anyway, if you prefer good old React or other conventional front-end tooling you can use them with prest as well, and for some heavily interactive pieces they might be really good.

Also, there are plenty of [Web APIs](https://fugu-tracker.web.app/) available in rust thanks to [wasm-bindgen](https://github.com/rustwasm/wasm-bindgen) on the client side, like the ones that enable Progressive features of prest. There is also the [WASI](https://github.com/bytecodealliance/wasmtime/blob/main/docs/WASI-intro.md) ecosystem to simplify devops and fascilitate serverless, [WebGPU](https://developer.chrome.com/blog/webgpu-io2023/) for cross-platform AI, complex UIs and games, and many other web-related tech being developed for all kinds of use cases.

### roadmap
This is a hobby project and plans change frequently, but there are things I'd likely work on next:
+ better docs starting with html and pwa tutorials
+ move build utils into a separate crate
+ [htmx types](https://github.com/vidhanio/htmx-types)
+ first release
