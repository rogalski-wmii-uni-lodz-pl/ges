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

    fn fix_evals(&mut self, first: usize) {
        let mut ev = Eval::new();
        let prev = self.prev[first];
        ev.reset_to(&self.evals[prev]);

        let mut node = first;
        while node != 0 {
            ev.next(node, self.data);
            self.evals[node].reset_to(&ev);
            node = self.next[node];
        }
    }

    fn fix_latest_feasible_departures(&mut self, last: usize) {
        let mut node = last;
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
    }

    pub fn add_route(&mut self, route: &Vec<usize>) {
        debug_assert!(route[0] == 0 && *route.last().unwrap() == 0);
        // we get the second one and the penultimate one because first and last is 0
        let first_non_depot = route[1];
        let last_non_depot = route[route.len() - 2];
        for (&before, &after) in route.iter().tuple_windows() {
            self.prev[after] = before;
            self.next[before] = after;
        }
        self.prev[0] = UNSERVED;
        self.next[0] = UNSERVED;

        self.fix_latest_feasible_departures(last_non_depot);
        self.fix_evals(first_non_depot);

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

    fn get_nodes_to_recalculate_from_after_removal(
        &self,
        pickup_idx: usize,
        delivery_idx: usize,
    ) -> (usize, usize) {
        let mut earliest = self.prev[pickup_idx];
        let mut latest = self.next[delivery_idx];

        if earliest == 0 {
            earliest = self.next[pickup_idx];
        }

        if earliest == delivery_idx {
            earliest = latest;
        }

        if latest == 0 {
            latest = self.prev[delivery_idx];
        }

        if latest == pickup_idx {
            latest = earliest;
        }

        (earliest, latest)
    }

    pub fn remove_pair(&mut self, pickup_idx: usize) {
        debug_assert!(pickup_idx != 0);
        debug_assert!(pickup_idx < PTS);

        let delivery_idx = self.data.pair_of(pickup_idx);

        let (before, after) =
            self.get_nodes_to_recalculate_from_after_removal(pickup_idx, delivery_idx);

        self.unlink_unsafe(pickup_idx);
        self.unlink_unsafe(delivery_idx);

        if before != 0 {
            self.fix_evals(before);
        }

        if after != 0 {
            self.fix_latest_feasible_departures(after);
        }
    }

    fn unlink_unsafe(&mut self, point_idx: usize) {
        let before = self.prev[point_idx];
        let after = self.next[point_idx];

        // needs to be in for is_removed to work
        self.next[point_idx] = UNSERVED;
        self.prev[point_idx] = UNSERVED;

        self.next[before] = after;
        self.prev[after] = before;

        // in case before or after is the depot
        self.next[0] = UNSERVED;
        self.prev[0] = UNSERVED;
    }

    fn get_nodes_to_recalculate_from_after_insertion(
        &self,
        pickup_idx: usize,
        delivery_idx: usize,
    ) -> (usize, usize) {
        let mut earliest = self.prev[pickup_idx];
        let mut latest = self.next[delivery_idx];

        if earliest == 0 {
            earliest = pickup_idx;
        }

        if latest == 0 {
            latest = delivery_idx;
        }

        (earliest, latest)
    }

    pub fn add_pair(&mut self, pickup_idx: usize, mov: &Move) {
        let delivery_idx = self.data.pair_of(pickup_idx);

        self.link_unsafe(pickup_idx, &mov.put_pickup_between);
        self.link_unsafe(delivery_idx, &mov.put_delivery_between);

        let (before, after) =
            self.get_nodes_to_recalculate_from_after_insertion(pickup_idx, delivery_idx);

        self.fix_evals(before);
        self.fix_latest_feasible_departures(after);
    }

    fn link_unsafe(&mut self, point_idx: usize, &Between(before, after): &Between) {
        debug_assert!(before != 0 && after != 0);
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
