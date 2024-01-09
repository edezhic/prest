In the [first example](https://prest.blog/todo) we created a simple todo app and in this one we'll enchance it with [Progressive Web App](https://web.dev/what-are-pwas/) capabilities to make it installable and provide some offline UX. Beware that compilation will require [WebAssembly](https://webassembly.org/) rust target. It can be added with `rustup target add wasm32-unknown-unknown`.

First, there will be a few additions to the app's dependencies:

{Cargo.toml}

`wasm-bindgen` will generate bindings necessary to interact with the JS runtime from our rust code while `prest-build` will help us compile our app's code into a service worker and bundle other required PWA assets like webmanifest. 

We'll move some of the code from the `main.rs` into a separate shared `lib.rs` file that will be used by the server and also compiled into a wasm library. It's possible to keep everything in a single file and use conditional compilation to select the right functions depending on the compilation target, but compiler will warn us that it's not a good idea because they are semantically different: binary target, which defaults to `main.rs`, is supposed to execute everything on it's own once started, while library target, which defaults to `lib.rs`, is supposed to provide utility to other running executables. In our case library will be used as a service worker by the browser to provide a local offline server of our app, while binary will be the server-side host of the app.

Once the build is started and all the dependencies are resolved cargo automatically detects a `build.rs`, compiles and runs it before the compilation of the library and binaries.

{build.rs}

This script just invokes a single function `build_pwa` from prest build utils. By default this fn will check the `PWA` env variable and whether it is compiled with `debug_assertions` - if the assertions are present and `PWA` value is not `debug` then it will skip the PWA bundling process entirely to speed up the overall build process and development. However, if you'll build with the `--release` profile or provide the `PWA=debug` then it will run.

This function builds our app as a library (`src/lib.rs`) into WASM target to be used in the service worker. It will use the same router produced by the `shared_routes` function as the host (`src/main.rs`) to handle [fetch events](https://developer.mozilla.org/en-US/docs/Web/API/FetchEvent) on the client side. Also, `build_pwa` runs wasm-bindgen on the resulting webassembly, injects SW event listeners into the js bindings, generates `.webmanifest` with PWA metadata and includes the default logo (if no other was provided). All the assets are moved to the special folder deep inside `target` dir that cargo creates for the build artifacts.

{src/lib.rs}

At this point `build.rs` is done and compilation proceeds to the `src/main.rs` binary which will import the same shared service, embed PWA build outputs and start the server just like the usual host:

{src/main.rs}

That's it! `Head`, `Scripts`, `build_pwa` and other utils are already adding everything necessary with default configs to get started. There are many ways how you can split app's handlers between shared and host-only, but the general rule of thumb should be - static content into the shared, dynamic into the host. While it's possible to use DB on the client just like on the host, their synchronization is a complex feature that should be avoided if possible.

To verify that it's working in chrome you can open the `/` page, then go to the `application` tab in the dev tools, check that the service worker is installed and toggle the `offline` mode to see what it will look like for a user that doesn't an internet connection at the moment. By the way, you can do the same with this blog and continue browsing the site since all the content is static and is compiled into the service worker. You can check out it's source code on the [about](https://prest.blog/about) page.

Now we have an installable app, but as of now it's just the same thing for every user. Quite likely that you'll want to distinguish them so let's [add authentication](https://prest.blog/todo-pwa-auth) to the mix.