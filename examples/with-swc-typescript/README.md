This example bundles and transpiles [TypeScript](https://www.typescriptlang.org/) into a prest app using [SWC](https://swc.rs/). Beware that as of now SWC requires [nightly](https://rust-lang.github.io/rustup/concepts/channels.html#working-with-nightly-rust) rust toolchain, but there is an ongoing effort to remove this limitation.

Right now the SWC usage code is extremely verbose so please don't freak out. Most likely some utils will be exported from the `prest-build` crate.

{script.ts}

{imported.ts}

{Cargo.toml}

{serve.rs}

{build.rs}