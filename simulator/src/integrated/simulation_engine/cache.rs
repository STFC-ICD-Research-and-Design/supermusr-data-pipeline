use super::actions::SelectionModeOptions;
use chrono::Utc;
use rand::{seq::SliceRandom, Rng, SeedableRng};
use std::collections::VecDeque;

pub(crate) trait SimulationEngineCache: Default + Extend<Self::Item> {
    type Item;

    fn extract_one(&mut self, selection_mode: SelectionModeOptions) -> &Self::Item;
    fn extract(&mut self, selection_mode: SelectionModeOptions, amount: usize) -> Vec<&Self::Item>;
    fn finish_one(&mut self, selection_mode: SelectionModeOptions);
    fn finish(&mut self, selection_mode: SelectionModeOptions, amount: usize);
}

impl<T> SimulationEngineCache for VecDeque<T> {
    type Item = T;

    fn extract_one(&mut self, selection_mode: SelectionModeOptions) -> &Self::Item {
        match selection_mode {
            SelectionModeOptions::PopFront => self.front(),
            SelectionModeOptions::ReplaceRandom => {
                let mut rng =
                    rand::rngs::StdRng::seed_from_u64(Utc::now().timestamp_subsec_nanos() as u64);
                self.get(rng.gen_range(0..self.len()))
            }
        }
        .unwrap()
    }

    fn extract(&mut self, selection_mode: SelectionModeOptions, amount: usize) -> Vec<&Self::Item> {
        match selection_mode {
            SelectionModeOptions::PopFront => self.iter().take(amount).collect(),
            SelectionModeOptions::ReplaceRandom => {
                let mut rng =
                    rand::rngs::StdRng::seed_from_u64(Utc::now().timestamp_subsec_nanos() as u64);
                let mut indices = (0..self.len()).collect::<Vec<_>>();
                let (random_indices, _) = indices.partial_shuffle(&mut rng, amount);
                random_indices
                    .iter()
                    .map(|i| self.get(*i).unwrap())
                    .collect()
            }
        }
    }

    fn finish_one(&mut self, selection_mode: SelectionModeOptions) {
        match selection_mode {
            SelectionModeOptions::PopFront => {
                self.pop_front();
            }
            SelectionModeOptions::ReplaceRandom => (),
        };
    }

    fn finish(&mut self, selection_mode: SelectionModeOptions, amount: usize) {
        match selection_mode {
            SelectionModeOptions::PopFront => {
                self.drain(0..amount);
            }
            SelectionModeOptions::ReplaceRandom => (),
        };
    }
}
