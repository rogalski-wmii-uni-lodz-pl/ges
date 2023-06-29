use data::{idx, Data, PTS};
use eval::Eval;
use itertools::Itertools;

pub mod evaluator;
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


#[cfg(test)]
mod tests {
    // use super::*;
}
