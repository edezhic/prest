//! Fork of [rust-embed](https://github.com/pyrossh/rust-embed) adjusted to be used in prest. Thanks to Peter John <pyros2097@gmail.com> and other contributors!
//! Changes in the API: 
//! - Implemented `.embed` function for the axum Router that adds routes to the embedded files
//! - Added shorthand macros for embedding: `include_as` and `include_build_output_as` 
//! - `interpolate-folder-path` and `include-exclude` are enabled without additional features
//! - `compression` feature is removed because RAM and cold starts are more important than disk space for most prest use cases 
//! - Derive macro is renamed RustEmbed -> Embed

use crate::*;
use std::borrow::Cow;

pub trait Embed {
    fn get(file_path: &str) -> Option<EmbeddedFile>;
    fn iter() -> Filenames;
}

pub trait EmbedRoutes {
    fn embed<T: Embed>(self, _: T) -> Self;
}
impl EmbedRoutes for Router {
    fn embed<T: Embed>(mut self, _: T) -> Self {
        for path in T::iter() {
            self = self.route(
                &format!("/{path}"),
                get(|headers: HeaderMap| async move { file_handler::<T>(&path, headers) })
            )
        }
        self
    }
}

fn file_handler<T: Embed + ?Sized>(path: &str, headers: HeaderMap) -> Response {
    let Some(asset) = T::get(&path) else {
        return StatusCode::NOT_FOUND.into_response();
    };

    let asset_etag = hex::encode(asset.metadata.sha256_hash());
    if let Some(request_etag) = headers.get(header::IF_NONE_MATCH) {
        if request_etag.as_bytes() == asset_etag.as_bytes() {
            return StatusCode::NOT_MODIFIED.into_response();
        }
    }
    Response::builder()
        .header(header::ETAG, asset_etag)
        .header(header::CONTENT_TYPE, asset.metadata.mimetype())
        .body(Body::from(asset.data))
        .unwrap()
}

/// Shorthand to embed build artifacts like PWA assets and others
/// 
/// Usage: `include_build_output_as!(StructName);`
#[macro_export]
macro_rules! include_build_output_as {
    ($struct_name:ident) => {
        #[derive(Embed)]
        #[folder = "$OUT_DIR"]
        struct $struct_name;
    };
}
/// One-liner for structs with derived Embed
///
/// Usage: `include_as!(StructName from "path" only "*.rs" except "secret.rs");` where `only...` and `except...` parts are optional and accept 1+ comma-separated arg
#[macro_export]
macro_rules! include_as {
    ($struct_name:ident from $path:literal $(only $($inc:literal),+)? $(except $($exc:literal),+)?) => {
        #[derive(Embed)]
        #[folder = $path]
        $( $( #[include = $inc] )+ )?
        $( $( #[exclude = $exc] )+ )?
        struct $struct_name;
    };
}

/// This enum exists for optimization purposes, to avoid boxing the iterator in
/// some cases. Do not try and match on it, as different variants will exist
/// depending on the compilation context.
pub enum Filenames {
    /// Release builds use a named iterator type, which can be stack-allocated.
    #[cfg(any(not(debug_assertions), feature = "lazy-embed"))]
    Embedded(std::slice::Iter<'static, &'static str>),

    /// The debug iterator type is currently unnameable and still needs to be
    /// boxed.
    #[cfg(all(debug_assertions, not(feature = "lazy-embed")))]
    Dynamic(Box<dyn Iterator<Item = Cow<'static, str>>>),
}

impl Iterator for Filenames {
    type Item = Cow<'static, str>;
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            #[cfg(any(not(debug_assertions), feature = "lazy-embed"))]
            Filenames::Embedded(names) => names.next().map(|x| Cow::from(*x)),

            #[cfg(all(debug_assertions, not(feature = "lazy-embed")))]
            Filenames::Dynamic(boxed) => boxed.next(),
        }
    }
}
