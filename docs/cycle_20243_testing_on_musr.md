# Cycle 2024/3 testing on MuSR

## Kubernetes based deployment of pipeline components

Verify that:

- Pipeline components can be created and function correctly once started (i.e. start, and process data from Kafka)
- Relevant components (i.e. event formation) can be scaled and message processing functions as expected (i.e. messages are processed only once and distributed across all instances)
- Pipeline components can be terminated without failure

## Prometheus based monitoring

Verify that:

- Redpanda metrics are collected
- Metrics of pipeline components are collected
- Metrics are written correctly to InfluxDB via Telegraf
