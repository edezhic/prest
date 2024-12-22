use super::{from_glue_value::from_glue_value, into_glue_expr::into_glue_expr, *};
use proc_macro2::TokenStream;

pub fn impl_table(struct_ident: Ident, table_name: String, columns: Vec<Column>) -> TokenStream {
    let fields_idents = columns.iter().map(|col| col.field_name.clone());
    let table_schema = columns.iter().map(column_schema);
    let add_columns = columns.iter().map(add_column);
    let from_row_extractions = columns.iter().enumerate().rev().map(from_glue_value);
    let into_row_items = columns.iter().rev().map(into_row_item);
    let find_fns = columns.iter().map(find_by);
    let check_fns = columns.iter().filter(|col| !col.pkey).map(check);
    let update_fns = columns.iter().filter(|col| !col.pkey).map(update);
    let get_all_as_strings = columns.iter().map(get_as_string);
    let range_fns = columns.iter().filter(|col| col.comparable()).map(in_range);

    let pkey_column = columns.iter().find(|c| c.pkey).unwrap();
    let Column {
        field_name: key_name_ident,
        field_name_str: key_name_str,
        full_type: key_type_token,
        ..
    } = &pkey_column;
    let id_from_str = if pkey_column.full_type_str != "String" {
        let key_type = pkey_column.full_type.clone();
        Some(q! {
            use std::str::FromStr;
            let id = #key_type::from_str(&id)?;
        })
    } else {
        None
    };

    let set_columns = columns.iter().filter(|col| !col.pkey).map(set_column);
    let save_fn = q!(
        fn save(&self) -> prest::Result<&Self> {
            if Self::find_by_pkey(self.get_pkey())?.is_some() {
                Self::update().filter(Self::pkey_filter(self.get_pkey()))
                    #(#set_columns)*
                    .exec()?;
            } else {
                self.insert_self()?;
            }
            Ok(&self)
        }
    );

    let pkey_expr = into_glue_expr(pkey_column, q!(pkey), false, false);
    let pkey_filter = q!(sql::col(#key_name_str).eq(#pkey_expr));

    let schema_name = ident(&format!("{}Schema", struct_ident.to_string()));

    let relative_path = format!("/table/{table_name}");
    let full_path = format!("/admin/db{relative_path}");

    let fields_idents2 = fields_idents.clone();
    let fields_idents3 = fields_idents.clone();
    let fields_idents4 = fields_idents.clone();
    let table_schema2 = table_schema.clone();
    let get_all_as_strings2 = get_all_as_strings.clone();

    q! {
        struct #schema_name;
        #[async_trait]
        impl TableSchemaTrait for #schema_name {
            fn name(&self) -> &'static str {
                #table_name
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
            async fn get_all(&self) -> prest::Result<Vec<Vec<String>>> {
                let mut rows = vec![];
                for item in #struct_ident::find_all()? {
                    let #struct_ident { #(#fields_idents3 ,)* } = item;
                    let mut row = vec![];
                    #(#get_all_as_strings)*
                    rows.push(row);
                }
                Ok(rows)
            }
            async fn get_row_by_id(&self, id: String) -> prest::Result<Vec<String>> {
                #id_from_str
                let Some(#struct_ident { #(#fields_idents4 ,)* }) = #struct_ident::find_by_pkey(&id)? else {
                    return Err(prest::e!("expected to find a row by id = {id}"))
                };
                let mut row = vec![];
                #(#get_all_as_strings2)*
                Ok(row)
            }
            async fn save(&self, req: Request) -> prest::Result<String> {
                let value: #struct_ident = Vals::from_request(req, &()).await?.0;
                value.save()?;
                Ok(value.get_pkey().to_string())
            }
            async fn remove(&self, req: Request) -> prest::Result {
                let value: #struct_ident = Vals::from_request(req, &()).await?.0;
                value.remove()?;
                Ok(())
            }
        }

        impl prest::Table for #struct_ident {
            const TABLE_NAME: &'static str = #table_name;
            const TABLE_SCHEMA: prest::ColumnsSchema = &[#(#table_schema2),*];
            const KEY: &'static str = #key_name_str;
            type Key = #key_type_token;

            fn get_pkey(&self) -> &Self::Key {
                &self.#key_name_ident
            }

            fn pkey_filter<'a, 'b>(pkey: &'a Self::Key) -> prest::sql::ExprNode<'b> { #pkey_filter }

            fn migrate() {
                let table = Self::TABLE_NAME;
                prest::sql::table(table)
                    .create_table_if_not_exists()
                    #(#add_columns)*
                    .exec()
                    .expect("migration for {table} should complete");
            }

            fn prepare_table() {
                Self::migrate();
                prest::DB_SCHEMA.add_table(&#schema_name);
            }

            fn from_row(mut row: Vec<prest::sql::Value>) -> prest::Result<Self> {
                #(#from_row_extractions)*
                Ok(Self { #(#fields_idents ,)* })
            }

            fn into_row(&self) -> prest::Result<sql::ExprList<'static>> {
                #(#into_row_items)*
                Ok(vec![#(#fields_idents2 ,)*].into())
            }

            #save_fn
        }

        impl #struct_ident {
            #(#find_fns)*
            #(#range_fns)*
            #(#update_fns)*
            #(#check_fns)*
        }
    }
}

fn add_column(col: &Column) -> TokenStream {
    let Column {
        field_name_str,
        sql_type,
        pkey,
        optional,
        unique,
        list,
        ..
    } = col;

    let col = if *list {
        format!("{field_name_str} LIST")
    } else {
        let unique = if !*pkey && *unique { " UNIQUE" } else { "" };
        let pkey = if *pkey { " PRIMARY KEY" } else { "" };
        let optional = if *optional { "" } else { " NOT NULL" };
        format!("{field_name_str} {sql_type}{pkey}{unique}{optional}")
    };

    q! { .add_column(#col) }
}

fn find_by(column: &Column) -> TokenStream {
    let Column {
        field_name_str,
        field_name,
        full_type,
        inner_type,
        optional,
        unique,
        ..
    } = column;

    let select = q!(Self::select()
        .filter(filter)
        .rows()?
        .into_iter()
        .map(Self::from_row)
        .collect::<Result<Vec<Self>>>());

    let find_null_fn = if *optional {
        let fn_name = find_by_null_(column);
        let filter = q!(sql::col(#field_name_str).eq(sql::null()));
        q!( pub fn #fn_name() -> prest::Result<Vec<Self>> { let filter = #filter; #select } )
    } else {
        q!()
    };

    let fn_name = find_by_(column);
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
        pub fn #fn_name(#field_name: &#fn_arg) -> #fn_value {
            let filter = #filter;
            #result
        }
        #find_null_fn
    }
}

fn in_range(col: &Column) -> TokenStream {
    let Column {
        inner_type,
        sql_type,
        ..
    } = col;
    let fn_name = find_in_range_(col);
    let filter_str = if sql_type.quoted() { "'{}'" } else { "{}" };
    let filter = q!(sql::col("timestamp")
        .gte(sql::expr(format!(#filter_str, min)))
        .and(sql::col("timestamp").lte(sql::expr(format!(#filter_str, max)))));

    let values = q!(Self::select()
        .filter(#filter)
        .rows()?
        .into_iter()
        .map(Self::from_row)
        .collect());
    q! { pub fn #fn_name(min: &#inner_type, max: &#inner_type) -> Result<Vec<Self>> { #values } }
}

fn update(col: &Column) -> TokenStream {
    let Column {
        field_name,
        full_type,
        ..
    } = col;
    let fn_name = update_(col);
    let set_column = set_column(col);
    q! {
        pub fn #fn_name(&mut self, #field_name: #full_type) -> prest::Result<&mut Self> {
            self.#field_name = #field_name;
            Self::update().filter(Self::pkey_filter(self.get_pkey()))
                #set_column
                .exec()?;
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
        pub fn #fn_name(&self, value: #full_type) -> prest::Result<bool> {
            if let Some(item) = Self::find_by_pkey(self.get_pkey())? {
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

fn into_row_item(column: &Column) -> TokenStream {
    let Column { field_name, .. } = column;

    let sql_expr = into_glue_expr(column, q!(self.#field_name), false, false);
    q!( let #field_name: sql::ExprNode<'static> = #sql_expr; )
}

fn set_column(column: &Column) -> TokenStream {
    let Column {
        field_name,
        field_name_str,
        ..
    } = column;
    let col: Expr = parse_quote!(#field_name_str);
    let value = into_glue_expr(column, q!(self.#field_name), false, false);
    q! { .set(#col, #value) }
}

fn find_by_(col: &Column) -> Ident {
    ident(&format!("find_by_{}", col.field_name_str))
}

fn find_by_null_(col: &Column) -> Ident {
    ident(&format!("find_by_null_{}", col.field_name_str))
}

fn find_in_range_(col: &Column) -> Ident {
    ident(&format!("find_in_range_{}", col.field_name_str))
}

fn update_(col: &Column) -> Ident {
    ident(&format!("update_{}", col.field_name_str))
}

fn check_(col: &Column) -> Ident {
    ident(&format!("check_{}", col.field_name_str))
}
