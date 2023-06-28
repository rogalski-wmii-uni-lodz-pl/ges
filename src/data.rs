use std::path::Path;
use verifier::{read, verify::instance::Instance};

pub const PTS: usize = 2000;
pub const SIZE: usize = PTS * PTS;
pub const MULT: u64 = 10000;

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

    pub fn pair_of(&self, idx: usize) -> usize {
        self.pts[idx].pair
    }
}
