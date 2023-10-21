use crate::*;

#[macro_export]
macro_rules! embed {
    ($struct_name:ident) => {
        #[derive(Embed)]
        #[folder = "$OUT_DIR/dist"]
        struct $struct_name;
    };
    ($struct_name:ident, $path:literal) => {
        #[derive(Embed)]
        #[folder = $path]
        struct $struct_name;
    };
}

pub trait Embed where Self: 'static {
    fn get(file_path: &str) -> Option<EmbeddedFile>;
    fn iter() -> Filenames;
    fn routes() -> Router {
        let mut router = Router::new().route("/dist/*any", get(Self::handle));
        if Self::get("sw.js").is_some() {
            router = router.route("/sw.js", get(|headers: HeaderMap| async {
                Self::handle(Path("sw.js".to_owned()), headers).await
            }))
        }
        router
    }
    fn handle(
        Path(path): Path<String>,
        headers: HeaderMap,
    ) -> impl std::future::Future<Output = Response> + Send {
        async move {
            let Some(asset) = Self::get(&path) else {
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
    }
}

/// This enum exists for optimization purposes, to avoid boxing the iterator in
/// some cases. Do not try and match on it, as different variants will exist
/// depending on the compilation context.
pub enum Filenames {
    /// Release builds use a named iterator type, which can be stack-allocated.
    #[cfg(any(not(debug_assertions), feature = "debug-embed"))]
    Embedded(std::slice::Iter<'static, &'static str>),

    /// The debug iterator type is currently unnameable and still needs to be
    /// boxed.
    #[cfg(all(debug_assertions, not(feature = "debug-embed")))]
    Dynamic(Box<dyn Iterator<Item = std::borrow::Cow<'static, str>>>),
}

impl Iterator for Filenames {
    type Item = std::borrow::Cow<'static, str>;
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            #[cfg(any(not(debug_assertions), feature = "debug-embed"))]
            Filenames::Embedded(names) => names.next().map(|x| std::borrow::Cow::from(*x)),

            #[cfg(all(debug_assertions, not(feature = "debug-embed")))]
            Filenames::Dynamic(boxed) => boxed.next(),
        }
    }
}
