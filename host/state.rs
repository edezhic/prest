#[macro_export]
macro_rules! state {
    ($struct_name:ident: $type:ty = $init:block) => {
        pub static $struct_name: prest::Lazy<$type> = prest::Lazy::new(|| {
            fn init() -> Result<$type> {
                let v = { $init };
                Ok(v)
            }
            init().unwrap()
        });
    };
    ($struct_name:ident: $type:ty = async $init:block) => {
        pub static $struct_name: prest::Lazy<$type> = prest::Lazy::new(|| {
            async fn init() -> Result<$type> {
                let v = { $init };
                Ok(v)
            }
            if let Ok(handle) = prest::RuntimeHandle::try_current() {
                if handle.runtime_flavor() != prest::RuntimeFlavor::CurrentThread {
                    prest::block_in_place(move || handle.block_on(init()).unwrap())
                } else {
                    panic!("Prest doesn't support async state inside of the tokio's current_thread runtime yet")
                }
            } else {
                prest::RuntimeBuilder::new_current_thread().enable_all().build().unwrap().block_on(init()).unwrap()
            }
        });
    };
}
