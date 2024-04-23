use crate::nexus::{NexusSettings, Run, RunLike, RunParameters};
use anyhow::Result;
use supermusr_common::tracer::Spanned;
use tracing::trace_span;

pub(crate) type SpannedRun = Spanned<Run>;

impl RunLike for SpannedRun {
    fn new(filename: Option<&std::path::Path>, parameters: RunParameters, settings: &NexusSettings) -> Result<Self> {
        let span = trace_span!("Run");
        Ok(Spanned::new(span, Run::new(filename, parameters, settings)?))
    }
}
