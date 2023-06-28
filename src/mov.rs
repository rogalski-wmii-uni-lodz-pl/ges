use crate::UNSERVED;

#[derive(Debug, Copy, Clone)]
pub struct Between(pub usize, pub usize);

#[derive(Debug)]
pub struct Move {
    pub put_pickup_between: Between,
    pub put_delivery_between: Between,
    pub times: usize,
}

impl Move {
    pub fn new() -> Self {
        let unassigned = Between(UNSERVED, UNSERVED);
        Self {
            put_pickup_between: unassigned,
            put_delivery_between: unassigned,
            times: 0,
        }
    }

    pub fn maybe_switch(&mut self, put_pickup_between: &Between, put_delivery_between: &Between) {
        self.times += 1;
        if rand::random::<usize>() % self.times == 0 {
            self.put_pickup_between = *put_pickup_between;
            self.put_delivery_between = *put_delivery_between;
        }
    }

    pub fn empty(&self) -> bool {
        return self.times == 0;
    }

    // pub fn pick(self, other: &Self) -> Self {
    //     let times = self.times + other.times;
    //     let which = if rand::random::<usize>() % times < self.times {
    //         &self
    //     } else {
    //         other
    //     };
    //     Self { times, ..(*which) }
    // }
}
