use crate::{UNSERVED, K_MAX, Sol};
use crate::data::Data;
use crate::mov::Move;

use self::{pickup_evaluator::PickupInsertionEvaluator, delivery_evaluator::DeliveryInsertionEvaluator};

pub mod pickup_evaluator;
pub mod delivery_evaluator;

pub struct Evaluator<'a> {
    // sol: &'a Sol,
    data: &'a Data,
    pickup_id: usize,
    delivery_id: usize,
    pickup_evaluator: PickupInsertionEvaluator<'a>,
    delivery_evaluator: DeliveryInsertionEvaluator<'a>,
    removed: [Option<usize>; K_MAX],
}

impl<'a> Evaluator<'a> {
    pub fn new(sol: &'a Sol, data: &'a Data) -> Self {
        Self {
            // sol,
            data,
            pickup_id: UNSERVED,
            delivery_id: UNSERVED,
            pickup_evaluator: PickupInsertionEvaluator::new(sol, data),
            delivery_evaluator: DeliveryInsertionEvaluator::new(sol, data),
            removed: [None; K_MAX],
        }
    }

    pub fn with_pickup(sol: &'a Sol, data: &'a Data, pickup_id: usize) -> Self {
        let mut evaluator = Self::new(sol, data);
        evaluator.reset(pickup_id);

        evaluator
    }

    pub fn reset(&mut self, pickup_id: usize) {
        self.pickup_id = pickup_id;
        self.delivery_id = self.data.pair_of(pickup_id);
        self.removed = [None; K_MAX];
    }

    pub fn check_add_to_route(&mut self, start: usize) -> Option<Move> {
        let mut mov = Move::new();
        self.pickup_evaluator.reset(self.pickup_id, start);

        while self.pickup_evaluator.can_continue() {
            self.pickup_evaluator.insert_pickup();

            if self.pickup_evaluator.pickup_insertion_is_feasible() {
                self.check_delivery_insertions(&mut mov);
            }

            self.pickup_evaluator.advance();
        }

        if mov.empty() {
            None
        } else {
            Some(mov)
        }
    }

    fn check_delivery_insertions(&mut self, mov: &mut Move) {
        self.delivery_evaluator
            .reset(self.delivery_id, &self.pickup_evaluator);

        self.delivery_evaluator.check_rest_of_route(mov);
    }
}
