**prest** is a **P**rogressive **REST**ful framework designed to simplify cross-platform app development. It's based on web technologies because they're supported on almost all devices and widely known among the developers, and built in Rust because it provides a nice balance of development experience, performance, safety and platform support. Just write some HTML and get a cross-platform UI. Add a couple of lines to make it installable, enable offline work and automatic background updates. Solve any tasks like auth, database integrations, loggers, serverless deployments and many others with ease thanks to interactive examples and [rich ecosystem](https://edezhic.medium.com/reliable-software-engineering-with-rust-5bb4553b5d54). 

[Blog](https://prest.blog). It's not on [crates.io](https://crates.io/crates/prest) yet because it depends on the latest unpublished [axum](https://github.com/tokio-rs/axum) changes. Awaiting it's 0.7 release to publish the first alpha version.

## Getting started

To get started you'll need the latest stable [rust toolchain](https://rustup.rs/). You can run examples with `cargo run -p NAME`.

Most examples are supported on [replit](https://replit.com/) and you can [fork it there](https://replit.com/@eDezhic/prest) to quickly run in the cloud. It includes [rust-analyzer](https://rust-analyzer.github.io/) and I suggest to use it in local development as well. If you also want to include Typescript then [VS Code](https://code.visualstudio.com/) might be a great choice.

Some examples require [WebAssembly](https://webassembly.org/) target (`rustup target add wasm32-unknown-unknown`) or other additional setup which can be found in the relevant docs.

Temporarily deployed to replit by compiling into the `musl` target and including binary into the repo due to [this issue](https://ask.replit.com/t/deployment-time-outs/73694). To rebuild the binary run `cargo build -p blog --target x86_64-unknown-linux-musl --release` and move `target/.../serve` it into the `_temp_deployment` folder.

## vs alternatives

Unlike most other web frameworks it achieves offline work by cross-compiling server code into a WebAssembly-based Service Worker to process requests on the client side. With this approach you don't have to write JS or use custom RPC/JSON/GraphQL protocols for client-server comms. Client files are downloaded and updated in the background without blocking UI.

There are already frameworks like Tauri and Dioxus, but I think that [PWA](https://web.dev/what-are-pwas/) capabilities enable a simpler approach. Inspiration came from [this simple PoC](https://github.com/richardanaya/wasm-service) - combination of a [Service Worker](https://developer.mozilla.org/en-US/docs/Web/API/Service_Worker_API) based on Rust compiled into [WASM](https://webassembly.org/) with [HTMX](https://htmx.org/) library. I want to highlight HTMX and [Hyperscript](https://hyperscript.org/) because they allow solving almost any UI problem without JavaScript, JSON and other common front-end tech. This way we can cross-compile some server rendering code for the Service Worker and get a restful client almost for free. Also, there are plenty of [Web APIs](https://fugu-tracker.web.app/) available through [wasm bindings](https://github.com/rustwasm/wasm-bindgen), [WASI](https://github.com/bytecodealliance/wasmtime/blob/main/docs/WASI-intro.md) ecosystem to simplify devops and fascilitate serverless, [WebGPU](https://developer.chrome.com/blog/webgpu-io2023/) for cross-platform AI, complex UIs and games, and many other web-related tech being developed for all kinds of use cases.

Not everything is ideal - Rust requires significant onboarding. Such performance and reliability come at the cost of learning how to use them properly. So one of the main goals of prest is to provide a simple default setup that allows devs to start their projects in minutes.


## acknowledgements 

Under the hood it's based on [Axum](https://github.com/tokio-rs/axum) for ergonomic routing on both server and client. It's also based on [http](https://docs.rs/http/latest/http/) and [tower](https://docs.rs/tower/latest/tower/) for compatability with many other crates. It includes a slightly modified forks of [Maud](https://maud.lambda.xyz/) to easily include HTML snippets inside Rust code and [Rust Embed](https://github.com/pyrossh/rust-embed) to easily bundle PWA and other assets with the compiled binary. [Tokio](https://docs.rs/tokio/latest/tokio/) is the only supported async runtime at the moment.

Interactions with Web APIs from Rust code are powered by [wasm-bindgen](https://rustwasm.github.io/wasm-bindgen/), [js-sys](https://rustwasm.github.io/wasm-bindgen/contributing/js-sys/index.html) and [web-sys](https://rustwasm.github.io/wasm-bindgen/contributing/web-sys/index.html). Also, if you want to include custom TypeScript code in the project they can generate TS declarations of the exported Rust types, and prest also includes [SWC](https://swc.rs/) to transpile and bundle that code. It also includes [grass](https://docs.rs/grass/latest/grass/) - speedy SASS -> CSS compiler for your UI needs. Most likely it will include more common web tooling in the future.

