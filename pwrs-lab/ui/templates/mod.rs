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
