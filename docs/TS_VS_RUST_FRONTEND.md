While prest is mostly focused on rust, I've made a significant effort to make it work well with typescript for the interfaces. Why not for example dioxus which is really awesome? Or some other rust -> wasm/native UI solution? Overall, as much as I'd love to write everything in Rust I have to consider smth:

First of all - ecosystem. Since the death of Flash UI development has been almost exclusively JS-based. Not because ppl love JS that much. Not because it's fast or reliable (because it's neither). But it is the most widely supported one, somewhat like Java with it's "installed on X billion devices" - cancer that we have to deal with. This ecosystem has pretty much everything you might want or need. No other UI dev tooling comes even close (except, again, gaming-related stuff, but it's a whole different story).

Second - there is no second. That's it. There are a whole bunch of reasons to pick other frontend tools, especially rust-based ones for full-stack in a single language experience, but none of them comes close in the development speed to JS. Especially if you're sprinkling some TypeScript on top to keep some sanity in the codebase.

Native interfaces are cool, but for 99% of the projects (excluding games) performance gains just aren't worth the effort. Especially considering how hard browser engine developers are optimizing them. 

Service worker part is already compiled into wasm and runs on the client side. However, it's fairly simple and requires just a few outside intefaces to integrate, and it's easily cross-compiled from the existing server code. It's closer to server-side development than client-side. 

Being able to write everything in rust sounds amazing to me. Yet reinventing all the libraries I could use with JS does not.

As awesome as dioxus is, it's will be hard to compete with JS ecosystem in the foreseeable future. But maybe prest can adopt dioxus's hot reloading? Definitely worth a try.