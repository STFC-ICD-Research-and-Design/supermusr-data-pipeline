use std::{io::Write, str::FromStr};

pub fn log_then_panic(string: String) {
    log_then_panic_t::<()>(string);
}

pub fn log_then_panic_t<T>(string: String) -> T {
    log::error!("{string}");
    panic!("{string}");
}

pub fn unwrap_string_or_env_var(source: Option<String>, var: &str) -> String {
    source.unwrap_or_else(|| {
        dotenv::var(var).unwrap_or_else(|e| log_then_panic_t(format!("{var}: {e}")))
    })
}
pub fn unwrap_num_or_env_var<T: FromStr + Clone>(source: Option<T>, var: &str) -> T
where
    <T as FromStr>::Err: std::fmt::Display,
{
    source.unwrap_or_else(|| {
        dotenv::var(var)
            .unwrap_or_else(|e| {
                log::error!("{var}: {e}");
                panic!("{var}: {e}");
            })
            .parse::<T>()
            .unwrap_or_else(|e| log_then_panic_t(format!("Environment Variable {var} : {e}")))
    })
}

pub fn get_user_confirmation(question: &str, on_confirm: &str, on_deny: &str) -> bool {
    println!("{question} (Y/N): ");
    if let Err(e) = std::io::stdout().flush() {
        log_then_panic(format!("Error flushing stdout: {e}"))
    }
    let mut buffer: String = String::new();
    if let Err(e) = std::io::stdin().read_line(&mut buffer) {
        log_then_panic(format!("Cannot read user input: {e}"))
    }
    buffer.truncate(1);
    match buffer.eq_ignore_ascii_case("Y") {
        true => {
            println!("{on_confirm}");
            true
        }
        false => {
            println!("{on_deny}");
            false
        }
    }
}
