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
}

impl PairInfo {
    pub fn new(id: usize, idx: usize, pair_idx: usize, removed_times: u64) -> Self {
        Self {
            id,
            idx,
            pair_idx,
            removed_times,
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
        }
    }
}

pub struct Combinations {
    pub k: usize,
    pub route_len: usize,
    pub cur_removed_times_total: u64,
    pub route: [PairInfo; PTS],
    pub route_position: [usize; PTS],
    pub pickups: [PairInfo; PTS / 2],
    pub combination_indices: [usize; K_MAX + 1],
    pub sum_of_next: [[u64; K_MAX + 1]; PTS],
}

impl Combinations {
    pub fn new() -> Self {
        let mut route = [Default::default(); PTS];
        route[0] = PairInfo::new(0, 0, 0, 0);

        Self {
            k: 0,
            route_len: 0,
            cur_removed_times_total: 0,
            route,
            route_position: [UNSERVED; PTS],
            pickups: [Default::default(); PTS / 2],
            combination_indices: [UNSERVED; K_MAX + 1],
            sum_of_next: [[u64::max_value(); K_MAX + 1]; PTS],
        }
    }

    pub fn k_combinations_of_route(&mut self, sol: &Sol, route_start: usize, k: usize) {
        self.k = k;
        self.fill_route_indices_and_set_len(route_start, sol);
        self.copy_pair_info(route_start, sol);
        self.calculate_sum_of_next_k();
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
    }

    fn calculate_sum_of_next_k(&mut self) {
        let pickups_len = self.pickups_len();
        for i in 0..pickups_len {
            for k in 0..=self.k.min(pickups_len - i) {
                // perhaps start at 1?
                self.sum_of_next[i][k] = self.pickups[(i + 1)..(i + 1 + k)]
                    .iter()
                    .map(|x| x.removed_times)
                    .sum();
            }
        }
    }

    fn pickups_len(&mut self) -> usize {
        (self.route_len - 2) / 2
    }

    fn initialize_combination_indices(&mut self) {
        self.set_initial_position_of_indices();
        self.set_k_plus_1_guard_value();
        self.fill_rest_of_combination_indices();
        self.initialize_removed_times_sum();
    }

    fn set_initial_position_of_indices(&mut self) {
        for i in 0..self.k {
            self.combination_indices[i] = i;
        }
    }
    fn set_k_plus_1_guard_value(&mut self) {
        self.combination_indices[self.k] = self.pickups_len()
    }

    fn fill_rest_of_combination_indices(&mut self) {
        self.combination_indices[self.k + 1..=K_MAX].fill(UNSERVED);
    }

    fn is_removed(&self, x: usize) -> bool {
        for i in 0..self.k {
            let pickup = self.pickups[self.combination_indices[i]];
            let delivery = self.route[pickup.pair_idx].id;

            if x == pickup.id || x == delivery {
                return true;
            }
        }

        return false;
    }

    fn initialize_removed_times_sum(&mut self) {
        self.cur_removed_times_total = self.combination_indices[0..self.k]
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
        let mut sum_of_removed_times_of_combination_indices_i_to_k = 0;
        for i in (0..self.k).rev() {
            sum_of_removed_times_of_combination_indices_i_to_k +=
                self.pickups[self.combination_indices[i]].removed_times;

            let can_advance_ith_iterator_one_position =
                self.combination_indices[i] + 1 != self.combination_indices[i + 1];

            if can_advance_ith_iterator_one_position {
                let new_sum = self.cur_removed_times_total
                    - sum_of_removed_times_of_combination_indices_i_to_k
                    + self.sum_of_next[self.combination_indices[i]][self.k - i];

                if new_sum < target_score {
                    let after_i = self.combination_indices[i] + 1;
                    for j in i..self.k {
                        self.combination_indices[j] = after_i + (j - i);
                    }
                    self.cur_removed_times_total = new_sum;
                    return true;
                }
            }
        }

        false
    }

    pub fn removed_idxs(&self) -> &[usize] {
        &self.combination_indices[0..self.k]
    }

    pub fn pickups(&self) -> &[PairInfo] {
        &self.pickups[0..PTS / 2]
    }
}

#[derive(Clone, Copy)]
pub struct CombinationIterator<'a> {
    comb: &'a Combinations,
    route_indices_to_skip: [usize; 2 * K_MAX],
    next_to_skip_idx: usize,
    cur: usize,
}

impl<'a> CombinationIterator<'a> {
    pub fn new(comb: &'a Combinations) -> Self {
        let mut s = Self {
            comb,
            route_indices_to_skip: CombinationIterator::prepare_skip_list(comb),
            next_to_skip_idx: 0,
            cur: 1,
        };

        s.skip_indices_if_cur_on_skip_list();

        s
    }

    fn skip_indices_if_cur_on_skip_list(&mut self) {
        while self.cur < self.comb.route_len - 1
            && self.next_to_skip_idx < 2 * K_MAX
            && self.route_indices_to_skip[self.next_to_skip_idx] == self.cur
        {
            self.cur += 1;
            self.next_to_skip_idx += 1;
        }
    }

    fn prepare_skip_list(comb: &Combinations) -> [usize; 2 * K_MAX] {
        let mut skip_list = [0; 2 * K_MAX];

        for (i, &r) in comb.removed_idxs().iter().enumerate() {
            let pickup = &comb.pickups()[r];
            skip_list[2 * i] = pickup.idx;
            skip_list[2 * i + 1] = pickup.pair_idx;
        }

        skip_list[0..(2 * comb.k)].sort_unstable();
        skip_list
    }
}

impl<'a> Iterator for CombinationIterator<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        let res = (self.cur < self.comb.route_len).then_some(self.comb.route[self.cur].id);

        self.cur += 1;
        self.skip_indices_if_cur_on_skip_list();

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
        assert_eq!(c.combination_indices[0], 0);
        assert_eq!(c.combination_indices[1], 6);
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
        assert_eq!(c.combination_indices[0], 0);
        assert_eq!(c.combination_indices[1], 1);
        assert_eq!(c.combination_indices[2], 6);

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
        assert_eq!(c.combination_indices[0], 0);
        assert_eq!(c.combination_indices[1], 6);

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
            dbg!(c.into_iter().route_indices_to_skip);
            dbg!(c.combination_indices);
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
            assert_eq!(
                c.next_combination_with_lower_score(u64::max_value()),
                i != nexts.len() - 1
            );
            dbg!(c.combination_indices);
        }
    }
}
