use std::ops::Not;

use itertools::Itertools;

use crate::data::PTS;
use crate::sol::Sol;
use crate::{K_MAX, UNSERVED};

#[derive(Clone, Copy, Debug)]
pub struct PairInfo {
    id: usize,
    idx: usize,
    pair_idx: usize,
    removed_times: u64,
    delivery: bool,
}

impl PairInfo {
    pub fn new(id: usize, idx: usize, pair_idx: usize, removed_times: u64, delivery: bool) -> Self {
        Self {
            id,
            idx,
            pair_idx,
            removed_times,
            delivery,
        }
    }

    pub fn idx(&self) -> usize {
        self.id
    }
}

impl Default for PairInfo {
    fn default() -> Self {
        Self {
            id: UNSERVED,
            idx: UNSERVED,
            pair_idx: UNSERVED,
            removed_times: 0,
            delivery: false,
        }
    }
}

pub struct Combinations {
    pub k: usize,
    pub route_len: usize,
    pub cur_removed_times_total: u64,
    pub route: [PairInfo; PTS],
    pub route_position: [usize; PTS],
    // pub combination_indices: [usize; K_MAX + 1],
    // pub removed: [usize; K_MAX],
    pub pickups: [PairInfo; PTS / 2],
    pub combination_indices2: [usize; K_MAX + 1],
    pub sum_of_next: [[u64; K_MAX + 1]; PTS],
}

impl Combinations {
    pub fn new() -> Self {
        let mut route = [Default::default(); PTS];
        route[0] = PairInfo::new(0, 0, 0, 0, false);

        Self {
            k: 0,
            route_len: 0,
            cur_removed_times_total: 0,
            route,
            route_position: [UNSERVED; PTS],
            // combination_indices: [UNSERVED; K_MAX + 1],
            // removed: [UNSERVED; K_MAX],
            pickups: [Default::default(); PTS / 2],
            combination_indices2: [UNSERVED; K_MAX + 1],
            sum_of_next: [[u64::max_value(); K_MAX + 1]; PTS],
        }
    }

    pub fn k_combinations_of_route(&mut self, sol: &Sol, route_start: usize, k: usize) {
        self.k = k;
        self.fill_route_indices_and_set_len(route_start, sol);
        self.copy_pair_info(route_start, sol);
        self.initialize_combination_indices();
    }

    fn fill_route_indices_and_set_len(&mut self, route_start: usize, sol: &Sol) {
        let mut cur = route_start;
        let mut i = 1;
        while cur != 0 {
            self.route_position[cur] = i;

            i += 1;
            cur = sol.next[cur];
        }

        self.route_len = i + 1;
    }

    fn copy_pair_info(&mut self, route_start: usize, sol: &Sol) {
        let mut cur = route_start;
        let mut i = 1;
        let mut pickup = 0;
        while cur != 0 {
            let pt = sol.data.pts[cur];
            let pi = PairInfo::new(
                cur,
                self.route_position[cur],
                self.route_position[pt.pair],
                sol.removed_times(cur),
                pt.is_delivery,
            );

            if !pt.is_delivery {
                self.pickups[pickup] = pi.clone();
                pickup += 1;
            }

            self.route[i] = pi;

            i += 1;
            cur = sol.next[cur];
        }

        self.route[i] = PairInfo {
            id: 0,
            pair_idx: 0,
            ..Default::default()
        };

        self.pickups[0..pickup].sort_unstable_by_key(|x| x.removed_times);

        for i in 0..pickup {
            for k in 0..=(self.k).min(pickup - i) {
                // perhaps start at 2?
                self.sum_of_next[i][k] = self.pickups[(i + 1)..(i + 1 + k)]
                    .iter()
                    .map(|x| x.removed_times)
                    .sum();
            }
        }
    }

    fn initialize_combination_indices(&mut self) {
        self.set_initial_position_of_indices();
        self.set_k_plus_1_guard_value();
        self.fill_rest_of_combination_indices();
        self.initialize_removed_times_sum();
    }

    fn set_initial_position_of_indices(&mut self) {
        // let mut cur = 1;
        // for idx in 0..self.k {
        //     while self.route[cur].delivery {
        //         cur += 1;
        //     }

        //     // self.set_index_and_removed(idx, cur);
        //     cur += 1;
        // }

        for i in 0..self.k {
            self.combination_indices2[i] = i;
        }
    }

    // fn set_index_and_removed(&mut self, idx: usize, next_pickup_idx: usize) {
    //     // self.combination_indices[idx] = next_pickup_idx;
    //     // self.removed[idx] = self.route[next_pickup_idx].idx;
    // }

    fn set_k_plus_1_guard_value(&mut self) {
        // self.combination_indices[self.k] = self.route_len - 1;
        self.combination_indices2[self.k] = (self.route_len - 2) / 2;
    }

    fn fill_rest_of_combination_indices(&mut self) {
        // self.combination_indices[self.k + 1..=K_MAX].fill(UNSERVED);
        self.combination_indices2[self.k + 1..=K_MAX].fill(UNSERVED);
    }

    pub fn is_removed(&self, x: usize) -> bool {
        for i in 0..self.k {
            let pickup = self.pickups[self.combination_indices2[i]];
            let delivery = self.route[pickup.pair_idx].id;

            if x == pickup.id || x == delivery {
                return true;
            }
        }

        return false;
        // self.combination_indices2[0..self.k].contains(i)
        //     || self.combination_indices2[0..self.k]
        //         .iter()
        //         .any(|iter| *i == self.route[*iter].pair_idx)
    }

    fn initialize_removed_times_sum(&mut self) {
        self.cur_removed_times_total = self.combination_indices2[0..self.k]
            .iter()
            .map(|&x| self.pickups[x].removed_times)
            .sum();
    }

    pub fn r(&self) -> Vec<usize> {
        // not extremely efficient, but not meant to be
        self.route[1..self.route_len]
            .iter()
            .filter_map(|x| self.is_removed(x.id).not().then_some(x.id))
            .collect_vec()
    }

    pub fn next_combination_with_lower_score(&mut self, target_score: u64) -> bool {
        let mut sub = 0;
        for i in (0..self.k).rev() {
            sub += self.pickups[self.combination_indices2[i]].removed_times;
            if self.combination_indices2[i] + 1 != self.combination_indices2[i + 1] {
                let new_sum = self.cur_removed_times_total
                    - sub
                    + self.sum_of_next[self.combination_indices2[i]][self.k - i];

                if new_sum < target_score {
                    let ith = self.combination_indices2[i];
                    for j in i..self.k {
                        self.combination_indices2[j] = ith + (j - i) + 1;
                    }
                    self.cur_removed_times_total = new_sum;
                    return true;
                } else {
                    return false;
                }
            }
        }

        false
    }

    // fn next_combination(&mut self) -> bool {
    //     for cur in (0..self.k).rev() {
    //         // let next_pickup = self.next_pickup_after(self.combination_indices[cur]);
    //         // let next_pickup_is_not_removed = next_pickup != self.combination_indices[cur + 1];

    //         // if next_pickup_is_not_removed {
    //         //     self.change_index_and_removed(cur, next_pickup);
    //         //     self.set_indices_larger_than_idx(cur);

    //         //     return true;
    //         // }
    //     }

    //     false
    // }

    // fn set_indices_larger_than_idx(&mut self, idx: usize) {
    //     let mut cur = self.combination_indices[idx];
    //     for it in idx + 1..self.k {
    //         cur = self.next_pickup_after(cur);
    //         self.change_index_and_removed(it, cur);
    //     }
    // }

    // fn change_index_and_removed(&mut self, idx: usize, next_pickup_idx: usize) {
    //     self.cur_removed_times_total -= self.route[self.combination_indices[idx]].removed_times;
    //     self.cur_removed_times_total += self.route[next_pickup_idx].removed_times;

    //     self.set_index_and_removed(idx, next_pickup_idx);
    // }

    pub fn next_pickup_after(&self, idx: usize) -> usize {
        let mut idx = idx + 1;

        while self.route[idx].delivery {
            idx += 1
        }

        idx
    }

    pub fn removed_idxs(&self) -> &[usize] {
        &self.combination_indices2[0..self.k]
    }

    pub fn pickups(&self) -> &[PairInfo] {
        &self.pickups[0..PTS / 2]
    }
}

#[derive(Clone, Copy)]
pub struct CombinationIterator<'a> {
    comb: &'a Combinations,
    removed: [usize; 2 * K_MAX],
    removed_idx: usize,
    cur: usize,
}

impl<'a> CombinationIterator<'a> {
    pub fn new(comb: &'a Combinations) -> Self {
        let mut removed = [0; 2 * K_MAX];
        for (i, &r) in comb.removed_idxs().iter().enumerate() {
            let pickup = &comb.pickups()[r];
            removed[2 * i] = pickup.idx;
            removed[2 * i + 1] = pickup.pair_idx;
        }

        removed[0..(2 * comb.k)].sort();

        let mut s = Self {
            comb,
            removed,
            removed_idx: 0,
            cur: 1,
        };

        s.adv();

        s
    }

    fn adv(&mut self) {
        while self.cur < self.comb.route_len - 1
            && self.removed_idx < 2 * K_MAX
            && self.removed[self.removed_idx] == self.cur
        {
            self.cur += 1;
            self.removed_idx += 1;
        }
    }
}

impl<'a> Iterator for CombinationIterator<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        let res = (self.cur < self.comb.route_len).then_some(self.comb.route[self.cur].id);

        self.cur += 1;
        self.adv();

        res
    }
}

impl<'a> IntoIterator for &'a Combinations {
    type Item = usize;
    type IntoIter = CombinationIterator<'a>;

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
            is_delivery: false,
        }; PTS];
        for (pickup, delivery) in (1..points).tuples() {
            pts[pickup].pair = delivery;
            pts[pickup].is_delivery = false;
            pts[delivery].pair = pickup;
            pts[delivery].is_delivery = true;
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

        let mut c = Combinations::new();
        c.k_combinations_of_route(&sol, 1, 1);

        assert_eq!(c.k, 1);
        assert_eq!(c.route_len, 14);
        assert_eq!(c.combination_indices2[0], 0);
        assert_eq!(c.combination_indices2[1], 6);
        assert_eq!(c.sum_of_next[0][0], 0);
        assert_eq!(c.sum_of_next[0][1], 0);

        let nexts = vec![
            LeftRemoved(vec![3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 0], vec![0]),
            LeftRemoved(vec![1, 2, 5, 6, 7, 8, 9, 10, 11, 12, 0], vec![1]),
            LeftRemoved(vec![1, 2, 3, 4, 7, 8, 9, 10, 11, 12, 0], vec![2]),
            LeftRemoved(vec![1, 2, 3, 4, 5, 6, 9, 10, 11, 12, 0], vec![3]),
            LeftRemoved(vec![1, 2, 3, 4, 5, 6, 7, 8, 11, 12, 0], vec![4]),
            LeftRemoved(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 0], vec![5]),
        ];

        check2(&mut c, nexts);

        dbg!("k = 2");

        let mut c = Combinations::new();
        c.k_combinations_of_route(&sol, 1, 2);

        assert_eq!(c.k, 2);
        assert_eq!(c.route_len, 14);
        assert_eq!(c.combination_indices2[0], 0);
        assert_eq!(c.combination_indices2[1], 1);
        assert_eq!(c.combination_indices2[2], 6);

        let nexts = vec![
            LeftRemoved(vec![5, 6, 7, 8, 9, 10, 11, 12, 0], vec![0, 1]),
            LeftRemoved(vec![3, 4, 7, 8, 9, 10, 11, 12, 0], vec![0, 2]),
            LeftRemoved(vec![3, 4, 5, 6, 9, 10, 11, 12, 0], vec![0, 3]),
            LeftRemoved(vec![3, 4, 5, 6, 7, 8, 11, 12, 0], vec![0, 4]),
            LeftRemoved(vec![3, 4, 5, 6, 7, 8, 9, 10, 0], vec![0, 5]),
            LeftRemoved(vec![1, 2, 7, 8, 9, 10, 11, 12, 0], vec![1, 2]),
            LeftRemoved(vec![1, 2, 5, 6, 9, 10, 11, 12, 0], vec![1, 3]),
            LeftRemoved(vec![1, 2, 5, 6, 7, 8, 11, 12, 0], vec![1, 4]),
            LeftRemoved(vec![1, 2, 5, 6, 7, 8, 9, 10, 0], vec![1, 5]),
            LeftRemoved(vec![1, 2, 3, 4, 9, 10, 11, 12, 0], vec![2, 3]),
            LeftRemoved(vec![1, 2, 3, 4, 7, 8, 11, 12, 0], vec![2, 4]),
            LeftRemoved(vec![1, 2, 3, 4, 7, 8, 9, 10, 0], vec![2, 5]),
            LeftRemoved(vec![1, 2, 3, 4, 5, 6, 11, 12, 0], vec![3, 4]),
            LeftRemoved(vec![1, 2, 3, 4, 5, 6, 9, 10, 0], vec![3, 5]),
            LeftRemoved(vec![1, 2, 3, 4, 5, 6, 7, 8, 0], vec![4, 5]),
        ];

        check2(&mut c, nexts);

        dbg!("ooo");

        let mut sol = Sol::new(&data);
        sol.add_route(&vec![0, 1, 3, 5, 4, 2, 6, 7, 8, 9, 10, 11, 12, 0]);
        c.k_combinations_of_route(&sol, 1, 1);
        assert_eq!(c.k, 1);
        assert_eq!(c.route_len, 14);
        assert_eq!(c.combination_indices2[0], 0);
        assert_eq!(c.combination_indices2[1], 6);

        let nexts = vec![
            LeftRemoved(vec![3, 5, 4, 6, 7, 8, 9, 10, 11, 12, 0], vec![0]),
            LeftRemoved(vec![1, 5, 2, 6, 7, 8, 9, 10, 11, 12, 0], vec![1]),
            LeftRemoved(vec![1, 3, 4, 2, 7, 8, 9, 10, 11, 12, 0], vec![2]),
            LeftRemoved(vec![1, 3, 5, 4, 2, 6, 9, 10, 11, 12, 0], vec![3]),
            LeftRemoved(vec![1, 3, 5, 4, 2, 6, 7, 8, 11, 12, 0], vec![4]),
            LeftRemoved(vec![1, 3, 5, 4, 2, 6, 7, 8, 9, 10, 0], vec![5]),
        ];

        check2(&mut c, nexts);

        // sol.remove_route(1);
        // sol.add_route(&vec![0, 1, 3, 5, 4, 2, 6, 7, 8, 9, 10, 11, 12, 0]);
        // c.k_combinations_of_route(&sol, 1, 2);

        // let nexts = vec![
        //     LeftRemoved(vec![5, 6, 7, 8, 9, 10, 11, 12, 0], vec![1, 3]),
        //     LeftRemoved(vec![3, 4, 7, 8, 9, 10, 11, 12, 0], vec![1, 5]),
        //     LeftRemoved(vec![3, 5, 4, 6, 9, 10, 11, 12, 0], vec![1, 7]),
        //     LeftRemoved(vec![3, 5, 4, 6, 7, 8, 11, 12, 0], vec![1, 9]),
        //     LeftRemoved(vec![3, 5, 4, 6, 7, 8, 9, 10, 0], vec![1, 11]),
        //     LeftRemoved(vec![1, 2, 7, 8, 9, 10, 11, 12, 0], vec![3, 5]),
        //     LeftRemoved(vec![1, 5, 2, 6, 9, 10, 11, 12, 0], vec![3, 7]),
        //     LeftRemoved(vec![1, 5, 2, 6, 7, 8, 11, 12, 0], vec![3, 9]),
        //     LeftRemoved(vec![1, 5, 2, 6, 7, 8, 9, 10, 0], vec![3, 11]),
        //     LeftRemoved(vec![1, 3, 4, 2, 9, 10, 11, 12, 0], vec![5, 7]),
        //     LeftRemoved(vec![1, 3, 4, 2, 7, 8, 11, 12, 0], vec![5, 9]),
        //     LeftRemoved(vec![1, 3, 4, 2, 7, 8, 9, 10, 0], vec![5, 11]),
        //     LeftRemoved(vec![1, 3, 5, 4, 2, 6, 11, 12, 0], vec![7, 9]),
        //     LeftRemoved(vec![1, 3, 5, 4, 2, 6, 9, 10, 0], vec![7, 11]),
        //     LeftRemoved(vec![1, 3, 5, 4, 2, 6, 7, 8, 0], vec![9, 11]),
        // ];
        // check2(&mut c, nexts);
    }

    fn check2(c: &mut Combinations, nexts: Vec<LeftRemoved>) {
        for (i, LeftRemoved(next, removed)) in nexts.iter().enumerate() {
            dbg!(next);
            dbg!(c.r());
            dbg!(c.into_iter().removed);
            dbg!(c.combination_indices2);
            assert_eq!(c.r(), *next);
            assert_eq!(c.into_iter().collect_vec(), *next);
            // assert_eq!(c.removed(), *removed);
            let removed2 = c
                .removed_idxs()
                .iter()
                .copied()
                // .map(|x| *x)
                .collect_vec();
            assert_eq!(removed2, *removed);
            assert_eq!(c.next_combination_with_lower_score(u64::max_value()), i != nexts.len() - 1);
            dbg!(c.combination_indices2);
        }
    }
}
