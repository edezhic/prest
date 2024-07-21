In the [previous example](https://prest.blog/todo-pwa-auth) we've added auth to the [todo PWA](https://prest.blog/todo-pwa). In this one we'll make all the todos public and provide real-time updates to the clients about every change in them. We'll add a new dependency here - [async-broadcast](https://docs.rs/async-broadcast/latest/async_broadcast/) which provides a simple mechanism to share changes with multiple streams:

{Cargo.toml:8}

Besides this manifest remains the same as well as build script and library so their contents are at the bottom.

Until now we've changed the state of the clients only based on their requests and it made sense, but now we'll update the todo list based on [server sent events](https://en.wikipedia.org/wiki/Server-sent_events) initiated by other users adding or modifying their todos. Render method of our todo won't be based on the `Render` trait anymore because it will need additional user data to disable controls for non-owners. Also, we won't be returning markup from `add`, `toggle` and `delete` handlers anymore but instead use them to modify the data accordingly and broadcast the changes to all active clients:

{src/main.rs}

We're using the htmx's `sse` extension which allows us to easily swap events payloads into the right places based on their names. It starts with the `hx-ext="sse"` and `sse-connect="/todos/subscribe"` attributes which activate the extension and connect to the specified route to listen for events. Then `sse-swap="EVENT_NAME"` attributes can be used on it and its children to listen to events with specified names and swap if got any. In this case we're using `add` to append and todo's `id` as names to make sure events reach the right places.

Now we have an installable collaborative real-time full-stack app! No react or another frontend framework involved and without even writing js. This is the end(for now) of the tutorials series, but you can also check out other examples from the menu.

Remaining code used in this example:

{Cargo.toml}

{build.rs}

{src/lib.rs}