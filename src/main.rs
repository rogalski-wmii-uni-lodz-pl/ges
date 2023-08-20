use std::path::Path;
use std::time::Duration;

use ges::data::Data;
use ges::routes::ROUTES;
use ges::Ges;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let path = Path::new(&args[1]);
    let instance = path
        .file_stem()
        .unwrap()
        .to_ascii_lowercase()
        .into_string()
        .unwrap();
    let data = Data::read(path);

    let mut solution = ges::sol::Sol::new(&data);
    solution.initialize();
    let mut ges = Ges::new(&data);

    let total_time = 600;
    let log = ges::Log::Quiet;

    let conf = ges::Conf {
        max_optimization_time: Duration::new(total_time, 0),
        log,
        target_routes: ROUTES[&instance],
    };

    print!(
        "{:?} {} {} ",
        instance,
        conf.target_routes,
        conf.max_optimization_time.as_secs()
    );

    ges.ges(&mut solution, conf);
}
