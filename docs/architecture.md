# Architecture

Only two pieces of infrastructure are considered here: the Kafka broker and the "broker adjacent compute".
As it is not determined if these are going to be the same physical infrastructure or not they are referred to as "Compute 1" and "Compute 2".

## "Compute 1"

- Responsible for Kafka broker.
- Requires sufficient storage for persisting `dat1` (trace) and `dev1` (event) messages from digitizers for as long as deemed necessary.
   - (1 cycle worth of `dev1`)
   - ("a while" worth of `dat1`)
- High availability required to avoid loss of experimental data
   - (without the broker running data has no persistent storage)

## "Compute 2"

- Responsible for compute tasks that operate on data streams:
   - Preprocessing/event formation.
   - NeXus file writing.
- Should be network adjacent to broker.
- If compute resources are sensible and there is sufficient overhead on Compute 1 then these can be absorbed into Compute 1.
