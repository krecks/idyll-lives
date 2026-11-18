//! H3: the chemistry bench: a lab bench with a back board and a shelf of test tubes, three flasks and a microscope. The
//! liquid in the middle flask is its own part and rises while a resident experiments (`bubble`), its bubbles on top.

use super::M;
use super::common::rr;
use super::kit::Run;
use super::kit::call::{moulding, tube};
use artkit::forms::smooth_outline;
use artkit::json::float_repr as r;
use artkit::materials::Mat;
use artkit::math::{P3, TAU, cos, cosd, sin, sind};
use artkit::motion::{Grow, grow};
use artkit::parts::{join, new_parts, settle};
use artkit::place::{Facing, GameFrame};
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fcone, fcyl, fextrude, fsphere, fsweep, revolve, torus};

const BENCH_TOP: f64 = 0.9;
const FLASK: [(f64, f64); 8] = [
    (0.0, 0.0),
    (0.054, 0.0),
    (0.06, 0.006),
    (0.06, 0.05),
    (0.024, 0.138),
    (0.0216, 0.15),
    (0.0216, 0.176),
    (0.025, 0.18),
];
const RISING: (f64, f64) = (-0.15, -0.05);

/// An Erlenmeyer flask on the bench, glass, with `liquid` up to `fill` (m, below 0.05) inside.
fn flask(fr: &GameFrame, name: &str, x: f64, z: f64, liquid: Option<Mat>, fill: f64) {
    revolve!(
        name,
        &FLASK,
        at = fr.at(x, BENCH_TOP, z),
        material = M.glass,
        segments = 16
    );
    if let Some(liquid) = liquid {
        revolve!(
            format!("{name}_liquid"),
            &[
                (0.0, 0.003),
                (0.054, 0.003),
                (0.056, 0.008),
                (0.056, fill),
                (0.0, fill)
            ],
            at = fr.at(x, BENCH_TOP, z),
            material = liquid,
            segments = 16
        );
    }
}

pub fn chemistry_set() -> Vec<Obj> {
    let f = GameFrame::new(0.0);
    let still = new_parts(|| {
        // the bench: a resin top with a raised lip, on a welded steel frame with a low shelf, levelling feet
        fbox!(
            f,
            "top",
            0.92,
            0.04,
            0.5,
            (0, 0.88, -0.15),
            M.chem_bench,
            bevel = 0.008
        );
        moulding!(
            f,
            "lip",
            &[(0.46, 0.1), (-0.46, 0.1), (-0.46, -0.4), (0.46, -0.4)],
            &[
                (0.0, 0.0),
                (0.0, 0.006),
                (0.004, 0.008),
                (0.008, 0.006),
                (0.008, 0.0)
            ],
            M.chem_bench,
            at = BENCH_TOP,
            plane = Run::Top
        );
        let frame = rr(0.04, 0.04, 0.006, 1);
        for sx in [-1, 1] {
            let x = sx as f64 * 0.42;
            for z in [-0.36, 0.06] {
                fsweep!(
                    f,
                    format!("leg{sx}{}", r(z)),
                    &[(x, 0.86, z), (x, 0.02, z)],
                    0,
                    M.chem_frame,
                    profile = &frame,
                    up = (1, 0, 0)
                );
                fcyl!(
                    f,
                    format!("foot{sx}{}", r(z)),
                    0.018,
                    0.02,
                    (x, 0.01, z),
                    M.scope,
                    verts = 8
                );
            }
            fsweep!(
                f,
                format!("rail_side{sx}"),
                &[(x, 0.835, -0.34), (x, 0.835, 0.04)],
                0,
                M.chem_frame,
                profile = rr(0.04, 0.03, 0.005, 1)
            );
        }
        for z in [-0.36, 0.06] {
            fsweep!(
                f,
                format!("rail{}", r(z)),
                &[(-0.4, 0.835, z), (0.4, 0.835, z)],
                0,
                M.chem_frame,
                profile = rr(0.04, 0.03, 0.005, 1)
            );
        }
        fbox!(
            f,
            "shelf_low",
            0.86,
            0.02,
            0.44,
            (0, 0.3, -0.15),
            M.chem_bench,
            bevel = 0.004,
            bevel_segments = 1
        );
        for (k, (x, w, h)) in [(-0.25, 0.22, 0.16), (0.02, 0.18, 0.12)]
            .into_iter()
            .enumerate()
        {
            fbox!(
                f,
                format!("storage_box{k}"),
                w,
                h,
                0.3,
                (x, 0.31 + h / 2.0, -0.17),
                M.jar,
                bevel = 0.008,
                bevel_segments = 1
            );
            fbox!(
                f,
                format!("storage_label{k}"),
                0.08,
                0.03,
                0.002,
                (x, 0.31 + h * 0.6, -0.019),
                M.label
            );
        }
        // the back board with a shelf of test tubes, and a periodic table on it
        fbox!(
            f,
            "board",
            0.8,
            0.5,
            0.04,
            (0, 1.15, -0.38),
            M.chem_board,
            bevel = 0.008
        );
        for row in 0..4 {
            for col in 0..9 {
                if row == 0 && 0 < col && col < 8 {
                    continue;
                }
                fbox!(
                    f,
                    format!("element{row}_{col}"),
                    0.026,
                    0.026,
                    0.002,
                    (
                        0.08 + 0.03 * col as f64,
                        1.06 + 0.03 * (3 - row) as f64,
                        -0.359
                    ),
                    [
                        M.liquid_green,
                        M.liquid_yellow,
                        M.liquid_blue,
                        M.liquid_orange
                    ][(col + row) % 4]
                );
            }
        }
        fbox!(
            f,
            "shelf",
            0.8,
            0.02,
            0.16,
            (0, 1.25, -0.3),
            M.chem_bench,
            bevel = 0.004
        );
        // the rack of test tubes on the shelf: a top plate with holes, a base and ends
        let holes: Vec<Vec<(f64, f64)>> = (0..5)
            .map(|k| {
                (0..8)
                    .map(|i| {
                        (
                            -0.24 + 0.12 * k as f64 + 0.017 * cos(TAU * i as f64 / 8.0),
                            0.017 * sin(TAU * i as f64 / 8.0),
                        )
                    })
                    .collect()
            })
            .collect();
        fextrude!(
            f,
            "rack",
            &rr(0.6, 0.05, 0.004, 1),
            0.012,
            (-0.06, 1.3, -0.29),
            M.chem_frame,
            holes = holes,
            plane = Facing::Top
        );
        fbox!(
            f,
            "rack_base",
            0.6,
            0.008,
            0.05,
            (-0.06, 1.264, -0.29),
            M.chem_frame
        );
        for x in [-0.36, 0.24] {
            fbox!(
                f,
                format!("rack_end{}", r(x)),
                0.012,
                0.05,
                0.05,
                (x, 1.285, -0.29),
                M.chem_frame
            );
        }
        for (k, colour) in [
            M.liquid_green,
            M.liquid_yellow,
            M.liquid_orange,
            M.liquid_blue,
            M.liquid_red,
        ]
        .into_iter()
        .enumerate()
        {
            let x = -0.3 + 0.12 * k as f64;
            revolve!(
                format!("tube{k}"),
                &[
                    (0.0, 0.0),
                    (0.012, 0.003),
                    (0.015, 0.012),
                    (0.015, 0.14),
                    (0.017, 0.142)
                ],
                at = f.at(x, 1.26, -0.29),
                material = M.glass,
                segments = 10
            );
            let fill = 0.05 + 0.012 * (k % 3) as f64;
            revolve!(
                format!("tube_liquid{k}"),
                &[(0.0, 0.002), (0.012, 0.007), (0.012, fill), (0.0, fill)],
                at = f.at(x, 1.26, -0.29),
                material = colour,
                segments = 10
            );
        }
        // the flasks, the rising one empty at rest
        flask(&f, "flask_green", -0.3, -0.1, Some(M.liquid_green), 0.045);
        flask(&f, "flask_rising", RISING.0, RISING.1, None, 0.0);
        flask(&f, "flask_blue", 0.25, -0.08, Some(M.liquid_blue), 0.03);
        // a beaker with a spout and graduations
        revolve!(
            "beaker",
            &[
                (0.0, 0.0),
                (0.038, 0.0),
                (0.04, 0.003),
                (0.04, 0.09),
                (0.043, 0.095)
            ],
            at = f.at(0.36, BENCH_TOP, -0.25),
            material = M.glass,
            segments = 14
        );
        revolve!(
            "beaker_liquid",
            &[(0.0, 0.003), (0.037, 0.003), (0.037, 0.04), (0.0, 0.04)],
            at = f.at(0.36, BENCH_TOP, -0.25),
            material = M.liquid_yellow,
            segments = 14
        );
        for k in 0..3 {
            fbox!(
                f,
                format!("beaker_mark{k}"),
                0.012,
                0.0015,
                0.001,
                (0.36, BENCH_TOP + 0.03 + 0.02 * k as f64, -0.2095),
                M.label
            );
        }
        // a retort stand: a base plate, a rod, a clamp holding a test tube over the burner
        fbox!(
            f,
            "stand_base",
            0.12,
            0.012,
            0.18,
            (-0.38, BENCH_TOP + 0.006, -0.25),
            M.chem_frame,
            bevel = 0.003,
            bevel_segments = 1
        );
        tube!(
            f,
            "stand_rod",
            &[(-0.4, BENCH_TOP, -0.31), (-0.4, BENCH_TOP + 0.32, -0.31)],
            0.005,
            M.chrome,
            verts = 6,
            round = 1
        );
        fbox!(
            f,
            "stand_boss",
            0.024,
            0.024,
            0.024,
            (-0.4, BENCH_TOP + 0.24, -0.31),
            M.chem_frame,
            bevel = 0.003,
            bevel_segments = 1
        );
        tube!(
            f,
            "stand_arm",
            &[
                (-0.4, BENCH_TOP + 0.24, -0.31),
                (-0.4, BENCH_TOP + 0.24, -0.22)
            ],
            0.004,
            M.chrome,
            verts = 6,
            round = 1
        );
        torus!(
            "stand_clamp",
            0.018,
            0.004,
            at = f.at(-0.4, BENCH_TOP + 0.24, -0.2),
            material = M.chem_frame,
            major_segments = 10,
            minor_segments = 4
        );
        revolve!(
            "stand_tube",
            &[
                (0.0, 0.0),
                (0.012, 0.003),
                (0.015, 0.012),
                (0.015, 0.12),
                (0.017, 0.122)
            ],
            at = f.at(-0.4, BENCH_TOP + 0.18, -0.2),
            material = M.glass,
            segments = 10
        );
        // the microscope: a cast base and arm, the stage, a turret of objectives, the tube and eyepiece, focus knobs
        let (sx, sz) = (0.05, -0.22);
        fextrude!(
            f,
            "scope_base",
            &smooth_outline(
                &[
                    (0.06, -0.075),
                    (0.06, 0.04),
                    (0.03, 0.075),
                    (-0.03, 0.075),
                    (-0.06, 0.04),
                    (-0.06, -0.075)
                ],
                1,
                true
            ),
            0.02,
            (sx, BENCH_TOP + 0.01, sz),
            M.scope,
            plane = Facing::Top,
            bevel = 0.005,
            bevel_segments = 1
        );
        fsweep!(
            f,
            "scope_arm",
            &[
                (sx, BENCH_TOP + 0.02, sz - 0.06),
                (sx, BENCH_TOP + 0.1, sz - 0.065),
                (sx, BENCH_TOP + 0.19, sz - 0.04),
                (sx, BENCH_TOP + 0.23, sz + 0.0)
            ],
            0,
            M.scope,
            profile = rr(0.036, 0.04, 0.012, 1),
            up = (1, 0, 0),
            resolution = 2,
            scale = [1.1, 1.0, 0.9, 0.8]
        );
        fbox!(
            f,
            "scope_stage",
            0.09,
            0.008,
            0.08,
            (sx, BENCH_TOP + 0.09, sz + 0.02),
            M.scope,
            bevel = 0.003
        );
        fbox!(
            f,
            "slide",
            0.06,
            0.002,
            0.022,
            (sx, BENCH_TOP + 0.095, sz + 0.02),
            M.glass
        );
        for s in [-1, 1] {
            fbox!(
                f,
                format!("stage_clip{s}"),
                0.006,
                0.002,
                0.03,
                (sx + s as f64 * 0.025, BENCH_TOP + 0.096, sz + 0.02),
                M.chrome
            );
        }
        revolve!(
            "scope_light",
            &[
                (0.0, 0.0),
                (0.016, 0.0),
                (0.018, 0.006),
                (0.012, 0.012),
                (0.0, 0.012)
            ],
            at = f.at(sx, BENCH_TOP + 0.02, sz + 0.02),
            material = M.chrome,
            segments = 10
        );
        fcyl!(
            f,
            "scope_turret",
            0.022,
            0.02,
            (sx, BENCH_TOP + 0.15, sz + 0.02),
            M.chrome,
            verts = 12
        );
        for k in 0..3 {
            let a = (90 + 120 * k) as f64;
            fcone!(
                f,
                format!("scope_objective{k}"),
                0.008,
                0.005,
                0.035,
                (
                    sx + 0.012 * cosd(a),
                    BENCH_TOP + 0.125,
                    sz + 0.02 + 0.012 * sind(a)
                ),
                M.chrome,
                verts = 8
            );
        }
        fcyl!(
            f,
            "scope_tube",
            0.016,
            0.12,
            (sx, BENCH_TOP + 0.21, sz + 0.005),
            M.scope,
            rot = (-20, 0, 0),
            verts = 12
        );
        fcyl!(
            f,
            "scope_eyepiece",
            0.013,
            0.03,
            (sx, BENCH_TOP + 0.275, sz - 0.018),
            M.chrome,
            rot = (-20, 0, 0),
            verts = 10
        );
        for s in [-1, 1] {
            revolve!(
                format!("scope_knob{s}"),
                &[
                    (0.0, 0.0),
                    (0.016, 0.0),
                    (0.017, 0.004),
                    (0.017, 0.01),
                    (0.012, 0.014),
                    (0.0, 0.014)
                ],
                at = f.at(sx + s as f64 * 0.02, BENCH_TOP + 0.08, sz - 0.05),
                material = M.chrome,
                segments = 12,
                rot = (0, -s * 90, 0)
            );
        }
        fbox!(
            f,
            "notes",
            0.14,
            0.002,
            0.18,
            (-0.02, BENCH_TOP + 0.001, 0.0),
            M.sheet,
            rot = (0, 0, -8)
        );
        for k in 0..5 {
            fbox!(
                f,
                format!("notes_line{k}"),
                0.1,
                0.0005,
                0.003,
                (-0.02, BENCH_TOP + 0.0022, -0.05 + 0.025 * k as f64),
                M.ink,
                rot = (0, 0, -8)
            );
        }
        fsweep!(
            f,
            "pencil",
            &[
                (0.07, BENCH_TOP + 0.005, -0.02),
                (0.08, BENCH_TOP + 0.005, 0.1)
            ],
            0.004,
            M.liquid_yellow,
            verts = 6
        );
    });
    join(&still, "body", Some(P3::ZERO));
    let p = new_parts(|| {
        revolve!(
            "liquid",
            &[
                (0.0, 0.003),
                (0.054, 0.003),
                (0.056, 0.008),
                (0.056, 0.05),
                (0.04, 0.09),
                (0.0, 0.09)
            ],
            at = f.at(RISING.0, BENCH_TOP, RISING.1),
            material = M.liquid_orange,
            segments = 16
        );
        for (k, (dx, dy, r)) in [
            (0.0, 0.093, 0.009),
            (0.015, 0.088, 0.007),
            (-0.012, 0.089, 0.006),
        ]
        .into_iter()
        .enumerate()
        {
            fsphere!(
                f,
                format!("bubble{k}"),
                r,
                (RISING.0 + dx, BENCH_TOP + dy, RISING.1 + 0.005 * k as f64),
                M.liquid_orange,
                segments = 8,
                rings = 6
            );
        }
    });
    // Hidden at rest by flattening into the bench top (on the flask's floor, it would show through the glass).
    let liquid = join(
        &p,
        "liquid",
        Some(f.at(RISING.0, BENCH_TOP - 0.01, RISING.1)),
    );
    liquid.edit(|o| o.scale.z = 0.001);
    // on its Model's `centre`: its back moved onto the wall face
    settle(&meshes());
    grow(
        liquid,
        "bubble",
        Grow {
            seconds: 1.5,
            ..Grow::default()
        },
    );
    meshes()
}
