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

/// Generates schema and helper functions to use struct as a table in the embedded database
#[proc_macro_derive(Table, attributes(key_column, unique_column))]
pub fn table_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
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

    let mut columns: Vec<Column> = fields.named.into_iter().map(decompose).collect();

    match columns.iter().filter(|c| c.key).count() {
        0 => {
            columns[0].key = true;
            columns[0].unique = true;
        }
        1 => {}
        _ => panic!("Table macro doesn't support more than one key at the moment"),
    };

    let key_column = columns.iter().find(|c| c.key).unwrap();

    let table_schema = columns.iter().map(
        |Column {
             name_str,
             type_string,
             column_type,
             key,
             unique,
             list,
             optional,
             custom_type,
             ..
         }| {
            quote! {
                ColumnSchema {
                    name: #name_str,
                    rust_type: #type_string,
                    glue_type: #column_type,
                    unique: #unique,
                    key: #key,
                    list: #list,
                    optional: #optional,
                    custom_type: #custom_type,
                }
            }
        },
    );

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
                FromRowTransform::Deserialize => quote!(let #inner = serde_json::from_str(&#inner).unwrap();),
                FromRowTransform::AsF32 => quote!(let #inner = #inner as f32;),
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
    let render_fields_idents = into_fields_idents.clone();

    let fields_idents_into_row_items = columns.iter().rev().map(|col| {
        let Column {
            name_ident,
            optional,
            custom_type,
            ..
        } = col;

        let into_row_fmt = row_value_fmt(col);

        let serialize = match custom_type {
            true => quote!(format!(#into_row_fmt, serde_json::to_string(#name_ident).unwrap())),
            false => quote!(format!(#into_row_fmt)),
        };

        if *optional {
            quote! {
                let #name_ident = if let Some(#name_ident) = #name_ident {
                    #serialize
                } else {
                    "NULL".to_owned()
                };
            }
        } else {
            quote! { let #name_ident = #serialize; }
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
            custom_type,
            ..
        } = col;
        let values = quote!(Self::select()
            .filter(filter)
            .rows()
            .unwrap()
            .into_iter()
            .map(Self::from_row)
            .collect());

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
        let fn_value = if *unique {
            quote!(Option<Self>)
        } else {
            quote!(Vec<Self>)
        };
        let fn_arg = if *optional { inner_type } else { field_type };
        let filter_str = format!("{name_str} = {}", row_value_fmt(col));
        let filter = match custom_type {
            true => quote!(format!(#filter_str, serde_json::to_string(#name_ident).unwrap())),
            false => quote!(format!(#filter_str)),
        };
        let fn_return = match unique {
            true => quote!(values.pop()),
            false => quote!(values),
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

    let range_fns = columns.iter().filter(|col| col.comparable).map(|col| {
        let Column {
            name_str,
            field_type,
            stringy_expr_node,
            stringy_in_sql,
            ..
        } = col;
        let values = quote!(Self::select()
            .filter(filter)
            .rows()
            .unwrap()
            .into_iter()
            .map(Self::from_row)
            .collect());

        let fn_name = find_in_range_(col);
        let filter_str = if *stringy_expr_node || *stringy_in_sql {
            format!("{name_str} >= '{{min}}' AND {name_str} <= '{{max}}'")
        } else {
            format!("{name_str} >= {{min}} AND {name_str} <= {{max}}")
        };
        let filter = quote!(format!(#filter_str));
        quote! {
            pub fn #fn_name(min: &#field_type, max: &#field_type) -> Vec<Self> {
                let filter = #filter.to_owned();
                let mut values: Vec<Self> = #values;
                values
            }
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
                Self::update().filter(Self::key_filter(self.get_key()))
                    #set_column
                    .exec()?;
                Ok(self)
            }
        }
    });

    let check_fns = columns.iter().map(|col| {
        let Column {
            name_ident,
            field_type,
            ..
        } = col;
        let fn_name = check_(col);
        quote! {
            pub fn #fn_name(&self, value: #field_type) -> prest::Result<bool> {
                if let Some(item) = Self::find_by_key(self.get_key()) {
                    Ok(item.#name_ident == value)
                } else {
                    Err(prest::Error::NotFound)
                }
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
                Self::update().filter(Self::key_filter(self.get_key()))
                    #(#set_columns)*
                    .exec()?;
            } else {
                self.insert_self()?;
            }
            Ok(&self)
        }
    );

    let schema_name = ident(&format!("{}Schema", struct_name.to_string()));

    let relative_path = format!("/table/{table_name_str}");
    let full_path = format!("/admin/db{relative_path}");

    let table_schema_clone = table_schema.clone();

    let get_all_as_strings = columns.iter().map(|col| {
        let Column {
            name_ident,
            list,
            optional,
            custom_type,
            ..
        } = col;

        let preprocessing = (*custom_type || *list || *optional)
            .then(|| quote!(let #name_ident = to_json_string(&#name_ident).unwrap();));

        quote! {
            #preprocessing
            let #name_ident: String = #name_ident.to_string();
            row.push(#name_ident);
        }
    });

    let get_all_as_strings_clone = get_all_as_strings.clone();
    let render_fields_idents_clone = render_fields_idents.clone();

    let id_from_str = if key_column.type_string != "String" {
        let key_type = key_column.field_type.clone();
        Some(quote! {
            use std::str::FromStr;
            let id = #key_type::from_str(&id)?;
        })
    } else {
        None
    };

    quote! {
        struct #schema_name;
        #[async_trait]
        impl TableSchemaTrait for #schema_name {
            fn name(&self) -> &'static str {
                #table_name_str
            }
            fn schema(&self) -> ColumnsSchema {
                &[#(#table_schema),*]
            }
            fn relative_path(&self) -> &'static str {
                #relative_path
            }
            fn full_path(&self) -> &'static str {
                #full_path
            }
            async fn get_all(&self) -> Vec<Vec<String>> {
                let mut rows = vec![];
                for item in #struct_name::find_all() {
                    let #struct_name { #(#render_fields_idents ,)* } = item;
                    let mut row = vec![];
                    #(#get_all_as_strings)*
                    rows.push(row);
                }
                rows
            }
            async fn get_row_by_id(&self, id: String) -> std::result::Result<Vec<String>, prest::Error> {
                #id_from_str
                let #struct_name { #(#render_fields_idents_clone ,)* } = #struct_name::find_by_key(&id).unwrap();
                let mut row = vec![];
                #(#get_all_as_strings_clone)*
                Ok(row)
            }
            async fn save(&self, req: Request) -> std::result::Result<String, prest::Error> {
                let value: #struct_name = Vals::from_request(req, &()).await?.0;
                value.save()?;
                Ok(value.get_key().to_string())
            }
            async fn remove(&self, req: Request) -> std::result::Result<(), prest::Error> {
                let value: #struct_name = Vals::from_request(req, &()).await?.0;
                value.remove()?;
                Ok(())
            }
        }

        impl prest::Table for #struct_name {
            const TABLE_NAME: &'static str = #table_name_str;
            const TABLE_SCHEMA: ColumnsSchema = &[#(#table_schema_clone),*];
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

            fn prepare_table() {
                Self::migrate();
                prest::DB_SCHEMA.add_table(&#schema_name);
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
            #(#range_fns)*
            #(#update_fns)*
            #(#check_fns)*
        }
    }
}

enum FromRowTransform {
    UuidFromU128,
    Deserialize,
    AsF32,
    None,
}

struct Column {
    name_ident: Ident,
    name_str: String,
    type_string: String,
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
    custom_type: bool,
    stringy_expr_node: bool,
    comparable: bool,
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

    if key && optional || key && list {
        panic!("Primary Key (first attribute by default) cannot be Option<...> or Vec<...>")
    }

    let custom_type = match inner_type_str {
        "Uuid" | "String" | "NaiveDateTime" | "bool" | "u128" | "u64" | "u32" | "u16" | "u8"
        | "i128" | "i64" | "i32" | "i16" | "i8" | "f64" | "f32" => false,
        _ => true,
    };

    let inner_type: syn::Type = syn::parse_str(inner_type_str).unwrap();

    let column_type = if list {
        "LIST"
    } else {
        match inner_type_str {
            "Uuid" => "UUID",
            "String" => "TEXT",
            "NaiveDateTime" => "TIMESTAMP",
            "bool" => "BOOLEAN",
            "u128" => "UINT128",
            "u64" => "UINT64",
            "u32" => "UINT32",
            "u16" => "UINT16",
            "u8" => "UINT8",
            "i128" => "INT128",
            "i64" => "INTEGER",
            "i32" => "INT32",
            "i16" => "INT16",
            "i8" => "INT8",
            "f64" | "f32" => "FLOAT",
            _ => "TEXT",
        }
    }
    .to_owned();

    let raw_dbvalue_variant = match inner_type_str {
        "Uuid" => "Uuid",
        "String" => "Str",
        "NaiveDateTime" => "Timestamp",
        "bool" => "Bool",
        "u128" => "U128",
        "u64" => "U64",
        "u32" => "U32",
        "u16" => "U16",
        "u8" => "U8",
        "i128" => "I128",
        "i64" => "I64",
        "i32" => "I32",
        "i16" => "I16",
        "i8" => "I8",
        "f64" | "f32" => "F64",
        _ => "Str",
    };
    let dbvalue_variant = ident(raw_dbvalue_variant);

    let from_row_transform = match inner_type_str {
        "Uuid" => FromRowTransform::UuidFromU128,
        "f32" => FromRowTransform::AsF32,
        _ if custom_type => FromRowTransform::Deserialize,
        _ => FromRowTransform::None,
    };

    let stringy_in_sql = list
        || custom_type
        || inner_type_str == "Uuid"
        || inner_type_str == "String"
        || inner_type_str == "NaiveDateTime";

    // proly a lot more
    let stringy_expr_node =
        inner_type_str != "String" && inner_type_str != "i64" && inner_type_str != "bool";

    let comparable = match inner_type_str {
        "NaiveDateTime" | "u128" | "u64" | "u32" | "u16" | "u8" | "i128" | "i64" | "i32"
        | "i16" | "i8" | "f64" | "f32" => true,
        _ => false,
    } && !optional
        && !list;

    Column {
        type_string: type_str.to_owned(),
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
        custom_type,
        stringy_expr_node,
        comparable,
    }
}

fn row_value_fmt(column: &Column) -> String {
    let Column {
        list,
        stringy_in_sql,
        name_str,
        custom_type,
        ..
    } = column;
    if *custom_type {
        format!("'{{}}'")
    } else if *list {
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
        custom_type,
        stringy_expr_node,
        ..
    } = column;
    let key: Expr = parse_quote!(#name_str);
    let fmt_str = if *list { "'{:?}'" } else { "'{}'" };
    let value = if *custom_type && *optional {
        quote!(if let Some(v) = &self.#name_ident { format!(#fmt_str, serde_json::to_string(v).unwrap()) } else { "NULL".to_owned() })
    } else if *custom_type {
        quote!(format!(#fmt_str, serde_json::to_string(&self.#name_ident).unwrap()))
    } else if *stringy_in_sql && *optional {
        quote!(if let Some(v) = &self.#name_ident { format!(#fmt_str, v.clone()) } else { "NULL".to_owned() })
    } else if *optional {
        quote!(if let Some(v) = &self.#name_ident { v.clone() } else { "NULL".to_owned() })
    } else if *stringy_in_sql {
        quote!(format!(#fmt_str, self.#name_ident.clone()))
    } else if *stringy_expr_node {
        quote!(self.#name_ident.to_string())
    } else {
        quote!(self.#name_ident.clone())
    };

    quote! { .set(#key, #value) }
}

fn find_by_(col: &Column) -> Ident {
    ident(&format!("find_by_{}", col.name_str))
}

fn find_by_null_(col: &Column) -> Ident {
    ident(&format!("find_by_null_{}", col.name_str))
}

fn find_in_range_(col: &Column) -> Ident {
    ident(&format!("find_in_range_{}", col.name_str))
}

fn update_(col: &Column) -> Ident {
    ident(&format!("update_{}", col.name_str))
}

fn check_(col: &Column) -> Ident {
    ident(&format!("check_{}", col.name_str))
}

fn ident(name: &str) -> Ident {
    Ident::new(name, Span::call_site())
}
