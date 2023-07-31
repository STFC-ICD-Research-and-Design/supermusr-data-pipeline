# trace-archiver

## Docker Installation
Ensure your working director is `trace-archiver-db`, and select one of the following commands:
1. `docker compose --profile main up -d`
1. `docker compose --profile full up -d`
1. `docker compose --profile tdengine up -d`
1. `docker compose --profile trace-archiver-only up -d`

1. `main` installs containers for TDEngine, RedPanda and the trace archiver program
1. `full` installs all the `main` containers as well as RedPanda's console
1. `tdengine` installs a container for TDEngine and the trace archiver program
1. `trace-archiver-only` installs a single container for the trace archiver program

## Introduction

This tool reads messages from the broker, extracts <code>DigitizerAnalogTraceMessage</code> instances and writes them to the TDEngine time-series database.

The process is:
1. Receive message from the kafka broker
1. Extract the payload of a <code>DigitizerAnalogTraceMessage</code> type
1. Process the message:
    1. Extract metadata from the <code>DigitizerAnalogTraceMessage</code> and test for critial malformations.
    1. Test the channel traces for non-critical malformation, and create reports for any found.
    1. Extract the channal trace data into SQL statements
1. Halt the process if any critical malformations or errors occur
1. Post the message:
    1. Send the SQL statements to TDEngine
    1. Verify the expected number of rows have been entered

The program makes the following assumptions:
* There should be 8 channels in a digitizer.
* The correct number of samples is the maximum size of the voltage list over all channels.

Critical malformations are:
* Missing timestamp.
* Missing metadata.
* Missing channel list.

Non-critical malformations are:
* Incorrect number of channels in the channel list.
* Channels with missing voltage lists.
* Channels with truncated voltage list (the correct voltage list size is calculated to be the maximum over all channels).

If a non-critical malformation is detected, the program will insert what data can be salvaged into what it determines to be the appropriate places.

## Actions on non-critical malformation
### Incorrect number of channels in the channel list
The program will extract up to eight channels placing them in the order determined by their channel indices.

### Channels with missing voltage lists
Channels missing voltage lists are discarded.

### Channels with truncated voltage list
Truncated voltage lists are padded at the end with zeroes. (Note here the assumption is that it's the latter data that is missing).

### Discarded channels
All discarded channels are replaced with all zero voltage lists of the correct size.

## Installation
