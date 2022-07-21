# frame-assembler

This tools is the first in the chain of live processing of events from the digitisers.

It has the following responsibilities:

- Collecting all digitisers event lists into a single event list, sorted by event timestamp, on a per frame basis
- Ensuring digitisers are in a consistent state and alerting on fault
  - Are all digitisers collecting data simultaneously? (inspect status packets)
  - Did all digitisers start and stop collection at the same time? (detect missing frames, drop partial data)

The current implementation is largely not useful, the architecture did not suit adding extra functionality and should be rethought.
The code there is now was good enough for the July test where there was only a single digitiser.
