//! The bench that faces the water: four slats on two cast-iron ends, its back to the land, and a brass plaque.

use super::{F, M};
use artkit::json::float_repr;
use artkit::kit::{BenchEnd, bench_end};
use artkit::parts::{join_all, settle_all};
use artkit::scene::Obj;
use artkit::{fbox, fsphere};

pub fn bench() -> Vec<Obj> {
    // the ends: the back leg curving up into the upright, the front leg into the arm, on pad feet
    for x in [-0.85, 0.85] {
        bench_end(
            &F,
            format!("end{}", float_repr(x)),
            x,
            0.44,
            0.20,
            -0.20,
            (0.90, -0.26),
            M.bench_iron,
            BenchEnd {
                arm: Some(0.70),
                t: 0.026,
                w: 0.042,
                steps: 3,
                corner: 1,
                scrolls: true,
                ..BenchEnd::default()
            },
        );
    }
    // the seat slats, rounded, held down by domed bolts over each end
    for (k, z) in [-0.1408, -0.0484, 0.044, 0.1364].into_iter().enumerate() {
        fbox!(
            F,
            format!("slat{k}"),
            1.72,
            0.035,
            0.085,
            (0, 0.4575, z),
            M.bench_wood,
            bevel = 0.009,
            bevel_segments = 2
        );
        for x in [-0.85, 0.85] {
            fsphere!(
                F,
                format!("bolt{k}{}", float_repr(x)),
                0.009,
                (x, 0.475, z),
                M.bench_iron,
                segments = 6,
                rings = 3,
                scale = (1, 1, 0.5)
            );
        }
    }
    // the back: three slats leaning back, the top one carrying a brass plaque
    for (k, (y, z)) in [(0.62, -0.18), (0.75, -0.225), (0.88, -0.27)]
        .into_iter()
        .enumerate()
    {
        fbox!(
            F,
            format!("back{k}"),
            1.72,
            0.085,
            0.028,
            (0, y, z),
            M.bench_wood,
            bevel = 0.009,
            bevel_segments = 2,
            rot = (-12, 0, 0)
        );
        for x in [-0.85, 0.85] {
            fsphere!(
                F,
                format!("back_bolt{k}{}", float_repr(x)),
                0.009,
                (x, y, z + 0.016),
                M.bench_iron,
                segments = 6,
                rings = 3,
                scale = (1, 0.5, 1)
            );
        }
    }
    fbox!(
        F,
        "plaque",
        0.26,
        0.09,
        0.006,
        (0, 0.88, -0.252),
        M.brass,
        bevel = 0.002
    );
    join_all("bench");
    settle_all()
}
