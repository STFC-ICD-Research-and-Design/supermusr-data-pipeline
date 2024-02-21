# trace-telemetry-exporter

A tool which uses the OpenMetrics format to record the following:

| Name                                | Description                                                                                                  | Labels                         |
|-------------------------------------|--------------------------------------------------------------------------------------------------------------|--------------------------------|
| digitiser_message_received_count    | The number of messages received from a digitiser.                                                            | `digitiser_id`                 |
| digitiser_last_message_timestamp    | The timestamp of the last message received from a digitiser (expressed in nanoseconds since the Unix Epoch). | `digitiser_id`                 |
| digitiser_last_message_frame_number | The frame number of the last message received from a digitiser.                                              | `digitiser_id`                 |
| digitiser_channel_count             | The number of channels in the last message received from a digitiser.                                        | `digitiser_id`                 |
| digitiser_sample_count              | The number of samples in a channel belonging to the last message received from a digitiser.                  | `digitiser_id` `channel_index` |
