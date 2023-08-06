use std::path::Path;

use ges::data::Data;
use ges::Ges;

// use rand::{self, seq::IteratorRandom};

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let path = Path::new(&args[1]);
    let data = Data::read(path);

    let mut s = ges::sol::Sol::new(&data);
    s.initialize();
    let mut ges = Ges::new(&data);

    ges.ges(&mut s);
}
