pub mod home;
pub mod platforms;
pub mod motivations;
pub mod internals;
pub mod external_link {
    pub fn render(text: &str, url: &str) -> maud::Markup {
        let href = format!("https://{}", url);
        maud::html!(a."external-link" href={(href)} target="_blank" {(text)})
    }
}

/*
pub fn forward_button(text: &str, page: &str) -> Markup {
    html!(
      a #(page) href={(format!("/{page}"))} hx-boost="true" .(STYLISH_NAV_BUTTON)."pr-4 border-r-4 bg-gradient-to-r text-end"
        hx-swap="innerHTML transition:true show:body:top"
          {(text)}
    )
}

pub fn back_button(fragment: &str) -> Markup {
    let htmx_scroll_to_fragment = if fragment != "" {
        format!("show:#{fragment}:top")
    } else {
        "".to_owned()
    };
    html!(
        a href={(format!("/#{fragment}"))} hx-boost="true" .(STYLISH_NAV_BUTTON)."pl-4 border-l-4 bg-gradient-to-l"
          hx-swap={(format!("innerHTML transition:true {htmx_scroll_to_fragment}"))}
          {"back home"}
    )
}
 */