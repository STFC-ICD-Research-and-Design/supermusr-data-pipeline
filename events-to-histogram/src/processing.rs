use ndarray::array;
use ndarray_stats::histogram::{self, Bins, Edges, Grid};
use std::collections::HashMap;
use supermusr_common::{Channel, Time};
use supermusr_streaming_types::{
    dev1_digitizer_event_v1_generated::DigitizerEventListMessage,
    flatbuffers::FlatBufferBuilder,
    frame_metadata_v1_generated::{FrameMetadataV1, FrameMetadataV1Args},
    hst1_histogram_v1_generated::{
        finish_histogram_message_buffer, Histogram, HistogramArgs, HistogramMessage,
        HistogramMessageArgs,
    },
};
use tracing::{info, warn};

pub(crate) fn make_bins_edges(start: Time, stop: Time, width: Time) -> Edges<Time> {
    let mut edges = vec![start];
    let mut i = start;
    while i < stop {
        i += width;
        edges.push(i);
    }
    Edges::from(edges)
}

struct HistogramCollection {
    grid: Grid<Time>,
    channels: HashMap<Channel, histogram::Histogram<Time>>,
}

impl HistogramCollection {
    fn new(edges: Edges<Time>) -> Self {
        let bins = Bins::new(edges);
        Self {
            grid: Grid::from(vec![bins]),
            channels: Default::default(),
        }
    }

    fn record(&mut self, channel: Channel, time: Time) {
        if let std::collections::hash_map::Entry::Vacant(e) = self.channels.entry(channel) {
            e.insert(histogram::Histogram::new(self.grid.clone()));
        }

        if self
            .channels
            .get_mut(&channel)
            .expect("Channel histogram should be created")
            .add_observation(&array![time])
            .is_err()
        {
            warn!("Bin not found for time {}", time);
        }
    }
}

pub(crate) fn process(
    trace: &DigitizerEventListMessage,
    time_bin_width: Time,
    time_bin_edges: Edges<Time>,
) -> Vec<u8> {
    info!(
        "Dig ID: {}, Metadata: {:?}",
        trace.digitizer_id(),
        trace.metadata()
    );

    let mut fbb = FlatBufferBuilder::new();

    let mut histograms = HistogramCollection::new(time_bin_edges);
    for (time, channel) in std::iter::zip(trace.time().unwrap(), trace.channel().unwrap()) {
        histograms.record(channel, time);
    }

    let metadata = FrameMetadataV1Args {
        frame_number: trace.metadata().frame_number(),
        period_number: trace.metadata().period_number(),
        running: trace.metadata().running(),
        protons_per_pulse: trace.metadata().protons_per_pulse(),
        timestamp: trace.metadata().timestamp(),
        veto_flags: trace.metadata().veto_flags(),
    };
    let metadata = FrameMetadataV1::create(&mut fbb, &metadata);

    let mut channels = Vec::default();
    for (ch, hist) in histograms.channels {
        let counts = fbb.create_vector(hist.counts().mapv(|v| v as u16).as_slice().unwrap());
        let args = HistogramArgs {
            channel: ch,
            counts: Some(counts),
        };
        channels.push(Histogram::create(&mut fbb, &args));
    }
    let channels = Some(fbb.create_vector(channels.as_slice()));

    let message = HistogramMessageArgs {
        metadata: Some(metadata),
        bin_width: time_bin_width,
        channels,
    };
    let message = HistogramMessage::create(&mut fbb, &message);
    finish_histogram_message_buffer(&mut fbb, message);

    fbb.finished_data().to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use supermusr_streaming_types::{
        dev1_digitizer_event_v1_generated::{
            finish_digitizer_event_list_message_buffer, root_as_digitizer_event_list_message,
            DigitizerEventListMessage, DigitizerEventListMessageArgs,
        },
        frame_metadata_v1_generated::{FrameMetadataV1, FrameMetadataV1Args, GpsTime},
        hst1_histogram_v1_generated::{
            histogram_message_buffer_has_identifier, root_as_histogram_message,
        },
    };

    #[test]
    fn test_make_bin_edges() {
        let edges = make_bins_edges(0, 10, 2);
        assert_eq!(Edges::from(vec![0, 2, 4, 6, 8, 10]), edges);
    }

    #[test]
    fn test_make_bin_edges_2() {
        let edges = make_bins_edges(0, 10, 3);
        assert_eq!(Edges::from(vec![0, 3, 6, 9, 12]), edges);
    }

    #[test]
    fn test_full_message() {
        tracing_subscriber::fmt::init();

        let mut fbb = FlatBufferBuilder::new();

        let time: GpsTime = Utc::now().into();

        let metadata = FrameMetadataV1Args {
            frame_number: 0,
            period_number: 0,
            protons_per_pulse: 0,
            running: true,
            timestamp: Some(&time),
            veto_flags: 0,
        };
        let metadata = FrameMetadataV1::create(&mut fbb, &metadata);

        let channel = vec![0, 0, 0, 0, 1, 1, 1, 1, 0, 0];
        let time = vec![0, 5, 1, 8, 1, 8, 10, 1, 9, 6];

        let event_count = time.len();
        let channel = Some(fbb.create_vector::<u32>(&channel));
        let time = Some(fbb.create_vector::<u32>(&time));
        let voltage = Some(fbb.create_vector::<u16>(&vec![0; event_count]));

        let message = DigitizerEventListMessageArgs {
            digitizer_id: 0,
            metadata: Some(metadata),
            time,
            channel,
            voltage,
        };
        let message = DigitizerEventListMessage::create(&mut fbb, &message);
        finish_digitizer_event_list_message_buffer(&mut fbb, message);

        let message = fbb.finished_data().to_vec();
        let message = root_as_digitizer_event_list_message(&message).unwrap();

        let bin_width = 2;
        let edges = make_bins_edges(0, 10, bin_width);
        let result = process(&message, bin_width, edges);

        assert!(histogram_message_buffer_has_identifier(&result));
        let message = root_as_histogram_message(&result).unwrap();

        assert_eq!(bin_width, message.bin_width());

        assert_eq!(2, message.channels().unwrap().len());

        for i in [0, 1] {
            match message.channels().unwrap().get(i).channel() {
                0 => {
                    assert_eq!(
                        vec![2, 0, 1, 1, 2],
                        message
                            .channels()
                            .unwrap()
                            .get(i)
                            .counts()
                            .unwrap()
                            .iter()
                            .collect::<Vec<_>>()
                    );
                }
                1 => {
                    assert_eq!(
                        vec![2, 0, 0, 0, 1],
                        message
                            .channels()
                            .unwrap()
                            .get(i)
                            .counts()
                            .unwrap()
                            .iter()
                            .collect::<Vec<_>>()
                    );
                }
                ch => panic!("Unexpected channel number: {ch}"),
            }
        }
    }
}
