use itertools::{self, Itertools};
use rand;
use std::{collections::BinaryHeap, path::Path};
use verifier::{read, verify::instance::Instance};

const PTS: usize = 2000;
const SIZE: usize = PTS * PTS;
const MULT: u64 = 10000;
const UNSERVED: usize = usize::MAX;

#[derive(Default, Copy, Clone, Debug)]
pub struct PointData {
    pub dem: i32,
    pub start: u64,
    pub due: u64,
    pub pair: usize,
    pub delivery: bool,
}

#[derive(Debug)]
pub struct Data {
    pub dist: Vec<u64>,
    pub pts: [PointData; PTS],
    pub max_cap: i32,
    pub time: Vec<u64>,
}

pub fn idx(a: usize, b: usize) -> usize {
    a * PTS + b
}

impl Data {
    pub fn read(path: &Path) -> Self {
        let g = read::<Instance>(path).unwrap();

        let mut dist = vec![u64::MAX; SIZE];
        let mut time = vec![u64::MAX; SIZE];
        let mut pts = [Default::default(); PTS];

        for a in g.pts.iter() {
            let (p, d) = a.pickup_delivery.unwrap();
            let pair = if p == 0 { d } else { p };

            pts[a.id as usize] = PointData {
                dem: a.demand,
                start: a.start as u64 * MULT,
                due: a.due as u64 * MULT,
                pair: pair as usize,
                delivery: d == a.id,
            };

            for b in g.pts.iter() {
                let xs = a.x - b.x;
                let xs2 = (xs * xs) as u64;

                let ys = a.y - b.y;
                let ys2 = (ys * ys) as u64;

                let shifted = xs2 + ys2;
                let sqrted = ((shifted as f64).sqrt() * MULT as f64).ceil() as u64;

                let loc = idx(a.id as usize, b.id as usize);
                dist[loc] = sqrted;
                time[loc] = sqrted + a.service as u64 * MULT;
            }
        }

        Data {
            dist,
            time,
            max_cap: g.max_capacity,
            pts,
        }
    }
}

#[derive(Eq)]
pub struct Removed {
    pub pickup: usize,
    pub delivery: usize,
    pub times: usize,
}

impl PartialEq for Removed {
    fn eq(&self, other: &Self) -> bool {
        self.times == other.times
    }
}

impl PartialOrd for Removed {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.times.cmp(&other.times))
    }
}

impl Ord for Removed {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.times.cmp(&other.times)
    }
}

pub struct Sol {
    // pub data: Rc<Data>,
    pub next: [usize; PTS],
    pub prev: [usize; PTS],
    pub latest_feasible_departure: [u64; PTS],
    pub removed_times: [u64; PTS],
    pub removed: BinaryHeap<Removed>,
    pub evals: Vec<Eval>,
}

impl Sol {
    pub fn new() -> Self {
        let next = [UNSERVED; PTS];
        let prev = [UNSERVED; PTS];
        let latest_feasible_departure = [0; PTS];
        let removed_times = [0; PTS];
        let removed = BinaryHeap::new();
        let evals: Vec<_> = (0..PTS).map(|_| Eval::new()).collect();

        Sol {
            next,
            prev,
            latest_feasible_departure,
            removed_times,
            removed,
            evals,
        }
    }

    pub fn add_route(&mut self, data: &Data, route: &Vec<usize>) {
        let pts = &data.pts;
        let mut prev = 0;
        let mut latest_feasible_departure = pts[0].due;
        for &n in route.iter().rev() {
            let drive_time = data.time[idx(n, prev)];
            latest_feasible_departure = latest_feasible_departure - drive_time;
            self.latest_feasible_departure[n] = latest_feasible_departure;
            latest_feasible_departure = std::cmp::min(pts[prev].due, latest_feasible_departure);
            prev = n;
        }

        self.prev[route[0]] = 0;
        for (&bef, &aft) in route.iter().tuple_windows() {
            self.prev[aft] = bef;
            self.next[bef] = aft;
        }
        self.next[*route.last().unwrap()] = 0;

        let mut ev = Eval::new();

        for &n in route.iter() {
            ev.next(n, data);
            self.evals[n].reset_to(&ev);
        }

        // debug_assert!((|| {
        // })());
        //
        debug_assert!({
            let mut e = Eval::new();
            route.iter().fold(true, |acc, &n| {
                e.next(n, data);
                acc && e.time < self.latest_feasible_departure[n]
            })
        });
    }

    pub fn check_add(&self, data: &Data, start: usize, pickup_id: usize) -> Option<usize> {
        // let mut cur = route_first;
        let pickup = data.pts[pickup_id];
        let delivery_id = pickup.pair;

        // let mut ev_route = Eval::new();
        // let mut ev_pickup = Eval::new();
        // let mut ev_delivery = Eval::new();

        // while cur != 0 {
        //     ev_pickup.reset_to(&ev_route);
        //     ev_pickup.next(pickup_id, data);
        //     let ok_pickup = ev_pickup.check(data);

        //     if ok_pickup {
        //         let mut curp = cur;
        //         while curp != 0 {
        //             ev_delivery.reset_to(&ev_pickup);
        //             ev_delivery.next(delivery_id, data);
        //             curp = self.next[curp];
        //         }
        //     }
        //     ev_route.next(cur, data);

        //     cur = self.next[cur];
        // }
        //
        let mut route = Vec::with_capacity(1000);

        let mut cur = start;

        route.push(self.prev[cur]);
        while cur != 0 {
            route.push(cur);
            cur = self.next[cur];
        }

        let insertions = route
            .iter()
            .tuple_windows()
            .map(|(&p, &n)| {
                let mut mov = None;
                let mut ev_pickup = Eval::new();
                let mut ev_delivery = Eval::new();
                ev_pickup.reset_to(&self.evals[p]);

                ev_pickup.next(pickup_id, data);

                let can_insert_pickup = ev_pickup.check(data);

                if can_insert_pickup {
                    ev_delivery.reset_to(&ev_pickup);
                    let mut curp = n;
                    while curp != 0 {
                        let can_insert_delivery = ev_delivery.check_insert(
                            delivery_id,
                            curp,
                            &data,
                            self.latest_feasible_departure[curp],
                        );

                        if can_insert_delivery {
                            let pick = (p, n);
                            let deli = (curp, self.next[curp]);

                            mov = match mov {
                                None => Some(Move::new(pick, deli)),
                                Some(m) => Some(m.maybe_switch(pick, deli)),
                            }
                        }

                        ev_delivery.next(curp, data);
                        curp = self.next[curp];
                    }
                }

                mov
            })
            .flatten()
            .reduce(|acc, m| acc.pick(&m));

        println!("{:?}", insertions);

        // route.iter()..for_each(|x| {
        //     let mut ev_pickup = Eval::new();
        //     ev_pickup.reset_to(self.evals[x])

        // });

        None
    }
}


#[derive(Debug)]
pub struct Move {
    pub pickup: (usize, usize),
    pub delivery: (usize, usize),
    pub times: usize,
}

impl Move {
    pub fn new(pickup: (usize, usize), delivery: (usize, usize)) -> Self {
        Self {
            pickup,
            delivery,
            times: 1,
        }
    }

    pub fn maybe_switch(self, pickup: (usize, usize), delivery: (usize, usize)) -> Self {
        let times = self.times + 1;
        if rand::random::<usize>() % times == 0 {
            Self {
                times,
                pickup,
                delivery,
            }
        } else {
            Self { times, ..self }
        }
    }

    pub fn pick(self, other: &Self) -> Self {
        let times = self.times + other.times;
        let which = if rand::random::<usize>() % times < self.times {
            &self
        } else {
            other
        };
        Self { times, ..(*which) }
    }
}

pub struct Eval {
    pub node: usize,
    pub dist: u64,
    pub time: u64,
    pub cap: i32,
}

impl Eval {
    pub fn new() -> Self {
        Eval {
            node: 0,
            dist: 0,
            time: 0,
            cap: 0,
        }
    }

    pub fn reset_to(&mut self, other: &Self) {
        self.node = other.node;
        self.dist = other.dist;
        self.time = other.time;
        self.cap = other.cap;
    }

    pub fn next(&mut self, next_node: usize, data: &Data) {
        let nn = &data.pts[next_node];
        self.dist += data.dist[idx(self.node, next_node)];
        self.time += data.time[idx(self.node, next_node)];
        self.time = std::cmp::max(self.time, nn.start);
        self.cap += nn.dem;

        self.node = next_node;
    }

    pub fn check_insert(
        &mut self,
        inserted_node_id: usize,
        next_node_id: usize,
        data: &Data,
        latest_feasible_departure_from_next: u64,
    ) -> bool {
        let ins = &data.pts[inserted_node_id];

        let ins_arrival = self.time + data.time[idx(self.node, inserted_node_id)];
        let ins_service_start = std::cmp::max(ins_arrival, ins.start);
        let next_arrival = ins_service_start + data.time[idx(inserted_node_id, next_node_id)];

        let c = self.cap + ins.dem;

        ins_service_start <= ins.due
            && next_arrival <= latest_feasible_departure_from_next
            && c >= 0
            && c <= data.max_cap
    }

    pub fn check(&self, data: &Data) -> bool {
        let nn = &data.pts[self.node];

        self.time <= nn.due && self.cap <= data.max_cap && self.cap >= 0
    }
}

#[cfg(test)]
mod tests {
    // use super::*;
}
