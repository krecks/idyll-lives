use super::super::common::{F, M, bolts, finish};
use super::bench::cast_end;
use artkit::fbox;
use artkit::json::float_repr;
use artkit::scene::Obj;

// --- the garden bench: three seat slats and three back slats on three iron frames ------------------------------
// 1.9 m on its two tiles: the content's three sitters, spread across them, 0.67 m apart.
pub(super) const GARDEN_BENCH_END: f64 = 0.92; // its end frames, either side

pub fn bench_garden3() -> Vec<Obj> {
    for (k, z) in [-0.08, 0.04, 0.16].into_iter().enumerate() {
        fbox!(
            F,
            format!("seat{k}"),
            1.9,
            0.03,
            0.1,
            (0, 0.45, z),
            M.bench_slats,
            bevel = 0.01
        );
    }
    let backs: Vec<(f64, f64)> = [0.6, 0.72, 0.84]
        .iter()
        .map(|&y| (y, -0.2 - (y - 0.6) * 0.15))
        .collect();
    for (k, &(y, z)) in backs.iter().enumerate() {
        fbox!(
            F,
            format!("back{k}"),
            1.9,
            0.09,
            0.025,
            (0, y, z),
            M.bench_slats,
            rot = (-12, 0, 0),
            bevel = 0.01
        );
    }
    // the support rises in line with the back slats, just in front of them, and curls back over the top
    let back = [
        (0.0, -0.205),
        (0.08, -0.17),
        (0.43, -0.16),
        (0.8, -0.214),
        (0.87, -0.234),
        (0.886, -0.28),
        (0.868, -0.313),
    ];
    for x in [-GARDEN_BENCH_END, 0.0, GARDEN_BENCH_END] {
        let end = x.abs() > 0.0;
        let front: &[(f64, f64)] = if end {
            &[
                (0.0, 0.2),
                (0.44, 0.205),
                (0.62, 0.235),
                (0.664, 0.26),
                (0.664, 0.2),
                (0.664, -0.21),
            ]
        } else {
            &[(0.0, 0.19), (0.435, 0.2)]
        };
        cast_end(
            &format!("frame{}", float_repr(x)),
            x,
            &back,
            front,
            (0.42, -0.2, 0.215),
            (0.1, -0.19, 0.196),
            if end {
                Some((0.678, -0.215, 0.28))
            } else {
                None
            },
            if end { Some((0.555, 0.15, 0.05)) } else { None },
        );
        let seat: Vec<(f64, f64, f64)> =
            [-0.08, 0.04, 0.16].iter().map(|&z| (x, 0.467, z)).collect();
        bolts(&format!("bolt_seat{}", float_repr(x)), &seat, M.iron, true);
        let bb: Vec<(f64, f64, f64)> = backs.iter().map(|&(y, z)| (x, y, z + 0.014)).collect();
        bolts(&format!("bolt_back{}", float_repr(x)), &bb, M.iron, false);
    }
    finish("benchGarden3", false)
}
