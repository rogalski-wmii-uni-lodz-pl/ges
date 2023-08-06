use rand::Rng;

use crate::{K_MAX, UNSERVED};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Between(pub usize, pub usize);

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Move {
    pub put_pickup_between: Between,
    pub put_delivery_between: Between,
    pub times: usize,
    pub removed: [usize; K_MAX],
}

impl Move {
    pub fn new() -> Self {
        let unassigned = Between(UNSERVED, UNSERVED);
        Self {
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

    pub fn empty(&self) -> bool {
        return self.times == 0;
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
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn empty() {
        let m = Move::new();

        assert!(m.empty());

        let ne = Move {
            times: 1,
            put_pickup_between: Between(1, 2),
            put_delivery_between: Between(1, 2),
            removed: [0; K_MAX],
        };

        assert!(!ne.empty());
    }

    #[test]
    fn empty_switching() {
        let other = Move {
            times: 1,
            put_pickup_between: Between(3, 4),
            put_delivery_between: Between(5, 6),
            removed: [0; K_MAX],
        };

        let mut m = Move::new();
        m.maybe_switch(&other.put_pickup_between, &other.put_delivery_between);
        assert_eq!(m, other);

        let mut m = Move::new();
        m.pick(&other);
        assert_eq!(m, other);

        let m = Move::new().pick2(other);
        assert_eq!(m, other);
    }


    #[test]
    fn non_empty_switching() {
        let mut a = Move {
            times: 5,
            put_pickup_between: Between(3, 4),
            put_delivery_between: Between(5, 6),
            removed: [1; K_MAX],
        };

        let b = Move {
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
