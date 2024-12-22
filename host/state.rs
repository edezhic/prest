/// Macro that simplifies lazy globals by reducing boilerplate, allowing `?` operator and async init
#[macro_export]
macro_rules! state {
    ($(($v:tt))? $struct_name:ident: $type:ty = $init:block) => {
        pub$(($v))? static $struct_name: prest::Lazy<$type> = prest::Lazy::new(|| {
            fn init() -> prest::AnyhowResult<$type> {
                let v = { $init };
                Ok(v)
            }
            init().expect("Prest initialization must finish successfully")
        });
    };

    ($(($v:tt))? $struct_name:ident: $type:ty = async $init:block) => {
        pub$(($v))? static $struct_name: prest::Lazy<$type> = prest::Lazy::new(|| {
            async fn init() -> prest::AnyhowResult<$type> {
                let v = { $init };
                Ok(v)
            }
            if let Ok(handle) = prest::RuntimeHandle::try_current() {
                if handle.runtime_flavor() != prest::RuntimeFlavor::CurrentThread {
                    prest::block_in_place(move || handle.block_on(init())
                        .expect("Prest initialization must finish successfully"))
                } else {
                    panic!("Prest doesn't support async state inside of the tokio's current_thread runtime yet")
                }
            } else {
                prest::RuntimeBuilder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("Runtime spawn should be fine outside of another runtime")
                    .block_on(init())
                    .expect("Prest initialization must finish successfully")
            }
        });
    };
}
