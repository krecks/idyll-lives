//! The writer's bureau and the book-club tea trolley.

use super::common::{M, books, knob, lamp, mug, pile};
use crate::ease::ease_ends;
use artkit::forms::{rounded_rect, smooth_outline};
use artkit::materials::Mat;
use artkit::math::{Axis, P3, PI, cosd, pow, sin, sind};
use artkit::parts::{hinged, join, new_parts, numbered, settle_all};
use artkit::place::{Facing, GameFrame};
use artkit::rand::Random;
use artkit::scene::Obj;
use artkit::{fbox, fcyl, fextrude, fsweep, kit, loft, revolve, ring, soft, swing};

// --- the writer's bureau (1x1): its back on the wall, the chair in front; the writer faces the bureau ------
// (content `seat.facing: "back"`). Built facing the front as it stands (2026-10-22: it was built the other way
// round and turned; the flap's clip is the same).

/// The writer's chair, its seat (pad top 0.46) centred at depth `z`, its back towards the front: raked back legs
/// in one curve each, turned front legs, a bowed crest rail over a vase-shaped splat, a stuffed pad.
fn bureau_chair(f: &GameFrame, z: f64) {
    let zb = z + 0.17; // the back legs' foot
    for s in [-1.0, 1.0] {
        let x = s * 0.18;
        fsweep!(
            f,
            numbered("back_leg"),
            &[
                (x, 0.0, zb),
                (x, 0.42, zb - 0.01),
                (x, 0.7, zb),
                (x, 0.93, zb + 0.025)
            ],
            0,
            M.honey,
            profile = rounded_rect(0.034, 0.03, 0.008, 1),
            up = (1, 0, 0),
            resolution = 2
        );
        revolve!(
            numbered("front_leg"),
            &[
                (0.0, 0.0),
                (0.013, 0.0),
                (0.016, 0.02),
                (0.014, 0.05),
                (0.019, 0.2),
                (0.016, 0.33),
                (0.02, 0.345),
                (0.02, 0.39),
                (0.0, 0.39)
            ],
            at = f.at(x, 0, z - 0.16),
            material = M.honey,
            segments = 10
        );
        kit::tube!(
            f,
            numbered("side_stretcher"),
            &[(x, 0.14, z - 0.16), (x, 0.14, zb)],
            0.009,
            M.honey,
            verts = 6,
            round = 1
        );
    }
    kit::tube!(
        f,
        "front_stretcher",
        &[(-0.18, 0.18, z - 0.16), (0.18, 0.18, z - 0.16)],
        0.009,
        M.honey,
        verts = 6,
        round = 1
    );
    // the seat: a board with a rounded front, the pad on it
    let mut seat = vec![(0.21, 0.19), (-0.21, 0.19)];
    seat.extend((0..9).map(|i| {
        (
            -0.21 + 0.42 * i as f64 / 8.0,
            -0.19 - 0.02 * sin(PI * i as f64 / 8.0),
        )
    }));
    fextrude!(
        f,
        "chair_seat",
        &smooth_outline(&seat, 1, true),
        0.035,
        (0, 0.41, z),
        M.honey,
        plane = Facing::Top,
        bevel = 0.008,
        bevel_segments = 1
    );
    soft!(
        f,
        "chair_pad",
        0.38,
        0.032,
        0.36,
        (0, 0.436, z - 0.01),
        M.seatpad,
        round = 0.018,
        puff = 0.008,
        piping = 0.004,
        flat = 2,
        round_segments = 1
    );
    // the crest rail, bowed back between the legs' tops, and the splat under it
    let crest = fsweep!(
        f,
        "chair_crest",
        &[
            (-0.2, 0.86, zb + 0.02),
            (-0.1, 0.875, zb + 0.028),
            (0.0, 0.88, zb + 0.032),
            (0.1, 0.875, zb + 0.028),
            (0.2, 0.86, zb + 0.02)
        ],
        0,
        M.honey,
        profile = rounded_rect(0.085, 0.024, 0.01, 1),
        up = (0, 0, 1),
        resolution = 2
    );
    ease_ends(crest);
    fbox!(
        f,
        "chair_rail",
        0.36,
        0.035,
        0.025,
        (0, 0.5, zb - 0.003),
        M.honey,
        bevel = 0.008,
        bevel_segments = 1
    );
    let splat = [
        (0.04, 0.0),
        (0.06, 0.06),
        (0.075, 0.14),
        (0.05, 0.22),
        (0.035, 0.27),
        (0.07, 0.31),
        (0.08, 0.34),
    ];
    let mut outline = splat.to_vec();
    outline.extend(splat.iter().rev().map(|&(u, v)| (-u, v)));
    fextrude!(
        f,
        "chair_splat",
        &smooth_outline(&outline, 1, true),
        0.016,
        (0, 0.5, zb + 0.012),
        M.honey,
        rot = (-4, 0, 0),
        bevel = 0.003,
        bevel_segments = 1
    );
}

pub fn writing_bureau() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let mut rnd = Random::new(5);
    let still = new_parts(|| {
        // two pedestals of drawers facing the writer, a kneehole between them, on a plinth each
        for s in [-1.0, 1.0] {
            let x = s * 0.31;
            fbox!(
                f,
                numbered("pedestal"),
                0.2,
                0.69,
                0.28,
                (x, 0.385, -0.3),
                M.body,
                bevel = 0.006,
                bevel_segments = 1
            );
            fbox!(
                f,
                numbered("pedestal_plinth"),
                0.21,
                0.04,
                0.29,
                (x, 0.02, -0.3),
                M.panel,
                bevel = 0.006,
                bevel_segments = 1
            );
            for (y, hh) in [(0.17, 0.2), (0.4, 0.2), (0.6, 0.16)] {
                kit::shaker!(
                    f,
                    numbered("drawer"),
                    0.172,
                    hh,
                    (x, y, -0.152),
                    0.016,
                    M.panel,
                    rail = 0.032
                );
                knob(f, &numbered("knob"), x, y, -0.144, 0.011, None, 8);
            }
        }
        fbox!(
            f,
            "modesty",
            0.42,
            0.6,
            0.015,
            (0, 0.42, -0.43),
            M.body,
            bevel = 0.004,
            bevel_segments = 1
        );
        fbox!(
            f,
            "modesty_panel",
            0.34,
            0.46,
            0.008,
            (0, 0.42, -0.418),
            M.panel,
            bevel = 0.003,
            bevel_segments = 1
        );
        // the desk top, its edge moulded
        fextrude!(
            f,
            "top",
            &rounded_rect(0.84, 0.32, 0.02, 2),
            0.04,
            (0, 0.75, -0.29),
            M.body,
            plane = Facing::Top,
            bevel = 0.01,
            bevel_segments = 2
        );
        // the hutch: a back, sides, a shelf of books over the pigeonholes, a moulded top and an arched crest
        fbox!(f, "hutch_back", 0.8, 0.52, 0.02, (0, 1.03, -0.43), M.body);
        for x in [-0.19, 0.19] {
            // panels on its back (on the wall; out in a room, it shows)
            fbox!(
                f,
                numbered("hutch_panel"),
                0.32,
                0.4,
                0.008,
                (x, 1.03, -0.442),
                M.panel,
                bevel = 0.003,
                bevel_segments = 1
            );
        }
        for x in [-0.4, 0.4] {
            fbox!(
                f,
                numbered("hutch_side"),
                0.022,
                0.52,
                0.17,
                (x, 1.03, -0.355),
                M.body,
                bevel = 0.004,
                bevel_segments = 1
            );
        }
        fbox!(
            f,
            "hutch_shelf",
            0.78,
            0.02,
            0.16,
            (0, 0.99, -0.355),
            M.body
        );
        fextrude!(
            f,
            "hutch_top",
            &rounded_rect(0.84, 0.2, 0.012, 2),
            0.03,
            (0, 1.305, -0.35),
            M.body,
            plane = Facing::Top,
            bevel = 0.008,
            bevel_segments = 2
        );
        let crest: Vec<(f64, f64)> = (0..21)
            .map(|i| {
                let k = 1.0 - i as f64 / 20.0;
                (-0.38 + 0.76 * k, 1.318 + 0.032 * pow(sin(PI * k), 0.7))
            })
            .collect();
        fextrude!(
            f,
            "crest",
            &crest,
            0.025,
            (0, 0, -0.42),
            M.panel,
            bevel = 0.004,
            bevel_segments = 1
        );
        for x in [-0.125, 0.0, 0.125] {
            fbox!(
                f,
                numbered("divider"),
                0.012,
                0.2,
                0.15,
                (x, 0.875, -0.35),
                M.body
            );
        }
        for (x, m, r) in [
            (0.19, M.page, 6),
            (0.06, M.cushion, -4),
            (-0.19, M.shade, 3),
        ] {
            fbox!(
                f,
                numbered("letters"),
                0.1,
                0.09,
                0.08,
                (x, 0.82, -0.35),
                m,
                rot = (0, r, 0)
            );
            fbox!(
                f,
                numbered("envelope"),
                0.1,
                0.004,
                0.06,
                (x, 0.869, -0.33),
                M.page,
                rot = (-60, r, 0)
            );
        }
        for x in [-0.32, 0.32] {
            // small drawers either side of the flap
            fbox!(
                f,
                numbered("little_case"),
                0.13,
                0.2,
                0.15,
                (x, 0.88, -0.355),
                M.body
            );
            for y in [0.83, 0.93] {
                fbox!(
                    f,
                    numbered("little_drawer"),
                    0.124,
                    0.08,
                    0.012,
                    (x, y, -0.27),
                    M.panel,
                    bevel = 0.003,
                    bevel_segments = 1
                );
                knob(f, &numbered("knob"), x, y, -0.264, 0.007, None, 6);
            }
        }
        books(
            f, &mut rnd, -0.2, 0.38, 1.0, -0.42, 0.12, 0.2, 0.95, false, false,
        );
        // an inkwell with its quill, a lamp and a mug of tea
        revolve!(
            "inkwell",
            &[
                (0.0, 0.0),
                (0.032, 0.0),
                (0.034, 0.03),
                (0.022, 0.045),
                (0.014, 0.05),
                (0.0, 0.05)
            ],
            at = f.at(-0.3, 0.77, -0.22),
            material = M.ink,
            segments = 12
        );
        fcyl!(
            f,
            "ink_cap",
            0.016,
            0.012,
            (-0.3, 0.826, -0.22),
            M.brass,
            verts = 10
        );
        fsweep!(
            f,
            "quill",
            &[
                (-0.29, 0.81, -0.22),
                (-0.27, 0.92, -0.23),
                (-0.24, 1.0, -0.245)
            ],
            0,
            M.page,
            profile = rounded_rect(0.004, 0.03, 0.0015, 1),
            up = (0, 1, 0),
            resolution = 2,
            scale = [0.15, 1.0, 0.3]
        );
        lamp(f, 0.33, 0.77, -0.21, 0.34, Some(M.lamp_amber));
        mug(f, -0.18, 0.77, -0.17, M.mug_b, None);
        bureau_chair(f, 0.092);
    });
    join(&still, "body", Some(P3::ZERO));
    // The writing flap, closed up over the pigeonholes, hinged on its foot: `fall_front` drops it onto the desk.
    let flap = new_parts(|| {
        kit::shaker!(
            f,
            "flap",
            0.5,
            0.2,
            (0.0, 0.875, -0.265),
            0.02,
            M.panel,
            rail = 0.035,
            raised = true
        );
        fbox!(
            f,
            "flap_lining",
            0.44,
            0.15,
            0.004,
            (0, 0.875, -0.277),
            M.seatpad
        );
        kit::pull!(
            f,
            "flap_pull",
            0.0,
            0.94,
            -0.255,
            0.06,
            M.brass,
            r = 0.004,
            out = 0.016
        );
        fcyl!(
            f,
            "flap_key",
            0.008,
            0.004,
            (0, 0.9, -0.254),
            M.brass,
            rot = (90, 0, 0),
            verts = 8
        );
    });
    let flap = hinged(join(&flap, "flap", None), f.at(0, 0.775, -0.265));
    let objs = settle_all(); // on its Model centre: the back on the wall face
    swing!(flap, "fall_front", 90.0, seconds = 0.8, axis = Axis::X);
    objs
}

// --- the book-club tea trolley (1x1): oak shelves on a brass frame, a tea set on a cloth ---------------------

/// Paints the faces of a turned part whose middles lie between heights `lo` and `hi` (its own, from its foot) in
/// `material`: a glazed band on china, with no more triangles.
fn band(obj: Obj, material: Mat, lo: f64, hi: f64) -> Obj {
    obj.edit(|o| {
        let k = o.mesh.slot(material);
        for f in 0..o.mesh.faces.len() {
            let c = o.mesh.corners(f);
            let mut z = 0.0_f32;
            for &v in c {
                z += o.mesh.verts[v as usize].z;
            }
            let z = (z / c.len() as f32) as f64;
            if lo <= z && z <= hi {
                o.mesh.faces[f].mat = k;
            }
        }
    });
    obj
}

/// The teapot, spout to the right (+x game): a lofted round body with a glazed band, a lid and knob, a spout
/// rising in one tapering curve, a looped handle.
fn teapot(f: &GameFrame, x: f64, y: f64, z: f64) {
    let P3 {
        x: bx,
        y: by,
        z: bz,
    } = f.at(x, y, z);
    let rows = [
        (0.0, 0.05),
        (0.006, 0.058),
        (0.03, 0.074),
        (0.058, 0.08),
        (0.085, 0.072),
        (0.102, 0.056),
        (0.11, 0.042),
    ];
    let secs: Vec<Vec<P3>> = rows
        .iter()
        .map(|&(h, r)| ring!(r, r, z = bz + h, verts = 18, at = (bx, by)))
        .collect();
    let pot = loft!(numbered("pot"), &secs, material = M.porcelain);
    band(pot, M.glaze, 0.045, 0.072);
    revolve!(
        numbered("pot_lid"),
        &[
            (0.0, 0.0),
            (0.045, 0.0),
            (0.045, 0.006),
            (0.032, 0.018),
            (0.012, 0.024),
            (0.008, 0.03),
            (0.012, 0.036),
            (0.0, 0.04)
        ],
        at = (bx, by, bz + 0.104),
        material = M.porcelain,
        segments = 12
    );
    fsweep!(
        f,
        numbered("spout"),
        &[
            (x + 0.07, y + 0.035, z),
            (x + 0.1, y + 0.06, z),
            (x + 0.118, y + 0.1, z),
            (x + 0.135, y + 0.12, z)
        ],
        0.016,
        M.porcelain,
        verts = 8,
        resolution = 2,
        scale = [1.0, 0.75, 0.5, 0.45]
    );
    fsweep!(
        f,
        numbered("pot_handle"),
        &[
            (x - 0.072, y + 0.09, z),
            (x - 0.11, y + 0.088, z),
            (x - 0.118, y + 0.055, z),
            (x - 0.096, y + 0.028, z),
            (x - 0.074, y + 0.026, z)
        ],
        0,
        M.porcelain,
        profile = rounded_rect(0.016, 0.008, 0.003, 1),
        up = (0, 1, 0),
        resolution = 2
    );
}

/// A teacup on its saucer, tea in it, a glazed band round its rim and an ear handle.
fn cup(f: &GameFrame, x: f64, y: f64, z: f64) {
    revolve!(
        numbered("saucer"),
        &[
            (0.0, 0.0),
            (0.032, 0.0),
            (0.053, 0.0073), // the rim rolled over
            (0.0552, 0.0085),
            (0.0553, 0.0101),
            (0.0538, 0.0112),
            (0.051, 0.0108),
            (0.03, 0.006),
            (0.0, 0.006)
        ],
        at = f.at(x, y, z),
        material = M.porcelain,
        segments = 12
    );
    let c = revolve!(
        numbered("cup"),
        &[
            (0.0, 0.0),
            (0.022, 0.0),
            (0.03, 0.012),
            (0.04, 0.04),
            (0.0428, 0.0494), // the lip rolled over
            (0.0424, 0.0503),
            (0.0405, 0.0504),
            (0.0393, 0.0497),
            (0.036, 0.04),
            (0.026, 0.014),
            (0.0, 0.014)
        ],
        at = f.at(x, y + 0.006, z),
        material = M.porcelain,
        segments = 12
    );
    band(c, M.glaze, 0.036, 0.051);
    fcyl!(
        f,
        numbered("cup_tea"),
        0.036,
        0.002,
        (x, y + 0.046, z),
        M.tea,
        verts = 10
    );
    fsweep!(
        f,
        numbered("cup_handle"),
        &kit::fillet!(
            &[
                (x + 0.04, y + 0.044, z),
                (x + 0.058, y + 0.042, z),
                (x + 0.056, y + 0.022, z),
                (x + 0.034, y + 0.02, z)
            ],
            0.008,
            steps = 2
        ),
        0.004,
        M.porcelain,
        verts = 5
    );
}

pub fn tea_trolley() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let mut rnd = Random::new(7);
    let (zb, zf) = (-0.41, 0.01);
    let still = new_parts(|| {
        for y in [0.32, 0.78] {
            fextrude!(
                f,
                numbered("shelf"),
                &rounded_rect(0.72, 0.42, 0.025, 2),
                0.03,
                (0, y, -0.2),
                M.honey,
                plane = Facing::Top,
                bevel = 0.006,
                bevel_segments = 1
            );
            for z in [zb, zf] {
                // brass gallery rails along the long sides, on little spindles
                kit::tube!(
                    f,
                    numbered("gallery"),
                    &[(-0.36, y + 0.05, z), (0.36, y + 0.05, z)],
                    0.0055,
                    M.brass,
                    verts = 6,
                    round = 1
                );
                for x in [-0.18, 0.0, 0.18] {
                    kit::tube!(
                        f,
                        numbered("spindle"),
                        &[(x, y + 0.015, z), (x, y + 0.05, z)],
                        0.003,
                        M.brass,
                        verts = 4,
                        round = 1
                    );
                }
            }
        }
        // the frame: each end one bent tube, up from the castors and over (the right end's top the push handle)
        for x in [-0.36, 0.36] {
            let top = if x < 0.0 { 0.86 } else { 0.92 };
            let mut path = vec![(x, 0.06, zb), (x, top, zb), (x, top, zf), (x, 0.06, zf)];
            if x > 0.0 {
                path = vec![
                    (x, 0.06, zb),
                    (x, 0.86, zb),
                    (x + 0.0, top, zb + 0.04),
                    (x, top, zf - 0.04),
                    (x, 0.86, zf),
                    (x, 0.06, zf),
                ];
            }
            kit::tube!(
                f,
                numbered("frame"),
                &kit::fillet!(&path, 0.04),
                0.012,
                M.brass,
                verts = 6,
                round = 1
            );
            for z in [zb, zf] {
                fbox!(
                    f,
                    numbered("fork"),
                    0.02,
                    0.03,
                    0.03,
                    (x, 0.055, z),
                    M.brass
                );
                fcyl!(
                    f,
                    numbered("castor"),
                    0.034,
                    0.018,
                    (x, 0.034, z),
                    M.dark,
                    rot = (0, 90, 0),
                    verts = 10
                );
            }
        }
        kit::tube!(
            f,
            "grip",
            &[(0.36, 0.92, zb + 0.08), (0.36, 0.92, zf - 0.08)],
            0.016,
            M.dark,
            verts = 8,
            round = 1
        );
        // the cloth on the top shelf, hanging over its front edge
        fbox!(f, "cloth", 0.6, 0.004, 0.42, (0, 0.797, -0.2), M.linen);
        fsweep!(
            f,
            "cloth_drop",
            &[(-0.3, 0.795, 0.012), (0.3, 0.795, 0.012)],
            0,
            M.linen,
            profile = vec![
                (0.004, 0.0),
                (0.004, -0.004),
                (-0.03, -0.008),
                (-0.07, -0.007),
                (-0.07, -0.003),
                (-0.03, -0.004),
                (0.0, 0.0)
            ],
            up = (0, 0, 1)
        );
        cup(f, 0.02, 0.799, -0.12);
        cup(f, 0.2, 0.799, -0.27);
        let jug = revolve!(
            "milk_jug",
            &[
                (0.0, 0.0),
                (0.026, 0.0),
                (0.032, 0.03),
                (0.03, 0.055),
                (0.026, 0.07),
                (0.03, 0.08),
                (0.027, 0.08),
                (0.022, 0.07),
                (0.0, 0.07)
            ],
            at = f.at(0.05, 0.799, -0.29),
            material = M.porcelain,
            segments = 12
        );
        band(jug, M.glaze, 0.06, 0.081);
        fsweep!(
            f,
            "jug_handle",
            &kit::fillet!(
                &[
                    (0.08, 0.86, -0.29),
                    (0.1, 0.855, -0.29),
                    (0.096, 0.82, -0.29),
                    (0.078, 0.815, -0.29)
                ],
                0.01
            ),
            0.004,
            M.porcelain,
            verts = 5
        );
        let bowl = revolve!(
            "sugar_bowl",
            &[
                (0.0, 0.0),
                (0.026, 0.0),
                (0.04, 0.02),
                (0.04, 0.035),
                (0.035, 0.045),
                (0.0, 0.045)
            ],
            at = f.at(0.22, 0.799, -0.1),
            material = M.porcelain,
            segments = 12
        );
        band(bowl, M.glaze, 0.018, 0.036);
        revolve!(
            "sugar_knob",
            &[(0.0, 0.0), (0.008, 0.0), (0.01, 0.008), (0.0, 0.016)],
            at = f.at(0.22, 0.844, -0.1),
            material = M.glaze,
            segments = 6
        );
        // below: the biscuit tin, a cake on its stand and two books
        fcyl!(
            f,
            "tin",
            0.1,
            0.12,
            (-0.2, 0.395, -0.22),
            M.glaze,
            verts = 16
        );
        fcyl!(
            f,
            "tin_lid",
            0.105,
            0.02,
            (-0.2, 0.465, -0.22),
            M.brass,
            verts = 16,
            bevel = 0.004,
            bevel_segments = 1
        );
        revolve!(
            "cake_stand",
            &[
                (0.0, 0.0),
                (0.05, 0.0),
                (0.05, 0.008),
                (0.018, 0.02),
                (0.016, 0.07),
                (0.12, 0.074),
                (0.122, 0.082),
                (0.0, 0.082)
            ],
            at = f.at(0.14, 0.335, -0.2),
            material = M.porcelain,
            segments = 16
        );
        // the cake, a slice cut from it, its icing running over the edge
        let mut cake = vec![(0.0, 0.0)];
        cake.extend(
            (40..361)
                .step_by(20)
                .map(|a| (0.09 * cosd(a as f64), 0.09 * sind(a as f64))),
        );
        fextrude!(
            f,
            "cake",
            &cake,
            0.06,
            (0.14, 0.447, -0.2),
            M.cake,
            plane = Facing::Top,
            bevel = 0.004,
            bevel_segments = 1
        );
        let mut icing = vec![(0.0, 0.0)];
        icing.extend(
            (40..361)
                .step_by(20)
                .map(|a| (0.092 * cosd(a as f64), 0.092 * sind(a as f64))),
        );
        fextrude!(
            f,
            "icing",
            &icing,
            0.012,
            (0.14, 0.483, -0.2),
            M.icing,
            plane = Facing::Top,
            bevel = 0.004,
            bevel_segments = 1
        );
        pile(f, &mut rnd, 0.0, -0.36, 0.335, 2, 0.18, 0.12);
    });
    join(&still, "body", Some(P3::ZERO));
    let pot = new_parts(|| teapot(f, -0.17, 0.799, -0.2));
    // `pour` tips it towards its spout about the foot's edge under the spout.
    let pot = hinged(join(&pot, "teapot", None), f.at(-0.12, 0.799, -0.2));
    let objs = settle_all(); // on its Model centre: the back on the wall face
    swing!(pot, "pour", -30.0, seconds = 0.8, axis = Axis::Y);
    objs
}
