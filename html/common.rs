use crate::*;

/// Renders into a `<head>` tag with builder-like interface
pub struct Head<'a> {
    title: &'a str,
    styles: Option<Vec<PreEscaped<&'a str>>>,
    stylesheets: Option<Vec<&'a str>>,
    favicon: Option<&'a str>,
    webmanifest: Option<&'a str>,
    viewport: Option<&'a str>,
    theme_color: Option<&'a str>,
    other: Option<Markup>,
}

impl<'a> Head<'a> {
    /// Add custom markup to the [`Head`]
    pub fn with(mut self, other: Markup) -> Self {
        self.other = Some(other);
        self
    }
    /// Set title in the [`Head`]
    pub fn title(mut self, title: &'a str) -> Self {
        self.title = title;
        self
    }
    /// Add stylesheet link to the [`Head`]
    pub fn css(mut self, path: &'a str) -> Self {
        if let Some(stylesheets) = &mut self.stylesheets {
            stylesheets.push(path)
        } else {
            self.stylesheets = Some(vec![path]);
        }
        self
    }
    /// Add inline css to the [`Head`]
    pub fn style(mut self, style: PreEscaped<&'a str>) -> Self {
        if let Some(styles) = &mut self.styles {
            styles.push(style)
        } else {
            self.styles = Some(vec![style]);
        }
        self
    }
    /// Builds a default [`Head`] with provided title
    pub fn with_title(title: &'a str) -> Self {
        Self::default().title(title)
    }
}

impl<'a> Default for Head<'a> {
    fn default() -> Self {
        let webmanifest = match is_pwa() {
            true => Some("/.webmanifest"),
            false => None,
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
                script src="https://cdn.tailwindcss.com?plugins=typography" {}
                link href="https://cdn.jsdelivr.net/npm/daisyui@4.5.0/dist/full.min.css" rel="stylesheet"{}
                @if let Some(stylesheets) = &self.stylesheets { @for stylesheet in stylesheets {link href={(stylesheet)} rel="stylesheet"{}}}
                @if let Some(styles) = &self.styles { @for style in styles { style {(style)}}}
                @if let Some(markup) = &self.other {(markup)}
            }
        )
    }
}

/// Renders into a bunch of `<script>` tags with builder-like interface
pub struct Scripts<'a> {
    pub others: Option<Vec<&'a str>>,
    pub inlines: Option<Vec<&'a str>>,
}

impl<'a> Default for Scripts<'a> {
    fn default() -> Self {
        Self {
            others: None,
            inlines: None,
        }
    }
}

impl<'a> Scripts<'a> {
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

impl<'a> Render for Scripts<'a> {
    fn render(&self) -> Markup {
        #[cfg(debug_assertions)]
        let htmx_src = "https://unpkg.com/htmx.org@1.9.10/dist/htmx.js";
        #[cfg(not(debug_assertions))]
        let htmx_src = "https://unpkg.com/htmx.org@1.9.10";

        html!(
            @if is_pwa() { script {(REGISTER_SW_SNIPPET)} }
            script src=(htmx_src) defer crossorigin {}
            script src="https://unpkg.com/htmx.org/dist/ext/sse.js" defer crossorigin {}
            @if let Some(srcs) = &self.others { @for src in srcs {
                script src={(src)} defer crossorigin {}
            }}
            @if let Some(scripts) = &self.inlines { @for script in scripts {
                script {(PreEscaped(script))}
            }}
        )
    }
}
