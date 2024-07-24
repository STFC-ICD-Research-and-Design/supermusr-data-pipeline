# Policy on using tracing, metrics and logging

## Tracing Levels

Tracing levels refer to the severity of the span or event being created. These can be one of:

1. Error
1. Warn
1. Info
1. Debug
1. Trace

### Error

This is to be used when a state arises which either results in the termination of the program or significantly affects its smooth running. For instance, if incorrect an configuration is given during start-up, or [TODO]

### Warn

This is to be used when a state arises which prevents the program doing its job properly, but could be recoverable in the future. For instance, a corrupted or unidentifiable kafka message, or series of invalid Run commands.

### Info

This is to be used to indicate the normal running of the program at a coarse-grained level, as well as collate spans from other components. For instance, the `Run` span in the `nexus-writer` tool which links to all traces relevant to the same run.

### Debug

This is for events and spans which may assist in debugging issues. Generally low priority. For instance, details of event formation.

### Trace

This is for the most fine-grained spans and traces. Generally each and every function should be instrumented at the trace level.

## Instrumenting Functions

This section describes the use of the `#[tracing::instrument]` macro placed over functions.

Every function that can fail should be instrumented (i.e. that has return type `Result<>`).

## Services

Services in Open Telemetry are the highest level category of traces.

Each component has its own service, which takes its name from the component module name, namely:

- nexus-writer
- digitiser-aggregator
- trace-to-events
- simulator
- run-simulator
- trace-archiver-hdf5
- trace-archiver-tdengine
- kafka-daq-report
- trace-reader

## Targets

Targets are used by subscribers to determine which traces to consume. Each subscriber has an associated level for each target and consumes all traces directed at that target at or below that level.

Each module has its own target by default, and if no target is specified, traces are directed towards the module target.

In addition to the module targets, some traces and spans are targeted at `otel` to indicate these should only be consumed by the OpenTelemetry subscriber.

## Diagrams

Let us define `metadata` as the fields:

- timestamp
- frame_number
- period_number
- veto_flags
- protons_per_pulse
- running

These fields correspond either to Frame Event List metadata or Digitiser Event List metadata depending on context.

### Run Span

```mermaid
erDiagram
    EVENT_LIST_TEMPLATE["event_list_template"] {
        service simulator
        int num_pulses
    }
    GEN_EVENT_LIST["generate_event_list_push_to_cache"] {
        service simulator
    }
    EVENT_LIST_TEMPLATE ||--|| GEN_EVENT_LIST : contains
    GEN_TRACE["generate_trace_push_to_cache"] {
        service simulator
    }
    EVENT_LIST_TEMPLATE ||--|| GEN_TRACE : contains
    SEND_CHANNEL_TRACE["channel trace"] {
        service simulator
        int channel
    }
    SEND_CHANNEL_TRACE ||..|| GEN_TRACE : followed_by
    SEND_TRACE["send_trace_message"] {
        service simulator
        int digitiser_id
    }
    SEND_TRACE ||--o{ SEND_CHANNEL_TRACE : contains
    TRACE_SRC_MSG["Trace Source Message"] {
        service trace-to-events
    }
    SEND_TRACE ||--|| TRACE_SRC_MSG : contains
    PROCESS["process"] {
        service trace-to-events
    }
    TRACE_SRC_MSG ||--|| PROCESS : contains
    FIND_CHNL_EVTS["find_channel_events"] {
        service trace-to-events
        int channel
        int num_pulses
    }
    PROCESS ||--o{ FIND_CHNL_EVTS : contains

    DA_KAFKA_MSG["process_kafka_message"] {
        service digitiser-aggregator
    }
    TRACE_SRC_MSG ||--|| DA_KAFKA_MSG : contains
    DA_DIG_EVT_MSG["process_digitiser_event_list_message"] {
        service digitiser-aggregator
    }
    DA_FRAME_COMPLETE["Frame Complete"] {
        service digitiser-aggregator
    }
    DA_DIG_EVT_MSG |o--|| DA_FRAME_COMPLETE : contains
    DA_KAFKA_MSG ||--|| DA_DIG_EVT_MSG : contains
    FRAME["Frame"] {
        service digitiser-aggregator
    }
    FRAME_DIGITISER["Digitiser Event List"] {
        service digitiser-aggregator
    }
    DA_DIG_EVT_MSG ||..|| FRAME_DIGITISER : followed_by
    FRAME ||--o{ FRAME_DIGITISER : contains

    NW_KAFKA_MSG["process_kafka_message"] {
        service nexus-writer
    }
    FRAME |o--|| NW_KAFKA_MSG : contains
    NW_FRM_EVT_MSG["process_frame_assembled_event_list_message"] {
        service nexus-writer
    }
    NW_KAFKA_MSG ||--|| NW_FRM_EVT_MSG : contains
    RUN["Run"] {
        service nexus-writer
    }
    RUN_FRAME["Frame Event List"] {
        service nexus-writer
        timestamp timestamp
        int frame_number
        int period_number
        int veto_flags
        int protons_per_pulse
        bool running
        bool is_completed
        bool is_expired
    }
    RUN ||--o{ RUN_FRAME : contains
    NW_FRM_EVT_MSG ||..|| RUN_FRAME : followed_by
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
