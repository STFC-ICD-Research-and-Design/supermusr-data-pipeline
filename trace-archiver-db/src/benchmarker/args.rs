use std::{ops::RangeInclusive, iter::StepBy, str::FromStr};
use itertools::{iproduct, ConsTuples, Product};

use crate::utils::{log_then_panic, log_then_panic_t};

///  A range object that includes an inclusive range object and a step size.
#[derive(Clone)]
pub struct SteppedRange (pub RangeInclusive<usize>, pub usize);

impl SteppedRange {
    pub fn from_string(src : String) -> Result<Self,anyhow::Error> {
        let params : Vec<usize> = src.split(':').map(|s|s.parse().unwrap_or_else(|e|log_then_panic_t(format!("{src}: {e}")))).collect();
        if params.len() != 3 {
            log_then_panic(format!("SteppedRange: Wrong number of parameters in {src}: {params:?}"))
        }
        Ok(SteppedRange(params[0]..=params[1],params[2]))
    }
    pub fn iter(&self) -> StepBy<RangeInclusive<usize>> {
        self.0.clone().into_iter().step_by(self.1)
    }
}

impl FromStr for SteppedRange {
    type Err = anyhow::Error;

    fn from_str(src: &str) -> Result<Self, Self::Err> {
        let params : Vec<usize> = src.split(':').map(|s|s.parse().unwrap_or_else(|e|log_then_panic_t(format!("{src}: {e}")))).collect();
        if params.len() != 3 {
            log_then_panic(format!("SteppedRange: Wrong number of parameters in {src}: {params:?}"))
        }
        Ok(SteppedRange(params[0]..=params[1],params[2]))
    }
}

#[derive(Default,PartialEq)]
pub struct Args {
    pub num_messages: usize,
    pub num_channels: usize,
    pub num_samples: usize,
}
impl Args {
    pub(super) fn new(m: usize, c: usize, s: usize) -> Args { Args {num_messages: m, num_channels: c, num_samples: s} }

    pub(super) fn extract_param(&self, args : &SeriesArgs) -> Result<usize,anyhow::Error> { 
        match args {
            SeriesArgs::NumMessagesVariable{num_messages: _, num_channels: _, num_samples: _} => Ok(self.num_messages),
            SeriesArgs::NumChannelsVariable{num_messages: _, num_channels: _, num_samples: _} => Ok(self.num_channels),
            SeriesArgs::NumSamplesVariable {num_messages: _, num_channels: _, num_samples: _} => Ok(self.num_samples),
        }
    }

    pub(super) fn is_matched(&self, args : &SeriesArgs) -> bool {
        match args {
            SeriesArgs::NumMessagesVariable{num_messages: m, num_channels: c, num_samples: s}
                => m.contains(&self.num_messages) && self.num_channels == *c && self.num_samples == *s,
                SeriesArgs::NumChannelsVariable{num_messages: m, num_channels: c, num_samples: s}
                => self.num_messages == *m && c.contains(&self.num_channels) && self.num_samples == *s,
                SeriesArgs::NumSamplesVariable{num_messages: m, num_channels: c, num_samples: s}
                => self.num_messages == *m && self.num_channels == *c && s.contains(&self.num_samples),
        }
    }

    pub(super) fn output_init(&self) -> String { format!("Running benchmark for {0} messages, {1} channels, {2} datapoints.", self.num_messages, self.num_channels, self.num_samples) }
}

impl std::fmt::Display for Args {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

/// Used to construct a series from benchmark results.
/// #Variants
/// *NumMessagesVariable - a structure which fixes the number of channels and samples, and sets upper and lower bounds on the number of messages.
/// *NumChannelsVariable - a structure which fixes the number of messages and samples, and sets upper and lower bounds on the number of channels.
/// *NumSamplesVariable - a structure which fixes the number of messages and channels, and sets upper and lower bounds on the number of samples.
#[derive(PartialEq)]
pub enum SeriesArgs {
    NumMessagesVariable{num_messages: RangeInclusive<usize>, num_channels: usize, num_samples: usize},
    NumChannelsVariable{num_messages: usize, num_channels: RangeInclusive<usize>, num_samples: usize},
    NumSamplesVariable {num_messages: usize, num_channels: usize, num_samples: RangeInclusive<usize>},
}


#[derive(Clone)]
pub(crate) struct ArgRanges {
    pub(crate) num_messages_range: SteppedRange,
    pub(crate) num_channels_range: SteppedRange,
    pub(crate) num_samples_range: SteppedRange,
}

type ParameterSpace = ConsTuples<Product<
                                    Product<
                                        StepBy<RangeInclusive<usize>>,
                                        StepBy<RangeInclusive<usize>>
                                    >,
                                    StepBy<RangeInclusive<usize>>
                                >,
                            ((usize, usize), usize)>;

impl ArgRanges {
    pub(crate) fn get_parameter_space(&self) -> ParameterSpace {
        iproduct!(self.num_messages_range.iter(),self.num_channels_range.iter(),self.num_samples_range.iter())
    }
    pub(crate) fn get_parameter_space_size(&self) -> usize {
        self.get_parameter_space().collect::<Vec<_>>().len()
    }
}
