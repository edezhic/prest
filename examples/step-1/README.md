The core RESTful functionality is powered by [axum](https://github.com/tokio-rs/axum)'s [Router](https://docs.rs/axum/latest/axum/struct.Router.html) - simple and extremely flexible framework to compose routes and middleware. Prest adds a couple of utils to it to simplify common host needs: server startup with a bunch of common middleware, embedding files by lazily reading from disk in debug mode and including into the binary for the releases based on [rust-embed](https://github.com/pyrossh/rust-embed), global state variables based on [Lazy](https://docs.rs/once_cell/latest/once_cell/sync/struct.Lazy.html) initialization, [anyhow](https://github.com/dtolnay/anyhow)'s [Result](https://docs.rs/anyhow/latest/anyhow/type.Result.html) type to simplify error handling, and a couple of others. Most things are either exposed, re-exported or used under the hood so that you can start building with just a single dependency:

{Cargo.toml}

While axum has built-in helpers for the state management, they can introduce type-related issues if you're merging and nesting routers as we will do later with shared router for both host and client. So, I recommend using good old rust statics for state variables like DB connections and others, which also have a nice property of having the initializaiton logic right in the declaration. Also, prest includes a simple macro that allows using `?` instead of unwraps and also runs async if necessary. This example showcases the basic structure of the prest host:

{src/main.rs}

Once started it will compose the router, attempt to get the `PORT` env variable or default to `80`, set up common middleware for tracing, compression and limiting request bodies, connect to the socket and start processing requests. You can check out the root path (`/`) which returns the extracted host and state info, as well as the header added by the middleware. Also, you can check out the `/Cargo.toml` and `/serve.rs` paths of the running app to see the contents of the embedded files.

In some cases you might want to have lower-level control - for example to configure a proxy or to customize the runtime settings. In these cases you can easily import underlying crates directly and use only those prest utils which fit your needs. Under the hood its powered by [tokio](https://docs.rs/tokio/latest/tokio/) - general-purpose async runtime which provides exceptional performance for a wide range of applications, [hyper](https://hyper.rs/) for extremely reliable and efficient HTTP processing and [tower-http](https://github.com/tower-rs/tower-http) for generic middleware. Prest, as well as most of the rust web ecosystem, also relies on [http](https://docs.rs/http/latest/http/) and [tower](https://docs.rs/tower/latest/tower/) crates for common types and compatability.

Starting with this code you can already include a bunch of html, css and js assets and get your website started. However, it won't be particularly convenient for development or the users, so let's move on to the next example where we'll explore a way to work with hypermedia without leaving the comfort of rust while also improving UX.

--------------------------------------------------------------------------------------------------------------------------------------------

Prest respects [HATEOAS](https://htmx.org/essays/hateoas/) constraint of the REST and focused on HTML to build cross-platform UIs. It's not universally the best approach for the frontend development though - you should consider [when to use hypermedia](https://htmx.org/essays/when-to-use-hypermedia/) before taking this architectural decision, but it's good enough for a lot of cases. I think even more than the linked essay suggests.

Cargo manifest is almost the same except we add a new dependency - [serde](https://serde.rs/), go-to serialization and deserialization library in the rust ecosystem. It's not re-exported from prest because app-level code is mostly using it's macros which are painful to re-export without forking and breaking compatability with other crates, but almost every rust app has it among dependencies so I'd suggest getting used to having it in every project: 

{Cargo.toml}

In this example we'll use the `html!` macro for templating forked from [maud](https://github.com/lambda-fairy/maud) to improve compatability, as well as [htmx](https://htmx.org/) for reactivity and [hyperscript](https://hyperscript.org/) to sprinkle a bit more interactivity. It's a pretty simple service that showcases how you can set up client<->server communication with shared layout, partial rendering, decent styles and some animations:

{src/main.rs}

Router has only one route but two handlers for GET and POST request methods: on get it simply returns static markup, while on post it deserializes the form data according to the `FormInputs` structure and returns a message that depends on the received value. Also, router includes `wrap_non_htmx` middleware which takes a function that it will use to modify response content based on special http header that htmx adds to it's requests.

Both handlers and middleware only return `Markup` which is produced by the html macro. This syntax might seem somewhat odd, but I like how minimalistic it is and how well it blends with the rest of rust (don't worry - there are plenty of templating libraries to work with raw html as well, but you might find this macro pretty convenient, especially for small components). Beside usual tags and attributes you can also find:

1. `Head` and `Scripts` structs that render into `<head>...</head>` and a bunch of `<script...>` tags respectfully to reduce boilerplate
2. `hx-target` with css selector value that tells htmx where to put the content of some response
3. `hx-swap` that specifies which part of the target show be swapped and enables the [View Transition API](https://developer.mozilla.org/en-US/docs/Web/API/View_Transitions_API) (also, `view-transition` class is added to the `main` tag to apply prest-included default transition animation)
4. `hx-boost` which replaces usual hyperlinks and forms navigation with scripted requests that do not reload the page on request and only replace the target with the response content
5. `hx-post` that specifies POST method and route for the request
6. `_="..."` which is a snippet of hyperscript code that toggles `party` class on the button (and some styles below that define the party animation)

Almost all `hx-...` attributes are automatically inherited by children (with an opt-out option) so you don't have to target and boost every link and form separately etc. On initial homepage request user gets the result of the home function wrapped into the page. There he can play with the party button or submit the form and receive only the result of the `submit` function swapped into main instead of home content with a nice animation, and from there he can get back the home content using a link. 

Common navigation header, head tag and other surrounding things remain in place and there is no blank page blink like with raw html navigation. But, at the same time, it's almost as easy as basic html linking. You can build something similar using pretty much any frontend framework, but I can't think of any single one that makes it so easy. Htmx and hyperscript can solve about 99% of UI tasks in an easier and more maintainable way than JS, while keeping related scripts and markup right next to each other. By the way, both libraries have pretty comprehensive docs with lots of examples!

Another big thing common in FE development is state management. You can generally split every solution in 2 pieces: state storage and change signals. HATEOAS principle - *Hypermedia As The Engine Of Application State* suggests to use html itself as the current state of the app. You already have it and it's easy to observe, debug etc. Also, html has a built-in mechanism to signal that something happened - [DOM events](https://en.wikipedia.org/wiki/DOM_event), and both htmx and hyperscript have exceptional tools to work with built-in ones and to create your own.

For styles in this and other examples I rely on [picocss](https://picocss.com/) (which is included by the `Head::example()`) because it's extremely simple and minimalistic. But you are free to use any css frameworks/libraries like [Tailwind](https://tailwindcss.com/) or it's no-build [Twind](https://twind.dev/) version, [Material UI](https://www.muicss.com/), [SCSS](https://prest.blog/with-grass-scss) or anything else.

Now we have a powerful server and a pretty smooth UI, while most of our code is html templates. With a database and a few other details that will be good enough for a lot of projects. But let's make a step further and make our interface installable so it feels like a native app.

[Next: hello-pwa](https://prest.blog/hello-pwa)

---------------------------------------------------------------------------------------------------------

Minimalistic todo app powered by [GlueSQL](https://gluesql.org/docs/)-wrapped [sled](http://sled.rs/) storage showcasing their usage. One of the most controversial examples because I wouldn't recommend using them for almost any production project yet, but I think that both projects have a lot of potential. Let me elaborate:

**sled** is an embedded database (somewhat like [RocksDB](https://rocksdb.org/)) written in rust from scratch. This crate and it's close relative 
[Komora project](https://github.com/komora-io) are building next gen storage-layer solutions that utilize latest research in the field and focused on modern hardware. Reinventing a database might sound like a bit crazy idea, but: 

* such systems require fine-grained memory control and safety more than any other and rust shines in this space
* rust itself introduces almost no overhead so these tools can compete with mature C counterparts
* sled has already been in development for years, has reached **v1 beta** and can beat a lot of mature competitors on common workloads
* future improvements would be much easier to implement than in C codebases because borrow checker will always validate that another refactor or subsystem rewrite doesn't introduce memory bugs

According to it's discord server and discussions around the web there are already at least a couple of dozens of projects using sled. And I expect this number to grow dramatically once it will reach it's first stable release. But sled itself is only focused on being an efficient low-level storage layer that is expected to be used by higher-level libraries like 

**GlueSQL** - SQL query parser and execution layer that can be attached to wide variety of storage options. It's even younger than sled, but can already be used from rust, python and javascript(both node and browser!). Also, it already supports sled, redis, in-memory, json, csv, and browser local, session and indexedDB storages. You can also define your own storage interfaces and even create composite ones that allow different storages for different tables while still supporting JOIN queries across them.

The main benefit of gluesql is that it allows to work with different storages on both client and server side using the same interface. As of now this interface has some issues and does not have anything like an ORM, but it ships with a query builder and you can use good old SQL. 

Enough with the introduction, let's get to the code. Here's our manifest:

{Cargo.toml}

And here goes the todo app:

{src/main.rs}
