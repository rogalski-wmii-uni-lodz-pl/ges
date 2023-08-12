use std::path::Path;

use ges::data::Data;
use ges::Ges;
use ges::routes::ROUTES;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let path = Path::new(&args[1]);
    let instance = path.file_stem().unwrap().to_ascii_lowercase().into_string().unwrap();
    let data = Data::read(path);

    let mut solution = ges::sol::Sol::new(&data);
    solution.initialize();
    let mut ges = Ges::new(&data);

    print!("{:?} {} {} ", instance, ROUTES[&instance], ges::TOTAL_TIME);
    ges.ges(&mut solution, ROUTES[&instance]);
}
