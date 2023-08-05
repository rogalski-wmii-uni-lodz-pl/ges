// use itertools::Itertools;

use itertools::Itertools;

use crate::{data::PTS, UNSERVED};

pub struct Comb<const K: usize> {
    pub route: [usize; PTS],
    pub len: usize,
    pub iters: [usize; K],
}

impl<const K: usize> Comb<K> {
    pub fn new() -> Self {
        Self {
            route: [UNSERVED; PTS],
            len: 0,
            iters: [0; K],
        }
    }

    pub fn reset(&mut self, v: &Vec<usize>) {
        self.len = v.len();
        self.route[..v.len()].copy_from_slice(v);
        for i in 0..K {
            self.iters[i] = i + 1
        }
    }

    pub fn r(&self) -> Vec<usize> {
        // not extremely efficient, but not meant to be
        self.route[0..self.len]
            .iter()
            .enumerate()
            .filter_map(|(i, x)| (!self.iters.contains(&i)).then_some(*x))
            .collect_vec()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn simple() {
        let mut c: Comb<3> = Comb::new();

        assert_eq!(c.len, 0);

        c.reset(&vec![0, 1, 2, 3, 4, 0]);

        assert_eq!(c.len, 6);
        assert_eq!(c.iters, [1, 2, 3]);
        assert_eq!(c.r(), vec![0, 4, 0]);
    }
}
