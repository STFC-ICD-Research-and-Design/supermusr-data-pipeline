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
- `start`:            Produce a run-start message to the `control` topic.
- `stop`:             Produce a run-stop message to the `control` topic.
- `log`:              Produce a run log data message to the `control` topic.
- `sample-env`:       Produce a sample environment log message to the `control` topic.
- `alarm`:            Produce an alarm message to the `control` topic.

## Defined Format

In `defined` mode, the behavior is given by the simulator object in the user-defined json file.
The file defines a sequence of actions which run one after the other.

The structure of the simulator object is:

### Simulator

- voltage-transformation: [`Transformation`](#Transformation)
- time_bins: `Integer`,
- sample_rate: `Integer`,
- digitiser_config: `DigitiserConfig`(#DigitiserConfig)
- event_lists: [`[EventListTemplate]`](#EventListTemplate)
- pulses: [`[PulseTemplate]`](#PulseAttributes)
- schedule: [`[Action]`](#DigitiserConfig)

### DigitiserConfig

### EventListTemplate
- pulses: [`[EventPulseTemplate]`],
- noises: [`[NoiseSource]`],
- num_pulses: IntRandomDistribution,

### PulseTemplate

### Action

An `Action` is one of the following
- `comment`: [`String`],
- `tracing-event`: [`TracingEvent`],
- `wait-ms`: [`Integer (U32)`],

- `send-run-start`

   - `name`: [`String`]
   - `instrument`: [`String`]

- `send-run-stop`

   - `name`: [`String`]

- `send-run-log-data`
- `send-sample-env-log`
- `send-alarm`
- `frame-loop`

   - `start`: [`Integer (u32)`]
   - `end`: [`Integer (U32)`]
   - `schedule`: [`[FrameAction]`]
- `set-timestamp`: [`Timestamp`],
- `set-veto-flags`: [`Integer (u16)`],
- `set-period`: [`Integer (u64)`],
- `set-protons-per-pulse`: [`Integer (u8)`],
- `set-running`: [`Bool`],

- `generate-trace`

   - `template-index`: [`Integer (u32)`],
   - `repeat`: [`Integer (u32)`],
- `generate-event-list`
   - `template-index`: [`Integer (u32)`],
   - `repeat`: [`Integer (u32)`],

### FrameAction

A `FrameAction` is one of the following:

- `comment`: [`String`],
- `tracing-event`: [`TracingEvent`],
- `wait-ms`: [`Integer (U32)`],

- `send-aggregated-frame-event-list`

   - `source-options`: [`SourceOptions`],
   - `channel-indices`: [`Interval<usize>`],

- `digitiser-loop`

   - `start`: [`Integer (u32)`]
   - `end`: [`Integer (U32)`]
   - `schedule`: [`[DigitiserAction]`]
- `set-timestamp`: [`Timestamp`],

- `generate-trace`

   - `template-index`: [`Integer (u32)`],
   - `repeat`: [`Integer (u32)`],

- `generate-event-list`

   - `template-index`: [`Integer (u32)`],
   - `repeat`: [`Integer (u32)`],

### DigitiserAction

A `DigitiserAction` is one of the following:

- `comment`: [`String`],
- `tracing-event`: [`TracingEvent`],
- `wait-ms`: [`Integer (U32)`],
- `send-digitiser-trace`: [`pop-front`|`replace-random`],
- `send-digitiser-event-list`: [`SourceOptions`],
- `generate-trace`
   - `template-index`: [`Integer (u32)`],
   - `repeat`: [`Integer (u32)`],
- `generate-event-list`
   - `template-index`: [`Integer (u32)`],
   - `repeat`: [`Integer (u32)`],

### Digitizer

- id : `Integer`
- channels : [`Interval`](#Interval)

### Pulse

- weight : `Float`
- attributes : [`PulseAttributes`](#PulseAttributes)

### PulseAttributes

A `PulseAttributes` object is one of the following

- Gaussian
   - pulse-type = "gaussian"
   - peak_height : [`RandomDistribution`](#RandomDistribution)
   - peak_time : [`RandomDistribution`](#RandomDistribution)
   - sd : [`RandomDistribution`](#RandomDistribution)

   ```json
    {
        "pulse-type": "gaussian",
        "peak_height": { "random-type": "uniform", "min": { "fixed-value": 30 }, "max": { "fixed-value": 70 }},
        "peak_time": { "random-type": "exponential", "lifetime": { "fixed-value": 2200 }},
        "sd": { "random-type": "uniform", "min": { "fixed-value": 5 }, "max": { "fixed-value": 20 }}
    }
    ```

- Biexp
   - type = "biexp"
   - start : [`RandomDistribution`](#RandomDistribution)
   - peak_height : [`RandomDistribution`](#RandomDistribution)
   - decay : [`RandomDistribution`](#RandomDistribution)
   - rise : [`RandomDistribution`](#RandomDistribution)

   ```json
   {
        "type": "biexp",
        "start" : { "random-type": "exponential", "lifetime" : { "fixed-value": 2200 }},
        "peak_height": { "random-type": "uniform", "min": { "fixed-value": 30 }, "max": { "fixed-value": 70 }},
        "decay": { "random-type": "uniform", "min": { "fixed-value": 5 }, "max": { "fixed-value": 10 }},
        "rise": { "random-type": "uniform", "min": { "fixed-value": 15 }, "max": { "fixed-value": 20 }}
   }
   ```

### NoiseSource

- bounds : [`Interval`](#Interval)
- attributes : [`NoiseAttributes`](#NoiseAttributes)
- smoothing-factor : [`Expression`](#Expression)

### NoiseAttributes

A NoiseSource object is one of the following

- Gaussian
   - type = "gaussian"
   - mean : [`RandomDistribution`](#RandomDistribution)
   - sd : [`RandomDistribution`](#RandomDistribution)
- Uniform
   - type = "uniform"
   - min : [`RandomDistribution`](#RandomDistribution)
   - max : [`RandomDistribution`](#RandomDistribution)

### Interval

- min : `Integer`
- max : `Integer`

### RandomDistribution

A Distribution object is one of the following

- Float
   - value : [`Expression`](#Expression)

   ```json
   { "random-type": "constant", "value": { "fixed-value": 40.0 } }
   ```

- Uniform
   - min : [`Expression`](#Expression)
   - max : [`Expression`](#Expression)

   ```json
   { "random-type": "uniform", "min" : { "fixed-value": 20.0 }, "max" : { "fixed-value": 60.0 }}
   ```

- Gaussian
   - mean : [`Expression`](#Expression)
   - sd : [`Expression`](#Expression)

   ```json
   { "random-type": "gaussian", "mean" : { "fixed-value": 40.0 }, "sd" : { "fixed-value": 20.0 }}
   ```

- Exponential
   - lifetime : [`Expression`](#Expression)
      - The mean value is equal to the lifetime parameter, however note that the exponential distribution is usually given by the `rate` parameter which is `1/lifetime`.

      ```json
      { "random-type": "exponential", "lifetime" : { "fixed-value": 40.0 }}
      ```

### Expression

An Expression object is one of the following

- Fixed
   - This distribution is a constant value

   ```json
   { "fixed-value": 40.0 }
   ```

- FrameTransform
   - frame-transform : [`Transformation`](#Expression)

   ```json
   { "frame-transform": { "scale": 50, "translate": 50 } }
   ```

### Transformation

- scale : `Float`
- translation : `Float`
