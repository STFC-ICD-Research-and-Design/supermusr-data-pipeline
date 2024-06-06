use crate::data::DigitiserData;
use std::time::Duration;
use supermusr_common::spanned::{SpanOnce, Spanned, SpannedMut};
use supermusr_common::DigitizerId;
use supermusr_streaming_types::FrameMetadata;
use tokio::time::Instant;

pub(super) struct PartialFrame<D> {
    span: SpanOnce,
    expiry: Instant,

    pub(super) metadata: FrameMetadata,
    pub(super) digitiser_data: DigitiserData<D>,
}

impl<D> PartialFrame<D> {
    pub(super) fn new(ttl: Duration, metadata: FrameMetadata) -> Self {
        let expiry = Instant::now() + ttl;

        Self {
            span: SpanOnce::default(),
            expiry,
            metadata,
            digitiser_data: Default::default(),
        }
    }
    pub(super) fn digitiser_ids(&self) -> Vec<DigitizerId> {
        let mut cache_digitiser_ids: Vec<DigitizerId> =
            self.digitiser_data.iter().map(|i| i.0).collect();
        cache_digitiser_ids.sort();
        cache_digitiser_ids
    }

    pub(super) fn push(&mut self, digitiser_id: DigitizerId, data: D) {
        self.digitiser_data.push((digitiser_id, data));
    }

    pub(super) fn push_veto_flags(&mut self, veto_flags: u16) {
        self.metadata.veto_flags |= veto_flags;
    }

    pub(super) fn is_complete(&self, expected_digitisers: &[DigitizerId]) -> bool {
        self.digitiser_ids() == expected_digitisers
    }

    pub(super) fn is_expired(&self) -> bool {
        Instant::now() > self.expiry
    }
}

impl<D> Spanned for PartialFrame<D> {
    fn span(&self) -> &SpanOnce {
        &self.span
    }
}

impl<D> SpannedMut for PartialFrame<D> {
    fn span_mut(&mut self) -> &mut SpanOnce {
        &mut self.span
    }
}
