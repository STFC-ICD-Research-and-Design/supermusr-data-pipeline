use super::{partial::PartialFrame, AggregatedFrame};
use crate::data::{Accumulate, DigitiserData};
use std::{collections::VecDeque, fmt::Debug, time::Duration};
use supermusr_common::{record_metadata_fields_to_span, spanned::SpannedAggregator, DigitizerId};
use supermusr_streaming_types::FrameMetadata;
use tracing::{info_span, warn};

pub(crate) struct FrameCache<D: Debug> {
    ttl: Duration,
    expected_digitisers: Vec<DigitizerId>,

    frames: VecDeque<PartialFrame<D>>,
}

impl<D: Debug> FrameCache<D>
where
    DigitiserData<D>: Accumulate<D>,
{
    pub(crate) fn new(ttl: Duration, expected_digitisers: Vec<DigitizerId>) -> Self {
        Self {
            ttl,
            expected_digitisers,
            frames: Default::default(),
        }
    }

    #[tracing::instrument(skip_all, fields(
        digitiser_id = digitiser_id,
        metadata_timestamp = metadata.timestamp.to_rfc3339(),
        metadata_frame_number = metadata.frame_number,
        metadata_period_number = metadata.period_number,
        metadata_veto_flags = metadata.veto_flags,
        metadata_protons_per_pulse = metadata.protons_per_pulse,
        metadata_running = metadata.running
    ))]
    pub(crate) fn push<'a>(
        &'a mut self,
        digitiser_id: DigitizerId,
        metadata: &FrameMetadata,
        data: D,
    ) {
        let frame = {
            match self
                .frames
                .iter_mut()
                .find(|frame| frame.metadata.equals_ignoring_veto_flags(metadata))
            {
                Some(frame) => {
                    frame.push(digitiser_id, data);
                    frame.push_veto_flags(metadata.veto_flags);
                    frame
                }
                None => {
                    let mut frame = PartialFrame::<D>::new(self.ttl, metadata.clone());

                    // Initialise the span field
                    if let Err(e) = frame.span_init() {
                        warn!("Frame span initiation failed {e}")
                    }

                    frame.push(digitiser_id, data);
                    self.frames.push_back(frame);
                    self.frames
                        .back()
                        .expect("self.frames should be non-empty, this should never fails")
                }
            }
        };

        // Link this span with the frame aggregator span associated with `frame`
        if let Err(e) = frame.link_current_span(|| {
            let span = info_span!(target: "otel",
                "Digitiser Event List",
                "metadata_timestamp" = tracing::field::Empty,
                "metadata_frame_number" = tracing::field::Empty,
                "metadata_period_number" = tracing::field::Empty,
                "metadata_veto_flags" = tracing::field::Empty,
                "metadata_protons_per_pulse" = tracing::field::Empty,
                "metadata_running" = tracing::field::Empty,
            );
            record_metadata_fields_to_span!(metadata, span);
            span
        }) {
            warn!("Frame span linking failed {e}")
        }
    }

    pub(crate) fn poll(&mut self) -> Option<AggregatedFrame<D>> {
        // Find a frame which is completed
        if self
            .frames
            .front()
            .is_some_and(|frame| frame.is_complete(&self.expected_digitisers) | frame.is_expired())
        {
            let frame = self
                .frames
                .pop_front()
                .expect("self.frames should be non-empty, this should never fail");
            if let Err(e) = frame.end_span() {
                warn!("Frame span drop failed {e}")
            }
            Some(frame.into())
        } else {
            None
        }
    }

    pub(crate) fn get_num_partial_frames(&self) -> usize {
        self.frames.len()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::data::EventData;
    use chrono::Utc;

    #[test]
    fn one_frame_in_one_frame_out() {
        let mut cache = FrameCache::<EventData>::new(Duration::from_millis(100), vec![0, 1, 4, 8]);

        let frame_1 = FrameMetadata {
            timestamp: Utc::now(),
            period_number: 1,
            protons_per_pulse: 8,
            running: true,
            frame_number: 1728,
            veto_flags: 4,
        };

        assert!(cache.poll().is_none());

        assert_eq!(cache.get_num_partial_frames(), 0);
        cache.push(0, &frame_1, EventData::dummy_data(0, 5, &[0, 1, 2]));
        assert_eq!(cache.get_num_partial_frames(), 1);

        assert!(cache.poll().is_none());

        cache.push(1, &frame_1, EventData::dummy_data(0, 5, &[3, 4, 5]));

        assert!(cache.poll().is_none());

        cache.push(4, &frame_1, EventData::dummy_data(0, 5, &[6, 7, 8]));

        assert!(cache.poll().is_none());

        cache.push(8, &frame_1, EventData::dummy_data(0, 5, &[9, 10, 11]));

        {
            let frame = cache.poll().unwrap();
            assert_eq!(cache.get_num_partial_frames(), 0);

            assert_eq!(frame.metadata, frame_1);

            let mut dids = frame.digitiser_ids;
            dids.sort();
            assert_eq!(dids, &[0, 1, 4, 8]);

            assert_eq!(
                frame.digitiser_data,
                EventData::new(
                    vec![
                        0, 1, 2, 3, 4, 0, 1, 2, 3, 4, 0, 1, 2, 3, 4, 0, 1, 2, 3, 4, 0, 1, 2, 3, 4,
                        0, 1, 2, 3, 4, 0, 1, 2, 3, 4, 0, 1, 2, 3, 4, 0, 1, 2, 3, 4, 0, 1, 2, 3, 4,
                        0, 1, 2, 3, 4, 0, 1, 2, 3, 4
                    ],
                    vec![0; 60],
                    vec![
                        0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 4, 4, 4, 4, 4,
                        5, 5, 5, 5, 5, 6, 6, 6, 6, 6, 7, 7, 7, 7, 7, 8, 8, 8, 8, 8, 9, 9, 9, 9, 9,
                        10, 10, 10, 10, 10, 11, 11, 11, 11, 11
                    ],
                )
            );
        }

        assert!(cache.poll().is_none());
    }

    #[tokio::test]
    async fn one_frame_in_one_frame_out_missing_digitiser_timeout() {
        let mut cache = FrameCache::<EventData>::new(Duration::from_millis(100), vec![0, 1, 4, 8]);

        let frame_1 = FrameMetadata {
            timestamp: Utc::now(),
            period_number: 1,
            protons_per_pulse: 8,
            running: true,
            frame_number: 1728,
            veto_flags: 4,
        };

        assert!(cache.poll().is_none());

        cache.push(0, &frame_1, EventData::dummy_data(0, 5, &[0, 1, 2]));

        assert!(cache.poll().is_none());

        cache.push(1, &frame_1, EventData::dummy_data(0, 5, &[3, 4, 5]));

        assert!(cache.poll().is_none());

        cache.push(8, &frame_1, EventData::dummy_data(0, 5, &[9, 10, 11]));

        assert!(cache.poll().is_none());

        tokio::time::sleep(Duration::from_millis(105)).await;

        {
            let frame = cache.poll().unwrap();

            assert_eq!(frame.metadata, frame_1);

            let mut dids = frame.digitiser_ids;
            dids.sort();
            assert_eq!(dids, &[0, 1, 8]);

            assert_eq!(
                frame.digitiser_data,
                EventData::new(
                    vec![
                        0, 1, 2, 3, 4, 0, 1, 2, 3, 4, 0, 1, 2, 3, 4, 0, 1, 2, 3, 4, 0, 1, 2, 3, 4,
                        0, 1, 2, 3, 4, 0, 1, 2, 3, 4, 0, 1, 2, 3, 4, 0, 1, 2, 3, 4,
                    ],
                    vec![0; 45],
                    vec![
                        0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 4, 4, 4, 4, 4,
                        5, 5, 5, 5, 5, 9, 9, 9, 9, 9, 10, 10, 10, 10, 10, 11, 11, 11, 11, 11
                    ],
                )
            );
        }

        assert!(cache.poll().is_none());
    }

    #[test]
    fn test_metadata_equality() {
        let mut cache = FrameCache::<EventData>::new(Duration::from_millis(100), vec![1, 2]);

        let timestamp = Utc::now();
        let frame_1 = FrameMetadata {
            timestamp,
            period_number: 1,
            protons_per_pulse: 8,
            running: true,
            frame_number: 1728,
            veto_flags: 4,
        };

        let frame_2 = FrameMetadata {
            timestamp,
            period_number: 1,
            protons_per_pulse: 8,
            running: true,
            frame_number: 1728,
            veto_flags: 5,
        };

        assert_eq!(frame_1, frame_2);

        assert_eq!(cache.frames.len(), 0);
        assert!(cache.poll().is_none());

        cache.push(1, &frame_1, EventData::dummy_data(0, 5, &[0, 1, 2]));
        assert_eq!(cache.frames.len(), 1);
        assert!(cache.poll().is_none());

        cache.push(2, &frame_2, EventData::dummy_data(0, 5, &[0, 1, 2]));
        assert_eq!(cache.frames.len(), 1);
        assert!(cache.poll().is_some());
    }
}
