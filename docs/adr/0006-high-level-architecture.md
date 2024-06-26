# 6. High level architecture

Date: 2024-06-26

## Status

Accepted

(note that this ADR was written retrospectively)

## Context

The SuperMuSR data pipeline is a distributed collection of processing and data management steps.
These steps should be clearly designated to ensure the creation of minimal, UNIX philosophy compliant tools for manipulating streamed data as well as clearly indicating the interface between them and other components of the overall data acquisition system.

## Decision

The overall/high level architecture will follow what is outlined in [`pipeline_v2.drawio`](../pipeline_v2.drawio).

## Consequences

- The architecture is well defined, with clear interfaces and areas of responsibility.
- The diagram and documentation are to be considered living documents and can be amended as required.
