use webmanifest::{DisplayMode, Icon, Manifest};

static DEFAULT_DESC: &str = "An installable web application";
static DEFAULT_BG: &str = "#1e293b";
static DEFAULT_THEME: &str = "#a21caf";
static DEFAULT_START: &str = "/";
static DEFAULT_DISPLAY: DisplayMode = DisplayMode::Standalone;
static DEFAULT_ICON_SRC: &str = "logo.png";
static DEFAULT_ICON_SIZE: &str = "512x512";

pub fn compose() -> String {
    let name = std::env::var("CARGO_PKG_NAME").unwrap();
    let desc = if let Ok(desc) = std::env::var("CARGO_PKG_DESCRIPTION") {
        desc
    } else {
        DEFAULT_DESC.to_string()
    };
    let manifest = Manifest::builder(&name)
        .description(&desc)
        .bg_color(DEFAULT_BG)
        .theme_color(DEFAULT_THEME)
        .start_url(DEFAULT_START)
        .display_mode(DEFAULT_DISPLAY.clone())
        .icon(&Icon::new(DEFAULT_ICON_SRC, DEFAULT_ICON_SIZE))
        .build()
        .unwrap();
    manifest
}
