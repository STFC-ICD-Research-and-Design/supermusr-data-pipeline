//! Defines structs which

mod broker_info;
mod digitiser_messages;
mod search;
mod trace_messages;

use crate::{Channel, DigitizerId, Timestamp};
use cfg_if::cfg_if;
use serde::{Deserialize, Serialize};

pub use broker_info::{BrokerInfo, BrokerTopicInfo};
pub use digitiser_messages::TraceWithEvents;
pub use search::{
    SearchBy, SearchMode, SearchStatus, SearchTarget, SearchTargetBy, SearchTargetMode,
};
pub use trace_messages::{SelectedTraceIndex, TracePlotly, TraceSummary};

/// Contains the names of the Kafka topics as set in the command line.
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

/// Contains the settings defined in the CLI used as default values in the UI's inputs.
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(Args))]
pub struct DefaultData {
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

    /// Default Kafka timeout for polling the broker for topic info.
    /// If this feature is failing, then increasing this value may help.
    #[cfg_attr(feature = "ssr", clap(long, default_value = "1000"))]
    pub(crate) poll_broker_timeout_ms: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ClientSideData {
    pub default_data: DefaultData,
    pub broker_name: String,
    pub link_to_redpanda_console: Option<String>,
}

cfg_if! {
    if #[cfg(feature = "ssr")] {
        mod server_only;

        // This should be imported only for server-side use.
        use clap::Args;
        pub(crate) use server_only::{Cache, BorrowedMessageError , SearchResults, EventListMessage, FBMessage, TraceMessage};

        ///
        #[derive(Default, Clone, Debug, Serialize, Deserialize)]
        pub struct ServerSideData {
            pub broker: String,
            pub topics: Topics,
            pub username: Option<String>,
            pub password: Option<String>,
            pub consumer_group: String,
        }
    }
}
