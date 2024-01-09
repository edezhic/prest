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

#[proc_macro_derive(Table, attributes(key_column, unique_column))]
pub fn table_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as _);
    TokenStream::from(impl_table(ast))
}

fn impl_table(ast: DeriveInput) -> TokenStream2 {
    let struct_name = ast.ident;
    let table_name_str = struct_name.to_string() + "s";
    let fields = match ast.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(it),
            ..
        }) => it,
        _ => panic!("Expected a `struct` with named fields"),
    };

    let mut columns: Vec<Column> = fields
        .named
        .into_iter()
        .map(decompose)
        .collect();

    match columns.iter().filter(|c| c.key).count() {
        0 => {
            columns[0].key = true;
            columns[0].unique = true;
        }
        1 => {},
        _ => panic!("Table macro doesn't support more than one key at the moment")
    };

    let key_column = columns.iter().find(|c| c.key).unwrap();

    let add_columns = columns.iter().map(
        |Column {
             name_str,
             column_type,
             key,
             optional,
             unique,
             ..
         }| {
            let unique = if !*key && *unique { " UNIQUE" } else { "" };
            let key = if *key { " PRIMARY KEY" } else { "" };
            let optional = if *optional { "" } else { " NOT NULL" };
            let col = format!("{name_str} {column_type}{key}{unique}{optional}");
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

    let fields_idents_into_row_items = columns.iter().rev().map(|col| {
        let Column {
            name_ident,
            optional,
            ..
        } = col;

        let into_row_fmt = row_value_fmt(col);

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
    });

    let mut into_row_name_format = String::new();
    let mut iter = columns.iter().peekable();
    while let Some(Column { name_str, .. }) = iter.next() {
        into_row_name_format += &format!("{{{name_str}}}");
        if iter.peek().is_some() {
            into_row_name_format += ", ";
        }
    }
    let into_row_format: Expr = parse_quote!(#into_row_name_format);

    let set_columns = columns.iter().skip(1).map(set_column);

    let find_fns = columns.iter().map(|col| {
        let Column {
            name_str,
            name_ident,
            field_type,
            inner_type,
            optional,
            unique,
            ..
        } = col;
        let values = quote!(Self::select().filter(filter).rows().unwrap().into_iter().map(Self::from_row).collect());

        let find_null_fn = if *optional {
            let fn_name = find_by_null_(col);
            let filter_str = format!("{name_str} = NULL");
            let filter = quote!(format!(#filter_str));
            quote!(
                pub fn #fn_name() -> Vec<Self> {
                    let filter = #filter.to_owned();
                    #values
                }
            )
        } else {
            quote!()
        };

        let fn_name = find_by_(col);
        let fn_value = if *unique { quote!(Option<Self>) } else { quote!(Vec<Self>) };
        let fn_arg = if *optional { inner_type } else { field_type };
        let filter_str = match optional {
            true => format!("{name_str} = {}", row_value_fmt(col)),
            false => format!("{name_str} = {}", row_value_fmt(col))
        };
        let filter = quote!(format!(#filter_str));
        let fn_return = match unique {
            true => quote!(values.pop()),
            false => quote!(values)
        };
        quote! {
            pub fn #fn_name(#name_ident: &#fn_arg) -> #fn_value {
                let filter = #filter.to_owned();
                let mut values: Vec<Self> = #values;
                #fn_return
            }
            #find_null_fn
        }
    });

    let update_fns = columns.iter().map(|col| {
        let Column {
            name_ident,
            field_type,
            ..
        } = col;
        let fn_name = update_(col);
        let set_column = set_column(col);
        quote! {
            pub fn #fn_name(&mut self, #name_ident: #field_type) -> prest::Result<&mut Self> {
                self.#name_ident = #name_ident;
                Self::update_by_key(self.get_key())
                    #set_column
                    .exec()?;
                Ok(self)
            }
        }
    });

    let Column {
        name_ident: key_name_ident,
        name_str: key_name_str,
        field_type: key_type_token,
        stringy_in_sql: key_stringy,
        ..
    } = &key_column;

    let find_by_key = find_by_(&key_column);
    let save_fn = quote!(
        fn save(&self) -> prest::Result<&Self> {
            if Self::#find_by_key(self.get_key()).is_some() {
                Self::update_by_key(self.get_key())
                    #(#set_columns)*
                    .exec()?;
            } else {
                self.insert_self()?;
            }
            Ok(&self)
        }
    );

    quote! {
        impl prest::Table for #struct_name {
            const TABLE_NAME: &'static str = #table_name_str;
            const KEY: &'static str = #key_name_str;
            const STRINGY_KEY: bool = #key_stringy;
            type Key = #key_type_token;

            fn get_key(&self) -> &Self::Key {
                &self.#key_name_ident
            }

            fn migrate() {
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
            
            #save_fn
        }

        impl #struct_name {
            #(#find_fns)*
            #(#update_fns)*
        }
    }
}

enum FromRowTransform {
    UuidFromU128,
    None,
}

struct Column {
    name_ident: Ident,
    name_str: String,
    field_type: Type,
    inner_type: Type,
    column_type: String,
    dbvalue_variant: Ident,
    from_row_transform: FromRowTransform,
    key: bool,
    stringy_in_sql: bool,
    optional: bool,
    list: bool,
    unique: bool,
}

fn decompose(field: Field) -> Column {
    let key = field
        .attrs
        .iter()
        .find(|a| a.path().to_token_stream().to_string() == "key_column")
        .is_some();

    let unique = field
        .attrs
        .iter()
        .find(|a| a.path().to_token_stream().to_string() == "unique_column")
        .is_some()
        || key;

    let name_ident = field.ident.expect("only named structs");
    let name_str = name_ident.to_string();
    let field_type = field.ty;
    let type_str = field_type.to_token_stream().to_string();
    let type_str = type_str.as_str();

    let (inner_type_str, optional, list) =
        if type_str.starts_with("Option < ") && type_str.ends_with(" >") {
            let inner_type_str = type_str
                .trim_start_matches("Option < ")
                .trim_end_matches(" >");
            (inner_type_str, true, false)
        } else if type_str.starts_with("Vec < ") && type_str.ends_with(" >") {
            let inner_type_str = type_str.trim_start_matches("Vec < ").trim_end_matches(" >");
            (inner_type_str, false, true)
        } else {
            (type_str, false, false)
        };

    let inner_type: syn::Type = syn::parse_str(inner_type_str).unwrap();

    match inner_type_str {
        "Uuid" | "String" | "bool" | "u64" | "f64" | "u8" => (),
        _ => panic!("Type {inner_type_str} is not supported in the Table derive macro"),
    };

    match (key, optional, list) {
        (true, true, _) => panic!("Key (first attribute) cannot be optional"),
        (true, _, true) => panic!("Key (first attribute) cannot be list"),
        _ => (),
    };

    let column_type = if list {
        "LIST"
    } else {
        match inner_type_str {
            "Uuid" => "UUID",
            "String" => "TEXT",
            "bool" => "BOOLEAN",
            "u64" => "UINT64",
            "u8" => "UINT8",
            "f64" => "FLOAT",
            _ => panic!("Unsupported gluesql type: {inner_type_str}"),
        }
    }
    .to_owned();

    let dbvalue_variant = match inner_type_str {
        "Uuid" => "Uuid",
        "String" => "Str",
        "bool" => "Bool",
        "u64" => "U64",
        "u8" => "U8",
        "f64" => "F64",
        _ => panic!("Unsupported gluesql value: {inner_type_str}"),
    };
    let dbvalue_variant = ident(dbvalue_variant);

    let from_row_transform = match inner_type_str {
        "Uuid" => FromRowTransform::UuidFromU128,
        _ => FromRowTransform::None,
    };

    let stringy_in_sql = inner_type_str == "Uuid" || inner_type_str == "String" || list;

    Column {
        column_type,
        dbvalue_variant,
        from_row_transform,
        name_ident,
        name_str,
        field_type,
        inner_type,
        key,
        stringy_in_sql,
        optional,
        list,
        unique,
    }
}

fn row_value_fmt(column: &Column) -> String {
    let Column {
        list,
        stringy_in_sql,
        name_str,
        ..
    } = column;
    if *list {
        format!("'{{{name_str}:?}}'")
    } else if *stringy_in_sql {
        format!("'{{{name_str}}}'")
    } else {
        format!("{{{name_str}}}")
    }
}

fn set_column(column: &Column) -> TokenStream2 {
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

fn find_by_(col: &Column) -> Ident {
    ident(&format!("find_by_{}", col.name_str))
}

fn find_by_null_(col: &Column) -> Ident {
    ident(&format!("find_by_null_{}", col.name_str))
}

fn update_(col: &Column) -> Ident {
    ident(&format!("update_{}", col.name_str))
}

fn ident(name: &str) -> Ident {
    Ident::new(name, Span::call_site())
}
