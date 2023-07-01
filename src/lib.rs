use data::{idx, Data, PTS};
use eval::Eval;
use itertools::Itertools;
use mov::{Between, Move};

pub mod data;
pub mod eval;
pub mod evaluator;
pub mod mov;

const UNSERVED: usize = usize::MAX;
const K_MAX: usize = 2;

// pub struct Removed {
//     pub pickup: usize,
//     pub delivery: usize,
//     pub times: usize,
// }

pub struct Sol<'a> {
    pub data: &'a Data,
    pub next: [usize; PTS],
    pub prev: [usize; PTS],
    pub latest_feasible_departure: [u64; PTS],
    pub removed_times: [u64; PTS],
    pub removed_heap: [usize; PTS],
    pub removed_idx: [usize; PTS],
    pub evals: Vec<Eval>,
    pub heap_size: usize,
}

impl<'a> Sol<'a> {
    pub fn new(data: &'a Data) -> Self {
        let next = [UNSERVED; PTS];
        let prev = [UNSERVED; PTS];
        let mut latest_feasible_departure = [0; PTS];
        latest_feasible_departure[0] = data.pts[0].due;
        let removed_times = [0; PTS];
        let removed_heap = (0..PTS).collect::<Vec<usize>>().try_into().unwrap();
        let removed_idx = [UNSERVED; PTS];
        let evals: Vec<_> = (0..PTS).map(|_| Eval::new()).collect();

        Sol {
            data,
            next,
            prev,
            latest_feasible_departure,
            removed_times,
            removed_heap,
            removed_idx,
            evals,
            heap_size: 0,
        }
    }

    fn recalc_route(&mut self, recalc_last: usize) {
        if recalc_last == 0 {
            return;
        }

        let mut node = recalc_last;
        let pts = &self.data.pts;
        let mut after_node = self.next[node];
        let mut latest_feasible_departure = self.latest_feasible_departure[after_node];

        while node != 0 {
            let drive_time = self.data.time[idx(node, after_node)];
            latest_feasible_departure = latest_feasible_departure - drive_time;
            self.latest_feasible_departure[node] = latest_feasible_departure;
            latest_feasible_departure =
                std::cmp::min(pts[after_node].due, latest_feasible_departure);
            after_node = node;
            node = self.prev[node];
        }

        let mut prev = node;
        node = after_node;

        let mut ev = Eval::new();

        while prev != recalc_last {
            ev.next(node, self.data);
            self.evals[node].reset_to(&ev);
            prev = node;
            node = self.next[node];
        }
    }

    pub fn add_route(&mut self, route: &Vec<usize>) {
        for (&bef, &aft) in route.iter().tuple_windows() {
            self.prev[aft] = bef;
            self.next[bef] = aft;
        }
        self.prev[0] = UNSERVED;
        self.next[0] = UNSERVED;

        let penultimate = route[route.len() - 2];
        self.next[penultimate] = 0;
        self.recalc_route(penultimate);

        debug_assert!({
            let mut e = Eval::new();
            route.iter().fold(true, |acc, &n| {
                e.next(n, self.data);
                acc && e.time <= self.latest_feasible_departure[n]
            })
        });
    }

    pub fn is_removed(&self, point_idx: usize) -> bool {
        self.next[point_idx] == UNSERVED
    }

    // pub fn increase_removed_count(&mut self, point_idx: usize) {
    //     debug_assert!(!self.is_removed(point_idx));

    //     self.removed_times[point_idx] += 1;
    //     let removed_times = self.removed_times[point_idx];
    //     let mut r = self.heap_size;
    //     self.removed_idx[point_idx] = r;
    //     self.heap_size += 1;

    //     while r > 0 && (self.removed_times[self.removed_heap[(r - 1) / 2]] < removed_times) {
    //         let parent = (r - 1) / 2;
    //         std::mem::swap(&mut self.removed_heap[parent], &mut self.removed_heap[r]);
    //         r = parent;
    //     }
    // }

    pub fn recalc_last(&self, pickup_idx: usize, delivery_idx: usize) -> usize {
        if self.next[delivery_idx] != 0 {
            // 0-...-delivery-*-...-0
            self.next[delivery_idx]
        } else if self.prev[delivery_idx] != pickup_idx {
            // 0-...-pickup-...-*-delivery-0
            self.prev[delivery_idx]
        } else {
            // 0-...-pickup-delivery-0
            self.prev[pickup_idx]
        }
    }

    pub fn remove_pair(&mut self, pickup_idx: usize) {
        debug_assert!(pickup_idx != 0);
        debug_assert!(pickup_idx < PTS);

        let delivery_idx = self.data.pair_of(pickup_idx);

        let recalc_last = self.recalc_last(pickup_idx, delivery_idx);
        self.unlink_unsafe(pickup_idx);
        self.unlink_unsafe(delivery_idx);

        self.recalc_route(recalc_last);
    }

    fn unlink_unsafe(&mut self, point_idx: usize) {
        let before = self.prev[point_idx];
        let after = self.next[point_idx];

        self.next[point_idx] = UNSERVED; // required for is_removed to work
        self.prev[point_idx] = UNSERVED; // for nice symmetry with the above

        self.next[before] = after;
        self.prev[after] = before;

        // in case before or after is the depot
        self.next[0] = UNSERVED;
        self.prev[0] = UNSERVED;
    }

    pub fn add_pair(&mut self, pickup_idx: usize, mov: &Move) {
        let delivery_idx = self.data.pair_of(pickup_idx);

        self.link_unsafe(pickup_idx, &mov.put_pickup_between);
        self.link_unsafe(delivery_idx, &mov.put_delivery_between);

        let recalc_from = self.recalc_last(pickup_idx, delivery_idx);
        self.recalc_route(recalc_from);
    }

    fn link_unsafe(&mut self, point_idx: usize, &Between(before, after): &Between) {
        self.next[before] = point_idx;
        self.next[point_idx] = after;

        self.prev[after] = point_idx;
        self.prev[point_idx] = before;

        // in case before or after is the depot
        self.next[0] = UNSERVED;
        self.prev[0] = UNSERVED;
    }
}

#[cfg(test)]
mod tests {
    // use super::*;
}
