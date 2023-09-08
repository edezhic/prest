static LISTENER_TEMPLATE: &str = "self.addEventListener('NAME', event => LISTENER);\n";
static DEFAULT_LISTENERS: [(&str, &str); 3] = [
    (
        "install",
        "event.waitUntil(Promise.all([__wbg_init('/sw.wasm'), self.skipWaiting()]))",
    ),
    ("activate", "event.waitUntil(self.clients.claim())"),
    ("fetch", "serve(self, event)")
];

pub fn append_sw_listeners(mut bindings: String) -> String {
    for listener in DEFAULT_LISTENERS {
        bindings += LISTENER_TEMPLATE.replace("NAME", listener.0).replace("LISTENER", listener.1).as_str();
    }
    bindings
}
