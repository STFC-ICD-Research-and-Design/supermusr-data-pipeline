# trace-reader

## Introduction
This program reads picscope .trace files, a binary file developed by E.M Schooneveld.

## Command Line Interface
trace-reader [OPTIONS] --broker <BROKER>

### Options:
```
  -b, --broker <BROKER>                                
  -u, --username <USERNAME>                            
  -p, --password <PASSWORD>                            
  -c, --group <CONSUMER_GROUP>                         [default: trace-producer]
  -t, --trace-topic <TRACE_TOPIC>                      [default: Traces]
  -f, --file-name <FILE_NAME>                          
  -n, --number-of-trace-events <NUMBER_OF_EVENTS>      [default: 1]
  -r, --random-sample                                  
  -h, --help                                           Print help
  -V, --version                                        Print version
```
The only mandatory option is `--broker`. This should be in format `"host":"port"`, e.g. `--broker localhost:19092`.

The trace topic is the kafka topic that trace messages are produced to.
The file name is the relative path that the .trace file is found.
The `number-of-trace-events` parameter is the number of Trace Events to read from the file, if this is zero, then all trace events will be read.
If this is greater than the number of trace events available, then some events will be duplicated (according to `random-sample`).

If `random-sample` is set then trace-events are read from the file randomly. Selection is made with replacement so duplication is possible.
If this flag is not set then trace-events are read in order.
If `number-of-trace-events` is greater than the number available then trace-events are the reader wraps around to the beginning of the file as often as necessary.

## Terminology
- Trace: This is continous block of voltage readings from a digitizer channel.
- Trace Event: This is a collection of traces, one for each channel on the digitizer.
Note that "Event" here is meant in a different sense than the trace-to-event tool. The overlap is a result of terminology used in the .trace file. To avoid confusion, the term "Trace Event" is used here.