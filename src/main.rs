use std::path::Path;

use ges::data::Data;
use ges::Ges;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let path = Path::new(&args[1]);
    let data = Data::read(path);

    let mut solution = ges::sol::Sol::new(&data);
    solution.initialize();
    let mut ges = Ges::new(&data);

    ges.ges(&mut solution);
}
