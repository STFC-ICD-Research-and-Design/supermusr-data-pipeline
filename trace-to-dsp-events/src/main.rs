//! This crate uses the benchmarking tool for testing the performance of implementated time-series databases.
//! 
#![allow(dead_code,unused_variables,unused_imports)]
#![warn(missing_docs)]

use std::env;
use std::fmt::Display;
use std::{fs::File, io::Write};
use std::{thread, time::Instant};

use common::Intensity;
use common::Time;

use anyhow::Result;

use dotenv;
use clap::{Parser, Subcommand};

use itertools::Itertools;
use tdengine::utils::log_then_panic_t;
use trace_simulator::generator::{PulseDistribution, RandomInterval};

use trace_to_dsp_events::TraceMakerFilter;
use trace_to_dsp_events::window::composite::CompositeWindow;
use trace_to_dsp_events::window::smoothing_window;
use trace_to_dsp_events::{
    trace_iterators::load_from_trace_file::load_trace_file,
    trace_iterators::save_to_file::SaveToFile,
    processing,
    SmoothingWindow,
    Integer, Real,
    EventFilter,
    detectors::event::Event,
    event_detector::EventsDetector,
    peak_detector::PeakDetector
};
use trace_to_dsp_events::window::{
    WindowFilter,
    noise_smoothing_window::NoiseSmoothingWindow
};
use trace_to_dsp_events::detectors::change_detector::FiniteDifferenceChangeDetector;
use trace_to_dsp_events::trace_iterators::finite_difference::{FiniteDifferencesFilter, self, FiniteDifferencesIter};

use tdengine::tdengine::TDEngine;
use trace_simulator;

#[derive(Parser)]
#[clap(author, version, about)]
pub(crate) struct Cli {
    #[command(subcommand)]
    mode : Option<Mode>
}

#[derive(Subcommand, Clone)]
enum Mode {
    #[clap(about = "Generate Random Traces and Extract Pulses")]
    Normal(NormalParameters),
    #[clap(about = "Read Database Traces and Extract Pulses")]
    Database(DatabaseParameters),
}

#[derive(Parser, Clone)]
struct NormalParameters {
    #[clap(long,short='l',default_value="500")]
    trace_length : usize,

    #[clap(long,short='p',default_value="3")]
    min_pulses : usize,

    #[clap(long,short='P',default_value="10")]
    max_pulses : usize,

    #[clap(long,short='v',default_value="0")]
    min_voltage : Intensity,

    #[clap(long,short='b',default_value="50")]
    base_voltage : Intensity,

    #[clap(long,short='V',default_value="10000")]
    max_voltage : Intensity,

    #[clap(long,short='n',default_value="80")]
    voltage_noise : Intensity,

    #[clap(long,short='d',default_value="2")]
    decay_factor : f64,

    #[clap(long,short='s',default_value="2")]
    std_dev_min : f64,

    #[clap(long,short='S',default_value="10")]
    std_dev_max : f64,

    #[clap(long,short='t',default_value="3.0")]
    time_wobble : f64,

    #[clap(long,short='w',default_value="0.001")]
    value_wobble : f64,

    #[clap(long,short='m',default_value="200")]
    min_peak : Intensity,

    #[clap(long,short='M',default_value="900")]
    max_peak : Intensity,
}

#[derive(Parser, Clone)]
struct DatabaseParameters {
}


#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    
    log::debug!("Parsing Cli");
    let cli = Cli::parse();

    

    match cli.mode {
        Some(Mode::Normal(npm)) => run_normal_mode(npm),
        Some(Mode::Database(dpm)) => (),
        None => run_normal_mode(NormalParameters::parse()),
    }    

    Ok(())
}

fn save_to_file<T : Display,I : Iterator<Item = T>>(name : &str, it : I) {
    let cd = env::current_dir().unwrap_or_else(|e|log_then_panic_t(format!("Cannot obtain current directory : {e}")));
    let path = cd.join(name);
    let mut file = File::create(path).unwrap_or_else(|e|log_then_panic_t(format!("Cannot create {name} : {e}")));
    it.for_each(|v|write!(file,"{v},").unwrap_or_else(|e|log_then_panic_t(format!("Cannot write to {name} : {e}"))));
    writeln!(&mut file).unwrap_or_else(|e|log_then_panic_t(format!("Cannot event to {name} : {e}")));
}

fn run_normal_mode(params : NormalParameters) {
    let cd = env::current_dir().unwrap_or_else(|e|log_then_panic_t(format!("Cannot obtain current directory : {e}")));

    let distrbution = PulseDistribution {
        std_dev: RandomInterval(params.std_dev_min,params.std_dev_max),
        decay_factor: RandomInterval(0.,params.decay_factor),
        time_wobble: RandomInterval(0.,params.time_wobble),
        value_wobble: RandomInterval(0.,params.value_wobble),
        peak: RandomInterval(params.min_peak as f64,params.max_peak as f64)
    };

    let pulses = trace_simulator::create_pulses(
        params.trace_length,
        params.min_pulses,
        params.max_pulses,
        &distrbution,
    );
    let trace = trace_simulator::create_trace(
        params.trace_length,
        pulses,
        params.min_voltage,
        params.base_voltage,
        params.max_voltage,
        params.voltage_noise,
    );

    let mut trace_file = load_trace_file("traces/MuSR_A41_B42_C43_D44_Apr2021_Ag_ZF_IntDeg_Slit60_short.traces").unwrap();
    let run = trace_file.get_event(243).unwrap();
    /*
    let fin_diffs = run.channel_trace(0)
        .iter()
        .enumerate()
        .map(processing::make_enumerate_real)
        .finite_differences()
        .window(CompositeWindow::new([
            Box::new(NoiseSmoothingWindow::new(40,3.,0.)),
            Box::new(NoiseSmoothingWindow::new(40,3.,0.))
        ]))
        .events(EventsDetector::new([1.0, 2.0]))
        .trace(30000);
    */

    run.channel_trace(0)
        .iter()
        .enumerate()
        .save_to_file("data/trace.csv")
        .unwrap();

    let events = run.channel_trace(0)
        .iter()
        .enumerate()
        .map(processing::make_enumerate_real)
        .window(SmoothingWindow::new(10000)) // Smooth signal
        .map(smoothing_window::extract::enumerated_mean)
        .events(PeakDetector::default());

    println!("Num events: {}", events.count());

    run.channel_trace(0)
        .iter()
        .enumerate()
        .map(processing::make_enumerate_real)
        .window(SmoothingWindow::new(10000)) // Smooth signal
        .map(smoothing_window::extract::enumerated_mean)
        .events(PeakDetector::default())
        .trace(30_000)
        .map(processing::make_enumerate_integeral)
        .save_to_file("data/retrace.csv")
        .unwrap();
/*
    run.channel_trace(0)
        .iter()
        .enumerate()
        .map(processing::make_enumerate_real)
        .window(NoiseSmoothingWindow::new(400,3.,0.)) // Smooth signal
        .map(smoothing_window::extract::enumerated_mean)
        .map(processing::make_enumerate_integeral)
        .save_to_file("data/trace2.csv")
        .unwrap();

    run.channel_trace(0)
        .iter()
        .enumerate()
        .save_to_file("data/trace3.csv")
        .unwrap(); */
/*
    trace.clone()
        .into_iter()
        .enumerate()
        .map(|(i,v)|(i as Real,v as Real))        // Convert to floating point representation
        .window(NoiseSmoothingWindow::new(40,3.)) // Smooth signal
        .map(|(i,(v,(m,s)))|(i,m))                // Extract enumerated mean
        .map(|(i,v)|(i as Integer, v as Integer)) // Convert to integers
        .save_to_file("data/trace1.csv")
        .unwrap();

    trace.clone()
        .into_iter()
        .enumerate()
        .map(|(i,v)|(i as Real,v as Real))        // Convert to floating point representation
        .window(NoiseSmoothingWindow::new(30,3.)) // Smooth signal
        .map(|(i,(v,(m,s)))|(i,m))                // Extract enumerated mean
        .finite_differences()                     // Calculate finite differences
        .map(|(i,v) : (_,[_;2])| (i,v[1]))        // Extract enumerated first difference
        .window(SmoothingWindow::new(10))          // Smooth signal
        .map(|(i,(v,(m,s)))|(i,m))                // Extract enumerated mean
        .map(|(i,v)|(i as Integer, v as Integer)) // Convert to integers
        .save_to_file("data/trace2.csv")
        .unwrap();

    trace.clone()
        .into_iter()
        .enumerate()
        .map(|(i,v)|(i as Real,v as Real))        // Convert to floating point representation
        .window(NoiseSmoothingWindow::new(30,3.)) // Smooth signal
        .map(|(i,(v,(m,s)))|(i,m))                // Extract enumerated mean
        .finite_differences()                     // Calculate finite differences
        .map(|(i,v) : (_,[_;2])| (i,v[1]))        // Extract enumerated first difference
        .window(SmoothingWindow::new(10))          // Smooth signal
        .map(|(i,(v,(m,s)))|(i,m))                // Extract enumerated mean
        .events(PeakDetector::default())
        .save_to_file("data/events2.csv")
        .unwrap();
 */
    /*
    trace.clone()
        .into_iter()
        .enumerate()
        .map(|(i,v)|(i as Real,v as Real))        // Convert to floating point representation
        .window(NoiseSmoothingWindow::new(10,2.)) // Smooth signal
        .map(|(i,(v,(m,s)))|(i,(m + 2.*s)))       // Extract enumerated mean
        .map(|(i,v)|(i as Integer, v as Integer)) // Convert to integers
        .save_to_file("data/trace.csv")
        .unwrap();

    trace.clone()
        .into_iter()
        .enumerate()
        .map(|(i,v)|(i as Real,v as Real))  // Convert to floating point representation
        .finite_differences()               // Calculate finite differences
        .map(|(i,v) : (_,[_;2])| (i,v[1]))  // Extract enumerated first difference
        .map(|(i,v)|(i as Integer, v as Integer)) // Convert to integers
        .save_to_file("data/trace4.csv")
        .unwrap();

    trace.clone()
        .into_iter()
        .enumerate()
        .map(|(i,v)|(i as Real,v as Real))        // Convert to floating point representation
        .finite_differences()                     // Calculate finite differences
        .map(|(i,v) : (_,[_;2])| (i,v[1]))        // Extract enumerated first difference
        .window(SmoothingWindow::new(10))         // Smooth signal
        .map(|(i,(v,(m,s)))|(i,m as Integer))     // Extract enumerated mean
        .map(|(i,v)|(i as Integer, v as Integer)) // Convert to integers
        .save_to_file("data/trace4.csv")
        .unwrap();

    trace.clone()
        .into_iter()
        .enumerate()
        .map(|(i,v)|(i as Real,v as Real))        // Convert to floating point representation
        .finite_differences()                     // Calculate finite differences
        .map(|(i,v) : (_,[_;2])| (i,v[1]))        // Extract enumerated first difference
        .window(SmoothingWindow::new(10))         // Smooth signal
        .map(|(i,(v,(m,s)))|(i,m as Integer))     // Extract enumerated mean
        .map(|(i,v)|(i as Integer, v as Integer)) // Convert to integers
        .save_to_file("data/trace.csv")
        .unwrap();
    */

    trace.clone()
        .into_iter()
        .enumerate()
        .map(|(i,v)|(i as Real,v as Real))
        .events(PeakDetector::default())
        .save_to_file("data/events.csv")
        .unwrap();

        
    /*
    trace.clone()
        .into_iter()
        .enumerate()
        .map(|(i,v)|(i as Real,v as Real))
        .finite_differences()
        .events(EventsDetector::new(10, 2.0, 0.9))
        .save_to_file("data/events.csv")
        .unwrap();
    
    trace.clone()
        .into_iter()
        .enumerate()
        .map(|(i,v)|(i as Real,v as Real))
        .finite_differences()
        .events(EventsDetector::new(10, 2.0, 0.3))
        .save_to_file("data/events.csv")
        .unwrap();
     */
}