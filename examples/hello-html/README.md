Prest respects [HATEOAS](https://htmx.org/essays/hateoas/) constraint of the REST and focused on HTML to build cross-platform UIs. It's not universally the best approach for the frontend development though - you should consider [when to use hypermedia](https://htmx.org/essays/when-to-use-hypermedia/) before taking this architectural decision, but it's good enough for a lot of cases. I think even more than the linked essay suggests.

Cargo manifest is almost the same except we add a new dependency [serde](https://serde.rs/) - go-to serialization and deserialization library in the rust ecosystem. It's not exported in prest because app-level code is mostly using it's macros which are pretty painful re-export without forking, but almost every rust app has it among dependencies so I'd suggest getting used to it: 

{Cargo.toml}

In this example we'll use an `html!` macro for templating forked from [maud](https://github.com/lambda-fairy/maud), as well as [htmx](https://htmx.org/) for reactivity and [hyperscript](https://hyperscript.org/) to sprinkle a bit more interactivity. 

{serve.rs}

Twind can also be a good addition to this stack because it allows adding styles easily and ergonomically right into the markup.

Now we have a powerful server and a pretty smooth UI. That's good enough for a lot of projects but let's make a step further and make it installable so it feels almost like a native app.

[Next: hello-pwa](https://prest.blog/hello-pwa)
