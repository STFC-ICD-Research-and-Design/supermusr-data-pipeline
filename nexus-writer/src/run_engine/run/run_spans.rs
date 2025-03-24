use supermusr_common::{record_metadata_fields_to_span, spanned::SpannedAggregator};
use supermusr_streaming_types::{
    aev2_frame_assembled_event_v2_generated::FrameAssembledEventListMessage, FrameMetadata,
};
use tracing::{info_span, warn, Span};

use crate::nexus::NexusFileInterface;

use super::Run;

pub(crate) trait RunSpan: SpannedAggregator {
    fn link_run_start_span(&mut self);
    fn link_frame_event_list_span(&mut self, frame_event_list: FrameAssembledEventListMessage);
    fn link_run_log_span(&mut self);
    fn link_sample_environment_log_span(&mut self);
    fn link_alarm_span(&mut self);
    fn link_run_stop_span(&mut self);
}

impl<I: NexusFileInterface> Run<I> {
    fn link_span(&mut self, f: impl Fn() -> Span) {
        if let Err(e) = self.span_init() {
            warn!("Run span initiation failed {e}")
        }
        if let Err(e) = self.link_current_span(f) {
            warn!("Run span linking failed {e}")
        }
    }
}

impl<I: NexusFileInterface> RunSpan for Run<I> {
    fn link_run_start_span(&mut self) {
        let collect_from = self.parameters().collect_from.to_rfc3339();
        self.link_span(move || info_span!("Run Start Command", "Start" = collect_from));
    }
    fn link_frame_event_list_span(&mut self, frame_event_list: FrameAssembledEventListMessage) {
        let completed = frame_event_list.complete();
        let metadata: Result<FrameMetadata, _> = frame_event_list.metadata().try_into();
        self.link_span(move || {
            let span = info_span!(
                "Frame Event List",
                "metadata_timestamp" = tracing::field::Empty,
                "metadata_frame_number" = tracing::field::Empty,
                "metadata_period_number" = tracing::field::Empty,
                "metadata_veto_flags" = tracing::field::Empty,
                "metadata_protons_per_pulse" = tracing::field::Empty,
                "metadata_running" = tracing::field::Empty,
                "frame_is_complete" = completed,
            );
            if let Ok(metadata) = &metadata {
                record_metadata_fields_to_span!(metadata, span);
            }
            span
        });
    }

    fn link_run_log_span(&mut self) {
        self.link_span(|| info_span!("Run Log Data"));
    }

    fn link_sample_environment_log_span(&mut self) {
        self.link_span(|| info_span!("Sample Environment Log"));
    }

    fn link_alarm_span(&mut self) {
        self.link_span(|| info_span!("Alarm"));
    }

    fn link_run_stop_span(&mut self) {
        let collect_until = self
            .parameters()
            .run_stop_parameters
            .as_ref()
            .map(|s| s.collect_until.to_rfc3339())
            .unwrap_or_default();
        self.link_span(|| info_span!("Run Stop Command", "Stop" = collect_until));
    }
}
