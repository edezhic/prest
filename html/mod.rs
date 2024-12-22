//! Fork of [maud](https://github.com/lambda-fairy/maud) adjusted to be used in prest. Thanks to Chris Wong <lambda.fairy@gmail.com> and other contributors!
//! Changes in the API:
//! - added utils for easier integration with HTMX
//! - added utils for common HTML elements: head, scripts
//! - native axum support and removed support for other frameworks
//! - support for tailwind classes with inline generated styles

pub use prest_html_macro::html;

mod common;
mod escape;
pub use common::*;
mod htmx;
pub use htmx::*;

use crate::*;
use core::fmt::{self, Arguments, Display, Write};
use std::{borrow::Cow, boxed::Box, string::String, sync::Arc};

/// Wrapper of a buffer that escapes HTML chars when it is written using [`fmt::Write`]
pub struct Escaper<'a>(&'a mut String);

impl<'a> Escaper<'a> {
    /// Creates an `Escaper` from a `String`.
    pub fn new(buffer: &'a mut String) -> Escaper<'a> {
        Escaper(buffer)
    }
}

impl<'a> fmt::Write for Escaper<'a> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        escape::escape_to_string(s, self.0);
        Ok(())
    }
}

/// Trait that defines how something is rendered into HTML
pub trait Render {
    /// Renders `self` as a block of `Markup`.
    fn render(&self) -> Markup {
        let mut buffer = String::new();
        self.render_to(&mut buffer);
        PreEscaped(buffer)
    }

    /// Appends a representation of `self` to the given buffer.
    ///
    /// Its default implementation just calls `.render()`, but you may
    /// override it with something more efficient.
    ///
    /// Note that no further escaping is performed on data written to
    /// the buffer. If you override this method, you must make sure that
    /// any data written is properly escaped, whether by hand or using
    /// the [`Escaper`](struct.Escaper.html) wrapper struct.
    fn render_to(&self, buffer: &mut String) {
        buffer.push_str(&self.render().into_string());
    }
}

impl Render for str {
    fn render_to(&self, w: &mut String) {
        escape::escape_to_string(self, w);
    }
}

impl Render for String {
    fn render_to(&self, w: &mut String) {
        str::render_to(self, w);
    }
}

impl<'a> Render for Cow<'a, str> {
    fn render_to(&self, w: &mut String) {
        str::render_to(self, w);
    }
}

impl<'a> Render for Arguments<'a> {
    fn render_to(&self, w: &mut String) {
        let _ = Escaper::new(w).write_fmt(*self);
    }
}

impl<'a, T: Render + ?Sized> Render for &'a T {
    fn render_to(&self, w: &mut String) {
        T::render_to(self, w);
    }
}

impl<'a, T: Render + ?Sized> Render for &'a mut T {
    fn render_to(&self, w: &mut String) {
        T::render_to(self, w);
    }
}

impl<T: Render + ?Sized> Render for Box<T> {
    fn render_to(&self, w: &mut String) {
        T::render_to(self, w);
    }
}

impl<T: Render + ?Sized> Render for Arc<T> {
    fn render_to(&self, w: &mut String) {
        T::render_to(self, w);
    }
}

impl<T: Render> Render for Option<T> {
    fn render_to(&self, w: &mut String) {
        if let Some(v) = self {
            T::render_to(v, w);
        }
    }
}

impl<T: Render> Render for Vec<T> {
    fn render_to(&self, w: &mut String) {
        for v in self.iter() {
            T::render_to(v, w);
        }
    }
}

macro_rules! impl_render_with_display {
    ($($ty:ty)*) => {
        $(
            impl Render for $ty {
                fn render_to(&self, w: &mut String) {
                    format_args!("{self}").render_to(w);
                }
            }
        )*
    };
}

impl_render_with_display! {
    char f32 f64
}

macro_rules! impl_render_with_itoa {
    ($($ty:ty)*) => {
        $(
            impl Render for $ty {
                fn render_to(&self, w: &mut String) {
                    w.push_str(itoa::Buffer::new().format(*self));
                }
            }
        )*
    };
}

impl_render_with_itoa! {
    i8 i16 i32 i64 i128 isize
    u8 u16 u32 u64 u128 usize
}

/// Utility that renders any value that implements [`Display`]
#[doc(hidden)]
pub fn display(value: impl Display) -> impl Render {
    struct DisplayWrapper<T>(T);

    impl<T: Display> Render for DisplayWrapper<T> {
        fn render_to(&self, w: &mut String) {
            format_args!("{0}", self.0).render_to(w);
        }
    }

    DisplayWrapper(value)
}

/// A wrapper that renders the inner value without escaping.
#[derive(Debug, Clone, Copy)]
pub struct PreEscaped<T>(pub T);

impl<T: AsRef<str>> Render for PreEscaped<T> {
    fn render_to(&self, w: &mut String) {
        w.push_str(self.0.as_ref());
    }
}

/// A block of markup is a string that does not need to be escaped.
///
/// The `html!` macro expands to an expression of this type.
pub type Markup = PreEscaped<String>;

impl<T: Into<String>> PreEscaped<T> {
    /// Converts the inner value to a string.
    pub fn into_string(self) -> String {
        self.0.into()
    }
}

impl<T: Into<String>> From<PreEscaped<T>> for String {
    fn from(value: PreEscaped<T>) -> String {
        value.into_string()
    }
}

impl<T: Default> Default for PreEscaped<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

use http::{header, HeaderMap, HeaderValue, Response};

impl IntoResponse for PreEscaped<String> {
    fn into_response(self) -> Response<Body> {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("text/html; charset=utf-8"),
        );
        (headers, self.0).into_response()
    }
}

#[doc(hidden)]
pub mod macro_private {
    use crate::{display, Render};
    use core::fmt::Display;

    #[doc(hidden)]
    #[macro_export]
    macro_rules! render_to {
        ($x:expr, $buffer:expr) => {{
            use $crate::macro_private::*;
            match ChooseRenderOrDisplay($x) {
                x => (&&x).implements_render_or_display().render_to(x.0, $buffer),
            }
        }};
    }

    pub use render_to;

    pub struct ChooseRenderOrDisplay<T>(pub T);

    pub struct ViaRenderTag;
    pub struct ViaDisplayTag;

    pub trait ViaRender {
        fn implements_render_or_display(&self) -> ViaRenderTag {
            ViaRenderTag
        }
    }
    pub trait ViaDisplay {
        fn implements_render_or_display(&self) -> ViaDisplayTag {
            ViaDisplayTag
        }
    }

    impl<T: Render> ViaRender for &ChooseRenderOrDisplay<T> {}
    impl<T: Display> ViaDisplay for ChooseRenderOrDisplay<T> {}

    impl ViaRenderTag {
        pub fn render_to<T: Render + ?Sized>(self, value: &T, buffer: &mut String) {
            value.render_to(buffer);
        }
    }

    impl ViaDisplayTag {
        pub fn render_to<T: Display + ?Sized>(self, value: &T, buffer: &mut String) {
            display(value).render_to(buffer);
        }
    }
}
