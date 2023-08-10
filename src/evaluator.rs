use crate::data::{Data, PTS};
use crate::eval::Eval;
use crate::mov::{Between, Move, Swap};
use crate::{sol::Sol, UNSERVED};
use std::ops::Not;

use self::comb::Combinations;

pub mod comb;

pub struct Evaluator<'a> {
    data: &'a Data,
    pickup_idx: usize,
    delivery_idx: usize,
    cc: Combinations,
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
            cc: Combinations::new(),
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
    }

    pub fn check_add_to_route2(&mut self, sol: &Sol, start: usize) -> Option<Move> {
        let mov = self.check_route2(self.pickup_idx, &mut sol.route_iter(start), sol);

        mov.is_empty().not().then_some(mov)
    }

    pub fn check_swap(&mut self, sol: &Sol, a_pickup: usize, b_pickup: usize) -> Option<Swap> {
        debug_assert!(a_pickup != b_pickup);
        debug_assert!(!sol.data.pts[a_pickup].is_delivery);
        debug_assert!(!sol.data.pts[b_pickup].is_delivery);

        let a_route = sol.first[a_pickup];
        let b_route = sol.first[b_pickup];
        debug_assert!(a_route != b_route);

        let mut s = Swap::new(a_pickup, b_pickup);
        let a_delivery = sol.data.pair_of(a_pickup);

        let mut a_iter = sol
            .route_iter(a_route)
            .filter(|&x| x != a_pickup && x != a_delivery);

        let a = self.check_route2(b_pickup, &mut a_iter, sol);

        if !a.is_empty() {
            let b_delivery = sol.data.pair_of(b_pickup);
            let mut b_iter = sol
                .route_iter(b_route)
                .filter(|&x| x != b_pickup && x != b_delivery);
            let b = self.check_route2(a_pickup, &mut b_iter, sol);

            if !b.is_empty() {
                s.a = a;
                s.b = b;
            }
        }

        s.is_empty().not().then_some(s)
    }

    pub fn check_add_to_route_with_k_removed2(
        &mut self,
        sol: &Sol,
        route_start: usize,
        k: usize,
    ) -> Option<Move> {
        let mut mov = Move::new(self.pickup_idx);

        self.cc.k_combinations_of_route(sol, route_start, k);
        if k < self.cc.len / 2 {
            let pickup_removed_times = sol.removed_times[self.pickup_idx];

            let mut ok = if self.cc.cur_removed_times_total >= pickup_removed_times {
                self.cc.next_combination_with_lower_score(pickup_removed_times)
            } else {
                true
            };

            while ok {
                // let comb_total_removed_times: u64 = self
                //     .cc
                //     .removed()
                //     .iter()
                //     .map(|&x| sol.removed_times[x])
                //     .sum();
                // let comb_has_lower_removal_score = comb_total_removed_times < pickup_removed_times;
                // if comb_has_lower_removal_score {
                let mut m = self.check_route2(self.pickup_idx, &mut self.cc.into_iter(), sol);

                if !m.is_empty() {
                    m.removed[..k].copy_from_slice(self.cc.removed());
                    mov.pick(&m)
                }
                // }
                // if !mov.is_empty() {
                //     break;
                // }

                ok = self.cc.next_combination_with_lower_score(pickup_removed_times);
            }
        }

        mov.is_empty().not().then_some(mov)
    }

    fn check_route2<T: Iterator<Item = usize> + Clone>(
        &self,
        pickup: usize,
        ci: &mut T,
        sol: &Sol<'_>,
    ) -> Move {
        let mut mov = Move::new(pickup);
        let delivery_idx = sol.data.pair_of(pickup);

        let mut pickup_evaluator = Eval::new();
        let mut delivery_evaluator = Eval::new();
        let mut prev = 0;

        while let Some(next) = ci.next() {
            pickup_evaluator.reset_to(&sol.evals[prev]);
            pickup_evaluator.next(pickup, self.data);

            if pickup_evaluator.is_feasible(self.data) {
                let mut ci2 = ci.clone();
                delivery_evaluator.reset_to(&pickup_evaluator);

                let mut prev2 = pickup;
                let mut next2 = next;
                while prev2 != 0 && delivery_evaluator.is_feasible(self.data) {
                    if delivery_evaluator.can_delivery_be_inserted(
                        delivery_idx,
                        next2,
                        self.data,
                        sol.latest_feasible_departure[next2],
                    ) {
                        mov.maybe_switch(&Between(prev, next), &Between(prev2, next2));
                    }

                    if delivery_evaluator.time > self.data.pts[delivery_idx].due {
                        break;
                    }

                    delivery_evaluator.next(next2, self.data);
                    prev2 = next2;
                    next2 = ci2.next().unwrap_or(0);
                }
            }

            if !pickup_evaluator.is_time_feasible(self.data) {
                break
            }

            prev = next;
        }

        mov
    }
}
