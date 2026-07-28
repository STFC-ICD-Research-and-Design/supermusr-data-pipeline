mod cached_dataset;
mod channel;
mod digitiser;
mod reader;

use crate::{
    Hdf5,
    hdf5trace::reader::{DigitiserReader, ReadCommand},
};
use chrono::ParseError;
use digital_muon_common::{
    DigitizerId, FrameNumber,
    spanned::{SpanWrapper, Spanned},
};
use hdf5::{File, OpenMode};
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};
use rdkafka::{ClientConfig, error::KafkaError};
use std::{fmt::Debug, num::ParseIntError, path::PathBuf, str::FromStr};
use thiserror::Error;
use tracing::{debug, info, info_span};

pub(crate) use digitiser::{HDF5Config, Hdf5Digitiser};

#[derive(Error, Debug)]
pub(crate) enum Error {
    #[error("Dataset name is zero-length")]
    DatasetNameZeroLength,
    #[error("Dataset {0} is a scalar, vector expected.")]
    DatasetScalar(String),
    #[error("{0}")]
    DateTime(#[from] ParseError),
    #[error("{0}")]
    HDF5(#[from] hdf5::Error),
    #[error("{0}")]
    Kafka(#[from] KafkaError),
    #[error("No digitisers from {0:?} selected.")]
    NoDigitisersSelected(Vec<DigitizerId>),
    #[error("Expecting Underscore in {0}")]
    NoUnderscore(String),
    #[error("{0}")]
    ParseInt(#[from] ParseIntError),
    #[error("Wrong Identifier. Expected {0}, got {1}")]
    WrongIdentifier(String, String),
    #[error("Frame Index {0} >= Number of Frames {1}")]
    FrameIndexTooLarge(usize, usize),
    #[error("Frame Number {0} not found.")]
    FrameNumberNotFound(FrameNumber),
    #[error("Frame Number {0} not found.")]
    TimestampNotFound(String),
    #[error("JSON Error: {0}")]
    Json(#[from] serde_json::Error),
    #[error(
        "Used `t` or `tc` for trace files whose `config_timestamp_as_rfc3339` attribute is `true`"
    )]
    ConfigReadSeqTimestampInTimestampAsRfc3339Attrib,
}

/// Extracts the `index` from a string of the form `.../identifier_index`,
/// where `identifier` is the expected name, for instance "Digitiiser" or "Channel".
///
/// # Parameters
/// - source: the string to extract from.
/// - identifier: determines the expected format of the string.
///
/// # Returns
/// A value of type T, where T can be parsed from a string stlice.
///
/// # Errors
/// Returns an error if:
/// - `source` is zero length.
/// - `source` has no underscore.
/// - `source` has the wrong identifier.
fn extract_from_dataset_name<T>(source: String, identifier: &'static str) -> Result<T, Error>
where
    T: FromStr,
    <T as FromStr>::Err: Debug,
    Error: From<<T as FromStr>::Err>,
{
    let source_parts = source
        .split('/')
        .next_back()
        .ok_or(Error::DatasetNameZeroLength)?
        .split('_')
        .collect::<Vec<_>>();
    if source_parts.len() < 2 {
        Err(Error::NoUnderscore(source.clone()))?;
    }
    if source_parts
        .first()
        .expect("First part should exist, this should never fail.")
        != &identifier
    {
        Err(Error::WrongIdentifier(
            identifier.to_string(),
            source_parts
                .first()
                .expect("First part should exist, this should never fail.")
                .to_string(),
        ))?
    }
    Ok(source_parts
        .get(1)
        .expect("Second part should exist, this should never fail.")
        .parse()?)
}

/// Runs the main loop when the program is run in `hdf5` mode.
///
/// # Parameters
/// - file_name: the file to read.
/// - client_config: the kafka config settings to use for the produer.
/// - trace_topic: the topic to produce trace messages to.
/// - args: the cli args specific to `hdf5` mode.
pub(crate) async fn read_hdf5_file(
    file_name: PathBuf,
    client_config: &ClientConfig,
    trace_topic: &str,
    key: &str,
    args: Hdf5,
) -> Result<(), Error> {
    // FIXME: Figure out which is the best file reader to use, probably `stdio` or `sec2`.
    // Also should we allow a logging option for debugging?
    let file = {
        let mut file_builder = File::with_options();
        file_builder.fapl().stdio();
        //file_builder.fapl().log_options(Some("mylog"), LogFlags::union(LogFlags::LOC_IO, LogFlags::TIME_READ), 0);
        file_builder.open_as(file_name, OpenMode::Read)?
    };

    let config = HDF5Config {
        timestamp_as_rfc3339: file
            .attr("config_timestamp_as_rfc3339")
            .and_then(|config| config.read_scalar::<bool>())
            .unwrap_or(true),
        multiple_channel_datasets: file
            .attr("config_multiple_channel_datasets")
            .and_then(|config| config.read_scalar::<bool>())
            .unwrap_or(true),
        cache_size: args.cache_size,
    };
    debug!("File config: {config:?}");

    let read_sequence: Vec<ReadCommand> = serde_json::from_str(&args.read)?;
    if read_sequence.iter().any(|read_command| {
        matches!(
            read_command,
            ReadCommand::TimestampRange(..) | ReadCommand::TimestampCount(..)
        )
    }) {
        return Err(Error::ConfigReadSeqTimestampInTimestampAsRfc3339Attrib);
    }

    let digitisers = Hdf5Digitiser::open_from(file, config)?
        .into_iter()
        .map(|digitiser| DigitiserReader::new(client_config, &read_sequence, digitiser))
        .collect::<Result<Vec<_>, Error>>()?;

    let digitiser_present = digitisers
        .iter()
        .map(|d| d.digitiser().get_id())
        .collect::<Vec<_>>();

    let mut digitisers = digitisers
        .into_iter()
        .filter(|d| d.is_id_contained_in(&args.digitizer_id))
        .collect::<Vec<_>>();

    // Run each command.
    for command_index in 0..read_sequence.len() {
        info!("Executing read command {command_index}");
        // Obtain the number of indices as the smallest across each digitiser.
        let num_indices = digitisers
            .iter()
            .map(|digitiser| digitiser.get_command(command_index).len())
            .min()
            .ok_or_else(|| Error::NoDigitisersSelected(digitiser_present.clone()))?;

        for index in 0..num_indices {
            read_hdf5_at_index(
                &mut digitisers,
                trace_topic,
                key,
                &args,
                command_index,
                index,
            )
            .await?;
        }
    }

    Ok(())
}

/// Read the messages at the given index, in the given slice of `DigitiserReaders` and produce them to the broker.
///
/// # Parameters
/// - digitisers: the slice of digitisers to operate on.
/// - trace_topic: the Kafka topic to produce to.
/// - key: the text to use for the produced message's key.
/// - args: the cli args specific to `hdf5` mode.
/// - index: the index of the message to read.
#[tracing::instrument(skip_all)]
async fn read_hdf5_at_index(
    digitisers: &mut [DigitiserReader],
    trace_topic: &str,
    key: &str,
    args: &Hdf5,
    command_index: usize,
    index: usize,
) -> Result<(), Error> {
    let mut spanned_digitisers = digitisers
        .iter_mut()
        .map(|digitiser| SpanWrapper::<_>::new(info_span!("Digitiser"), digitiser))
        .collect::<Vec<_>>();

    spanned_digitisers.iter_mut().for_each(|spanned_digitiser| {
        spanned_digitiser
            .span()
            .get()
            .expect("Digitiser has span, this should never fail.")
            .clone()
            .in_scope(|| spanned_digitiser.ensure_elements_cached(command_index, index));
    });

    spanned_digitisers
        .par_iter_mut()
        .map(|spanned_digitiser| {
            let span = spanned_digitiser.span().get().expect("Digitiser has span");
            span.in_scope(|| {
                spanned_digitiser.read_at_index(trace_topic, key, args, command_index, index)
            })
        })
        .collect::<Result<Vec<_>, Error>>()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::OverwriteFields;

    use super::*;
    use digital_muon_streaming_types::{
        dat2_digitizer_analog_trace_v2_generated::root_as_digitizer_analog_trace_message,
        flatbuffers::FlatBufferBuilder,
    };
    use std::{fs::File, io::Read};

    #[tokio::test]
    async fn test_malformed_read_field() {
        let config = ClientConfig::new();
        assert!(matches!(
            read_hdf5_file(
                "test_assets/test.hdf5".into(),
                &config,
                "",
                "",
                Hdf5 {
                    summary_only: false,
                    read: "".into(),
                    digitizer_id: vec![],
                    cache_size: None,
                    sample_rate: 0,
                    overwrite_fields: OverwriteFields::default()
                }
            )
            .await
            .expect_err("This function return Err, this should never fail."),
            Error::Json(..)
        ));
    }

    #[test]
    fn test() {
        let file = hdf5::File::open_as(
            PathBuf::from_str("test_assets/test.hdf5").unwrap(),
            OpenMode::Read,
        )
        .unwrap();
        let config = HDF5Config {
            timestamp_as_rfc3339: file
                .attr("config_timestamp_as_rfc3339")
                .and_then(|config| config.read_scalar::<bool>())
                .unwrap_or(true),
            multiple_channel_datasets: file
                .attr("config_multiple_channel_datasets")
                .and_then(|config| config.read_scalar::<bool>())
                .unwrap_or(true),
            cache_size: None,
        };

        let digitisers = Hdf5Digitiser::open_from(file, config).unwrap();
        assert_eq!(digitisers.len(), 1);

        let mut fbb = FlatBufferBuilder::new();
        // First Message
        assert!(
            digitisers[0]
                .create_message(&mut fbb, 0, 1_000_000_000, &OverwriteFields::default())
                .is_ok()
        );
        let dat_test = root_as_digitizer_analog_trace_message(fbb.unfinished_data()).unwrap();

        let data = {
            let mut file = File::open("test_assets/test.dat2").unwrap();
            let mut data = Vec::new();
            file.read_to_end(&mut data).unwrap();
            data
        };

        let dat_true = root_as_digitizer_analog_trace_message(&data).unwrap();
        assert_eq!(dat_test.digitizer_id(), dat_true.digitizer_id());
        assert_eq!(
            dat_test.metadata().frame_number(),
            dat_true.metadata().frame_number()
        );
        assert_eq!(
            dat_test.metadata().period_number(),
            dat_true.metadata().period_number()
        );
        assert_eq!(
            dat_test.metadata().protons_per_pulse(),
            dat_true.metadata().protons_per_pulse()
        );
        assert_eq!(dat_test.metadata().running(), dat_true.metadata().running());
        assert_eq!(
            dat_test.metadata().timestamp(),
            dat_true.metadata().timestamp()
        );
        for (channel_test, channel_true) in dat_test
            .channels()
            .unwrap()
            .iter()
            .zip(dat_true.channels().unwrap().iter())
        {
            assert_eq!(channel_test.channel(), channel_true.channel());
            assert!(channel_test.voltage().is_some());
            assert_eq!(
                channel_test.voltage().unwrap().iter().collect::<Vec<_>>(),
                channel_true.voltage().unwrap().iter().collect::<Vec<_>>()
            );
        }

        // Second Message
        fbb.reset();
        assert!(
            digitisers[0]
                .create_message(&mut fbb, 1, 1_000_000_000, &OverwriteFields::default())
                .is_ok()
        );
        let dat_test = root_as_digitizer_analog_trace_message(fbb.unfinished_data()).unwrap();

        let data = {
            let mut file = File::open("test_assets/test.dat2").unwrap();
            let mut data = Vec::new();
            file.read_to_end(&mut data).unwrap();
            data
        };

        let dat_true = root_as_digitizer_analog_trace_message(&data).unwrap();
        assert_eq!(dat_test.digitizer_id(), dat_true.digitizer_id());
        assert_eq!(
            dat_test.metadata().frame_number(),
            dat_true.metadata().frame_number()
        );
        assert_eq!(
            dat_test.metadata().period_number(),
            dat_true.metadata().period_number()
        );
        assert_eq!(
            dat_test.metadata().protons_per_pulse(),
            dat_true.metadata().protons_per_pulse()
        );
        assert_eq!(dat_test.metadata().running(), dat_true.metadata().running());
        assert_eq!(
            dat_test.metadata().timestamp(),
            dat_true.metadata().timestamp()
        );
        for (channel_test, channel_true) in dat_test
            .channels()
            .unwrap()
            .iter()
            .zip(dat_true.channels().unwrap().iter())
        {
            assert_eq!(channel_test.channel(), channel_true.channel());
            assert!(channel_test.voltage().is_some());
            assert_eq!(
                channel_test.voltage().unwrap().iter().collect::<Vec<_>>(),
                channel_true.voltage().unwrap().iter().collect::<Vec<_>>()
            );
        }
    }
}
