use crate::*;

/// Renders into a `<head>` tag with builder-like interface
pub struct Head<'a> {
    title: &'a str,
    styles: Option<Vec<&'a str>>,
    stylesheets: Option<Vec<&'a str>>,
    favicon: Option<&'a str>,
    webmanifest: Option<&'a str>,
    viewport: Option<&'a str>,
    theme_color: Option<&'a str>,
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
        if let Some(stylesheets) = &mut self.stylesheets {
            stylesheets.push(path)
        } else {
            self.stylesheets = Some(vec![path]);
        }
        self
    }
    pub fn style(mut self, style: &'a str) -> Self {
        if let Some(styles) = &mut self.styles {
            styles.push(style)
        } else {
            self.styles = Some(vec![style]);
        }
        self
    }
    /// Builds a [`Head`] with configs used across examples
    pub fn example(title: &'a str) -> Self {
        Self::default()
            .title(title)
            .css("https://cdn.jsdelivr.net/npm/@picocss/pico@1/css/pico.min.css")
    }
}

impl<'a> Default for Head<'a> {
    fn default() -> Self {
        let webmanifest = match cfg!(debug_assertions) {
            true => None,
            false => Some("/.webmanifest"),
        };
        Self {
            title: "Prest app",
            viewport: Some("width=device-width, initial-scale=1.0"),
            webmanifest,
            styles: None,
            stylesheets: Some(vec!["/default-view-transition.css"]),
            favicon: None,
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
                @if let Some(stylesheets) = self.stylesheets.clone() { @for stylesheet in stylesheets {
                    link href={(stylesheet.clone())} rel="stylesheet" {}
                }}
                @if let Some(styles) = self.styles.clone() { @for style in styles {
                    style {(style)}
                }}
                @if let Some(markup) = self.other.clone() {(markup)}
            }
        )
    }
}

/// Renders into a bunch of `<script>` tags with builder-like interface
pub struct Scripts<'a> {
    pub register_sw: bool,
    pub others: Option<Vec<&'a str>>,
    pub inlines: Option<Vec<&'a str>>,
}

impl<'a> Scripts<'a> {
    pub fn empty() -> Self {
        Self {
            register_sw: false,
            others: None,
            inlines: None,
        }
    }
    pub fn include(mut self, path: &'a str) -> Self {
        if let Some(srcs) = &mut self.others {
            srcs.push(path)
        } else {
            self.others = Some(vec![path])
        }
        self
    }
    pub fn inline(mut self, script: &'a str) -> Self {
        if let Some(scripts) = &mut self.inlines {
            scripts.push(script)
        } else {
            self.inlines = Some(vec![script])
        }
        self
    }
}

impl<'a> Default for Scripts<'a> {
    fn default() -> Self {
        let others = Some(vec![
            "https://unpkg.com/htmx.org@1.9.0",
            "https://unpkg.com/hyperscript.org@0.9.11",
        ]);
        Self {
            register_sw: cfg!(debug_assertions),
            others,
            inlines: None,
        }
    }
}
impl<'a> Render for Scripts<'a> {
    fn render(&self) -> Markup {
        html!(
            @if self.register_sw { script {(REGISTER_SW_SNIPPET)} }
            @if let Some(srcs) = self.others.clone() { @for src in srcs {
                script src={(src)} defer crossorigin {}
            }}
            @if let Some(scripts) = self.inlines.clone() { @for script in scripts {
                script {(PreEscaped(script))}
            }}
        )
    }
}
