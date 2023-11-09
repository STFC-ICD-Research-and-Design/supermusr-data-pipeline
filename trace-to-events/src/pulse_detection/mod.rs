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
pub(crate) mod events;
pub(crate) mod trace_iterators;
pub(crate) mod window;

pub(crate) use datatype::{EventData, RealArray, Temporal, TracePoint};
pub(crate) use detectors::{basic_muon_detector, threshold_detector, Assembler, Detector};
pub(crate) use events::{EventFilter, EventPoint};
#[cfg(test)]
pub(crate) use window::WindowFilter;

pub(crate) use pulse::Pulse;

pub(crate) type Real = f64;

#[cfg(test)]
mod tests {
    //use crate::window::composite::CompositeWindow;
    use common::Intensity;

    use super::*;

    #[test]
    fn sample_data() {
        let input = vec![
            1.0, 1.0, 1.1, 1.0, 0.9, 1.0, 1.0, 1.1, 1.0, 0.9, 1.0, 1.1, 1.0, 1.0, 0.9, 1.0, 1.0,
            1.1, 1.0, 1.0, 1.0, 1.0, 1.1, 0.9, 1.0, 1.1, 1.0, 1.0, 0.9, 1.0, 1.1, 1.0, 1.0, 1.1,
            1.0, 0.8, 0.9, 1.0, 1.2, 0.9, 1.0, 1.0, 1.1, 1.2, 1.0, 1.5, 1.0, 3.0, 2.0, 5.0, 3.0,
            2.0, 1.0, 1.0, 1.0, 0.9, 1.0, 1.0, 3.0, 2.6, 4.0, 3.0, 3.2, 2.0, 1.0, 1.0, 0.8, 4.0,
            4.0, 2.0, 2.5, 1.0, 1.0, 1.0,
        ];
        let output = input
            .iter()
            .map(|x| (x * 1000.) as Intensity)
            .into_iter()
            .enumerate()
            .map(|(i, v)| (i as Real, v as Real));
        //.finite_differences()
        //.window(CompositeWindow::<1,Real>::trivial())
        //.events(EventsDetector::new())
        //.collect();
        for line in output {
            println!("{line:?}")
        }
    }
}
