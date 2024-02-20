# trace-telemetry-exporter

A tool which uses the OpenMetrics format to record the following:

| Name                                | Description                                                   | Labels                         |
|-------------------------------------|---------------------------------------------------------------|--------------------------------|
| digitiser_message_received_count    | The number of messages received by a digitiser.               | `digitiser_id`                 |
| digitiser_last_message_timestamp    | The timestamp of the last message received by a digitiser.    | `digitiser_id`                 |
| digitiser_last_message_frame_number | The frame number of the last message received by a digitiser. | `digitiser_id`                 |
| digitiser_channel_count             | The number of channels in a digitiser.                        | `digitiser_id`                 |
| digitiser_sample_count              | The number of samples in a channel of a digitiser             | `digitiser_id` `channel_index` |
