# trace-to-events

## Introduction

## Command Line Interface

```shell
simulator [OPTIONS] --broker <BROKER> [COMMAND]
```

For instance:

```shell
simulator
```

For instructions run:

```shell
simulator --help
```

### Commands

- `Single`:            Detects events using a constant phase discriminator. Events consist only of a time value.
- `Continuous`:        Detects events using differential discriminators. Event lists consist of time and voltage values.
- `Json`:              Print this message or the help of the given subcommand(s)

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

   ```json
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

   ```json
   {
        "type": "biexp",
        "start" : { "lifetime" : 2200 },
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

   ```json
   40.0
   ```

- Uniform
   - min : Float
   - max : Float

   ```json
   {"min" : 20.0, "max" : 60.0}
   ```

- Gaussian
   - mean : Float
   - sd : Float

   ```json
   { "mean" : 40.0, "sd" : 20.0 }
   ```

- Exponential
   - lifetime : Float
      - The mean value is equal to the lifetime parameter, however note that the exponential distribution is usually given by the `rate` parameter which is `1/lifetime`.

      ```json
      { "lifetime" : 40.0 }
      ```

### Transformation

- scale : Float
- translation : Float
