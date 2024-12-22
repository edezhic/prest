mod analyze;
mod expand;
mod from_glue_value;
mod into_glue_expr;

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{quote as q, ToTokens};
use syn::{
    parse_macro_input, parse_quote, Data, DataStruct, DeriveInput, Expr, Field, Fields, Ident, Type,
};

pub(crate) use gluesql_core::ast::DataType as SqlType;
use SqlType::*;

/// Generates schema and helper functions to use struct as a table in the embedded database
#[proc_macro_derive(Table, attributes(pkey_column, unique_column))]
pub fn table_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let struct_ident = ast.ident;
    let table_name = struct_ident.to_string() + "s";

    // supports only struct with named fields
    let fields = match ast.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(it),
            ..
        }) => it,
        _ => panic!("Expected a `struct` with named fields"),
    };

    // decompose
    let mut columns: Vec<Column> = fields.named.into_iter().map(analyze::from_field).collect();

    // can't have multiple primary keys
    match columns.iter().filter(|c| c.pkey).count() {
        0 => {
            columns[0].pkey = true;
            columns[0].unique = true;
        }
        1 => {}
        _ => panic!("Table macro doesn't support more than one pkey at the moment"),
    };

    // expand
    TokenStream::from(expand::impl_table(struct_ident, table_name, columns))
}

struct Column {
    field_name: Ident,
    field_name_str: String,
    full_type: Type,
    full_type_str: String,
    // type inside Option or Vec
    inner_type: Type,
    // type in sql syntax (gluesql_core::ast::DataType)
    sql_type: SqlType,
    // is primary pkey
    pkey: bool,
    // is Option<...>
    optional: bool,
    // is Vec<...>
    list: bool,
    // should be UNIQUE
    unique: bool,
    // requires serialization/deserialization
    serialized: bool,
}

impl Column {
    fn from_row_transform(&self) -> FromRowTransform {
        match self.sql_type {
            Uuid => FromRowTransform::UuidFromU128,
            _ if self.serialized => FromRowTransform::Deserialize,
            _ => FromRowTransform::None,
        }
    }

    fn value_variant(&self) -> &str {
        match self.sql_type {
            Uuid => "Uuid",
            Text => "Str",
            Timestamp => "Timestamp",
            Boolean => "Bool",
            Uint128 => "U128",
            Uint64 => "U64",
            Uint32 => "U32",
            Uint16 => "U16",
            Uint8 => "U8",
            Int128 => "I128",
            Int => "I64",
            Int32 => "I32",
            Int16 => "I16",
            Int8 => "I8",
            Float32 => "F32",
            Float => "F64",
            _ => "Str",
        }
    }
}

enum FromRowTransform {
    UuidFromU128,
    Deserialize,
    None,
}

fn ident(name: &str) -> Ident {
    Ident::new(name, Span::call_site())
}

trait TypeProps {
    fn impl_into_exprnode(&self) -> bool;
    fn int_or_smaller(&self) -> bool;
    fn integer(&self) -> bool;
    fn numeric(&self) -> bool;
    fn comparable(&self) -> bool;
    fn quoted(&self) -> bool;
}

impl TypeProps for SqlType {
    fn impl_into_exprnode(&self) -> bool {
        matches!(self, Boolean | Int)
    }
    fn int_or_smaller(&self) -> bool {
        matches!(self, Uint32 | Uint16 | Uint8 | Int | Int32 | Int16 | Int8)
    }
    fn integer(&self) -> bool {
        self.int_or_smaller() || matches!(self, Uint128 | Uint64 | Int128)
    }
    fn numeric(&self) -> bool {
        self.integer() || matches!(self, Float | Float32)
    }
    fn comparable(&self) -> bool {
        self.numeric() || matches!(self, Timestamp | Date | Time)
    }
    fn quoted(&self) -> bool {
        !self.impl_into_exprnode() && !self.numeric()
    }
}

impl TypeProps for Column {
    fn impl_into_exprnode(&self) -> bool {
        self.sql_type.impl_into_exprnode()
    }
    fn int_or_smaller(&self) -> bool {
        self.sql_type.int_or_smaller()
    }
    fn integer(&self) -> bool {
        self.sql_type.integer()
    }
    fn numeric(&self) -> bool {
        self.sql_type.numeric()
    }
    fn comparable(&self) -> bool {
        self.sql_type.comparable()
    }
    fn quoted(&self) -> bool {
        self.sql_type.quoted()
    }
}

fn column_schema(col: &Column) -> proc_macro2::TokenStream {
    let Column {
        field_name_str,
        full_type_str,
        sql_type,
        pkey,
        unique,
        list,
        optional,
        serialized,
        ..
    } = col;
    let numeric = sql_type.numeric();
    let comparable = sql_type.comparable();
    let sql_type = sql_type.to_string();
    q! {
        ColumnSchema {
            name: #field_name_str,
            rust_type: #full_type_str,
            sql_type: #sql_type,
            unique: #unique,
            pkey: #pkey,
            list: #list,
            optional: #optional,
            serialized: #serialized,
            numeric: #numeric,
            comparable: #comparable,
        }
    }
}
