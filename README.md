# For an (outdated) overview visit [pwrs.app](https://pwrs.app/)

To get started with the development you'll need the [rust toolchain](https://rustup.rs/), then you can start the host with `cargo run`
Available features (to be added with `--features="FEATURE1 FEATURE2 ..."`):
* `sw` - include service worker
* `oauth` - include google auth (requires env vars or `.env` with `GOOGLE_CLIENT_ID` and `GOOGLE_CLIENT_SECRET`!)  

TODO:
* auth UI
* built-in https (+ cert generation for debug?) 

Notes about design choices:
* WRY - awesome but I decided to focus on PWA thing, seems to have better platform support. 
* WASI - awesome but early. Need wider library support and more system APIs(at least full TLS) to get real.
* Maud - questionable but I love the rusty minimalistic syntax.
* Grass(SCSS) - simple to start and scalable for complex projects, does not enforce anything. 
* TypeScript - type and memory safety all the way down, writing browser code in Rust is painful DX
* Axum - elegance and possibility to use without runtime for the SW.
* Tokio+Hyper - currently the most popular http server stack, but thanks to axum's flexibility can be replaced in the future
* GlueSQL - uniform and familiar interface over any storage even on the client.
