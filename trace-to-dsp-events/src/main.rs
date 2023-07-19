//! This crate uses the benchmarking tool for testing the performance of implementated time-series databases.
//! 
#![allow(dead_code,unused_variables,unused_imports)]
#![warn(missing_docs)]

use std::{thread, time::Instant};

use common::Intensity;
use common::Time;

use anyhow::Result;

use dotenv;
use clap::{Parser, Subcommand};

use itertools::Itertools;
use trace_to_events::{EventFilter, Event, EventsDetector};

#[derive(Parser)]
#[clap(author, version, about)]
pub(crate) struct Cli {
    #[command(subcommand)]
    mode : Option<Mode>
}

#[derive(Subcommand, Clone)]
enum Mode {
    #[clap(about = "Listen to messages on the kafka server.")]
    Normal(NormalParameters),
}

#[derive(Parser, Clone)]
struct NormalParameters {
}


#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    
    log::debug!("Parsing Cli");
    let cli = Cli::parse();

    match cli.mode {
        Some(Mode::Normal(npm)) => run_normal_mode(),
        None => run_normal_mode(),
    }



    let input = vec![
        1.0, 1.0, 1.1, 1.0, 0.9, 1.0, 1.0, 1.1, 1.0, 0.9, 1.0, 1.1, 1.0, 1.0, 0.9, 1.0, 1.0, 1.1, 1.0,
        1.0, 1.0, 1.0, 1.1, 0.9, 1.0, 1.1, 1.0, 1.0, 0.9, 1.0, 1.1, 1.0, 1.0, 1.1, 1.0, 0.8, 0.9, 1.0,
        1.2, 0.9, 1.0, 1.0, 1.1, 1.2, 1.0, 1.5, 1.0, 3.0, 2.0, 5.0, 3.0, 2.0, 1.0, 1.0, 1.0, 0.9, 1.0,
        1.0, 3.0, 2.6, 4.0, 3.0, 3.2, 2.0, 1.0, 1.0, 0.8, 4.0, 4.0, 2.0, 2.5, 1.0, 1.0, 1.0
    ];
    
    let mut events : Vec<_> = input.iter().map(|x|(x*20.) as Intensity).collect_vec().into_iter()
    .enumerate()
    .events(EventsDetector::new(5, 5.0, 0.0))
    .collect();
    for (i,line) in input.iter().enumerate() {
        if let Some(event) = events.first() {
            if event.time == i as Time {
                print!("S|");
            } else if event.time + event.width == i as Time {
                print!("E|");
                events.remove(0);
            } else {
                print!(" |");
            }
        }
        else {
            print!(" |");
        }
        for _ in 0..(10.*line) as usize { print!(" "); }
        println!("x");
    }

    Ok(())
}

fn run_normal_mode() {

}