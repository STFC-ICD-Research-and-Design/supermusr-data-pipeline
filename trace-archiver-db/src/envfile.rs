use anyhow::Result;

use std::{env, fs::File, io::Write};

use dotenv;

use crate::utils::get_user_confirmation;

use super::Cli;
use super::utils::{log_then_panic, log_then_panic_t, unwrap_num_or_env_var};

pub(crate) fn write_env(cli : &Cli) {
    let cd = env::current_dir().unwrap_or_else(|e|log_then_panic_t(format!("Cannot obtain current directory : {e}")));
    let path = cd.join(".env");
    if path.exists() {
        let path_str = path.to_str().unwrap_or_else(||log_then_panic_t(format!("Cannot parse path {path:?}")));
        if !get_user_confirmation(&format!("File {path_str} already exists. Overwrite? (Y/N): "), "Overwriting file", "File was not modified. Exiting") {
            return;
        }
    }

    let mut file = File::create(path).unwrap_or_else(|e|log_then_panic_t(format!("Cannot create .env file : {e}")));
    if let Err(e) = write_file(&mut file, cli) {
        log_then_panic(format!("Cannot write to .env file : {e}"));
    }
    if let Err(e) = file.flush() {
        log_then_panic(format!("Cannot flush to .env file : {e}"));
    }
}

fn write_file(file : &mut File, cli : &Cli) -> Result<()> {
    write_line(file, &cli.td_broker_url,                            "TDENGINE_URL = taos://localhost")?;
    write_line(file, &cli.td_broker_port.map(|x|x.to_string()),     "TDENGINE_PORT = 6030")?;
    write_line(file, &cli.td_database,                              "TDENGINE_DATABASE = tracelogs")?;
    write_line(file, &cli.td_num_channels.map(|x|x.to_string()),    "TDENGINE_NUM_CHANNELS = 8")?;
    write_line(file, &cli.td_username,                              "TDENGINE_USER = user")?;
    write_line(file, &cli.td_password,                              "TDENGINE_PASSWORD = password")?;
    writeln!(file, "\n")?;
    
    write_line(file, &cli.kafka_broker_url,                         "REDPANDA_URL = localhost")?;
    write_line(file, &cli.kafka_broker_port.map(|x|x.to_string()),  "REDPANDA_PORT = 19092")?;
    write_line(file, &cli.kafka_username,                           "REDPANDA_USER = user")?;
    write_line(file, &cli.kafka_password,                           "REDPANDA_PASSWORD = password")?;
    write_line(file, &cli.kafka_consumer_group,                     "REDPANDA_CONSUMER_GROUP = ")?;
    write_line(file, &cli.kafka_trace_topic,                        "REDPANDA_TOPIC_SUBSCRIBE = MyTopic")?;
    writeln!(file, "\n")?;
    
    writeln!(file, "BENCHMARK_DELAY = 0")?;
    writeln!(file, "BENCHMARK_REPEATS = 80")?;
    writeln!(file, "BENCHMARK_NUM_CHANNELS_RANGE = 8:8:1")?;
    writeln!(file, "BENCHMARK_NUM_SAMPLES_RANGE = 10000:10000:1")?;
    Ok(())
}
fn write_line(file : &mut File, input : &Option<String>, default : &str) -> Result<(),std::io::Error> {
    match input {
        Some(string) => writeln!(file, "{string}"),
        None => writeln!(file, "{default}"),
    }
}