use super::*;
use proc_macro2::TokenStream;

pub fn into_glue_expr(column: &Column, path: TokenStream, deref: bool, inner: bool) -> TokenStream {
    let Column {
        sql_type,
        list,
        optional,
        serialized,
        ..
    } = column;
    let optional = *optional && !inner;
    match (list, optional, serialized) {
        (true, false, _) => {
            let literal = |ts: TokenStream| q!(sql::Expr::Literal(sql::AstLiteral::#ts));

            let item_into_expr = match sql_type {
                _ if *serialized => literal(q!(HexString(prest::hex::encode(prest::into_bitcode(item)?)))),
                _ if sql_type.integer() => literal(q!(Number(item.into()))),
                SqlType::Boolean => literal(q!(Boolean(*item))),
                SqlType::Text => literal(q!(QuotedString(item.to_string()))),
                _ => unimplemented!("Vec of this type is not currently supported due to complexities related to the untyped nature of gluesql lists"),
            };

            q!({
                let mut elem = vec![];
                for item in #path.iter() {
                    elem.push(#item_into_expr)
                }
                sql::ExprNode::Expr(std::borrow::Cow::Owned(sql::Expr::Array { elem }))
            })
        }
        (false, false, true) => q!(sql::bytea(prest::into_bitcode(&#path)?)),
        (false, true, true) => q!(
            if let Some(v) = &#path { sql::bytea(prest::into_bitcode(v)?)}
            else { sql::null() }
        ),
        (false, true, false) => {
            let inner = match sql_type {
                SqlType::Text => q!(sql::text(v.clone())),
                SqlType::Uuid => q!(sql::uuid(v.to_string())),
                SqlType::Boolean => node_literal(q!(Boolean(*v))),
                // these do not implement Into<NumNode>
                SqlType::Int128 | SqlType::Uint128 => q!(sql::num(v.to_string())),
                _ if sql_type.numeric() => q!(sql::num(*v)),
                _ => q!(sql::expr(v.to_string())),
            };
            q!( if let Some(v) = &#path { #inner } else { sql::null() } )
        }
        (false, false, false) => {
            match sql_type {
                SqlType::Boolean => node_literal(q!(Boolean(#path.clone()))),
                SqlType::Text => q!(sql::text(#path.clone())),
                SqlType::Uuid => q!(sql::uuid(#path.to_string())),
                // these do not implement Into<NumNode>
                SqlType::Int128 | SqlType::Uint128 => q!(sql::num(#path.to_string())),
                _ if sql_type.numeric() && deref => q!(sql::num(*#path)),
                _ if sql_type.numeric() => q!(sql::num(#path)),
                _ => q!(sql::expr(format!("'{}'", &#path))),
            }
        }
        (true, true, _) => {
            unreachable!("doesn't support combinations of Vec<> and Option<> in the analyzer")
        }
    }
}

fn node_literal(variant: TokenStream) -> TokenStream {
    let expr = expr_literal(variant);
    q!(prest::sql::ExprNode::Expr(std::borrow::Cow::Owned(#expr)))
}

fn expr_literal(variant: TokenStream) -> TokenStream {
    q!(prest::sql::Expr::Literal(prest::sql::AstLiteral::#variant))
}
