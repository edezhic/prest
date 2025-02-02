use super::*;
use proc_macro2::TokenStream;

pub fn from_glue_value((index, col): (usize, &Column)) -> TokenStream {
    let Column {
        field_name,
        inner_type,
        sql_type,
        optional,
        list,
        ..
    } = col;

    let value_variant = ident(col.value_variant());

    let transform = match col.value_transform() {
        ValueTransform::UuidU128 => q!(let v = prest::Uuid::from_u128(v)),
        ValueTransform::SerDe => q!(let v = prest::from_bitcode(&v)?),
        ValueTransform::None => q!(),
    };

    let error_arms = q!(
        Some(other) => {
            let column = &Self::FIELD_SCHEMAS[#index];
            return Err(prest::e!("unexpected value {other:?} for {column:?}"))
        }
        None => {
            let column = &Self::FIELD_SCHEMAS[#index];
            return Err(prest::e!("row too short, missing {column:?}"))
        }
    );

    if *list {
        let err = q!(
            let column = &Self::FIELD_SCHEMAS[#index];
            return Err(prest::e!("unexpected list item value {item:?} in {column:?}"))
        );
        let match_and_push = if sql_type.int_or_smaller() {
            q!(
                use prest::sql::Value::*;
                let v = match item {
                    I8(v) => v as #inner_type,
                    I16(v) => v as #inner_type,
                    I32(v) => v as #inner_type,
                    I64(v) => v as #inner_type,
                    U8(v) => v as #inner_type,
                    U16(v) => v as #inner_type,
                    U32(v) => v as #inner_type,
                    item => { #err }
                };
                list.push(v);
            )
        } else {
            q!(
                if let prest::sql::Value::#value_variant(v) = item {
                    #transform;
                    list.push(v);
                } else { #err }
            )
        };
        q!(
            let #field_name = match row.pop() {
                Some(prest::sql::Value::List(values)) => {
                    let mut list = vec![];
                    for item in values.into_iter() { #match_and_push }
                    list
                }
                #error_arms
            };
        )
    } else {
        let res = if *optional { q!(Some(v)) } else { q!(v) };
        let null_arm = match optional {
            true => q!( Some(prest::sql::Value::Null) => None, ),
            false => q!(),
        };
        q! {
            let #field_name = match row.pop() {
                Some(prest::sql::Value::#value_variant(v)) => {#transform; #res}
                #null_arm
                #error_arms
            };
        }
    }
}
