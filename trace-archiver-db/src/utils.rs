use std::str::FromStr;

pub(crate) fn log_then_panic(string : String) {
    log_then_panic_t::<()>(string);
}

pub(crate) fn log_then_panic_t<T>(string : String) -> T {
    log::error!("{string}");
    panic!("{string}");
}

pub(crate) fn unwrap_string_or_env_var(source : &Option<String>, var : &str) -> String {
    source.clone().unwrap_or_else(||
        dotenv::var(var).unwrap_or_else(|e| log_then_panic_t(format!("{var}: {e}")))
    )
}
pub(crate) fn unwrap_num_or_env_var<T : FromStr + Clone>(source : &Option<T>, var : &str) -> T where <T as FromStr>::Err: std::fmt::Display {
    source.clone().unwrap_or_else(||
        dotenv::var(var)
            .unwrap_or_else(|e| {
                log::error!("{var}: {e}");
                panic!("{var}: {e}");
            })
            .parse::<T>().unwrap_or_else(|e| log_then_panic_t(format!("Environment Variable {var} : {e}")))
    )
}
