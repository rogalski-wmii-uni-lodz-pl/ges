use std::{collections::HashSet, mem::replace};

use itertools::Itertools;
use rand::seq::IteratorRandom;
use rand::Rng;

use crate::data::{idx, Data, PTS};
use crate::eval::Eval;
use crate::evaluator::Evaluator;
use crate::mov::{Between, Move, Swap};
use crate::{K_MAX, UNSERVED};

use self::heap::Heap;

mod heap;

pub struct Sol<'a> {
    pub data: &'a Data,
    pub next: [usize; PTS],
    pub prev: [usize; PTS],
    pub latest_feasible_departure: [u64; PTS],
    pub evals: Vec<Eval>,
    pub first: [usize; PTS],
    pub routes: HashSet<usize>,
    pub heap: Heap,
    // pub removed_times: [u64; PTS],
    // pub removed_idx: [usize; PTS],
    // pub heap_size: usize,
}

impl<'a> Sol<'a> {
    pub fn new(data: &'a Data) -> Self {
        let unserved = [UNSERVED; PTS];
        let mut latest_feasible_departure = [0; PTS];
        latest_feasible_departure[0] = data.pts[0].due;
        let evals: Vec<_> = (0..PTS).map(|_| Eval::new()).collect();

        Sol {
            data,
            next: unserved.clone(),
            prev: unserved.clone(),
            latest_feasible_departure,
            heap: Heap::new(),
            evals,
            first: unserved.clone(),
            routes: HashSet::new(),
        }
    }

    pub fn initialize(&mut self) {
        for i in 1..self.data.points {
            let p = self.data.pts[i];
            if !p.is_delivery {
                let vec = vec![0, i, p.pair, 0];
                self.add_route(&vec)
            }
        }
    }

    pub fn random_route_first(&self) -> usize {
        *self
            .routes
            .iter()
            .sorted()
            .choose(&mut rand::thread_rng())
            .unwrap()
    }

    fn fix_evals(&mut self, first: usize) {
        let mut ev = Eval::new();
        let prev = self.prev[first];
        ev.reset_to(&self.evals[prev]);

        let mut node = first;
        while node != 0 {
            ev.next(node, self.data);
            self.evals[node].reset_to(&ev);
            node = self.next[node];
        }
    }

    fn fix_latest_feasible_departures(&mut self, last: usize) {
        let mut node = last;
        let pts = &self.data.pts;
        let mut after_node = self.next[node];
        let mut latest_feasible_departure = self.latest_feasible_departure[after_node];

        while node != 0 {
            let drive_time = self.data.time[idx(node, after_node)];
            latest_feasible_departure = latest_feasible_departure - drive_time;
            latest_feasible_departure = std::cmp::min(pts[node].due, latest_feasible_departure);
            self.latest_feasible_departure[node] = latest_feasible_departure;
            after_node = node;
            node = self.prev[node];
        }
    }

    pub fn add_route(&mut self, route: &Vec<usize>) {
        debug_assert!(route[0] == 0 && *route.last().unwrap() == 0);
        // we get the second one and the penultimate one because first and last is 0
        let first_non_depot = route[1];
        let last_non_depot = route[route.len() - 2];
        for (&before, &after) in route.iter().tuple_windows() {
            self.prev[after] = before;
            self.next[before] = after;
            self.first[before] = first_non_depot;
        }
        self.prev[0] = 0;
        self.next[0] = 0;

        self.fix_latest_feasible_departures(last_non_depot);
        self.fix_evals(first_non_depot);

        self.routes.insert(first_non_depot);

        debug_assert!({
            let mut e = Eval::new();
            route.iter().fold(true, |acc, &n| {
                e.next(n, self.data);
                acc && e.time <= self.latest_feasible_departure[n]
            })
        });
    }

    pub fn is_removed(&self, point_idx: usize) -> bool {
        self.next[point_idx] == UNSERVED
    }


    pub fn routes_number(&self) -> usize {
        self.routes.iter().count()
    }

    pub fn perturb(&mut self, ev: &mut Evaluator) {
        if rand::thread_rng().gen_bool(0.5) {
            self.random_move(ev);
        } else {
            self.random_swap(ev);
            // self.random_move(ev);
        }
    }

    fn random_swap(&mut self, ev: &mut Evaluator<'_>) {
        let mut a_pickup = self.random_pickup();
        let mut b_pickup = self.random_pickup();

        while b_pickup == a_pickup || self.in_same_route(a_pickup, b_pickup) {
            a_pickup = self.random_pickup();
            b_pickup = self.random_pickup();
        }

        let maybe = ev.check_swap(self, a_pickup, b_pickup);

        if let Some(swap) = maybe {
            self.make_swap(&swap);
        }
    }

    fn in_same_route(&mut self, a_pickup: usize, b_pickup: usize) -> bool {
        self.first[a_pickup] == self.first[b_pickup]
    }

    fn random_move(&mut self, ev: &mut Evaluator<'_>) {
        let pickup = self.random_pickup();

        self.remove_pair(pickup);

        let mov = self.try_insert_1(pickup, ev);
        debug_assert!(mov.is_some());
        // debug_assert!(mov.unwrap().removed == [0; K_MAX]);
        self.make_move(&mov.unwrap());
    }

    fn random_pickup(&mut self) -> usize {
        let mut idx = self.random_idx();

        while self.is_removed(idx)
            || self.data.pts[idx].is_delivery
            || self.only_pickup_in_route(idx)
        {
            idx = self.random_idx();
        }

        idx
    }

    fn random_idx(&mut self) -> usize {
        rand::thread_rng().gen_range(1..self.data.points)
    }

    pub fn try_insert_1(&self, pickup: usize, ev: &mut Evaluator) -> Option<Move> {
        ev.reset(pickup);

        self.routes
            .iter()
            .filter_map(|&route| ev.check_add_to_route(&self, route))
            .reduce(Move::pick2)
    }

    pub fn try_insert_k(&self, pickup: usize, ev: &mut Evaluator) -> Option<Move> {
        ev.reset(pickup);

        for k in 1..=K_MAX {
            let mov = self
                .routes
                .iter()
                .filter_map(|&route| ev.check_add_to_route_with_k_removed(&self, route, k))
                .reduce(Move::pick2);

            if mov.is_some() {
                return mov;
            }
        }

        None
    }

    pub fn try_insert(&self, pickup: usize, ev: &mut Evaluator) -> Option<Move> {
        ev.reset(pickup);

        let mov = self.try_insert_1(pickup, ev);

        mov.or_else(|| self.try_insert_k(pickup, ev))
    }

    pub fn only_pickup_in_route(&self, pickup: usize) -> bool {
        let delivery = self.data.pair_of(pickup);

        let empty_route = [0, pickup, delivery, 0];
        let route_fragment = [
            self.prev[pickup],
            self.prev[delivery],
            self.next[pickup],
            self.next[delivery],
        ];

        route_fragment == empty_route
    }


    pub fn remove_route(&mut self, first: usize) {
        debug_assert!(self.prev[first] == 0);
        self.routes.remove(&first);
        let mut idx = first;
        while idx != 0 {
            self.first[idx] = UNSERVED;
            if !self.data.pts[idx].is_delivery {
                self.heap.push(idx);
                self.heap.removed_times[idx] += 1;
            }
            self.prev[idx] = UNSERVED;
            idx = replace(&mut self.next[idx], UNSERVED);
        }

        self.heap.sort()
    }

    pub fn inc(&mut self) {
        self.heap.inc()
    }

    pub fn prn_heap(&self) {
        self.heap.prn();
    }

    pub fn push(&mut self, idx: usize) {
        self.heap.push(idx)
    }

    pub fn top(&mut self) -> Option<usize> {
        self.heap.top()
    }

    pub fn pop(&mut self) {
        self.heap.pop()
    }

    fn get_first_after_removal(&self, pickup_idx: usize, delivery_idx: usize) -> usize {
        let f = self.first[pickup_idx];
        let after_pickup = self.next[pickup_idx];

        if f != pickup_idx {
            f
        } else if after_pickup != delivery_idx {
            after_pickup
        } else {
            self.next[delivery_idx]
        }
    }

    fn fix_route(&mut self, first: usize) {
        debug_assert!(self.prev[first] == 0);
        let mut ev = Eval::new();
        let mut prev = self.prev[first];
        ev.reset_to(&self.evals[prev]);

        let mut node = first;
        while node != 0 {
            self.first[node] = first;
            ev.next(node, self.data);
            self.evals[node].reset_to(&ev);
            prev = node;
            node = self.next[node];
        }

        self.fix_latest_feasible_departures(prev);
    }

    pub fn remove_pair(&mut self, pickup_idx: usize) {
        self.routes.remove(&pickup_idx);
        debug_assert!(pickup_idx != 0);
        debug_assert!(pickup_idx < PTS);

        let delivery_idx = self.data.pair_of(pickup_idx);

        let first_after_removal = self.get_first_after_removal(pickup_idx, delivery_idx);
        self.routes.insert(first_after_removal);

        self.unlink_unsafe(pickup_idx);
        self.unlink_unsafe(delivery_idx);

        self.fix_route(first_after_removal);
    }

    fn unlink_unsafe(&mut self, point_idx: usize) {
        let before = self.prev[point_idx];
        let after = self.next[point_idx];
        debug_assert!(before != UNSERVED);
        debug_assert!(after != UNSERVED);

        // needs to be in for is_removed to work
        self.next[point_idx] = UNSERVED;
        self.prev[point_idx] = UNSERVED;

        self.next[before] = after;
        self.prev[after] = before;

        self.first[point_idx] = UNSERVED;

        // in case before or after is the depot
        self.next[0] = 0;
        self.prev[0] = 0;
    }

    pub fn make_swap(&mut self, swap: &Swap) {
        let a_pickup_idx = swap.a.pickup;
        debug_assert!(self.next[a_pickup_idx] != UNSERVED);
        debug_assert!(self.prev[a_pickup_idx] != UNSERVED);
        let a_delivery_idx = self.data.pair_of(a_pickup_idx);
        debug_assert!(self.next[a_delivery_idx] != UNSERVED);
        debug_assert!(self.prev[a_delivery_idx] != UNSERVED);

        let b_pickup_idx = swap.b.pickup;
        debug_assert!(self.next[b_pickup_idx] != UNSERVED);
        debug_assert!(self.prev[b_pickup_idx] != UNSERVED);
        let b_delivery_idx = self.data.pair_of(b_pickup_idx);
        debug_assert!(self.next[b_delivery_idx] != UNSERVED);
        debug_assert!(self.prev[b_delivery_idx] != UNSERVED);

        self.remove_pair(a_pickup_idx); // TODO: inefficient, fix
        self.remove_pair(b_pickup_idx); // TODO: inefficient, fix

        self.routes.remove(&swap.a.put_pickup_between.0);
        self.routes.remove(&swap.a.put_pickup_between.1);
        self.routes.remove(&swap.b.put_pickup_between.0);
        self.routes.remove(&swap.b.put_pickup_between.1);

        self.link_unsafe(a_pickup_idx, &swap.a.put_pickup_between);
        self.link_unsafe(a_delivery_idx, &swap.a.put_delivery_between);
        self.link_unsafe(b_pickup_idx, &swap.b.put_pickup_between);
        self.link_unsafe(b_delivery_idx, &swap.b.put_delivery_between);

        let first = self.first[a_pickup_idx];
        self.routes.insert(first);
        self.fix_route(first);

        let first = self.first[b_pickup_idx];
        self.routes.insert(first);
        self.fix_route(first);
    }

    pub fn make_move(&mut self, mov: &Move) {
        let pickup_idx = mov.pickup;
        debug_assert!(self.next[pickup_idx] == UNSERVED);
        debug_assert!(self.prev[pickup_idx] == UNSERVED);
        let delivery_idx = self.data.pair_of(pickup_idx);
        debug_assert!(self.next[delivery_idx] == UNSERVED);
        debug_assert!(self.prev[delivery_idx] == UNSERVED);

        for &removed in mov.removed.iter().filter(|&&x| x != 0) {
            self.remove_pair(removed); // TODO: inefficient, fix
            self.heap.push(removed);
            self.heap.inc();
        }

        self.routes.remove(&mov.put_pickup_between.0);
        self.routes.remove(&mov.put_pickup_between.1);

        self.link_unsafe(pickup_idx, &mov.put_pickup_between);
        self.link_unsafe(delivery_idx, &mov.put_delivery_between);

        let first = self.first[pickup_idx];
        self.routes.insert(first);
        self.fix_route(first);
    }

    fn link_unsafe(&mut self, point_idx: usize, &Between(before, after): &Between) {
        debug_assert!(before != 0 || after != 0);
        debug_assert!(before == 0 || self.next[before] == after);
        debug_assert!(after == 0 || self.prev[after] == before);
        debug_assert!(self.prev[point_idx] == UNSERVED);
        debug_assert!(self.next[point_idx] == UNSERVED);
        let first = if before == 0 {
            point_idx
        } else {
            self.first[before]
        };

        self.first[point_idx] = first;

        self.next[before] = point_idx;
        self.next[point_idx] = after;

        self.prev[after] = point_idx;
        self.prev[point_idx] = before;

        // in case before or after is the depot
        self.next[0] = 0;
        self.prev[0] = 0;
    }

    pub fn check_routes(&mut self) -> bool {
        for &r in self.routes.iter() {
            let mut z = r;

            let mut route = vec![];
            while z != 0 {
                route.push(z);
                z = self.next[z];
            }

            debug_assert!(self.prev[r] == 0);
            debug_assert!(self.first[r] == r);

            for (&p, &n) in route.iter().tuple_windows() {
                debug_assert!(self.prev[n] == p);
                debug_assert!(self.first[n] == r);
                debug_assert!(self.next[p] == n);
            }

            debug_assert!(route.len() % 2 == 0);

            debug_assert!(self.next[*route.last().unwrap()] == 0);
            println!("{route:?}");
        }

        for i in 1..self.data.points {
            debug_assert!(self.prev[i] != i);
            debug_assert!(self.next[i] != i);
            debug_assert!((self.next[i] == UNSERVED) == (self.prev[i] == UNSERVED));
            debug_assert!(
                self.prev[i] == 0
                    || (self.prev[i] == UNSERVED && self.next[i] == UNSERVED)
                    || self.next[self.prev[i]] == i
            );
            debug_assert!(
                self.next[i] == 0
                    || (self.prev[i] == UNSERVED && self.next[i] == UNSERVED)
                    || self.next[i] == UNSERVED
                    || self.prev[self.next[i]] == i
            );
        }
        true
    }

    pub fn rs(&self) -> Vec<Vec<usize>> {
        let mut routes = vec![];
        for &first in self.routes.iter() {
            let mut z = first;
            let mut route = vec![];
            while z != 0 {
                route.push(z);
                z = self.next[z];
            }
            routes.push(route);
        }

        routes
    }

    pub fn eprn(&self) {
        let rs = self.rs();

        eprintln!("Instance name:");
        eprintln!("Authors:");
        eprintln!("Reference:");
        eprintln!("Date:");
        eprintln!("Solution :");
        for (i, r) in rs.iter().sorted().enumerate() {
            eprint!("Route {} :", i + 1);
            for x in r {
                eprint!(" {x}");
            }
            eprintln!("");
        }
    }

    pub fn route_iter(&self, start: usize) -> RouteIterator {
        RouteIterator {
            solution: &self,
            prev: 0,
            finished: false,
            cur: start,
        }
    }
}

#[derive(Clone, Copy)]
pub struct RouteIterator<'a> {
    solution: &'a Sol<'a>,
    prev: usize,
    cur: usize,
    finished: bool,
}

impl<'a> Iterator for RouteIterator<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            None
        } else {
            self.prev = self.cur;
            self.cur = self.solution.next[self.cur];
            self.finished = self.prev + self.cur == 0;
            Some(self.prev)
        }
    }
}
