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

const UNSERVED: usize = usize::MAX;
const K_MAX: usize = 10;

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

    pub fn ges(&mut self, solution: &mut Sol) {
        loop {
            println!("routes: {}", solution.routes_number());
            solution.eprn();

            let random_route_first = solution.random_route_first();
            solution.remove_route(random_route_first);

            self.stats.reset();
            self.stats.add_iteration(solution.heap.size);
            while let Some(top) = solution.top() {
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

            println!("after {}", self.stats.iterations().current());
        }
    }
}

#[cfg(test)]
mod tests {
    // use super::*;
}
