# trace-archiver

## Introduction

Simple tool to save all received trace messages to HDF5 files in a given directory.
Useful for diagnostics.

A file is created for each received trace message and saved in a file named
using the format `frame_{timestamp}_{digitizer_id}_{frame_number}.h5`.

The structure of the HDF5 file is as follows:

```text
.
|- metadata
|  |- frame_timestamp
|  |  |- seconds
|  |  |- nanoseconds
|  |- digitizer_id
|  |- frame_number
|  |- channel_numbers   [n channels]
|- channel_data         [n channels, n time points]
```

## Command Line

The program is executed from the command line, for instance:

```shell
trace-archiver --broker localhost:19092 --group g1 --trace-topic trace_in
```

For detailed instructions about each parameter run

```shell
trace-archiver --help
```
