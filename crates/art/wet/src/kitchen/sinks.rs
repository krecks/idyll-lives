//! The sinks: a farmhouse sink (a white apron basin in a painted cabinet under a wood worktop) and a double
//! counter sink (two basins in a two-metre counter run). The tap and its stream are built in the sink's own
//! GameFrame.

use super::M;
use super::common::{py_max, shaker, tube};
use artkit::forms::rounded_rect;
use artkit::materials::Mat;
use artkit::math::{P3, cos, sin};
use artkit::motion::{Grow, grow};
use artkit::parts::{join, new_parts, settle_all};
use artkit::place::{Coord, Facing, GameFrame};
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fcyl, fextrude, fsphere, fsweep, loft, revolve, ring, soft, torus};

// --- the sinks ----------------------------------------------------------------------------------------
// Shared by the two kitchen sinks (a family script stays self-contained).

/// A tap in one piece: a riser from (x, base_y, z) up to `top`, arcing forward to a downward spout with a spray
/// head, on a round escutcheon, a lever on a short body beside it. Returns the spout tip's game point (where the
/// water starts).
#[allow(clippy::too_many_arguments)] // (as the script called it)
fn gooseneck<X: Coord>(
    f: &GameFrame,
    name: &str,
    x: X,
    base_y: f64,
    z: f64,
    spout_z: f64,
    top: f64,
    material: Mat,
    r: f64,
    tip_y: Option<f64>,
) -> (X, f64, f64) {
    let tip_y = tip_y.unwrap_or(base_y + 0.16);
    let pts = [
        (x, base_y, z),
        (x, top - 0.05, z),
        (x, top - 0.004, z + (spout_z - z) * 0.35),
        (x, top - 0.004, z + (spout_z - z) * 0.8),
        (x, top - 0.03, spout_z),
        (x, tip_y + 0.04, spout_z),
        (x, tip_y, spout_z),
    ];
    tube(f, name, &pts, r, material, Some(10), 3, 1.0);
    fcyl!(
        f,
        format!("{name}_head"),
        r * 1.2,
        0.045,
        (x, tip_y + 0.0225, spout_z),
        material,
        verts = 10,
        bevel = 0.004,
        bevel_segments = 1
    );
    let b = f.at(x, base_y, z);
    revolve!(
        format!("{name}_base"),
        &[
            (0.0, 0.0),
            (r * 1.9, 0.0),
            (r * 1.9, 0.006),
            (r * 1.5, 0.016),
            (r * 1.1, 0.024),
            (0.0, 0.024)
        ],
        at = (b.x, b.y, b.z),
        material = material,
        segments = 14
    );
    let lx = x.f() + (if x.f() <= 0.0 { 0.075 } else { -0.075 });
    fcyl!(
        f,
        format!("{name}_valve"),
        0.018,
        0.045,
        (lx, base_y + 0.0225, z),
        material,
        verts = 10,
        bevel = 0.004,
        bevel_segments = 1
    );
    fsweep!(
        f,
        format!("{name}_lever"),
        &[
            (lx, base_y + 0.04, z),
            (lx, base_y + 0.058, z + 0.02),
            (lx, base_y + 0.066, z + 0.075)
        ],
        0.0,
        material,
        profile = rounded_rect(0.014, 0.008, 0.003, 1),
        up = (1, 0, 0),
        resolution = 2,
        scale = [1.3, 1.0, 1.15]
    );
    (x, tip_y, spout_z)
}

/// A sink bowl in one piece, open at the top: a flat rim `lip` wide round a w × d opening at height `rim_y`, the
/// walls running down with rounded corners to a floor at `floor_y` that dips to the drain. `apron` (a height): the
/// outside comes down square to there, as a fireclay block's (an apron-front sink); else it tucks under the rim (a
/// bowl set into a worktop).
#[allow(clippy::too_many_arguments)] // (as the script called it)
fn bowl<X: Coord>(
    fx: &GameFrame,
    name: &str,
    x: X,
    z: f64,
    w: f64,
    d: f64,
    rim_y: f64,
    floor_y: f64,
    material: Mat,
    lip: f64,
    apron: Option<f64>,
    verts: usize,
    square: i32,
) -> Obj {
    let b = fx.at(x, 0, z);
    let (bx, by) = (b.x, b.y);
    let (hw, hd) = (w / 2.0, d / 2.0);

    let ring = |a: f64, b: f64, y: f64, sq: i32| -> Vec<P3> {
        ring!(
            a,
            b,
            z = y,
            verts = verts,
            square = sq as f64,
            at = (bx, by)
        )
    };
    let outside = if let Some(apron) = apron {
        vec![
            ring(hw + lip - 0.006, hd + lip - 0.006, apron, 12),
            ring(hw + lip, hd + lip, apron + 0.006, 12),
            ring(hw + lip, hd + lip, rim_y - 0.006, 12),
        ]
    } else {
        vec![
            ring(hw - 0.03, hd - 0.03, floor_y - 0.012, square),
            ring(hw, hd, floor_y + 0.03, square),
            ring(hw, hd, rim_y - 0.008, square),
            ring(hw + lip, hd + lip, rim_y - 0.005, square),
        ]
    };
    let top = vec![
        ring(
            hw + lip - 0.004,
            hd + lip - 0.004,
            rim_y,
            if apron.is_some() { 12 } else { square },
        ),
        ring(hw + 0.003, hd + 0.003, rim_y, square),
    ];
    let inside = vec![
        ring(hw, hd, rim_y - 0.006, square),
        ring(hw - 0.006, hd - 0.006, floor_y + 0.03, square),
        ring(hw - 0.025, hd - 0.025, floor_y + 0.004, square),
        ring(hw - 0.06, hd - 0.06, floor_y, square),
        ring(0.0, 0.0, floor_y - 0.004, square),
    ];
    let sections: Vec<Vec<P3>> = [outside, top, inside].concat();
    loft!(name, &sections, material = material)
}

/// A drain in a bowl's floor: a dark hole in a chrome ring, a strainer cross over it.
fn drain<X: Coord>(fx: &GameFrame, name: &str, x: X, y: f64, z: f64, r: f64) {
    fcyl!(fx, name, r, 0.006, (x, y, z), M.dark, verts = 12);
    torus!(
        format!("{name}_ring"),
        r,
        0.005,
        at = fx.at(x, y + 0.003, z),
        material = M.chrome,
        major_segments = 12,
        minor_segments = 4
    );
    for i in 0..2 {
        fbox!(
            fx,
            format!("{name}_bar{i}"),
            if i != 0 { 2.0 * r } else { 0.006 },
            0.004,
            if i != 0 { 0.006 } else { 2.0 * r },
            (x, y + 0.002, z),
            M.chrome
        );
    }
}

/// A potted herb on game height `y`: a tapered terracotta pot with a rim, a mound of leaves on it.
fn herb(fx: &GameFrame, name: &str, x: f64, y: f64, z: f64, r: f64) {
    let b = fx.at(x, y, z);
    let (bx, by, bz) = (b.x, b.y, b.z);
    let sections: Vec<Vec<P3>> = [
        (0.0, 0.78),
        (0.1, 0.95),
        (0.1, 1.06),
        (0.13, 1.06),
        (0.13, 0.98),
    ]
    .iter()
    .map(|&(h, k)| ring!(r * k, r * k, z = bz + h, verts = 14, at = (bx, by)))
    .collect();
    loft!(format!("{name}_pot"), &sections, material = M.pot);
    fcyl!(
        fx,
        format!("{name}_soil"),
        r * 0.94,
        0.01,
        (x, y + 0.122, z),
        M.board,
        verts = 12
    );
    for i in 0..7 {
        let a = i as f64 * 2.4;
        let d = if i == 0 {
            0.0
        } else {
            r * (0.45 + 0.08 * (i % 3) as f64)
        };
        let h = if i == 0 {
            0.06
        } else {
            0.03 + 0.012 * (i % 3) as f64
        };
        fsphere!(
            fx,
            format!("{name}_leaves{i}"),
            r * (if i == 0 { 0.62 } else { 0.48 }),
            (x + d * cos(a), y + 0.13 + h, z + d * sin(a)),
            if i % 2 != 0 { M.plant } else { M.bottle },
            segments = 8,
            rings = 5,
            scale = (1, 1, 0.8)
        );
    }
}

/// A soap dish on game height `y` with a bar of soap in it.
#[allow(clippy::too_many_arguments)] // (as the script called it)
fn soap_dish(fx: &GameFrame, name: &str, x: f64, y: f64, z: f64, w: f64, d: f64) {
    let b = fx.at(x, y, z);
    let (bx, by, bz) = (b.x, b.y, b.z);
    let sections: Vec<Vec<P3>> = [
        (0.0, 0.8),
        (0.006, 0.9),
        (0.02, 1.0),
        (0.022, 0.96),
        (0.012, 0.85),
    ]
    .iter()
    .map(|&(h, k)| {
        ring!(
            w / 2.0 * k,
            d / 2.0 * k,
            z = bz + h,
            verts = 16,
            square = 3.0,
            at = (bx, by)
        )
    })
    .collect();
    loft!(name, &sections, material = M.porcelain);
    soft!(
        fx,
        format!("{name}_soap"),
        w * 0.55,
        0.025,
        d * 0.55,
        (x, y + 0.026, z),
        M.egg,
        round = 0.01,
        round_segments = 1
    );
}

/// The stream: a thin cylinder from the spout tip down to the basin, joined with its origin at the
/// tip and scaled to nothing along its length, so it is hidden until `grow` runs the clip.
fn water<X: Coord>(f: &GameFrame, spout: (X, f64, f64), floor_y: f64) -> Obj {
    let (x, top_y, z) = spout;
    let length = py_max(0.06, top_y - floor_y);
    let parts = new_parts(|| {
        fcyl!(
            f,
            "water",
            0.012,
            length,
            (x, (top_y + floor_y) / 2.0, z),
            M.water,
            verts = 10
        );
    });
    let obj = join(&parts, "water", Some(f.at(x, top_y, z)));
    obj.edit(|o| o.scale.z = 0.001);
    obj
}

// --- farmhouse sink: a white apron basin in a painted cabinet under a wood worktop ---------------------

/// A fireclay apron-front basin standing proud of a painted cabinet with a face frame and two shaker doors,
/// under a wooden worktop cut round it; a gooseneck tap with a lever, a soap dish and a potted herb.
pub fn farmhouse_sink() -> Vec<Obj> {
    let fk = GameFrame::new(0.145);
    let mut spout = None;
    let still = new_parts(|| {
        fbox!(
            fk,
            "plinth",
            0.96,
            0.08,
            0.54,
            (0, 0.04, -0.15),
            M.trim,
            bevel = 0.006
        );
        fbox!(
            fk,
            "cabinet",
            0.98,
            0.64,
            0.56,
            (0, 0.40, -0.14),
            M.body,
            bevel = 0.006
        );
        let hole: Vec<(f64, f64)> = rounded_rect(0.88, 0.56, 0.002, 1)
            .iter()
            .map(|&(u, v)| (u, v + 0.01))
            .collect();
        fextrude!(
            fk,
            "face_frame",
            &rounded_rect(0.98, 0.64, 0.004, 1),
            0.02,
            (0, 0.40, 0.15),
            M.body,
            holes = [hole],
            bevel = 0.003,
            bevel_segments = 1
        );
        for s in [-1, 1] {
            let sf = s as f64;
            shaker(
                &fk,
                &format!("door{s}"),
                0.436,
                0.552,
                (sf * 0.2205, 0.41, 0.15),
                M.body,
                0.02,
                0.065,
                0.004,
            );
            pull!(
                &fk,
                format!("pull{s}"),
                (sf * 0.05, 0.49),
                (sf * 0.05, 0.63),
                0.16,
                out = 0.026,
                w = 0.012,
                t = 0.012,
                r = Some(0.005)
            );
        }
        // the painted shoulders round the basin up to the worktop, and the worktop cut round the basin
        fbox!(
            fk,
            "shoulder_back",
            1.0,
            0.2,
            0.15,
            (0, 0.82, -0.405),
            M.body,
            bevel = 0.006
        );
        for s in [-1, 1] {
            fbox!(
                fk,
                format!("shoulder{s}"),
                0.17,
                0.2,
                0.51,
                (s as f64 * 0.415, 0.82, -0.075),
                M.body,
                bevel = 0.006
            );
        }
        let top = [
            (-0.5, -0.48),
            (0.5, -0.48),
            (0.5, 0.18),
            (0.33, 0.18),
            (0.33, -0.33),
            (-0.33, -0.33),
            (-0.33, 0.18),
            (-0.5, 0.18),
        ];
        fextrude!(
            fk,
            "worktop",
            &top,
            0.04,
            (0, 0.93, 0),
            M.wood,
            plane = Facing::Top,
            bevel = 0.006,
            bevel_segments = 2
        );
        // the apron basin: one fireclay piece, square outside, a rounded bowl inside, a drain in its floor
        bowl(
            &fk,
            "apron",
            0,
            -0.075,
            0.6,
            0.45,
            0.95,
            0.785,
            M.porcelain,
            0.03,
            Some(0.72),
            32,
            6,
        );
        drain(&fk, "drain", 0, 0.783, -0.075, 0.035);
        // the tap, a soap dish and a potted herb on the worktop
        spout = Some(gooseneck(
            &fk,
            "tap",
            0,
            0.95,
            -0.38,
            -0.2,
            1.228,
            M.chrome,
            0.025,
            Some(1.16),
        ));
        soap_dish(&fk, "soap_dish", 0.36, 0.95, -0.36, 0.18, 0.11);
        herb(&fk, "herb", -0.36, 0.95, -0.38, 0.06);
    });
    join(&still, "body", Some((0, 0, 0).into()));
    let water = water(&fk, spout.expect("the tap is built"), 0.785);
    settle_all();
    grow(
        water,
        "water_on",
        Grow {
            seconds: 0.4,
            ..Grow::default()
        },
    );
    meshes()
}

// --- double counter sink: two basins in a two-metre counter run ----------------------------------------

/// Two steel bowls set into a stone worktop over a two-metre run of four shaker doors, a gooseneck tap with a
/// lever behind each bowl, a soap dish and a potted herb between them. Built in the counters' drawn 0.66 m depth
/// (the counters' COUNTER66 drawn numbers, crates/art/wet/src/kitchen_counters: the worktop -0.43..0.23).
pub fn counter_sink_double() -> Vec<Obj> {
    let fk = GameFrame::new(0.085);
    let still = new_parts(|| {
        fbox!(
            fk,
            "plinth",
            1.996,
            0.1,
            0.55,
            (0, 0.05, -0.136),
            M.trim,
            bevel = 0.006
        );
        fbox!(
            fk,
            "carcass",
            1.996,
            0.66,
            0.60,
            (0, 0.43, -0.11),
            M.body,
            bevel = 0.006
        );
        // the rails round the bowls under the worktop (the carcass stops below the bowls)
        fbox!(
            fk,
            "rail_front",
            1.996,
            0.12,
            0.05,
            (0, 0.82, 0.165),
            M.body
        );
        fbox!(fk, "rail_back", 1.996, 0.12, 0.1, (0, 0.82, -0.36), M.body);
        for s in [-1, 1] {
            fbox!(
                fk,
                format!("rail_end{s}"),
                0.02,
                0.12,
                0.6,
                (s as f64 * 0.988, 0.82, -0.11),
                M.body
            );
        }
        for (i, x) in [-0.74, -0.26, 0.26, 0.74].into_iter().enumerate() {
            shaker(
                &fk,
                &format!("door{i}"),
                0.468,
                0.74,
                (x, 0.49, 0.2),
                M.body,
                0.02,
                0.065,
                0.004,
            );
            let hx = x + (if [0, 2].contains(&i) { 0.19 } else { -0.19 });
            pull!(
                &fk,
                format!("handle{i}"),
                (hx, 0.62),
                (hx, 0.8),
                0.21,
                out = 0.018,
                w = 0.012,
                t = 0.01,
                r = Some(0.004)
            );
        }
        // the worktop with the two bowls' openings, and the bowls set into it
        let holes: Vec<Vec<(f64, f64)>> = [-1, 1]
            .into_iter()
            .map(|s| {
                rounded_rect(0.625, 0.385, 0.04, 3)
                    .iter()
                    .map(|&(u, v)| (u + s as f64 * 0.42, v + 0.033))
                    .collect()
            })
            .collect();
        fextrude!(
            fk,
            "worktop",
            &rounded_rect(2.0, 0.66, 0.004, 1),
            0.07,
            (0, 0.915, -0.10),
            M.stone,
            plane = Facing::Top,
            holes = holes,
            bevel = 0.006,
            bevel_segments = 2
        );
        for s in [-1, 1] {
            let x = s as f64 * 0.42;
            bowl(
                &fk,
                &format!("basin{s}"),
                x,
                -0.067,
                0.6,
                0.36,
                0.955,
                0.79,
                M.metal,
                0.025,
                None,
                24,
                6,
            );
            drain(&fk, &format!("drain{s}"), x, 0.788, -0.067, 0.035);
            gooseneck(
                &fk,
                &format!("tap{s}"),
                x,
                0.95,
                -0.341,
                -0.17,
                1.235,
                M.chrome,
                0.028,
                Some(1.17),
            );
        }
        soap_dish(&fk, "soap_dish", -0.2, 0.95, -0.37, 0.16, 0.1);
        herb(&fk, "herb", 0.0, 0.95, -0.36, 0.06);
    });
    join(&still, "body", Some((0, 0, 0).into()));
    // Each bowl's tap runs for the one washing at it (`water_on_<slot>`): slot 0 is the bowl at -x, 1 at +x, as
    // sim-core spreads the two slots along the width (objectMotion.ts `slotOf`).
    let waters: Vec<Obj> = [-0.42, 0.42]
        .into_iter()
        .map(|x| water(&fk, (x, 1.17, -0.17), 0.79))
        .collect();
    settle_all();
    for (slot, water) in waters.into_iter().enumerate() {
        grow(
            water,
            &format!("water_on_{slot}"),
            Grow {
                seconds: 0.4,
                ..Grow::default()
            },
        );
    }
    meshes()
}
