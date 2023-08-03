# Powers for apps 💪
### For an outdated overview visit [pwrs.app](https://pwrs.app/)

**PWRs** is a **P**rogressive **W**eb **R**u**s**t framework designed to quickly start your cross-platform apps. Beware that it's an early WIP - I've verified that rust with the dependencies used here provide a nice development experience so I've decided to try to compose them in a easy-to-start way: basic example is just ~20LoC (~30 for the page template and ~30 for the cargo config).

Beside PWRs this repo contains a bunch of example apps which can be run with `cargo run -p NAME` and optional `--features="sw"` flag to include the service worker:

- `basic` is a minimalistic boilerplate for a PWRS app
- `with-tracing` ...
- `with-oauth` ...
- `with-htmx` ...

Before geting started you'll need the nightly [rust toolchain](https://rustup.rs/)

TODO:
- fix `with-tls`
- fix `with-gluesql`
- `with-language-models`
- `with-scraper`
- `with-diesel`
- `with-blockchain`
- `with-webrtc`?
- combine pwrs into a single crate with build, host and sw features
- tests
- docs
- `native` wry/tauri stuff
- -host: catch panic and compression layers, other tower-http middleware?

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
