use crate::data::{idx, Data};

pub struct Eval {
    pub node: usize,
    pub distance: u64,
    pub time: u64,
    pub capacity: i32,
}

impl Eval {
    pub fn new() -> Self {
        Eval {
            node: 0,
            distance: 0,
            time: 0,
            capacity: 0,
        }
    }

    pub fn reset_to(&mut self, other: &Self) {
        self.node = other.node;
        self.distance = other.distance;
        self.time = other.time;
        self.capacity = other.capacity;
    }

    pub fn next(&mut self, next_node: usize, data: &Data) {
        let nn = &data.pts[next_node];
        let i = idx(self.node, next_node);
        self.distance += data.dist[i];
        self.time += data.time[i];
        self.time = std::cmp::max(self.time, nn.start);
        self.capacity += nn.dem;

        self.node = next_node;
    }

    pub fn can_delivery_be_inserted(
        &mut self,
        inserted_node_id: usize,
        next_node_id: usize,
        data: &Data,
        latest_feasible_departure_from_next: u64,
    ) -> bool {
        let inserted_node = &data.pts[inserted_node_id];

        let inserted_arrival = self.time + data.time_between(self.node, inserted_node_id);
        let inserted_service_start = inserted_arrival.max(inserted_node.start);
        let next_arrival =
            inserted_service_start + data.time_between(inserted_node_id, next_node_id);

        // let capacity_after_insertion = self.capacity + inserted_node.dem;

        let arrived_at_inserted_before_due_time = inserted_arrival <= inserted_node.due;
        let arrived_at_next_no_later_than_feasible =
            next_arrival <= latest_feasible_departure_from_next;

        arrived_at_inserted_before_due_time && arrived_at_next_no_later_than_feasible
        // && capacity_after_insertion <= data.max_cap && c >= 0
        // this check is unnecessary because pickups are always before deliveries, so if pickup has
        // not violated the capacity constraint, then adding delivery will not violate the capacity
        // constraint as well
    }

    pub fn is_feasible(&self, data: &Data) -> bool {
        let node = &data.pts[self.node];

        self.time <= node.due && self.capacity <= data.max_cap
        // && self.cap >= 0 this chceck is unnecessary because pickups are always before deliveries
    }

    pub fn arrives_too_late(&self, data: &Data) -> bool {
        self.time > data.pts[self.node].due
    }
}
