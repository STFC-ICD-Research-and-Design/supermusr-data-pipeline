use crate::{
    OverwriteFields,
    hdf5trace::{
        Error,
        cached_dataset::CachedDataset,
        channel::{Hdf5AllChannels, Hdf5Channel},
        extract_from_dataset_name,
    },
};
use chrono::{DateTime, Datelike, Utc};
use digital_muon_common::{Channel, DigitizerId, FrameNumber};
use digital_muon_streaming_types::{
    dat2_digitizer_analog_trace_v2_generated::{
        DigitizerAnalogTraceMessage, DigitizerAnalogTraceMessageArgs,
        finish_digitizer_analog_trace_message_buffer,
    },
    flatbuffers::{FlatBufferBuilder, WIPOffset},
    frame_metadata_v2_generated::{FrameMetadataV2, FrameMetadataV2Args, GpsTime},
};
use hdf5::{File, Group, types::VarLenUnicode};
use ndarray::Array1;
use tracing::{info, warn};

/// Identifier to use for hdf5 groups of the form ".../channel_index".
const CHANNEL: &str = "channel";
/// Identifier to use for hdf5 groups of the form ".../digitiser_index".
const DIGITISER: &str = "digitiser";

#[derive(Default, Debug)]
pub(crate) struct HDF5Config {
    /// True if the timestamp is stored as a RFC3339 string, otherwise ns since epoch.
    pub(crate) timestamp_as_rfc3339: bool,
    /// True if channels are stored in separate datasets, false if channels are stored as 2D array.
    pub(crate) multiple_channel_datasets: bool,
    /// If set, the amount of a dataset to cache before usage. This only applies to channel trace data,
    /// and possibly timestamp data (if it is stored as string values).
    pub(crate) cache_size: Option<usize>,
}

/// Encapsulates timestamps.
enum Timestamps {
    /// Timestamps are stored as strings.
    RFC3999(CachedDataset<VarLenUnicode>),
    /// Timestamps are stored as nanoseconds since epoch values.
    EpochNS(Array1<i64>),
}

impl Timestamps {
    /// Given a timestamp, determine the index in the list of traces where the frame is located.
    /// 
    /// Note this is only implemented for trace files whose `HDF5Config::timestamp_as_rfc3339` flag is `false`.
    ///
    /// # Parameters
    /// - timestamp: the timestamp to find.
    pub(crate) fn get_index_from_timestamp(
        &self,
        timestamp: &DateTime<Utc>,
    ) -> Result<usize, Error> {
        let ns = timestamp.timestamp_nanos_opt()
            .ok_or(Error::TimestampNotFound(timestamp.to_rfc3339()))?;
        match self {
            Timestamps::RFC3999(_cached_dataset) => {
                unimplemented!()
            },
            Timestamps::EpochNS(array_base) => {
                array_base
                    .iter()
                    .position(|v| ns.eq(v))
                    .or_else(|| array_base.iter().position(|v| ns.le(v)))
                    .ok_or(Error::TimestampNotFound(timestamp.to_rfc3339()))
            },
        }
    }
}

/// Encapsulates the channel trace data, as either a single dataset, or multiple groups, depending on the file format.
enum Channels {
    /// The trace data is stored in multiple groups, one per channel.
    Multiple(Vec<Hdf5Channel>),
    /// All trace data is stored in a single three-dimensional dataset.
    Single(Hdf5AllChannels),
}

/// Encapsulates the metadata and the hdf5 structures of a digitiser in a hdf5 file.
pub(crate) struct Hdf5Digitiser {
    /// The id of the digitiser.
    digitiser_id: DigitizerId,
    /// The list of periods numbers for each digitiser message.
    period_numbers: Array1<u64>,
    /// The list of frame numbers for each digitiser message.
    frame_numbers: Array1<FrameNumber>,
    /// The list of sample rates for each digitiser message.
    ///
    /// This is optional, for legacy compatability.
    sample_rates: Option<Array1<u64>>,
    /// The list of running flags for each digitiser message.
    ///
    /// This is optional, for legacy compatability.
    running: Option<Array1<bool>>,
    /// The list of protons per pulse for each digitiser message.
    ///
    /// This is optional, for legacy compatability.
    protons_per_pulse: Option<Array1<u8>>,
    /// The list of veto flags for each digitiser message.
    ///
    /// This is optional, for legacy compatability.
    veto_flags: Option<Array1<u16>>,
    /// The list of timestamps for each digitiser message.
    timestamps: Timestamps,
    /// The channel trace data.
    channels: Channels,
    /// The number of frames worth of data this instance constains.
    num_frames: usize,
}

impl Hdf5Digitiser {
    /// Creates a vector of instances, one for each `digitiser_index` group found in the given hdf5 file.
    ///
    /// # Parameters
    /// - file: the file to load from.
    /// - config: the configuration settings to use when loading.
    pub(crate) fn open_from(file: File, config: HDF5Config) -> Result<Vec<Hdf5Digitiser>, Error> {
        let mut digitisers = Vec::<Hdf5Digitiser>::new();
        for group in file
            .groups()
            .expect("Groups should be accessible, this should never fail.")
        {
            digitisers.push(Self::open_digitiser(group, &config)?);
        }
        Ok(digitisers)
    }

    /// Creates an instances from the given hdf5 group.
    ///
    /// # Parameters
    /// - group: the group to load from.
    /// - config: the configuration settings to use when loading.
    fn open_digitiser(group: Group, config: &HDF5Config) -> Result<Self, Error> {
        let digitiser_id: DigitizerId = extract_from_dataset_name(group.name(), DIGITISER)?;

        let frame_numbers = group.dataset("frame_number")?.read_1d()?;
        let num_frames = frame_numbers.len();

        let period_numbers = group.dataset("period_number")?.read_1d()?;
        assert_eq!(period_numbers.len(), num_frames);

        let protons_per_pulse = group
            .dataset("protons_per_pulse")
            .ok()
            .map(|dataset| dataset.read_1d())
            .transpose()?;
        protons_per_pulse
            .as_ref()
            .inspect(|protons_per_pulse| assert_eq!(protons_per_pulse.len(), num_frames));
        let sample_rates = group
            .dataset("sample_rate")
            .ok()
            .map(|dataset| dataset.read_1d())
            .transpose()?;
        sample_rates
            .as_ref()
            .inspect(|sample_rates| assert_eq!(sample_rates.len(), num_frames));
        let veto_flags = group
            .dataset("veto_flags")
            .ok()
            .map(|dataset| dataset.read_1d())
            .transpose()?;
        veto_flags
            .as_ref()
            .inspect(|veto_flags| assert_eq!(veto_flags.len(), num_frames));
        let running = group
            .dataset("running")
            .ok()
            .map(|dataset| dataset.read_1d())
            .transpose()?;
        running
            .as_ref()
            .inspect(|running| assert_eq!(running.len(), num_frames));

        let timestamps = if config.timestamp_as_rfc3339 {
            let timestamps =
                CachedDataset::new(group.dataset("timestamp")?, config.cache_size.as_ref())?;
            assert_eq!(timestamps.get_num_elements(), num_frames);
            Timestamps::RFC3999(timestamps)
        } else {
            let timestamps: Array1<_> = group.dataset("timestamp")?.read_1d()?;
            assert_eq!(timestamps.len(), num_frames);
            Timestamps::EpochNS(timestamps)
        };

        let channels = if config.multiple_channel_datasets {
            let channel_datasets = group
                .datasets()
                .expect("Datasets should be accessible, this should never fail.")
                .into_iter()
                .filter_map(|dataset| {
                    (!["frame_number", "period_number", "timestamp"]
                        .contains(&dataset.name().split('/').next_back()?))
                    .then_some(dataset)
                });

            let channels = channel_datasets
                .into_iter()
                .map(|dataset| {
                    let channel: Channel = extract_from_dataset_name(dataset.name(), CHANNEL)?;
                    assert_eq!(*dataset.shape(), vec![num_frames]);
                    let channel = Hdf5Channel::new(
                        channel,
                        CachedDataset::new(dataset, config.cache_size.as_ref())?,
                    );
                    Ok(channel)
                })
                .collect::<Result<Vec<_>, Error>>()?;
            Channels::Multiple(channels)
        } else {
            let channels = group.dataset("channels")?.read_1d()?;
            let trace_index = group.dataset("trace_index")?.read_1d()?;
            let traces = group.dataset("traces")?;
            info!(
                "Digitiser {digitiser_id} has traces dataset of size {:?}.",
                traces.shape()
            );
            Channels::Single(Hdf5AllChannels::new(channels, trace_index, traces))
        };
        Ok(Hdf5Digitiser {
            digitiser_id,
            period_numbers,
            frame_numbers,
            timestamps,
            protons_per_pulse,
            running,
            sample_rates,
            veto_flags,
            channels,
            num_frames,
        })
    }

    /// Given a frame number, determine the index in the list of traces where the frame is located.
    ///
    /// # Parameters
    /// - frame_number: the frame number to find.
    ///
    /// # Returns
    /// Returns `None` if the frame number is not found.
    pub(crate) fn get_index_from_frame_number(
        &self,
        frame_number: FrameNumber,
    ) -> Result<usize, Error> {
        self.frame_numbers
            .iter()
            .position(|v| frame_number.eq(v))
            .or_else(|| self.frame_numbers.iter().position(|v| frame_number.le(v)))
            .ok_or(Error::FrameNumberNotFound(frame_number))
    }
    
    /// Given a timestamp, determine the index in the list of traces where the frame is located.
    /// 
    /// Note this is only implemented for trace files whose `HDF5Config::timestamp_as_rfc3339` flag is `false`.
    ///
    /// # Parameters
    /// - timestamp: the timestamp to find.
    pub(crate) fn get_index_from_timestamp(
        &self,
        timestamp: &DateTime<Utc>,
    ) -> Result<usize, Error> {
        self.timestamps.get_index_from_timestamp(timestamp)
    }

    /// Given an index, ensure the necessary data is in the cache.
    /// This should each time before the `create_message` method is used.
    ///
    /// This method is idempotent, so does nothing if the required index is already cached.
    ///
    /// # Parameters
    /// - index: the index to ensure is cached.
    #[tracing::instrument(skip_all)]
    pub(crate) fn ensure_elements_cached(&mut self, index: usize) {
        if let Timestamps::RFC3999(timestamps) = &mut self.timestamps {
            timestamps.ensure_elements_cached(index);
        }
        if let Channels::Multiple(hdf5_channels) = &mut self.channels {
            hdf5_channels
                .iter_mut()
                .for_each(|channel: &mut Hdf5Channel| channel.ensure_elements_cached(index))
        }
    }

    /// Returns the number of frames.
    pub(crate) fn get_num_frames(&self) -> usize {
        self.num_frames
    }

    /// Returns the id of the digitiser.
    pub(crate) fn get_id(&self) -> DigitizerId {
        self.digitiser_id
    }

    /// Loads a FlatBufferBuilder with a new DigitizerAnalogTraceMessage instance with a custom timestamp.
    ///
    /// # Parameters
    /// - fbb: mutable reference to the FlatBufferBuilder to use.
    /// - index: the index of the trace to use.
    /// - sample_rate: the number of measurements in each channel.
    /// - shift_timestamp_date_to_today: if true, changes timestamp date to current day.
    ///
    /// # Returns
    /// A string result, or an error.
    #[tracing::instrument(skip_all)]
    pub(crate) fn create_message(
        &self,
        fbb: &mut FlatBufferBuilder<'_>,
        index: usize,
        sample_rate_default: u64,
        overwrite_fields: &OverwriteFields,
    ) -> Result<(), Error> {
        if index >= self.num_frames {
            Err(Error::FrameIndexTooLarge(index, self.num_frames))?;
        }
        fbb.reset();

        let frame_number = *self
            .frame_numbers
            .get(index)
            .expect("Index should be in range, this should never fail.");
        let period_number = *self
            .period_numbers
            .get(index)
            .expect("Index should be in range, this should never fail.");
        let mut timestamp: DateTime<Utc> = match &self.timestamps {
            Timestamps::RFC3999(timestamps) => timestamps.get_element(index).parse()?,
            Timestamps::EpochNS(timestamps) => DateTime::from_timestamp_nanos(
                *timestamps
                    .get(index)
                    .expect("Index should be in range, this should never fail."),
            ),
        };
        if overwrite_fields.shift_to_today {
            timestamp = timestamp
                .with_day(Utc::now().day())
                .expect("Timestamp with current day should be possible, this should never fail.")
                .with_month(Utc::now().month())
                .expect("Timestamp with current month should be possible, this should never fail.")
                .with_year(Utc::now().year())
                .expect("Timestamp with current year should be possible, this should never fail.");
        }

        let channels = match &self.channels {
            Channels::Multiple(hdf5_channels) => {
                let trace = hdf5_channels
                    .iter()
                    .map(|c| c.create_channel(fbb, index))
                    .collect::<Vec<_>>();
                fbb.create_vector_from_iter(trace.iter())
            }
            Channels::Single(hdf5_channel) => hdf5_channel.create_channels(fbb, index),
        };

        let protons_per_pulse = self.protons_per_pulse.as_ref().map(|protons_per_pulse| {
            *protons_per_pulse
                .get(index)
                .expect("Index should be in range, this should never fail.")
        });

        let sample_rate = self.sample_rates.as_ref().map(|sample_rates| {
            *sample_rates
                .get(index)
                .expect("Index should be in range, this should never fail.")
        });

        let running = self.running.as_ref().map(|running| {
            *running
                .get(index)
                .expect("Index should be in range, this should never fail.")
        });

        let veto_flags = self.veto_flags.as_ref().map(|veto_flags| {
            *veto_flags
                .get(index)
                .expect("Index should be in range, this should never fail.")
        });

        let gps_time = GpsTime::from(timestamp);
        let metadata: FrameMetadataV2Args = FrameMetadataV2Args {
            frame_number,
            period_number: overwrite_fields
                .overwrite_period_number
                .unwrap_or(period_number),
            protons_per_pulse: overwrite_fields
                .overwrite_protons_per_pulse
                .unwrap_or(protons_per_pulse.unwrap_or_default()),
            running: overwrite_fields
                .overwrite_running
                .unwrap_or(running.unwrap_or(true)),
            timestamp: Some(&gps_time),
            veto_flags: overwrite_fields
                .overwrite_veto_flags
                .unwrap_or(veto_flags.unwrap_or_default()),
        };
        let metadata: WIPOffset<FrameMetadataV2> = FrameMetadataV2::create(fbb, &metadata);

        let message = DigitizerAnalogTraceMessageArgs {
            digitizer_id: self.digitiser_id,
            metadata: Some(metadata),
            sample_rate: sample_rate.unwrap_or(sample_rate_default),
            channels: Some(channels),
        };
        let message = DigitizerAnalogTraceMessage::create(fbb, &message);
        finish_digitizer_analog_trace_message_buffer(fbb, message);
        Ok(())
    }
}
