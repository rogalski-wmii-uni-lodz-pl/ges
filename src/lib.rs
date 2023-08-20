use data::Data;
use evaluator::Evaluator;
use sol::Sol;
use stats::Stats;
use std::time::Duration;

pub mod data;
pub mod eval;
pub mod evaluator;
pub mod mov;
pub mod routes;
pub mod sol;
pub mod stats;

const UNSERVED: usize = usize::MAX;
pub const K_MAX: usize = 10;
// pub const TOTAL_TIME: u64 = 600;

pub struct Ges<'a> {
    evaluator: Evaluator<'a>,
    stats: Stats,
}

pub enum Log {
    Quiet,
    Verbose,
    Extra,
}

pub struct Conf {
    pub max_optimization_time: Duration,
    pub target_routes: usize,
    pub log: Log,
}

impl<'a> Ges<'a> {
    pub fn new(data: &'a Data) -> Self {
        Self {
            evaluator: Evaluator::new(&data),
            stats: Stats::new(),
        }
    }

    pub fn ges(&mut self, solution: &mut Sol, conf: Conf) {
        loop {
            let routes = solution.routes_number();

            let total = self.stats.total_time();

            if routes <= conf.target_routes || total >= conf.max_optimization_time {
                self.stats.print_after_route_removal(solution);
                if !matches!(conf.log, Log::Quiet) {
                    solution.eprn();
                }

                if let Log::Extra = conf.log {
                    for i in 0..solution.data.points {
                        println!("{} {}", i, solution.heap.removed_times[i]);
                    }
                }

                break;
            }

            let random_route_first = solution.random_route_first();
            solution.remove_route(random_route_first);

            self.stats.reset();
            self.stats.add_iteration(solution.heap.size);
            while let Some(top) = solution.top() {
                let total = self.stats.total_time();
                if total >= conf.max_optimization_time {
                    break;
                }
                let maybe = solution.try_insert(top, &mut self.evaluator);

                if let Some(mov) = maybe {
                    solution.pop();
                    solution.make_move(&mov);
                    debug_assert!(solution.check_routes());
                } else {
                    solution.inc();
                    for _ in 0..50 {
                        solution.perturb(&mut self.evaluator);
                    }
                }

                self.stats.add_iteration(solution.heap.size);
                if !matches!(conf.log, Log::Quiet) {
                    self.stats.print_occasionally(solution);
                }
            }

            if !matches!(conf.log, Log::Quiet) {
                self.stats.print_after_route_removal(solution);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    // use super::*;
}
