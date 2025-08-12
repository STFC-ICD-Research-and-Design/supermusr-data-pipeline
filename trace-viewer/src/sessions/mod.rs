//! These structs implement the session engine, which processes requests
//! from the [crate::app::server_functions] module.
//!
mod session;
mod session_engine;

pub use session::SessionSearchBody;
pub use session_engine::{SessionEngine, SessionEngineSettings};
