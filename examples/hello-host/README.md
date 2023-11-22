The core server functionality is powered by:
* `tokio` - general-purpose asynchronous runtime
* `hyper` - efficient and safe HTTP processor
* `axum` - flexible middleware and routing

Together they compose a reliable, concurrent and multi-threaded host that can easily handle thousands of requests per second. In prest tokio and hyper usage is hidden inside the `serve` function that is added by prest to the axum's [`Router`](https://docs.rs/axum/latest/axum/struct.Router.html) so that you can focus on your app's logic instead of dealing with the server setup. However, if you want to get low-level control to use a different runtime or apply specific optimizations to a high-load service you can do that without breaking compatability with the rest of prest.

Router is the core RESTful abstraction here and most prest's utils like `serve` are extending it. ...

* paths
* handlers 
* middleware
* nesting & merging
* ?

### State 
Many ways to handle it. My preferrence - global statics

* axum state - particularly type-safe way but it also adds complexity when working with multiple router
* axum extractors - less type-safe and less problems

