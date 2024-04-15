# trace-to-events

## Introduction

## Command Line Interface

```shell
simulator [OPTIONS] --broker <BROKER> [COMMAND]
```

For instance:

```shell
simulator --broker localhost:19092 \
    --trace-topic Traces \
    --event-topic SimulatedEvents \
    defined \
      --path "trace.json" \
      --repeat=10
```

For instructions run:

```shell
simulator --help
```

### Commands

- `single`:           Produce a single trace.
- `continuous`:       Produce a regular infinite sequence of traces are regular intervals.
- `defined`:          Produce traces in the manner specified by a json file.

## Defined Format

In `defined` mode, the behavior is given by the simulator object in the user-defined json file.
The structure of the simulator object is:

### Simulator

- voltage: [`Interval`](#Interval)
- voltage-transformation: [`Transformation`](#Transformation)
- "sample-rate": `Integer`
- "traces": [`[TraceMessage]`](#TraceMessage)

### TraceMessage

- digitizers: [`[Digitizer]`](#Digitizer)
- frames : `[Integer]`
- pulses : [`[Pulse]`](#Pulse)
- noises : [`[NoiseSource]`](#NoiseSource)
- num-pulses : [`Distribution`](#Distribution)
- time-bins : `Integer`
- timestamp : `Timestamp`,
- frame-delay-us : `Integer`,

### Digitizer

- id : `Integer`
- channels : [`Interval`](#Interval)

### Pulse

- weight : `Float`
- attributes : [`PulseAttributes`](#PulseAttributes)

### PulseAttributes

A `PulseAttributes` object is one of the following

- Gaussian
   - type = "gaussian"
   - peak_height : [`Distribution`](#Distribution)
   - peak_time : [`Distribution`](#Distribution)
   - sd : [`Distribution`](#Distribution)

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
   - start : [`Distribution`](#Distribution)
   - peak_height : [`Distribution`](#Distribution)
   - decay : [`Distribution`](#Distribution)
   - rise : [`Distribution`](#Distribution)

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
   - mean : [`Distribution`](#Distribution)
   - sd : [`Distribution`](#Distribution)
   - smoothing : `Integer`
- Uniform
   - type = "uniform"
   - min : [`Distribution`](#Distribution)
   - max : [`Distribution`](#Distribution)
   - smoothing : `Integer`

### Interval

- min : `Integer`
- max : `Integer`

### Distribution

A Distribution object is one of the following

- Float
   - This distribution is a constant value

   ```json
   40.0
   ```

- Uniform
   - min : `Float`
   - max : `Float`

   ```json
   {"min" : 20.0, "max" : 60.0}
   ```

- Gaussian
   - mean : `Float`
   - sd : `Float`

   ```json
   { "mean" : 40.0, "sd" : 20.0 }
   ```

- Exponential
   - lifetime : `Float`
      - The mean value is equal to the lifetime parameter, however note that the exponential distribution is usually given by the `rate` parameter which is `1/lifetime`.

      ```json
      { "lifetime" : 40.0 }
      ```

### Transformation

- scale : `Float`
- translation : `Float`
