use itertools::Itertools;

use crate::data::{Data, PTS};
use crate::eval::Eval;
use crate::mov::{Between, Move, Swap};
use crate::{sol::Sol, K_MAX, UNSERVED};
use std::ops::Not;

use self::comb::{Comb, Comb2, Comb2Iter};
use self::{
    delivery_evaluator::DeliveryInsertionEvaluator, pickup_evaluator::PickupInsertionEvaluator,
};

pub mod delivery_evaluator;
pub mod pickup_evaluator;

pub mod comb;

pub struct Evaluator<'a> {
    data: &'a Data,
    pickup_idx: usize,
    delivery_idx: usize,
    pickup_evaluator: PickupInsertionEvaluator<'a>,
    delivery_evaluator: DeliveryInsertionEvaluator<'a>,
    removed: [usize; 2 * K_MAX],
    jump_forward: [usize; PTS],
    // jump_backward: [i32; PTS],
}

impl<'a> Evaluator<'a> {
    pub fn new(data: &'a Data) -> Self {
        let mut jump_forward = [0; PTS];
        jump_forward
            .iter_mut()
            .enumerate()
            .for_each(|(i, x)| *x = i);
        Self {
            data,
            pickup_idx: UNSERVED,
            delivery_idx: UNSERVED,
            pickup_evaluator: PickupInsertionEvaluator::new(data),
            delivery_evaluator: DeliveryInsertionEvaluator::new(data),
            removed: [0; 2 * K_MAX],
            jump_forward,
            // jump_backward: [0; PTS],
        }
    }

    pub fn with_pickup(data: &'a Data, pickup_idx: usize) -> Self {
        let mut evaluator = Self::new(data);
        evaluator.reset(pickup_idx);

        evaluator
    }

    pub fn reset(&mut self, pickup_idx: usize) {
        self.pickup_idx = pickup_idx;
        self.delivery_idx = self.data.pair_of(pickup_idx);
        self.removed.iter().for_each(|&x| {
            self.jump_forward[x] = x;
            // self.jump_backward[x] = 0;
        });
        self.removed = [0; 2 * K_MAX];
    }

    pub fn remove(&mut self, sol: &Sol, idx: usize) {
        let mut x = sol.next[self.jump_forward[idx]];

        while x != self.jump_forward[x] {
            x = self.jump_forward[x];
        }

        self.jump_forward[idx] = x;
    }

    pub fn remove_pair(&mut self, sol: &Sol, pickup_idx: usize) {
        let delivery_idx = self.data.pair_of(pickup_idx);
        self.remove(sol, delivery_idx);
        self.remove(sol, pickup_idx);
    }

    pub fn unremove_pair(&mut self, pickup_idx: usize) {
        let delivery_idx = self.data.pair_of(pickup_idx);
        self.jump_forward[pickup_idx] = pickup_idx;
        self.jump_forward[delivery_idx] = delivery_idx;
    }

    pub fn check_add_to_route(&mut self, sol: &Sol, start: usize) -> Option<Move> {
        let mut mov = Move::new(self.pickup_idx);
        self.pickup_evaluator
            .reset(sol, sol.prev[start], self.pickup_idx, start);

        while self.pickup_evaluator.can_continue() {
            self.pickup_evaluator.insert_pickup(sol);

            if self.pickup_evaluator.pickup_insertion_is_feasible() {
                self.check_delivery_insertions(sol, &mut mov);
            }

            self.pickup_evaluator.advance(sol, &self.jump_forward);
        }

        mov.is_empty().not().then_some(mov)
    }

    fn check_delivery_insertions(&mut self, sol: &Sol, mov: &mut Move) {
        self.delivery_evaluator
            .reset(self.delivery_idx, &self.pickup_evaluator);

        self.delivery_evaluator
            .check_rest_of_route(sol, &self.jump_forward, mov);
    }

    pub fn check_add_to_route_with_k_removed(
        &mut self,
        sol: &Sol,
        route_start: usize,
        k: usize,
    ) -> Option<Move> {
        let mut mov = Move::new(self.pickup_idx);

        let route_pickups = self.route_pickups(route_start, sol);
        if k < route_pickups.len() {
            let pickup_removed_times = sol.removed_times[self.pickup_idx];

            let mut cc = Comb::new();
            cc.reset(&route_pickups, k);

            for comb in route_pickups.iter().combinations(k) {
                let comb = comb.iter().copied().copied().collect_vec();

                // let not_whole_route = k != route_pickups.len();

                let comb_total_removed_times: u64 =
                    comb.iter().map(|&x| sol.removed_times[x]).sum();
                let comb_has_lower_removal_score = comb_total_removed_times < pickup_removed_times;
                if comb_has_lower_removal_score {
                    self.remove_all(&comb, sol);
                    let mut m = self.check_route(route_start, sol);
                    self.unremove_all(&comb);

                    if !m.is_empty() {
                        m.removed[..k].copy_from_slice(&comb);
                        mov.pick(&m)
                    }
                }
                // if !mov.is_empty() {
                //     break;
                // }
            }
        }

        mov.is_empty().not().then_some(mov)
    }

    pub fn check_swap(&mut self, sol: &Sol, a_pickup: usize, b_pickup: usize) -> Option<Swap> {
        debug_assert!(a_pickup != b_pickup);
        debug_assert!(!sol.data.pts[a_pickup].delivery);
        debug_assert!(!sol.data.pts[b_pickup].delivery);

        let a_route = sol.first[a_pickup];
        let b_route = sol.first[b_pickup];
        debug_assert!(a_route != b_route);
        let comb = vec![a_pickup, b_pickup];

        self.reset(a_pickup);
        self.remove_all(&comb, sol);
        let a = self.check_route(a_route, sol);

        let mut s = Swap::new(a_pickup, b_pickup);

        if !a.is_empty() {
            self.pickup_idx = b_pickup;
            self.delivery_idx = self.data.pair_of(b_pickup);
            let b = self.check_route(b_route, sol);

            if !b.is_empty() {
                s.a = a;
                s.b = b;
            }
        }
        self.unremove_all(&comb);

        s.is_empty().not().then_some(s)
    }

    fn check_route(&mut self, route_start: usize, sol: &Sol<'_>) -> Move {
        let mut mov = Move::new(self.pickup_idx);
        let mut start = route_start;
        while start != self.jump_forward[start] {
            start = self.jump_forward[start];
        }

        self.pickup_evaluator.reset(sol, 0, self.pickup_idx, start);

        while self.pickup_evaluator.can_continue() {
            self.pickup_evaluator.insert_pickup(sol);

            if self.pickup_evaluator.pickup_insertion_is_feasible() {
                self.check_delivery_insertions(sol, &mut mov);
            }

            self.pickup_evaluator.advance(sol, &self.jump_forward);
        }

        mov
    }

    pub fn check_add_to_route_with_k_removed2(
        &mut self,
        sol: &Sol,
        route_start: usize,
        k: usize,
    ) -> Option<Move> {
        let mut mov = Move::new(self.pickup_idx);

        let route_pickups = self.route_pickups(route_start, sol);
        if k < route_pickups.len() {
            let pickup_removed_times = sol.removed_times[self.pickup_idx];

            let mut cc = Comb2::new();
            cc.from_route(sol, route_start, k);

            let mut ok = true;

            while ok {
                let comb_total_removed_times: u64 =
                    cc.removed().iter().map(|&x| sol.removed_times[x]).sum();
                let comb_has_lower_removal_score = comb_total_removed_times < pickup_removed_times;
                if comb_has_lower_removal_score {
                    let mut m = self.check_route2(&mut cc.into_iter(), sol);

                    if !m.is_empty() {
                        m.removed[..k].copy_from_slice(cc.removed());
                        mov.pick(&m)
                    }
                }
                // if !mov.is_empty() {
                //     break;
                // }

                ok = cc.next_comb();
            }
        }

        mov.is_empty().not().then_some(mov)
    }

    // fn check_route2(&mut self, ci: impl Iterator<Item = usize>, sol: &Sol<'_>) -> Move {
    fn check_route2(&mut self, ci: &mut Comb2Iter, sol: &Sol<'_>) -> Move {
        // let mut it = ci.tuple_windows();
        let mut mov = Move::new(self.pickup_idx);
        // let (depot, start) = *(it.peek().unwrap());

        let mut pickup_evaluator = Eval::new();
        let mut delivery_evaluator = Eval::new();
        let mut prev = ci.next().unwrap();

        while let Some(next) = ci.next() {
            pickup_evaluator.reset_to(&sol.evals[prev]);
            pickup_evaluator.next(self.pickup_idx, sol.data);

            if pickup_evaluator.is_feasible(sol.data) {
                let mut ci2 = ci.clone();
                delivery_evaluator.reset_to(&pickup_evaluator);

                let mut prev2 = self.pickup_idx;
                let mut next2 = next;
                while prev2 != 0 && delivery_evaluator.is_feasible(sol.data) {
                    if delivery_evaluator.can_delivery_be_inserted(
                        self.delivery_idx,
                        next2,
                        self.data,
                        sol.latest_feasible_departure[next2],
                    ) {
                        mov.maybe_switch(&Between(prev, next), &Between(prev2, next2));
                    }

                    delivery_evaluator.next(next2, sol.data);
                    prev2 = next2;
                    next2 = ci2.next().unwrap_or(0);
                }

                // fn check_delivery_insertions(&mut self, sol: &Sol, mov: &mut Move) {
                //     self.delivery_evaluator
                //         .reset(self.delivery_idx, &self.pickup_evaluator);

                //     self.delivery_evaluator
                //         .check_rest_of_route(sol, &self.jump_forward, mov);
                // }
            }

            prev = next;
        }

        // for (prev, next) in ci.tuple_windows() {
        //     pickup_evaluator.reset_to(&sol.evals[prev]);
        //     let ci2 = ci.clone();
        // }

        // println!("{start}");

        // self.pickup_evaluator.reset(sol, 0, self.pickup_idx, start);

        // while self.pickup_evaluator.can_continue() {
        //     self.pickup_evaluator.insert_pickup(sol);

        //     if self.pickup_evaluator.pickup_insertion_is_feasible() {
        //         self.check_delivery_insertions(sol, &mut mov);
        //     }

        //     self.pickup_evaluator.advance(sol, &self.jump_forward);
        // }

        mov
    }

    pub fn unremove_all(&mut self, comb: &Vec<usize>) {
        for &x in comb.iter() {
            self.unremove_pair(x)
        }
    }

    pub fn remove_all(&mut self, comb: &Vec<usize>, sol: &Sol) {
        for &x in comb.iter() {
            self.remove_pair(sol, x)
        }
    }

    fn route_pickups(&mut self, route_start: usize, sol: &Sol) -> Vec<usize> {
        let pts = &self.data.pts;
        sol.route_iter(route_start)
            .filter_map(|x| (!pts[x].delivery).then_some(x))
            .collect_vec()
    }
}
