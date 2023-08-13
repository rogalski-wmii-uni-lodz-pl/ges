use data::Data;
use evaluator::Evaluator;
use sol::Sol;
use stats::Stats;

pub mod data;
pub mod eval;
pub mod evaluator;
pub mod mov;
pub mod sol;
pub mod stats;
pub mod routes;

const UNSERVED: usize = usize::MAX;
pub const K_MAX: usize = 10;
pub const TOTAL_TIME: u64 = 600;

pub struct Ges<'a> {
    evaluator: Evaluator<'a>,
    stats: Stats,
}

impl<'a> Ges<'a> {
    pub fn new(data: &'a Data) -> Self {
        Self {
            evaluator: Evaluator::new(&data),
            stats: Stats::new(),
        }
    }

    pub fn ges(&mut self, solution: &mut Sol, target_routes: usize) {
        loop {
            let routes = solution.routes_number();

            let total = self.stats.total_time().as_secs() ;
            if routes <= target_routes || total >= TOTAL_TIME {
                let rs = if solution.heap.size > 0 {
                    1
                } else {
                    0
                };
                print!("routes: {} ", routes + rs);
                self.stats.print_after_route_removal();
                break;
            }

            let random_route_first = solution.random_route_first();
            solution.remove_route(random_route_first);

            self.stats.reset();
            self.stats.add_iteration(solution.heap.size);
            while let Some(top) = solution.top() {

                let total = self.stats.total_time().as_secs() ;
                if total >= TOTAL_TIME {
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
                self.stats.print_occasionally(solution);
            }

            self.stats.print_after_route_removal();
        }
    }
}

#[cfg(test)]
mod tests {
    // use super::*;
}
