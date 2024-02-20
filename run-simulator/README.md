# run-simulator

## Introduction

This tool sends either a <code>RunStart</code> message, or a <code>RunStop</code> message to the designated Kafka broker assigned to the given topic.

## Command Line
The program is executed from the command line, for instance:
```
run-simulator --broker localhost:19092 --topic Control --run-name Test --time "2024-01-30 15:17:03.618842621Z" run-start --instrument-name SuperMuSR
```

For detailed instructions about each parameter run
```
run-simulator --help
```

## Time Format
If  the ``time`` command line argument is ommitted then the current time is used. This argument expects a date/time given in the following format:
```
"[YYYY]-[mm]-[dd] [HH]:[MM]:[SS].[nnnnnnnnn]Z"
```
or
```
[YYYY]-[mm]-[dd]T[HH]:[MM]:[SS].[nnnnnnnnn]Z
```
That is the hyphen-separated date followed by colon-separated time terminated by "Z". Between the date and time can go either a space, or a "T".

If a space is used the timestamp must be enclosed by quoatation marks.

For instance:
```
"2024-01-30 15:17:03.618842621Z"
```
or
```
2024-01-30T15:17:03.618842621Z
```