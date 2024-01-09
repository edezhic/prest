In the [previous example](https://prest.blog/todo-pwa) we've made our [todo app](https://prest.blog/todo) installable and now we'll provide authentication mechanisms. As always we're starting with the manifest and now we'll need to activate the `auth` feature of prest:

{Cargo.toml:6}

Everything else remains just like in the previous one, and you can find it's full content at the end of this tutorial. Same with the build script and the library as they remain untouched. Since authentication is the server-side business we'll only need to modify our binary:

{src/main.rs}

Now our handlers will need to consider whether requests are authorized to read/write specific todos and their code becomes significantly larger, so we'll be moving away from closures and use functions. Also, here we introduced two new extractors: 

* `Auth` - struct that provides auth-related methods to authenticate and an optional user field
* `User` - little utility that returns user data if there is some `auth.user` and returns `UNAUTHORIZED` if none

Also, our `todos` handler got new templates with sign in / sign up forms, and an optional login with google button that renders depending on whether `GOOGLE_CLIENT_ID` and `GOOGLE_CLIENT_SECRET` env variables required for the auth are provided. By default after the auth flow they will redirect back to the `/`, but this behaviour can be customized by sending `redirect` field with the forms or by adding `next` query param to the google auth route.

Handlers for the routes specified in these forms and the button are automatically appended to the router in the `.run()` function. It will also set up the session and user management middleware, storage for them and other required utils.

That's it! Now users can install the app and handle their own todos without spying on each other. But maybe you actually want todos to be public? Let's make it happen in the [next example](https://prest.blog/todo-pwa-auth-sync).

Remaining code used in this example:

{Cargo.toml}

{build.rs}

{src/lib.rs}