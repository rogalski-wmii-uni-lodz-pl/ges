use rand::Rng;

use crate::{K_MAX, UNSERVED};

#[derive(Debug, Copy, Clone)]
pub struct Between(pub usize, pub usize);

#[derive(Debug)]
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
        let r = rand::thread_rng().gen_range(0..self.times);
        if r == 0 {
            self.put_pickup_between = *put_pickup_between;
            self.put_delivery_between = *put_delivery_between;
        }
    }

    pub fn empty(&self) -> bool {
        return self.times == 0;
    }

    pub fn pick(&mut self, other: &Self) {
        self.times += other.times;
        let r = rand::thread_rng().gen_range(0..self.times);
        if r < other.times {
            self.put_pickup_between = other.put_pickup_between;
            self.put_delivery_between = other.put_delivery_between;
            self.removed = other.removed;
        }
    }
}
