First of all - this is a **Rust-first** project. Some of the challenges could have been easily resolved by including C, JS or other languages, but the goal is to enable comfortable development of full-stack cross-platform apps in a single language while depending only on the default rust tooling. Thanks to the rust's low-level capabilities one can customize/optimize pretty much any app's component and compile it for all kinds of platforms. The range of possibilities is so wide that only C and C++ can exceed it, but rust provides much better development and maintenance experience in most cases. So, rust is an excellent choice as a foundation for any kind of application.

Second but not less important - prest is a **web-focused** framework. It respects [HATEOAS](https://htmx.org/essays/hateoas/) constraint of the REST and focused on HTML to build cross-platform UIs. There are plenty of ways to build UIs and interactions between system's components, but web standards are the most widely supported ones - they can work fine on decades-old hardware and will be supported in the comming decades as well. They are also well-known among the developers and can progressively enchance user experiences with near-native performance using [WebAssembly](https://webassembly.org/) and [WebGPU](https://developer.chrome.com/blog/webgpu-io2023/), which are also well supported in the rust ecosystem.

Architectural inspiration came from [this proof-of-concept](https://github.com/richardanaya/wasm-service) - combination of a rust-based [Service Worker](https://developer.mozilla.org/en-US/docs/Web/API/Service_Worker_API) compiled into wasm with [HTMX](https://htmx.org/) library. This will likely sound pretty wild if you haven't worked with these technologies, but the underlying idea is simple - extend the regular REST architecture with a client-side worker that can respond to some of the requests. Thanks to the [wasm-bindgen](https://github.com/rustwasm/wasm-bindgen) and rust compiler's wasm target support you can easily build rendering code for both server and the service worker. Thanks to HTMX you can easily build dynamic UIs without writing a single line of javascript. And thanks to [progressive web capabilities](https://web.dev/what-are-pwas/) this combo easily becomes a native-like installable application. These days [PWAs can do quite a lot](https://whatpwacando.today/).

### going native

In some cases you might need direct access to the client OS apis to enable some features that aren't supported in web environments (at least yet), so prest aims to support bindings to platforms' built-in webviews to enable distribution of the same restful apps as native packages using [WRY](https://github.com/tauri-apps/wry) and [TAO](https://github.com/tauri-apps/tao). This way you can start building a usual web app and reuse it's code to build native client packages if needed. Unlike with React Native or Flutter you aren't locked in a specific framework's architecture but based on mature standards and provided with tools to expand capabilities as needed.

### core dependencies

The core RESTful functionality is powered by [axum](https://github.com/tokio-rs/axum)'s [Router](https://docs.rs/axum/latest/axum/struct.Router.html) - simple and extremely flexible framework to compose routes and middleware. Prest adds a couple of utils to it to simplify common host needs: server startup with a bunch of common middleware, embedding files by lazily reading from disk in debug mode and including into the binary for the releases based on [rust-embed](https://github.com/pyrossh/rust-embed), global state variables with [Lazy](https://docs.rs/once_cell/latest/once_cell/sync/struct.Lazy.html) initialization, integration for auth and a couple of others. Also, like most of the rust web ecosystem, prest relies on [http](https://docs.rs/http/latest/http/) and [tower](https://docs.rs/tower/latest/tower/) crates for common types and compatability.

While axum has built-in helpers for the state management, they can introduce type-related issues if you're merging and nesting routers. So, I recommend using good old rust statics for state variables like DB connections and others, which also have a nice property of having the initializaiton logic right in the declaration. Also, prest includes a simple macro that allows using `?` instead of unwraps and also runs async if necessary.

In some cases you might want to have lower-level control - for example to configure a proxy or to customize the runtime settings. In these cases you can easily import underlying crates directly and use only those prest utils which fit your needs. Under the hood its powered by [tokio](https://docs.rs/tokio/latest/tokio/) - general-purpose async runtime which provides exceptional performance for a wide range of applications, [hyper](https://hyper.rs/) for extremely reliable and efficient HTTP processing and [tower-http](https://github.com/tower-rs/tower-http) for generic middleware. 

### templating utilities

Prest includes the `html!` macro, which is forked from [maud](https://github.com/lambda-fairy/maud) to improve compatability with other prest's components. It allows to easily mix usual rust code with html templates and render them exceptionally fast.  By default prest's `Head` utility also imports [Tailwind](https://tailwindcss.com/) and [DaisyUI](https://daisyui.com/), and `Script` imports [HTMX](https://htmx.org/). Such combination provides everything necessary to develop UIs with styles and interactions right inside of the html and keep relevant bits of code close to each other. Also, if you want to bundle TypeScript, JavaScript, SASS or SCSS with your app prest provides a couple of build utilities that can run together with the rest of the app's build pipeline.

Another big thing common in FE development is state management. You can generally split every solution in 2 pieces: state storage and change signals. HATEOAS principle - *Hypermedia As The Engine Of Application State* suggests to use html itself as the current state of the app. You already have it and it's easy to observe, debug etc. Also, html has a built-in mechanism to signal that something happened - [DOM events](https://en.wikipedia.org/wiki/DOM_event), and both htmx and hyperscript have exceptional tools to work with built-in ones and to create your own.

### embedded database 

Prest also packs with **[sled](http://sled.rs/)** - embedded database (somewhat like [RocksDB](https://rocksdb.org/)) written in rust from scratch. This crate and it's close relative 
[Komora project](https://github.com/komora-io) are building next gen storage-layer solutions that utilize latest research in the field and focused on modern hardware. Reinventing a database might sound like a bit crazy idea, but: 

* such systems require fine-grained memory control and safety more than any other and rust shines in this space
* rust itself introduces almost no overhead so these tools can compete with mature C counterparts
* sled has already been in development for years, has reached **v1 alpha** and can beat a lot of mature competitors on common workloads
* future improvements would be much easier to implement than in C codebases because borrow checker will always validate that another refactor or subsystem rewrite doesn't introduce memory bugs

According to it's discord server and discussions around the web there are already at least a couple of dozens of projects using sled. And I expect this number to grow dramatically once it will reach it's first stable release. But sled itself is only focused on being an efficient low-level storage layer that is expected to be used by higher-level libraries like 

**[GlueSQL](https://gluesql.org/docs/)** - SQL query parser and execution layer that can be attached to wide variety of storage options. It's even younger than sled, but can already be used from rust, python and javascript(both node and browser!). Also, it already supports sled, redis, in-memory, json, csv, and browser local, session and indexedDB storages. You can also define your own storage interfaces and even create composite ones that allow different storages for different tables while still supporting JOIN queries across them.

The main benefit of gluesql is that it allows to work with different storages on both client and server side using the same interface. As of now this interface has some issues and does not have anything like an ORM, but it ships with a query builder and you can use good old SQL. On top of that prest adds a `Table` derive macro that provides easy-to-use common CRUD operations.

This combo enables a zero-setup database in prest for your apps which can rely on in-memory storage, efficient disk persistance and even store data in the browser - all with the same sql-based interface.

### authentication

User + session management is powered by [axum-login](https://github.com/maxcountryman/axum-login) which is quite fast and flexible. OAuth and OpenID flows are powered by [openidconnect-rs](https://github.com/ramosbugs/openidconnect-rs) which provides extensible strongly-typed interfaces. Even their basic examples are still pretty verbose, but prest integrates them together with the embedded database and other components so that you can just forward user requests to the relevant endpoints and the rest will be taken care of. Secure passwords handling is possible thanks to the [password-auth](https://docs.rs/password-auth/latest/password_auth/) crate.

### others

While I've mentioned a whole bunch of libraries already, the list of dependencies that powers prest and makes it all possible is much longer so a huge thanks to everyone involved in their development:

{Cargo.toml:47-109}

Besides them there are `prest-build` dependencies: [SWC](https://swc.rs/) that powers typescript/js bundling, [grass](https://github.com/connorskees/grass) that powers SASS/SCSS processing, [webmanifest](https://github.com/mild-times/webmanifest) that generates PWA manifests and [toml](https://github.com/toml-rs/toml) which allows simple deserialization of configs.