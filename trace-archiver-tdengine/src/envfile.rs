use anyhow::Result;

use std::{env, fs::File, io::Write};

//use dotenv;

use super::Cli;
use crate::error::DotEnvWriteError;

pub fn get_user_confirmation(
    question: &str,
    on_confirm: &str,
    on_deny: &str,
) -> Result<bool, std::io::Error> {
    println!("{question} (Y/N): ");
    std::io::stdout().flush()?;
    let mut buffer: String = String::new();
    std::io::stdin().read_line(&mut buffer)?;
    buffer.truncate(1);
    let response = buffer.eq_ignore_ascii_case("Y");
    if response {
        println!("{on_confirm}");
    } else {
        println!("{on_deny}");
    }
    Ok(response)
}

pub(crate) fn write_env(cli: &Cli) -> Result<(), DotEnvWriteError> {
    let cd = env::current_dir().map_err(DotEnvWriteError::CannotObtainCurrentDirectory)?;
    let path = cd.join(".env");
    if path.exists() {
        let path_str = path.to_str().ok_or(DotEnvWriteError::CannotParsePath)?;
        if !get_user_confirmation(
            &format!("File {path_str} already exists. Overwrite? (Y/N): "),
            "Overwriting file",
            "File was not modified. Exiting",
        )
        .map_err(DotEnvWriteError::IOError)?
        {
            return Ok(());
        }
    }

    let mut file = File::create(path).map_err(DotEnvWriteError::CannotCreateDotEnvFile)?;
    write_file(&mut file, cli).map_err(DotEnvWriteError::CannotWriteToDotEnvFile)?;
    file.flush()
        .map_err(DotEnvWriteError::CannotFlushDotEnvFile)
}

fn write_file(file: &mut File, cli: &Cli) -> Result<(), std::io::Error> {
    write_line(file, cli.td_broker.as_deref(), "TDENGINE_BROKER", "localhost:6041")?;
    write_line(file, cli.td_database.as_deref(), "TDENGINE_DATABASE", "tracelogs")?;
    write_line(file, cli.td_num_channels.map(|x| x.to_string()).as_deref(), "TDENGINE_NUM_CHANNELS", "8",)?;
    write_line(file, cli.td_username.as_deref(), "TDENGINE_USER","user")?;
    write_line(file, cli.td_password.as_deref(), "TDENGINE_PASSWORD", "password")?;
    writeln!(file, "\n")?;

    write_line(file, cli.kafka_broker.as_deref(), "KAFKA_BROKER", "localhost:19092")?;
    write_line(file, cli.kafka_username.as_deref(), "KAFKA_USER"," user")?;
    write_line(file, cli.kafka_password.as_deref(), "KAFKA_PASSWORD", "password")?;
    write_line(file, Some(&cli.kafka_consumer_group), "KAFKA_CONSUMER_GROUP", "trace-consumer")?;
    write_line(file, cli.kafka_topic.as_deref(), "KAFKA_TOPIC", "Traces")?;
    writeln!(file, "\n")?;

    writeln!(file, "BENCHMARK_DELAY = 0")?;
    writeln!(file, "BENCHMARK_REPEATS = 80")?;
    writeln!(file, "BENCHMARK_NUM_CHANNELS_RANGE = 8:8:1")?;
    writeln!(file, "BENCHMARK_NUM_SAMPLES_RANGE = 10000:10000:1")?;
    Ok(())
}
fn write_line(
    file: &mut File,
    input: Option<&str>,
    key: &str,
    default: &str,
) -> Result<(), std::io::Error> {
    match input {
        Some(string) => writeln!(file, "{key} = {string}"),
        None => writeln!(file, "{key} = {default}"),
    }
}
