use crate::data::{Data, PTS};
use crate::eval::Eval;
use crate::mov::Between;
use crate::{Sol, UNSERVED};

pub struct PickupInsertionEvaluator<'a> {
    sol: &'a Sol<'a>,
    data: &'a Data,
    after_pickup: usize,
    before_pickup: usize,
    idx: usize,
    evaluator: Eval,
}

impl<'a> PickupInsertionEvaluator<'a> {
    pub fn new(sol: &'a Sol, data: &'a Data) -> Self {
        Self {
            sol,
            data,
            idx: UNSERVED,
            after_pickup: UNSERVED,
            before_pickup: UNSERVED,
            evaluator: Eval::new(),
        }
    }

    pub fn reset(&mut self, idx: usize, start: usize) {
        self.idx = idx;
        self.after_pickup = start;
        self.before_pickup = self.sol.prev[self.after_pickup];
        self.evaluator.reset_to(&self.sol.evals[self.before_pickup]);
    }

    pub fn can_continue(&self) -> bool {
        self.after_pickup != 0
    }

    pub fn pickup_insertion_is_feasible(&self) -> bool {
        self.evaluator.is_feasible(self.data)
    }

    pub fn advance(&mut self, jump_forward: &[i32; PTS]) {
        self.before_pickup = self.after_pickup;
        self.after_pickup =
            (self.sol.next[self.after_pickup] as i32 + jump_forward[self.after_pickup]) as usize;
    }

    pub fn insert_pickup(&mut self) {
        self.evaluator.reset_to(&self.sol.evals[self.before_pickup]);
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
}
