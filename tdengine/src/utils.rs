use std::{io::Write, str::FromStr};
/*
pub fn log_then_panic(string: String) {
    log_then_panic_t::<()>(string);
}

pub fn log_then_panic_t<T>(string: String) -> T {
    log::error!("{string}");
    panic!("{string}");
} */
/*
pub fn unwrap_string_or_env_var(source: Option<String>, var: &str) -> String {
    source.unwrap_or_else(|| log_then_panic_t(format!("Cannot find {var}")))
}
pub fn unwrap_num_or_env_var<T: FromStr + Clone>(source: Option<T>, var: &str) -> T
where
    <T as FromStr>::Err: std::fmt::Display,
{
    source.unwrap_or_else(||log_then_panic_t(format!("Cannot find {var}")))
}
 */
