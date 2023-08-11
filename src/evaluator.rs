use crate::data::Data;
use crate::eval::Eval;
use crate::mov::{Between, Move, Swap};
use crate::{sol::Sol, UNSERVED};
use std::ops::Not;

use self::comb::Combinations;

pub mod comb;

pub struct Evaluator<'a> {
    data: &'a Data,
    combinations: Combinations,
    pickup_idx: usize,
}

impl<'a> Evaluator<'a> {
    pub fn new(data: &'a Data) -> Self {
        Self {
            data,
            combinations: Combinations::new(),
            pickup_idx: UNSERVED,
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

    pub fn check_add_to_route_with_k_removed(
        &mut self,
        sol: &Sol,
        route_start: usize,
        k: usize,
    ) -> Option<Move> {
        let mut mov = Move::new(self.pickup_idx);

        self.combinations
            .k_combinations_of_route(sol, route_start, k);

        let pickups_in_route = self.combinations.route_len / 2;
        let not_removing_whole_route = k < pickups_in_route;
        if not_removing_whole_route {
            let pickup_removed_times = sol.removed_times(self.pickup_idx);

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
        pickup_iterator: &mut ClonableIterator,
        sol: &Sol,
    ) -> Move {
        let mut mov = Move::new(pickup);
        let delivery_idx = sol.data.pair_of(pickup);
        let delivery_due = self.data.pts[delivery_idx].due;

        let mut before_pickup = 0;
        let mut normal_route_eval = Eval::new();
        let mut insertion_eval = Eval::new();

        while let Some(after_pickup) = pickup_iterator.next() {
            insertion_eval.reset_to(&normal_route_eval);
            insertion_eval.next(pickup, self.data);

            if insertion_eval.arrives_too_late(self.data) {
                break;
            }

            if insertion_eval.is_feasible(self.data) {
                let mut delivery_iterator = pickup_iterator.clone();

                let mut before_delivery = pickup;
                let mut after_delivery = after_pickup;
                while before_delivery != 0 && insertion_eval.is_feasible(self.data) {
                    if insertion_eval.can_delivery_be_inserted(
                        delivery_idx,
                        after_delivery,
                        self.data,
                        sol.latest_feasible_departure[after_delivery],
                    ) {
                        mov.maybe_switch(
                            &Between(before_pickup, after_pickup),
                            &Between(before_delivery, after_delivery),
                        );
                    }

                    let too_late_for_delivery = insertion_eval.time > delivery_due;
                    if too_late_for_delivery {
                        break;
                    }

                    insertion_eval.next(after_delivery, self.data);
                    before_delivery = after_delivery;
                    after_delivery = delivery_iterator.next().unwrap_or(0);
                }
            }

            normal_route_eval.next(after_pickup, self.data);
            before_pickup = after_pickup;
        }

        mov
    }
}
