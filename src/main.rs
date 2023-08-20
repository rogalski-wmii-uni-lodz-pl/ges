use std::path::Path;
use std::time::Duration;

use ges::data::Data;
use ges::routes::ROUTES;
use ges::Ges;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    path: String,

    #[arg(short, long)]
    max_time: Option<u64>,

    #[arg(short, long)]
    target_routes: Option<usize>,

    #[arg(short, long)]
    quiet: bool,

    #[arg(short, long)]
    verbose: bool,

    #[arg(short, long)]
    extra: bool,
}

fn main() {
    let args = Args::parse();

    let path = Path::new(&args.path);
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

    let mut conf = ges::Conf::default();

    conf.target_routes = if let Some(x) = args.target_routes {
        x
    } else {
        ROUTES[&instance]
    };

    conf.max_optimization_time = if let Some(x) = args.max_time {
        Duration::new(x, 0)
    } else {
        Duration::new(u64::max_value(), 0)
    };

    conf.log = if args.quiet {
        ges::Log::Quiet
    } else if args.verbose {
        ges::Log::Verbose
    } else if args.extra {
        ges::Log::Extra
    } else {
        ges::Log::Quiet
    };

    print!(
        "{:?} {} {:?} ",
        instance,
        conf.target_routes,
        conf.max_optimization_time
    );

    ges.ges(&mut solution, conf);
}
