# kafka-daq-report

A simple TUI tool that listens on the trace topic and reports in a table view the following for each DAQ that is seen to be sending messages:

- Number of messages received
- Timestamp of first message received since starting kafka-daq-stats
- Timestamp of last message received
- Frame number of the last message received
- Number of channels present in the last message received
- A flag indicating if the number of channels has ever changed
- Number of samples in the first channel of the last message received
- A flag indicating if the number of samples is not identical in each channel
- A flag indicating if the number of samples has ever changed
