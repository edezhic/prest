use super::{from_glue_value::from_glue_value, into_glue_expr::into_glue_expr, *};
use proc_macro2::TokenStream;

pub fn impl_table(struct_ident: Ident, table_name: String, columns: Vec<Column>) -> TokenStream {
    let fields_idents = columns.iter().map(|col| col.field_name.clone());
    let schema = columns.iter().map(column_schema);
    let from_row_extractions = columns.iter().enumerate().rev().map(from_glue_value);
    let into_row_items = columns.iter().rev().map(|c| {
        let name = c.field_name.clone();
        into_row_item(c, q!(self.#name))
    });
    let into_expr_list_items = columns.iter().rev().map(into_expr_list_item);
    let find_fns = columns.iter().map(select_by);
    let check_fns = columns.iter().filter(|col| !col.pkey).map(check);
    let update_fns = columns
        .iter()
        .enumerate()
        .filter(|(_, col)| !col.pkey)
        .map(update);
    let get_all_as_strings = columns.iter().map(get_as_string);
    let range_fns = columns.iter().filter(|col| col.comparable()).map(in_range);

    let pk_index = columns.iter().position(|c| c.pkey).unwrap();
    let pkey = columns.get(pk_index).unwrap();
    let Column {
        field_name: key_name_ident,
        field_name_str: key_name_str,
        full_type: key_type_token,
        ..
    } = &pkey;
    let id_from_str = if pkey.full_type_str != "String" {
        let key_type = pkey.full_type.clone();
        Some(q! {
            use std::str::FromStr;
            let id = #key_type::from_str(&id)?;
        })
    } else {
        None
    };

    let pkey_expr = into_glue_expr(pkey, q!(pkey), true, false);
    let pk_filter_sql_node = q!(sql::col(#key_name_str).eq(#pkey_expr));

    let pk_range_fn = pk_range(pkey);

    let schema_name = ident(&format!("{}Schema", struct_ident.to_string()));

    let relative_path = format!("/table/{table_name}");
    let full_path = format!("/admin/db{relative_path}");

    let fields_idents2 = fields_idents.clone();
    let fields_idents3 = fields_idents.clone();
    let fields_idents4 = fields_idents.clone();
    let fields_idents5 = fields_idents.clone();
    let get_all_as_strings2 = get_all_as_strings.clone();

    q! {
        struct #schema_name;
        #[async_trait]
        impl StructSchemaTrait for #schema_name {
            fn name(&self) -> &'static str {
                #table_name
            }
            fn fields(&self) -> FieldSchemas {
                #struct_ident::FIELD_SCHEMAS.clone()
            }
            fn relative_path(&self) -> &'static str {
                #relative_path
            }
            fn full_path(&self) -> &'static str {
                #full_path
            }
            async fn get_all_as_strings(&self) -> prest::Result<Vec<Vec<String>>> {
                let mut rows = vec![];
                for item in #struct_ident::get_all().await? {
                    let #struct_ident { #(#fields_idents3 ,)* } = item;
                    let mut row = vec![];
                    #(#get_all_as_strings)*
                    rows.push(row);
                }
                Ok(rows)
            }
            async fn get_as_strings_by_id(&self, id: String) -> prest::Result<Vec<String>> {
                #id_from_str
                let Some(#struct_ident { #(#fields_idents4 ,)* }) = #struct_ident::get_by_pkey(id.clone()).await? else {
                    return Err(prest::e!("expected to find a row by id = {id}"))
                };
                let mut row = vec![];
                #(#get_all_as_strings2)*
                Ok(row)
            }
            async fn save(&self, req: Request) -> prest::Result<String> {
                let value: #struct_ident = Vals::from_request(req, &()).await?.0;
                value.save().await?;
                Ok(value.get_pkey().to_string())
            }
            async fn remove(&self, req: Request) -> prest::Result {
                let value: #struct_ident = Vals::from_request(req, &()).await?.0;
                value.remove().await?;
                Ok(())
            }
        }

        #[prest::async_trait]
        impl prest::Storage for #struct_ident {
            const STRUCT_NAME: &'static str = #table_name;
            const FIELD_SCHEMAS: prest::FieldSchemas = &[#(#schema),*];
            const PK_INDEX: usize = #pk_index;
            type Key = #key_type_token;

            fn get_pkey(&self) -> &Self::Key {
                &self.#key_name_ident
            }

            fn pk_filter_sql_node<'a, 'b>(pkey: &'a Self::Key) -> prest::sql::ExprNode<'b> { #pk_filter_sql_node }

            fn schema() -> &'static dyn StructSchemaTrait { &#schema_name }

            fn from_row(mut row: Vec<prest::sql::Value>) -> prest::Result<Self> {
                #(#from_row_extractions)*
                Ok(Self { #(#fields_idents ,)* })
            }

            fn into_expr_list(&self) -> prest::Result<sql::ExprList<'static>> {
                #(#into_expr_list_items)*
                Ok(vec![#(#fields_idents2 ,)*].into())
            }

            fn into_row(&self) -> prest::Result<Vec<prest::sql::Value>> {
                #(#into_row_items)*
                Ok(vec![#(#fields_idents5 ,)*].into())
            }
        }

        impl #struct_ident {
            #(#find_fns)*
            #(#range_fns)*
            #(#update_fns)*
            #(#check_fns)*
            #pk_range_fn
        }
    }
}

fn select_by(column: &Column) -> TokenStream {
    let Column {
        field_name_str,
        field_name,
        full_type,
        inner_type,
        optional,
        unique,
        ..
    } = column;

    let select = q!(Self::select().filter(filter).rows().await);

    let find_null_fn = if *optional {
        let fn_name = select_by_null_(column);
        let filter = q!(sql::col(#field_name_str).eq(sql::null()));
        q!( pub async fn #fn_name() -> prest::Result<Vec<Self>> { let filter = #filter; #select } )
    } else {
        q!()
    };

    let fn_name = select_by_(column);
    let fn_value = match *unique {
        true => q!(Result<Option<Self>>),
        false => q!(Result<Vec<Self>>),
    };
    let result = match unique {
        true => q!(Ok(#select?.pop())),
        false => select,
    };

    let fn_arg = if *optional { inner_type } else { full_type };

    let filter_expr = into_glue_expr(column, q!(#field_name), true, true);
    let filter = q!(sql::col(#field_name_str).eq(#filter_expr));

    q! {
        pub async fn #fn_name(#field_name: &#fn_arg) -> #fn_value {
            let filter = #filter;
            #result
        }
        #find_null_fn
    }
}

fn in_range(col: &Column) -> TokenStream {
    let Column {
        field_name_str,
        inner_type,
        sql_type,
        ..
    } = col;
    let fn_name = find_in_range_(col);
    let filter_str = if sql_type.quoted() { "'{}'" } else { "{}" };
    let filter = q!(sql::col(#field_name_str)
        .gte(sql::expr(format!(#filter_str, min)))
        .and(sql::col(#field_name_str).lte(sql::expr(format!(#filter_str, max)))));

    let values = q!(Self::select().filter(#filter).rows().await);

    q! { pub async fn #fn_name(min: &#inner_type, max: &#inner_type) -> Result<Vec<Self>> { #values } }
}

fn pk_range(col: &Column) -> TokenStream {
    let Column { full_type, .. } = col;
    let fn_name = get_in_range_(col);
    q! {
        pub async fn #fn_name(min: #full_type, max: #full_type) -> prest::Result<Vec<Self>> {
            let payload = prest::DB
                .read(prest::Query::PKRange {
                    name: Self::STRUCT_NAME,
                    pkey_min: min.into_sql_key(),
                    pkey_max: max.into_sql_key(),
                })
                .await?;

            let prest::Payload::Rows(rows) = payload else {
                panic!("unexpected DB pk_range return payload: {payload:?}")
            };

            Ok(rows
                .into_iter()
                .map(|r| Self::from_row(r))
                .collect::<Result<Vec<_>>>()?)
        }
    }
}

fn update((index, col): (usize, &Column)) -> TokenStream {
    let Column {
        field_name,
        field_name_str,
        full_type,
        ..
    } = col;
    let fn_name = update_(col);
    let arg_name = ident(&format!("new_{field_name_str}"));
    let into_row_item = into_row_item(col, q!(#arg_name));
    q! {
        pub async fn #fn_name(&mut self, #arg_name: #full_type) -> prest::Result<&mut Self> {
            #into_row_item
            let pkey = self.get_pkey().clone().into_sql_key();
            let payload = prest::DB
                .write(prest::Transaction::UpdateField {
                    name: Self::STRUCT_NAME,
                    key: pkey,
                    column: #index,
                    value: #field_name,
                })
                .await?;

            let prest::Payload::Success = payload else {
                panic!("unexpected DB insert_self return payload: {payload:?}")
            };
            self.#field_name = #arg_name;
            Ok(self)
        }
    }
}

fn check(col: &Column) -> TokenStream {
    let Column {
        field_name,
        full_type,
        ..
    } = col;
    let fn_name = check_(col);
    q! {
        pub async fn #fn_name(&self, value: #full_type) -> prest::Result<bool> {
            if let Some(item) = Self::get_by_pkey(self.get_pkey().clone()).await? {
                Ok(item.#field_name == value)
            } else {
                Err(prest::Error::NotFound)
            }
        }
    }
}

fn get_as_string(col: &Column) -> TokenStream {
    let Column {
        field_name,
        list,
        optional,
        serialized,
        ..
    } = col;

    let preprocessing = (*serialized || *list || *optional)
        .then(|| q!(let #field_name = prest::to_json_string(&#field_name)?;));

    q! {
        #preprocessing
        let #field_name: String = #field_name.to_string();
        row.push(#field_name);
    }
}

fn into_row_item(column: &Column, path: TokenStream) -> TokenStream {
    let Column {
        field_name,
        optional,
        list,
        serialized,
        ..
    } = column;

    if *list && *serialized {
        return q!(let #field_name: prest::sql::Value = prest::sql::Value::List(
            #path
                .iter()
                .map(|v| prest::sql::Value::Bytea(prest::into_bitcode(v).expect("valid encoding")))
                .collect()
        ););
    }

    let transform = match column.value_transform() {
        ValueTransform::UuidU128 => q!(let v = #path.as_u128()),
        ValueTransform::SerDe => q!(let v = prest::into_bitcode(&#path)?),
        ValueTransform::None => q!(let v = #path.clone()),
    };

    let value_variant = ident(column.value_variant());
    let mut value = q!(sql::Value::#value_variant(v));

    if *list && *serialized {
        value = q!( prest::sql::Value::List(v.into_iter().map(|v| #value).collect()) )
    } else if *list {
        value = q!( prest::sql::Value::List(v.into_iter().map(|v| #value).collect()) )
    }

    if *optional {
        value = q!(if let Some(v) = v { #value } else { prest::sql::Value::Null })
    }

    q!(
        #transform;
        let #field_name: prest::sql::Value = #value;
    )
}

fn into_expr_list_item(column: &Column) -> TokenStream {
    let Column { field_name, .. } = column;

    let sql_expr = into_glue_expr(column, q!(self.#field_name), false, false);
    q!( let #field_name: sql::ExprNode<'static> = #sql_expr; )
}

// fn set_column(column: &Column) -> TokenStream {
//     let Column {
//         field_name,
//         field_name_str,
//         ..
//     } = column;
//     let col: Expr = parse_quote!(#field_name_str);
//     let value = into_glue_expr(column, q!(self.#field_name), false, false);
//     q! { .set(#col, #value) }
// }

fn select_by_(col: &Column) -> Ident {
    ident(&format!("select_by_{}", col.field_name_str))
}

fn select_by_null_(col: &Column) -> Ident {
    ident(&format!("select_by_null_{}", col.field_name_str))
}

fn find_in_range_(col: &Column) -> Ident {
    ident(&format!("find_in_range_{}", col.field_name_str))
}

fn get_in_range_(col: &Column) -> Ident {
    ident(&format!("get_in_{}_range", col.field_name_str))
}

fn update_(col: &Column) -> Ident {
    ident(&format!("update_{}", col.field_name_str))
}

fn check_(col: &Column) -> Ident {
    ident(&format!("check_{}", col.field_name_str))
}
