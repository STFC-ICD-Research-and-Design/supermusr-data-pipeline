use anyhow::Result;
use std::{ops::Div, time::{Instant, Duration}};

#[derive(Default)]
pub(crate) struct Timer {
    start : Option<Instant>,
    duration : Duration,
    cumulative_duration : Duration,
    duration_per_byte : Duration,
    cumulative_duration_per_byte : Duration,
}
impl Timer {
    pub(crate) fn record(&mut self) -> Option<()> {
        if self.start.is_none() {
            self.start = Some(Instant::now());
            Some(())
        } else {
            None
        }
    }
    pub(crate) fn end(&mut self) -> Option<()> {
        self.duration = Instant::now().checked_duration_since(self.start?)?;
        self.start = None;
        Some(())
    }

    pub(crate) fn accumulate(&mut self) -> Option<()> {
        self.cumulative_duration = self.cumulative_duration.checked_add(self.duration)?;
        Some(())
    }

    pub(crate) fn accumulate_per_byte(&mut self, num_bytes : usize) -> Option<()> {
        self.duration_per_byte = self.duration.div(num_bytes as u32);
        self.cumulative_duration_per_byte = self.cumulative_duration_per_byte.checked_add(self.duration_per_byte)?;
        Some(())
    }
}

#[derive(Default)]
pub(crate) struct TimerSuite {
    pub(crate) full : Timer,
    pub(crate) iteration : Timer,
    pub(crate) processing : Timer,
    num_messages : u128,
    target_messages : u128,
    num_bytes : usize,
}


impl TimerSuite {
    pub(crate) fn new(target_messages : u128) -> Self {
        Self {
            target_messages,
            ..Default::default()
        }
    }

    pub(crate) fn has_finished(&self) -> bool {
        self.num_messages == self.target_messages
    }
    pub(crate) fn print(&self) {
        println!("Timing for {0} messages.", self.num_messages);
        println!("Total time: {0}us, Total Time/Message: {1}us", self.full.cumulative_duration.as_micros(), self.full.cumulative_duration.as_micros()/self.num_messages);
        println!("Loop time: {0}us, Loop Time/Message: {1}us", self.iteration.cumulative_duration.as_micros(), self.iteration.cumulative_duration.as_micros()/self.num_messages);
        println!("Processing time: {0}us, Processing Time/Message: {1}us", self.processing.cumulative_duration.as_micros(), self.processing.cumulative_duration.as_micros()/self.num_messages);
    }

    pub(crate) fn next_message(&mut self, num_bytes : usize) {
        self.num_messages += 1;
        self.iteration.accumulate();
        self.processing.accumulate();
        self.iteration.accumulate_per_byte(num_bytes);
        self.processing.accumulate_per_byte(num_bytes);
        self.num_bytes += num_bytes;
    }
}