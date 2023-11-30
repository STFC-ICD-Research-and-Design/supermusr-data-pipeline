//! This crate provides tools for converting raw trace data into
//! a stream of events which represent pulses in the trace stream.
//!
//! A raw trace takes the form of a Vec (or some other similar container)
//! of scalar values. Typical usage of this crate may look like:
//! ```rust
//! let events = trace.iter()
//!     .enumerate()
//!     .map(make_real_enumerate)                       // converts to (Real,Real) format
//!     .window(SmoothedWindow(4))                      // A moving average window of length 4
//!     .events(PulseDetector(ChangeDetector(0.5),1))   // Registers an event when the averaged
//!                                                     // signal changes by 0.5*sigma, where sigma is
//!                                                     // the standard deviation from the moving
//!                                                     //average window
//! ```

pub(crate) mod datatype;
pub(crate) mod pulse;

pub(crate) mod detectors;
pub(crate) mod iterators;
pub(crate) mod window;

pub(crate) use datatype::{EventData, EventPoint, RealArray, Stats, Temporal, TracePoint};
pub(crate) use detectors::{basic_muon_detector, threshold_detector, Assembler, Detector};
pub(crate) use iterators::{AssembleFilter, EventFilter, SaveToFileFilter};
#[cfg(test)]
pub(crate) use window::WindowFilter;

pub(crate) use pulse::Pulse;

pub(crate) type Real = f64;
