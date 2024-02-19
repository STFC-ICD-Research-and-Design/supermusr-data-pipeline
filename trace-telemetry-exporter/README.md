# trace-telemetry-exporter

A tool which uses the OpenMetrics format to record the following:

### Digitiser message received count.
- Labels: `digitiser_message_received_count` (dig. ID)

### Digitiser last message timestamp.
- Labels: `digitiser_last_message_timestamp` (dig. ID)

### Digitiser last message frame number.
- Labels: `digitiser_last_message_frame_number` (dig. ID)

### Digitiser channel count.
- Labels: `digitiser_channel_count` (dig. ID)

### Digitiser sample count.
- Labels: `digitiser_sample_count` (dig. ID) (channel index)
- Description: The number of samples in a particular channel of a certain digitiser.
