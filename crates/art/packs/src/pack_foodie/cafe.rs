//! The espresso bar and the cookbook lectern.

use super::common::{_lofted, M, WALL};
use artkit::forms::rounded_rect;
use artkit::kit;
use artkit::materials::Mat;
use artkit::math::{Axis, P3, PI, TAU, cos, cosd, radians, sin, sind};
use artkit::motion::{Path, keys};
use artkit::parts::{aside, hinged, join, new_parts, numbered};
use artkit::place::{Facing, GameFrame};
use artkit::scene::{Obj, RotationMode};
use artkit::{bevel::ease, forms::eased, shapes::AUTO_WIDTH};
use artkit::{
    fbox, fcone, fcyl, fextrude, fsphere, fsweep, grow, jitter, revolve, settle, soft, swing,
};

// --- the espresso bar (1x1): a lever espresso machine on a teal-and-oak cart ----------------------------------
const GROUPS: [f64; 2] = [-0.16, 0.04]; // the group heads' x

/// An espresso cup on its saucer on `y`, crema in it, its ear handle to the side.
fn _cup(f: &GameFrame, x: f64, y: f64, z: f64, m: Option<Mat>) {
    let m = m.unwrap_or(M.cup);
    revolve!(
        numbered("saucer"),
        &[
            &[(0.0, 0.0)][..],
            // (its rim rolled over)
            &eased(
                &[(0.026, 0.0), (0.042, 0.007), (0.04, 0.009), (0.0, 0.004)],
                AUTO_WIDTH
            ),
        ]
        .concat(),
        at = f.at(x, y, z),
        material = M.cup,
        segments = 12
    );
    revolve!(
        numbered("cup"),
        &[
            (0.0, 0.0),
            (0.02, 0.0),
            (0.028, 0.016),
            (0.03, 0.044),
            (0.027, 0.044),
            (0.025, 0.026),
            (0.0, 0.026)
        ],
        at = f.at(x, y + 0.005, z),
        material = m,
        segments = 12
    );
    fcyl!(
        f,
        numbered("crema"),
        0.026,
        0.002,
        (x, y + 0.045, z),
        M.coffee,
        verts = 10
    );
    fsweep!(
        f,
        numbered("cup_handle"),
        &[
            (x + 0.028, y + 0.04, z),
            (x + 0.044, y + 0.036, z),
            (x + 0.042, y + 0.02, z),
            (x + 0.028, y + 0.016, z)
        ],
        0.0035,
        m,
        verts = 4,
        resolution = 2
    );
}

pub fn espresso_bar() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let zc = WALL + 0.25;
    let still = new_parts(|| {
        fextrude!(
            f,
            "top",
            &rounded_rect(0.76, 0.5, 0.02, 2),
            0.04,
            (0, 0.88, zc),
            M.oak_top,
            plane = Facing::Top,
            bevel = 0.008,
            bevel_segments = 1
        );
        for y in [0.42, 0.13] {
            fbox!(
                f,
                numbered("shelf"),
                0.7,
                0.03,
                0.44,
                (0, y, zc),
                M.oak_top,
                bevel = 0.005,
                bevel_segments = 1
            );
            fbox!(
                f,
                numbered("shelf_lip"),
                0.7,
                0.03,
                0.012,
                (0, y + 0.025, zc + 0.214),
                M.oak_top
            );
        }
        let post = rounded_rect(0.04, 0.04, 0.008, 1);
        for x in [-0.35, 0.35] {
            for z in [WALL + 0.03, zc + 0.22] {
                fsweep!(
                    f,
                    numbered("post"),
                    &[(x, 0.07, z), (x, 0.87, z)],
                    0.0,
                    M.teal,
                    profile = &post,
                    up = (1, 0, 0)
                );
                fcyl!(
                    f,
                    numbered("fork"),
                    0.012,
                    0.03,
                    (x, 0.07, z),
                    M.chrome,
                    verts = 8
                );
                fcyl!(
                    f,
                    numbered("castor"),
                    0.037,
                    0.022,
                    (x, 0.037, z),
                    M.dark,
                    rot = (0, 90, 0),
                    verts = 12
                );
            }
        }
        kit::tube!(
            f,
            "towel_rail",
            &[(-0.33, 0.75, 0.064), (0.33, 0.75, 0.064)],
            0.008,
            M.brass,
            verts = 8,
            round = 1
        );
        let towel = fsweep!(
            f,
            "towel",
            &kit::fillet!(
                &[
                    (0.16, 0.6, 0.06),
                    (0.16, 0.75, 0.056),
                    (0.16, 0.762, 0.064),
                    (0.16, 0.75, 0.072),
                    (0.16, 0.66, 0.074)
                ],
                0.008,
                steps = 5
            ),
            0.0,
            M.chalk,
            profile = rounded_rect(0.006, 0.16, 0.003, 1),
            up = (0, -1, 0)
        );
        jitter!(towel, 0.003, size = 0.06, seed = 2.0, up = 0.3);
        // the machine: a cream body on a chrome base, a copper boiler dome with a brass eagle, gauge, group heads, levers
        fextrude!(
            f,
            "machine_base",
            &rounded_rect(0.46, 0.32, 0.03, 2),
            0.03,
            (-0.06, 0.915, -0.26),
            M.chrome,
            plane = Facing::Top,
            bevel = 0.008,
            bevel_segments = 1
        );
        fextrude!(
            f,
            "machine",
            &rounded_rect(0.42, 0.3, 0.05, 3),
            0.3,
            (-0.06, 1.08, -0.26),
            M.machine,
            plane = Facing::Top,
            bevel = 0.025,
            bevel_segments = 2
        );
        let boiler = revolve!(
            "boiler",
            &[
                (0.0, 0.0),
                (0.12, 0.0),
                (0.122, 0.02),
                (0.12, 0.06),
                (0.1, 0.15),
                (0.06, 0.21),
                (0.02, 0.23),
                (0.0, 0.23)
            ],
            at = f.at(-0.06, 1.23, -0.27),
            material = M.copper,
            segments = 20
        );
        kit::paint(boiler, M.brass, |c| c.z < 0.025);
        revolve!(
            "eagle_post",
            &[
                (0.0, 0.0),
                (0.016, 0.0),
                (0.01, 0.02),
                (0.006, 0.03),
                (0.0, 0.03)
            ],
            at = f.at(-0.06, 1.455, -0.27),
            material = M.brass,
            segments = 8
        );
        fsphere!(
            f,
            "eagle",
            0.025,
            (-0.06, 1.482, -0.27),
            M.brass,
            scale = (1, 0.8, 1.1),
            segments = 8,
            rings = 5
        );
        for s in [-1, 1] {
            fextrude!(
                f,
                numbered("eagle_wing"),
                &[(0.0, 0.0), (0.06, 0.02), (0.05, 0.04), (0.0, 0.015)],
                0.004,
                (-0.06 + s as f64 * 0.012, 1.482, -0.27),
                M.brass,
                rot = (0, 0, 90 - 90 * s)
            );
        }
        fcyl!(
            f,
            "gauge",
            0.032,
            0.012,
            (0.1, 1.13, -0.105),
            M.chrome,
            rot = (90, 0, 0),
            verts = 14,
            bevel = 0.003,
            bevel_segments = 1
        );
        fcyl!(
            f,
            "gauge_face",
            0.026,
            0.004,
            (0.1, 1.13, -0.099),
            M.cup,
            rot = (90, 0, 0),
            verts = 14
        );
        fbox!(
            f,
            "gauge_needle",
            0.004,
            0.02,
            0.002,
            (0.105, 1.135, -0.096),
            M.tomato,
            rot = (0, -40, 0)
        );
        fextrude!(
            f,
            "drip_tray",
            &rounded_rect(0.4, 0.13, 0.015, 2),
            0.02,
            (-0.06, 0.915, -0.05),
            M.chrome,
            plane = Facing::Top,
            holes = vec![rounded_rect(0.36, 0.1, 0.01, 2)],
            bevel = 0.004,
            bevel_segments = 1
        );
        fbox!(
            f,
            "drip_grate",
            0.36,
            0.006,
            0.1,
            (-0.06, 0.912, -0.05),
            M.dark
        );
        for x in GROUPS {
            fbox!(
                f,
                numbered("group_neck"),
                0.05,
                0.04,
                0.05,
                (x, 1.12, -0.09),
                M.chrome,
                bevel = 0.008,
                bevel_segments = 1
            );
            revolve!(
                numbered("group"),
                &[
                    (0.0, 0.0),
                    (0.03, 0.0),
                    (0.036, 0.008),
                    (0.036, 0.034),
                    (0.03, 0.04),
                    (0.0, 0.04)
                ],
                at = f.at(x, 1.07, -0.07),
                material = M.chrome,
                segments = 12
            );
            revolve!(
                numbered("portafilter"),
                &[
                    (0.0, 0.0),
                    (0.02, 0.0),
                    (0.032, 0.014),
                    (0.034, 0.02),
                    (0.0, 0.02)
                ],
                at = f.at(x, 1.05, -0.07),
                material = M.chrome,
                segments = 12
            );
            fcone!(
                f,
                numbered("spout"),
                0.012,
                0.006,
                0.02,
                (x, 1.042, -0.07),
                M.chrome,
                verts = 8
            );
            fsweep!(
                f,
                numbered("pf_handle"),
                &[(x, 1.06, -0.04), (x, 1.058, 0.0), (x, 1.054, 0.07)],
                0.011,
                M.black,
                verts = 8,
                scale = [0.8, 1.0, 1.1]
            );
            kit::tube!(
                f,
                numbered("lever"),
                &[(x, 1.16, -0.1), (x, 1.25, -0.06), (x, 1.34, 0.0)],
                0.011,
                M.chrome,
                verts = 8,
                round = 2
            );
            fsphere!(
                f,
                numbered("lever_knob"),
                0.025,
                (x, 1.35, 0.005),
                M.black,
                scale = (1, 1, 1.3),
                segments = 10,
                rings = 6
            );
            _cup(f, x, 0.935, -0.07, None);
        }
        // the grinder at the back right, a milk jug before it
        fextrude!(
            f,
            "grinder_base",
            &rounded_rect(0.12, 0.14, 0.02, 2),
            0.12,
            (0.26, 0.96, -0.33),
            M.black,
            plane = Facing::Top,
            bevel = 0.01,
            bevel_segments = 1
        );
        revolve!(
            "hopper",
            &[
                (0.0, 0.0),
                (0.035, 0.0),
                (0.035, 0.03),
                (0.07, 0.14),
                (0.072, 0.15),
                (0.0, 0.15)
            ],
            at = f.at(0.26, 1.02, -0.33),
            material = M.machine,
            segments = 14
        );
        let beans = fsphere!(
            f,
            "hopper_beans",
            0.068,
            (0.26, 1.165, -0.33),
            M.beans,
            scale = (1, 1, 0.35),
            segments = 12,
            rings = 6
        );
        jitter!(beans, 0.003, size = 0.015);
        revolve!(
            "milk_jug",
            &[
                (0.0, 0.0),
                (0.042, 0.0),
                (0.046, 0.02),
                (0.04, 0.08),
                (0.042, 0.11),
                (0.038, 0.11),
                (0.034, 0.08),
                (0.0, 0.08)
            ],
            at = f.at(0.24, 0.9, -0.1),
            material = M.chrome,
            segments = 14
        );
        fsweep!(
            f,
            "milk_handle",
            &kit::fillet!(
                &[
                    (0.278, 1.0, -0.1),
                    (0.3, 0.99, -0.1),
                    (0.296, 0.93, -0.1),
                    (0.275, 0.925, -0.1)
                ],
                0.012,
                steps = 2
            ),
            0.005,
            M.chrome,
            verts = 5
        );
        // below: cups on the middle shelf, bean sacks and a syrup bottle
        for (i, x) in [-0.26, -0.18, -0.1].into_iter().enumerate() {
            _cup(
                f,
                x,
                0.435,
                -0.18,
                if i == 1 { Some(M.book_teal) } else { None },
            );
        }
        _lofted(
            f,
            "syrup",
            0.2,
            0.435,
            -0.25,
            &[
                (0.0, 0.03),
                (0.005, 0.035),
                (0.17, 0.035),
                (0.2, 0.014),
                (0.24, 0.012),
            ],
            M.bottle,
            10,
        );
        fcyl!(
            f,
            "syrup_pump",
            0.006,
            0.05,
            (0.2, 0.7, -0.25),
            M.chrome,
            verts = 6
        );
        fbox!(
            f,
            "syrup_nozzle",
            0.04,
            0.008,
            0.008,
            (0.215, 0.722, -0.25),
            M.chrome
        );
        for (name, w, hh, d, x, z, m) in [
            ("sack_a", 0.24, 0.2, 0.16, 0.17, -0.24, M.sack),
            ("sack_b", 0.2, 0.16, 0.14, -0.16, -0.22, M.wicker),
        ] {
            let sack = soft!(
                f,
                name,
                w,
                hh,
                d,
                (x, 0.145 + hh / 2.0, z),
                m,
                round = 0.05,
                puff = 0.02,
                flat = 2
            );
            jitter!(sack, 0.008, size = 0.06, seed = x);
            fsweep!(
                f,
                format!("{name}_tie"),
                &[
                    (x - w * 0.3, 0.145 + hh, z),
                    (x, 0.145 + hh + 0.025, z),
                    (x + w * 0.3, 0.145 + hh, z)
                ],
                0.015,
                m,
                verts = 6,
                resolution = 2
            );
        }
    });
    join(&still, "body", Some(P3::from((0, 0, 0))));
    // The coffee running from the left group's spout into its cup, built full length and scaled to nothing at the
    // spout: `brew` grows it.
    let gx = GROUPS[0];
    let stream = new_parts(|| {
        for dx in [-0.006, 0.006] {
            fcyl!(
                f,
                numbered("stream"),
                0.004,
                0.08,
                (gx + dx, 0.992, -0.07),
                M.coffee,
                verts = 6
            );
        }
    });
    let stream = join(&stream, "stream", Some(f.at(gx, 1.032, -0.07)));
    stream.set_scale_axis(2, 0.001);
    // The steam wand on the machine's right side, hanging from its ball joint: `brew` swings it out.
    let wand = new_parts(|| {
        fsphere!(
            f,
            "wand_joint",
            0.018,
            (0.165, 1.12, -0.16),
            M.chrome,
            segments = 10,
            rings = 6
        );
        kit::tube!(
            f,
            "wand",
            &kit::fillet!(
                &[
                    (0.165, 1.12, -0.16),
                    (0.168, 1.0, -0.15),
                    (0.176, 0.97, -0.13)
                ],
                0.02
            ),
            0.007,
            M.chrome,
            verts = 6,
            round = 1
        );
        fcone!(
            f,
            "wand_tip",
            0.008,
            0.004,
            0.02,
            (0.176, 0.955, -0.128),
            M.chrome,
            verts = 8
        );
    });
    let wand = hinged(join(&wand, "wand", None), f.at(0.165, 1.12, -0.16));
    let objs = settle!(centre = false);
    grow!(stream, "brew", seconds = 0.4);
    swing!(wand, "brew", 40.0, seconds = 0.6, axis = Axis::Y);
    objs
}

// --- the cookbook lectern (1x1): a turned walnut lectern with an open cookbook; a stool of cookbooks -------
const DESK: (f64, f64, f64) = (0.0, 1.02, -0.04); // the reading desk's middle (game)
const TILT: f64 = 25.0; // and its slope (degrees, front down)

/// A point on the desk, `ly` up off it and `lz` down its slope towards the front, as game coordinates.
fn _on_desk(lx: f64, ly: f64, lz: f64) -> (f64, f64, f64) {
    let (c, s) = (cosd(TILT), sind(TILT));
    let (x, y, z) = DESK;
    (x + lx, y + ly * c - lz * s, z + ly * s + lz * c)
}

/// A book's box with its page edges cream on the same mesh: the three sides but its spine (front) when lying.
fn _pages(obj: Obj) -> Obj {
    let normals: Vec<artkit::math::V3> = obj.read(|o| o.mesh.face_normals());
    obj.edit(|o| {
        let k = o.mesh.slot(M.page);
        for (face, n) in o.mesh.faces.iter_mut().zip(&normals) {
            if n.z.abs() < 0.5 && n.y > -0.5 {
                face.mat = k;
            }
        }
    });
    obj
}

pub fn cookbook_lectern() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let still = new_parts(|| {
        // three feet curving out from the column's foot, the turned column, the sloped desk with its book ledge
        for k in 0..3 {
            let a = (120 * k + 90) as f64;
            let (c, s) = (cosd(a), sind(a));
            fsweep!(
                f,
                numbered("foot"),
                &[
                    (0.03 * c, 0.12, -0.05 + 0.03 * s),
                    (0.11 * c, 0.06, -0.05 + 0.11 * s),
                    (0.18 * c, 0.02, -0.05 + 0.18 * s),
                    (0.2 * c, 0.016, -0.05 + 0.2 * s)
                ],
                0.0,
                M.walnut,
                profile = rounded_rect(0.03, 0.035, 0.01, 1),
                up = (0, 0, 1),
                resolution = 2
            );
            fsphere!(
                f,
                numbered("foot_pad"),
                0.018,
                (0.2 * c, 0.016, -0.05 + 0.2 * s),
                M.walnut,
                scale = (1, 1, 0.9),
                segments = 8,
                rings = 4
            );
        }
        revolve!(
            "column",
            &[
                (0.0, 0.08),
                (0.05, 0.08),
                (0.055, 0.1),
                (0.04, 0.14),
                (0.03, 0.18),
                (0.034, 0.3),
                (0.03, 0.42),
                (0.04, 0.46),
                (0.03, 0.5),
                (0.026, 0.72),
                (0.04, 0.78),
                (0.06, 0.86),
                (0.06, 0.876),
                (0.056, 0.88), // (its top edge chamfered)
                (0.0, 0.88)
            ],
            at = f.at(0, 0.05, -0.05),
            material = M.walnut,
            segments = 14
        );
        fextrude!(
            f,
            "desk",
            &rounded_rect(0.56, 0.42, 0.02, 2),
            0.04,
            DESK,
            M.walnut,
            plane = Facing::Top,
            rot = (TILT, 0.0, 0.0),
            bevel = 0.01,
            bevel_segments = 2
        );
        fbox!(
            f,
            "ledge",
            0.5,
            0.03,
            0.03,
            _on_desk(0.0, 0.03, 0.2),
            M.honey_wood,
            rot = (TILT, 0.0, 0.0),
            bevel = 0.008,
            bevel_segments = 1
        );
        // a scalloped apron under the desk's front edge, and the bracket the desk sits on
        let mut apron = vec![(0.22, 0.0), (-0.22, 0.0)];
        apron.extend((0..13).map(|i| {
            (
                -0.22 + 0.44 * i as f64 / 12.0,
                -0.04 + 0.012 * cos(TAU * i as f64 / 4.0),
            )
        }));
        fextrude!(
            f,
            "apron",
            &apron,
            0.016,
            _on_desk(0.0, -0.03, 0.16),
            M.walnut,
            rot = (TILT, 0.0, 0.0),
            bevel = 0.003,
            bevel_segments = 1
        );
        fbox!(
            f,
            "desk_bracket",
            0.14,
            0.05,
            0.2,
            _on_desk(0.0, -0.04, 0.0),
            M.walnut,
            rot = (TILT, 0.0, 0.0),
            bevel = 0.008,
            bevel_segments = 1
        );
        // two brass page clips holding the book open
        for s in [-1, 1] {
            let sf = s as f64;
            kit::tube!(
                f,
                numbered("clip"),
                &[
                    _on_desk(sf * 0.245, 0.0, 0.17),
                    _on_desk(sf * 0.245, 0.07, 0.17),
                    _on_desk(sf * 0.2, 0.07, 0.12)
                ],
                0.003,
                M.brass,
                verts = 4,
                round = 2
            );
            fsphere!(
                f,
                numbered("clip_end"),
                0.007,
                _on_desk(sf * 0.2, 0.07, 0.12),
                M.brass,
                segments = 6,
                rings = 4
            );
        }
        // the open cookbook: its cover, two page blocks curving up from the spine, a recipe photo and a ribbon
        fbox!(
            f,
            "book_cover",
            0.48,
            0.012,
            0.34,
            _on_desk(0.0, 0.026, 0.0),
            M.cover,
            rot = (TILT, 0.0, 0.0),
            bevel = 0.004,
            bevel_segments = 1
        );
        for s in [-1, 1] {
            let sf = s as f64;
            let prof = [
                (sf * 0.005, 0.0),
                (sf * 0.22, 0.0),
                (sf * 0.225, 0.02),
                (sf * 0.15, 0.03),
                (sf * 0.06, 0.028),
                (sf * 0.012, 0.012),
            ];
            let pts = [_on_desk(0.0, 0.032, -0.155), _on_desk(0.0, 0.032, 0.155)];
            // (the block's fore-edge rounded, its ends eased)
            let swapped: Vec<(f64, f64)> = eased(&prof, AUTO_WIDTH)
                .iter()
                .map(|&(u, v)| (v, u))
                .collect();
            let pages = fsweep!(
                f,
                numbered("pages"),
                &pts,
                0.0,
                M.page,
                profile = swapped,
                up = (0, 0, 1)
            );
            ease(pages, 60.0, 1);
        }
        fbox!(
            f,
            "photo",
            0.13,
            0.002,
            0.09,
            _on_desk(0.11, 0.064, -0.06),
            M.photo,
            rot = (TILT, 0.0, 0.0)
        );
        for k in 0..4 {
            fbox!(
                f,
                numbered("text"),
                0.15,
                0.002,
                0.008,
                _on_desk(0.11, 0.064, 0.03 + 0.025 * k as f64),
                M.board,
                rot = (TILT, 0.0, 0.0)
            );
        }
        fsweep!(
            f,
            "ribbon",
            &[
                _on_desk(-0.02, 0.064, -0.16),
                _on_desk(-0.02, 0.064, 0.15),
                _on_desk(-0.02, 0.03, 0.18),
                _on_desk(-0.02, -0.03, 0.2)
            ],
            0.0,
            M.tomato,
            profile = rounded_rect(0.002, 0.012, 0.001, 1),
            up = (0, 0, 1),
            resolution = 2
        );
    });
    join(&still, "body", Some(P3::from((0, 0, 0))));
    // a turned stool with a pile of cookbooks, a recipe-card box and a jar of wooden spoons on the floor by it:
    // drawn, not the lectern's size
    let beside = new_parts(|| {
        revolve!(
            "stool",
            &[
                (0.0, 0.0),
                (0.13, 0.0),
                (0.13, 0.02),
                (0.11, 0.04),
                (0.1, 0.2),
                (0.11, 0.36),
                (0.13, 0.38),
                (0.15, 0.4),
                (0.15, 0.416),
                (0.146, 0.42), // (its seat's edge chamfered)
                (0.0, 0.42)
            ],
            at = f.at(0.29, 0, -0.27),
            material = M.honey_wood,
            segments = 16
        );
        let mut y = 0.42;
        for (w, hh, m, r) in [
            (0.26, 0.05, M.book_teal, 6),
            (0.24, 0.04, M.book_mustard, -8),
            (0.25, 0.06, M.cover, 3),
            (0.22, 0.04, M.book_sage, 12),
        ] {
            _pages(fbox!(
                f,
                numbered("cookbook"),
                w,
                hh,
                w * 0.75,
                (0.29, y + hh / 2.0, -0.27),
                m,
                rot = (0, 0, r)
            ));
            y += hh;
        }
        // a recipe-card box on the floor, its lid propped open on its cards
        fbox!(
            f,
            "card_box",
            0.16,
            0.09,
            0.11,
            (-0.12, 0.045, 0.1),
            M.honey_wood,
            rot = (0, 0, 12),
            bevel = 0.004,
            bevel_segments = 1
        );
        fbox!(
            f,
            "card_box_lid",
            0.16,
            0.008,
            0.11,
            (-0.12, 0.13, 0.04),
            M.honey_wood,
            rot = (-60, 0, 12),
            bevel = 0.003,
            bevel_segments = 1
        );
        for k in 0..4 {
            fbox!(
                f,
                numbered("card"),
                0.14,
                0.06,
                0.002,
                (-0.12 + 0.004 * k as f64, 0.1, 0.09 - 0.018 * k as f64),
                [M.page, M.photo][k % 2],
                rot = (-10, 0, 12)
            );
        }
        revolve!(
            "spoon_jar",
            &[
                (0.0, 0.0),
                (0.045, 0.0),
                (0.05, 0.14),
                (0.045, 0.14),
                (0.04, 0.012),
                (0.0, 0.012)
            ],
            at = f.at(-0.31, 0, -0.3),
            material = M.cup,
            segments = 12
        );
        for (dx, dz, lean) in [(-0.015, 0.0, -10), (0.015, 0.01, 8), (0.0, -0.015, 3)] {
            let tip = (-0.31 + dx * 3.0 + lean as f64 * 0.002, 0.3, -0.3 + dz * 2.0);
            fsweep!(
                f,
                numbered("spoon"),
                &[(-0.31 + dx, 0.03, -0.3 + dz), tip],
                0.007,
                M.honey_wood,
                verts = 5,
                scale = [0.8, 1.0]
            );
            fsphere!(
                f,
                numbered("spoon_bowl"),
                0.022,
                (tip.0, 0.31, tip.2),
                M.honey_wood,
                scale = (1, 0.5, 1.3),
                segments = 8,
                rings = 5
            );
        }
    });
    aside(&beside, "beside");
    // The page on the right-hand block (game -x), curling gently up from the spine, its origin on the spine, then
    // tilted onto the desk; `page` turns it over about the spine (its own Y, after the tilt).
    let (hx, hy, hz) = _on_desk(0.0, 0.0625, 0.0);
    let page = new_parts(|| {
        let curl: Vec<(f64, f64, f64)> = (0..7)
            .map(|k| {
                let kf = k as f64;
                (
                    hx - 0.215 * kf / 6.0,
                    hy + 0.012 * sin(PI * kf / 6.0) * (1.0 - kf / 12.0),
                    hz,
                )
            })
            .collect();
        fsweep!(
            f,
            "page_leaf",
            &curl,
            0.0,
            M.page,
            profile = rounded_rect(0.003, 0.3, 0.001, 1),
            up = (0, 0, 1)
        );
        for k in 0..5 {
            fbox!(
                f,
                numbered("page_text"),
                0.15,
                0.002,
                0.008,
                (hx - 0.11, hy + 0.0105, hz - 0.08 + 0.035 * k as f64),
                M.board
            );
        }
    });
    let page = join(&page, "page", Some(f.at(hx, hy, hz)));
    page.set_rotation_mode(RotationMode::YXZ); // the tilt (X) after the turn about the spine (Y)
    page.set_rotation_euler((radians(TILT), 0.0, 0.0));
    let objs = settle!(centre = false);
    keys(
        page,
        "page",
        Path::Rotation,
        1,
        &[(0.0, 0.0), (0.9, radians(-172.0))],
    );
    objs
}
