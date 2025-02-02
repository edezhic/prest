#![allow(dead_code)]
// fork of the tokio main macro

use proc_macro2::{Span, TokenStream, TokenTree};
use quote::{quote, quote_spanned, ToTokens};
use syn::parse::{Parse, ParseStream, Parser};
use syn::{braced, Attribute, Ident, Path, Signature, Visibility};

// syn::AttributeArgs does not implement syn::Parse
type AttributeArgs = syn::punctuated::Punctuated<syn::Meta, syn::Token![,]>;

#[derive(Debug, Default)]
struct Config {
    log_filters: Vec<(String, String)>,
    manifest: Manifest,
    tables: Vec<Ident>,
}

#[proc_macro_attribute]
pub fn init(
    args: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    init_pc2(args.into(), item.into()).into()
}

pub(crate) fn init_pc2(args: TokenStream, item: TokenStream) -> TokenStream {
    // If any of the steps for this macro fail, we still want to expand to an item that is as close
    // to the expected output as possible. This helps out IDEs such that completions and other
    // related features keep working.
    let input: ItemFn = match syn::parse2(item.clone()) {
        Ok(it) => it,
        Err(e) => return token_stream_with_error(item, e),
    };

    if input.sig.ident != "main" || !input.sig.inputs.is_empty() {
        let msg = "init macro should be only used on the main function without arguments";
        let e = syn::Error::new_spanned(&input.sig.ident, msg);
        return token_stream_with_error(expand(input, Default::default()), e);
    }

    let config = AttributeArgs::parse_terminated
        .parse2(args)
        .and_then(|args| build_config(&input, args));

    match config {
        Ok(config) => expand(input, config),
        Err(e) => token_stream_with_error(expand(input, Default::default()), e),
    }
}

fn build_config(input: &ItemFn, args: AttributeArgs) -> Result<Config, syn::Error> {
    if input.sig.asyncness.is_none() {
        let msg = "the `async` keyword is missing from the function declaration";
        return Err(syn::Error::new_spanned(input.sig.fn_token, msg));
    }

    // parse all source files in search for Storage derivations

    let mut log_filters = vec![];

    for arg in args {
        match arg {
            syn::Meta::NameValue(namevalue) => {
                let ident = namevalue
                    .path
                    .get_ident()
                    .ok_or_else(|| {
                        syn::Error::new_spanned(&namevalue, "Must have specified ident")
                    })?
                    .to_string()
                    .to_lowercase();
                match ident.as_str() {
                    "log_filters" => {
                        let args = match &namevalue.value {
                            syn::Expr::Array(arr) => arr,
                            expr => {
                                return Err(syn::Error::new_spanned(
                                    expr,
                                    "Must be an array of tuples",
                                ))
                            }
                        };
                        for arg in args.elems.iter() {
                            let tuple = match arg {
                                syn::Expr::Tuple(tuple) => tuple,
                                arg => return Err(syn::Error::new_spanned(arg, "Must be a tuple")),
                            };
                            let mut tuple = tuple.elems.iter();
                            let filter = match tuple.next() {
                                Some(syn::Expr::Lit(syn::ExprLit { lit, .. })) => lit,
                                Some(v) => {
                                    return Err(syn::Error::new_spanned(v, "Must be a literal"))
                                }
                                None => {
                                    return Err(syn::Error::new_spanned(arg, "Missing log value"))
                                }
                            };
                            let filter = parse_string(
                                filter.clone(),
                                syn::spanned::Spanned::span(filter),
                                "log",
                            )?;

                            let level = match tuple.next() {
                                Some(syn::Expr::Lit(syn::ExprLit { lit, .. })) => lit,
                                Some(v) => {
                                    return Err(syn::Error::new_spanned(v, "Must be a literal"))
                                }
                                None => {
                                    return Err(syn::Error::new_spanned(arg, "Missing log value"))
                                }
                            };
                            let level = parse_string(
                                level.clone(),
                                syn::spanned::Spanned::span(level),
                                "filter",
                            )?;

                            if tuple.next().is_some() {
                                return Err(syn::Error::new_spanned(
                                    arg,
                                    "Unexpected 3rd tuple item",
                                ));
                            }

                            log_filters.push((filter, level));
                        }
                    }
                    name => {
                        let msg = format!(
                            "Unknown attribute {name} is specified; expected `log_filters`",
                        );
                        return Err(syn::Error::new_spanned(namevalue, msg));
                    }
                }
            }
            other => {
                return Err(syn::Error::new_spanned(
                    other,
                    "Unknown attribute inside the macro",
                ));
            }
        }
    }

    let manifest = get_manifest();

    use std::{fs, io};
    fn find_tables(dir: fs::ReadDir, tables: &mut Vec<String>) -> io::Result<()> {
        for file in dir {
            let file = file?;
            if file.file_name().to_string_lossy() == "target" {
                continue;
            }
            match file.metadata()? {
                data if data.is_dir() => find_tables(fs::read_dir(file.path())?, tables)?,
                _ => {
                    let content = std::fs::read_to_string(file.path())?;
                    let mut expecting = false;
                    for line in content.lines() {
                        if expecting
                            && (line.starts_with("pub") || line.starts_with("struct"))
                            && line.contains("struct")
                        {
                            let struct_to_end = line.split("struct ").nth(1).unwrap();
                            let struct_name = struct_to_end.split(" ").nth(0).unwrap();
                            tables.push(struct_name.to_owned());
                            expecting = false;
                        }
                        if line.starts_with("#[derive(") && line.contains("Storage") {
                            expecting = true;
                        }
                    }
                }
            };
        }
        Ok(())
    }

    let mut tables = vec![];
    find_tables(fs::read_dir(&manifest.manifest_dir).unwrap(), &mut tables)
        .expect("Tables search must succeed");
    let tables = tables.into_iter().map(|t| ident(&t)).collect();

    Ok(Config {
        log_filters,
        manifest,
        tables,
    })
}

fn expand(mut input: ItemFn, config: Config) -> TokenStream {
    input.sig.asyncness = None;

    // If type mismatch occurs, the current rustc points to the last statement.
    // let (last_stmt_start_span, last_stmt_end_span) = {
    let last_stmt_start_span = {
        let mut last_stmt = input.stmts.last().cloned().unwrap_or_default().into_iter();

        // `Span` on stable Rust has a limitation that only points to the first
        // token, not the whole tokens. We can work around this limitation by
        // using the first/last span of the tokens like
        // `syn::Error::new_spanned` does.
        let start = last_stmt.next().map_or_else(Span::call_site, |t| t.span());
        // let end = last_stmt.last().map_or(start, |t| t.span());
        // (start, end)
        start
    };

    let body_ident = quote! { body };

    let rt = quote_spanned! {last_stmt_start_span=>
        #[allow(clippy::expect_used, clippy::diverging_sub_expression, clippy::needless_return)]
        return prest::RT.block_on(#body_ident);
    };

    let Manifest {
        name,
        version,
        manifest_dir,
        persistent,
        domain,
    } = config.manifest;

    let domain = match domain {
        Some(v) => quote!( Some(#v) ),
        None => quote!(None),
    };
    let init_config = quote!(
        prest::APP_CONFIG._init(#manifest_dir, #name, #version, #persistent, #domain)
    );

    let filters = config.log_filters.into_iter().map(|(filter, level)| {
        let level = ident(&level.to_ascii_uppercase());
        quote!((#filter, prest::logs::Level::#level))
    });

    let init_tracing = quote!(
        let __________ = std::thread::spawn(|| prest::logs::init_tracing_subscriber(&[ #(#filters ,)* ]))
    );

    let register_tables = config
        .tables
        .into_iter()
        .map(|table| quote!( prest::DB._register_schema(#table::schema()); ));

    let body = input.body();
    let body = quote! {
        let _start = std::time::Instant::now();
        #init_config;
        #init_tracing;
        prest::Lazy::force(&prest::RT);
        let _ = prest::dotenv();
        std::thread::spawn(|| {
            prest::Lazy::force(&prest::SYSTEM_INFO);
        });
        std::thread::spawn(|| {
            prest::Lazy::force(&prest::DB);
            #(#register_tables)*
        });
        prest::RT.block_on(async {
            prest::DB.migrate().await.expect("DB migration should be successful");
        });
        prest::info!(target: "prest", "Initialized {} v{} in {}ms", APP_CONFIG.name, &APP_CONFIG.version, _start.elapsed().as_millis());
        prest::RT.set_ready();
        let body = async #body;
    };

    input.into_tokens(body, rt)
}

fn parse_int(int: syn::Lit, span: Span, field: &str) -> Result<usize, syn::Error> {
    match int {
        syn::Lit::Int(lit) => match lit.base10_parse::<usize>() {
            Ok(value) => Ok(value),
            Err(e) => Err(syn::Error::new(
                span,
                format!("Failed to parse value of `{field}` as integer: {e}"),
            )),
        },
        _ => Err(syn::Error::new(
            span,
            format!("Failed to parse value of `{field}` as integer."),
        )),
    }
}

fn parse_string(int: syn::Lit, span: Span, field: &str) -> Result<String, syn::Error> {
    match int {
        syn::Lit::Str(s) => Ok(s.value()),
        syn::Lit::Verbatim(s) => Ok(s.to_string()),
        _ => Err(syn::Error::new(
            span,
            format!("Failed to parse value of `{field}` as string."),
        )),
    }
}

fn parse_path(lit: syn::Lit, span: Span, field: &str) -> Result<Path, syn::Error> {
    match lit {
        syn::Lit::Str(s) => {
            let err = syn::Error::new(
                span,
                format!(
                    "Failed to parse value of `{}` as path: \"{}\"",
                    field,
                    s.value()
                ),
            );
            s.parse::<syn::Path>().map_err(|_| err.clone())
        }
        _ => Err(syn::Error::new(
            span,
            format!("Failed to parse value of `{field}` as path."),
        )),
    }
}

fn parse_bool(bool: syn::Lit, span: Span, field: &str) -> Result<bool, syn::Error> {
    match bool {
        syn::Lit::Bool(b) => Ok(b.value),
        _ => Err(syn::Error::new(
            span,
            format!("Failed to parse value of `{field}` as bool."),
        )),
    }
}

fn token_stream_with_error(mut tokens: TokenStream, error: syn::Error) -> TokenStream {
    tokens.extend(error.into_compile_error());
    tokens
}

#[derive(Debug, Default)]
struct Manifest {
    name: String,
    version: String,
    manifest_dir: String,
    persistent: bool,
    domain: Option<String>,
}

fn get_manifest() -> Manifest {
    let name = std::env::var("CARGO_PKG_NAME").unwrap();
    let version = std::env::var("CARGO_PKG_VERSION").unwrap();

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let manifest = std::fs::read_to_string(format!("{manifest_dir}/Cargo.toml")).unwrap();
    let parsed = manifest.parse::<toml::Table>().unwrap();
    let metadata = parsed.get("package").map(|t| t.get("metadata")).flatten();

    let persistent = metadata
        .map(|cfgs| cfgs.get("persistent").map(|v| v.as_bool()))
        .flatten()
        .flatten()
        .unwrap_or(true);

    let domain = metadata
        .map(|cfgs| {
            cfgs.get("domain")
                .map(|v| v.as_str().map(ToString::to_string))
        })
        .flatten()
        .flatten();

    Manifest {
        name,
        version,
        manifest_dir,
        persistent,
        domain,
    }
}

struct ItemFn {
    outer_attrs: Vec<Attribute>,
    vis: Visibility,
    sig: Signature,
    brace_token: syn::token::Brace,
    inner_attrs: Vec<Attribute>,
    stmts: Vec<proc_macro2::TokenStream>,
}

impl ItemFn {
    /// Get the body of the function item in a manner so that it can be
    /// conveniently used with the `quote!` macro.
    fn body(&self) -> Body<'_> {
        Body {
            brace_token: self.brace_token,
            stmts: &self.stmts,
        }
    }

    /// Convert our local function item into a token stream.
    fn into_tokens(
        self,
        body: proc_macro2::TokenStream,
        last_block: proc_macro2::TokenStream,
    ) -> TokenStream {
        let mut tokens = proc_macro2::TokenStream::new();
        // Outer attributes are simply streamed as-is.
        for attr in self.outer_attrs {
            attr.to_tokens(&mut tokens);
        }

        // Inner attributes require extra care, since they're not supported on
        // blocks (which is what we're expanded into) we instead lift them
        // outside of the function. This matches the behavior of `syn`.
        for mut attr in self.inner_attrs {
            attr.style = syn::AttrStyle::Outer;
            attr.to_tokens(&mut tokens);
        }

        self.vis.to_tokens(&mut tokens);
        self.sig.to_tokens(&mut tokens);

        self.brace_token.surround(&mut tokens, |tokens| {
            body.to_tokens(tokens);
            last_block.to_tokens(tokens);
        });

        tokens
    }
}

impl Parse for ItemFn {
    #[inline]
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        // This parse implementation has been largely lifted from `syn`, with
        // the exception of:
        // * We don't have access to the plumbing necessary to parse inner
        //   attributes in-place.
        // * We do our own statements parsing to avoid recursively parsing
        //   entire statements and only look for the parts we're interested in.

        let outer_attrs = input.call(Attribute::parse_outer)?;
        let vis: Visibility = input.parse()?;
        let sig: Signature = input.parse()?;

        let content;
        let brace_token = braced!(content in input);
        let inner_attrs = Attribute::parse_inner(&content)?;

        let mut buf = proc_macro2::TokenStream::new();
        let mut stmts = Vec::new();

        while !content.is_empty() {
            if let Some(semi) = content.parse::<Option<syn::Token![;]>>()? {
                semi.to_tokens(&mut buf);
                stmts.push(buf);
                buf = proc_macro2::TokenStream::new();
                continue;
            }

            // Parse a single token tree and extend our current buffer with it.
            // This avoids parsing the entire content of the sub-tree.
            buf.extend([content.parse::<TokenTree>()?]);
        }

        if !buf.is_empty() {
            stmts.push(buf);
        }

        Ok(Self {
            outer_attrs,
            vis,
            sig,
            brace_token,
            inner_attrs,
            stmts,
        })
    }
}

struct Body<'a> {
    brace_token: syn::token::Brace,
    // Statements, with terminating `;`.
    stmts: &'a [TokenStream],
}

impl ToTokens for Body<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.brace_token.surround(tokens, |tokens| {
            for stmt in self.stmts {
                stmt.to_tokens(tokens);
            }
        });
    }
}

fn ident(name: &str) -> Ident {
    Ident::new(name, Span::call_site())
}
