# trace-reader

## Introduction

This program reads picscope .trace files, a binary file developed by E.M Schooneveld.

## Command Line Interface

For command line instructions run

```shell
trace-reader --help
```

### Options

The `number-of-trace-events` parameter is the number of traces events that are extracted from the file (either randomly or in sequence). The file defines how many channels the digitizer has, each trace event contains one trace for each channel. The number of messages dispatched is equal to the number of trace events times the number of channels.

If `random-sample` is set then trace-events are read from the file randomly. Selection is made with replacement so duplication is possible.
If this flag is not set then trace-events are read in order.
If `number-of-trace-events` is greater than the number available then trace-events are the reader wraps around to the beginning of the file as often as necessary.

### Example

The following reads 18 trace events (sampled randomly with replacement) from `Traces/MuSR_A41_B42_C43_D44_Apr2021_Ag_ZF_IntDeg_Slit60_short.trace` and dispatches them to the topic `Traces` on the Kafka broker located at `localhost:19092`:

```shell
trace-reader --broker localhost:19092 --consumer-group trace-producer --trace-topic Traces --file-name Traces/MuSR_A41_B42_C43_D44_Apr2021_Ag_ZF_IntDeg_Slit60_short.traces --number-of-trace-events 18 --random-sample
```

## Terminology

- Trace: This is continous block of voltage readings from a digitizer channel.
- Trace Event: This is a collection of traces, one for each channel on the digitizer.

Note that "Event" here is meant in a different sense than the trace-to-event tool. The overlap is a result of terminology used in the .trace file. To avoid confusion, the term "Trace Event" is used here.
