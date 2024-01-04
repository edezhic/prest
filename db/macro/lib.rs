#![allow(unused_imports)]
use ::proc_macro::TokenStream;
use ::proc_macro2::{Span, TokenStream as TokenStream2};
use ::quote::{quote, quote_spanned, ToTokens};
use ::syn::{
    parse::{Parse, ParseStream, Parser},
    punctuated::Punctuated,
    spanned::Spanned,
    Result, *,
};

#[proc_macro_derive(Table)]
pub fn table_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as _);
    TokenStream::from(impl_table(ast))
}

fn impl_table(ast: DeriveInput) -> TokenStream2 {
    let name = ast.ident;
    let table_name = name.to_string() + "s";
    let fields = match ast.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(it),
            ..
        }) => it,
        _ => panic!("Expected a `struct` with named fields"),
    };

    let columns: Vec<Column> = fields
        .named
        .into_iter()
        .enumerate()
        .map(decompose)
        .collect();

    let add_columns = columns.iter().map(
        |Column {
             name_str,
             column_type,
             key,
             optional,
             ..
         }| {
            let key = if *key { " PRIMARY KEY" } else { "" };
            let optional = if *optional { "" } else { " NOT NULL" };
            let col = format!("{name_str} {column_type}{key}{optional}");
            quote! { .add_column(#col) }
        },
    );

    let from_row_extractions = columns.iter().rev().map(
        |Column {
             name_ident,
             dbvalue_variant,
             from_row_transform,
             optional,
             list,
             ..
         }| {
            let inner = ident("inner");
            let transform = match from_row_transform {
                FromRowTransform::UuidFromU128 => quote!(let #inner = prest::Uuid::from_u128(#inner);),
                FromRowTransform::None => quote!(),
            };
            if *list {
                quote!(
                    let #name_ident = if let Some(prest::DbValue::List(list)) = row.pop() {
                        list.iter().map(|v| {
                            match v {
                                prest::DbValue::#dbvalue_variant(#inner) => {
                                    let #inner = #inner.clone();
                                    #transform
                                    #inner
                                }
                                _ => panic!("unexpected vec item type"),
                            }
                        }).collect()
                    } else {
                        panic!("unexpected from row value");
                    };
                )
            } else if *optional {
                quote! {
                    let #name_ident = if let Some(prest::DbValue::#dbvalue_variant(#inner)) = row.pop() {
                        #transform
                        Some(#inner)
                    } else {
                        None
                    };
                }
            } else {
                quote! {
                    let #name_ident = if let Some(prest::DbValue::#dbvalue_variant(#inner)) = row.pop() {
                        #transform
                        #inner
                    } else {
                        panic!("unexpected from row value");
                    };
                }
            }
        },
    );

    let from_fields_idents = columns.iter().map(|col| col.name_ident.clone());
    let into_fields_idents = from_fields_idents.clone();

    let fields_idents_into_row_items = columns.iter().rev().map(
        |Column {
             name_str,
             name_ident,
             optional,
             list,
             stringy_in_sql,
             ..
         }| {
            let into_row_fmt = if *list {
                format!("'{{{name_str}:?}}'")
            } else if *stringy_in_sql {
                format!("'{{{name_str}}}'")
            } else {
                format!("{{{name_str}}}")
            };

            if *optional {
                quote! {
                    let #name_ident = if let Some(#name_ident) = #name_ident {
                        format!(#into_row_fmt)
                    } else {
                        "NULL".to_owned()
                    };
                }
            } else {
                quote! {
                    let #name_ident = format!(#into_row_fmt);
                }
            }
        },
    );

    let mut into_row_format = String::new();
    let mut iter = columns.iter().peekable();
    while let Some(Column { name_str, .. }) = iter.next() {
        into_row_format += &format!("{{{name_str}}}");
        if iter.peek().is_some() {
            into_row_format += ", ";
        }
    }
    let into_row_format: Expr = parse_quote!(#into_row_format);

    let mut iter = columns.iter();
    iter.next();
    let set_values = iter.map(set_value);

    let update_fns = columns.iter().map(|col| {
            let Column {
                name_str,
                name_ident,
                field_type,
                ..
            } = col;
            let fn_name = ident(&format!("update_{name_str}"));
            let set_value = set_value(col);
            quote! {
                pub fn #fn_name(&mut self, #name_ident: #field_type) -> prest::Result<&mut Self> {
                    self.#name_ident = #name_ident;
                    Self::update_by_key(self.get_key())
                        #set_value
                        .exec()?;
                    Ok(self)
                }
            }
        },
    );

    let Column {
        name_ident: key_name_ident,
        name_str: key_name_str,
        field_type: key_type_token,
        stringy_in_sql: key_stringy,
        ..
    } = &columns[0];

    quote! {
        impl prest::Table for #name {
            const TABLE_NAME: &'static str = #table_name;
            const KEY: &'static str = #key_name_str;
            const STRINGY_KEY: bool = #key_stringy;
            type Key = #key_type_token;

            fn get_key(&self) -> &Self::Key {
                &self.#key_name_ident
            }

            fn init_table() {
                prest::table(Self::TABLE_NAME)
                    .create_table_if_not_exists()
                    #(#add_columns)*
                    .exec()
                    .unwrap();
            }

            fn from_row(mut row: ::std::vec::Vec<prest::DbValue>) -> Self {
                #(#from_row_extractions)*
                Self { #(#from_fields_idents ,)* }
            }

            fn into_row(&self) -> ::std::string::String {
                let Self { #(#into_fields_idents ,)* } = self;
                #(#fields_idents_into_row_items)*
                format!(#into_row_format)
            }
            fn save(&self) -> prest::Result<&Self> {
                if Self::find_by_key(self.get_key()).is_some() {
                    Self::update_by_key(self.get_key())
                        #(#set_values)*
                        .exec()?;
                } else {
                    self.insert_self()?;
                }
                Ok(&self)
            }
        }

        impl #name {
            #(#update_fns)*
        }
    }
}

#[derive(Debug)]
enum FromRowTransform {
    UuidFromU128,
    None,
}

#[derive(Debug)]
struct Column {
    name_ident: Ident,
    name_str: String,
    field_type: Type,
    column_type: String,
    dbvalue_variant: Ident,
    from_row_transform: FromRowTransform,
    key: bool,
    stringy_in_sql: bool,
    optional: bool,
    list: bool,
}

fn decompose((index, field): (usize, Field)) -> Column {
    let key = index == 0;
    let name_ident = field.ident.expect("only named structs");
    let name_str = name_ident.to_string();
    let field_type = field.ty;
    let type_str = field_type.to_token_stream().to_string();
    let type_str = type_str.as_str();

    let (type_str, optional, list) =
        if type_str.starts_with("Option < ") && type_str.ends_with(" >") {
            let type_str = type_str
                .trim_start_matches("Option < ")
                .trim_end_matches(" >");
            (type_str, true, false)
        } else if type_str.starts_with("Vec < ") && type_str.ends_with(" >") {
            let type_str = type_str.trim_start_matches("Vec < ").trim_end_matches(" >");
            (type_str, false, true)
        } else {
            (type_str, false, false)
        };

    match type_str {
        "Uuid" | "String" | "bool" | "u64" | "f64" | "u8" => (),
        _ => panic!("Type {type_str} is not supported in the Table derive macro"),
    };

    match (key, optional, list) {
        (true, true, _) => panic!("Key (first attribute) cannot be optional"),
        (true, _, true) => panic!("Key (first attribute) cannot be list"),
        _ => (),
    };

    let column_type = if list {
        "LIST"
    } else {
        match type_str {
            "Uuid" => "UUID",
            "String" => "TEXT",
            "bool" => "BOOLEAN",
            "u64" => "UINT64",
            "u8" => "UINT8",
            "f64" => "FLOAT",
            _ => panic!("Unsupported gluesql type: {type_str}"),
        }
    }
    .to_owned();

    let dbvalue_variant = match type_str {
        "Uuid" => "Uuid",
        "String" => "Str",
        "bool" => "Bool",
        "u64" => "U64",
        "u8" => "U8",
        "f64" => "F64",
        _ => panic!("Unsupported gluesql value: {type_str}"),
    };
    let dbvalue_variant = ident(dbvalue_variant);

    let from_row_transform = match type_str {
        "Uuid" => FromRowTransform::UuidFromU128,
        _ => FromRowTransform::None,
    };

    let stringy_in_sql = type_str == "Uuid" || type_str == "String" || list;

    Column {
        column_type,
        dbvalue_variant,
        from_row_transform,
        name_ident,
        name_str,
        field_type,
        key,
        stringy_in_sql,
        optional,
        list,
    }
}

fn set_value(column: &Column) -> TokenStream2 {
    let Column {
        name_ident,
        name_str,
        stringy_in_sql,
        list,
        optional,
        ..
    } = column;
    let key: Expr = parse_quote!(#name_str);
    let fmt_str = if *list { "'{:?}'" } else { "'{}'" };
    let value = match (stringy_in_sql, optional) {
        (true, true) => {
            quote!(if let Some(v) = &self.#name_ident { format!(#fmt_str, v.clone()) } else { "NULL".to_owned() })
        }
        (true, false) => quote!(format!(#fmt_str, self.#name_ident.clone())),
        (false, true) => {
            quote!(if let Some(v) = &self.#name_ident { v.clone() } else { "NULL".to_owned() })
        }
        (false, false) => quote!(self.#name_ident.clone()),
    };
    quote! { .set(#key, #value) }
}

fn ident(name: &str) -> Ident {
    Ident::new(name, Span::call_site())
}
