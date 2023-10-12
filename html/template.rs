use crate::*;

#[macro_export]
macro_rules! template {
    ($($markup:tt)*) => { get(|| async {html!($($markup)*)})};
}