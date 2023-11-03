This is a hobby project, there are no solid long-term plans and api is unstable. However, there are things I'll likely work on next:

- only prest logo and menu in header. Intro, github => readme, roadmap => menu/todo app?, list of examples => separate page into menu. Modules => rust docs?
- fill in examples' readmes and clarify prios in the roadmap
- embed blog styles.css without build.rs?
- move scripts from Head into a separate thing
- update PWAOptions: remove fav, new logo - P as cog with bar + REST?. 
- disable cache for htmx requests to avoid caching broken pages
- extract todo ui into a separate crate to refine Storybook-like workflow
- move target_sw into target

### Examples
+ with-[seaql](https://www.sea-ql.org/)-postgres toolkit stuff
+ with-[oxc](https://github.com/web-infra-dev/oxc)-typescript
+ into-[hermit](https://github.com/dylibso/hermit)-cosmo...

### Publish
It's not on [crates.io](https://crates.io/crates/prest) yet because it depends on the latest unpublished [axum](https://github.com/tokio-rs/axum) changes. Awaiting it's 0.7 release to publish the first alpha version.
