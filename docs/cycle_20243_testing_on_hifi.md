# Cycle 2024/3 testing on HIFI

## Basic verification of data pipeline for detector data

Verify that:

- Trace data is being received from all expected digitisers
- Frame metadata is identical across digitisers and is correct
- Event formation runs
- Frame aggregator runs

## Creation of NeXus files triggered by IBEX/SECI and containing correct metadata

Verify that:

- NeXus writer is triggered by a run start message from IBEX/SECI
- NeXus writer is triggered by a run stop message from IBEX/SECI
- Metadata sent via IBEX/SECI is stored in the NeXus file correctly (this could be interpreted as one of each metadata type or all of the metadata required for analysis in WIMDA)

## Measurement of data rate and compute resource usage

Measure:

- Network data rate between digitiser switch and Kafka broker
- Resource used on broker by:
   - Redpanda
   - Event formation
   - Aggregator
   - NeXus writer

## Suitability of NeXus file for downstream filtering

Verify that:

- NeXus files contain experimental/detector data for the appropriate time period (i.e. they contain all of the data for the run and no more)
- NeXus files contain metadata for the appropriate time period
- It is possible to sensibly align detector and metadata in time
- NeXus file contains sufficient metadata for filtering and histogram generation

## Basic verification of filtering and histogram generation

Verify that the tool:

- Reads NeXus event files (written by NeXus writer)
- Filter by frame timestamp
- Filter by amplitude
- Writes a NeXus file that WIMDA can correctly read

## Verification/inspection of produced events and histograms

Some form of manual inspection of the histograms, comparing them to those captured by the existing data acquisition.
Details TBC.
