pub fn check_attr_name_alias(name: &str) -> &str {
    match name {
        "get" => "hx-get",
        "put" => "hx-put",
        "post" => "hx-post",
        "patch" => "hx-patch",
        "delete" => "hx-delete",
        "into" => "hx-target",
        "vals" => "hx-vals",
        "trigger" => "hx-trigger",
        "include" => "hx-include",
        "boost" => "hx-boost",
        "history-elt" => "hx-history-elt",
        "before-request" => "hx-on--before-request",
        "after-request" => "hx-on--after-request",
        _ => name,
    }
}

pub fn check_attr_shorthand(name: &str) -> Option<&str> {
    match name {
        "swap-inner" => Some(r#"hx-swap="innerHTML""#),
        "swap-full" => Some(r#"hx-swap="outerHTML""#),
        "swap-textContent" => Some(r#"hx-swap="textContent""#),
        "swap-beforebegin" => Some(r#"hx-swap="beforebegin""#),
        "swap-afterbegin" => Some(r#"hx-swap="afterbegin""#),
        "swap-beforeend" => Some(r#"hx-swap="beforeend""#),
        "swap-afterend" => Some(r#"hx-swap="afterend""#),
        "swap-delete" => Some(r#"hx-swap="delete""#),
        "swap-none" => Some(r#"hx-swap="none""#),
        _ => None,
    }
}