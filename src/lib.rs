use data::{idx, Data, PTS};
use eval::Eval;
use itertools::Itertools;
use mov::Between;

use crate::mov::Move;

pub mod data;
pub mod eval;
pub mod mov;

const UNSERVED: usize = usize::MAX;
const K_MAX: usize = 2;

pub struct Removed {
    pub pickup: usize,
    pub delivery: usize,
    pub times: usize,
}

pub struct Sol {
    // pub data: Rc<Data>,
    pub next: [usize; PTS],
    pub prev: [usize; PTS],
    pub latest_feasible_departure: [u64; PTS],
    pub removed_times: [u64; PTS],
    pub removed_order: [usize; PTS],
    pub evals: Vec<Eval>,
}

impl Sol {
    pub fn new() -> Self {
        let next = [UNSERVED; PTS];
        let prev = [UNSERVED; PTS];
        let latest_feasible_departure = [0; PTS];
        let removed_times = [0; PTS];
        let removed_order = (0..PTS).collect::<Vec<usize>>().try_into().unwrap();
        let evals: Vec<_> = (0..PTS).map(|_| Eval::new()).collect();

        Sol {
            next,
            prev,
            latest_feasible_departure,
            removed_times,
            removed_order,
            evals,
        }
    }

    pub fn add_route(&mut self, data: &Data, route: &Vec<usize>) {
        let pts = &data.pts;
        let mut prev = 0;
        let mut latest_feasible_departure = pts[0].due;
        for &n in route.iter().rev() {
            let drive_time = data.time[idx(n, prev)];
            latest_feasible_departure = latest_feasible_departure - drive_time;
            self.latest_feasible_departure[n] = latest_feasible_departure;
            latest_feasible_departure = std::cmp::min(pts[prev].due, latest_feasible_departure);
            prev = n;
        }

        self.prev[route[0]] = 0;
        for (&bef, &aft) in route.iter().tuple_windows() {
            self.prev[aft] = bef;
            self.next[bef] = aft;
        }
        self.next[*route.last().unwrap()] = 0;

        let mut ev = Eval::new();

        for &n in route.iter() {
            ev.next(n, data);
            self.evals[n].reset_to(&ev);
        }

        debug_assert!({
            let mut e = Eval::new();
            route.iter().fold(true, |acc, &n| {
                e.next(n, data);
                acc && e.time < self.latest_feasible_departure[n]
            })
        });
    }
}

pub struct Evaluator<'a> {
    sol: &'a Sol,
    data: &'a Data,
    pickup_id: usize,
    delivery_id: usize,
    pickup_evaluator: Eval,
    delivery_evaluator: Eval,
    removed: [Option<usize>; K_MAX],
}

impl<'a> Evaluator<'a> {
    pub fn new(sol: &'a Sol, data: &'a Data) -> Self {
        Self {
            sol,
            data,
            pickup_id: UNSERVED,
            delivery_id: UNSERVED,
            pickup_evaluator: Eval::new(),
            delivery_evaluator: Eval::new(),
            removed: [None; K_MAX],
        }
    }

    pub fn with_pickup(sol: &'a Sol, data: &'a Data, pickup_id: usize) -> Self {
        let mut e = Self::new(sol, data);
        e.reset(pickup_id);

        e
    }

    pub fn reset(&mut self, pickup_id: usize) {
        self.pickup_id = pickup_id;
        self.delivery_id = self.data.pair_of(pickup_id);
        self.removed = [None; K_MAX];
    }

    pub fn check_add_to_route(&mut self, start: usize) -> Option<Move> {
        let mut after_pickup = start;
        let mut before_pickup = self.sol.prev[after_pickup];
        let mut mov = Move::new();

        while after_pickup != 0 {
            self.pickup_evaluator
                .reset_to(&self.sol.evals[before_pickup]);
            self.pickup_evaluator.next(self.pickup_id, self.data);

            let can_insert_pickup = self.pickup_evaluator.check(self.data);

            if can_insert_pickup {
                let pickup_goes_between = Between(before_pickup, after_pickup);
                self.check_delivery_insertions(pickup_goes_between, after_pickup, &mut mov);
            }

            before_pickup = after_pickup;
            after_pickup = self.sol.next[after_pickup];
        }

        if mov.empty() {
            None
        } else {
            Some(mov)
        }
    }

    fn check_delivery_insertions(
        &mut self,
        put_pickup_between: Between,
        cur: usize,
        mov: &mut Move,
    ) {
        self.delivery_evaluator.reset_to(&self.pickup_evaluator);

        let mut after_delivery_id = cur;
        let mut last_in_route = cur; // I don't like this, but too dumb to figure out something smarter
        while last_in_route != 0 {
            let can_insert_delivery = self.delivery_evaluator.check_insert(
                self.delivery_id,
                after_delivery_id,
                self.data,
                self.sol.latest_feasible_departure[after_delivery_id],
            );

            if can_insert_delivery {
                let put_delivery_between =
                    Between(self.sol.prev[after_delivery_id], after_delivery_id);

                mov.maybe_switch(&put_pickup_between, &put_delivery_between);
            }

            self.delivery_evaluator.next(after_delivery_id, self.data);

            last_in_route = after_delivery_id;
            after_delivery_id = self.sol.next[after_delivery_id];
        }
    }
}

#[cfg(test)]
mod tests {
    // use super::*;
}
