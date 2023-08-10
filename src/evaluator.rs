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
    combinations: Combinations,
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
            combinations: Combinations::new(),
        }
    }

    pub fn with_pickup(data: &'a Data, pickup_idx: usize) -> Self {
        let mut evaluator = Self::new(data);
        evaluator.reset(pickup_idx);

        evaluator
    }

    pub fn reset(&mut self, pickup_idx: usize) {
        self.pickup_idx = pickup_idx;
    }

    pub fn check_add_to_route(&mut self, sol: &Sol, start: usize) -> Option<Move> {
        let mut iterator = sol.route_iter(start);
        let mov = self.check_insertions_into_route(self.pickup_idx, &mut iterator, sol);

        mov.is_empty().not().then_some(mov)
    }

    pub fn check_swap(&mut self, sol: &Sol, a_pickup: usize, b_pickup: usize) -> Option<Swap> {
        debug_assert!(a_pickup != b_pickup);
        debug_assert!(!sol.data.pts[a_pickup].is_delivery);
        debug_assert!(!sol.data.pts[b_pickup].is_delivery);
        debug_assert!(sol.first[a_pickup] != sol.first[b_pickup]);

        let mut s = Swap::new(a_pickup, b_pickup);

        let a = self.check_remove_and_insert(sol, a_pickup, b_pickup);
        if !a.is_empty() {
            let b = self.check_remove_and_insert(sol, b_pickup, a_pickup);
            if !b.is_empty() {
                s.a = a;
                s.b = b;
            }
        }

        s.is_empty().not().then_some(s)
    }

    fn check_remove_and_insert(&mut self, sol: &Sol, to_remove: usize, to_insert: usize) -> Move {
        let to_remove_pair = sol.data.pair_of(to_remove);

        let route_of_to_remove = sol.first[to_remove];

        let mut route_iterator = sol
            .route_iter(route_of_to_remove)
            .filter(|&x| x != to_remove && x != to_remove_pair);

        self.check_insertions_into_route(to_insert, &mut route_iterator, sol)
    }

    pub fn check_add_to_route_with_k_removed2(
        &mut self,
        sol: &Sol,
        route_start: usize,
        k: usize,
    ) -> Option<Move> {
        let mut mov = Move::new(self.pickup_idx);

        self.combinations
            .k_combinations_of_route(sol, route_start, k);

        if k < self.combinations.len / 2 {
            let pickup_removed_times = sol.removed_times[self.pickup_idx];

            let mut ok = if self.combinations.cur_removed_times_total >= pickup_removed_times {
                self.combinations
                    .next_combination_with_lower_score(pickup_removed_times)
            } else {
                true
            };

            while ok {
                let mut m = self.check_insertions_into_route(
                    self.pickup_idx,
                    &mut self.combinations.into_iter(),
                    sol,
                );

                if !m.is_empty() {
                    m.removed[..k].copy_from_slice(self.combinations.removed());
                    mov.pick(&m)
                }

                ok = self
                    .combinations
                    .next_combination_with_lower_score(pickup_removed_times);
            }
        }

        mov.is_empty().not().then_some(mov)
    }

    fn check_insertions_into_route<ClonableIterator: Iterator<Item = usize> + Clone>(
        &self,
        pickup: usize,
        route_iterator: &mut ClonableIterator,
        sol: &Sol,
    ) -> Move {
        let mut mov = Move::new(pickup);
        let delivery_idx = sol.data.pair_of(pickup);

        let mut pickup_evaluator = Eval::new();
        let mut delivery_evaluator = Eval::new();
        let mut prev = 0;

        while let Some(next) = route_iterator.next() {
            pickup_evaluator.reset_to(&sol.evals[prev]);
            pickup_evaluator.next(pickup, self.data);

            if pickup_evaluator.is_feasible(self.data) {
                let mut ci2 = route_iterator.clone();
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
                break;
            }

            prev = next;
        }

        mov
    }
}
