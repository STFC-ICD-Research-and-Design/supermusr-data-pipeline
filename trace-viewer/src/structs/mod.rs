mod broker_info;
mod search;
mod trace_messages;

use crate::{Channel, DigitizerId, Timestamp};
use cfg_if::cfg_if;
use serde::{Deserialize, Serialize};

pub use broker_info::{BrokerInfo, BrokerTopicInfo};
pub use search::{
    SearchBy, SearchMode, SearchResults, SearchStatus, SearchTarget, SearchTargetBy,
    SearchTargetMode,
};
pub use trace_messages::{SelectedTraceIndex, TracePlotly, TraceSummary};

cfg_if! {
    if #[cfg(feature = "ssr")] {
        use clap::Args;
    }
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(Args))]
pub struct Topics {
    /// Kafka trace topic.
    #[cfg_attr(feature = "ssr", clap(long))]
    pub trace_topic: String,

    /// Kafka digitiser event list topic.
    #[cfg_attr(feature = "ssr", clap(long))]
    pub digitiser_event_topic: String,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(Args))]
pub struct Select {
    /// The timestamp of the frame to search for, should be in the format "YYYY-MM-DD hh:mm:ss.f <timezone>".
    #[cfg_attr(feature = "ssr", clap(long))]
    pub(crate) timestamp: Option<Timestamp>,

    /// The digitiser Ids to search for.
    #[cfg_attr(feature = "ssr", clap(long))]
    pub(crate) digitiser_ids: Option<Vec<DigitizerId>>,

    /// The channels to search for.
    #[cfg_attr(feature = "ssr", clap(long))]
    pub(crate) channels: Option<Vec<Channel>>,

    /// The maximum number of messages to collect.
    #[cfg_attr(feature = "ssr", clap(long))]
    pub(crate) number: Option<usize>,
}
