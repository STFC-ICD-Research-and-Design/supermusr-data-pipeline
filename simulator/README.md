# Simulator

## Introduction

## Command Line Interface

```shell
simulator [OPTIONS] --broker <BROKER> [COMMAND]
```

For instance:

```shell
simulator --broker localhost:19092 \
    defined \
    --digitiser-trace-topic Traces \
    --digitiser-event-topic SimulatedEvents \
      --path "trace.json" \
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

### Top-Level Simulator

The structure of the top-level object is:

- voltage-transformation: [`Transformation`](#Transformation)
- time-bins: `Integer`,
- sample-rate: `Integer`,
- digitiser-config: [`DigitiserConfig`](#DigitiserConfig)
- event-lists: [`[EventListTemplate]`](#EventListTemplate)
- pulses: [`[PulseTemplate]`](#PulseTemplate)
- schedule: [`[Action]`](#Action)

```json
{
    "voltage-transformation": {"scale": 1, "translate": 0 },
    "time-bins": 30000,
    "sample-rate": 1000000000,
    "digitiser-config": {
        "auto-digitisers": {
            "num-digitisers": 32,
            "num-channels-per-digitiser": 8
        }
    },
    "pulses" : [PulseTemplate],
    "event-lists" : [EventListTemplate],
    "schedule" : [Action],
}
```

### DigitiserConfig

Configuring the digitisers and channels must be done prior to sending any messages trace or event messages.

#### Automatically Assign Channels

The following configuration creates the given number of channels, with ids equal to the channel index. No digitisers are created, so this must not be used if [`send-digitiser-event-list`](#digitiseraction-senddigitisereventlist) or [`send-digitiser-trace`](#digitiseraction-senddigitisertrace) actions are used.

```json
"digitiser-config": {
      "auto-aggregated-frame": {
         "num-channels": Integer
   }
}
```

#### Manually Assign Channels

This configuration allows you to manually specify which channel ids are created. No digitisers are created, so this must not be used if [`send-digitiser-event-list`](#digitiseraction-senddigitisereventlist) or [`send-digitiser-trace`](#digitiseraction-senddigitisertrace) actions are used.

```json
"digitiser-config": {
   "manual-aggregated-frame": {
      "channels": [Integer]
   }
}
```

#### Automatically Assign Digitisers and Channels

This configuration creates the given number of digitisers and assigns the given number channels to each one.
Digitiser ids are equal to the digitiser index, and channel ids are given by the channel index offset by `digitiser-id * num-channels-per-digitiser`.

```json
"digitiser-config": {
   "auto-digitisers": {
      "num-digitisers": Integer,
      "num-channels-per-digitiser": Integer
   }
}
```

#### Manually Assign Digitisers and Channels

This configuration allows you to manually specify which digitisers are created, and what channel ids they have.

```json
"ManualDigitisers-digitiser-config": {
        "auto-digitisers": {
            "num-digitisers": 32,
            "num-channels-per-digitiser": 8
        }
    }
```

### PulseTemplate

A pulse template defines a pulse that can be referenced in an event list template. A pulse template can be one of the following:

- Flat
   - pulse-type = "flat"
   - start : [`FloatRandomDistribution`](#FloatRandomDistribution)
   - width : [`FloatRandomDistribution`](#FloatRandomDistribution)
   - height : [`FloatRandomDistribution`](#FloatRandomDistribution)

- Triangular
   - pulse-type = "triangular"
   - start : [`FloatRandomDistribution`](#FloatRandomDistribution)
   - width : [`FloatRandomDistribution`](#FloatRandomDistribution)
   - peak-time : [`FloatRandomDistribution`](#FloatRandomDistribution) (between 0 and 1, pulse peak occurs at `start + peak-time*width` )
   - height : [`FloatRandomDistribution`](#FloatRandomDistribution)

   ```json
   {
      "pulse-type": "triangular",
      "start": {
         "random-type": "exponential",
         "lifetime": {
            "float": 2200
         }
      },
      "width": {
         "random-type": "uniform",
         "min": {
            "float": 20
         },
         "max": {
            "float": 50
         }
      },
      "peak_time": {
         "random-type": "uniform",
         "min": {
            "float": 0.25
         },
         "max": {
            "float": 0.75
         }
      },
      "height": {
         "random-type": "uniform",
         "min": {
            "float": 30
         },
         "max": {
            "float": 70
         }
      }
   }
   ```

- Gaussian
   - pulse-type = "gaussian"
   - peak_height : [`FloatRandomDistribution`](#FloatRandomDistribution)
   - peak_time : [`FloatRandomDistribution`](#FloatRandomDistribution)
   - sd : [`FloatRandomDistribution`](#FloatRandomDistribution)

   ```json
   {
      "pulse-type": "gaussian",
      "peak_height": {
         "random-type": "uniform",
         "min": {
            "float": 30
         },
         "max": {
            "float": 70
         }
      },
      "peak_time": {
         "random-type": "exponential",
         "lifetime": {
            "float": 2200
         }
      },
      "sd": {
         "random-type": "uniform",
         "min": {
            "float": 5
         },
         "max": {
            "float": 20
         }
      }
   }
   ```

- Biexp
   - type = "biexp"
   - start : [`FloatRandomDistribution`](#FloatRandomDistribution)
   - peak_height : [`FloatRandomDistribution`](#FloatRandomDistribution)
   - decay : [`FloatRandomDistribution`](#FloatRandomDistribution)
   - rise : [`FloatRandomDistribution`](#FloatRandomDistribution)

   ```json
   {
      "type": "biexp",
      "start": {
         "random-type": "exponential",
         "lifetime": {
            "float": 2200
         }
      },
      "peak_height": {
         "random-type": "uniform",
         "min": {
            "float": 30
         },
         "max": {
            "float": 70
         }
      },
      "decay": {
         "random-type": "uniform",
         "min": {
            "float": 5
         },
         "max": {
            "float": 10
         }
      },
      "rise": {
         "random-type": "uniform",
         "min": {
            "float": 15
         },
         "max": {
            "float": 20
         }
      }
   }
   ```

### NoiseSource

- bounds : [`Interval`](#Interval)
- attributes : [`NoiseAttributes`](#NoiseAttributes)
- smoothing-factor : [`FloatExpression`](#FloatExpression)

### NoiseAttributes

A NoiseSource object is one of the following

- Gaussian
   - type = "gaussian"
   - mean : [`FloatRandomDistribution`](#FloatRandomDistribution)
   - sd : [`FloatRandomDistribution`](#FloatRandomDistribution)
- Uniform
   - type = "uniform"
   - min : [`FloatRandomDistribution`](#FloatRandomDistribution)
   - max : [`FloatRandomDistribution`](#FloatRandomDistribution)

### Interval

- min : `Integer`
- max : `Integer`

### IntRandomDistribution

This discrete integer distribution object, is one of the following

- Constant
   - value : [`IntExpression`](#IntExpression)

   ```json
   {
      "random-type": "constant",
      "value": {
         "int": 40
      }
   }
   ```

- Uniform
   - min : [`IntExpression`](#IntExpression)
   - max : [`IntExpression`](#IntExpression)

   ```json
   {
      "random-type": "uniform",
      "min": {
         "int": 20
      },
      "max": {
         "int": 60
      }
   }
   ```

### IntExpression

An Expression object is one of the following

- Int
   - This expression is a constant value

   ```json
   {
      "int": 40
   }
   ```

- IntEnv
   - Extracts value from an environment variable (panics if not available or invalid)

   ```json
   {
      "int-env": String
   }
   ```

- IntFunc
   - This expression calculates the value from a linear transform upon the current frame number

   ```json
   {
      "int-func": {
         "scale": 50,
         "translate": 50
      }
   }
   ```

### FloatRandomDistribution

A continuous floating point distribution object is one of the following

- Constant
   - value : [`FloatExpression`](#FloatExpression)

   ```json
   {
      "random-type": "constant",
      "value": {
         "float": 40.0
      }
   }
   ```

- Uniform
   - min : [`FloatExpression`](#FloatExpression)
   - max : [`FloatExpression`](#FloatExpression)

   ```json
   {
      "random-type": "uniform",
      "min": {
         "float": 20.0
      },
      "max": {
         "float": 60.0
      }
   }
   ```

- Gaussian
   - mean : [`FloatExpression`](#FloatExpression)
   - sd : [`FloatExpression`](#FloatExpression)

   ```json
   {
      "random-type": "gaussian",
      "mean": {
         "float": 40.0
      },
      "sd": {
         "float": 20.0
      }
   }
   ```

- Exponential
   - lifetime : [`FloatExpression`](#FloatExpression)
      - The mean value is equal to the lifetime parameter, however note that the exponential distribution is usually given by the `rate` parameter which is `1/lifetime`.

      ```json
      {
         "random-type": "exponential",
         "lifetime" : {
            "float": 40.0
         }
      }
      ```

### FloatExpression

An Expression object is one of the following

- Float
   - This expression is a constant value

   ```json
   {
      "float": 40.0
   }
   ```

- FloatEnv
   - Extracts value from an environment variable (panics if not available or invalid)

   ```json
   {
      "float-env": String
   }
   ```

- FloatFunc
   - This expression calculates the value from a linear transform upon the current frame number

   ```json
   {
      "float-func": {
         "scale": 50,
         "translate": 50
      }
   }
   ```

### Transformation

- scale : `Float`
- translation : `Float`

### EventListTemplate

`Pulses` is a list of references (by index) to pulses defined in the top-level [`simulator`](#top-level-simulator) object,
along with a `weight` value which defines the likelihood of that pulse being chosen.

- pulses,
   - weight : Float
   - pulse-index : Integer
- noises: [`[NoiseSource]`],
- num_pulses: [IntRandomDistribution](#IntRandomDistribution),

```json
{
  "pulses": [
    {
      "weight": 1,
      "pulse-index": 0
    },
    {
      "weight": 1,
      "pulse-index": 1
    }
  ],
  "noises": [],
  "num-pulses": {
    "random-type": "constant",
    "value": {
      "int": 50
    }
  }
}
}
```

### Action

An `Action` is one of the following

#### Comment

```json
{
   "comment": "This action does nothing"
}
```

#### TracingEvent

Emits a tracing event of the given level and message, either at info, or debug level.

```json
{
  "tracing-event": {
    "level": "info",
    "message": "An info tracing event"
  }
}
```

```json
{
   "tracing-event": {
      "level": "debug",
      "message": "A debug tracing event"
   }
}
```

#### WaitMs

Pauses the simulation's schedule for the given milliseconds. This does not interupt other threads.

```json
{
   "wait-ms": 20
}
```

#### SendRunStart

Sends a `RunStart` message to the topic `control-topic` specified in the Cli.

- `name`: [`String`]
- `instrument`: [`String`]

#### SendRunStop

Sends a `RunStop` message to the topic `control-topic` specified in the Cli.

- `name`: [`String`]

#### SendRunLogData

Sends a `LogData` message to the topic `runlog-topic` specified in the Cli.
`value-type` should be one of `"int8", "int16", "int32", "int64", "uint8", "uint16", "uint32", "uint64", "float32", "float64"`.

```json
{
   "source-name" : String,
   "value-type" : String,
   "value" : [String]
}
```

#### SendSampleEnvLog

Sends a `SampleEnvironmentData` message to the topic `selog-topic` specified in the Cli.
`values-type` should be one of `"int8", "int16", "int32", "int64", "uint8", "uint16", "uint32", "uint64", "float32", "float64"`.
`location` should be one of `"unknown", "start", "middle", "end"`.

```json
{
   "name" : String,
   "values-type" : String,
   "location": String,
   "values" : [String]
}
```

#### SendAlarm

Sends a `Alarm` message to the topic `alarm-topic` specified in the Cli.
`severity` should be one of `"OK", "MINOR", "MAJOR", "INVALID"`.

```json
{
   "source-name" : String,
   "severity" : String,
   "message": String
}
```

#### SetTimestamp

Changes the timestamp in the global metadata. Can be one of:

```json
{
  "set-timestamp": {
    "advance-by-ms": 20
  }
}
```

to advance the timestamp by the given milliseconds, or

```json
{
   "set-timestamp": "now"
}
```

to set the timestamp to the current system time.

#### SetVetoFlags

Sets the veto flags in the global metadata.

```json
{
   "set-veto-flags": Integer(u16)
}
```

#### SetPeriod

Sets the period in the global metadata.

```json
{
   "set-period": Integer(u64)
}
```

#### SetProtonsPerPulse

Sets the protons per pulse field in the global metadata.

```json
{
   "set-protons-per-pulse": Integer(u8)
}
```

#### SetRunning

Sets the running flag in the global metadata.

```json
{
   "set-running": bool
}
```

#### GenerateTrace

This action creates the given number of traces and stores them in the trace cache.
Once created these can be emitted by the [DigitiserAction](#DigitiserAction) ["send-digitiser-trace"](#digitiseraction-SendDigitiserTrace).

- `template-index`: [`Integer`],
- `repeat`: [`Integer`],

`template-index` refers to the index of the EventListTemplate in the top-level [simulator](#top-level-simulator) object, `repeat` is the number of traces to add to the trace cache.

```json
{
  "generate-trace": {
    "event-list-index": 0,
    "repeat": 8
  }
}
```

#### GenerateEventList

This action creates the given number of event lists and stores them in the event list cache.
Once created these can be emitted by the [DigitiserAction](#DigitiserAction) [`send-digitiser-event-list`](#SendDigitiserEventList) or the [FrameAction](#FrameAction) ][`send-aggregated-frame-event-list`](#SendAggregatedFrameEventList).

- `template-index`: [`Integer`],
- `repeat`: [`Integer`],

`template-index` refers to the index of the EventListTemplate in the top-level [simulator](#top-level-simulator) object, `repeat` is the number of event-lists to add to the event-lists cache.

```json
{
  "generate-event-list": {
    "event-list-index": 0,
    "repeat": 8
  }
}
```

#### FrameLoop

This is a loop in which [FrameAction](#frameaction) events can be scheduled. The Frame Number in the global metadata is controled using this action.

- `start`: [`Integer (u32)`]
- `end`: [`Integer (U32)`]
- `schedule`: [`[FrameAction]`]

```json
{
   "frame-loop": {
      "start": 0,
      "end": 8,
      "schedule": []
   }
}
```

### FrameAction

A `FrameAction` is one of the following:

#### FrameAction: Comment

Frame Comment behaves the same as in [Comment](#Comment).

#### FrameAction: TracingEvent

Frame TracingEvent behaves the same as in [TracingEvent](#TracingEvent).

#### FrameAction: WaitMs

Frame WaitMs behaves the same as in [WaitMs](#WaitMs).

#### FrameAction: SetTimestamp

Frame SetTimestamp behaves the same as in [SetTimestamp](#SetTimestamp).

#### FrameAction: GenerateTrace

Frame GenerateTrace behaves the same as in [GenerateTrace](#GenerateTrace).

#### FrameAction: GenerateEventList

Frame GenerateEventList behaves the same as in [GenerateEventList](#GenerateEventList).

#### FrameAction: SendAggregatedFrameEventList

- `source-options`: [`SourceOptions`],
- `channel-indices`: [`Interval<usize>`],

Sends an `FrameAssembledEventList` message to the topic `frame-event-topic` specified in the Cli. Can be one of the following

```json
{
   "send-aggregated-frame-event-list": {
      "source": "no-source",
      "channel-indices": {
         "min": Integer,
         "max": Integer
      }
   }
}
```

To send empty event lists for channels with the given index range.

```json
{
   "send-aggregated-frame-event-list": {
      "source": {
         "select-from-cache": "pop-front"
      },
      "channel-indices": {
         "min": Integer,
         "max": Integer
      }
   }
}
```

To send cached event lists for channels with the given index range.
In this case the cached event lists are popped from the front of the cache as they are sent.
This option removes all cached event lists that are selected.

```json
{
   "send-aggregated-frame-event-list": {
      "source": {
         "select-from-cache": "replace-random"
      },
      "channel-indices": {
         "min": Integer,
         "max": Integer
      }
   }
}
```

To send cached event lists for channels with the given index range.
In this case the cached event lists are selected with replacement at random from the cache.
This option does not remove any cached event lists.

#### DigitiserLoop

This is a loop in which DigitiserActions events can be scheduled.
This should only be used if digitisers have been configured by the [DigitiserConfig](#DigitiserConfig) field.
The Digitiser Id is controled using this action. Note, the index refers to the indices of the configured digitisers (not their Ids directly).

- `start`: [`Integer (u32)`]
- `end`: [`Integer (U32)`]
- `schedule`: [`[DigitiserAction]`]

```json
{
   "digitiser-loop": {
      "start": 0,
      "end": 8,
      "schedule": []
   }
}
```

### DigitiserAction

A `DigitiserAction` is one of the following:

#### DigitiserAction: Comment

Digitiser Comment behaves the same as in [Comment](#Comment).

#### DigitiserAction: TracingEvent

Digitiser TracingEvent behaves the same as in [TracingEvent](#TracingEvent).

#### DigitiserAction: WaitMs

Digitiser WaitMs behaves the same as in [WaitMs](#WaitMs).

#### DigitiserAction: GenerateTrace

Digitiser GenerateTrace behaves the same as in [GenerateTrace](#GenerateTrace).

#### DigitiserAction: GenerateEventList

Digitiser GenerateEventList behaves the same as in [GenerateEventList](#GenerateEventList).

#### DigitiserAction: SendDigitiserTrace

Sends an `DigitizerAnalogTrace` message to the topic `trace-topic`, specified in the Cli. Can be one of the following

```json
{
   "send-digitiser-trace": "pop-front"
}
```

To send cached traces for the current digitiser, this selects the appropriate number of traces for the channels in this digitiser. In this case the cached traces are popped from the front of the cache as they are sent. This option removes all cached traces that are selected.

```json
{
   "send-digitiser-trace": "replace-random"
}
```

To send cached traces for the current digitiser, this selects the appropriate number of traces for the channels in this digitiser. In this case the cached traces are selected with replacement at random from the cache. This option does not remove any cached traces.

#### DigitiserAction: SendDigitiserEventList

Sends an `DigitizerEventList` message to the topic `event-topic`, specified in the Cli. Can be one of the following

```json
{
   "send-digitiser-event-list": "no-source"
}
```

to send empty event lists for channels of the current digitiser.

```json
{
   "send-digitiser-event-list": {
      "select-from-cache" : "pop-front"
   }
}
```

to send cached event lists for the current digitiser, this selects the appropriate number of event lists for the channels in this digitiser. In this case the cached event lists are popped from the front of the cache as they are sent. This option removes all cached event lists that are selected.

```json
{
   "send-digitiser-event-list": {
      "select-from-cache": "replace-random"
   }
}
```

to send cached event lists for the current digitiser, this selects the appropriate number of event lists for the channels in this digitiser. In this case the cached event lists are selected with replacement at random from the cache. This option does not remove any cached event lists.
