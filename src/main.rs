use std::path::Path;

use ges::{data::Data, evaluator::Evaluator};

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let path = Path::new(&args[1]);
    let data = Data::read(path);

    let routes = vec![
        vec![0, 32, 171, 65, 86, 115, 94, 51, 174, 136, 189, 0],
        vec![0, 177, 3, 88, 8, 186, 127, 98, 157, 137, 183, 0],
        // vec![0, 177, 88, 8, 186, 127, 157, 137, 183, 0],
        vec![0, 21, 23, 182, 75, 163, 194, 145, 195, 52, 92, 0],
        vec![0, 161, 104, 18, 54, 185, 132, 7, 181, 117, 49, 0],
        vec![0, 60, 211, 82, 180, 84, 191, 125, 4, 72, 17, 0],
        vec![0, 148, 103, 197, 203, 124, 141, 69, 200, 0],
        vec![0, 170, 134, 50, 156, 112, 168, 79, 205, 29, 87, 42, 123, 0],
        vec![
            0, 114, 159, 38, 150, 22, 151, 16, 140, 204, 187, 142, 111, 63, 56, 0,
        ],
        vec![0, 190, 5, 10, 193, 46, 128, 106, 167, 207, 34, 95, 158, 0],
        vec![0, 57, 118, 83, 143, 176, 36, 206, 33, 121, 165, 188, 108, 0],
        vec![0, 93, 55, 135, 58, 202, 184, 199, 37, 81, 138, 0],
        vec![0, 133, 48, 26, 152, 40, 153, 169, 89, 105, 15, 59, 198, 0],
        vec![0, 164, 210, 66, 147, 160, 47, 91, 70, 0],
        vec![0, 101, 144, 119, 166, 35, 126, 71, 9, 1, 99, 53, 201, 0],
        vec![0, 30, 120, 19, 192, 196, 97, 14, 96, 130, 28, 74, 149, 0],
        vec![0, 20, 41, 85, 80, 31, 25, 172, 77, 110, 162, 0],
        vec![0, 73, 116, 12, 129, 11, 6, 122, 139, 0],
        vec![0, 62, 131, 44, 102, 146, 208, 68, 76, 0],
        vec![0, 45, 178, 27, 173, 154, 209, 24, 61, 100, 64, 179, 109, 0],
        vec![0, 113, 155, 78, 175, 13, 43, 2, 90, 67, 39, 107, 212, 0],
    ];

    let mut total = 0;
    for route in routes.iter() {
        let mut e = ges::eval::Eval::new();
        for x in route.iter() {
            e.next(*x, &data);
            let ok = e.is_feasible(&data);
            assert!(ok);
        }
        total += e.distance;
    }

    let mut s = ges::Sol::new(&data);

    for r in routes {
        s.add_route(&r);
    }

    let pickup_idx = 3;

    s.remove_pair(pickup_idx);

    let mut ev = Evaluator::with_pickup(&s, &data, pickup_idx);

    let mov = ev.check_add_to_route(177);

    println!("{:#?}", mov);

    s.add_pair(pickup_idx, &mov.unwrap());
    // s.check_add(&data, 177, 3);

    println!("{:#?}", total);
}
