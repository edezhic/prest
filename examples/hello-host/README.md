The core RESTful functionality is powered by [axum](https://github.com/tokio-rs/axum)'s [Router](https://docs.rs/axum/latest/axum/struct.Router.html) - simple and extremely flexible framework to compose routes and middleware that supports static and closure handlers, wildcard paths, nesting, and many other things. Prest adds a couple of utils to it to simplify common host needs: server startup, embedding files, global state. Everything is either exposed, re-exported or used under the hood so that you only need to add a single dependency:

`/Cargo.toml`
{Cargo.toml}

While axum has built-in helpers for the state management, they can introduce type-related issues when you're merging and nesting routers as we will do later with shared router for both host and client. So, I recommend using good old rust lazy statics for state variables like DB connections and others, which also have a nice property of having the initializaiton logic right in the declaration. This example showcases the basic structure of the host:

`/serve.rs`
{serve.rs}

Once you run this host it will compose the router, check out the `PORT` env variable or default to `80` if no value found, set up common middleware for tracing, compression and limiting request bodies, connect to the socket and start processing requests. You can check out the root path (with default browser GET method) which returns extracted host and state info, as well as the header added by the middleware. Also, you can check out the `/Cargo.toml` and `/serve.rs` paths for the embedded files.

In some cases you might want to have lower-level control - for example to configure proxy or customize the runtime settings. In these cases you can easily import underlying crates directly and use only those prest utils which fit your needs. 

Starting with this code you can already embed a bunch of html, css and js assets and get your website started. However, it won't be particularly convenient for development or the users, so let's move on to the next example where we'll explore a way to work with hypermedia without leaving the comfort of rust while also improving UX.

[Next: hello-html](https://prest.blog/hello-html)