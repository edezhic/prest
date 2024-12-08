use crate::*;

static DEFAULT_CSS: PreEscaped<&str> = PreEscaped(
    r#"*{transition: all 100ms}@keyframes fade-in {from {opacity: 0;transform: translateX(90px);}}@keyframes fade-out {to {opacity: 0;transform: translateX(-90px);}}::view-transition-old(slide-it) {animation: 300ms cubic-bezier(0.4, 0, 1, 1) both fade-out;}::view-transition-new(slide-it) {animation: 420ms cubic-bezier(0, 0, 0.2, 1) 90ms both fade-in;}[hx-history-elt] {view-transition-name: slide-it;}*, ::before, ::after {box-sizing: border-box;border-width: 0;border-style: solid;border-color: currentColor;}::before, ::after {--tw-content: '';}html {line-height: 1.5;-webkit-text-size-adjust: 100%;-moz-tab-size: 4;tab-size: 4;font-family: 'fontFamily.sans', ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, "Noto Sans", sans-serif, "Apple Color Emoji", "Segoe UI Emoji", "Segoe UI Symbol", "Noto Color Emoji";}body {margin: 0;line-height: inherit;}hr {height: 0;color: inherit;border-top-width: 1px;}abbr:where([title]) {text-decoration: underline dotted;}h1, h2, h3, h4, h5, h6 {font-size: inherit;font-weight: inherit;}a {color: inherit;text-decoration: inherit;}b, strong {font-weight: bolder;}code, kbd, samp, pre {font-family: 'fontFamily.mono', ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace;font-size: 1em;}small {font-size: 80%;}sub, sup {font-size: 75%;line-height: 0;position: relative;vertical-align: baseline;}sub {bottom: -0.25em;}sup {top: -0.5em;}table {text-indent: 0;border-color: inherit;border-collapse: collapse;}button, input, optgroup, select, textarea {font-family: inherit;font-size: 100%;font-weight: inherit;line-height: inherit;color: inherit;margin: 0;padding: 0;}button, select {text-transform: none;}button, [type='button'], [type='reset'], [type='submit'] {-webkit-appearance: button;background-color: transparent;background-image: none;}:-moz-focusring {outline: auto;}:-moz-ui-invalid {box-shadow: none;}progress {vertical-align: baseline;}::-webkit-inner-spin-button, ::-webkit-outer-spin-button {height: auto;}[type='search'] {-webkit-appearance: textfield;outline-offset: -2px;}::-webkit-search-decoration {-webkit-appearance: none;}::-webkit-file-upload-button {-webkit-appearance: button;font: inherit;}summary {display: list-item;}blockquote, dl, dd, h1, h2, h3, h4, h5, h6, hr, figure, p, pre {margin: 0;}fieldset {margin: 0;padding: 0;}legend {padding: 0;}ol, ul, menu {list-style: none;margin: 0;padding: 0;}textarea {resize: vertical;}input::placeholder, textarea::placeholder {opacity: 1;color: #9ca3af;}button, [role="button"] {cursor: pointer;}:disabled {cursor: default;}img, svg, video, canvas, audio, iframe, embed, object {display: block;vertical-align: middle;}img, video {max-width: 100%;height: auto;}[hidden] {display: none;}"#,
);

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
            stylesheets: None,
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
                @if let Some(stylesheets) = &self.stylesheets { @for stylesheet in stylesheets {link href={(stylesheet)} rel="stylesheet"{}}}
                style {(DEFAULT_CSS)}
                @if let Some(styles) = &self.styles { @for style in styles { style {(style)}}}
                @if let Some(markup) = &self.other {(markup)}
            }
        )
    }
}

/// Renders into a bunch of `<script>` tags with builder-like interface
pub struct Scripts<'a> {
    pub default_bundle: bool,
    pub others: Option<Vec<&'a str>>,
    pub inlines: Option<Vec<&'a str>>,
    pub stylesheets: Option<Vec<&'a str>>,
    pub hyperscripts: Option<Vec<&'a str>>,
}

impl<'a> Default for Scripts<'a> {
    fn default() -> Self {
        Self {
            default_bundle: true,
            others: None,
            inlines: None,
            // inlines: Some(vec![include_str!("./htmx_patch.js")]),
            stylesheets: None,
            hyperscripts: None,
        }
    }
}

impl<'a> Scripts<'a> {
    /// Add script src to the [`Scripts`]
    pub fn include(mut self, path: &'a str) -> Self {
        if let Some(srcs) = &mut self.others {
            srcs.push(path)
        } else {
            self.others = Some(vec![path])
        }
        self
    }
    /// Add inline js to the [`Scripts`]
    pub fn inline(mut self, script: &'a str) -> Self {
        if let Some(scripts) = &mut self.inlines {
            scripts.push(script)
        } else {
            self.inlines = Some(vec![script])
        }
        self
    }
    /// Add stylesheet link to the [`Scripts`]
    pub fn css(mut self, path: &'a str) -> Self {
        if let Some(stylesheets) = &mut self.stylesheets {
            stylesheets.push(path)
        } else {
            self.stylesheets = Some(vec![path]);
        }
        self
    }
    /// Add inline hyperscipt to the [`Scripts`]
    pub fn hyperscript(mut self, script: &'a str) -> Self {
        if let Some(scripts) = &mut self.hyperscripts {
            scripts.push(script)
        } else {
            self.hyperscripts = Some(vec![script])
        }
        self
    }
}

impl<'a> Render for Scripts<'a> {
    fn render(&self) -> Markup {
        html!(
            @if is_pwa() { script {(REGISTER_SW_SNIPPET)} }
            @if let Some(stylesheets) = &self.stylesheets { @for stylesheet in stylesheets {link href={(stylesheet)} rel="stylesheet"{}}}
            @if self.default_bundle {
                script src="/prest.js" {}    
            }
            @if let Some(srcs) = &self.others { @for src in srcs {
                script src={(src)} crossorigin {}
            }}
            @if let Some(scripts) = &self.inlines { @for script in scripts {
                script {(PreEscaped(script))}
            }}
            @if let Some(scripts) = &self.hyperscripts { @for script in scripts {
                script type="text/hyperscript" {(PreEscaped(script))}
            }}
        )
    }
}
