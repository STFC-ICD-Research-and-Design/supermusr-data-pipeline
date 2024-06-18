# 5. Use OpenMetrics

Date: 2024-06-17

## Status

Accepted

(note that this ADR was written retrospectively)

## Context

The SuperMuSR data pipeline is composed of many components, each of them having task specific measures of function/success.

To help ensure the digitisers and data pipeline are operating as expected, such measures should be made available for alerting, monitoring and logging.

## Decision

Each component should expose relevant metrics that effectively describe how well it is functioning via the textual [OpenMetrics](https://openmetrics.io/) format.

Relevant metrics will differ between components, however given the low cost of reporting anything that *could* be a useful indicator or diagnostic item should be considered in scope.

## Consequences

- Each pipeline component will provide text format metrics via HTTP
