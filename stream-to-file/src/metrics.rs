use lazy_static::lazy_static;
use prometheus_client::metrics::{counter::Counter, family::Family};

lazy_static! {
    pub(crate) static ref MESSAGES_RECEIVED: Family::<&'static str, Counter> =
        Family::<&str, Counter>::default();
    pub(crate) static ref FAILURES: Family::<&'static str, Counter> =
        Family::<&str, Counter>::default();
}
