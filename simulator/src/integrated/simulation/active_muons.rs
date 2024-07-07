use std::{collections::VecDeque, slice::Iter};

use supermusr_common::Time;

use crate::integrated::simulation_elements::muon::MuonEvent;

pub(super) struct ActiveMuons<'a> {
    active: VecDeque<&'a MuonEvent>,
    muon_iter: Iter<'a, MuonEvent>,
}

impl<'a> ActiveMuons<'a> {
    pub(super) fn new(source: &'a [MuonEvent]) -> Self {
        Self {
            active: Default::default(),
            muon_iter: source.iter(),
        }
    }
    pub(super) fn drop_spent_muons(&mut self, time: Time) {
        while self
            .active
            .front()
            .and_then(|m| (m.get_end() < time).then_some(m))
            .is_some()
        {
            self.active.pop_front();
        }
    }
    pub(super) fn push_new_muons(&mut self, time: Time) {
        while let Some(iter) = self
            .muon_iter
            .next()
            .and_then(|iter| (iter.get_start() > time).then_some(iter))
        {
            self.active.push_back(iter)
        }
    }
    pub(super) fn iter(&self) -> std::collections::vec_deque::Iter<'_, &MuonEvent> {
        self.active.iter()
    }
}
