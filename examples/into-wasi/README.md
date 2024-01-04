> Requires [WebAssembly with WASI](https://webassembly.org/) rust target!
>
> Can be added with `rustup target add wasm32-wasi`

Builds into a [WASI](https://github.com/bytecodealliance/wasmtime/blob/main/docs/WASI-intro.md)-powered host using `cargo build -p into-wasi --target wasm32-wasi`. The build will also succeed for non-wasm32 targets to avoid cargo check and rust-analyzer errors, but will immediately panic if run. With a valid target it will generate a wasi binary compatible with multiple wasm runtimes. Prest's serve function does not support WASI at the moment, but there are forks of tokio and hyper with wasi support which we can use to easily build a simple server:

{Cargo.toml}

{src/main.rs}

Overall WASI is an interesting yet controversial idea. It's utility right now is... questionable at best. If your stack supports compilation into wasi, most likely it supports compilation into musl and can be packed into a scratch docker container. In this case pretty much all of the theoretical wasi benefits vanish in practice and it makes very little sense: musl binary will be faster, docker container will be more secure, and compatability with the surrounding infrastructure would be much better.

However, wasi is good in theory - secure, performant, easy to deploy and distribute. There are plenty of developments in it's ecosystem starting from standards and all the way up to diverse efficient runtimes and support in the existing infrastructure tooling. Nowhere near casual musl containers yet but progress is quite remarkable.

You can find a lot of different opinions about it. For example, [AssemblyScript project maintainers object](https://www.assemblyscript.org/standards-objections.html) to the inclusion of the WASI (and the underlying Component Model) standards into the WebAssembly spec with pretty solid argumentation. On the other hand, Solomon Hykes, author of Docker, wrote a couple of years ago: 

> If WASM+WASI existed in 2008, we wouldn't have needed to created Docker. That's how important it is. Webassembly on the server is the future of computing. A standardized system interface was the missing link. Let's hope WASI is up to the task!

In my view, it's a pretty good idea but requires a lot of investments from hundreds of projects to build support for it to make it real. How it will play out in the future? I don't know. As of now I can barely come up with a usage scenario where WASI will be a better target than musl. While musl is only for linux and WASI is cross-platform, [cosmopolitan](https://github.com/jart/cosmopolitan) can solve this problem for C codebases (and rust as well, [but not yet](https://twitter.com/JustineTunney/status/1719991825711120803?t=pUn2m7srGPzrj4NTW6J-0w&s=19)) in a much more efficient way. Also, most tech stacks that support musl can also run on windows and macos already. In some sense WASI is trying to reinvent too many wheels, and building compatability with the new ones instead of improving the old ones doesn't seem particularly exciting.

For engineers involved into backend development I'd recommend watching out for wasi. It's not good yet, but it's growing and might become a bigger thing than docker, musl and several other foundational projects for modern servers. But it also might die quitely so I don't recommend investing too much time in it.