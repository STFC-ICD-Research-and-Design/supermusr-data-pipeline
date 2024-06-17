# 3. Initial flatbuffer formats

Date: 2024-06-17

## Status

Accepted

(note that this ADR was written retrospectively)

## Context

The SuperMuSR data pipeline will use Google Flatbuffers as the definition and encoding for data passed through the pipeline.
This requires schemas to be written and distributed between the various components.

Given previous in-kind work on ESS streaming, where possible [existing schemas](https://github.com/ess-dmsc/streaming-data-types) should be used.

## Decision

The following existing ESS schemas will be used to facilitate communication from the instrument control system to the data pipeline:

- `6s4t_run_start.fbs`
- `al00_alarm.fbs`
- `df12_det_spec_map.fbs`
- `f144_logdata.fbs`
- `pl72_run_start.fbs`
- `se00_data.fbs`

The following new schemas will be created, specifically for SuperMuSR:

- `aev2_frame_assembled_event_v2.fbs` - an ISIS frame of event data from the entire instrument
- `dat2_digitizer_analog_trace_v2.fbs` - an ISIS frame of ADC data from a single digitizer (i.e. 8 channels)
- `dev2_digitizer_event_v2.fbs` - an ISIS frame of event data from a single digitizer (i.e. 8 channels)

## Consequences

- The interface between the digitisers and software components is fully defined
- The interface may change as requirements become more clear in the future (particularly around the output of event formation). Versioning and the ability to make some backwards compatible changes are possible using Flatbuffers.
