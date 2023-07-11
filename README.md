# For an (outdated) overview visit [pwrs.app](https://pwrs.app/)

To get started with the development you'll need the [rust toolchain](https://rustup.rs/), then you can start the host with `cargo run`
Available features (can be added with `--features="FEATURE1 FEATURE2 ..."`):
* `sw` - include service worker that can render UI in the browser 
* `oauth` - include google auth (requires env vars or `.env` with `GOOGLE_CLIENT_ID` and `GOOGLE_CLIENT_SECRET`!)  
* `https` - include TLS (optional env vars or `.env` with `TLS_CERT_PATH` and `TLS_KEY_PATH`; otherwise cert and key will be generated)

TODO:
* signup confirmation email - mail-send
* auth UI - https://github.com/tokio-rs/axum/blob/main/examples/form/src/main.rs
* other UIs?
* anyhow errors in handlers? - https://github.com/tokio-rs/axum/tree/main/examples/anyhow-error-response
* compression
* tracing level env config, ports env config?

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
