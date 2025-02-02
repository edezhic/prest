use crate::*;

use sql::Key;

pub trait IntoSqlKey {
    fn into_sql_key(self) -> Key;
}

macro_rules! into_key {
    ($type:tt, $variant:tt) => {
        impl IntoSqlKey for $type {
            fn into_sql_key(self) -> Key {
                Key::$variant(self)
            }
        }
    };
}

into_key!(bool, Bool);
into_key!(i8, I8);
into_key!(i16, I16);
into_key!(i32, I32);
into_key!(i64, I64);
into_key!(i128, I128);
into_key!(u8, U8);
into_key!(u16, U16);
into_key!(u32, U32);
into_key!(u64, U64);
into_key!(u128, U128);
into_key!(String, Str);
into_key!(NaiveDateTime, Timestamp);
into_key!(NaiveDate, Date);
into_key!(NaiveTime, Time);

impl IntoSqlKey for Uuid {
    fn into_sql_key(self) -> Key {
        Key::Uuid(self.as_u128())
    }
}

impl IntoSqlKey for Vec<u8> {
    fn into_sql_key(self) -> Key {
        Key::Bytea(self)
    }
}

impl IntoSqlKey for f32 {
    fn into_sql_key(self) -> Key {
        Key::F32(sql::OrderedFloat(self))
    }
}

impl IntoSqlKey for f64 {
    fn into_sql_key(self) -> Key {
        Key::F64(sql::OrderedFloat(self))
    }
}

// TODO: remaining possible key types
// Decimal(v) => Ok(Key::Decimal(v)),
// Inet(v) => Ok(Key::Inet(v)),
// Interval(v) => Ok(Key::Interval(v)),
