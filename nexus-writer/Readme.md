# nexus-writer

## Introduction
This program listens to a command-line specified kafka broker for run control, and event list messages and creates and writes event lists into NeXus files accordingly.
One NeXus file per run is written, with all event lists whose timestamp falls within the run's start and stop time.
If a run stop message has not yet been received, then all event list messages whose timestamp comes after the run's start time, are incorperated.

## Command Line Interface
For command line instructions run
```
nexus-writer --help
```

### Options
Given a run name of `run-name`, then the file for this run is saved at `file-name/run-name.nxs`.

The `cache-run-ttl-ms` parameter specifies how long a terminated run should be kept in memory before being flushed (removed from memory). This is to allow delayed event-list messages to be collected. If none is specified then a default time of 2000ms is used.

The `cache-poll-interval-ms` parameter specifies how often the program will check that a run is ready to be flushed. If none is specified then a default interval of 200ms is used.

If the options `digitiser-event-topic`, `frame-event-topic`, or `histogram-topic` are specified, then the program will listen on the given topics for
the types `DigitizerEventListMessage`, `FrameAssembledEventListMessage`, or `HistogramMessage` respectively.

The mandatory parameter `control-topic` specifies which topic to listen for run start and run stop messages.

### Example
The following script runs the nexus-writer program as a backgroud process, and listens for frame-event messages on topic `FrameEvents`. Runs are saved in the folder `./output/Saves/...`.
```
nexus-writer --broker localhost:19092 --consumer-group nexus-writer \
    --control-topic Controls --frame-event-topic FrameEvents \
    --cache-run-ttl-ms 4000 \
    --file-name output/Saves &
```

## Behaviour
The program assumes that:
 - the first control message it will receive is a `RunStart`,
 - that control messages will alternate between `RunStart` and `RunStop`, and
 - for each `RunStart` the following `RunStop` will have the correct run name.

The program has a cache in which runs are kept in memory.
When a run is created, it is said to be 'ongoing'. It remains 'ongoing' until the corresponding `RunStop` message is consumed, at which point the run is said to be 'terminated'.

If a `RunStart` is consumed from the control topic, then
- If there are no runs in memory, or the last run in memory is terminated, then create a new run and push it to memory,
- If there are runs in memory, and the last run is ongoing, then an error message is printed and no new run is created.

When a `FrameAssembledEventListMessage` is produced on topic `frame-event-topic` (or the equivalent for digitiser event messages),
the program does the following:
- Consumes the message and,
- If there are no runs in memory, discard the message
- If there are runs, but none of them have a valid time-range for the message, discard the message
- If a run is found in memory with a valid time-range for the message, then:
    - Write the message to the run's NeXus file,
    - Update the run's `last_modified` field to the present time

If a `RunStop` is consumed from the control topic, then
- If there are no runs in memory, or the last run is terminated, then an error message is printed, the `RunStop` discarded.
- If there are runs in memory, and the last run is ongoing, then:
    - this run is terminated and
    - its `last_modified` field is updated to the present time
    - note that at present there is no check that the `RunStop` run name matches the ongoing `RunStart` run name, this may be changed in the future.

A timer is set to tick on intervals of `cache-poll-interval-ms`.
On each tick, any run in memory is removed if:
    - it has been terminated, and
    - at least `cache-run-ttl-ms` of time has passed since the time in its `last_modified` field,
This allows event-list messages to catch up with its run if it has been delayed in the pipeline.