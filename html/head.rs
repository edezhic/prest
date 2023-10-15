use crate::*;

pub struct Head<'a> {
    title: &'a str,
    favicon: Option<&'a str>,
    manifest: Option<&'a str>,
    viewport: Option<&'a str>,
    theme_color: Option<&'a str>,
    register_sw: bool,
    include_htmx: bool,
    include_picocss: bool,
    other: Option<Markup>,
}

impl<'a> Head<'a> {
    pub fn with(mut self, other: Markup) -> Head<'a> {
        self.other = Some(other);
        self
    }
    pub fn title(mut self, title: &'a str) -> Head<'a> {
        self.title = title;
        self
    }
    pub fn pwa() -> Head<'a> {
        Self {
            title: "Prest PWA",
            favicon: Some("/favicon.ico"),
            manifest: Some("/.webmanifest"),
            viewport: Some("width=device-width, initial-scale=1.0"),
            theme_color: Some("#a21caf"),
            register_sw: true,
            include_htmx: true,
            include_picocss: true,
            other: None,
        }
    }
}

impl<'a> Default for Head<'a> {
    fn default() -> Self {
        Self {
            title: "Prest app",
            favicon: None,
            manifest: None,
            viewport: Some("width=device-width, initial-scale=1.0"),
            theme_color: None,
            register_sw: false,
            include_htmx: true,
            include_picocss: true,
            other: None,
        }
    }
}

impl<'a> Render for Head<'a> {
    fn render(&self) -> Markup {
        html!(
            head {
                title {(self.title)}
                @if let Some(href) = self.favicon { link rel="icon" href=(href) {} }
                @if let Some(href) = self.manifest { link rel="manifest" href=(href) {} }
                @if let Some(viewport) = self.viewport { meta name="viewport" content=(viewport); }
                @if let Some(color) = self.theme_color { meta name="theme-color" content=(color); }
                @if self.register_sw { script {(REGISTER_SW_SNIPPET)} }
                @if self.include_htmx { script src="https://unpkg.com/htmx.org@1.9.0" integrity="sha384-aOxz9UdWG0yBiyrTwPeMibmaoq07/d3a96GCbb9x60f3mOt5zwkjdbcHFnKH8qls" crossorigin="anonymous"{} }
                @if self.include_picocss { link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/@picocss/pico@1/css/pico.min.css"{} }
                @if let Some(markup) = &self.other {(markup.clone())}
            }
        )
    }
}