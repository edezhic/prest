In general rust supports plenty of compilation targets including desktops, servers, mobile and wasm. So, the future of cross-compilation to build the same codebase for all the major consumer platforms is pretty bright. However, in prest this is not a priority and left as a last resort for cases where web capabilities cannot cover or even emulate some native apis - sometimes early product seems simple enough and you might not see a need for direct access to android or ios apis, but later on there might be some shiny new features on platforms or in your app that you'll want to use. So, it's pretty good to have such an option and prest should at least remain compatible with such integrations.

In such cases you can build a client-side app based on platforms' built-in webviews as native packages for desktops and [mobile](https://github.com/tauri-apps/wry/blob/dev/MOBILE.md) using [WRY](https://github.com/tauri-apps/wry) and [TAO](https://github.com/tauri-apps/tao):

{Cargo.toml}

{src/main.rs}

You won't need to split or rewrite your existing prest app to do that. For example, you can make your native binary's webview connect to the existing remote host while additionally providing some localhost endpoints in the WRY-wrapped binary for native APIs. This solution is relatively simple and RESTful, but still somewhat weird and hasn't been tried in practice so proceed with caution.

Another option would be to provide a single endpoint to signal that webview should be replaced with some other view that will take over the UI. You can also add json or other types of endpoints to the existing host to support this kind of UIs. As a last resort you can start with a separate set of endpoints that use exiting DB models and business logic, and create a totally separate client using native kits, Flutter, [Tauri](https://tauri.app/) or anything else.