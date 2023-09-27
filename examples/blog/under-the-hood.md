First of all it's a framework - it enforces some limitations on the app's codebase, but also provides some structure and a bunch of features with as little boilerplate as possible.

Second, it's based on Web standards because they are as cross-platform as it gets. Also, whenever possible features are added progressively.

Third, it's written in Rust because it provides reliability, performance and a great WASM ecosystem.

There are plenty of dependencies...

Notes about architectural choices:
* WRY - awesome but I decided to focus on PWA thing, seems to have better platform support and easier to use. 
* WASI - awesome but early. Need wider library support and more system APIs(at least full TLS) to get real.
* Maud - questionable but I love the rusty minimalistic syntax.
* Grass(SCSS) - simple to start and scalable for complex projects, does not enforce anything. 
* TypeScript - type and memory safety all the way down, writing browser code in Rust is painful DX
* Axum - elegance and possibility to use without runtime for the SW.
* Tokio - currently the most popular async runtime
* GlueSQL - uniform and familiar interface over any storage even on the client.
* Rustls - rust all the way down + potentially improved security due to cleaner code.