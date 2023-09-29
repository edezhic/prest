The initial inspiration comes from [this](https://github.com/richardanaya/wasm-service) simple PoC - a combination of a Service Worker based on Rust compiled into WebAssembly and HTMX library. While such mix looks odd at first sight it actually provides a couple of extremely nice properties:

1. Really RESTful architecture - you don't even have to reach beyond HTML to build an interactive installable app
2. Client-Server code sharing thanks to WASM
3. Progressive enchancement - your app can work in ancient browsers and gets smoother in modern ones

Based on web technologies - as cross-platform as it gets. Written in Rust for safety and performance. Also, if you have sophisticated needs beyond web capabilities Rust provides almost infinite flexibility: JS engines, WebGPU, GlueSQL, AI, blockchains etc.

Let me elaborate why these points are worth your attention:

Usually REST assumes that requests are going all the way to the server and this round-trip often involves a noticable delay, so UI engineers turn to JS and platform-native code to make smooth UX for the end users. However, Prest builds a Service Worker that can process requests on the client-side which allows solving most UI problems with simple HTML.

Relatively few frameworks allow you to share client and server code, and even those often do that in framework-specific and somewhat obscure ways. On the other hand, Prest shares routers, templates and handlers which makes it trivial to reason about shared code in simple HTTP terms.

Prest combines the simplicity of casual server-oriented frameworks like Django with fancy client-side interactivity of libraries like React. Unlike most other full-stack frameworks like NextJS, Prest builds on an extremely reliable and performant language - Rust, follows a simple RESTful architecture and produces native-like apps thanks to PWA capabilities.

Rust combines well with JS thanks to wasm-bindgen and you can even preserve type safety thanks with TypeScript bindings.

WASI ecosystem for the bright future.


