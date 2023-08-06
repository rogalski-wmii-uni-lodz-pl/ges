// use itertools::Itertools;

use std::ops::Not;

use itertools::Itertools;

use crate::data::PTS;
use crate::{K_MAX, UNSERVED};

pub struct Comb {
    pub pickups: [usize; PTS],
    pub len: usize,
    pub iters: [usize; K_MAX + 1],
    pub k: usize,
}

impl Comb {
    pub fn new() -> Self {
        Self {
            pickups: [UNSERVED; PTS],
            len: 0,
            iters: [UNSERVED; K_MAX + 1],
            k: 0,
        }
    }

    pub fn reset(&mut self, v: &Vec<usize>, k: usize) {
        self.k = k;
        self.len = v.len();
        self.pickups[..v.len()].copy_from_slice(v);
        for i in 0..k {
            self.iters[i] = i + 1
        }
        self.iters[k] = self.len - 1;
        for i in (k + 1)..=K_MAX {
            self.iters[i] = UNSERVED
        }
    }

    pub fn r(&self) -> Vec<usize> {
        // not extremely efficient, but not meant to be
        self.pickups[0..self.len]
            .iter()
            .enumerate()
            .filter_map(|(i, x)| self.in_iter(&i).not().then_some(*x))
            .collect_vec()
    }

    pub fn in_iter(&self, i: &usize) -> bool {
        self.iters[0..self.k].contains(&i)
    }

    pub fn next_comb(&mut self) -> bool {
        for i in (0..self.k).rev() {
            if self.iters[i] + 1 != self.iters[i + 1] {
                self.iters[i] += 1;

                for (it, val) in (i + 1..self.k).zip((self.iters[i] + 1)..) {
                    self.iters[it] = val;
                }

                return true;
            }
        }

        false
    }
}

pub struct CombIter<'a> {
    comb: &'a Comb,
    cur: usize,
}

impl<'a> CombIter<'a> {
    pub fn new(comb: &'a Comb) -> Self {
        Self { comb, cur: 0 }
    }
}

impl<'a> Iterator for CombIter<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        let res = (self.cur < self.comb.len).then_some(self.comb.pickups[self.cur]);
        self.cur += 1;
        println!("{}", self.cur);
        while self.cur < self.comb.len && self.comb.in_iter(&self.cur) {
            self.cur += 1;
        }

        res
    }
}

impl<'a> IntoIterator for &'a Comb {
    type Item = usize;
    type IntoIter = CombIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter::new(self)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn simple() {
        let mut c = Comb::new();

        assert_eq!(c.len, 0);

        c.reset(&vec![0, 1, 2, 3, 4, 0], 1);
        let nexts = vec![
            vec![0, 2, 3, 4, 0],
            vec![0, 1, 3, 4, 0],
            vec![0, 1, 2, 4, 0],
            vec![0, 1, 2, 3, 0],
        ];

        check(&mut c, nexts);

        c.reset(&vec![0, 1, 2, 3, 4, 0], 2);
        assert_eq!(c.len, 6);

        let nexts = vec![
            vec![0, 3, 4, 0],
            vec![0, 2, 4, 0],
            vec![0, 2, 3, 0],
            vec![0, 1, 4, 0],
            vec![0, 1, 3, 0],
            vec![0, 1, 2, 0],
        ];
        check(&mut c, nexts);

        c.reset(&vec![0, 1, 2, 3, 4, 5, 6, 0], 3);
        assert_eq!(c.len, 8);

        let nexts = vec![
            vec![0, 4, 5, 6, 0],
            vec![0, 3, 5, 6, 0],
            vec![0, 3, 4, 6, 0],
            vec![0, 3, 4, 5, 0],
            vec![0, 2, 5, 6, 0],
            vec![0, 2, 4, 6, 0],
            vec![0, 2, 4, 5, 0],
            vec![0, 2, 3, 6, 0],
            vec![0, 2, 3, 5, 0],
            vec![0, 2, 3, 4, 0],
            vec![0, 1, 5, 6, 0],
            vec![0, 1, 4, 6, 0],
            vec![0, 1, 4, 5, 0],
            vec![0, 1, 3, 6, 0],
            vec![0, 1, 3, 5, 0],
            vec![0, 1, 3, 4, 0],
            vec![0, 1, 2, 6, 0],
            vec![0, 1, 2, 5, 0],
            vec![0, 1, 2, 4, 0],
            vec![0, 1, 2, 3, 0],
        ];

        check(&mut c, nexts);

    }

    fn check(c: &mut Comb, nexts: Vec<Vec<usize>>) {
        for next in nexts.iter().skip(1) {
            assert!(c.next_comb());
            assert_eq!(c.r(), *next);
            assert_eq!(c.into_iter().collect_vec(), *next);
        }
        assert!(!c.next_comb());
    }
}
