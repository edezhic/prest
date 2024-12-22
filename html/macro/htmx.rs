pub fn check_attr_name_alias(name: &str) -> Option<&str> {
    match name {
        "get" => Some("hx-get"),
        "put" => Some("hx-put"),
        "post" => Some("hx-post"),
        "patch" => Some("hx-patch"),
        "delete" => Some("hx-delete"),
        "target" => Some("hx-target"),
        "vals" => Some("hx-vals"),
        "swap" => Some("hx-swap"),
        "trigger" => Some("hx-trigger"),
        "include" => Some("hx-include"),
        "boost" => Some("hx-boost"),
        "history-elt" => Some("hx-history-elt"),
        "before-request" => Some("hx-on--before-request"),
        "after-request" => Some("hx-on--after-request"),
        "sse-msg" => Some("sse-swap"),
        _ => None,
    }
}

pub fn check_attr_shorthand(name: &str) -> Option<&str> {
    match name {
        "swap-this" => Some(r#"hx-target="this""#),
        "swap-inner" => Some(r#"hx-swap="innerHTML""#),
        "swap-full" => Some(r#"hx-swap="outerHTML""#),
        "swap-textContent" => Some(r#"hx-swap="textContent""#),
        "swap-before-begin" => Some(r#"hx-swap="beforebegin""#),
        "swap-after-begin" => Some(r#"hx-swap="afterbegin""#),
        "swap-before-end" => Some(r#"hx-swap="beforeend""#),
        "swap-after-end" => Some(r#"hx-swap="afterend""#),
        "swap-delete" => Some(r#"hx-swap="delete""#),
        "swap-none" => Some(r#"hx-swap="none""#),
        "replace-url" => Some(r#"hx-replace-url="true""#),
        "push-url" => Some(r#"hx-push-url="true""#),
        "no-replace-url" => Some(r#"hx-replace-url="false""#),
        "no-push-url" => Some(r#"hx-push-url="false""#),
        // complex ones
        "sse" => Some(r#"hx-ext="sse" sse-connect"#),
        "into" => Some(r#"hx-swap="innerHTML" hx-target"#),
        "put-before" => Some(r#"hx-swap="beforebegin" hx-target"#),
        "into-end-of" => Some(r#"hx-swap="beforeend" hx-target"#),
        "put-after" => Some(r#"hx-swap="afterend" hx-target"#),
        "swap-this-no-transition" => Some(r#"hx-target="this" hx-swap="transition:false""#),
        _ => None,
    }
}
