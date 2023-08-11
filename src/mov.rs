use rand::Rng;

use crate::{K_MAX, UNSERVED};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Between(pub usize, pub usize);

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Move {
    pub pickup: usize,
    pub put_pickup_between: Between,
    pub put_delivery_between: Between,
    pub times: usize,
    pub removed: [usize; K_MAX],
}

impl Move {
    pub fn new(pickup: usize) -> Self {
        let unassigned = Between(UNSERVED, UNSERVED);
        Self {
            pickup,
            put_pickup_between: unassigned,
            put_delivery_between: unassigned,
            times: 0,
            removed: [0; K_MAX],
        }
    }

    pub fn maybe_switch(&mut self, put_pickup_between: &Between, put_delivery_between: &Between) {
        self.times += 1;
        let r = rand::thread_rng().gen_range(1..=self.times);
        if r == 1 {
            self.put_pickup_between = *put_pickup_between;
            self.put_delivery_between = *put_delivery_between;
        }
    }

    pub fn pick(&mut self, other: &Self) {
        self.times += other.times;
        let r = rand::thread_rng().gen_range(1..=self.times);
        if r <= other.times {
            self.put_pickup_between = other.put_pickup_between;
            self.put_delivery_between = other.put_delivery_between;
            self.removed = other.removed;
        }
    }

    pub fn pick2(self, other: Self) -> Self {
        let times = self.times + other.times;
        let r = rand::thread_rng().gen_range(1..=times);
        Self {
            times,
            ..(if r <= other.times { other } else { self })
        }
    }

    pub fn is_not_empty(&self) -> bool {
        self.times != 0
    }
}

pub struct Swap {
    pub a: Move,
    pub b: Move,
}

impl Swap {
    pub fn new(a_pickup: usize, b_pickup: usize) -> Self {
        Self {
            a: Move::new(a_pickup),
            b: Move::new(b_pickup),
        }
    }

    pub fn is_not_empty(&self) -> bool {
        self.a.is_not_empty()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn empty() {
        let m = Move::new(1);

        assert!(!m.is_not_empty());

        let ne = Move {
            pickup: 1,
            times: 1,
            put_pickup_between: Between(1, 2),
            put_delivery_between: Between(1, 2),
            removed: [0; K_MAX],
        };

        assert!(ne.is_not_empty());
    }

    #[test]
    fn empty_switching() {
        let other = Move {
            pickup: 1,
            times: 1,
            put_pickup_between: Between(3, 4),
            put_delivery_between: Between(5, 6),
            removed: [0; K_MAX],
        };

        let mut m = Move::new(1);
        m.maybe_switch(&other.put_pickup_between, &other.put_delivery_between);
        assert_eq!(m, other);

        let mut m = Move::new(1);
        m.pick(&other);
        assert_eq!(m, other);

        let m = Move::new(1).pick2(other);
        assert_eq!(m, other);
    }

    #[test]
    fn non_empty_switching() {
        let mut a = Move {
            pickup: 1,
            times: 5,
            put_pickup_between: Between(3, 4),
            put_delivery_between: Between(5, 6),
            removed: [1; K_MAX],
        };

        let b = Move {
            pickup: 1,
            times: 10,
            put_pickup_between: Between(5, 6),
            put_delivery_between: Between(7, 8),
            removed: [2; K_MAX],
        };

        let mut a_res = a.clone();
        a_res.times = 15;

        let mut b_res = b.clone();
        b_res.times = 15;

        let m = a.clone().pick2(b);
        assert!(m == a_res || m == b_res);

        a.pick(&b);
        assert!(a == a_res || a == b_res);
    }
}
