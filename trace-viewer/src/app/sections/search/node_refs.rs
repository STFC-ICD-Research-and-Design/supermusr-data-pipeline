use chrono::{NaiveDate, NaiveTime};
use leptos::{html::Input, prelude::*};

#[derive(Default, Clone, Copy)]
pub(crate) struct SearchBrokerNodeRefs {
    //let search_mode_ref = NodeRef::<Input>::new();
    //let search_by_ref = NodeRef::<Input>::new();
    pub(crate) date_ref: NodeRef<Input>,
    pub(crate) time_ref: NodeRef<Input>,
    pub(crate) number_ref: NodeRef<Input>,
    pub(crate) channels_ref: NodeRef<Input>,
    pub(crate) digitiser_ids_ref: NodeRef<Input>,
}

impl SearchBrokerNodeRefs {
    pub(crate) fn get_time(&self) -> NaiveTime {
        self.time_ref
            .get()
            .expect("time ref Should exists, this should never fail.")
            .value()
            .parse::<NaiveTime>()
            .expect("time should be NaiveTime, this should never fail.")
    }

    pub(crate) fn get_date(&self) -> NaiveDate {
        self.date_ref
            .get()
            .expect("date ref Should exists, this should never fail.")
            .value()
            .parse::<NaiveDate>()
            .expect("date should be NaiveDate, this should never fail.")
    }

    pub(crate) fn get_channels(&self) -> Vec<u32> {
        self.channels_ref
            .get()
            .expect("channels ref should exist, this should never fail.")
            .value()
            .split(",")
            .map(|x| x.parse())
            .collect::<Result<Vec<_>, _>>()
            .expect("")
    }

    pub(crate) fn get_number(&self) -> usize {
        self.number_ref
            .get()
            .expect("number ref should exist, this should never fail.")
            .value()
            .parse()
            .unwrap_or(1)
    }
}
