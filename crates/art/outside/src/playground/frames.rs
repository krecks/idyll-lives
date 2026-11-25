//! The slide and the climbing frame: a tower to climb, and a way down.

use super::common::{F, M, X, mesh};
use artkit::forms::rounded_rect;
use artkit::json::float_repr;
use artkit::kit;
use artkit::materials::Mat;
use artkit::math::{P3, PI, TAU, V3, cos, cosd, sin, sind};
use artkit::parts::{aside, join_all, new_parts};
use artkit::place::{Facing, Spot};
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fcyl, fextrude, fsphere, fsweep, loft, settle, sweep};

// Game axes: the tower at the back (deck at 1.4 m), the ladder behind it, the chute running down to the front.
const SLIDE_TOP: (f64, f64) = (1.43, -0.29); // (height, depth) where the chute leaves the deck
const SLIDE_END: (f64, f64) = (0.28, 0.95); // and where its run-out ends
const SQ: [(i32, i32); 4] = [(-1, -1), (1, -1), (1, 1), (-1, 1)];

/// The chute: a U-section trough swept down a smoothstep curve (flat at the top and at the run-out), a rolled lip
/// along the top of each wall. Returns the trough.
fn chute(name: &str, steps: usize) -> Obj {
    let ((y0, z0), (y1, z1)) = (SLIDE_TOP, SLIDE_END);
    let mut path = Vec::new();
    for i in 0..=steps {
        let t = i as f64 / steps as f64;
        let s = t * t * (3.0 - 2.0 * t);
        path.push(V3::from(F.at(0, y0 + (y1 - y0) * s, z0 + (z1 - z0) * t)));
    }
    // the section, across (u) and up (v) from the path: a floor and two walls, as one closed outline
    let (w, h, th) = (0.25, 0.16, 0.025);
    let section = [
        (-w, h),
        (-w, 0.0),
        (w, 0.0),
        (w, h),
        (w - th, h),
        (w - th, th),
        (-w + th, th),
        (-w + th, h),
    ];
    let n = section.len();
    let (mut vs, mut faces, mut lips) = (Vec::new(), Vec::new(), (Vec::new(), Vec::new()));
    for (i, &p) in path.iter().enumerate() {
        let t = (path[(i + 1).min(steps)] - path[i.saturating_sub(1)]).normalized();
        let mut up = t.cross(X).normalized();
        if up.z < 0.0 {
            up = -up;
        }
        for &(u, v) in &section {
            vs.push(p + X * u + up * v);
        }
        lips.0.push(p + X * (-w + th / 2.0) + up * (h + 0.004));
        lips.1.push(p + X * (w - th / 2.0) + up * (h + 0.004));
    }
    for i in 0..steps {
        for k in 0..n {
            let k2 = (k + 1) % n;
            faces.push(vec![
                (i * n + k) as u32,
                (i * n + k2) as u32,
                ((i + 1) * n + k2) as u32,
                ((i + 1) * n + k) as u32,
            ]);
        }
    }
    for base in [0, steps * n] {
        // the ends: the U as three quads
        let q: Vec<u32> = (0..n).map(|k| (base + k) as u32).collect();
        faces.push(vec![q[0], q[1], q[6], q[7]]);
        faces.push(vec![q[1], q[2], q[5], q[6]]);
        faces.push(vec![q[2], q[3], q[4], q[5]]);
    }
    let obj = mesh(name, vs, &faces, M.chute, false);
    for (k, pts) in [lips.0, lips.1].iter().enumerate() {
        sweep!(
            format!("{name}_lip{k}"),
            pts,
            0.019,
            material = M.chute,
            verts = 8
        );
    }
    // the run-out's end, rolled under so no edge faces a child
    sweep!(
        format!("{name}_end"),
        &[
            V3::from(F.at(-w + 0.01, y1 + 0.004, z1 + 0.002)),
            V3::from(F.at(w - 0.01, y1 + 0.004, z1 + 0.002))
        ],
        0.016,
        material = M.chute,
        verts = 8
    );
    obj
}

// Headroom on the deck (M-community-07): the roof's eaves 1.03 m over it (a child stands under them), an open way in
// from the ladder between its handrails (no bar across the deck's back), and an arch over the chute's mouth rising
// 0.55 m over its floor (a child sits up under it).
const SLIDE_EAVES: f64 = 2.46;
const SLIDE_ARCH: (f64, f64) = (0.27, 0.55); // the arch over the chute's mouth: half its width, its rise over the floor

pub fn slide() -> Vec<Obj> {
    // the tower: four posts on base plates, a guard either side (a frame round a panel with a porthole), beams
    // round the top under a hipped roof
    for x in [-0.28, 0.28] {
        for z in [-0.31, -0.69] {
            fcyl!(
                F,
                format!("post{}{}", float_repr(x), float_repr(z)),
                0.035,
                SLIDE_EAVES,
                (x, SLIDE_EAVES / 2.0, z),
                M.body,
                verts = 12
            );
            fbox!(
                F,
                format!("base_plate{}{}", float_repr(x), float_repr(z)),
                0.12,
                0.012,
                0.12,
                (x, 0.006, z),
                M.steel,
                bevel = 0.003,
                bevel_segments = 1
            );
            fsphere!(
                F,
                format!("post_cap{}{}", float_repr(x), float_repr(z)),
                0.04,
                (x, SLIDE_EAVES, z),
                M.body,
                segments = 10,
                rings = 5
            );
        }
        let lp = kit::fillet!(
            &[
                (x, 1.62, -0.5),
                (x, 1.62, -0.655),
                (x, 1.86, -0.655),
                (x, 1.86, -0.345),
                (x, 1.62, -0.345),
                (x, 1.62, -0.5),
            ],
            0.04
        );
        fsweep!(
            F,
            format!("guard{}", float_repr(x)),
            &lp[..lp.len() - 1],
            0.02,
            M.body,
            verts = 8,
            closed = true
        );
        let port: Vec<(f64, f64)> = (0..16)
            .map(|k| {
                (
                    0.075 * cos(TAU * k as f64 / 16.0),
                    0.075 * sin(TAU * k as f64 / 16.0),
                )
            })
            .collect();
        crate::eased(fextrude!(
            F,
            format!("panel{}", float_repr(x)),
            &rounded_rect(0.28, 0.21, 0.03, 2),
            0.014,
            (x, 1.74, -0.5),
            M.roof,
            plane = Facing::Side,
            holes = [port]
        ));
        fbox!(
            F,
            format!("beam_side{}", float_repr(x)),
            0.05,
            0.06,
            0.44,
            (x, SLIDE_EAVES - 0.03, -0.5),
            M.body,
            bevel = 0.008,
            bevel_segments = 1
        );
    }
    for z in [-0.31, -0.69] {
        fbox!(
            F,
            format!("beam{}", float_repr(z)),
            0.6,
            0.06,
            0.05,
            (0, SLIDE_EAVES - 0.03, z),
            M.body,
            bevel = 0.008,
            bevel_segments = 1
        );
    }
    // the deck: boards across, a frame under them
    for i in 0..5 {
        fbox!(
            F,
            format!("deck{i}"),
            0.62,
            0.05,
            0.082,
            (0, 1.405, -0.72 + 0.044 + i as f64 * 0.088),
            M.tread,
            bevel = 0.006,
            bevel_segments = 1
        );
    }
    fbox!(F, "deck_frame", 0.6, 0.06, 0.06, (0, 1.35, -0.31), M.body);
    fbox!(
        F,
        "deck_frame_back",
        0.6,
        0.06,
        0.06,
        (0, 1.35, -0.69),
        M.body
    );
    // the roof: a hip with an eave lip, its slopes swept in, and a ball on top
    let square = |y: f64, r: f64| -> Vec<P3> {
        SQ.iter()
            .map(|&(a, b)| F.at(r * a as f64, y, -0.5 + r * b as f64))
            .collect()
    };
    loft!(
        "roof",
        &[
            square(SLIDE_EAVES - 0.01, 0.31),
            square(SLIDE_EAVES, 0.375),
            square(SLIDE_EAVES + 0.03, 0.375),
            square(SLIDE_EAVES + 0.1, 0.26),
            square(SLIDE_EAVES + 0.2, 0.13),
            square(SLIDE_EAVES + 0.29, 0.02)
        ],
        material = M.roof,
        smooth = false
    );
    fsphere!(
        F,
        "roof_knob",
        0.04,
        (0, SLIDE_EAVES + 0.3, -0.5),
        M.roof,
        segments = 10,
        rings = 6
    );
    // the ladder behind, leaning on the deck; its rails, one bent tube each, go on up as handrails either side of
    // the way in and over to the posts
    for x in [-0.25, 0.25] {
        fsweep!(
            F,
            format!("ladder{}", float_repr(x)),
            &kit::fillet!(
                &[
                    (x, 0.0, -0.94),
                    (x, 1.42, -0.7),
                    (x, 1.95, -0.69),
                    (x * 1.12, 1.95, -0.69)
                ],
                0.06
            ),
            0.025,
            M.body,
            verts = 8
        );
        fcyl!(
            F,
            format!("ladder_foot{}", float_repr(x)),
            0.026,
            0.012,
            (x, 0.006, -0.938),
            M.steel,
            verts = 10
        );
    }
    for i in 1..6 {
        let t = i as f64 / 6.0;
        fbox!(
            F,
            format!("rung{i}"),
            0.48,
            0.03,
            0.08,
            (0, 1.42 * t, -0.94 + 0.24 * t),
            M.tread,
            bevel = 0.008,
            bevel_segments = 1
        );
        for g in [-1, 0, 1] {
            // grip grooves along each tread
            fbox!(
                F,
                format!("groove{i}{g}"),
                0.46,
                0.004,
                0.006,
                (0, 1.42 * t + 0.015, -0.94 + 0.24 * t + g as f64 * 0.022),
                M.steel
            );
        }
    }
    // the chute, an arch over its mouth and the legs under its run-out
    chute("chute", 22);
    let (rx, ry) = SLIDE_ARCH;
    let arch: Vec<(f64, f64, f64)> = (0..15)
        .map(|i| {
            (
                rx * cos(PI * i as f64 / 14.0),
                SLIDE_TOP.0 + ry * sin(PI * i as f64 / 14.0),
                -0.27,
            )
        })
        .collect();
    fsweep!(F, "arch", &arch, 0.02, M.body, verts = 8);
    for x in [-0.18, 0.18] {
        fsweep!(
            F,
            format!("leg{}", float_repr(x)),
            &[(x, 0.0, 0.84), (x, 0.3, 0.84)],
            0.025,
            M.body,
            verts = 8
        );
        fbox!(
            F,
            format!("foot{}", float_repr(x)),
            0.09,
            0.012,
            0.09,
            (x, 0.006, 0.84),
            M.steel,
            bevel = 0.003,
            bevel_segments = 1
        );
    }
    fsweep!(
        F,
        "leg_bar",
        &[(-0.18, 0.2, 0.84), (0.18, 0.2, 0.84)],
        0.016,
        M.body,
        verts = 6
    );
    join_all("slide");
    settle!(centre = false);
    meshes()
}

// A wooden fort on four posts: a deck at 1.1 m, railings on three sides, a rope net up the front, a
// fireman's pole beside it and a pitched roof with a flag.

/// A sawn board from game point `a` to `b`, `w` wide (across, as near level as it goes) and `t` thick.
fn board(name: &str, a: impl Spot, b: impl Spot, w: f64, t: f64, material: Mat) -> Obj {
    // eased, its UVs laid on the eased board (`boxed`)
    crate::boxed(crate::eased(sweep!(
        name,
        &[a.through(&F), b.through(&F)],
        0.0,
        material = material,
        profile = vec![
            (-w / 2.0, -t / 2.0),
            (w / 2.0, -t / 2.0),
            (w / 2.0, t / 2.0),
            (-w / 2.0, t / 2.0)
        ],
        up = (0, 0, 1)
    )))
}

pub fn climbing_frame() -> Vec<Obj> {
    let deck = 1.1;
    for x in [-0.85, 0.85] {
        for z in [-0.85, 0.85] {
            crate::boxed(fbox!(
                F,
                format!("post{}{}", float_repr(x), float_repr(z)),
                0.1,
                2.3,
                0.1,
                (x, 1.15, z),
                M.timber,
                bevel = 0.012,
                bevel_segments = 1
            ));
            fbox!(
                F,
                format!("shoe{}{}", float_repr(x), float_repr(z)),
                0.12,
                0.08,
                0.12,
                (x, 0.04, z),
                M.steel,
                bevel = 0.004,
                bevel_segments = 1
            );
        }
    }
    // the deck: boards on two joists, knee braces from the posts up under it
    for i in 0..9 {
        fbox!(
            F,
            format!("deck{i}"),
            1.8,
            0.06,
            0.192,
            (0, deck, -0.9 + 0.1 + i as f64 * 0.2),
            M.timber,
            bevel = 0.006,
            bevel_segments = 1
        );
    }
    for z in [-0.82, 0.82] {
        fbox!(
            F,
            format!("joist{}", float_repr(z)),
            1.6,
            0.12,
            0.06,
            (0, deck - 0.09, z),
            M.timber
        );
    }
    for x in [-0.85, 0.85] {
        for (z, dz) in [(-0.85, 1.0), (0.85, -1.0)] {
            board(
                &format!("brace{}{}", float_repr(x), float_repr(z)),
                (x, deck - 0.4, z + dz * 0.05),
                (x, deck - 0.12, z + dz * 0.33),
                0.07,
                0.04,
                M.timber,
            );
        }
    }
    // railings: a top and bottom rail and flat balusters with rounded tops on the back and both sides
    for side in ["back", "left", "right"] {
        for (k, y) in [deck + 0.12, deck + 0.5].into_iter().enumerate() {
            if side == "back" {
                fbox!(
                    F,
                    format!("{side}_rail{k}"),
                    1.6,
                    0.05,
                    0.05,
                    (0, y, -0.85),
                    M.timber,
                    bevel = 0.01,
                    bevel_segments = 1
                );
            } else {
                let x = if side == "left" { -0.85 } else { 0.85 };
                fbox!(
                    F,
                    format!("{side}_rail{k}"),
                    0.05,
                    0.05,
                    1.6,
                    (x, y, 0),
                    M.timber,
                    bevel = 0.01,
                    bevel_segments = 1
                );
            }
        }
        for i in 0..7 {
            let c = -0.6 + i as f64 * 0.2;
            let at = if side == "back" {
                (c, deck + 0.31, -0.85)
            } else {
                (if side == "left" { -0.85 } else { 0.85 }, deck + 0.31, c)
            };
            let (w, d) = if side == "back" {
                (0.08, 0.02)
            } else {
                (0.02, 0.08)
            };
            fbox!(
                F,
                format!("{side}_baluster{i}"),
                w,
                0.34,
                d,
                at,
                M.timber,
                bevel = 0.006,
                bevel_segments = 1
            );
        }
    }
    // the roof: rafters, purlins, then five courses of shingles a slope lapping down from a ridge board, gables
    for z in [-0.85, 0.85] {
        fbox!(
            F,
            format!("beam{}", float_repr(z)),
            1.8,
            0.1,
            0.08,
            (0, 2.25, z),
            M.timber,
            bevel = 0.008,
            bevel_segments = 1
        );
        for s in [-1, 1] {
            board(
                &format!("rafter{}{s}", float_repr(z)),
                (s as f64 * 0.85, 2.3, z),
                (0, 2.71, z),
                0.08,
                0.05,
                M.timber,
            );
        }
        fextrude!(
            F,
            format!("gable{}", float_repr(z)),
            &[(-0.82, 2.3), (0.82, 2.3), (0.0, 2.68)],
            0.02,
            (0, 0, z - 0.03 * (if z > 0.0 { 1.0 } else { -1.0 })),
            M.timber
        );
    }
    let slope = 30;
    for s in [-1, 1] {
        for k in 0..5 {
            let t = 0.06 + k as f64 * 0.2;
            let (x, y) = (
                s as f64 * (0.02 + t * cosd(slope as f64)),
                2.74 - t * sind(slope as f64) + 0.02,
            );
            fbox!(
                F,
                format!("shingle{s}{k}"),
                0.24,
                0.03,
                1.96,
                (x, y, 0),
                M.shingle,
                rot = (0, -s * (slope + 3), 0),
                bevel = 0.006,
                bevel_segments = 1
            );
        }
    }
    fsweep!(
        F,
        "ridge",
        &[(0, 2.77, -0.97), (0, 2.77, 0.97)],
        0,
        M.shingle,
        profile = rounded_rect(0.06, 0.12, 0.025, 2),
        up = (0, 0, 1)
    );
    // the rope net up the front, from the deck's edge down to the ground a little out: a rope through each row of
    // knots, the knots where they cross, a bar along the top and stakes at its foot
    let (top, bottom) = ((deck - 0.02, 0.86), (0.04, 0.96));
    let xs: Vec<f64> = (0..6).map(|i| -0.75 + i as f64 * 0.18).collect();
    let mut rows = Vec::new();
    for j in 0..6 {
        let t = j as f64 / 5.0;
        rows.push((
            top.0 + (bottom.0 - top.0) * t,
            top.1 + (bottom.1 - top.1) * t,
        ));
    }
    for (i, &x) in xs.iter().enumerate() {
        fsweep!(
            F,
            format!("net_down{i}"),
            &[
                (x, deck + 0.02, 0.86),
                (x, top.0, top.1),
                (x, bottom.0, bottom.1),
                (x, 0.0, 0.97)
            ],
            0.014,
            M.rope,
            verts = 6
        );
    }
    for (j, &(y, z)) in rows.iter().enumerate() {
        let mut sag = vec![(xs[0] - 0.04, y, z)];
        for w in xs.windows(2) {
            let (a, b) = (w[0], w[1]);
            sag.push((a, y, z));
            sag.push(((a + b) / 2.0, y - 0.01, z));
        }
        sag.push((xs[xs.len() - 1], y, z));
        sag.push((xs[xs.len() - 1] + 0.04, y, z));
        fsweep!(F, format!("net_across{j}"), &sag, 0.014, M.rope, verts = 6);
        for (i, &x) in xs.iter().enumerate() {
            fsphere!(
                F,
                format!("knot{i}{j}"),
                0.022,
                (x, y, z),
                M.rope,
                segments = 6,
                rings = 4
            );
        }
    }
    fsweep!(
        F,
        "net_bar",
        &[
            (xs[0] - 0.06, deck + 0.02, 0.88),
            (xs[xs.len() - 1] + 0.06, deck + 0.02, 0.88)
        ],
        0.03,
        M.timber,
        verts = 8
    );
    for x in [xs[0], xs[xs.len() - 1]] {
        fcyl!(
            F,
            format!("peg{}", float_repr(x)),
            0.03,
            0.06,
            (x, 0.03, 0.96),
            M.timber,
            verts = 8
        );
    }
    // the fireman's pole, beside the net, hung from a bend off the front beam, a collar at its foot
    fsweep!(
        F,
        "pole",
        &kit::fillet!(
            &[(0.55, 0.0, 0.95), (0.55, 2.04, 0.95), (0.55, 2.2, 0.85)],
            0.06
        ),
        0.025,
        M.rail,
        verts = 10
    );
    fbox!(
        F,
        "pole_bracket",
        0.08,
        0.1,
        0.03,
        (0.55, 2.2, 0.83),
        M.steel,
        bevel = 0.004,
        bevel_segments = 1
    );
    fcyl!(
        F,
        "pole_foot",
        0.06,
        0.02,
        (0.55, 0.01, 0.95),
        M.steel,
        verts = 10
    );
    join_all("climbingFrame");
    // the flag on the ridge, at the back: a pole with a ball, the flag rippling out from it. Drawn with the
    // frame, not part of its size (the frame is 2.86 m tall; the pole stands 0.59 m above the ridge)
    let carried = new_parts(|| {
        fcyl!(
            F,
            "flagpole",
            0.018,
            0.82,
            (0, 3.04, -0.85),
            M.rail,
            verts = 8
        );
        fsphere!(
            F,
            "flag_ball",
            0.03,
            (0, 3.45 - 0.03, -0.85),
            M.flag,
            segments = 8,
            rings = 5
        );
        fsweep!(
            F,
            "flag",
            &[
                (0.01, 3.33, -0.85),
                (0.1, 3.33, -0.82),
                (0.2, 3.32, -0.87),
                (0.3, 3.31, -0.84)
            ],
            0,
            M.flag,
            profile = vec![(-0.1, -0.004), (0.1, -0.004), (0.1, 0.004), (-0.1, 0.004)],
            up = (0, 0, 1),
            resolution = 3,
            scale = [1.0, 0.98, 0.95, 0.9]
        );
    });
    aside(&carried, "flag");
    settle!(centre = false);
    meshes()
}
