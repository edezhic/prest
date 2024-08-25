//! Fork of [rust-embed](https://github.com/pyrossh/rust-embed) adjusted to be used in prest. Thanks to Peter John <pyros2097@gmail.com> and other contributors!
//! Changes in the API:
//! - Implemented `.embed` function for the axum Router that adds routes to the embedded files
//! - Added shorthand macros for embedding: `embed_as` and `embed_build_output_as`
//! - `interpolate-folder-path` and `include-exclude` are enabled without additional features
//! - `compression` feature is removed because RAM and cold starts are more important than disk space for most prest use cases
//! - Derive macro is renamed RustEmbed -> Embed

use crate::*;

pub use prest_embed_macro::Embed;
pub use prest_embed_utils::*;

use std::borrow::Cow;

/// Derived trait for structs that embed files
pub trait Embed {
    fn iter() -> __Filenames;
    fn get(file_path: &str) -> Option<EmbeddedFile>;
    fn get_content(file_path: &str) -> Option<String> {
        if let Some(file) = Self::get(file_path) {
            Some(std::str::from_utf8(&file.data).unwrap().to_owned())
        } else {
            None
        }
    }
}

/// Convenience trait that generates routes for the embedded files
#[doc(hidden)]
pub trait EmbedRoutes {
    fn embed<T: Embed>(self, _: T) -> Self;
}
impl EmbedRoutes for Router {
    fn embed<T: Embed>(mut self, _: T) -> Self {
        for path in T::iter() {
            self = self.route(
                &format!("/{path}"),
                get(|headers: HeaderMap| async move { file_handler::<T>(&path, headers) }),
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

    #[allow(unused_mut)]
    let mut response = Response::builder();

    #[cfg(not(debug_assertions))]
    if path == "sw.js" || path == "sw.wasm" {
        response = response.header(
            header::CACHE_CONTROL,
            "max-age=60, stale-while-revalidate=3600, stale-if-error=604800",
        );
    }

    response
        .header(header::ETAG, asset_etag)
        .header(header::CONTENT_TYPE, asset.metadata.mimetype())
        .body(Body::from(asset.data))
        .unwrap()
}

/// Shorthand to embed build artifacts like PWA assets and others
///
/// Usage: `embed_build_output_as!(StructName);`
#[macro_export]
macro_rules! embed_build_output_as {
    ($struct_name:ident) => {
        #[derive(Embed)]
        #[folder = "$OUT_DIR"]
        pub struct $struct_name;
    };
}
/// One-liner for structs with derived Embed
///
/// Usage: `embed_as!(StructName from "path" only "*.rs" except "secret.rs");` where `only...` and `except...` parts are optional and accept 1+ comma-separated arg
#[macro_export]
macro_rules! embed_as {
    ($struct_name:ident from $path:literal $(only $($inc:literal),+)? $(except $($exc:literal),+)?) => {
        #[derive(Embed)]
        #[folder = $path]
        $( $( #[include = $inc] )+ )?
        $( $( #[exclude = $exc] )+ )?
        pub struct $struct_name;
    };
}

/// This enum exists for optimization purposes, to avoid boxing the iterator in
/// some cases. Do not try and match on it, as different variants will exist
/// depending on the compilation context.
#[doc(hidden)]
pub enum __Filenames {
    /// Release builds use a named iterator type, which can be stack-allocated.
    #[cfg(any(not(debug_assertions), target_arch = "wasm32"))]
    Embedded(std::slice::Iter<'static, &'static str>),

    /// The debug iterator type is currently unnameable and still needs to be
    /// boxed.
    #[cfg(all(debug_assertions, not(target_arch = "wasm32")))]
    Dynamic(Box<dyn Iterator<Item = Cow<'static, str>>>),
}

impl Iterator for __Filenames {
    type Item = Cow<'static, str>;
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            #[cfg(any(not(debug_assertions), target_arch = "wasm32"))]
            __Filenames::Embedded(names) => names.next().map(|x| Cow::from(*x)),

            #[cfg(all(debug_assertions, not(target_arch = "wasm32")))]
            __Filenames::Dynamic(boxed) => boxed.next(),
        }
    }
}
