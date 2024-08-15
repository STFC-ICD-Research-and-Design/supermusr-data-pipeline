# Policy on using Tracing and OpenTelemetry in the SuperMuSR Data Pipeline

This document describes how tracing is used in the SuperMuSR Data Pipeline, it is not intended as an introduction to tracing or the tracing crate. Please see appropriate documentation or tutorials if necessary.

## Introduction

As the data pipeline works via the interaction of several independent processes, communicating via a common broker, determining cause and effect can be tricky.

### Terminology

|Term|Definition|
|---|---|
|Tracing|This refers both to the technique of collecting structured, hierarchical and temporal data from software, as well as the rust crate which implements it|
|OpenTelemetry|This is a layer of software which collects tracing data from different processes and allows them to be linked causally.|
|Jaeger|This is a third-party `collector` of tracing data that can collect, store and display traces in a meaningful manner.|
|Trace|In OpenTelemetry a trace is a rooted tree of spans, however in general, a `trace` is either a span or an event. |
|Span|A span is an interval of time in which a program does some work, for instance, a function execution. Spans are used to track the flow of data through the pipeline.|
|Event|A singlular moment within a span in which an event occured, also called a `log`. Events are mostly used to record error messages.|
|Field|A key/value pair which can be a propery of a span or event. For instance metadata of received flatbuffer messages are recorded in fields.|
|Subscriber|A subscriber collects traces for specific targets at specific levels and outputs them in a subscriber-specific way. The data pipeline uses a subscriber which outputs traces to stdout, and a subscriber which delivers traces to Jaeger. The tracing level of each can be specified separately by environment variables.|
|Service|A service is roughly synonymous with a process, or component of the pipeline. The child/parent relationships of spans can cross service boundaries, for instance the trace-to-events component and digitiser-aggregator.|

## Tracing Levels

Tracing levels refer to the severity of the span or event being created. If a subscribe records tracing data at level `INFO` then it records all tracing data at or below the `INFO` level. That is a lower number indicates a higher priority.

These can be one of:

|Priority|Level|Usage|Example|
|---|---|---|---|
|1|Error|When a state arises which either results in the termination of the program or significantly affects its smooth running.|An incorrect configuration is given during start-up.|
|2|Warn|When a state arises which prevents the program doing its job properly, but could be recoverable in the future.|A corrupted or unidentifiable Kafka message, or series of invalid Run commands.|
|3|Info|To indicate the normal running of the program at a coarse-grained level, as well as collate spans from other components.|The `Run` span in the `nexus-writer` tool which links to all traces relevant to the same run.|
|4|Debug|For events and spans which may assist in debugging issues. Generally low priority.|Details of event formation|
|5|Trace|For the most fine-grained spans and traces. Generally each and every function should be instrumented at the trace level.|IO, or disk write functions|

## Subscribers

## Target

Targets are used by subscribers to determine which traces to consume. Each subscriber has an associated level for each target and consumes all traces directed at that target at or below that level.

Each module has its own target by default, and if no target is specified, traces are directed towards the module target.

In addition to the module targets, some traces and spans are targeted at `otel` to indicate these should only be consumed by the OpenTelemetry subscriber.

## Instrumenting Functions

This section describes the use of the `#[tracing::instrument]` macro placed over functions. Using the macro is preferred over defining spans directly. If necessary, the `name` field can be overridden.

Every function that can fail should be instrumented (i.e. that has return type `Result<>`), and should have use `err(level = WARN)` or `err(level = ERROR)` depending on the type of error.

## Diagrams

The following diagrams define all spans which exist at the `INFO` level (and some at use at the `DEBUG` level, though not all). The first one shows the span structure in the simulator (not part of the data-pipeline). If Kafka messages are simulated, then the `process_kafka_message` which begins the `trace-to-events` component will have the final `Simulated Digitiser Trace` span as parent.

### Simulator

This diagram shows the normal structure of the simulator running in `defined` mode setup to generate digitiser trace messages. Please refer to [Mermaid documentation](https://mermaid.js.org/syntax/entityRelationshipDiagram.html#relationship-syntax) for an explanation of the connecting symbols.

```mermaid
erDiagram
    SIM_RUN_SIMULATION["run_configured_simulation"] {
        service simulator
    }
    SIM_GEN_DIGITISERS["generate_digitisers"] {
        service simulator
    }
    SIM_RUN_SIMULATION ||--|| SIM_GEN_DIGITISERS : "contains one"
    SIM_DIGITISER["digitisers"] {
        service simulator
        int id
    }
    SIM_GEN_DIGITISERS ||--o{ SIM_DIGITISER : "contains many"
    SIM_RUN_SCHEDULE["Debug: run_schedule"] {
        service simulator
    }
    SIM_RUN_SIMULATION ||--|| SIM_RUN_SCHEDULE : "contains one"
    SIM_RUN_FRAME["Debug: run_frame"] {
        service simulator
        int frame_number
    }
    SIM_RUN_SCHEDULE ||--o{ SIM_RUN_FRAME : "contains many"
    SIM_RUN_DIGITISER["Debug: run_digitiser"] {
        service simulator
        int digitiser_id
    }
    SIM_RUN_FRAME ||--o{ SIM_RUN_DIGITISER : "contains many"

    SIM_GEN_EVENT_LIST["Debug: generate_event_lists"] {
        service simulator
    }

    SIM_GEN_DIG_TRACE_PUSH["Debug: generate_trace_push_to_cache"] {
        service simulator
    }
    SIM_RUN_DIGITISER ||--|{ SIM_GEN_DIG_TRACE_PUSH : "contains many"
    SIM_GEN_DIG_TRACE_PUSH ||--|| SIM_GEN_EVENT_LIST : "contains one"

    SIM_CHANNEL_TRACE["channel trace"] {
        service simulator
        int channel
        int expected_pulses
    }
    SIM_GEN_DIG_TRACE_PUSH ||..|| SIM_CHANNEL_TRACE : "followed by one"

    SIM_SEND_DIG_TRACE["send_digitiser_trace_message"] {
        service simulator
    }
    SIM_RUN_DIGITISER ||--o{ SIM_SEND_DIG_TRACE : "contains many"
    SIM_SEND_DIG_TRACE ||--o{ SIM_CHANNEL_TRACE : "contains many"
    SIM_DIG_TRACE["Simulated Digitiser Trace"] {
        service simulator
    }
    SIM_SEND_DIG_TRACE ||..|| SIM_DIG_TRACE : "followed by one"

    EF_KAF_MSG["process_kafka_message"] {
        service trace-to-events
        int kafka_message_timestamp_ms
    }
    SIM_DIG_TRACE ||--|| EF_KAF_MSG : "contains one"
```

### Data Pipeline

Let us define `metadata` as the fields:

|Type|Name|
|---|---|
|DateTime|timestamp|
|int|frame_number|
|int|period_number|
|int|veto_flags|
|int|protons_per_pulse|
|bool|running|

These fields correspond either to Frame Event List metadata, Digitiser Event List metadata, or Digitiser Trace metadata, depending on context.

```mermaid
erDiagram
    EF_KAF_MSG["process_kafka_message"] {
        service trace-to-events
        int kafka_message_timestamp_ms
    }
    EF_SPANNED_ROOT["spanned_root_as_digitizer_analog_trace_message"] {
        service trace-to-events
    }
    EF_KAF_MSG ||--|| EF_SPANNED_ROOT : "contains one"
    EF_DIG_TRACE_MSG["process_digitiser_trace_message"] {
        service trace-to-events
        int digitiser_id
        metadata metadata
    }
    EF_KAF_MSG ||--|| EF_DIG_TRACE_MSG : "contains one"

    PROCESS["process"] {
        service trace-to-events
    }
    EF_DIG_TRACE_MSG ||--|| PROCESS : "contains one"
    FIND_CHNL_EVTS["find_channel_events"] {
        service trace-to-events
        int channel
        int num_pulses
    }
    PROCESS ||--o{ FIND_CHNL_EVTS : "contains many"

    DA_KAF_MSG["process_kafka_message"] {
        service digitiser-aggregator
        int kafka_message_timestamp_ms
    }
    DA_SPANNED_ROOT["spanned_root_as_digitizer_event_list_message"] {
        service trace-to-events
    }
    DA_KAF_MSG ||--|| DA_SPANNED_ROOT : "contains one"
    DA_DIG_EVT_MSG["process_digitiser_event_list_message"] {
        service digitiser-aggregator
        int digitiser_id
        metadata metadata
    }
    DA_KAF_MSG ||--|| DA_DIG_EVT_MSG : "contains one"
    DA_FRAME_COMPLETE["Frame Complete"] {
        service digitiser-aggregator
    }
    DA_DIG_EVT_MSG |o--|| DA_FRAME_COMPLETE : "may contain one"
    EF_DIG_TRACE_MSG ||--|| DA_KAF_MSG : "contains one"
    FRAME["Frame"] {
        service digitiser-aggregator
    }
    FRAME_DIGITISER["Digitiser Event List"] {
        service digitiser-aggregator
    }
    DA_DIG_EVT_MSG ||..|| FRAME_DIGITISER : "followed by one"
    FRAME ||--o{ FRAME_DIGITISER : "contains many"

    NW_KAFKA_MSG["process_kafka_message"] {
        service nexus-writer
    }
    NW_SPANNED_ROOT["spanned_root_as"] {
        service trace-to-events
    }
    NW_KAFKA_MSG ||--|| NW_SPANNED_ROOT : "contains one"
    FRAME |o--|| NW_KAFKA_MSG : contains
    NW_FRM_EVT_MSG["process_frame_assembled_event_list_message"] {
        service nexus-writer
    }
    NW_KAFKA_MSG ||--|| NW_FRM_EVT_MSG : "contains one"
    RUN["Run"] {
        service nexus-writer
    }
    RUN_FRAME["Frame Event List"] {
        service nexus-writer
        timestamp timestamp
        metadata metadata
    }
    RUN ||--o{ RUN_FRAME : "contains many"
    NW_FRM_EVT_MSG ||..|| RUN_FRAME : "followed by one"
```

### Digitiser Trace Message Arrives in Event Formation

```mermaid
sequenceDiagram
participant B as Broker
participant E as Event Formation
participant D as Event Detector
participant C as Channel Detector
B ->> E: Digitiser Trace Message
E -->>+ E: Info Span: Trace Source Message
B -->> E: Extract Parent from Header
E -->> E: Debug: Kafka Message Details
E ->>+ D: Function: process
D -->>+ E: Info Span: process
loop For each Channel
D -)+ C: Function: find_channel_events
C -->>+ E: Info Span: find_channel_events
Note over E: Fields: channel, num_pulses
E -->>- C: End Span: find_channel_events
C -)- D: Return: find_channel_events
end
E -->>- D: End Span: process
D ->>- E: Return: process
E -->>- E: End Span: Trace Source Message
```

### Digitiser Event Message Arrives in Digitiser Aggregator

```mermaid
sequenceDiagram
participant B as Broker
participant A as Digitiser Aggregator
participant C as Frame Cache
participant F as Current Frame
participant FS as Current Frame Span
FS -->>+ FS: Existing Info Span: Frame (otel)
Note over FS: Fields: frame `metadata`, is_complete, is_expired
B ->> A: Digitiser Event List Message
A -->>+ A: Info Span: on_message
B -->> A: Extract Parent from Header
A ->>+ C: Method: push
C -->>+ A: Info Span: push
Note over A: Fields: digitiser_id, digitiser `metadata`
C ->> F: Method: push
A -->>- C: End Span: push
C ->>- A: Return: Current Frame
A -->>+ FS: InfoSpan: Digitiser Event List (otel)
Note over FS: Fields: digitiser_id, digitiser `metadata`
A -->> FS: Follows From
FS -->>- A: End Span: Digitiser Event List
A -->>- A: End Span: on_message
FS -->>- FS: End Span: Frame
```

### Frame Event Message Arrives in Nexus Writer

```mermaid
sequenceDiagram
participant B as Broker
participant W as Nexus Writer
participant E as Nexus Engine
participant R as Current Run
participant RS as Current Run Span
participant F as Run File
RS -->>+ RS: Existing Info Span: Run (otel)
B ->> W: Frame Event List Message
W -->>+ W: Info Span: process_kafka_message
B -->> W: Extract Parent from Header
W ->>+ E: Method: process_event_list
E -->>+ W: Info Span: process_event_list
E ->> R: Method: push_message
R ->> F: Method: push_message_to_runfile
F -->>+ W: Trace Span: push_message_to_runfile
W -->>- F: End Span: push_message_to_runfile
W -->>- E: End Span: process_event_list
E ->>- W: Return: Get Current Run
W -->>+ RS: InfoSpan: Frame Event List (otel)
W -->> RS: Follows From
RS -->>- W: End Span: Frame Event List
W -->>- W: End Span: process_kafka_message
RS -->>- RS: Existing Info Span: Run
```
