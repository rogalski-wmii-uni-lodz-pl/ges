use std::time::Instant;

use data::Data;
use evaluator::Evaluator;
use itertools::Itertools;
use rand::seq::IteratorRandom;
use sol::Sol;

pub mod data;
pub mod eval;
pub mod evaluator;
pub mod mov;
pub mod sol;

const UNSERVED: usize = usize::MAX;
const K_MAX: usize = 3;

// pub struct Removed {
//     pub pickup: usize,
//     pub delivery: usize,
//     pub times: usize,
// }
//

pub struct Ges<'a> {
    ev: Evaluator<'a>,
}

impl<'a> Ges<'a> {
    pub fn new(data: &'a Data) -> Self {
        Self {
            ev: Evaluator::new(&data),
        }
    }

    pub fn ges(&mut self, solution: &mut Sol) {
        let mut total = 0;
        let start = Instant::now();
        loop {
            println!("routes: {}", solution.routes.iter().count());
            let random_route_first = *solution
                .routes
                .iter()
                .sorted()
                .choose(&mut rand::thread_rng())
                .unwrap();
            // let v = s.routes.iter().collect_vec();
            println!("{random_route_first:?}");
            solution.eprn();
            solution.remove_route(random_route_first);

            let mut times: u64 = 0;
            let mut max = solution.heap_size;
            let mut min = solution.heap_size;
            while let Some(top) = solution.top() {
                times += 1;
                total += 1;
                let maybe = solution.try_insert(top, &mut self.ev);

                if let Some(mov) = maybe {
                    max = solution.heap_size.max(max);
                    min = solution.heap_size.min(min);
                    solution.pop();
                    solution.make_move(&mov);
                    debug_assert!(solution.check_routes());
                } else {
                    solution.inc();
                    for _ in 0..50 {
                        solution.perturb(&mut self.ev);
                    }
                }

                if times % 10000 == 0 {
                    solution.eprn();
                    print!("{times} {min} {max}: ");
                    solution.prn_heap();
                }

                if total % 10000 == 0 {
                    let elapsed = start.elapsed();
                    println!("total {total} after {elapsed:?}");
                }
            }

            println!("after {times}");
        }
    }
}

#[cfg(test)]
mod tests {
    // use super::*;
}
