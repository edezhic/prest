Prest respects [HATEOAS](https://htmx.org/essays/hateoas/) constraint of the REST and focused on HTML to build cross-platform UIs. It's not universally the best approach for the frontend development though - you should consider [when to use hypermedia](https://htmx.org/essays/when-to-use-hypermedia/) before taking this architectural decision, but it's good enough for a lot of cases. I think even more than the linked essay suggests.

In this example we'll use forked from [maud](https://github.com/lambda-fairy/maud) `html!` macro for templating, as well as [htmx](https://htmx.org/) for reactivity and [hyperscript](https://hyperscript.org/) to sprinkle a bit more interactivity. Cargo manifest remains the same as in the host example (except for the package name).

{serve.rs}

Twind can also be a good addition to this stack because it allows adding styles easily and ergonomically right into the markup.

Now we have a powerful server and a pretty smooth UI. That's good enough for a lot of projects but let's make a step further and make it installable so it feels almost like a native app.

[Next: hello-pwa](https://prest.blog/hello-pwa)
