//! The rides: the step-through city bike, the mountain, road, cargo and e-bikes, the kids' bike on stabilisers and the
//! e-scooter. No clips: the renderer turns their parts by code, each a top-level node on its pivot: `frame`,
//! `wheel_front`, `wheel_rear`, `cranks`, `stand`.

use super::common::{F, M, arc_points, bent, board, rod};
use super::play::{Bike, bike, wheel};
use artkit::far::{is_low, low};
use artkit::forms::rounded_rect;
use artkit::math::{P3, cosd, pow, sind};
use artkit::parts::numbered;
use artkit::parts::{Settle, join, join_all, new_parts, settle_with};
use artkit::place::Facing;
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fcyl, fextrude, fsweep, loft, revolve, ring, soft, torus};

// Each stands along x, its front wheel to +x, its middle line at depth 0, on the floor. No clips: the renderer moves
// its parts by code, each a top-level node with its origin on its pivot (NOTES.md): `frame`, `wheel_front` and
// `wheel_rear` (turned about their axles, game z), `cranks` (turned about the bottom bracket), `stand` (hidden while
// it's ridden).
fn city() -> Bike {
    Bike {
        r: 0.335,
        k: 1.08,
        rear: -0.55,
        front: 0.55,
        spokes: 24,
        segments: (36, 12),
        bb: (-0.07, 0.29),
        sc: (-0.205, 0.62),
        hb: (0.364, 0.715),
        ht: (0.333, 0.8),
        crown: Some((0.378, 0.677)),
        head: ((0.381, 0.668), (0.311, 0.86)),
        fork_mid: (0.466, 0.46),
        tubes: (0.019, 0.017, 0.021),
        // a step-through: the top tube curves down from the head tube into the seat tube low down, over the down tube
        step: Some((
            vec![(0.333, 0.8), (0.21, 0.6), (0.03, 0.47), (-0.132, 0.44)],
            vec![(0.364, 0.715), (0.25, 0.5), (0.08, 0.34), (-0.07, 0.29)],
        )),
        stay_dz: 0.04,
        guards: vec![(15.0, 125.0), (35.0, 172.0)],
        post: (-0.283, 0.81),
        saddle: (-0.36, 0.812),
        saddle_rows: vec![
            (0.0, 0.04, 0.022),
            (0.033, 0.104, 0.0385),
            (0.088, 0.098, 0.0385),
            (0.154, 0.046, 0.033),
            (0.22, 0.02, 0.024),
            (0.253, 0.011, 0.013),
        ],
        stem: ((0.311, 0.86), (0.302, 0.985)),
        // swept back to the rider: out and a little forward from the stem, then back to the grips
        bars: vec![
            (0.165, 0.972, -0.31),
            (0.235, 0.978, -0.235),
            (0.315, 0.985, -0.15),
            (0.302, 0.985, -0.04),
            (0.302, 0.985, 0.04),
            (0.315, 0.985, 0.15),
            (0.235, 0.978, 0.235),
            (0.165, 0.972, 0.31),
        ],
        bend: 0.06,
        grips: vec![
            ((0.236, 0.978, -0.234), (0.164, 0.972, -0.31)),
            ((0.236, 0.978, 0.234), (0.164, 0.972, 0.31)),
        ],
        grip_r: 0.018,
        bell: Some((0.31, 1.0, 0.11)),
        ring: 0.08,
        ring_dz: 0.05,
        cog: 0.038,
        chain_drop: 0.475,
        crank: (0.17, 0.0),
        crank_dz: 0.08,
        pedal_dz: 0.05,
        pedal: (0.09, 0.022, 0.07),
        drive: -1.0,
        kickstand: false,
        colour: M.city,
        ..Bike::default()
    }
}
const CITY_STAND: (f64, f64) = (-0.13, 0.27); // the centre stand's pivot, behind the bottom bracket

fn kid() -> Bike {
    Bike {
        r: 0.18,
        k: 0.62,
        tyre: Some(0.022),
        rear: -0.28,
        front: 0.28,
        spokes: 12,
        segments: (24, 12),
        rim_segments: 20,
        tyre_minor: 5,
        bb: (-0.05, 0.19),
        sc: (-0.126, 0.4),
        hb: (0.15, 0.42),
        ht: (0.128, 0.475),
        crown: Some((0.158, 0.401)),
        head: ((0.161, 0.392), (0.119, 0.498)),
        fork_mid: (0.22, 0.262),
        tubes: (0.016, 0.015, 0.017),
        stay_dz: 0.032,
        guards: vec![(25.0, 125.0), (40.0, 165.0)],
        post: (-0.157, 0.485),
        saddle: (-0.215, 0.49),
        saddle_rows: vec![
            (0.0, 0.028, 0.016),
            (0.022, 0.062, 0.028),
            (0.06, 0.058, 0.03),
            (0.105, 0.028, 0.024),
            (0.15, 0.012, 0.017),
            (0.172, 0.006, 0.009),
        ],
        stem: ((0.119, 0.498), (0.14, 0.54)),
        // high-rise: up from the stem in a V to the grips, a brace across
        bars: vec![
            (0.16, 0.66, -0.265),
            (0.16, 0.66, -0.13),
            (0.142, 0.545, -0.07),
            (0.14, 0.54, 0.0),
            (0.142, 0.545, 0.07),
            (0.16, 0.66, 0.13),
            (0.16, 0.66, 0.265),
        ],
        bend: 0.03,
        grips: vec![
            ((0.16, 0.66, -0.14), (0.16, 0.66, -0.26)),
            ((0.16, 0.66, 0.14), (0.16, 0.66, 0.26)),
        ],
        grip_r: 0.016,
        bell: Some((0.153, 0.627, 0.06)),
        ring: 0.05,
        ring_dz: 0.035,
        cog: 0.026,
        chain_drop: 0.52,
        crank: (0.105, 0.0),
        crank_dz: 0.062,
        pedal_dz: 0.036,
        pedal: (0.06, 0.018, 0.05),
        pedal_segments: 1,
        drive: -1.0,
        kickstand: false,
        colour: M.kid,
        guard_colour: Some(M.kid_trim),
        guard_k: Some(0.95),
        ..Bike::default()
    }
}

// The other bikes: a mountain bike, a road bike, a long cargo bike and an e-bike. Each is built along x like the city
// bike; the cargo bike is centred after building (its long front box puts its bounds ahead of the rider).
fn mtb() -> Bike {
    Bike {
        r: 0.345,
        k: 1.08,
        tyre: Some(0.034),
        rear: -0.56,
        front: 0.56,
        spokes: 28,
        segments: (32, 10),
        rim_segments: 28,
        tyre_minor: 5,
        bb: (-0.08, 0.30),
        sc: (-0.23, 0.64),
        hb: (0.37, 0.70),
        ht: (0.33, 0.82),
        crown: Some((0.39, 0.66)),
        head: ((0.39, 0.65), (0.31, 0.84)),
        fork_mid: (0.49, 0.44),
        tubes: (0.022, 0.019, 0.024),
        stay_dz: 0.045,
        guards: vec![],
        post: (-0.31, 0.83),
        saddle: (-0.40, 0.835),
        saddle_rows: vec![
            (0.0, 0.03, 0.018),
            (0.03, 0.07, 0.03),
            (0.08, 0.065, 0.03),
            (0.14, 0.03, 0.026),
            (0.2, 0.012, 0.018),
            (0.225, 0.006, 0.01),
        ],
        stem: ((0.31, 0.84), (0.36, 0.95)),
        // flat: straight out from the stem to the grips, barely swept
        bars: vec![
            (0.34, 0.955, -0.30),
            (0.36, 0.96, -0.18),
            (0.37, 0.962, -0.06),
            (0.37, 0.962, 0.06),
            (0.36, 0.96, 0.18),
            (0.34, 0.955, 0.30),
        ],
        bend: 0.015,
        grips: vec![
            ((0.36, 0.96, -0.17), (0.34, 0.955, -0.30)),
            ((0.36, 0.96, 0.17), (0.34, 0.955, 0.30)),
        ],
        grip_r: 0.017,
        bell: Some((0.35, 0.975, 0.11)),
        ring: 0.078,
        ring_dz: 0.05,
        cog: 0.04,
        chain_drop: 0.48,
        crank: (0.17, 0.0),
        crank_dz: 0.08,
        pedal_dz: 0.05,
        pedal: (0.09, 0.022, 0.07),
        drive: -1.0,
        kickstand: false,
        colour: M.mtb,
        knobs: 18,
        ..Bike::default()
    }
}

fn road() -> Bike {
    Bike {
        r: 0.345,
        k: 1.06,
        tyre: Some(0.016),
        rear: -0.56,
        front: 0.56,
        spokes: 20,
        segments: (40, 12),
        rim_segments: 36,
        tyre_minor: 4,
        bb: (-0.06, 0.30),
        sc: (-0.23, 0.66),
        hb: (0.38, 0.71),
        ht: (0.33, 0.84),
        crown: Some((0.39, 0.68)),
        head: ((0.39, 0.67), (0.31, 0.86)),
        fork_mid: (0.48, 0.44),
        tubes: (0.016, 0.014, 0.018),
        stay_dz: 0.038,
        guards: vec![],
        post: (-0.32, 0.86),
        saddle: (-0.41, 0.865),
        saddle_rows: vec![
            (0.0, 0.026, 0.015),
            (0.03, 0.06, 0.026),
            (0.08, 0.055, 0.026),
            (0.14, 0.025, 0.022),
            (0.2, 0.01, 0.015),
            (0.22, 0.005, 0.008),
        ],
        stem: ((0.31, 0.86), (0.41, 0.93)),
        // drops: out to the hoods, over the top, then down and back to the ends
        bars: vec![
            (0.31, 0.80, -0.20),
            (0.37, 0.86, -0.20),
            (0.41, 0.93, -0.16),
            (0.42, 0.95, -0.08),
            (0.42, 0.95, 0.08),
            (0.41, 0.93, 0.16),
            (0.37, 0.86, 0.20),
            (0.31, 0.80, 0.20),
        ],
        bend: 0.04,
        grips: vec![
            ((0.32, 0.81, -0.20), (0.29, 0.79, -0.21)),
            ((0.32, 0.81, 0.20), (0.29, 0.79, 0.21)),
        ],
        grip_r: 0.014,
        bell: None,
        ring: 0.085,
        ring_dz: 0.05,
        cog: 0.032,
        chain_drop: 0.50,
        crank: (0.17, 0.0),
        crank_dz: 0.075,
        pedal_dz: 0.045,
        pedal: (0.085, 0.02, 0.06),
        drive: -1.0,
        kickstand: false,
        colour: M.road,
        ..Bike::default()
    }
}

fn cargo() -> Bike {
    Bike {
        r: 0.30,
        k: 1.05,
        tyre: Some(0.03),
        rear: -0.72,
        front: 1.35,
        spokes: 28,
        segments: (32, 10),
        rim_segments: 28,
        tyre_minor: 5,
        bb: (-0.28, 0.30),
        sc: (-0.42, 0.64),
        hb: (0.40, 0.64),
        ht: (0.35, 0.80),
        crown: Some((0.41, 0.62)),
        head: ((0.40, 0.62), (0.33, 0.82)),
        fork_mid: (0.90, 0.40),
        tubes: (0.022, 0.019, 0.024),
        // a step-through, so the rider can get on round the box
        step: Some((
            vec![(0.35, 0.80), (0.18, 0.60), (-0.12, 0.46), (-0.33, 0.44)],
            vec![(0.40, 0.64), (0.22, 0.46), (0.0, 0.33), (-0.28, 0.30)],
        )),
        stay_dz: 0.05,
        guards: vec![(20.0, 140.0), (30.0, 170.0)],
        post: (-0.50, 0.82),
        saddle: (-0.58, 0.825),
        saddle_rows: vec![
            (0.0, 0.045, 0.022),
            (0.035, 0.11, 0.038),
            (0.09, 0.105, 0.038),
            (0.155, 0.05, 0.032),
            (0.22, 0.022, 0.024),
            (0.255, 0.012, 0.013),
        ],
        stem: ((0.33, 0.82), (0.30, 0.98)),
        bars: vec![
            (0.16, 0.97, -0.31),
            (0.23, 0.975, -0.235),
            (0.31, 0.98, -0.15),
            (0.30, 0.98, -0.04),
            (0.30, 0.98, 0.04),
            (0.31, 0.98, 0.15),
            (0.23, 0.975, 0.235),
            (0.16, 0.97, 0.31),
        ],
        bend: 0.06,
        grips: vec![
            ((0.23, 0.975, -0.234), (0.16, 0.97, -0.31)),
            ((0.23, 0.975, 0.234), (0.16, 0.97, 0.31)),
        ],
        grip_r: 0.018,
        bell: Some((0.31, 0.995, 0.11)),
        ring: 0.08,
        ring_dz: 0.055,
        cog: 0.042,
        chain_drop: 0.50,
        crank: (0.17, 0.0),
        crank_dz: 0.085,
        pedal_dz: 0.05,
        pedal: (0.095, 0.022, 0.075),
        drive: -1.0,
        kickstand: false,
        colour: M.cargo,
        ..Bike::default()
    }
}

fn ebike() -> Bike {
    Bike {
        r: 0.335,
        k: 1.08,
        tyre: Some(0.032),
        rear: -0.57,
        front: 0.57,
        spokes: 28,
        segments: (36, 12),
        rim_segments: 32,
        tyre_minor: 5,
        bb: (-0.07, 0.29),
        sc: (-0.21, 0.62),
        hb: (0.37, 0.72),
        ht: (0.34, 0.81),
        crown: Some((0.39, 0.68)),
        head: ((0.39, 0.67), (0.31, 0.86)),
        fork_mid: (0.47, 0.46),
        tubes: (0.021, 0.018, 0.023),
        step: Some((
            vec![(0.34, 0.81), (0.22, 0.61), (0.04, 0.48), (-0.14, 0.45)],
            vec![(0.37, 0.72), (0.26, 0.51), (0.09, 0.35), (-0.07, 0.29)],
        )),
        stay_dz: 0.042,
        guards: vec![(15.0, 125.0), (35.0, 172.0)],
        post: (-0.30, 0.83),
        saddle: (-0.38, 0.835),
        saddle_rows: vec![
            (0.0, 0.038, 0.021),
            (0.032, 0.098, 0.036),
            (0.086, 0.092, 0.036),
            (0.15, 0.044, 0.031),
            (0.215, 0.019, 0.023),
            (0.248, 0.01, 0.012),
        ],
        stem: ((0.31, 0.86), (0.31, 1.0)),
        bars: vec![
            (0.20, 0.985, -0.30),
            (0.27, 0.99, -0.22),
            (0.33, 0.995, -0.12),
            (0.32, 0.995, -0.03),
            (0.32, 0.995, 0.03),
            (0.33, 0.995, 0.12),
            (0.27, 0.99, 0.22),
            (0.20, 0.985, 0.30),
        ],
        bend: 0.05,
        grips: vec![
            ((0.27, 0.99, -0.22), (0.20, 0.985, -0.30)),
            ((0.27, 0.99, 0.22), (0.20, 0.985, 0.30)),
        ],
        grip_r: 0.018,
        bell: Some((0.33, 1.005, 0.10)),
        ring: 0.082,
        ring_dz: 0.05,
        cog: 0.04,
        chain_drop: 0.48,
        crank: (0.17, 0.0),
        crank_dz: 0.08,
        pedal_dz: 0.05,
        pedal: (0.09, 0.022, 0.07),
        drive: -1.0,
        kickstand: false,
        colour: M.ebike,
        ..Bike::default()
    }
}

/// The bottom bracket's shell, across the frame where its tubes meet.
fn bb_shell(s: &Bike, width: f64) {
    fcyl!(
        F,
        numbered("bb_shell"),
        0.024 * s.k,
        width,
        (s.bb.0, s.bb.1, s.z),
        s.colour,
        rot = (90, 0, 0),
        verts = low(10, 5)
    );
}

/// The cranks' axle through the bottom bracket (in the cranks, which turn about it).
fn spindle(s: &Bike) {
    fcyl!(
        F,
        numbered("spindle"),
        0.009 * s.k,
        2.0 * s.crank_dz,
        (s.bb.0, s.bb.1, s.z),
        M.rim,
        rot = (90, 0, 0),
        verts = low(8, 4)
    );
}

/// The chain's top run's height at game x.
fn chain_top(s: &Bike, x: f64) -> f64 {
    let (y0, y1) = (s.bb.1 + s.ring, s.r + s.ring * s.chain_drop);
    y0 + (x - s.bb.0) * (y1 - y0) / (s.rear - s.bb.0)
}

/// The city bike's own things: a chain guard, a wicker basket on the bars with a lamp, a carrier with a red
/// light, the centre stand's plate.
fn city_extras(s: &Bike) {
    let (z, bx, by) = (s.z, s.bb.0, s.bb.1);
    bb_shell(s, 0.075);
    // the chain guard over the top of the chain ring and the top run, between the ring and the crank
    let (ro, ri) = (s.ring + 0.028, s.ring - 0.012);
    let xe = s.rear + 0.075;
    let step = low(15, 30);
    let mut out: Vec<(f64, f64)> = (-30..91)
        .step_by(step)
        .map(|a| (bx + ro * cosd(a as f64), by + ro * sind(a as f64)))
        .collect();
    out.push((xe, chain_top(s, xe) + 0.028));
    out.push((xe, chain_top(s, xe) - 0.012));
    let mut a = 90;
    while a > -31 {
        out.push((bx + ri * cosd(a as f64), by + ri * sind(a as f64)));
        a -= step as i32;
    }
    let d = s.drive;
    fextrude!(
        F,
        "chain_guard",
        &out,
        0.004,
        (0, 0, z + 0.064 * d),
        s.colour
    );
    rod(
        "guard_stay",
        (xe + 0.01, chain_top(s, xe) - 0.005, z + 0.064 * d),
        (xe + 0.02, chain_top(s, xe) - 0.03, z + 0.03 * d),
        0.004,
        M.rim,
        4,
    );
    // the basket, wicker, on the bars' clamp, on two stays down to the fork ends; a lamp on its front
    let (cx, cy0, cy1, hx, hz) = (0.515, 0.735, 0.955, 0.135, 0.17);
    let p = F.at(cx, 0, z);
    let (px, pz) = (p.x, p.y);
    let n: usize = low(24, 8);
    loft!(
        "basket",
        &[
            ring!(
                hx * 0.92,
                hz * 0.92,
                z = cy0,
                verts = n,
                square = 4,
                at = (px, pz)
            ),
            ring!(hx, hz, z = cy1, verts = n, square = 4, at = (px, pz)),
            ring!(
                hx - 0.012,
                hz - 0.012,
                z = cy1,
                verts = n,
                square = 4,
                at = (px, pz)
            ),
            ring!(
                hx * 0.92 - 0.012,
                hz * 0.92 - 0.012,
                z = cy0 + 0.015,
                verts = n,
                square = 4,
                at = (px, pz)
            )
        ],
        material = M.basket
    );
    let rim: Vec<(f64, f64, f64)> =
        rounded_rect(2.0 * hx + 0.004, 2.0 * hz + 0.004, 0.05, low(3, 1))
            .iter()
            .map(|&(u, v)| (cx + u, cy1, z + v))
            .collect();
    fsweep!(
        F,
        "basket_rim",
        &rim,
        0.009,
        M.basket,
        verts = low(6, 4),
        closed = true
    );
    if !is_low() {
        // woven bands round it, the weave's ribs between them
        for f in [0.3, 0.62] {
            let sc_ = 0.92 + 0.08 * f + 0.004;
            let band: Vec<(f64, f64, f64)> = rounded_rect(2.0 * hx * sc_, 2.0 * hz * sc_, 0.045, 3)
                .iter()
                .map(|&(u, v)| (cx + u, cy0 + (cy1 - cy0) * f, z + v))
                .collect();
            fsweep!(
                F,
                numbered("basket_band"),
                &band,
                0.006,
                M.basket_dark,
                verts = 4,
                closed = true
            );
        }
        for i in 0..n {
            let a = 360.0 * (i as f64 + 0.5) / n as f64;
            let (c, s_) = (cosd(a), sind(a));
            let e = 2.0 / 4.0; // the rings' squareness (ring square=4)
            let (u, v) = (pow(c.abs(), e).copysign(c), pow(s_.abs(), e).copysign(s_));
            rod(
                &numbered("basket_rib"),
                (cx + hx * 0.925 * u, cy0 + 0.01, z + hz * 0.925 * v),
                (cx + hx * 1.003 * u, cy1 - 0.012, z + hz * 1.003 * v),
                0.0035,
                M.basket_dark,
                3,
            );
        }
    }
    for sg in [-1.0, 1.0] {
        rod(
            &numbered("basket_stay"),
            (cx + 0.08, cy0 + 0.005, z + sg * 0.07),
            (s.front, s.r, z + sg * 0.062),
            0.005,
            M.rim,
            low(6, 4),
        );
    }
    fbox!(
        F,
        "basket_bracket",
        0.075,
        0.03,
        0.06,
        (0.35, 0.94, z),
        M.rim,
        bevel = low(0.004, 0.0),
        bevel_segments = 1
    );
    fcyl!(
        F,
        "lamp",
        0.026,
        0.045,
        (cx + hx + 0.022, 0.86, z),
        M.rim,
        rot = (0, 90, 0),
        verts = low(12, 6)
    );
    fcyl!(
        F,
        "lamp_glass",
        0.022,
        0.006,
        (cx + hx + 0.046, 0.86, z),
        M.lamp,
        rot = (0, 90, 0),
        verts = low(12, 6)
    );
    // the carrier over the back wheel, on stays to the axle and the seat stays; a red light at its back
    let (ch, x0, x1) = (0.735, -0.3, -0.8);
    for sg in [-1.0, 1.0] {
        let dz = sg * 0.065;
        bent(
            &numbered("carrier_rail"),
            &[
                (x0, ch - 0.02, z + dz * 0.5),
                (x0 - 0.04, ch, z + dz),
                (x1, ch, z + dz),
            ],
            0.0065,
            s.colour,
            0.03,
            low(6, 4),
            3,
        );
        rod(
            &numbered("carrier_strut"),
            (x0, ch - 0.02, z + dz * 0.5),
            (-0.245, 0.585, z + sg * 0.016),
            0.005,
            s.colour,
            low(6, 4),
        );
        for xa in [x1 + 0.02, -0.64] {
            rod(
                &numbered("carrier_stay"),
                (xa, ch, z + dz),
                (s.rear - 0.01, s.r + 0.01, z + sg * 0.046),
                0.0055,
                s.colour,
                low(6, 4),
            );
        }
    }
    for xa in [-0.42, -0.56, x1] {
        rod(
            &numbered("carrier_bar"),
            (xa, ch, z - 0.065),
            (xa, ch, z + 0.065),
            0.005,
            s.colour,
            low(6, 4),
        );
    }
    fbox!(
        F,
        "rear_light",
        0.025,
        0.035,
        0.07,
        (x1 - 0.014, ch - 0.025, z),
        M.reflector,
        bevel = low(0.006, 0.0),
        bevel_segments = 1
    );
    // the centre stand's plate under the chainstays
    fbox!(
        F,
        "stand_plate",
        0.05,
        0.016,
        0.06,
        (CITY_STAND.0, CITY_STAND.1 + 0.017, z),
        s.colour,
        bevel = low(0.003, 0.0),
        bevel_segments = 1
    );
}

/// A double-leg centre stand, down: a yoke on its pivot, two legs splayed to rubber feet either side.
fn city_stand(s: &Bike) {
    let (px, py) = CITY_STAND;
    let z = s.z;
    fcyl!(
        F,
        "stand_yoke",
        0.014,
        0.07,
        (px, py, z),
        M.rim,
        rot = (90, 0, 0),
        verts = low(10, 5)
    );
    for sg in [-1.0, 1.0] {
        fsweep!(
            F,
            numbered("stand_leg"),
            &[
                (px, py, z + sg * 0.03),
                (px + 0.02, 0.16, z + sg * 0.1),
                (px + 0.05, 0.016, z + sg * 0.155)
            ],
            0.011,
            M.rim,
            verts = low(8, 4),
            resolution = low(3, 1),
            scale = [1.0, 0.9, 0.8]
        );
        fbox!(
            F,
            numbered("stand_foot"),
            0.055,
            0.016,
            0.04,
            (px + 0.052, 0.008, z + sg * 0.158),
            M.tyre,
            bevel = low(0.004, 0.0),
            bevel_segments = 1
        );
    }
}

/// The kids' bike's own things: a chain case, a brace across the bars, streamers from the grips' ends, the
/// stabilisers (their brackets, their little wheels on the floor either side of the back wheel).
fn kid_extras(s: &Bike) {
    let (z, bx, by) = (s.z, s.bb.0, s.bb.1);
    bb_shell(s, 0.06);
    // the chain case: round the ring and the cog, between the ring and the crank
    let (ro, rc) = (s.ring + 0.018, s.cog + 0.018);
    let step = low(20, 40);
    let mut case: Vec<(f64, f64)> = (-80..81)
        .step_by(step)
        .map(|a| (bx + ro * cosd(a as f64), by + ro * sind(a as f64)))
        .collect();
    case.extend(
        (100..261)
            .step_by(step)
            .map(|a| (s.rear + rc * cosd(a as f64), s.r + rc * sind(a as f64))),
    );
    fextrude!(
        F,
        "chain_case",
        &case,
        0.004,
        (0, 0, z + (s.ring_dz + 0.012) * s.drive),
        M.kid_trim
    );
    rod(
        "bar_brace",
        (0.153, 0.615, z - 0.105),
        (0.153, 0.615, z + 0.105),
        0.007,
        M.rim,
        low(6, 4),
    );
    // streamers: four ribbons hanging from each grip's end, a cap over it
    let colours = [M.streamer_pink, M.streamer_yellow, M.streamer_blue];
    for sg in [-1, 1] {
        let sf = sg as f64;
        let (gx, gh, gz) = (0.16, 0.66, z + sf * 0.268);
        fcyl!(
            F,
            numbered("grip_cap"),
            0.019,
            0.012,
            (gx, gh, gz),
            M.streamer_pink,
            rot = (90, 0, 0),
            verts = low(10, 5)
        );
        for j in 0..3 {
            let a = (120 * j + 30 * sg) as f64;
            let (ox, oz) = (0.008 * cosd(a), 0.008 * sind(a));
            let pts = [
                (gx + ox, gh - 0.01, gz + sf * 0.004 + oz),
                (gx + ox - 0.012, gh - 0.06, gz + sf * 0.006 + oz * 1.4),
                (gx + ox - 0.03, gh - 0.11, gz + sf * 0.008 + oz * 1.8),
                (gx + ox - 0.05, gh - 0.15, gz + sf * 0.01 + oz * 2.0),
            ];
            fsweep!(
                F,
                numbered("streamer"),
                &pts,
                0,
                colours[j as usize],
                profile = vec![
                    (-0.004, -0.0006),
                    (0.004, -0.0006),
                    (0.004, 0.0006),
                    (-0.004, 0.0006)
                ],
                up = (1, 0, 0)
            );
        }
    }
    // the stabilisers: a bracket from the back axle out and down to each little wheel, standing on the floor
    let (sr, sx) = (0.06, s.rear - 0.03);
    for sg in [-1.0, 1.0] {
        let sz = z + sg * 0.175;
        fsweep!(
            F,
            numbered("stabiliser"),
            &[
                (s.rear, s.r, z + sg * 0.045),
                (s.rear - 0.012, 0.12, z + sg * 0.13),
                (sx, sr, sz - sg * 0.018)
            ],
            0.0075,
            M.rim,
            verts = low(6, 4)
        );
        torus!(
            numbered("stabiliser_tyre"),
            sr - 0.014,
            0.014,
            at = F.at(sx, sr, sz),
            material = M.tyre,
            rot = (90, 0, 0),
            major_segments = low(16, 8),
            minor_segments = low(5, 4)
        );
        fcyl!(
            F,
            numbered("stabiliser_hub"),
            sr - 0.022,
            0.02,
            (sx, sr, sz),
            M.trim,
            rot = (90, 0, 0),
            verts = low(12, 6)
        );
        fcyl!(
            F,
            numbered("stabiliser_axle"),
            0.007,
            0.04,
            (sx, sr, sz),
            M.rim,
            rot = (90, 0, 0),
            verts = low(6, 4)
        );
    }
}

/// The mountain bike's own things: a suspension fork's crown and stanchions, and a disc brake at each wheel.
fn mtb_extras(s: &Bike) {
    let z = s.z;
    let (cx, cy) = s.crown.unwrap_or(s.hb);
    fbox!(
        F,
        "fork_crown",
        0.19,
        0.05,
        0.11,
        (cx, cy, z),
        M.motor,
        bevel = low(0.008, 0.0),
        bevel_segments = 1
    );
    for sg in [-1.0, 1.0] {
        let dz = sg * 0.055;
        rod(
            "stanchion",
            (cx, cy - 0.01, z + dz),
            (s.fork_mid.0 - 0.02, s.fork_mid.1 + 0.02, z + dz),
            0.017,
            M.rim,
            low(8, 4),
        );
        rod(
            "fork_lower",
            (s.fork_mid.0 - 0.01, s.fork_mid.1 + 0.01, z + dz),
            (s.front, s.r, z + dz),
            0.026,
            M.tyre,
            low(8, 4),
        );
    }
    for x in [s.front, s.rear] {
        fcyl!(
            F,
            "disc",
            0.085,
            0.004,
            (x, s.r, z + 0.052 * s.drive),
            M.rim,
            rot = (90, 0, 0),
            verts = low(16, 8)
        );
        fbox!(
            F,
            "caliper",
            0.035,
            0.09,
            0.03,
            (x + 0.02, s.r + 0.06, z + 0.052 * s.drive),
            M.motor,
            bevel = low(0.005, 0.0),
            bevel_segments = 1
        );
    }
}

/// The road bike's own things: a bottle on the down tube, a brake caliper at each wheel.
fn road_extras(s: &Bike) {
    let z = s.z;
    bb_shell(s, 0.07);
    board(
        "bottle",
        (0.19, 0.49, z),
        (0.12, 0.37, z),
        0.055,
        0.055,
        M.lamp,
        (0, 1, 0),
        low(0.012, 0.0),
    );
    fcyl!(
        F,
        "bottle_cap",
        0.018,
        0.02,
        (0.115, 0.365, z),
        M.rim,
        rot = (0, 90, 0),
        verts = low(8, 4)
    );
    let crown = s.crown.unwrap_or(s.hb);
    fbox!(
        F,
        "front_brake",
        0.03,
        0.06,
        0.05,
        (crown.0 + 0.02, crown.1 - 0.02, z),
        M.motor,
        bevel = low(0.004, 0.0),
        bevel_segments = 1
    );
    fbox!(
        F,
        "rear_brake",
        0.03,
        0.06,
        0.05,
        (-0.45, 0.48, z),
        M.motor,
        bevel = low(0.004, 0.0),
        bevel_segments = 1
    );
}

/// The cargo bike's own things: the long box between the rider and the front wheel, with a padded bench, a
/// wooden floor and rails, a lamp on its front and a red light at the back.
fn cargo_extras(s: &Bike) {
    let z = s.z;
    bb_shell(s, 0.08);
    let (x0, x1, y0, y1, hz) = (0.42, 1.00, 0.30, 0.66, 0.30); // the box's ends (x), its floor and top rail (y), its half depth (z)
    fbox!(
        F,
        "box_floor",
        x1 - x0,
        0.025,
        2.0 * hz,
        ((x0 + x1) / 2.0, y0, z),
        M.box_wood,
        bevel = low(0.006, 0.0),
        bevel_segments = 1
    );
    for sg in [-1.0, 1.0] {
        fbox!(
            F,
            "box_side",
            x1 - x0,
            y1 - y0,
            0.03,
            ((x0 + x1) / 2.0, (y0 + y1) / 2.0, z + sg * (hz - 0.015)),
            s.colour,
            bevel = low(0.008, 0.0),
            bevel_segments = 1
        );
        rod(
            "box_rail",
            (x0, y1, z + sg * hz),
            (x1, y1, z + sg * hz),
            0.012,
            M.box_wood,
            6,
        );
    }
    fbox!(
        F,
        "box_front",
        0.03,
        y1 - y0,
        2.0 * hz,
        (x1, (y0 + y1) / 2.0, z),
        s.colour,
        bevel = low(0.008, 0.0),
        bevel_segments = 1
    );
    fbox!(
        F,
        "box_back",
        0.03,
        y1 - y0,
        2.0 * hz,
        (x0, (y0 + y1) / 2.0, z),
        s.colour,
        bevel = low(0.008, 0.0),
        bevel_segments = 1
    );
    fbox!(
        F,
        "box_pad",
        x1 - x0 - 0.09,
        0.05,
        2.0 * hz - 0.09,
        ((x0 + x1) / 2.0, y0 + 0.05, z),
        M.saddle,
        bevel = low(0.012, 0.0),
        bevel_segments = 1
    );
    fcyl!(
        F,
        "lamp",
        0.028,
        0.045,
        (x1 + 0.02, 0.56, z),
        M.rim,
        rot = (0, 90, 0),
        verts = low(12, 6)
    );
    fcyl!(
        F,
        "lamp_glass",
        0.024,
        0.006,
        (x1 + 0.044, 0.56, z),
        M.lamp,
        rot = (0, 90, 0),
        verts = low(12, 6)
    );
    fbox!(
        F,
        "rear_light",
        0.02,
        0.03,
        0.06,
        (-0.62, 0.70, z),
        M.reflector,
        bevel = low(0.004, 0.0),
        bevel_segments = 1
    );
}

/// The e-bike's own things: the battery in the down tube, the motor at the bottom bracket, a display on the
/// bars, built-in lights, and a carrier over the back wheel.
fn ebike_extras(s: &Bike) {
    let z = s.z;
    bb_shell(s, 0.08);
    fcyl!(
        F,
        "motor",
        0.06,
        0.10,
        (s.bb.0, s.bb.1, z),
        M.motor,
        rot = (90, 0, 0),
        verts = low(14, 7)
    );
    board(
        "battery",
        (0.26, 0.55, z),
        (0.02, 0.37, z),
        0.13,
        0.085,
        M.battery,
        (0, 1, 0),
        low(0.01, 0.0),
    );
    fbox!(
        F,
        "display",
        0.085,
        0.03,
        0.05,
        (0.33, 1.005, z),
        M.screen,
        bevel = low(0.006, 0.0),
        bevel_segments = 1
    );
    let crown = s.crown.unwrap_or(s.hb);
    fbox!(
        F,
        "head_light",
        0.06,
        0.04,
        0.06,
        (crown.0 + 0.03, crown.1 + 0.02, z),
        M.lamp,
        bevel = low(0.008, 0.0),
        bevel_segments = 1
    );
    fbox!(
        F,
        "tail_light",
        0.02,
        0.03,
        0.06,
        (-0.56, 0.73, z),
        M.reflector,
        bevel = low(0.004, 0.0),
        bevel_segments = 1
    );
    let ch = 0.76;
    for sg in [-1.0, 1.0] {
        let dz = sg * 0.07;
        rod(
            "rack_rail",
            (-0.34, ch, z + dz),
            (-0.72, ch, z + dz),
            0.007,
            M.rim,
            low(6, 4),
        );
        rod(
            "rack_stay",
            (-0.68, ch, z + dz),
            (-0.56, s.r + 0.01, z + sg * 0.05),
            0.006,
            M.rim,
            low(6, 4),
        );
    }
    for x in [-0.40, -0.56, -0.70] {
        rod(
            "rack_bar",
            (x, ch, z - 0.07),
            (x, ch, z + 0.07),
            0.005,
            M.rim,
            low(6, 4),
        );
    }
}

/// A bike `S` in its nodes (the note above): `extras` and `stand` (its pivot at game (x, height) `pivot`) build what
/// this one has of its own.
fn ride(
    name_: &str,
    s: &Bike,
    extras: fn(&Bike),
    stand: Option<fn(&Bike)>,
    pivot: (f64, f64),
    centre: bool,
) -> Vec<Obj> {
    let rear = new_parts(|| wheel(s.rear, s.z, s));
    let (frame, front, cranks) = bike(s);
    let more = new_parts(|| extras(s));
    let cranks_more = new_parts(|| spindle(s));
    let kick = new_parts(|| {
        if let Some(stand) = stand {
            stand(s);
        }
    });
    let how = Settle {
        centre,
        floor: true,
    };
    if is_low() {
        // from afar: parked, all one still part
        return settle_with(&[join_all(name_)], how);
    }
    let mut body = frame;
    body.extend(more);
    join(&body, "frame", Some(P3::from((0, 0, 0))));
    join(&front, "wheel_front", Some(F.at(s.front, s.r, s.z)));
    join(&rear, "wheel_rear", Some(F.at(s.rear, s.r, s.z)));
    let mut turning = cranks;
    turning.extend(cranks_more);
    join(&turning, "cranks", Some(F.at(s.bb.0, s.bb.1, s.z)));
    if !kick.is_empty() {
        join(&kick, "stand", Some(F.at(pivot.0, pivot.1, s.z)));
    }
    settle_with(&meshes(), how)
}

pub fn bicycle() -> Vec<Obj> {
    ride(
        "bicycle",
        &city(),
        city_extras,
        Some(city_stand),
        CITY_STAND,
        false,
    )
}

pub fn mountain_bike() -> Vec<Obj> {
    ride("mountainBike", &mtb(), mtb_extras, None, (0.0, 0.0), false)
}

pub fn road_bike() -> Vec<Obj> {
    ride("roadBike", &road(), road_extras, None, (0.0, 0.0), false)
}

pub fn cargo_bike() -> Vec<Obj> {
    ride("cargoBike", &cargo(), cargo_extras, None, (0.0, 0.0), true)
}

pub fn e_bike() -> Vec<Obj> {
    ride("eBike", &ebike(), ebike_extras, None, (0.0, 0.0), false)
}

pub fn bicycle_child() -> Vec<Obj> {
    ride("bicycleChild", &kid(), kid_extras, None, (0.0, 0.0), false)
}

// The e-scooter: a low deck between two small solid wheels, a raked stem from the front up to T-bars.
const SC_R: f64 = 0.105; // its wheels' radius
const SC_REAR: f64 = -0.36;
const SC_FRONT: f64 = 0.37;
const SC_DECK: (f64, f64, f64) = (-0.25, 0.25, 0.125); // the deck's ends (x) and its top (with the grip tape)
const SC_TOP: (f64, f64) = (0.284, 1.015); // the stem's top, under the bars
const SC_STAND: (f64, f64, f64) = (0.1, 0.072, 0.07); // the side stand's pivot, under the deck's left edge (+z: the rider's left)

/// The point of the steering axis (front axle up to the stem's top) at height `h`: (x, h).
fn sc_axis(h: f64) -> (f64, f64) {
    let t = (h - SC_R) / (SC_TOP.1 - SC_R);
    (SC_FRONT + (SC_TOP.0 - SC_FRONT) * t, h)
}

/// A scooter wheel at axle (x, SC_R, 0): a solid tyre, a rim with five spokes round a hub, or the hub motor's drum
/// filling it (`motor`); the back one's brake disc on its left (+z).
fn sc_wheel(x: f64, motor: bool) {
    let y = SC_R;
    torus!(
        numbered("tyre"),
        SC_R - 0.025,
        0.025,
        at = F.at(x, y, 0),
        material = M.tyre,
        rot = (90, 0, 0),
        major_segments = low(32, 12),
        minor_segments = low(8, 4)
    );
    let rim = SC_R - 0.048;
    if motor {
        // the motor: a drum as wide as the tyre, a lip either side
        revolve!(
            numbered("motor"),
            &[
                (0.0, -0.034),
                (0.03, -0.034),
                (0.046, -0.03),
                (0.054, -0.024),
                (rim + 0.004, -0.02),
                (rim + 0.004, 0.02),
                (0.054, 0.024),
                (0.046, 0.03),
                (0.03, 0.034),
                (0.0, 0.034)
            ],
            at = F.at(x, y, 0),
            material = M.scooter_metal,
            segments = low(20, 8),
            rot = (90, 0, 0)
        );
        if !is_low() {
            for s in [-1.0, 1.0] {
                fcyl!(
                    F,
                    numbered("motor_cap"),
                    0.022,
                    0.006,
                    (x, y, s * 0.036),
                    M.scooter_trim,
                    rot = (90, 0, 0),
                    verts = 10
                );
            }
        }
    } else {
        revolve!(
            numbered("rim"),
            &[
                (rim - 0.012, -0.02),
                (rim + 0.004, -0.02),
                (rim + 0.004, 0.02),
                (rim - 0.012, 0.02)
            ],
            at = F.at(x, y, 0),
            material = M.scooter_metal,
            segments = low(24, 8),
            rot = (90, 0, 0)
        );
        fcyl!(
            F,
            numbered("hub"),
            0.02,
            0.05,
            (x, y, 0),
            M.scooter_metal,
            rot = (90, 0, 0),
            verts = low(12, 6)
        );
        if !is_low() {
            for i in 0..5 {
                let a = 72 * i + 18;
                let m = 0.5 * (0.02 + rim - 0.012);
                fbox!(
                    F,
                    numbered("rim_spoke"),
                    rim - 0.012 - 0.015,
                    0.012,
                    0.016,
                    (x + m * cosd(a as f64), y + m * sind(a as f64), 0),
                    M.scooter_metal,
                    rot = (0, a, 0)
                );
            }
        }
        fcyl!(
            F,
            numbered("brake_disc"),
            0.05,
            0.003,
            (x, y, 0.03),
            M.rim,
            rot = (90, 0, 0),
            verts = low(16, 8)
        );
    }
    fcyl!(
        F,
        numbered("axle"),
        0.009,
        0.1,
        (x, y, 0),
        M.rim,
        rot = (90, 0, 0),
        verts = low(8, 4)
    );
}

/// The scooter's still body: deck and grip tape, the back fork, the fenders, the neck and the head tube, the fork,
/// the stem with its latch, the bars and grips, a brake lever and a throttle, a bell, the display, the lamps.
fn sc_frame() {
    let (x0, x1, top) = SC_DECK;
    let l = x1 - x0;
    // the deck: a rounded slab, its edges rounded, grip tape on top, a trim strip along each side
    fextrude!(
        F,
        "deck",
        &rounded_rect(l, 0.165, 0.05, low(4, 1)),
        0.058,
        ((x0 + x1) / 2.0, top - 0.004 - 0.029, 0),
        M.scooter,
        plane = Facing::Top,
        bevel = low(0.012, 0.0),
        bevel_segments = low(2, 1)
    );
    fextrude!(
        F,
        "grip_tape",
        &rounded_rect(l - 0.04, 0.135, 0.035, low(4, 1)),
        0.004,
        ((x0 + x1) / 2.0, top - 0.002, 0),
        M.grip_tape,
        plane = Facing::Top
    );
    if !is_low() {
        for s in [-1.0, 1.0] {
            fbox!(
                F,
                numbered("deck_trim"),
                l - 0.08,
                0.008,
                0.004,
                ((x0 + x1) / 2.0, top - 0.03, s * 0.0835),
                M.scooter_trim
            );
        }
    }
    // the back fork: a plate either side from the deck's tail to the axle; the fender over the wheel, a light at its end
    for s in [-1.0, 1.0] {
        board(
            &numbered("rear_fork"),
            (x0 + 0.03, top - 0.03, s * 0.042),
            (SC_REAR, SC_R, s * 0.042),
            0.04,
            0.008,
            M.scooter,
            (0, 0, 1),
            low(0.003, 0.0),
        );
    }
    let prof = vec![
        (-0.032, 0.0),
        (-0.028, 0.008),
        (0.0, 0.011),
        (0.028, 0.008),
        (0.032, 0.0),
        (0.0, 0.004),
    ];
    for (x, (a0, a1)) in [(SC_REAR, (28.0, 152.0)), (SC_FRONT, (45.0, 140.0))] {
        fsweep!(
            F,
            numbered("fender"),
            &arc_points((x, SC_R, 0.0), SC_R + 0.016, a0, a1, low(10, 4), true),
            0,
            M.scooter,
            profile = &prof,
            up = (0, 1, 0)
        );
    }
    let (ex, ey) = (
        SC_REAR + (SC_R + 0.016) * cosd(152.0),
        SC_R + (SC_R + 0.016) * sind(152.0),
    );
    fbox!(
        F,
        "tail_light",
        0.012,
        0.022,
        0.04,
        (ex - 0.004, ey + 0.004, 0),
        M.reflector,
        bevel = low(0.003, 0.0),
        bevel_segments = 1
    );
    // the neck from the deck's front up to the head tube; the head tube, the fork blades to the axle
    let (hx0, hy0) = sc_axis(0.2);
    let (hx1, hy1) = sc_axis(0.32);
    fsweep!(
        F,
        "neck",
        &[
            (x1 - 0.06, top - 0.03, 0),
            (x1 + 0.03, top - 0.015, 0),
            (hx0 - 0.01, hy0 + 0.01, 0)
        ],
        0.024,
        M.scooter,
        verts = low(10, 6),
        resolution = low(3, 1),
        scale = [1.0, 0.95, 0.85]
    );
    fsweep!(
        F,
        "head_tube",
        &[(hx0, hy0, 0), (hx1, hy1, 0)],
        0.026,
        M.scooter,
        verts = low(12, 6)
    );
    let (cx, cy) = sc_axis(0.19);
    for s in [-1.0, 1.0] {
        fsweep!(
            F,
            numbered("fork"),
            &[(cx, cy, s * 0.02), (SC_FRONT, SC_R, s * 0.042)],
            0.009,
            M.scooter,
            verts = low(6, 4),
            scale = [1.2, 0.9]
        );
    }
    fbox!(
        F,
        "fork_crown",
        0.04,
        0.025,
        0.1,
        (cx, cy, 0),
        M.scooter,
        bevel = low(0.006, 0.0),
        bevel_segments = 1
    );
    // the stem: the folding latch over the head tube, then the stem up to the bars
    let (lx, ly) = sc_axis(0.36);
    let mid = sc_axis(0.6);
    fsweep!(
        F,
        "stem",
        &[(hx1, hy1, 0), (mid.0, mid.1, 0), (SC_TOP.0, SC_TOP.1, 0)],
        0.019,
        M.scooter,
        verts = low(12, 6),
        scale = [1.0, 0.95, 0.88]
    );
    fbox!(
        F,
        "latch",
        0.05,
        0.07,
        0.05,
        (lx + 0.012, ly, 0),
        M.scooter_trim,
        rot = (0, 5, 0),
        bevel = low(0.01, 0.0),
        bevel_segments = low(2, 1)
    );
    fcyl!(
        F,
        "latch_hinge",
        0.014,
        0.07,
        (lx + 0.02, ly - 0.03, 0),
        M.rim,
        rot = (90, 0, 0),
        verts = low(8, 4)
    );
    // the T-bars and grips; a brake lever on the left (+z), a thumb throttle on the right, a bell
    let (bx, by) = (SC_TOP.0, SC_TOP.1 + 0.015);
    fsweep!(
        F,
        "bars",
        &[(bx, by, -0.29), (bx, by, 0.29)],
        0.012,
        M.scooter,
        verts = low(10, 5)
    );
    fbox!(
        F,
        "bar_clamp",
        0.04,
        0.04,
        0.06,
        (bx, by - 0.005, 0),
        M.scooter,
        bevel = low(0.008, 0.0),
        bevel_segments = 1
    );
    for s in [-1.0, 1.0] {
        fsweep!(
            F,
            numbered("grip"),
            &[(bx, by, s * 0.17), (bx, by, s * 0.29)],
            0.017,
            M.grip_tape,
            verts = low(10, 5),
            scale = [1.0, 1.08]
        );
        fcyl!(
            F,
            numbered("grip_end"),
            0.018,
            0.008,
            (bx, by, s * 0.294),
            M.scooter_trim,
            rot = (90, 0, 0),
            verts = low(10, 5)
        );
    }
    fsweep!(
        F,
        "brake_lever",
        &[
            (bx + 0.01, by, 0.15),
            (bx + 0.035, by + 0.002, 0.2),
            (bx + 0.04, by, 0.26)
        ],
        0.005,
        M.rim,
        verts = low(6, 4),
        resolution = low(2, 1)
    );
    fbox!(
        F,
        "brake_mount",
        0.03,
        0.03,
        0.025,
        (bx + 0.006, by, 0.155),
        M.scooter,
        bevel = low(0.004, 0.0),
        bevel_segments = 1
    );
    fbox!(
        F,
        "throttle",
        0.035,
        0.012,
        0.025,
        (bx - 0.018, by + 0.004, -0.155),
        M.scooter,
        bevel = low(0.004, 0.0),
        bevel_segments = 1
    );
    fcyl!(
        F,
        "bell",
        0.015,
        0.012,
        (bx, by + 0.016, 0.11),
        M.rim,
        verts = low(8, 5)
    );
    // the display on the bars' clamp, tilted up to the rider; the headlight on the stem's front
    soft!(
        F,
        "display",
        0.06,
        0.022,
        0.075,
        (bx - 0.012, by + 0.024, 0),
        M.scooter,
        round = 0.008,
        flat = 1,
        round_segments = low(2, 1)
    );
    fbox!(
        F,
        "screen",
        0.04,
        0.003,
        0.05,
        (bx - 0.014, by + 0.0355, 0),
        M.screen
    );
    let (hx, hy) = sc_axis(0.9);
    fcyl!(
        F,
        "headlight",
        0.022,
        0.03,
        (hx + 0.03, hy, 0),
        M.scooter,
        rot = (0, 90, 0),
        verts = low(12, 6)
    );
    fcyl!(
        F,
        "headlight_glass",
        0.018,
        0.004,
        (hx + 0.046, hy, 0),
        M.lamp,
        rot = (0, 90, 0),
        verts = low(12, 6)
    );
    fbox!(
        F,
        "headlight_mount",
        0.03,
        0.02,
        0.025,
        (hx + 0.012, hy, 0),
        M.scooter
    );
    if !is_low() {
        // the motor's cable down the fork, the brake's down the stem
        fsweep!(
            F,
            "motor_cable",
            &[
                (SC_FRONT - 0.01, SC_R + 0.02, 0.05),
                (cx + 0.02, cy, 0.04),
                (hx1 + 0.03, hy1, 0.02),
                (hx1 + 0.02, hy1 + 0.1, 0.0)
            ],
            0.004,
            M.tyre,
            verts = 4,
            resolution = 2
        );
        let end = sc_axis(0.7);
        fsweep!(
            F,
            "brake_cable",
            &[
                (bx + 0.01, by - 0.01, 0.15),
                (bx + 0.05, by - 0.12, 0.04),
                (end.0, end.1, 0.0)
            ],
            0.003,
            M.tyre,
            verts = 4,
            resolution = 2
        );
    }
}

/// The side stand, down: a short leg from its pivot under the deck's left edge out and back to a foot.
fn sc_stand() {
    let (px, py, pz) = SC_STAND;
    fbox!(
        F,
        "stand_mount",
        0.04,
        0.02,
        0.02,
        (px, py + 0.006, pz - 0.008),
        M.scooter,
        bevel = low(0.004, 0.0),
        bevel_segments = 1
    );
    fsweep!(
        F,
        "stand_leg",
        &[
            (px, py, pz),
            (px - 0.07, 0.03, pz + 0.07),
            (px - 0.1, 0.012, pz + 0.105)
        ],
        0.008,
        M.scooter,
        verts = low(8, 4),
        resolution = low(2, 1),
        scale = [1.0, 0.9, 0.85]
    );
    fbox!(
        F,
        "stand_foot",
        0.035,
        0.012,
        0.03,
        (px - 0.104, 0.006, pz + 0.11),
        M.tyre,
        bevel = low(0.003, 0.0),
        bevel_segments = 1
    );
}

pub fn e_scooter() -> Vec<Obj> {
    let frame = new_parts(sc_frame);
    let front = new_parts(|| sc_wheel(SC_FRONT, true));
    let rear = new_parts(|| sc_wheel(SC_REAR, false));
    let kick = new_parts(sc_stand);
    let how = Settle {
        centre: false,
        floor: true,
    };
    if is_low() {
        return settle_with(&[join_all("eScooter")], how);
    }
    join(&frame, "frame", Some(P3::from((0, 0, 0))));
    join(&front, "wheel_front", Some(F.at(SC_FRONT, SC_R, 0)));
    join(&rear, "wheel_rear", Some(F.at(SC_REAR, SC_R, 0)));
    join(
        &kick,
        "stand",
        Some(F.at(SC_STAND.0, SC_STAND.1, SC_STAND.2)),
    );
    settle_with(&meshes(), how)
}
