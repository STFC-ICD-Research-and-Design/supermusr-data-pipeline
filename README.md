# SuperMuSR data pipeline

[![CI](https://github.com/DanNixon/supermusr-data-pipeline/actions/workflows/ci.yml/badge.svg)](https://github.com/DanNixon/supermusr-data-pipeline/actions/workflows/ci.yml)

See [docs](./docs).


## Nexus Writer

### Module Level Comments
- nexus (Done)
    - units (Done)
    - classes (Done)
    - logs (Done)
        - alarm (Done)
        - f114 (Done)
        - se00 (Done)
    - file_interface (Done)
        - mock_nexus_file (Done)
        - nexus_file (Done)
 - run_engine (Done)
    - run (Done)
        - run_parameters (Done)
        - run_spans (Done)
    - engine (Done)
    - run_message (Done)
    - settings (Done)
- hdf5_handlers (Done)
    - attributes (Done)
    - dataset_flatbuffers (Done)
    - dataset (Done)
    - error (Done)
    - group (Done)
- error (Done)
- flush_to_archive (Done)
- kafka_topic_interface (Done)
- message_handers (Done)
- nexus_structure
    - entry
        - period
        - runlog
        - selog
        - event_data
        - sample
            - geometry
        - instrument
            - source
    - logs
        - alarm_log
        - log
        - value_log

### Structs

- Cli
- Topics
- nexus
    - ::NexusGroup (Done)
    - ::file_interface::nexus_file::NexusFile (Done)
- nexus_structure
    - ::Root (Done)
    - ::entry::Entry
    - ::entry::event_data::EventData
    - ::entry::instrument::Instrument
    - ::entry::instrument::source::Source
    - ::entry::period::Period
    - ::entry::runlog::RunLog
    - ::entry::sample::Sample
    - ::entry::sample::geometry::Geometry
    - ::entry::selog::SELog
    - ::logs::alarm_log::AlarmLog
    - ::logs::log::Log
    - ::logs::log::LogSettings
    - ::logs::value_log::ValueLog
- run_engine
    - ::engine::NexusEngine (Done)
    - ::run::Run (Done)
    - ::run::run_parameters::NexusConfiguration (Done)
    - ::run::run_parameters::RunParameters (Done)
    - ::run::run_parameters::RunStopParameters (Done)
    - ::run_messages::InitialiseNewNexusRun
    - ::run_messages::InitialiseNewNexusStructure
    - ::run_messages::PushFrameEventList
    - ::run_messages::PushLog
    - ::run_messages::PushRunStart
    - ::run_messages::SetEndTime
    - ::run_messages::UpdatePeriodList
    - ::settings::ChunkSizeSettings
    - ::settings::NexusSettings

### Traits

hdf5_handlers::AttributeExt
hdf5_handlers::DatasetExt
hdf5_handlers::DatasetFlatbuffersExt
hdf5_handlers::GroupExt
hdf5_handlers::HasAttributesExt
hdf5_handlers::error::ConvertResult
nexus::NexusMessageHandler
nexus::NexusSchematic
nexus::file_interface::NexusFileInterface
nexus::logs::AlarmMessage
nexus::logs::LogMessage
nexus::units::DatasetUnitExt
run_engine::engine::FindValidRun
run_engine::run::run_spans::RunSpan
run_engine::run_messages::HandlesAllNexusMessages

### Functions

flush_to_archive::archive_flush_task
flush_to_archive::create_archive_flush_task
flush_to_archive::flush_to_archive
flush_to_archive::move_file_to_archive
hdf5_handlers::group::get_dataset_builder
main
message_handlers::increment_message_received_counter
message_handlers::process_payload_on_alarm_topic
message_handlers::process_payload_on_control_topic
message_handlers::process_payload_on_frame_event_list_topic
message_handlers::process_payload_on_runlog_topic
message_handlers::process_payload_on_sample_env_topic
message_handlers::push_alarm
message_handlers::push_f144_sample_environment_log
message_handlers::push_frame_event_list
message_handlers::push_run_log
message_handlers::push_run_start
message_handlers::push_run_stop
message_handlers::push_se00_sample_environment_log
message_handlers::report_parse_message_failure
message_handlers::spanned_root_as
nexus::logs::adjust_nanoseconds_by_origin_to_sec
nexus::logs::remove_prefixes
nexus::logs::se00::get_se00_len
nexus_structure::entry::extract_run_number
process_kafka_message
run_engine::settings::get_path_glob_pattern

## Digitiser Aggregator

## Trace to Events

## Simulator

## Others
