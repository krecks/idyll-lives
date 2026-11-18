//! The long park bench and the taxi rank's shelter: seats for three.

use super::common::own;
use artkit::forms::rounded_rect;
use artkit::json::float_repr;
use artkit::math::{P3, PI, TAU, cos, sin};
use artkit::parts::join_all;
use artkit::place::{Facing, GameFrame};
use artkit::scene::{Obj, meshes};
use artkit::{
    fbox, fcyl, fextrude, fsphere, fsweep, kit_bench_end, kit_chequer, kit_fillet, kit_taxi_word,
    settle,
};

// --- the long park bench: wooden slats on three cast-iron supports --------------------------------------------
const SEAT: f64 = 0.45; // the content's `seat.height`; the seat runs centred on depth 0 (`seat.forward` 0)
// 2.5 m on its three tiles, its three sitters a metre apart (it was 2.86 m, past a long bench's 1.8-2.5)
const BENCH_LONG: f64 = 2.5;
const BENCH_LEGS: [f64; 3] = [-1.21, 0.0, 1.21];

pub fn park_bench() -> Vec<Obj> {
    let m = own("parkBench");
    let f = GameFrame::new(0.0);
    let frame = m.body;
    let rail = SEAT - 0.035; // the supports' seat rails, under the slats
    {
        // three cast-iron supports, each in one piece (the kit's bench end), arms and scrolls on the end two
        for x in BENCH_LEGS {
            kit_bench_end!(
                f,
                format!("end{}", float_repr(x)),
                x,
                rail,
                0.2,
                -0.25,
                (0.885, -0.3),
                frame,
                arm = if x != 0.0 { Some(0.682) } else { None },
                t = 0.028,
                w = 0.044,
                foot = 0.1
            );
        }
        for (k, z) in [-0.17, -0.085, 0.0, 0.085, 0.17].into_iter().enumerate() {
            fbox!(
                f,
                format!("slat{k}"),
                BENCH_LONG,
                0.035,
                0.072,
                (0, SEAT - 0.0175, z),
                m.wood,
                bevel = 0.009
            );
            for x in [BENCH_LEGS[0], BENCH_LEGS[2]] {
                // domed bolts over the end supports
                fsphere!(
                    f,
                    format!("bolt{k}{}", float_repr(x)),
                    0.009,
                    (x, SEAT, z),
                    frame,
                    segments = 6,
                    rings = 3,
                    scale = (1, 1, 0.5)
                );
            }
        }
        for (k, y) in [0.6, 0.72, 0.84].into_iter().enumerate() {
            let z = -0.2 - (y - 0.42) * 0.1 / 0.48 + 0.03;
            fbox!(
                f,
                format!("back{k}"),
                BENCH_LONG,
                0.085,
                0.03,
                (0, y, z),
                m.wood,
                bevel = 0.009,
                rot = (-12, 0, 0)
            );
            for x in [BENCH_LEGS[0], BENCH_LEGS[2]] {
                fsphere!(
                    f,
                    format!("back_bolt{k}{}", float_repr(x)),
                    0.009,
                    (x, y, z + 0.017),
                    frame,
                    segments = 6,
                    rings = 3,
                    scale = (1, 0.5, 1)
                );
            }
        }
    }
    join_all("parkBench");
    settle!(centre = false);
    meshes()
}

// --- the taxi rank's shelter (was the bus shelter): a glazed frame with a bench for three --------------------------
// Its id stays `busStop`, so the towns and saves that have one keep it; it reads as a taxi rank: the taxi's yellow
// and chequers along the fascia, a lit TAXI sign on the roof, the fares on the back glass.
const SHELTER_SEAT: f64 = 0.46; // the content's `seat.height`
const SHELTER_BENCH: f64 = -0.31; // the bench's middle, its two slats either side (the content's `seat.forward` -0.3: the front one)
// The posts' depths: 1.2 m deep under the roof on its one tile, a shelter's depth (it was 0.98 m, the bench filling it).
const SHELTER_BACK: f64 = -0.55;
const SHELTER_FRONT: f64 = 0.55;

pub fn bus_stop() -> Vec<Obj> {
    let m = own("busStop");
    let f = GameFrame::new(0.0);
    let frame = m.body;
    let (back, front) = (SHELTER_BACK, SHELTER_FRONT);
    let (mid, deep) = ((back + front) / 2.0, front - back);
    {
        // four posts on bolted foot plates, capped
        for x in [-1.42, 1.42] {
            for z in [back, front] {
                fbox!(
                    f,
                    format!("post{}{}", float_repr(x), float_repr(z)),
                    0.07,
                    2.32,
                    0.07,
                    (x, 1.16, z),
                    frame,
                    bevel = 0.012
                );
                fbox!(
                    f,
                    format!("plate{}{}", float_repr(x), float_repr(z)),
                    0.1,
                    0.012,
                    0.1,
                    (x, 0.006, z),
                    frame,
                    bevel = 0.003,
                    bevel_segments = 1
                );
                for k in 0..4 {
                    let a = TAU * (k as f64 + 0.5) / 4.0;
                    fcyl!(
                        f,
                        format!("bolt{}{}{k}", float_repr(x), float_repr(z)),
                        0.007,
                        0.012,
                        (x + 0.05 * cos(a), 0.016, z + 0.05 * sin(a)),
                        frame,
                        verts = 6
                    );
                }
            }
        }
        // the roof: a curved canopy on a beam along the front and the back, a gutter at the back
        let arc: Vec<(f64, f64)> = (0..13)
            .map(|i| (back - 0.0525 + (deep + 0.105) * i as f64 / 12.0, 0.0))
            .collect();
        let lower: Vec<(f64, f64)> = arc
            .iter()
            .enumerate()
            .map(|(i, &(u, _))| (u, 2.352 + 0.05 * sin(PI * i as f64 / 12.0)))
            .collect();
        let upper: Vec<(f64, f64)> = lower.iter().rev().map(|&(u, y)| (u, y + 0.03)).collect();
        let roof: Vec<(f64, f64)> = lower.iter().chain(&upper).copied().collect();
        fextrude!(
            f,
            "roof",
            &roof,
            2.98,
            (0, 0, 0),
            frame,
            plane = Facing::Side,
            bevel = 0.006,
            bevel_segments = 1
        );
        for z in [back, front] {
            fbox!(
                f,
                format!("beam{}", float_repr(z)),
                2.9,
                0.09,
                0.06,
                (0, 2.315, z),
                frame,
                bevel = 0.01
            );
        }
        fbox!(
            f,
            "fascia",
            2.98,
            0.11,
            0.025,
            (0, 2.36, front + 0.037),
            frame,
            bevel = 0.008
        );
        fbox!(
            f,
            "fascia_stripe",
            2.7,
            0.08,
            0.004,
            (0, 2.36, front + 0.0505),
            m.taxi
        );
        kit_chequer!(
            f,
            "fascia_check",
            -1.33,
            1.33,
            2.395,
            front + 0.0525,
            0.035,
            1,
            m.ink
        );
        fsweep!(
            f,
            "gutter",
            &[(-1.49, 2.34, back - 0.035), (1.49, 2.34, back - 0.035)],
            0,
            frame,
            profile = rounded_rect(0.04, 0.035, 0.008, 1)
        );
        // the glazing: the back in two panes and the right-hand end, in rails and a mullion, a white band across each
        fbox!(f, "glass_back", 2.78, 1.9, 0.015, (0, 1.27, back), m.glass);
        fbox!(
            f,
            "glass_side",
            0.015,
            1.9,
            deep - 0.08,
            (1.42, 1.27, mid),
            m.glass
        );
        fbox!(
            f,
            "mullion",
            0.04,
            1.96,
            0.04,
            (0, 1.27, back),
            frame,
            bevel = 0.006
        );
        for y in [0.3, 2.24] {
            fbox!(
                f,
                format!("rail_back{}", float_repr(y)),
                2.8,
                0.05,
                0.05,
                (0, y, back),
                frame,
                bevel = 0.008
            );
            fbox!(
                f,
                format!("rail_side{}", float_repr(y)),
                0.05,
                0.05,
                deep - 0.06,
                (1.42, y, mid),
                frame,
                bevel = 0.008
            );
        }
        for s in [-1, 1] {
            fbox!(
                f,
                format!("band_back{s}"),
                1.3,
                0.05,
                0.004,
                (s as f64 * 0.69, 1.45, back + 0.01),
                m.paper
            );
        }
        fbox!(
            f,
            "band_side",
            0.004,
            0.05,
            deep - 0.14,
            (1.41, 1.45, mid),
            m.paper
        );
        // the bench: two slats on cantilevered brackets from the back rail, each on a slim leg
        for (k, z) in [SHELTER_BENCH + 0.08, SHELTER_BENCH - 0.08]
            .into_iter()
            .enumerate()
        {
            fbox!(
                f,
                format!("slat{k}"),
                2.5,
                0.035,
                0.15,
                (0, SHELTER_SEAT - 0.0175, z),
                m.wood,
                bevel = 0.009
            );
        }
        for x in [-1.0, 0.0, 1.0] {
            let path = [
                P3::from((x, 0.44, SHELTER_BENCH + 0.15)),
                P3::from((x, 0.42, back + 0.03)),
                P3::from((x, 0.3, back + 0.03)),
            ];
            fsweep!(
                f,
                format!("bracket{}", float_repr(x)),
                &kit_fillet!(&path, 0.03, steps = 2),
                0,
                frame,
                profile = rounded_rect(0.04, 0.03, 0.008, 1),
                up = (1, 0, 0)
            );
            fbox!(
                f,
                format!("bracket_leg{}", float_repr(x)),
                0.04,
                0.42,
                0.04,
                (x, 0.21, SHELTER_BENCH - 0.12),
                frame,
                bevel = 0.006
            );
        }
        // the fares on the back glass (the rank's name in a yellow head, the fares in lines), a map beside it
        fbox!(
            f,
            "fares_frame",
            0.56,
            0.78,
            0.03,
            (-0.95, 1.45, back + 0.02),
            frame,
            bevel = 0.008
        );
        fbox!(
            f,
            "fares",
            0.5,
            0.68,
            0.01,
            (-0.95, 1.43, back + 0.038),
            m.paper
        );
        fbox!(
            f,
            "fares_head",
            0.5,
            0.09,
            0.012,
            (-0.95, 1.725, back + 0.04),
            m.taxi
        );
        kit_taxi_word!(
            f,
            "fares_word",
            (-0.95, 1.725),
            0.05,
            back + 0.046,
            1,
            m.ink
        );
        for k in 0..7 {
            fbox!(
                f,
                format!("fare{k}"),
                0.26,
                0.012,
                0.004,
                (-1.02, 1.62 - 0.06 * k as f64, back + 0.044),
                m.dark
            );
            fbox!(
                f,
                format!("price{k}"),
                0.08,
                0.012,
                0.004,
                (-0.78, 1.62 - 0.06 * k as f64, back + 0.044),
                m.dark
            );
        }
        fbox!(
            f,
            "map_frame",
            0.5,
            0.6,
            0.03,
            (-0.35, 1.4, back + 0.02),
            frame,
            bevel = 0.008
        );
        fbox!(
            f,
            "map",
            0.44,
            0.54,
            0.01,
            (-0.35, 1.4, back + 0.038),
            m.map
        );
        for (k, (x, y, w, h)) in [
            (-0.4, 1.45, 0.3, 0.012),
            (-0.3, 1.35, 0.012, 0.36),
            (-0.36, 1.3, 0.25, 0.012),
        ]
        .into_iter()
        .enumerate()
        {
            fbox!(
                f,
                format!("route{k}"),
                w,
                h,
                0.004,
                (x, y, back + 0.045),
                if k != 1 { m.blue } else { m.dark }
            );
        }
        // the rank's sign on the roof, at the front right: a yellow case, a light box lettered TAXI front and back
        // (lit at night), the taxi's chequers under it
        let (sx, sy, sz) = (1.18, 2.72, front - 0.02);
        fbox!(
            f,
            "sign_pole",
            0.05,
            0.22,
            0.05,
            (sx, 2.52, sz),
            frame,
            bevel = 0.006
        );
        fbox!(
            f,
            "sign_case",
            0.6,
            0.32,
            0.1,
            (sx, sy, sz),
            m.taxi,
            bevel = 0.015
        );
        fbox!(
            f,
            "sign_light",
            0.52,
            0.16,
            0.106,
            (sx, sy + 0.06, sz),
            m.sign_light,
            bevel = 0.004,
            bevel_segments = 1
        );
        for face in [1, -1] {
            kit_taxi_word!(
                f,
                format!("sign_word{face}"),
                (sx, sy + 0.06),
                0.1,
                sz + face as f64 * 0.053,
                face,
                m.ink
            );
            kit_chequer!(
                f,
                format!("sign_check{face}"),
                sx - 0.263,
                sx + 0.263,
                sy - 0.035,
                sz + face as f64 * 0.05,
                0.035,
                face,
                m.ink
            );
        }
    }
    join_all("busStop");
    settle!(centre = false);
    meshes()
}
