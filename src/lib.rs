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
const K_MAX: usize = 10;

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

    pub fn ges(&mut self, s: &mut Sol) {
    for _ in 0.. {
        println!("routes: {}", s.routes.iter().count());
        let r = *s
            .routes
            .iter()
            .sorted()
            .choose(&mut rand::thread_rng())
            .unwrap();
        // let v = s.routes.iter().collect_vec();
        println!("{r:?}");
        s.eprn();
        s.remove_route(r);
        while let Some(top) = s.top() {
            let maybe = s.try_insert(top, &mut self.ev);

            if let Some(mov) = maybe {
                s.pop();
                s.make_move(top, &mov);
                debug_assert!(s.check_routes());
            } else {
                s.inc();
                for _ in 0..50 {
                    s.perturb(&mut self.ev);
                }
                s.prn_heap();
            }
        }
    }
    }
}

#[cfg(test)]
mod tests {
    // use super::*;
}
