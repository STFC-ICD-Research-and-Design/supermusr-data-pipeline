use crate::data::DigitiserData;
use std::fmt::Debug;
use std::time::Duration;
use supermusr_common::DigitizerId;
use supermusr_streaming_types::FrameMetadata;
use tokio::time::Instant;

pub(crate) trait PartialFrameLike<D>:
    Debug + AsRef<PartialFrame<D>> + AsMut<PartialFrame<D>>
{
    fn new(ttl: Duration, metadata: FrameMetadata) -> Self;
}

impl<D> AsRef<Self> for PartialFrame<D> {
    fn as_ref(&self) -> &Self {
        self
    }
}
impl<D> AsMut<Self> for PartialFrame<D> {
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}

impl<D: Debug> PartialFrameLike<D> for PartialFrame<D> {
    fn new(ttl: Duration, metadata: FrameMetadata) -> Self {
        let expiry = Instant::now() + ttl;

        Self {
            expiry,
            metadata,
            digitiser_data: Default::default(),
        }
    }
}

#[derive(Debug)]
pub(crate) struct PartialFrame<D> {
    expiry: Instant,

    pub(super) metadata: FrameMetadata,
    pub(super) digitiser_data: DigitiserData<D>,
}

impl<D> PartialFrame<D> {
    pub(super) fn digitiser_ids(&self) -> Vec<DigitizerId> {
        let mut cache_digitiser_ids: Vec<DigitizerId> =
            self.digitiser_data.iter().map(|i| i.0).collect();
        cache_digitiser_ids.sort();
        cache_digitiser_ids
    }

    pub(super) fn push(&mut self, digitiser_id: DigitizerId, data: D) {
        self.digitiser_data.push((digitiser_id, data));
    }

    pub(super) fn is_complete(&self, expected_digitisers: &[DigitizerId]) -> bool {
        self.digitiser_ids() == expected_digitisers
    }

    pub(super) fn is_expired(&self) -> bool {
        Instant::now() > self.expiry
    }
}
