use itertools::Itertools;

use crate::{data::PTS, UNSERVED};

pub struct Heap {
    pub removed_times: [u64; PTS],
    pub removed_idx: [usize; PTS],
    pub size: usize,
}

impl Heap {
    pub fn new() -> Self {
        Self {
            removed_times: [0; PTS],
            removed_idx: [UNSERVED; PTS],
            size: 0,
        }
    }

    pub fn inc(&mut self) {
        self.inc_removed_times_of_top();
        self.slide_top_into_correct_place();

        debug_assert!(self.check_if_is_sorted());
    }

    fn inc_removed_times_of_top(&mut self) {
        self.removed_times[self.removed_idx[self.size - 1]] += 1;
    }

    fn check_if_is_sorted(&mut self) -> bool {
        self.removed_idx[..self.size]
            .iter()
            .map(|&x| self.removed_times[x])
            .tuple_windows()
            .all(|(p, n)| p <= n)
    }

    fn slide_top_into_correct_place(&mut self) {
        let top = self.size - 1;
        let top_removed_times = self.removed_times[self.removed_idx[top]];

        let mut cur = top;

        while cur != 0 && self.removed_times[self.removed_idx[cur - 1]] > top_removed_times {
            cur -= 1;
        }

        self.removed_idx[cur..=top].rotate_right(1);
    }

    pub fn prn(&self) {
        println!(
            "{:?}",
            self.removed_idx[0..self.size]
                .iter()
                .map(|&x| (x, self.removed_times[x]))
                .collect_vec()
        );
    }

    pub fn push(&mut self, idx: usize) {
        self.removed_idx[self.size] = idx;
        self.size += 1;
    }

    pub fn sort(&mut self) {
        self.removed_idx[0..self.size].sort_unstable_by_key(|&x| self.removed_times[x]);
    }

    pub fn top(&self) -> Option<usize> {
        (self.size != 0).then(|| self.removed_idx[self.size - 1])
    }

    pub fn pop(&mut self) {
        self.size -= 1
    }
}
