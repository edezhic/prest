`prest` is focused on **P**rogressive browser features and **REST**ful architecture to deliver native-like experiences based on client-side rendering of hypermedia resources. It is achieved with the Rust->WASM powered Service Worker that shares UI routes and middleware with the server to enable offline work.

Examples currently serve as both documentation and tests. You can find their list and descriptions on the [homepage](https://prest.blog) which is generated from the root readme.

Under the hood `prest` re-exports a whole bunch of libraries for common web development solutions and provides thin wrappers around them to simplify onboarding. Also, it embeds a slightly modified fork of [maud](https://maud.lambda.xyz/) for deeper integration. The goal is to make the basic setup as simple as possible while keeping full compatability with the underlying crates to allow any customizations. Most notable are:

* [axum](https://github.com/tokio-rs/axum) - ergonomic and flexible router with a bunch of utils for middleware management. Itself based on [http](https://docs.rs/http/latest/http/) and [tower](https://docs.rs/tower/latest/tower/) types which are widely used in diverse web libraries and frameworks.
* [tokio](https://docs.rs/tokio/latest/tokio/) - async runtime for the server, most commonly used in the rust ecosystem. Also already [partially supports WASI](https://docs.rs/tokio_wasi/latest/tokio/#wasm-support) and more work is ongoing.
* [rust-embed](https://github.com/pyrossh/rust-embed) - asset bundler optimized for fast debug compilation and packs all the files inside the release binary for fast runtime performance and convenient devops.
* [js-sys](https://rustwasm.github.io/wasm-bindgen/contributing/js-sys/index.html) and [web-sys](https://rustwasm.github.io/wasm-bindgen/contributing/web-sys/index.html) for all kinds of interactions with JS and Web APIs from rust.

And some that are available for the [build scripts](https://doc.rust-lang.org/cargo/reference/build-scripts.html): 

* [wasm-bindgen](https://rustwasm.github.io/wasm-bindgen/) for JS<->WASM bindings generation. Also supports Rust->TS bindgen for type safety all the way.
* [swc](https://swc.rs/) - fast TypeScript compiler.
* [grass](https://docs.rs/grass/latest/grass/) - fast SASS/SCSS into CSS compiler.

As well as other dependencies for more narrow and specific purposes.