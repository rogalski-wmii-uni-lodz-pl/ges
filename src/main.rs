use std::path::Path;

use ges::{data::Data, evaluator::Evaluator, mov::Move};

use itertools::Itertools;
use rand::{self, seq::IteratorRandom};


fn main() {
    let args: Vec<String> = std::env::args().collect();

    let path = Path::new(&args[1]);
    let data = Data::read(path);

    // let routes = vec![
    //     vec![0, 32, 171, 65, 86, 115, 94, 51, 174, 136, 189, 0],
    //     vec![0, 177, 3, 88, 8, 186, 127, 98, 157, 137, 183, 0],
    //     // vec![0, 177, 88, 8, 186, 127, 157, 137, 183, 0],
    //     vec![0, 21, 23, 182, 75, 163, 194, 145, 195, 52, 92, 0],
    //     vec![0, 161, 104, 18, 54, 185, 132, 7, 181, 117, 49, 0],
    //     vec![0, 60, 211, 82, 180, 84, 191, 125, 4, 72, 17, 0],
    //     vec![0, 148, 103, 197, 203, 124, 141, 69, 200, 0],
    //     vec![0, 170, 134, 50, 156, 112, 168, 79, 205, 29, 87, 42, 123, 0],
    //     vec![
    //         0, 114, 159, 38, 150, 22, 151, 16, 140, 204, 187, 142, 111, 63, 56, 0,
    //     ],
    //     vec![0, 190, 5, 10, 193, 46, 128, 106, 167, 207, 34, 95, 158, 0],
    //     vec![0, 57, 118, 83, 143, 176, 36, 206, 33, 121, 165, 188, 108, 0],
    //     vec![0, 93, 55, 135, 58, 202, 184, 199, 37, 81, 138, 0],
    //     vec![0, 133, 48, 26, 152, 40, 153, 169, 89, 105, 15, 59, 198, 0],
    //     vec![0, 164, 210, 66, 147, 160, 47, 91, 70, 0],
    //     vec![0, 101, 144, 119, 166, 35, 126, 71, 9, 1, 99, 53, 201, 0],
    //     vec![0, 30, 120, 19, 192, 196, 97, 14, 96, 130, 28, 74, 149, 0],
    //     vec![0, 20, 41, 85, 80, 31, 25, 172, 77, 110, 162, 0],
    //     vec![0, 73, 116, 12, 129, 11, 6, 122, 139, 0],
    //     vec![0, 62, 131, 44, 102, 146, 208, 68, 76, 0],
    //     vec![0, 45, 178, 27, 173, 154, 209, 24, 61, 100, 64, 179, 109, 0],
    //     vec![0, 113, 155, 78, 175, 13, 43, 2, 90, 67, 39, 107, 212, 0],
    // ];

    let mut s = ges::Sol::new(&data);
    // println!("{:#?}", data.points);
    for i in 1..data.points {
        let p = data.pts[i];
        if !p.delivery {
            let vec = vec![0, i, p.pair, 0];
            s.add_route(&vec)
        }
    }


    // let mut r = StdRng::seed_from_u64(11);

    let mut ev = Evaluator::new(&data);

    for _ in 0.. {
        println!("routes: {}", s.routes.iter().count());
        let r = *s.routes.iter().sorted().choose(&mut rand::thread_rng()).unwrap();
        // let v = s.routes.iter().collect_vec();
        println!("{r:?}");
        s.eprn();
        s.remove_route(r);
        while let Some(top) = s.top() {
            let mov = s.try_insert(top, &mut ev);

            if !mov.empty() {
                s.pop();
                s.make_move(top, &mov);
                debug_assert!(s.check_routes());
            } else {
                s.inc();
                for _ in 0..100 {
                    s.perturb(&mut ev);
                }
                s.prn_heap();
            }
        }
    }

}
