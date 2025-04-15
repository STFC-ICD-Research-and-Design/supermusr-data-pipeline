# nexus-writer source code description

This file explains the nexus-writer source code file structure.

## Top-Level Modules

### main.rs

This module operates in a similar way to other components, containing the `clap` Cli structure.

### message_handlers.rs

Consists of functions which bridge the main `main` module and `run_engine`.

### flush_to_archive.rs

Implements

### error.rs

### run_engine.rs

Manages the run cache, accepts flatbuffer messages via the `message_handler` module and passes them to the appropriate `run` object.

### hdf5_handlers.rs

Extension Traits for `hdf5` objects.

### nexus

Defines general structs and traits used in the `nexus_structure` module.

### nexus_structure

This is where the nexus file structure is found, files and directories are set out in a way that roughly follows the group structure of the nexus format.

## Lower-Level Modules

### nexus.rs

Defines the `NexusSchematic` trait which is implemented by every struct in the `nexus_structure` module.
This trait standardises the creating and opening of the appropriate fields in nexus files.

Defines the `NexusMessageHandler<M>` trait which takes a generic `M`. This trait is implemented by structs in `nexus_structure` module which process messages `M`.

Defines the `NexusGroup<S>` struct which takes a generic `S : NexusSchematic`. This struct is a wrapper around the structs defined in the `nexus_structure` module.

Note that for every `S : NexusSchematic` which implements `NexusMessageHandler<M>`, `NexusGroup<S>` automatically implements `NexusMessageHandler<M>` and passes `M` to `S`.

#### file_interface.rs

This defines a trait which abstracts the hdf5 file functionality. This interface is injected into the Run object. This allows the hdf5 file to be mocked in unit tests.

#### logs.rs

This defines the `LogMessage` and `AlarmMessage` traits which are implemented on appropriate types of log message (e.g. flatbuffer logs).
This trait allows performs tasks such as getting the approprate `type_description` for the message, or writing its time/value to a given `Dataset`.

#### classes.rs

Implements the enum `NexusClass` which standardises the adding of attributes specifying the nexus classes of groups.

#### units.rs

Implements the enum `NexusUnits` which standardises the adding of attributes specifying physical units.

### nexus_structure.rs

Defines the `Root` struct which is the top level of the nexus format.

#### entry.rs

Defines the `Entry` struct which contains most of the fields, and implements `NexusSchematic` for it.

##### event_data.rs

Defines the `EventData` struct, and implements `NexusSchematic` for it.

##### runlog.rs

Defines the `RunLog` struct, and implements `NexusSchematic` for it.

`RunLog` maintains a `HashMap` of `NexusGroup<Log>` keyed by `String`.

##### selog.rs

Defines the `SELog` struct, and implements `NexusSchematic` for it.

`RunLog` maintains a `HashMap` of `NexusGroup<ValueLog>` keyed by `String`.

#### logs

Defines the `Log`, `ValueLog` and `AlarmLog` structs which are used in `runlog.rs` and `selog.rs`, and implements `NexusSchematic` for them.

The struct `ValueLog` consists of `Option<Log>` and `Option<AlarmLog>` fields.
