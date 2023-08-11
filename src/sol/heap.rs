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
        let top = self.size - 1;
        self.removed_times[self.removed_idx[top]] += 1;
        let mut h = top;
        let v = self.removed_times[self.removed_idx[top]];
        let i = self.removed_idx[top];
        while h != 0 {
            if self.removed_times[self.removed_idx[h - 1]] > v {
                self.removed_idx[h] = self.removed_idx[h - 1];
                h -= 1;
            } else {
                break;
            }
        }
        self.removed_idx[h] = i;
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
