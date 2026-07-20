pub mod server_api;
pub mod shared;
pub mod tracer;

// map literal macro
#[macro_export]
macro_rules! map {
    ($($key:expr => $value:expr),* $(,)?) => {{
        core::convert::From::from([$(($key,$value),)*])
    }};
}


#[macro_export]
macro_rules! function_name {
    () => {{
        fn asdlfkjh() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        type_name_of(asdlfkjh)
            .rsplit("::")
            .find(|&part| part != "asdlfkjh" && part != "{{closure}}")
            .expect("Short function name")
    }};
}
