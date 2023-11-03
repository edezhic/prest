use crate::*;

pub struct Head<'a> {
    title: &'a str,
    styles: Option<Vec<&'a str>>,
    favicon: Option<&'a str>,
    webmanifest: Option<&'a str>,
    viewport: Option<&'a str>,
    theme_color: Option<&'a str>,
    include_picocss: bool,
    other: Option<Markup>,
}

impl<'a> Head<'a> {
    pub fn with(mut self, other: Markup) -> Self {
        self.other = Some(other);
        self
    }
    pub fn title(mut self, title: &'a str) -> Self {
        self.title = title;
        self
    }
    pub fn css(mut self, path: &'a str) -> Self {
        if let Some(styles) = &mut self.styles {
            styles.push(path)
        } else {
            self.styles = Some(vec![path]);
        }
        self
    }
    pub fn fav(mut self, path: &'a str) -> Self {
        self.favicon = Some(path);
        self
    }
    pub fn webmanifest(mut self, path: &'a str) -> Self {
        self.webmanifest = Some(path);
        self
    }
    pub fn default_pwa() -> Self {
        if cfg!(debug_assertions) {
            Self::default().title("Prest PWA")
        } else {
            Self::default().title("Prest PWA").webmanifest("/.webmanifest")
        }
    }
}

impl<'a> Default for Head<'a> {
    fn default() -> Self {
        Self {
            title: "Prest app",
            viewport: Some("width=device-width, initial-scale=1.0"),
            include_picocss: true,
            styles: None,
            favicon: None,
            webmanifest: None,
            theme_color: None,
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
                @if let Some(href) = self.webmanifest { link rel="manifest" href=(href) {} }
                @if let Some(viewport) = self.viewport { meta name="viewport" content=(viewport); }
                @if let Some(color) = self.theme_color { meta name="theme-color" content=(color); }
                @if let Some(stylesheets) = self.styles.clone() { @for stylesheet in stylesheets {
                    link href={(stylesheet.clone())} rel="stylesheet" {}
                }}
                @if self.include_picocss { link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/@picocss/pico@1/css/pico.min.css"{} }
            }
        )
    }
}

pub struct Scripts<'a> {
    pub register_sw: bool,
    pub include_htmx: bool,
    pub include_hyperscript: bool,
    pub other_srcs: Option<Vec<&'a str>>,
    pub other: Option<Markup>,
}

impl<'a> Scripts<'a> {
    pub fn include(mut self, path: &'a str) -> Self {
        if let Some(srcs) = &mut self.other_srcs {
            srcs.push(path)
        } else {
            self.other_srcs = Some(vec![path])
        }
        self
    }
    pub fn empty() -> Self {
        Self {
            register_sw: false,
            include_htmx: false,
            include_hyperscript: false,
            other_srcs: None,
            other: None,
        }
    }
    pub fn with_sw(mut self) -> Self {
        self.register_sw = true;
        self
    }
    pub fn default_pwa() -> Self {
        if cfg!(debug_assertions) {
            Self::default()
        } else {
            Self::default().with_sw()
        }
    }
}

impl<'a> Default for Scripts<'a> {
    fn default() -> Self {
        Self {
            register_sw: false,
            include_htmx: true,
            include_hyperscript: true,
            other_srcs: None,
            other: None,
        }
    }
}
impl<'a> Render for Scripts<'a> {
    fn render(&self) -> Markup {
        html!(
            @if self.register_sw { script {(REGISTER_SW_SNIPPET)} }
            @if let Some(srcs) = self.other_srcs.clone() { @for src in srcs {
                script src={(src)} {}
            }}
            @if self.include_htmx { script defer src="https://unpkg.com/htmx.org@1.9.0" crossorigin="anonymous"{} }
            @if self.include_hyperscript { script defer src="https://unpkg.com/hyperscript.org@0.9.11" crossorigin="anonymous"{} }
            @if let Some(markup) = &self.other {(markup.clone())}
        )
    }
}
