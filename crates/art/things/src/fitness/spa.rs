//! Wellness at home: the massage table with its trolley of towels and oil, and the meditation cushion.

use artkit::math::{P3, TAU, cos, sin};
use artkit::parts::{aside, join, new_parts, numbered};
use artkit::place::{Facing, GameFrame};
use artkit::scene::Obj;
use artkit::{fbox, fcyl, fextrude, fsphere, kit, loft, palette, ring, settle, soft, torus};

// --- the massage table (2x2): the table in the left column, the masseur's spot and a trolley on the right ------

palette! { T:
    body = ("#7E9A92", satin),    // the vinyl upholstery: the pad and the face cradle; the main part the player colours
    oak = ("#C9A274", satin),     // the table's frame and legs, the trolley
    steel = ("#C9CDD1", metal),   // the cradle's rods, the height knobs, the casters' forks
    rubber = ("#3A3D41", satin),  // feet, casters
    towel = ("#F2EEE6", satin),   // folded towels
    sage = ("#B9CCBF", satin),    // a sage towel
    oil = ("#B7772E", gloss),     // amber oil bottles
    pump = ("#2E2F31", satin),    // their pumps
}

/// The table's middle across (game x): the left tile column.
const TABLE_X: f64 = -0.5;
/// The pad's top: furniture.json `beds.massageTable`.
pub const PAD_TOP: f64 = 0.78;
const PAD_T: f64 = 0.065;
const PAD_W: f64 = 0.7;
/// The pad's ends (game depth): the head end at the back, the face cradle beyond it, its back on the wall's face.
const PAD_BACK: f64 = -0.65;
const PAD_FRONT: f64 = 0.92;
/// The face cradle's middle (depth): under the lying resident's head, 0.84 m behind the place's middle.
const CRADLE_Z: f64 = -0.79;

/// A horseshoe opening to the front (+depth): outer radius `ro`, inner `ri`, its legs `leg` long, as a game outline
/// (x, depth) about its arc's middle.
fn horseshoe(ro: f64, ri: f64, leg: f64, n: usize) -> Vec<(f64, f64)> {
    let mut pts = vec![(ro, leg)];
    for i in 0..=n {
        let a = TAU / 2.0 * i as f64 / n as f64;
        pts.push((ro * cos(a), -ro * sin(a)));
    }
    pts.push((-ro, leg));
    pts.push((-ri, leg));
    for i in (0..=n).rev() {
        let a = TAU / 2.0 * i as f64 / n as f64;
        pts.push((ri * cos(a), -ri * sin(a)));
    }
    pts.push((ri, leg));
    pts
}

pub fn massage_table() -> Vec<Obj> {
    let f = GameFrame::new(0.0);
    let x0 = TABLE_X;
    // The trolley at the right back, clear of the masseur's spot (place 1 at [0.15, -0.25]): drawn with the table,
    // not part of its size. Two oak shelves on four posts and casters; folded towels below, oil and a rolled towel
    // on top.
    let (tx, tz) = (0.55, -0.7);
    let (tw, td) = (0.42, 0.32);
    let trolley = new_parts(|| {
        for (sx, sz) in [(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)] {
            let (px, pz) = (tx + sx * (tw / 2.0 - 0.02), tz + sz * (td / 2.0 - 0.02));
            fbox!(
                f,
                numbered("trolley_post"),
                0.03,
                0.6,
                0.03,
                (px, 0.36, pz),
                T.oak,
                bevel = 0.004,
                bevel_segments = 1
            );
            fbox!(
                f,
                numbered("caster_fork"),
                0.026,
                0.03,
                0.02,
                (px, 0.045, pz),
                T.steel
            );
            fcyl!(
                f,
                numbered("caster"),
                0.026,
                0.018,
                (px, 0.026, pz),
                T.rubber,
                rot = (0, 90, 0),
                verts = 12
            );
        }
        for (name, y) in [("trolley_shelf", 0.16), ("trolley_top", 0.6)] {
            fbox!(
                f,
                name,
                tw,
                0.018,
                td,
                (tx, y, tz),
                T.oak,
                bevel = 0.004,
                bevel_segments = 1
            );
        }
        // the top's gallery: a low rail round three sides
        fbox!(
            f,
            "gallery_back",
            tw,
            0.03,
            0.012,
            (tx, 0.624, tz - td / 2.0 + 0.006),
            T.oak,
            bevel = 0.003,
            bevel_segments = 1
        );
        for s in [-1.0, 1.0] {
            fbox!(
                f,
                numbered("gallery_side"),
                0.012,
                0.03,
                td,
                (tx + s * (tw / 2.0 - 0.006), 0.624, tz),
                T.oak,
                bevel = 0.003,
                bevel_segments = 1
            );
        }
        // three folded towels on the shelf, the middle one sage
        for (k, (dx, dz, m)) in [
            (0.0, 0.0, T.towel),
            (0.008, -0.006, T.sage),
            (-0.005, 0.004, T.towel),
        ]
        .into_iter()
        .enumerate()
        {
            soft!(
                f,
                numbered("towel"),
                0.32,
                0.045,
                0.24,
                (tx + dx, 0.169 + 0.0225 + 0.045 * k as f64, tz + dz),
                m,
                round = 0.014,
                puff = 0.004,
                resting = true
            );
        }
        // two amber bottles with pumps, and a rolled towel on top
        for (k, (bx, bz)) in [(0.43, -0.78), (0.5, -0.8)].into_iter().enumerate() {
            let h = [0.16, 0.12][k];
            kit::turned!(
                f,
                numbered("bottle"),
                bx,
                0.609,
                bz,
                &[
                    (0.0, 0.0),
                    (0.028, 0.0),
                    (0.03, 0.006),
                    (0.03, h - 0.03),
                    (0.016, h - 0.012),
                    (0.012, h),
                    (0.0, h)
                ],
                T.oil,
                segments = 14
            );
            fcyl!(
                f,
                numbered("bottle_pump"),
                0.013,
                0.025,
                (bx, 0.609 + h + 0.012, bz),
                T.pump,
                verts = 10
            );
            fbox!(
                f,
                numbered("bottle_spout"),
                0.008,
                0.008,
                0.03,
                (bx, 0.609 + h + 0.028, bz + 0.012),
                T.pump
            );
        }
        soft!(
            f,
            "towel_roll",
            0.09,
            0.09,
            0.26,
            (0.64, 0.654, tz + 0.02),
            T.towel,
            round = 0.04,
            puff = 0.003,
            resting = true
        );
    });
    aside(&trolley, "trolley");

    let top_of_frame = PAD_TOP - PAD_T;
    let parts = new_parts(|| {
        // The pad: vinyl over foam, its edges rounded, a little crowned.
        soft!(
            f,
            "pad",
            PAD_W,
            PAD_T,
            PAD_FRONT - PAD_BACK,
            (x0, PAD_TOP - PAD_T / 2.0, (PAD_BACK + PAD_FRONT) / 2.0),
            T.body,
            round = 0.025,
            puff = 0.005,
            resting = true
        );
        // the board under it and the apron round it
        fbox!(
            f,
            "board",
            PAD_W - 0.02,
            0.018,
            PAD_FRONT - PAD_BACK - 0.02,
            (x0, top_of_frame - 0.009, (PAD_BACK + PAD_FRONT) / 2.0),
            T.oak,
            bevel = 0.004,
            bevel_segments = 1
        );
        let apron_y = top_of_frame - 0.018 - 0.045;
        for s in [-1.0, 1.0] {
            fbox!(
                f,
                numbered("apron_side"),
                0.022,
                0.09,
                PAD_FRONT - PAD_BACK - 0.04,
                (
                    x0 + s * (PAD_W / 2.0 - 0.03),
                    apron_y,
                    (PAD_BACK + PAD_FRONT) / 2.0
                ),
                T.oak,
                bevel = 0.005,
                bevel_segments = 1
            );
        }
        for z in [PAD_BACK + 0.031, PAD_FRONT - 0.031] {
            fbox!(
                f,
                numbered("apron_end"),
                PAD_W - 0.082,
                0.09,
                0.022,
                (x0, apron_y, z),
                T.oak,
                bevel = 0.005,
                bevel_segments = 1
            );
        }
        // Four legs, each an upper and a lower piece (the height adjusts) with a knob, splayed out a little; a
        // stretcher along each side and across each end, low down.
        let leg_top = apron_y - 0.045;
        for lz in [PAD_BACK + 0.1, PAD_FRONT - 0.1] {
            for s in [-1.0, 1.0] {
                let lx = x0 + s * (PAD_W / 2.0 - 0.07);
                fbox!(
                    f,
                    numbered("leg"),
                    0.05,
                    leg_top - 0.32,
                    0.05,
                    (lx, (leg_top + 0.32) / 2.0, lz),
                    T.oak,
                    bevel = 0.006,
                    bevel_segments = 1
                );
                fbox!(
                    f,
                    numbered("leg_foot"),
                    0.044,
                    0.33,
                    0.044,
                    (lx, 0.185, lz),
                    T.oak,
                    bevel = 0.006,
                    bevel_segments = 1
                );
                fbox!(
                    f,
                    numbered("foot_pad"),
                    0.05,
                    0.02,
                    0.05,
                    (lx, 0.01, lz),
                    T.rubber,
                    bevel = 0.004,
                    bevel_segments = 1
                );
                fcyl!(
                    f,
                    numbered("knob"),
                    0.016,
                    0.022,
                    (lx + s * 0.034, 0.35, lz),
                    T.steel,
                    rot = (0, 90, 0),
                    verts = 12,
                    bevel = 0.003,
                    bevel_segments = 1
                );
            }
        }
        for s in [-1.0, 1.0] {
            fbox!(
                f,
                numbered("stretcher"),
                0.03,
                0.05,
                PAD_FRONT - PAD_BACK - 0.25,
                (
                    x0 + s * (PAD_W / 2.0 - 0.07),
                    0.13,
                    (PAD_BACK + PAD_FRONT) / 2.0
                ),
                T.oak,
                bevel = 0.005,
                bevel_segments = 1
            );
        }
        fbox!(
            f,
            "stretcher_cross",
            PAD_W - 0.19,
            0.04,
            0.03,
            (x0, 0.13, (PAD_BACK + PAD_FRONT) / 2.0),
            T.oak,
            bevel = 0.005,
            bevel_segments = 1
        );
        // The face cradle beyond the head end: a padded horseshoe opening towards the table, on a plate, held out
        // on two steel rods from under the pad.
        let cradle_top = PAD_TOP - 0.01;
        let ct = 0.06;
        fextrude!(
            f,
            "cradle",
            &horseshoe(0.135, 0.06, 0.08, 16),
            ct,
            (x0, cradle_top - ct / 2.0, CRADLE_Z),
            T.body,
            plane = Facing::Top,
            bevel = 0.018,
            bevel_segments = 3
        );
        fextrude!(
            f,
            "cradle_plate",
            &horseshoe(0.125, 0.07, 0.09, 12),
            0.016,
            (x0, cradle_top - ct - 0.008, CRADLE_Z),
            T.oak,
            plane = Facing::Top,
            bevel = 0.004,
            bevel_segments = 1
        );
        for s in [-1.0, 1.0] {
            kit::tube!(
                f,
                numbered("cradle_rod"),
                &[
                    (x0 + s * 0.1, top_of_frame - 0.03, PAD_BACK + 0.15),
                    (x0 + s * 0.1, top_of_frame - 0.03, PAD_BACK - 0.02),
                    (x0 + s * 0.1, cradle_top - ct - 0.02, CRADLE_Z + 0.04)
                ],
                0.011,
                T.steel,
                verts = 10,
                round = 2
            );
        }
    });
    join(&parts, "table", Some(P3::ZERO));
    settle!(centre = false)
}

// --- the meditation cushion (1x1): a pleated zafu on a square zabuton ------------------------------------------

palette! { Z:
    body = ("#4C5872", fabric),    // the zafu's cotton (a slate indigo); the main part the player colours
    zabuton = ("#D9CCB0", fabric), // the mat's cotton, oat
}

/// The zabuton's thickness, and how far the zafu sinks into it.
const ZABUTON_T: f64 = 0.045;
const SINK: f64 = 0.005;
/// The zafu's top, the seat (furniture.json `seats.meditationCushion`): under 0.2 m, so it is sat on cross-legged
/// (seats.ts `floorSit`).
const SEAT: f64 = 0.18;
const ZAFU_H: f64 = SEAT - ZABUTON_T + SINK;

/// A ring of `verts` points at height `z` (kernel), radius `r` waved in `flutes` pleats `depth` deep (a share of
/// `r`), about the origin.
fn pleated(r: f64, z: f64, verts: usize, flutes: usize, depth: f64) -> Vec<P3> {
    (0..verts)
        .map(|k| {
            let a = TAU * k as f64 / verts as f64;
            let rr = r * (1.0 + depth * cos(flutes as f64 * a));
            P3 {
                x: rr * cos(a),
                y: rr * sin(a),
                z,
            }
        })
        .collect()
}

pub fn meditation_cushion() -> Vec<Obj> {
    let f = GameFrame::new(0.0);
    let parts = new_parts(|| {
        // the zabuton: a cotton mat, stuffed thin (tufts cost 580 triangles and did not show)
        soft!(
            f,
            "zabuton",
            0.7,
            ZABUTON_T,
            0.7,
            (0, ZABUTON_T / 2.0, 0),
            Z.zabuton,
            round = 0.018,
            puff = 0.006,
            resting = true
        );
        // The zafu, 0.36 m across: a flat round top and bottom, its side gathered in pleats and bulging, a piped
        // seam round the top, a covered button in the middle and a carrying loop on its side.
        let y0 = ZABUTON_T - SINK;
        let (v, n, r) = (60, 20, 0.174);
        let rings = [
            ring!(0.14, 0.14, z = y0, verts = v),
            pleated(0.158, y0 + 0.008, v, n, 0.015),
            pleated(r, y0 + 0.035, v, n, 0.03),
            pleated(r, y0 + 0.07, v, n, 0.032),
            pleated(0.168, y0 + 0.104, v, n, 0.026),
            pleated(0.15, y0 + 0.126, v, n, 0.014),
            ring!(0.13, 0.13, z = y0 + 0.134, verts = v),
            ring!(0.07, 0.07, z = y0 + ZAFU_H, verts = v),
            ring!(0.012, 0.012, z = y0 + ZAFU_H - 0.004, verts = v),
        ];
        loft!("zafu", &rings, material = Z.body);
        torus!(
            "zafu_piping",
            0.135,
            0.005,
            at = f.at(0, y0 + 0.131, 0),
            material = Z.body,
            major_segments = 40,
            minor_segments = 3
        );
        fsphere!(
            f,
            "zafu_button",
            0.016,
            (0, y0 + ZAFU_H - 0.003, 0),
            Z.body,
            scale = (1, 1, 0.4),
            segments = 10,
            rings = 5
        );
        kit::tube!(
            f,
            "zafu_loop",
            &[
                (-0.05, y0 + 0.095, 0.164),
                (0.0, y0 + 0.08, 0.19),
                (0.05, y0 + 0.095, 0.164)
            ],
            0.006,
            Z.body,
            verts = 6,
            round = 2
        );
    });
    join(&parts, "cushion", Some(P3::ZERO));
    settle!()
}

/// The palettes of the two, for their `Model`s.
pub fn table_palette() -> artkit::materials::Palette {
    T.palette()
}
pub fn cushion_palette() -> artkit::materials::Palette {
    Z.palette()
}
