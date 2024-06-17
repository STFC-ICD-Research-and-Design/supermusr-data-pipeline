# 2. Use ESS-like architecture for streaming components

Date: 2024-06-17

## Status

Accepted

(note that this ADR was written retrospectively)

## Context

The desire is to adopt a streaming based architecture for SuperMuSR.

This has the following benefits:

- allows flexible/modular manipulation of data "in flight" (i.e. before a file is written to the archive)
- provides a means of rapid iteration on signal processing
- increases recoverability in the event of a single component failure

## Decision

Adopt an ESS like architecture, along with appropriate technology decisions.

Specifically:

- Data will be exchanged via Apache Kafka with a broker
- The format of the exchanged data will be defined and encoded using Google Flatbuffers

SuperMuSR specific items/differences:

- The broker will be Redpanda
- Some additional schemas will be required
- More in flight processing will be needed, this will act as a consumer and producer, publishing the transformed data back to the broker on an appropriate topic

## Consequences

- Redpanda broker now part of physical/infrastructure architecture
- Interface between digitisers and software components is further defined
