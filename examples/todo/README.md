Simple CRUD app with a database, partial rendering and decent styles. Every rust crate starts with the `toml` manifest:

{Cargo.toml}

It contains core metadata of the package that cargo needs to build it properly. With this tiny config we can move on to writing rust. As of now prest is designed to be imported in bulk:

{src/main.rs:1}

This might not be the idiomatic rust way, but it's more beginner friendly in my view. Then we define the `Todo` struct and deriving some traits that we'll use later:

{src/main.rs:3-10}

Here we have serialization traits and defaults to be able to deserialize a todo value from just the `task` field. Also, there is a `Table` derive macro that implements a trait with the same name and a couple of helper methods which will allow us to easily work with a gluesql `Todos`(struct name + `s`) table. 

Then there is a manual implementation of the `Render` trait which will allow us to use todo values directly in the markup:

{src/main.rs:12-22}

Since this is a quite unusal templating solution and used together with [HTMX](https://htmx.org/) and built-in [Tailwind](https://tailwindcss.com/) classes let's go through it a bit:

1. if the tag is not specified, like the parent tag of a todo, then it's rendered as a `div`
2. `$"..."` works somewhat like class html attribute in tailwind but `html!` macro converts them into embedded css
3. `hx-target="this"` sets htmx to swap this element on requests from it or it's children
4. `hx-swap="outerHTML"` sets htmx to swap the whole element (by default it swaps it's children)
5. `hx-vals=(json!(self))` adds json-serialized todo fields which htmx will form-encode and send with requests
6. `hx-patch="/"` sets this element to send `PATCH` method requests to `/` when triggered
7. `checked[self.done]` adds `checked` attribute if `self.done` is true
8. `hx-delete="/"` sets this element to send `DELETE` method requests to `/` when triggered

Then goes the function that renders the whole page based on provided `content`:

{src/main.rs:24-35}

It includes a couple of utility structs `Head` and `Scripts` which render into the `head` and a bunch of `script` tags respectfully. They are not essential but provide a shorthand for meaningful defaults. Other things there are common html, htmx attributes that I've mentioned above and a `hx-on--after-request="this.reset()"` attribute that just invokes a little js snippet after an htmx request that clears out the form.

Alright, now we have all the components for our service so let's move on to the `main` function where it all will come together:

{src/main.rs:37-52}

The first lines invokes the `init` macro with the only table we defined so it will setup basic prest configs and make sure there is a table in the database prepared for our todos.

Then goes the definition of the router from just a single route `/` but with closure-based handlers for 4 methods: `get`, `put`, `patch` and `delete`.

* on `GET` it retrieves all the todos from the database using the derived method and renders the whole list
* on `PUT` it deserializes the todo value from the form-encoded data, saves it in the DB and renders
* on `PATCH` it deserializes the todo value, toggles the `done` attribute in it and in the DB, and renders
* on `DELETE` it deserializes the todo value, removes it from the DB and returns an empty body

Then goes the `wrap_non_htmx` middleware that wraps response bodies with the provided function for requests that weren't initiated by htmx.  As the result if you'll go the `/` in browser it will initiate a usual `GET` request that will render the todos and wrap them into the page markup. However, when either the form that submit's new todos, todo checkbox or the delete button will trigger an htmx request the responses won't be wrapped and would be swapped straight into the opened page. 

Finally comes the `.run()` utility method provided by prest that: attempts to read variables from `.env` in this or parent folders, starts tracing, adds middleware to catch panics, livereload in debug or compression and request body limits in release configs, checks the `PORT` env variable or defaults to `80`, checks the `DB_PATH` variable and initializes storage files there or defaults to in-memory storage, and finally starts the multi-threaded concurrent web server which will process the requests. 

Hurray, now the app is running and you can play around with it. It's already a dynamic full-stack app which can handle a bunch of use cases, but one of the core premises of prest is to provide installable native-like UX so that's what we'll add in [the next example](https://prest.blog/todo-pwa).

The full code of the current example for copy-pasting and reading convenience:

{src/main.rs}
