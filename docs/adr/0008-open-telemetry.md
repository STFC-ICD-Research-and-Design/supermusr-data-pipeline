# 8. open-telemetry

Date: 2024-08-21

## Status

Accepted

(note that this ADR was written retrospectively)

## Context

In software, tracing is used to track cause and effect within the flow a program by recording logs, events and spans, which are collected and analysed.
For software involving multiple isolated processes, a tool is required to combine the spans emitted by the different processes and integrate them into consistent traces.
OpenTelemetry is a standard for accomplishing this which has implementations in many different languages (including rust) and is widely used.
It integrates nicely with the `tracing` crate and Kafka.
Third party collectors and analysis tool exist such as Jaeger.

## Decision

1. Include, in the `common` crate, the following dependencies (underscore is deliberate):
    1. `opentelemetry`
    2. `opentelemetry-otlp`
    3. `opentelemetry_sdk`
    4. `tracing-opentelemetry`
    5. `tracing-subscriber`
2. Develop structs, functions, and macros in the `common` crate that allows traces to be embedded in the headers of Kafka messages, so they can be used in upstream components.
3. Design structs, functions, and macros in the `common` crate such that none of the dependencies above need to be included in any other component, and modifications to existing compnents are minimal and can be implemented logically.
4. Tracing subscribers should be initialised by the above code using a macro called `init_tracer`. These should include the `stdout` subscriber, whose filtering is controlled by the `RUST_LOG` environment variable, and an OpenTelemetry subscriber which is controlled by command line arguments.

## Consequences

1. Third-party tool such as Jaeger can be used to collect and analyse tracing data from the whole pipeline.
2. Any future component should extract traces from Kafka headers when consuming messages, and embed them when producing.
3. Any future component should initialise its subscribers using the `init_tracer` macro (even if it does not used OpenTelemetry).
