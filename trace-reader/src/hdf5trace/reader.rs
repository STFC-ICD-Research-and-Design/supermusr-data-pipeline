use digital_muon_common::{DigitizerId, FrameNumber};
use serde::Deserialize;

use digital_muon_streaming_types::flatbuffers::FlatBufferBuilder;
use rdkafka::{
    ClientConfig,
    producer::{BaseRecord, DefaultProducerContext, Producer, ThreadedProducer},
    util::Timeout,
};
use std::ops::Range;
use tracing::{error, info, info_span};

use crate::{
    Hdf5,
    hdf5trace::{Error, Hdf5Digitiser},
};

/// Specifies a range of 
#[derive(Clone, Deserialize)]
pub(crate) enum ReadCommand {
    /// Read indices with frame number between these values.
    #[serde(rename = "f")]
    FrameRange(FrameNumber, FrameNumber),
    /// Read indices starting from the index with frame number equal to the first value and with count specified by the second.
    #[serde(rename = "fc")]
    FrameCount(FrameNumber, usize),
    /// Read indices between these values. 
    #[serde(rename = "i")]
    IndexRange(usize, usize),
    /// Read indices starting from the first value and with count specified by the second.
    #[serde(rename = "ic")]
    IndexCount(usize, usize),
    /// Read indices with timestamps between these values. FIXME: To Implement.
    #[serde(rename = "t")]
    TimestampRange(String, String),
    /// Read indices starting from the index with timestamp equal to the first value and with count specified by the second. FIXME: To Implement.
    #[serde(rename = "tc")]
    TimestampCount(String, usize),
    /// Read all available indices.
    #[serde(rename = "all")]
    All,
}

/// Encapsulates the tools needed to read the digitiser messages from a hdf5 file and produce them to the kafka broker.
pub(crate) struct DigitiserReader {
    /// Encapsulates the metadata and link to the hdf5 file for the digitiser messages.
    digitiser: Hdf5Digitiser,
    /// The sequence of read_instructions to run through
    read_sequence: Vec<Range<usize>>,
    /// The kafka producer this digitiser uses.
    producer: ThreadedProducer<DefaultProducerContext>,
}

impl DigitiserReader {
    /// Creates a new instance.
    ///
    /// # Parameters
    /// - client_config: the kafka config settings to use for the produer.
    /// - args: the cli args specific to `hdf5` mode.
    /// - digitiser:
    pub(crate) fn new(
        client_config: &ClientConfig,
        read_sequence: &[ReadCommand],
        digitiser: Hdf5Digitiser,
    ) -> Result<Self, Error> {
        let read_sequence = read_sequence
            .iter()
            .map(|command| match command {
                &ReadCommand::FrameRange(from, to) => Ok(digitiser
                    .get_index_from_frame_number(from)?
                    ..digitiser.get_index_from_frame_number(to)?),
                &ReadCommand::FrameCount(from, count) => Ok(digitiser
                    .get_index_from_frame_number(from)?
                    ..(digitiser.get_index_from_frame_number(from)?) + count),
                &ReadCommand::IndexRange(from, to) => Ok(from..to),
                &ReadCommand::IndexCount(from, count) => Ok(from..(from + count)),
                ReadCommand::TimestampRange(_from, _to) => unimplemented!(),
                ReadCommand::TimestampCount(_from, _count) => unimplemented!(),
                ReadCommand::All => Ok(0..digitiser.get_num_frames()),
            })
            .collect::<Result<Vec<_>, Error>>()?;

        info!(
            "Reader for digitiser {} read command sequence : {read_sequence:?}.",
            digitiser.get_id()
        );
        let producer = client_config.create()?;
        Ok(Self {
            digitiser,
            read_sequence,
            producer,
        })
    }

    /// Read the digitiser message at the given index and produce it to the broker.
    ///
    /// # Parameters
    /// - trace_topic: the Kafka topic to produce to.
    /// - key: the text to use for the produced message's key.
    /// - args: the cli args specific to `hdf5` mode.
    /// - index: the index of the message to read.
    pub(crate) fn read_at_index(
        &self,
        trace_topic: &str,
        key: &str,
        args: &Hdf5,
        command_index: usize,
        index: usize,
    ) -> Result<(), Error> {
        let mut fbb = FlatBufferBuilder::new();
        self.digitiser.create_message(
            &mut fbb,
            self.read_sequence
                .get(command_index)
                .expect("This should never fail")
                .start
                + index,
            args.sample_rate,
            &args.overwrite_fields,
        )?;
        info_span!("Send").in_scope(|| self.send_record(&mut fbb, trace_topic, key));
        Ok(())
    }

    /// Sends the FlatBuffer payload to the desired Kafka topic.
    ///
    /// # Parameters
    /// - fbb: mutable reference to the FlatBufferBuilder to use.
    /// - trace_topic: the Kafka topic to produce to.
    /// - key: the text to use for the produced message's key.
    pub(crate) fn send_record(&self, fbb: &mut FlatBufferBuilder, trace_topic: &str, key: &str) {
        let base_record = BaseRecord::to(trace_topic)
            .payload(fbb.finished_data())
            .key(key);

        let mut result = self.producer.send(base_record);
        while let Err((_, base_record)) = result {
            result = self.producer.send(base_record);
        }
    }

    pub(crate) fn is_id_contained_in(&self, ids: &[DigitizerId]) -> bool {
        if ids.is_empty() {
            true
        } else {
            ids.contains(&self.digitiser.get_id())
        }
    }

    /// Given an index, ensure the necessary data is in the cache.
    /// This should each time before the `create_message` method is used.
    ///
    /// This method is idempotent, so does nothing if the required index is already cached.
    ///
    /// # Parameters
    /// - index: the index to ensure is cached.
    #[tracing::instrument(skip_all)]
    pub(crate) fn ensure_elements_cached(&mut self, command_index: usize, index: usize) {
        self.digitiser.ensure_elements_cached(
            self.read_sequence
                .get(command_index)
                .expect("This should never fail.")
                .start
                + index,
        );
    }

    pub(crate) fn digitiser(&self) -> &Hdf5Digitiser {
        &self.digitiser
    }

    pub(crate) fn get_command(&self, command_index: usize) -> &Range<usize> {
        self.read_sequence
            .get(command_index)
            .expect("This should never fail.")
    }
}

impl Drop for DigitiserReader {
    fn drop(&mut self) {
        if let Err(e) = self.producer.flush(Timeout::Never) {
            error!("{e}");
        }
    }
}
