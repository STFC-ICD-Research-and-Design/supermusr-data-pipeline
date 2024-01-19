use std::time::{Instant, Duration};

#[derive(Default)]
pub(crate) struct Timer {
    start : Option<Instant>,
    duration : Duration,
    cumulative_duration : Duration,
    ns_per_byte_in : f64,
    cumulative_ns_per_byte_in : f64,
    ns_per_byte_out : f64,
    cumulative_ns_per_byte_out : f64,
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

    pub(crate) fn accumulate_per_byte(&mut self, num_bytes_in : usize, num_bytes_out : usize) -> Option<()> {
        self.ns_per_byte_in = self.duration.as_nanos() as f64 / num_bytes_in as f64;
        self.cumulative_ns_per_byte_in += self.ns_per_byte_in;
        self.ns_per_byte_out = self.duration.as_nanos() as f64 / num_bytes_out as f64;
        self.cumulative_ns_per_byte_out += self.ns_per_byte_out;
        Some(())
    }
    fn print(&self, name : &str, num_messages : u128) {
        println!("{name}: Time: {0}us, Time/Message: {1}us, Time/BytesIn/Message {2:.prec$}ns, Time/BytesOut/Message {3:.prec$}ns",
            self.cumulative_duration.as_micros(),
            self.cumulative_duration.as_micros()/num_messages,
            self.cumulative_ns_per_byte_in/num_messages as f64,
            self.cumulative_ns_per_byte_out/num_messages as f64,
            prec = 3
        );
    }
}

#[derive(Default)]
pub(crate) struct TimerSuite {
    pub(crate) full : Timer,
    pub(crate) iteration : Timer,
    pub(crate) processing : Timer,
    num_messages : u128,
    target_messages : u128,
    num_bytes_in : usize,
    num_bytes_out : usize,
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
        println!("Timing for {0} messages, with {1} total bytes in and {2} total bytes out.", self.num_messages, self.num_bytes_in, self.num_bytes_out);
        self.full.print      ("Total      ", self.num_messages);
        self.iteration.print ("Loop       ", self.num_messages);
        self.processing.print("Processing ", self.num_messages);
    }

    pub(crate) fn next_message(&mut self, num_bytes_in : usize, num_bytes_out : usize) {
        self.num_messages += 1;
        self.iteration.accumulate();
        self.processing.accumulate();
        self.iteration.accumulate_per_byte(num_bytes_in,num_bytes_out);
        self.processing.accumulate_per_byte(num_bytes_in,num_bytes_out);
        self.num_bytes_in += num_bytes_in;
        self.num_bytes_out += num_bytes_out;
    }
    pub(crate) fn append_results(&self) {

    }
}