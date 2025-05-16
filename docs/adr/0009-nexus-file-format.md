# 9. NeXus file format

Date: 2025-05-15

## Status

Accepted

(note that this ADR was written retrospectively)

## Context

The SuperMuSR data pipeline writes its output to hdf5 files which must be compatible with tools such as "MuonData event filtering", "WiMDA", and possibly "Mantid".

To ensure this, it is advisable that the pipeline's output format conform to a standard compatible with these tools, and which is publically available to allow future tools to easily read the pipeline's output files.

## Decision

The Nexus Writer will write output conforming to a standard detailed in the components doc files. Wherever possible this standard conforms to [Muon Instrument Definition: Version 2 – 'muonTD'](https://www.nexusformat.org/content/Muon_Time_Differential.html). The main deviation is that the [NXdata](https://manual.nexusformat.org/classes/base_classes/NXdata.html) class in the `raw_data_1/detector_1` field is replaced with the [NXevent_data](https://manual.nexusformat.org/classes/base_classes/NXevent_data.html) class, as this is designed for storing streamed event data.

## Consequences

The Nexus Writer must be carefully documented, especially where the output format is concerned. Any file writes which do not conform to the above decision should be clearly explained.
