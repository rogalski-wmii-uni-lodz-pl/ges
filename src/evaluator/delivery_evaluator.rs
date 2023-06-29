use crate::{Sol, UNSERVED};
use crate::mov::{Between, Move};
use crate::eval::Eval;
use crate::data::Data;

use super::pickup_evaluator::PickupInsertionEvaluator;

pub struct DeliveryInsertionEvaluator<'a> {
    sol: &'a Sol,
    data: &'a Data,
    idx: usize,
    evaluator: Eval,
    put_pickup_between: Between,
    after_delivery_id: usize,
    last_in_route: usize,
}

impl<'a> DeliveryInsertionEvaluator<'a> {
    pub fn new(sol: &'a Sol, data: &'a Data) -> Self {
        Self {
            sol,
            data,
            idx: UNSERVED,
            evaluator: Eval::new(),
            put_pickup_between: Between(UNSERVED, UNSERVED),
            after_delivery_id: UNSERVED,
            last_in_route: UNSERVED,
        }
    }

    pub fn reset(&mut self, idx: usize, pickup_evaluator: &PickupInsertionEvaluator) {
        self.idx = idx;
        self.evaluator.reset_to(pickup_evaluator.evaluator());
        self.put_pickup_between = pickup_evaluator.get_between();
        self.after_delivery_id = pickup_evaluator.after_pickup();
        self.last_in_route = self.after_delivery_id;
    }

    pub fn can_continue(&self) -> bool {
        self.last_in_route != 0
    }

    pub fn check_next_node(&mut self, mov: &mut Move) {
        let before_delivery_id = self.sol.prev[self.after_delivery_id];

        if self.can_insert_delivery() {
            let put_delivery_between = Between(before_delivery_id, self.after_delivery_id);
            mov.maybe_switch(&self.put_pickup_between, &put_delivery_between);
        }

        self.advance_to_next_node();
    }

    pub fn can_insert_delivery(&mut self) -> bool {
        self.evaluator.check_insert(
            self.idx,
            self.after_delivery_id,
            self.data,
            self.sol.latest_feasible_departure[self.after_delivery_id],
        )
    }

    pub fn advance_to_next_node(&mut self) {
        self.evaluator.next(self.after_delivery_id, self.data);
        self.last_in_route = self.after_delivery_id;
        self.after_delivery_id = self.sol.next[self.after_delivery_id];
    }

    pub fn check_rest_of_route(&mut self, mov: &mut Move) {
        while self.can_continue() {
            self.check_next_node(mov);
        }
    }
}

