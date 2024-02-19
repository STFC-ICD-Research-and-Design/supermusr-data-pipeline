# trace-to-events

## Introduction

## Command Line Interface
simulator [OPTIONS] --broker <BROKER> [COMMAND]

For instance:
```
simulator
```


For instructions run:
```
simulator --help
```

### Commands:
-  `Single`:            Detects events using a constant phase discriminator. Events consist only of a time value.
-  `Continuous`:        Detects events using differential discriminators. Event lists consist of time and voltage values.
-  `Json`:              Print this message or the help of the given subcommand(s)

### Constant Phase Discriminator:
`trace-to-events --broker <BROKER> constant-phase-discriminator --threshold-trigger <THRESHOLD_TRIGGER>`

```
      --threshold-trigger <THRESHOLD_TRIGGER>
          constant phase threshold for detecting muon events, use format (threshold,duration,cool_down).
  -h, --help
          Print help
```
A threshold is given by a triple of the form "threshold,duration,cool_down", threshold is the real threshold value, duration is how long the signal should be beyond the threshold to trigger an event (should be positive), and cool_down is how long before another detection can be found (should be non-negative).

### Advanced Muon Detector:
`trace-to-events --broker <BROKER> advanced-muon-detector [OPTIONS] --baseline-length <BASELINE_LENGTH> --smoothing-window-size <SMOOTHING_WINDOW_SIZE> --muon-onset <MUON_ONSET> --muon-fall <MUON_FALL> --muon-termination <MUON_TERMINATION>`
```
      --baseline-length <BASELINE_LENGTH>
          Size of initial portion of the trace to use for determining the baseline. Initial portion should be event free.
      --smoothing-window-size <SMOOTHING_WINDOW_SIZE>
          Size of the moving average window to use for the lopass filter.
      --muon-onset <MUON_ONSET>
          Differential threshold for detecting muon onset (threshold,duration,cool_down). See README.md.
      --muon-fall <MUON_FALL>
          Differential threshold for detecting muon peak (threshold,duration,cool_down). See README.md.
      --muon-termination <MUON_TERMINATION>
          Differential threshold for detecting muon termination (threshold,duration,cool_down). See README.md.
      --max-amplitude <MAX_AMPLITUDE>
          Optional parameter which (if set) filters out events whose peak is greater than the given value.
      --min-amplitude <MIN_AMPLITUDE>
          Optional parameter which (if set) filters out events whose peak is less than the given value.
  -h, --help
          Print help
```

## Json Format
### Simulator
 - voltage: `Interval`
 - voltage-transformation: `Transformation`
 - "sample-rate": `Integer`
 - "traces": `[TraceMessage]`

### TraceMessage
 - digitizers: `[Digitizer]`
 - frames : `[Integer]`
 - pulses : `[Pulse]`
 - noises : `[NoiseSource]`
 - num-pulses : `Distribution`
 - time-bins : `Integer`
 - timestamp : `Timestamp`,
 - frame-delay-us : `Integer`,

### Digitizer
 - id : `Integer`
 - channels : `Interval`

### Pulse
 - weight : `Float`
 - attributes : `PulseAttributes`

### PulseAttributes
A `PulseAttributes` object is one of the following
 - Gaussian
    - type = "gaussian"
    - peak_height : `Distribution`
    - peak_time : `Distribution`
    - sd : `Distribution`
    ```
    {
        "type": "gaussian",
        "peak_height": { "min": 30, "max": 70 },
        "peak_time": { "lifetime": 2200 },
        "sd": { "min": 5, "max": 20 }
    }
    ```
 - Biexp
    - type = "biexp"
    - start : `Distribution`
    - peak_height : `Distribution`
    - decay : `Distribution`
    - rise : `Distribution`
    ```
    {
        "type": "biexp",
        "start" : { "lifetime" : 2200 }
        "peak_height": { "min": 30, "max": 70 },
        "decay": { "min": 5, "max": 10 },
        "rise": { "min": 15, "max": 20 }
    }
    ```


### NoiseSource
A NoiseSource object is one of the following
 - Gaussian
    - type = "gaussian"
    - mean : Distribution
    - sd : Distribution
    - smoothing : Integer
 - Uniform
    - type = "uniform"
    - min : Distribution
    - max : Distribution
    - smoothing : Integer

### Interval
 - min : Integer
 - max : Integer

### Distribution
A Distribution object is one of the following
 - Float
    - This distribution is a constant value
    ```
    40.0
    ```
 - Uniform
    - min : Float
    - max : Float
    ```
    {"min" : 20.0, "max" : 60.0}
    ```
 - Gaussian
    - mean : Float
    - sd : Float
    ```
    { "mean" : 40.0, "sd" : 20.0 }
    ```
 - Exponential
    - lifetime : Float
        - The mean value is equal to the lifetime parameter, however note that the exponential distribution is usually given by the `rate` parameter which is `1/lifetime`.
    ```
    { "lifetime" : 40.0 }
    ```

### Transformation
 - scale : Float
 - translation : Float