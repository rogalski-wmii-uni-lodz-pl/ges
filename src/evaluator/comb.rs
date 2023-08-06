// use itertools::Itertools;

use itertools::Itertools;

use crate::data::PTS;
use crate::{K_MAX, UNSERVED};

pub struct Comb {
    pub route: [usize; PTS],
    pub len: usize,
    pub iters: [usize; K_MAX],
}

impl Comb {
    pub fn new() -> Self {
        Self {
            route: [UNSERVED; PTS],
            len: 0,
            iters: [0; K_MAX],
        }
    }

    pub fn reset(&mut self, v: &Vec<usize>) {
        self.len = v.len();
        self.route[..v.len()].copy_from_slice(v);
        for i in 0..K_MAX {
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

pub struct CombIter<'a> {
    comb: &'a Comb,
    cur: usize,
}

impl<'a> CombIter<'a> {
    pub fn new(comb: &'a Comb) -> Self {
        Self { comb, cur: 0 }
    }
}

impl<'a> Iterator for CombIter<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        let res = (self.cur < self.comb.len).then_some(self.comb.route[self.cur]);
        self.cur += 1;

        res
    }
}

impl<'a> IntoIterator for &'a Comb {
    type Item = usize;
    type IntoIter = CombIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter::new(self)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn simple() {
        let mut c = Comb::new();

        assert_eq!(c.len, 0);

        c.reset(&vec![0, 1, 2, 3, 4, 0]);

        assert_eq!(c.len, 6);
        assert_eq!(c.iters, [1, 2]);
        assert_eq!(c.r(), vec![0, 3, 4, 0]);

        assert_eq!(c.into_iter().collect_vec(), vec![0, 1, 2, 3, 4, 0]);
    }
}
