use crate::data::{Data, PTS};
use crate::eval::Eval;
use crate::mov::Between;
use crate::{sol::Sol, UNSERVED};

pub struct PickupInsertionEvaluator<'a> {
    data: &'a Data,
    after_pickup: usize,
    before_pickup: usize,
    idx: usize,
    evaluator: Eval,
}

impl<'a> PickupInsertionEvaluator<'a> {
    pub fn new(data: &'a Data) -> Self {
        Self {
            data,
            idx: UNSERVED,
            after_pickup: UNSERVED,
            before_pickup: UNSERVED,
            evaluator: Eval::new(),
        }
    }

    pub fn reset(&mut self, sol: &Sol, before_idx: usize, idx: usize, start: usize) {
        self.idx = idx;
        self.after_pickup = start;
        self.before_pickup = before_idx;
        self.evaluator.reset_to(&sol.evals[self.before_pickup]);
    }

    pub fn can_continue(&self) -> bool {
        self.after_pickup != self.before_pickup // both after_pickup and before_pickup is 0
    }

    pub fn pickup_insertion_is_feasible(&self) -> bool {
        self.evaluator.is_feasible(self.data)
    }

    pub fn advance(&mut self, sol: &Sol, jump_forward: &[usize; PTS]) {
        self.before_pickup = self.after_pickup;
        self.after_pickup = sol.next[self.after_pickup];
        while self.after_pickup != jump_forward[self.after_pickup] {
            self.after_pickup = jump_forward[self.after_pickup];
        }
    }

    pub fn insert_pickup(&mut self, sol: &Sol) {
        self.evaluator.reset_to(&sol.evals[self.before_pickup]);
        self.evaluator.next(self.idx, self.data);
    }

    pub fn get_between(&self) -> Between {
        Between(self.before_pickup, self.after_pickup)
    }

    pub fn evaluator(&self) -> &Eval {
        &self.evaluator
    }

    pub fn after_pickup(&self) -> usize {
        self.after_pickup
    }

    pub fn idx(&self) -> usize {
        self.idx
    }
}
