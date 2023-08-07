// use itertools::Itertools;

use std::ops::Not;

use itertools::Itertools;

use crate::data::PTS;
use crate::sol::Sol;
use crate::{K_MAX, UNSERVED};

#[derive(Clone, Copy)]
pub struct PairInfo {
    idx: usize,
    pair_idx: usize,
    delivery: bool,
}

impl PairInfo {
    pub fn new(idx: usize, pair_idx: usize, delivery: bool) -> Self {
        Self {
            idx,
            pair_idx,
            delivery,
        }
    }
}

impl Default for PairInfo {
    fn default() -> Self {
        Self {
            idx: UNSERVED,
            pair_idx: UNSERVED,
            delivery: false,
        }
    }
}

pub struct Comb2 {
    pub route: [PairInfo; PTS],
    pub route_idx: [usize; PTS],
    pub len: usize,
    pub iters: [usize; K_MAX + 1],
    pub removed: [usize; K_MAX],
    pub k: usize,
}

impl Comb2 {
    pub fn new() -> Self {
        let mut route = [Default::default(); PTS];
        route[0] = PairInfo::new(0, 0, false);

        Self {
            route,
            route_idx: [UNSERVED; PTS],
            len: 0,
            iters: [UNSERVED; K_MAX + 1],
            removed: [UNSERVED; K_MAX],
            k: 0,
        }
    }

    pub fn from_route(&mut self, sol: &Sol, route_start: usize, k: usize) {
        self.k = k;
        self.fill_indices(route_start, sol);
        self.fill_pair_info(route_start, sol);

        let mut cur = 1;
        for i in 0..k {
            while self.route[cur].delivery {
                cur += 1;
            }

            self.set_iters_and_removed(i, cur);
            cur += 1;
        }

        self.iters[k] = self.len - 1;
        for i in (k + 1)..=K_MAX {
            self.iters[i] = UNSERVED
        }
    }

    fn set_iters_and_removed(&mut self, it: usize, val: usize) {
        self.iters[it] = val;
        self.removed[it] = self.route[val].idx;
    }

    fn fill_pair_info(&mut self, route_start: usize, sol: &Sol<'_>) {
        let mut cur = route_start;
        let mut i = 1;
        while cur != 0 {
            let pt = sol.data.pts[cur];
            self.route[i] = PairInfo::new(cur, self.route_idx[pt.pair], pt.delivery);

            i += 1;
            cur = sol.next[cur];
        }

        self.route[i] = PairInfo {
            idx: 0,
            pair_idx: 0,
            delivery: false,
        };
    }

    fn fill_indices(&mut self, route_start: usize, sol: &Sol<'_>) {
        let mut cur = route_start;
        let mut i = 1;
        while cur != 0 {
            self.route_idx[cur] = i;

            i += 1;
            cur = sol.next[cur];
        }

        self.len = i + 1;
    }

    pub fn in_iter(&self, i: &usize) -> bool {
        for iter in self.iters[0..self.k].iter() {
            if *i == *iter || *i == self.route[*iter].pair_idx {
                return true;
            }
        }

        false
    }

    pub fn r(&self) -> Vec<usize> {
        // not extremely efficient, but not meant to be
        self.route[0..self.len]
            .iter()
            .enumerate()
            .filter_map(|(i, x)| self.in_iter(&i).not().then_some(x.idx))
            .collect_vec()
    }

    pub fn next_comb(&mut self) -> bool {
        for i in (0..self.k).rev() {
            let next = self.next_pickup_idx(self.iters[i]);
            if next != self.iters[i + 1] {
                self.set_iters_and_removed(i, next);

                let mut next2 = self.iters[i];
                for it in i + 1..self.k {
                    next2 = self.next_pickup_idx(next2);
                    self.set_iters_and_removed(it, next2);
                }

                return true;
            }
        }

        false
    }

    pub fn next_pickup_idx(&self, idx: usize) -> usize {
        let mut idx = idx + 1;

        while self.route[idx].delivery {
            idx += 1
        }

        idx
    }

    pub fn removed_idxs(&self) -> &[usize] {
        &self.iters[0..self.k]
    }

    pub fn removed(&self) -> &[usize] {
        &self.removed[0..self.k]
    }
}

#[derive(Clone, Copy)]
pub struct Comb2Iter<'a> {
    comb: &'a Comb2,
    cur: usize,
}

impl<'a> Comb2Iter<'a> {
    pub fn new(comb: &'a Comb2) -> Self {
        Self { comb, cur: 0 }
    }
}

impl<'a> Iterator for Comb2Iter<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        let res = (self.cur < self.comb.len).then_some(self.comb.route[self.cur].idx);
        self.cur += 1;
        while self.cur < self.comb.len && self.comb.in_iter(&self.cur) {
            self.cur += 1;
        }

        res
    }
}

impl<'a> IntoIterator for &'a Comb2 {
    type Item = usize;
    type IntoIter = Comb2Iter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter::new(self)
    }
}

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
    use crate::{
        data::{Data, PointData},
        sol::Sol,
    };

    use super::*;

    struct LeftRemoved(Vec<usize>, Vec<usize>);

    #[test]
    fn second() {
        let points = 13;

        let mut pts = [PointData {
            dem: 0,
            start: 0,
            due: 0,
            pair: 0,
            delivery: false,
        }; PTS];
        for (pickup, delivery) in (1..points).tuples() {
            pts[pickup].pair = delivery;
            pts[pickup].delivery = false;
            pts[delivery].pair = pickup;
            pts[delivery].delivery = true;
        }

        let matrix = vec![0; PTS * PTS];
        let data = Data {
            dist: matrix.clone(),
            pts,
            max_cap: 0,
            time: matrix.clone(),
            points,
        };

        let mut sol = Sol::new(&data);

        sol.add_route(&vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 0]);

        let mut c = Comb2::new();
        c.from_route(&sol, 1, 1);

        assert_eq!(c.k, 1);
        assert_eq!(c.len, 14);
        assert_eq!(c.iters[0], 1);
        assert_eq!(c.iters[1], 13);

        let nexts = vec![
            LeftRemoved(vec![0, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 0], vec![1]),
            LeftRemoved(vec![0, 1, 2, 5, 6, 7, 8, 9, 10, 11, 12, 0], vec![3]),
            LeftRemoved(vec![0, 1, 2, 3, 4, 7, 8, 9, 10, 11, 12, 0], vec![5]),
            LeftRemoved(vec![0, 1, 2, 3, 4, 5, 6, 9, 10, 11, 12, 0], vec![7]),
            LeftRemoved(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 11, 12, 0], vec![9]),
            LeftRemoved(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 0], vec![11]),
        ];

        check2(&mut c, nexts);

        let mut c = Comb2::new();
        c.from_route(&sol, 1, 2);

        assert_eq!(c.k, 2);
        assert_eq!(c.len, 14);
        assert_eq!(c.iters[0], 1);
        assert_eq!(c.iters[1], 3);
        assert_eq!(c.iters[2], 13);

        let nexts = vec![
            LeftRemoved(vec![0, 5, 6, 7, 8, 9, 10, 11, 12, 0], vec![1, 3]),
            LeftRemoved(vec![0, 3, 4, 7, 8, 9, 10, 11, 12, 0], vec![1, 5]),
            LeftRemoved(vec![0, 3, 4, 5, 6, 9, 10, 11, 12, 0], vec![1, 7]),
            LeftRemoved(vec![0, 3, 4, 5, 6, 7, 8, 11, 12, 0], vec![1, 9]),
            LeftRemoved(vec![0, 3, 4, 5, 6, 7, 8, 9, 10, 0], vec![1, 11]),
            LeftRemoved(vec![0, 1, 2, 7, 8, 9, 10, 11, 12, 0], vec![3, 5]),
            LeftRemoved(vec![0, 1, 2, 5, 6, 9, 10, 11, 12, 0], vec![3, 7]),
            LeftRemoved(vec![0, 1, 2, 5, 6, 7, 8, 11, 12, 0], vec![3, 9]),
            LeftRemoved(vec![0, 1, 2, 5, 6, 7, 8, 9, 10, 0], vec![3, 11]),
            LeftRemoved(vec![0, 1, 2, 3, 4, 9, 10, 11, 12, 0], vec![5, 7]),
            LeftRemoved(vec![0, 1, 2, 3, 4, 7, 8, 11, 12, 0], vec![5, 9]),
            LeftRemoved(vec![0, 1, 2, 3, 4, 7, 8, 9, 10, 0], vec![5, 11]),
            LeftRemoved(vec![0, 1, 2, 3, 4, 5, 6, 11, 12, 0], vec![7, 9]),
            LeftRemoved(vec![0, 1, 2, 3, 4, 5, 6, 9, 10, 0], vec![7, 11]),
            LeftRemoved(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 0], vec![9, 11]),
        ];

        check2(&mut c, nexts);

        sol.remove_route(1);
        sol.add_route(&vec![0, 1, 3, 5, 4, 2, 6, 7, 8, 9, 10, 11, 12, 0]);
        c.from_route(&sol, 1, 1);
        assert_eq!(c.k, 1);
        assert_eq!(c.len, 14);
        assert_eq!(c.iters[0], 1);
        assert_eq!(c.iters[1], 13);

        let nexts = vec![
            LeftRemoved(vec![0, 3, 5, 4, 6, 7, 8, 9, 10, 11, 12, 0], vec![1]),
            LeftRemoved(vec![0, 1, 5, 2, 6, 7, 8, 9, 10, 11, 12, 0], vec![3]),
            LeftRemoved(vec![0, 1, 3, 4, 2, 7, 8, 9, 10, 11, 12, 0], vec![5]),
            LeftRemoved(vec![0, 1, 3, 5, 4, 2, 6, 9, 10, 11, 12, 0], vec![7]),
            LeftRemoved(vec![0, 1, 3, 5, 4, 2, 6, 7, 8, 11, 12, 0], vec![9]),
            LeftRemoved(vec![0, 1, 3, 5, 4, 2, 6, 7, 8, 9, 10, 0], vec![11]),
        ];

        check2(&mut c, nexts);

        sol.remove_route(1);
        sol.add_route(&vec![0, 1, 3, 5, 4, 2, 6, 7, 8, 9, 10, 11, 12, 0]);
        c.from_route(&sol, 1, 2);

        let nexts = vec![
            LeftRemoved(vec![0, 5, 6, 7, 8, 9, 10, 11, 12, 0], vec![1, 3]),
            LeftRemoved(vec![0, 3, 4, 7, 8, 9, 10, 11, 12, 0], vec![1, 5]),
            LeftRemoved(vec![0, 3, 5, 4, 6, 9, 10, 11, 12, 0], vec![1, 7]),
            LeftRemoved(vec![0, 3, 5, 4, 6, 7, 8, 11, 12, 0], vec![1, 9]),
            LeftRemoved(vec![0, 3, 5, 4, 6, 7, 8, 9, 10, 0], vec![1, 11]),
            LeftRemoved(vec![0, 1, 2, 7, 8, 9, 10, 11, 12, 0], vec![3, 5]),
            LeftRemoved(vec![0, 1, 5, 2, 6, 9, 10, 11, 12, 0], vec![3, 7]),
            LeftRemoved(vec![0, 1, 5, 2, 6, 7, 8, 11, 12, 0], vec![3, 9]),
            LeftRemoved(vec![0, 1, 5, 2, 6, 7, 8, 9, 10, 0], vec![3, 11]),
            LeftRemoved(vec![0, 1, 3, 4, 2, 9, 10, 11, 12, 0], vec![5, 7]),
            LeftRemoved(vec![0, 1, 3, 4, 2, 7, 8, 11, 12, 0], vec![5, 9]),
            LeftRemoved(vec![0, 1, 3, 4, 2, 7, 8, 9, 10, 0], vec![5, 11]),
            LeftRemoved(vec![0, 1, 3, 5, 4, 2, 6, 11, 12, 0], vec![7, 9]),
            LeftRemoved(vec![0, 1, 3, 5, 4, 2, 6, 9, 10, 0], vec![7, 11]),
            LeftRemoved(vec![0, 1, 3, 5, 4, 2, 6, 7, 8, 0], vec![9, 11]),
        ];
        check2(&mut c, nexts);
    }

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

    fn check2(c: &mut Comb2, nexts: Vec<LeftRemoved>) {
        for LeftRemoved(next, removed) in nexts.iter().skip(1) {
            assert!(c.next_comb());
            assert_eq!(c.r(), *next);
            assert_eq!(c.into_iter().collect_vec(), *next);
            assert_eq!(c.removed(), *removed);
            let removed2 = c
                .removed_idxs()
                .iter()
                .map(|x| c.route[*x].idx)
                .collect_vec();
            assert_eq!(removed2, *removed);
        }
        assert!(!c.next_comb());
    }
}
