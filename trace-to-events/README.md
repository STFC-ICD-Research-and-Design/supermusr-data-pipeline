# trace-to-events

## Introduction
This tool converts traces to events using a composable pipeline of signal modifiers and event detectors.
Traces are consumed from a kafka broker, processed into events and the resulting event list messages is sent to the same broker.

## Command Line Interface
trace-to-events [OPTIONS] --broker <BROKER> [COMMAND]

### Commands:
-  `simple`:       Detects events using a constant phase discriminator. Events consist only of a time value.
-  `basic`:        Detects events using differential discriminators. Event lists consist of time and voltage values.
-  `help`:         Print this message or the help of the given subcommand(s)

### Options:
```
      --broker <BROKER>                                
      --username <USERNAME>                            
      --password <PASSWORD>                            
      --group <CONSUMER_GROUP>                         [default: trace-to-event]
      --trace-topic <TRACE_TOPIC>                      [default: Traces]
      --event-topic <EVENT_TOPIC>                      [default: Events]
      --observability-address <OBSERVABILITY_ADDRESS>  [default: 127.0.0.1:9090]
      --save-file-name <SAVE_FILE_NAME>                
  -h, --help                                           Print help
  -V, --version                                        Print version
```
The only mandatory option is `--broker`. This should be in format `"host":"port"`, e.g. `--broker localhost:19092`.

The trace topic is the kafka topic that trace messages are consumed from, and event topic is the topic that event messages are produced to.

## Configuring the Detector Pipeline
Given an iterator of type u16 (aliased as Intensity in the crate), the pipeline is setup as follows:
```
// trace : Vec<u16>
    let raw = trace
        .into_iter()
        .enumerate()
        .map(|(i, v)| (i as Real, v as Real));
```
Then the trace signal can be processed using window functions (e.g. smoothing, finite differences). For instance:
```
let smoothed = raw
    .window(Baseline::new(100, 0.1))
    .window(SmoothingWindow::new(5))    // this produces values of type Stats { value : Real, mean : Real, variance : Real }
    .map(|(i, stats)| (i, stats.mean));
```
The signal is first baselined (the baseline is estimated from an exponential average of the first 100 values, and then subtracted off),
then a moving average window of size 5 is applied to the signal, finally the mean of the signal is extracted.

Next the signal is transformed into events:
```
let events = smoothed
    .window(FiniteDifferences::<2>::new())  // this produces size 2 arrays: [trace value, 1st-difference of trace]
    .events(BasicMuonDetector::new(
        &basic_parameters.muon_onset.0,
        &basic_parameters.muon_fall.0,
        &basic_parameters.muon_termination.0,
    ))
```
and finally the events are assembled into pulses.
```
let pulses = events.assemble(BasicMuonAssembler::default())
    .collect();
```

## Data Types
- Real: an alias for f64
- Intensity: an alias for u16
- Time: an alias for u32
- TraceArray: a wrapper for an array of TraceValue types.
- RealArray: a specification of TraceArray to Real values.
- Pulse: 

## Traits
- Temporal: abstracts all possible types representing time, mainly (Real or Time)
- TraceValue: abstracts over all possible types representing voltage (mainly Real or RealArray).
- TracePoint: abstracts types representing points in the trace signal. TracePoint is mainly just a tuple `(T,V)`, where `T : Temporal` and `V : TraceValue`.

- EventData:
- EventPoint:
