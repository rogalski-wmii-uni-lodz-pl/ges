use crate::sol::Sol;
use std::time::Instant;

pub struct HeapSize {
    min: usize,
    max: usize,
}

impl HeapSize {
    pub fn new() -> Self {
        Self {
            min: usize::max_value(),
            max: usize::min_value(),
        }
    }

    pub fn change(&mut self, heap_size: usize) {
        self.min = self.min.min(heap_size);
        self.max = self.max.max(heap_size);
    }
}

#[derive(Default)]
pub struct Iterations {
    total: usize,
    current: usize,
}

impl Iterations {
    pub fn inc(&mut self) {
        self.total += 1;
        self.current += 1;
    }

    pub fn current(&self) -> usize {
        self.current
    }
}

pub struct Time {
    start: Instant,
    route_start: Instant,
}

impl Time {
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            start: now.clone(),
            route_start: now.clone(),
        }
    }
}

pub struct Stats {
    time: Time,
    iterations: Iterations,
    heap_size: HeapSize,
}

impl Stats {
    pub fn new() -> Self {
        Self {
            time: Time::new(),
            iterations: Iterations::default(),
            heap_size: HeapSize::new(),
        }
    }

    pub fn reset(&mut self) {
        self.heap_size = HeapSize::new();
        self.iterations.current = 0;
        self.time.route_start = Instant::now();
    }

    pub fn add_iteration(&mut self, heap_size: usize) {
        self.heap_size.change(heap_size);
        self.iterations.inc();
    }

    fn print_stats_for_this_route(&self, solution: &Sol) {
        let elapsed = self.time.route_start.elapsed();
        solution.eprn();
        print!(
            "routes {}, iters: {}, after {elapsed:?}, <{}-{}>: ",
            solution.routes_number() + 1,
            self.iterations.current,
            self.heap_size.min,
            self.heap_size.max
        );
        solution.prn_heap();
    }

    fn print_total_stats(&self) {
        let elapsed = self.time.start.elapsed();
        println!("total {} after {elapsed:?}", self.iterations.total);
    }

    pub fn print_occasionally(&self, solution: &Sol) {
        if self.iterations.current % 10000 == 0 {
            self.print_stats_for_this_route(solution);
        }

        if self.iterations.total % 10000 == 0 {
            self.print_total_stats();
        }
    }

    pub fn print_after_route_removal(&self) {
        println!(
            "after {}, {:?}",
            self.iterations().current(),
            self.time.route_start.elapsed()
        );
    }

    pub fn iterations(&self) -> &Iterations {
        &self.iterations
    }

    pub fn total_time(&self) -> std::time::Duration {
        self.time.start.elapsed()
    }
}
