use crate::data::{Accumulate, DigitiserData};
use std::{collections::VecDeque, fmt::Debug, time::Duration};
use supermusr_common::{
    spanned::{FindSpan, FindSpanMut, SpannedAggregator},
    DigitizerId,
};
use supermusr_streaming_types::FrameMetadata;

use super::{partial::PartialFrame, AggregatedFrame};

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

    pub(crate) fn push(&mut self, digitiser_id: DigitizerId, metadata: &FrameMetadata, data: D) {
        match self
            .frames
            .iter_mut()
            .find(|frame| frame.metadata.equals_ignoring_veto_flags(metadata))
        {
            Some(frame) => {
                frame.push(digitiser_id, data);
                frame.push_veto_flags(metadata.veto_flags)
            }
            None => {
                let mut frame = PartialFrame::<D>::new(self.ttl, metadata.clone());
                frame.push(digitiser_id, data);
                self.frames.push_back(frame);
            }
        }
    }

    pub(crate) fn poll(&mut self) -> Option<AggregatedFrame<D>> {
        match self.frames.front() {
            Some(frame) => {
                if frame.is_complete(&self.expected_digitisers) || frame.is_expired() {
                    Some(
                        self.frames
                            .pop_front()
                            .expect("cache should have a frame, this should never fail")
                            .into(),
                    )
                } else {
                    None
                }
            }
            None => None,
        }
    }
}

impl<D: Debug + 'static> FindSpan<PartialFrame<D>> for FrameCache<D> {
    type Key = FrameMetadata;
}

impl<D: Debug + 'static> FindSpanMut<PartialFrame<D>> for FrameCache<D> {
    fn find_span_mut(&mut self, metadata: FrameMetadata) -> Option<&mut impl SpannedAggregator> {
        self.frames
            .iter_mut()
            .find(|frame| frame.metadata.equals_ignoring_veto_flags(&metadata))
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

        cache.push(0, &frame_1, EventData::dummy_data(0, 5, &[0, 1, 2]));

        assert!(cache.poll().is_none());

        cache.push(1, &frame_1, EventData::dummy_data(0, 5, &[3, 4, 5]));

        assert!(cache.poll().is_none());

        cache.push(4, &frame_1, EventData::dummy_data(0, 5, &[6, 7, 8]));

        assert!(cache.poll().is_none());

        cache.push(8, &frame_1, EventData::dummy_data(0, 5, &[9, 10, 11]));

        {
            let frame = cache.poll().unwrap();

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
}
