**prest** is based on **P**rogressive browser features and **REST**ful architecture. The goal is to make the basic setup as simple as possible while leaving enough room for any customizations. Thanks to rich Rust->WASM support it allows sharing of the server code with a client-side Service Worker to process requests. It works especially well with [htmx](https://htmx.org/) ..., [hyperscript](https://hyperscript.org/) ... . Also, it embeds a slightly modified fork of [maud](https://maud.lambda.xyz/) for deeper integration. It heavily depends on [axum](https://github.com/tokio-rs/axum) for ergonomic cross-platform routing, [http](https://docs.rs/http/latest/http/) and [tower](https://docs.rs/tower/latest/tower/) for common types, [tokio](https://docs.rs/tokio/latest/tokio/) for the async runtime, ... . [rust-embed](https://github.com/pyrossh/rust-embed) - asset bundler optimized for fast debug compilation and packs all the files inside the release binary for fast runtime performance and convenient devops. It also reexports a lot of them to speed up onboarding.

[js-sys](https://rustwasm.github.io/wasm-bindgen/contributing/js-sys/index.html) and [web-sys](https://rustwasm.github.io/wasm-bindgen/contributing/web-sys/index.html) for all kinds of interactions with JS and Web APIs from rust.

And some tools for the [build scripts](https://doc.rust-lang.org/cargo/reference/build-scripts.html): 

* [wasm-bindgen](https://rustwasm.github.io/wasm-bindgen/) for JS<->WASM bindings generation. Also supports Rust->TS bindgen for type safety all the way.
* [swc](https://swc.rs/) - fast TypeScript compiler.
* [grass](https://docs.rs/grass/latest/grass/) - fast SASS/SCSS into CSS compiler.

Examples currently serve as both documentation and tests. You can find their list and descriptions on the [homepage](https://prest.blog) which is transpiled from the `./README.md`. Their core dependencies are usually included in their names.