use std::iter::{repeat, once};

use itertools::Itertools;

use flatbuffers::{ForwardsUOffset, Vector};

use taos::{taos_query::common::views::TimestampView, ColumnView, Value};

use common::Intensity;
use streaming_types::dat1_digitizer_analog_trace_v1_generated::ChannelTrace;

use super::{framedata::FrameData, error_reporter::{self, TDEngineErrorReporter}};


/// Creates a timestamp view from the current frame_data object
pub(super) fn create_timestamp_view(frame_data : &FrameData) -> TimestampView {
    let frame_timestamp_ns = frame_data.timestamp.timestamp_nanos();
    let sample_time_ns = frame_data.sample_time.num_nanoseconds().unwrap();

    // Create the timestamps for each sample
    TimestampView::from_nanos(
        (0..frame_data.num_samples).map(|i| i as i64)
            .map(|i|frame_timestamp_ns + sample_time_ns * i)
            .collect())
}

/// Creates a vector of intensity values of size equal to the correct number of samples
/// These are extracted from the channel trace if available. If not then a vector of zero
/// Values is created
/// #Arguments
/// *channel - a reference to the channel trace to extract from
/// #Return
/// A vector of intensities
pub(super) fn create_voltage_values_from_channel_trace(frame_data : &FrameData, channel: &ChannelTrace) -> Vec<Intensity> {
    let voltage = channel.voltage().unwrap_or_default();
    if frame_data.num_samples == voltage.len() {
        // Can this be replaced with a pointer to the memory buffer? TODO
        voltage.iter().collect_vec()
    } else {
        let padding = repeat(Intensity::default())
            .take(frame_data.num_samples)
            .skip(voltage.len());

        voltage.iter()
            .chain(padding).collect_vec()
    }
    
}

/// CreateCreates a vector of column views which can be bound to a TDEngine statement
/// consisting of a timestamp view and the predefined number of channel views. If the
/// number of channel traces is greater than the predefined number then the surplus
/// channels are discarded. If the number of channel traces is insufficient then views
/// consisting of zero intensities are appended as neecessary.
/// #Arguments
/// *message - the DigitizerAnalogTraceMessage instance to extract from
/// #Return
/// A vector of column views
pub(super) fn create_column_views(frame_data : &FrameData, channels : &Vector<'_, ForwardsUOffset<ChannelTrace>>) -> Vec<ColumnView> {
    let timestamp_view = ColumnView::Timestamp(create_timestamp_view(frame_data));

    let num_channels = frame_data.num_channels;

    let null_channel_samples = repeat(Intensity::default()).take(frame_data.num_samples);
    let channel_padding = repeat(null_channel_samples)
        .take(num_channels)
        .skip(channels.len())
        .map(|v|ColumnView::from_unsigned_small_ints(v.collect_vec()));

    let channel_views = channels
        .iter()
        .map(|c|ColumnView::from_unsigned_small_ints(create_voltage_values_from_channel_trace(frame_data, &c)))
        .take(num_channels)                         // Cap the channel list at the given channel count
        .chain(channel_padding);                    // Append any additional channels if needed
    once(timestamp_view).chain(channel_views).collect_vec()
}

/// Creates a vector of taos_query values which contain the tags to be used for the tdengine
/// statement.
/// #Arguments
/// *channels - a flatbuffers vector of ChannelTraces from which the tags are created
/// #Returns
/// A vector of taos_query values
pub(super) fn create_tags(frame_data : &FrameData, error : &TDEngineErrorReporter, channels : &Vector<'_, ForwardsUOffset<ChannelTrace>>) -> Vec<Value> {
    let channel_padding = repeat(Value::UInt(0))
        .take(frame_data.num_channels)
        .skip(channels.len());

    let channel_id_tags = channels.iter()
        .map(|c|Value::UInt(c.channel()))
        .take(frame_data.num_channels)         // Cap the channel list at the given channel count
        .chain(channel_padding);                    // Append any additional channels if needed

    once(Value::UInt(frame_data.frame_number))
        .chain(channel_id_tags)
        .chain(once(Value::Bool(error.has_error())))
        .collect_vec()
}