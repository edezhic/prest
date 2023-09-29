This is a hobby project so there are no solid long-term plans. However, there are things that I'll most likely spend time on in the foreseeable future:

### More examples
- `with-language-model`
- `with-diesel`
- `with-blockchain`
- tracing for the SW - https://github.com/old-storyai/tracing-wasm
- Askama/other template engines
- GlueSQL for the SW
- `with-webrtc`?
- `blog`
- `native` wry/tauri stuff
and many others. The goal here is to try to work with different things from Prest. Especially things which might run on both client and server like GlueSQL.

### More middleware
Catch panic, compression, ...

### API simplification and customization
Current API is very rigid and nowhere near stability. It's quite simple already but should be simplified and integrated with more other Rust web dev crates before even considering stabilization. Also, hardcoded things should be moved into Default impls of config structs/enums.

### Publication & Stabilisation 
As of now there is a number of unstable deps, Axum probably being the brightest example because the required version is not even published yet, so these things are on hold for unknown time.

### WASI
A really cool thing that simplifies devops x1000, but devex still sucks because of the missing ecosystem support and features. 

### Other things
host: catch panic, compression layers, other tower-http (and not only) middleware? just reexport or ...?
Tests, docs etc
