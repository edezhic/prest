#[macro_export]
macro_rules! state {
    ($struct_name:ident: $type:ty = $init:block) => {
        pub static $struct_name: prest::Lazy<$type> = prest::Lazy::new(|| {
            fn init() -> Result<$type, Box<dyn std::error::Error>> {
                let v = { $init };
                Ok(v)
            }
            init().unwrap()
        });
    };
    ($struct_name:ident: $type:ty = async $init:block) => {
        pub static $struct_name: prest::Lazy<$type> = prest::Lazy::new(|| {
            async fn init() -> Result<$type, Box<dyn std::error::Error>> {
                let v = { $init };
                Ok(v)
            }
            prest::block_on(init()).unwrap()
        });
    };
}