use crate::{Sol, UNSERVED};
use crate::mov::Between;
use crate::eval::Eval;
use crate::data::Data;


pub struct PickupInsertionEvaluator<'a> {
    sol: &'a Sol,
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
        self.evaluator.check(self.data)
    }

    pub fn advance(&mut self) {
        self.before_pickup = self.after_pickup;
        self.after_pickup = self.sol.next[self.after_pickup];
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

