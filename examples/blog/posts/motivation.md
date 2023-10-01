[Rust and it's ecosystem](https://medium.com/globant/reliable-software-engineering-with-rust-5bb4553b5d54) are amazing - almost infinite flexibility, reliability and performance packed in a nice syntax. It's definitely great for systems programming, but I want to build apps on top of these systems with ease. Rust nicely fits backend needs and thanks to the rich [WebAssembly](https://webassembly.org/) support it can also handle frontend even faster than JS.

There are frameworks like Tauri and Dioxus for cross-platform apps, but I want to have a full Rust stack without complex React-like UI. I strongly believe that the default setup shouldn't require knowledge beyond HTML to make something happen on the screen, and the whole stack shouldn't require more than 1 language. A lot of inspiration came from [this simple PoC](https://github.com/richardanaya/wasm-service) - combination of a [Service Worker](https://developer.mozilla.org/en-US/docs/Web/API/Service_Worker_API) based on Rust compiled into WASM with [HTMX](https://htmx.org/) library. This way I can cross-compile rendering code for BE and FE, and with [PWA](https://web.dev/what-are-pwas/) capabilities I can install it like a native app. So, I decided to compose a Rust framework based on common web standards to simplify cross-platform development:

1. Easy start - a couple of lines of code to start serving content
2. Fully RESTful architecture even on the client - no Virtual DOM and other tricks
4. Progressive enchancement - app can work in ancient browsers while also be installable and faster in modern ones
5. Limitless capabilities thanks to Rust - integrate AI, blockchains, SQL DBs on both client and server, OS components and even run on bare metal

Long story short - I want a simple framework based on familiar web technologies, while also being able to expand app's features in any direction without having to pull additional languages. Technically you can also achieve that in C/C++ but with worse ergonomics and reliability, and in many other languages with different caveats. But only Rust provides convenient devex, top-tier performance, outstanding safety and limitless integrations in a single package.

I want to emphasize focus on HTML and REST in Prest because they are both supported on almost all devices and widely known among the developers. If you haven't worked with web before - I strongly recommend checking out [hypermedia.systems](https://hypermedia.systems/). Such basis makes onboarding extremely simple even for programming newbies and familiar to devs with any web experience. Thanks to the Service Worker you don't need separate client and server rendering pipelines - pretty much everything is possible with usual HTTP requests and HTML responses.

While this approach can handle needs of most apps, I realise that modern UI development consists mostly of JS/TS, SCSS and other tech. And Rust shines here as well! Prest comes with [wasm-bindgen](https://rustwasm.github.io/docs/wasm-bindgen/) for type-safe Rust<->TS integration, [SWC](https://swc.rs/) - TypeScript transpiler and bundler, [grass](https://github.com/connorskees/grass) - SASS preprocessor and more tools will be integrated for developers convenicence.

As you can see, Rust ecosystem has all the tooling you'll need to make apps of any level of complexity. And I've decided to build Prest as a relatively thin wrapper around these tools to lower the entry threshold and simplify the first steps.

And the ambition doesn't end here. There are plenty of new technologies being developed primarily in Rust for the next generation of applications:

* [WASI](https://github.com/bytecodealliance/wasmtime/blob/main/docs/WASI-intro.md) for extremely simple DevOps
* [WebGPU](https://developer.chrome.com/blog/webgpu-io2023/) for cross-platform AI and complex UIs
* All kinds of other [Web APIs](https://fugu-tracker.web.app/) to match capabilities of native apps
* Tons of projects rewriting and improving legacy C/C++ codebases

Altogether Rust provides great compatibility with legacy tech, pleasant development experience today and a bright future. This is how I became a convinced Rustacean. 