use super::*;

pub fn from_field(field: Field) -> Column {
    let pkey = field
        .attrs
        .iter()
        .find(|a| a.path().to_token_stream().to_string() == "pkey")
        .is_some();

    let unique = field
        .attrs
        .iter()
        .find(|a| a.path().to_token_stream().to_string() == "unique")
        .is_some()
        || pkey;

    let field_name = field.ident.expect("only named structs");
    let field_name_str = field_name.to_string();
    let full_type = field.ty;
    let type_str = full_type.to_token_stream().to_string();
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

    if pkey && optional || pkey && list {
        panic!("Primary Key (first attribute by default) cannot be Option<...> or Vec<...>")
    }

    let inner_type: syn::Type = syn::parse_str(inner_type_str).unwrap();

    let serialized = match inner_type_str {
        "Uuid" | "String" | "NaiveDateTime" | "bool" | "u128" | "u64" | "u32" | "u16" | "u8"
        | "i128" | "i64" | "i32" | "i16" | "i8" | "f64" | "f32" => false,
        _ => true,
    };

    use SqlType::*;
    let sql_type = match inner_type_str {
        "Uuid" => Uuid,
        "NaiveDateTime" => Timestamp,
        "bool" => Boolean,
        "u128" => Uint128,
        "u64" => Uint64,
        "u32" => Uint32,
        "u16" => Uint16,
        "u8" => Uint8,
        "i128" => Int128,
        "i64" => Int,
        "i32" => Int32,
        "i16" => Int16,
        "i8" => Int8,
        "f32" => Float32,
        "f64" => Float,
        "String" => Text,
        _ if serialized => Bytea,
        _ => panic!("Unsupported inner type str = {inner_type_str}"),
        // _ => Text, // fallback?
    };

    Column {
        full_type_str: type_str.replace(' ', ""),
        sql_type,
        field_name,
        field_name_str,
        full_type,
        inner_type,
        pkey,
        optional,
        list,
        unique,
        serialized,
    }
}
