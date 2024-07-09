# trace-telemetry-exporter

## Introduction

A tool which uses the OpenMetrics format to record the following:

| Name                                | Description                                                                                                  | Labels                         |
|-------------------------------------|--------------------------------------------------------------------------------------------------------------|--------------------------------|
| digitiser_message_received_count    | The number of messages received from a digitiser.                                                            | `digitiser_id`                 |
| digitiser_last_message_timestamp    | The timestamp of the last message received from a digitiser (expressed in nanoseconds since the Unix Epoch). | `digitiser_id`                 |
| digitiser_last_message_frame_number | The frame number of the last message received from a digitiser.                                              | `digitiser_id`                 |
| digitiser_channel_count             | The number of channels in the last message received from a digitiser.                                        | `digitiser_id`                 |
| digitiser_sample_count              | The number of samples in a channel belonging to the last message received from a digitiser.                  | `digitiser_id` `channel_index` |


## Command Line

The program is executed from the command line, for instance:

```shell
trace-archiver --broker localhost:19092 --group g1 --trace-topic trace_in
```

For detailed instructions about each parameter run

```shell
trace-archiver --help
```
