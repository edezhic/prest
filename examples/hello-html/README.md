Prest respects [HATEOAS](https://htmx.org/essays/hateoas/) constraint of the REST and focused on HTML to build cross-platform UIs. It's not universally the best approach for the frontend development though - you should consider [when to use hypermedia](https://htmx.org/essays/when-to-use-hypermedia/) before taking this architectural decision, but it's good enough for a lot of cases. I think even more than the linked essay suggests.

Cargo manifest is almost the same except we add a new dependency - [serde](https://serde.rs/), go-to serialization and deserialization library in the rust ecosystem. It's not re-exported from prest because app-level code is mostly using it's macros which are painful to re-export without forking, but almost every rust app has it among dependencies so I'd suggest getting used to having it in every project: 

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
