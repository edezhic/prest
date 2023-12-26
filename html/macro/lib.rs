#![doc(html_root_url = "https://docs.rs/maud_macros/0.25.0")]
// TokenStream values are reference counted, and the mental overhead of tracking
// lifetimes outweighs the marginal gains from explicit borrowing
#![allow(clippy::needless_pass_by_value)]

extern crate proc_macro;

mod ast;
mod escape;
mod generate;
mod parse;

use proc_macro2::{Ident, Span, TokenStream, TokenTree};
use proc_macro_error::proc_macro_error;
use quote::quote;

/// Compose HTML templates right inside of Rust code with ease.
///
/// As of now it is identical to the original so check out the [maud book](https://maud.lambda.xyz/) for details
#[proc_macro]
#[proc_macro_error]
pub fn html(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    expand(input.into()).into()
}

fn expand(input: TokenStream) -> TokenStream {
    let output_ident = TokenTree::Ident(Ident::new("__maud_output", Span::mixed_site()));
    // Heuristic: the size of the resulting markup tends to correlate with the
    // code size of the template itself
    let size_hint = input.to_string().len();
    let markups = parse::parse(input);
    let stmts = generate::generate(markups, output_ident.clone());
    quote!({
        extern crate alloc;
        //extern crate prest;
        let mut #output_ident = alloc::string::String::with_capacity(#size_hint);
        #stmts
        prest::PreEscaped(#output_ident)
    })
}
