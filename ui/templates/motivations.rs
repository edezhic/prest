use crate::external_link::render as link;
pub fn render() -> maud::Markup {
    maud::html!(
        //(back_button(""))
        ."highlight" {  
            p{"Progressive Web Application ("(link("PWA", "web.dev/learn/pwa"))") is a web app that can also feel like a native one. This is a nice way to build apps because you don't need to start with a complicated setup - even static HTML with generated service worker and manifest can do the trick with aggressive caching to enable offline pages. Then you can add more features that users can opt-in for."}
            p{"To develop these features JS is the default choice as it is native for the browsers, but it is problematic when you need performance, safety and system interactions, especially on the backend side. While Node significantly extended JS use cases, it didn't resolve performance and scalability concerns. JS is really convenient when you need small scripts for specific tasks, but devex and performance quickly degrades as the app's codebase grows. Typescript solves some of these issues, but still many devs reasonably prefer to have a separate BE stack."}
            p{"That leads to another problem - integration between your BE and FE. Quite often it happens only in the REST API boundary and sometimes that's fine, with tools like graphql you can even make this integration mostly type-safe. But, most features require both stacks to work together so whatever separation of concerns you had between FE and BE becomes more of a problem than a solution."}
            p{"So we need reliability, performance and integration with both FE tooling and underlying systems to enable efficient web development. This project started as an experiment to figure out whether Rust can deliver that and now I'm convinced that it can. Let's start with an overview of the main benefits of rust:"}
            ol{
                li{b{"speed"}" - rust is a compiled language like C so it doesn't have to bundle a runtime and can use lots of "(link("zero-cost abstractions", "stackoverflow.com/a/75717287"))" to avoid overhead. This way your apps will require minimal resources but you can also push the available hardware to the limits when needed"}
                li{b{"safety"}" - rust like C is a strongly-typed language, but unlike C it also includes memory control system - "(link("borrow checker", "doc.rust-lang.org/1.8.0/book/references-and-borrowing.html"))" which safeguards against memory leaks, dangling pointers, race conditions and many other memory-related problems"}
            }
            p{"Also, rust accumulated best practices from many languages in it's type system, error handling strategy, standard library and many other aspects. However, it also has some notable issues:"}
            ol{
                li{"rich yet often unstable ecosystem"}
                li{"because of all these type and memory checks builds might take a while"}
                li{"this kind of memory management hasn't been implemented in any other mainstream language yet so pretty much everyone gets confused by it at the beginning"}
            }
            p{"All of that together makes rust a nice fit for low-level/high-load services but lame for prototyping and early stage web development. So, we need a setup which allows to quickly add some HTML/CSS/JS and deploy, while also leaving a clear path to scale for any needs - that's what "b{"pwrs"}" is all about, while pwrsapp is meant to be a simple yet flexible boilerplate to jump-start your projects."}
            p{"However, if you already have some kind of preferred framework and don't want to rebase whole project on rust - don't worry! You can cherry-pick the pieces of pwrs that seem interesting to you and embed these modules into your existing stack with practically zero runtime overhead."}
            p{"If you're just starting your rust journey but have some programming experience I'd highly recommend "(link("the official docbook", "doc.rust-lang.org/book"))}
        }
        //(back_button("motivations"))
    )
}