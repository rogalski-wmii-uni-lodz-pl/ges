use crate::data::{idx, Data};

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
