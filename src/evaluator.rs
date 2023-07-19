use crate::data::{Data, PTS};
use crate::mov::Move;
use crate::{Sol, K_MAX, UNSERVED};

use self::{
    delivery_evaluator::DeliveryInsertionEvaluator, pickup_evaluator::PickupInsertionEvaluator,
};

pub mod delivery_evaluator;
pub mod pickup_evaluator;

pub struct Evaluator<'a> {
    sol: &'a Sol<'a>,
    data: &'a Data,
    pickup_idx: usize,
    delivery_idx: usize,
    pickup_evaluator: PickupInsertionEvaluator<'a>,
    delivery_evaluator: DeliveryInsertionEvaluator<'a>,
    removed: [usize; 2 * K_MAX],
    jump_forward: [i32; PTS],
    jump_backward: [i32; PTS],
}

impl<'a> Evaluator<'a> {
    pub fn new(sol: &'a Sol, data: &'a Data) -> Self {
        Self {
            sol,
            data,
            pickup_idx: UNSERVED,
            delivery_idx: UNSERVED,
            pickup_evaluator: PickupInsertionEvaluator::new(data),
            delivery_evaluator: DeliveryInsertionEvaluator::new(data),
            removed: [0; 2 * K_MAX],
            jump_forward: [0; PTS],
            jump_backward: [0; PTS],
        }
    }

    pub fn with_pickup(sol: &'a Sol, data: &'a Data, pickup_idx: usize) -> Self {
        let mut evaluator = Self::new(sol, data);
        evaluator.reset(pickup_idx);

        evaluator
    }

    pub fn reset(&mut self, pickup_idx: usize) {
        self.pickup_idx = pickup_idx;
        self.delivery_idx = self.data.pair_of(pickup_idx);
        self.removed.iter().for_each(|&x| {
            self.jump_forward[x] = 0;
            self.jump_backward[x] = 0;
        });
        self.removed = [0; 2 * K_MAX];
    }

    pub fn remove(&mut self, idx: usize) {
        self.jump_forward[idx] = self.sol.next[idx] as i32 - idx as i32
    }

    pub fn remove_pair(&mut self, pickup_idx: usize) {
        let delivery_idx = self.data.pair_of(pickup_idx);
        self.remove(delivery_idx);
        self.remove(pickup_idx);
    }

    pub fn check_add_to_route(&mut self, start: usize) -> Option<Move> {
        let mut mov = Move::new();
        self.pickup_evaluator
            .reset(&self.sol, self.pickup_idx, start);

        while self.pickup_evaluator.can_continue() {
            self.pickup_evaluator.insert_pickup(&self.sol);

            if self.pickup_evaluator.pickup_insertion_is_feasible() {
                self.check_delivery_insertions(&mut mov);
            }

            self.pickup_evaluator.advance(&self.sol, &self.jump_forward);
        }

        mov_into_option(mov)
    }

    fn check_delivery_insertions(&mut self, mov: &mut Move) {
        self.delivery_evaluator
            .reset(self.delivery_idx, &self.pickup_evaluator);

        self.delivery_evaluator
            .check_rest_of_route(self.sol, &self.jump_forward, mov);
    }
}

fn mov_into_option(mov: Move) -> Option<Move> {
    if mov.empty() {
        None
    } else {
        Some(mov)
    }
}
