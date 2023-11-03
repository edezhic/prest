**prest** is a **P**rogressive **REST**ful framework designed to simplify cross-platform app development.

Beware that [it's still very early](https://prest.blog/roadmap), but I think that the [overall idea](https://prest.blog/intro) has potential. To get started you'll need the latest stable [rust toolchain](https://rustup.rs/). You can run examples with `cargo run -p NAME`.

Most examples are supported on [replit](https://replit.com/) and you can [fork it there](https://replit.com/@eDezhic/prest) to quickly run in the cloud. It includes [rust-analyzer](https://rust-analyzer.github.io/) and I suggest to use it in local development as well. If you also want to include Typescript then [VS Code](https://code.visualstudio.com/) might be a great choice.

Some examples require [WebAssembly](https://webassembly.org/) target (`rustup target add wasm32-unknown-unknown`) or other additional setup which can be found in the relevant docs.

Temporarily deployed to replit by compiling into the `musl` target and including binary into the repo due to [this issue](https://ask.replit.com/t/deployment-time-outs/73694). To rebuild the binary run `cargo build -p blog --target x86_64-unknown-linux-musl --release` and move `target/.../serve` it into the `_temp_deployment` folder.