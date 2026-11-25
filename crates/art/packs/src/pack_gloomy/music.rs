//! The midnight record cabinet and the nocturne cello.

use super::common::{M, WALL, candle, leg, open_book, vase};
use artkit::forms::{rounded_rect, smooth_outline};
use artkit::math::{Axis, P3, PI, TAU, cos, cosd, pow, sin, sind, sqrt};
use artkit::motion::{Spin, spin};
use artkit::parts::{hinged, join, new_parts, numbered};
use artkit::place::{Facing, GameFrame};
use artkit::rand::Random;
use artkit::scene::Obj;
use artkit::{fbox, fcyl, fextrude, fsphere, fsweep, kit, loft, settle, slide, swing, torus};

// --- the midnight record cabinet (1x1): a walnut console, a turntable under a hinged lid -------------------
const TOP: f64 = 0.62;

pub fn record_cabinet() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let mut rnd = Random::new(8);
    let zf = -0.01; // the console's front
    let zc = (zf + WALL) / 2.0;
    let d = zf - WALL;
    let still = new_parts(|| {
        // splayed, tapered mid-century legs with brass ferrules
        for x in [-0.4, 0.4] {
            for z in [WALL + 0.05, zf - 0.05] {
                leg(
                    f,
                    &numbered("leg"),
                    x,
                    z,
                    0.145,
                    M.ink,
                    0.019,
                    (0.018_f64.copysign(x), 0.014_f64.copysign(z - zc)),
                    Some(M.brass),
                    8,
                );
            }
        }
        fbox!(
            f,
            "bottom",
            0.9,
            0.03,
            d,
            (0, 0.155, zc),
            M.walnut,
            bevel = 0.006,
            bevel_segments = 1
        );
        // a shallow arched apron under the front
        let edge: Vec<(f64, f64)> = (0..11)
            .map(|i| -0.36 + 0.72 * i as f64 / 10.0)
            .map(|u| {
                let q = 1.0 - pow(u / 0.36, 2.0);
                (u, 0.136 - 0.024 * sqrt(if q > 0.0 { q } else { 0.0 }))
            })
            .collect();
        fextrude!(
            f,
            "apron",
            &[vec![(0.36, 0.145), (-0.36, 0.145)], edge].concat(),
            0.018,
            (0, 0, zf - 0.014),
            M.walnut,
            bevel = 0.003,
            bevel_segments = 1
        );
        fbox!(f, "deck", 0.9, 0.03, d, (0, 0.545, zc), M.walnut); // under the turntable and the top
        for x in [-0.435, 0.435] {
            fbox!(
                f,
                numbered("side"),
                0.03,
                TOP - 0.14,
                d,
                (x, (TOP + 0.14) / 2.0, zc),
                M.walnut,
                bevel = 0.008,
                bevel_segments = 1
            );
        }
        // the back: dark inside the bays, walnut outside (it shows when the cabinet stands free)
        fbox!(f, "back", 0.84, 0.42, 0.004, (0, 0.35, WALL + 0.008), M.ink);
        fbox!(
            f,
            "back_outside",
            0.84,
            0.42,
            0.006,
            (0, 0.35, WALL + 0.003),
            M.walnut
        );
        fbox!(
            f,
            "divider",
            0.02,
            0.37,
            d - 0.02,
            (0, 0.355, zc - 0.01),
            M.walnut,
            bevel = 0.004,
            bevel_segments = 1
        );
        // the left (game -x) bay: LP sleeves standing on end, one record pulled half out
        let mut x = -0.415;
        while x < -0.03 {
            let t = rnd.uniform(0.008, 0.016);
            let m = rnd.choice(&[M.sleeve_a, M.sleeve_b, M.sleeve_c, M.sleeve_d]);
            let h = if rnd.random() < 0.8 { 0.31 } else { 0.26 };
            let n = numbered("lp");
            let r = rnd.uniform(-4.0, 4.0);
            fbox!(
                f,
                n,
                t,
                h,
                0.31,
                (x + t / 2.0, 0.17 + h / 2.0, WALL + 0.025 + 0.155),
                m,
                rot = (0.0, r, 0.0)
            );
            x += t + 0.003;
        }
        fcyl!(
            f,
            "lp_out",
            0.15,
            0.003,
            (-0.2, 0.33, -0.1),
            M.vinyl,
            rot = (0, 90, 0),
            verts = 24,
            smooth = false
        );
        fcyl!(
            f,
            "lp_out_label",
            0.05,
            0.0035,
            (-0.2, 0.33, -0.1),
            M.label,
            rot = (0, 90, 0),
            verts = 12,
            smooth = false
        );
        // the right bay: a slatted door in a walnut frame, a brass bar pull
        fbox!(
            f,
            "door_back",
            0.41,
            0.36,
            0.012,
            (0.215, 0.355, zf - 0.012),
            M.ink
        );
        fextrude!(
            f,
            "door_frame",
            &rounded_rect(0.41, 0.36, 0.006, 2),
            0.02,
            (0.215, 0.355, zf + 0.004),
            M.walnut,
            holes = vec![rounded_rect(0.37, 0.32, 0.003, 1)],
            bevel = 0.003,
            bevel_segments = 1
        );
        let pitch = 0.37 / 8.0;
        for k in 0..8 {
            fbox!(
                f,
                numbered("door_slat"),
                0.03,
                0.32,
                0.014,
                (0.03 + pitch * (k as f64 + 0.5), 0.355, zf + 0.002),
                M.slat
            );
        }
        kit::pull!(
            f,
            "door_pull",
            0.055,
            0.4,
            zf + 0.014,
            0.09,
            M.brass,
            r = 0.005,
            out = 0.022,
            upright = true,
            verts = 6
        );
        // the top: the turntable's well on the left, a board on the right with sleeves and a candle
        fbox!(
            f,
            "top_right",
            0.45,
            0.06,
            d,
            (0.225, TOP - 0.03, zc),
            M.walnut,
            bevel = 0.01,
            bevel_segments = 1
        );
        fbox!(
            f,
            "well_front",
            0.45,
            0.06,
            0.02,
            (-0.225, TOP - 0.03, zf - 0.01),
            M.walnut,
            bevel = 0.004,
            bevel_segments = 1
        );
        fbox!(
            f,
            "well_back",
            0.45,
            0.06,
            0.02,
            (-0.225, TOP - 0.03, WALL + 0.01),
            M.walnut,
            bevel = 0.004,
            bevel_segments = 1
        );
        fbox!(
            f,
            "well_side",
            0.02,
            0.06,
            d,
            (-0.44, TOP - 0.03, zc),
            M.walnut,
            bevel = 0.004,
            bevel_segments = 1
        );
        fbox!(
            f,
            "well_floor",
            0.43,
            0.01,
            d - 0.04,
            (-0.225, 0.565, zc),
            M.ink
        );
        // the turntable: its plinth, speed buttons, the tonearm on its pivot with a counterweight and a headshell
        fbox!(
            f,
            "plinth",
            0.38,
            0.03,
            0.32,
            (-0.225, 0.585, zc),
            M.mauve,
            bevel = 0.008,
            bevel_segments = 1
        );
        for k in 0..2 {
            fcyl!(
                f,
                numbered("speed"),
                0.008,
                0.006,
                (-0.39 + 0.022 * k as f64, 0.601, zf - 0.075),
                M.chrome,
                verts = 8
            );
        }
        kit::turned!(
            f,
            "tonearm_base",
            -0.08,
            0.6,
            -0.36,
            &[
                (0.0, 0.0),
                (0.022, 0.0),
                (0.022, 0.008),
                (0.013, 0.012),
                (0.009, 0.026),
                (0.0, 0.027),
            ],
            M.chrome,
            segments = 10
        );
        kit::tube!(
            f,
            "tonearm",
            &[
                (-0.075, 0.624, -0.39),
                (-0.08, 0.624, -0.36),
                (-0.1, 0.624, -0.28),
                (-0.13, 0.623, -0.205),
                (-0.155, 0.621, -0.172),
            ],
            0.0035,
            M.chrome,
            verts = 6,
            round = 2
        );
        fcyl!(
            f,
            "counterweight",
            0.012,
            0.028,
            (-0.075, 0.624, -0.39),
            M.chrome,
            rot = (90, 0, 0),
            verts = 12
        );
        fbox!(
            f,
            "headshell",
            0.016,
            0.005,
            0.032,
            (-0.162, 0.6205, -0.16),
            M.chrome,
            rot = (0, 0, 28),
            bevel = 0.0015,
            bevel_segments = 1
        );
        fbox!(
            f,
            "cartridge",
            0.01,
            0.006,
            0.016,
            (-0.164, 0.6175, -0.155),
            M.ink,
            rot = (0, 0, 28)
        );
        fcyl!(
            f,
            "arm_rest",
            0.004,
            0.024,
            (-0.05, 0.612, -0.24),
            M.chrome,
            verts = 6
        );
        for (k, (m, tilt)) in [(M.sleeve_a, -14), (M.sleeve_b, -9), (M.sleeve_c, -4)]
            .into_iter()
            .enumerate()
        {
            fbox!(
                f,
                numbered("sleeve"),
                0.31,
                0.31,
                0.01,
                (0.24, TOP + 0.15, WALL + 0.04 + k as f64 * 0.025),
                m,
                rot = (tilt, 0, 0)
            );
        }
        candle(f, 0.34, TOP, -0.12, 0.1, 0.03, true, true, 2, 3);
        // a floor vase of dried branches at the front right, each branching, with honesty-pod seed heads
        let top = vase(f, 0.33, 0.0, 0.3, 1.2);
        for (a, (dx, dz, h)) in [(-0.04, -0.02, 0.4), (0.06, 0.02, 0.33), (0.0, 0.05, 0.36)]
            .into_iter()
            .enumerate()
        {
            let a = a as i32;
            let (p0, p1) = ((0.33, top - 0.05, 0.3), (0.33 + dx, top + h, 0.3 + dz));
            let mid = (0.33 + dx * 0.45, top + h * 0.45, 0.3 + dz * 0.45 + 0.01);
            fsweep!(
                f,
                numbered("twig"),
                &[p0, mid, p1],
                0.005,
                M.twig,
                verts = 5,
                resolution = 2,
                scale = [1.0, 0.8, 0.45]
            );
            for (k, (bx, by, bz)) in [(0.05, 0.08, 0.0)].into_iter().enumerate() {
                let kf = k as f64;
                let at = (
                    0.33 + dx * (0.55 + 0.15 * kf),
                    top + h * (0.55 + 0.15 * kf),
                    0.3 + dz * (0.55 + 0.15 * kf) + 0.005,
                );
                let end = (at.0 + bx * (1 - 2 * (a % 2)) as f64, at.1 + by, at.2 + bz);
                fsweep!(
                    f,
                    numbered("sprig"),
                    &[at, end],
                    0.0028,
                    M.twig,
                    verts = 4,
                    scale = [1.0, 0.5]
                );
                fcyl!(
                    f,
                    numbered("pod"),
                    0.016,
                    0.003,
                    end,
                    M.dried,
                    rot = (90, 0, 30 * a + 50 * k as i32),
                    verts = 7,
                    smooth = false
                );
            }
            for k in 0..2 {
                fcyl!(
                    f,
                    numbered("pod"),
                    0.018 - 0.002 * k as f64,
                    0.003,
                    (
                        p1.0 + 0.012 * (2 * k - 1) as f64,
                        p1.1 - 0.01 * k as f64,
                        p1.2
                    ),
                    M.dried,
                    rot = (90, 0, 40 * k + 20 * a),
                    verts = 7,
                    smooth = false
                );
            }
        }
    });
    join(&still, "body", Some(P3::ZERO));
    // The platter and its record, origin on the spindle: `platter_spin` turns it (33 rpm) while the record plays.
    let plat = new_parts(|| {
        kit::turned!(
            f,
            "platter",
            -0.225,
            0.6,
            zc,
            &[(0.0, 0.0), (0.131, 0.0), (0.131, 0.012), (0.0, 0.012)],
            M.platter,
            segments = 28
        );
        kit::turned!(
            f,
            "record",
            -0.225,
            0.6,
            zc,
            &[
                (0.0, 0.012),
                (0.125, 0.012),
                (0.1255, 0.015),
                (0.123, 0.016),
                (0.0, 0.016),
            ],
            M.vinyl,
            segments = 28
        );
        fcyl!(
            f,
            "record_label",
            0.04,
            0.002,
            (-0.225, 0.617, zc),
            M.label,
            verts = 16
        );
        fbox!(
            f,
            "label_mark",
            0.012,
            0.0015,
            0.03,
            (-0.225, 0.6185, zc + 0.022),
            M.page
        );
        fcyl!(
            f,
            "spindle",
            0.004,
            0.012,
            (-0.225, 0.62, zc),
            M.chrome,
            verts = 6
        );
    });
    let plat = hinged(join(&plat, "platter", None), f.at(-0.225, 0.606, zc));
    // The lid over the well, hinged along its back's top edge (as a piano hinge, so its back edge tips forward, not
    // into the wall): a walnut frame round a sunk panel, a finger lip at the front; `lid_open` lifts it.
    let lid = new_parts(|| {
        fextrude!(
            f,
            "lid",
            &rounded_rect(0.45, d, 0.008, 2),
            0.025,
            (-0.225, TOP + 0.0125, zc),
            M.walnut,
            plane = Facing::Top,
            holes = vec![rounded_rect(0.37, d - 0.08, 0.004, 1)],
            bevel = 0.004,
            bevel_segments = 1
        );
        fbox!(
            f,
            "lid_panel",
            0.372,
            0.017,
            d - 0.078,
            (-0.225, TOP + 0.0105, zc),
            M.slat,
            bevel = 0.002,
            bevel_segments = 1
        );
        fbox!(
            f,
            "lid_pull",
            0.09,
            0.012,
            0.012,
            (-0.225, TOP + 0.006, zf + 0.006),
            M.slat,
            bevel = 0.005,
            bevel_segments = 1
        );
    });
    let lid = hinged(join(&lid, "lid", None), f.at(-0.225, TOP + 0.025, WALL));
    let objs = settle!(centre = false);
    swing!(lid, "lid_open", -80.0, seconds = 0.9, axis = Axis::X);
    spin(
        plat,
        "platter_spin",
        Spin {
            turns: -1.0,
            seconds: 1.8,
            ..Spin::default()
        },
    );
    objs
}

// --- the nocturne cello (1x1): a stool, the cello on its stand, a music stand --------------------------------
const REST: (f64, f64, f64) = (0.22, 0.02, 0.36); // the end pin's foot on the stand
const PLAY: (f64, f64, f64) = (0.0, 0.02, 0.3); // and played

/// The cello, upright with its end pin's foot at REST, its face to the front: a body lofted from its outline, the
/// plates arched over the ribs, f-holes, a tailpiece and a carved bridge, the neck, fingerboard, pegbox and scroll,
/// four strings.
fn cello_body(f: &GameFrame) {
    let (x, y0, z) = REST;
    let out = kit::figure8((0.43, 0.233, 0.22), (0.75, 0.177, 0.19), 9, 1);
    let cy = 0.58;
    let secs = [
        (0.35, -0.071),
        (0.72, -0.066),
        (0.93, -0.056),
        (1.0, -0.045),
        (1.0, 0.04),
        (0.93, 0.052),
        (0.72, 0.061),
        (0.35, 0.066),
    ];
    let sections: Vec<Vec<P3>> = secs
        .iter()
        .map(|&(s, dz)| {
            out.iter()
                .map(|&(u, v)| f.at(x + s * u, cy + s * (v - cy), z + dz))
                .collect()
        })
        .collect();
    loft!("cello_body", &sections, material = M.varnish);
    // the f-holes, between the waist's bouts
    for s in [-1.0, 1.0] {
        fsweep!(
            f,
            numbered("f_hole"),
            &[
                (x + s * 0.074, 0.49, z + 0.063),
                (x + s * 0.062, 0.535, z + 0.064),
                (x + s * 0.07, 0.585, z + 0.064),
                (x + s * 0.059, 0.63, z + 0.063)
            ],
            0.0035,
            M.ebony,
            verts = 6,
            resolution = 2,
            scale = [1.7, 0.65, 0.65, 1.6]
        );
    }
    // the end pin out of its collar
    fsweep!(
        f,
        "end_pin",
        &[(x, y0, z), (x, 0.235, z)],
        0.006,
        M.chrome,
        verts = 8,
        scale = [0.35, 1.0]
    );
    fcyl!(
        f,
        "end_collar",
        0.016,
        0.02,
        (x, 0.215, z),
        M.ebony,
        verts = 12,
        bevel = 0.003,
        bevel_segments = 1
    );
    // the tailpiece with its fine tuners, the carved bridge (feet, kidneys, a heart)
    let tail = [
        (0.016, 0.0),
        (0.034, 0.165),
        (0.03, 0.195),
        (0.0, 0.203),
        (-0.03, 0.195),
        (-0.034, 0.165),
        (-0.016, 0.0),
    ];
    fextrude!(
        f,
        "tailpiece",
        &smooth_outline(&tail, 1, true),
        0.008,
        (x, 0.27, z + 0.069),
        M.ebony,
        bevel = 0.002,
        bevel_segments = 1
    );
    for k in 0..4 {
        fcyl!(
            f,
            numbered("tuner"),
            0.0035,
            0.008,
            (x - 0.018 + 0.012 * k as f64, 0.455, z + 0.076),
            M.chrome,
            rot = (90, 0, 0),
            verts = 6
        );
    }
    let arch = [
        (0.045, 0.0),
        (0.03, 0.0),
        (0.024, 0.008),
        (0.0, 0.011),
        (-0.024, 0.008),
        (-0.03, 0.0),
        (-0.045, 0.0),
        (-0.043, 0.012),
        (-0.036, 0.019),
        (-0.042, 0.027),
        (-0.03, 0.033),
        (0.0, 0.0355),
        (0.03, 0.033),
        (0.042, 0.027),
        (0.036, 0.019),
        (0.043, 0.012),
    ];
    let heart: Vec<(f64, f64)> = (0..8)
        .map(|k| {
            (
                0.004 * cos(TAU * k as f64 / 8.0),
                0.023 + 0.004 * sin(TAU * k as f64 / 8.0),
            )
        })
        .collect();
    fextrude!(
        f,
        "bridge",
        &arch,
        0.005,
        (x, 0.53, z + 0.066),
        M.bridge,
        plane = Facing::Top,
        holes = vec![heart]
    );
    // the neck, its fingerboard, the nut, the pegbox with its pegs, the scroll
    let prof: Vec<(f64, f64)> = (0..9)
        .map(|i| {
            (
                0.0225 * cos(PI * i as f64 / 8.0),
                0.03 * sin(PI * i as f64 / 8.0),
            )
        })
        .collect();
    fsweep!(
        f,
        "neck",
        &[(x, 0.9, z + 0.058), (x, 1.21, z + 0.058)],
        0,
        M.varnish,
        profile = prof,
        up = (1, 0, 0),
        scale = [1.08, 0.85],
        angle = 70
    );
    fsweep!(
        f,
        "fingerboard",
        &[(x, 0.62, z + 0.083), (x, 1.205, z + 0.065)],
        0,
        M.ebony,
        profile = rounded_rect(0.052, 0.012, 0.004, 1),
        up = (1, 0, 0),
        scale = [1.0, 0.64]
    );
    fbox!(
        f,
        "nut",
        0.034,
        0.006,
        0.012,
        (x, 1.205, z + 0.068),
        M.bridge
    );
    for s in [-1.0, 1.0] {
        fbox!(
            f,
            numbered("pegbox_cheek"),
            0.008,
            0.13,
            0.034,
            (x + s * 0.016, 1.27, z + 0.045),
            M.varnish,
            bevel = 0.003,
            bevel_segments = 1
        );
    }
    fbox!(
        f,
        "pegbox_back",
        0.04,
        0.13,
        0.01,
        (x, 1.27, z + 0.03),
        M.varnish,
        bevel = 0.003,
        bevel_segments = 1
    );
    fbox!(
        f,
        "pegbox_well",
        0.024,
        0.12,
        0.02,
        (x, 1.27, z + 0.045),
        M.ebony
    );
    kit::scroll!(
        f,
        "scroll",
        (x, 1.335, z + 0.045),
        (1.385, z + 0.04),
        M.varnish,
        turns = -1.2,
        end = 0.22,
        t = 0.042,
        w = 0.02,
        steps = 14
    );
    for (s, yk) in [(-1.0, 1.245), (1.0, 1.265), (-1.0, 1.29), (1.0, 1.31)] {
        fcyl!(
            f,
            numbered("peg"),
            0.005,
            0.07,
            (x + s * 0.032, yk, z + 0.045),
            M.ebony,
            rot = (0, 90, 0),
            verts = 8
        );
        fsphere!(
            f,
            numbered("peg_head"),
            0.017,
            (x + s * 0.07, yk, z + 0.045),
            M.ebony,
            scale = (0.35, 0.75, 1.0),
            segments = 10,
            rings = 6
        );
    }
    for k in 0..4 {
        let dx = -0.015 + 0.01 * k as f64;
        fsweep!(
            f,
            numbered("string"),
            &[
                (x + dx * 0.9, 0.465, z + 0.078),
                (x + dx, 0.532, z + 0.101),
                (x + dx * 0.75, 1.205, z + 0.075)
            ],
            0.0012,
            M.string,
            verts = 4
        );
    }
}

pub fn cello() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let still = new_parts(|| {
        // the stool: a padded round seat (top 0.45, hips on the footprint's middle) on a turned column and three
        // splayed legs with a foot ring
        let sz = -0.08;
        kit::turned!(
            f,
            "stool_top",
            0.0,
            0.38,
            sz,
            &[
                (0.0, 0.0),
                (0.19, 0.0),
                (0.2, 0.012),
                (0.2, 0.03),
                (0.19, 0.04),
                (0.0, 0.04),
            ],
            M.rosewood,
            segments = 28
        );
        kit::lathe!(
            f,
            "stool_cushion",
            0.0,
            0.42,
            sz,
            &[
                (0.186, 0.0),
                (0.194, 0.008),
                (0.19, 0.02),
                (0.168, 0.028),
                (0.1, 0.0305),
                (0.0, 0.031),
            ],
            M.plum,
            verts = 28
        );
        torus!(
            "stool_piping",
            0.192,
            0.0035,
            at = f.at(0, 0.428, sz),
            material = M.mauve,
            major_segments = 28,
            minor_segments = 4
        );
        fsphere!(
            f,
            "stool_button",
            0.012,
            (0, 0.449, sz),
            M.mauve,
            scale = (1, 1, 0.45),
            segments = 8,
            rings = 4
        );
        kit::turned!(
            f,
            "stool_column",
            0.0,
            0.12,
            sz,
            &[
                (0.0, 0.0),
                (0.035, 0.0),
                (0.035, 0.02),
                (0.022, 0.05),
                (0.028, 0.14),
                (0.02, 0.22),
                (0.03, 0.24),
                (0.04, 0.26),
                (0.0, 0.26),
            ],
            M.rosewood,
            segments = 14
        );
        for k in 0..3 {
            let a = (90 + 120 * k) as f64;
            let (c, s) = (cosd(a), sind(a));
            fsweep!(
                f,
                numbered("stool_leg"),
                &[
                    (0.03 * c, 0.2, sz + 0.03 * s),
                    (0.11 * c, 0.09, sz + 0.11 * s),
                    (0.16 * c, 0.0, sz + 0.16 * s)
                ],
                0.014,
                M.rosewood,
                verts = 8,
                resolution = 2,
                scale = [1.0, 0.9, 0.75]
            );
        }
        torus!(
            "stool_ring",
            0.105,
            0.007,
            at = f.at(0, 0.1, sz),
            material = M.rosewood,
            major_segments = 20,
            minor_segments = 5
        );
        // the cello's stand: three curved feet round a hub, a cup for the end pin, a post behind with a cradle for the
        // back and a yoke round the neck
        let (x, _, z) = REST;
        let hz = z - 0.08;
        kit::turned!(
            f,
            "stand_hub",
            x,
            0.0,
            hz,
            &[
                (0.0, 0.0),
                (0.025, 0.0),
                (0.025, 0.04),
                (0.018, 0.06),
                (0.0, 0.06),
            ],
            M.rosewood,
            segments = 12
        );
        for k in 0..3 {
            let a = (90 + 120 * k) as f64;
            let (c, s) = (cosd(a), sind(a));
            fsweep!(
                f,
                numbered("stand_foot"),
                &[
                    (x + 0.015 * c, 0.035, hz + 0.015 * s),
                    (x + 0.07 * c, 0.02, hz + 0.07 * s),
                    (x + 0.125 * c, 0.008, hz + 0.125 * s)
                ],
                0,
                M.rosewood,
                profile = rounded_rect(0.018, 0.024, 0.006, 1),
                up = (0, 0, 1),
                resolution = 2,
                scale = [1.0, 0.9, 0.8]
            );
            fsphere!(
                f,
                numbered("stand_pad"),
                0.012,
                (x + 0.125 * c, 0.008, hz + 0.125 * s),
                M.rosewood,
                scale = (1, 1, 0.65),
                segments = 8,
                rings = 4
            );
        }
        kit::turned!(
            f,
            "stand_cup",
            x,
            0.0,
            z,
            &[
                (0.0, 0.0),
                (0.03, 0.0),
                (0.033, 0.016),
                (0.03, 0.03),
                (0.022, 0.027),
                (0.0, 0.022),
            ],
            M.rosewood,
            segments = 14
        );
        kit::tube!(
            f,
            "stand_heel",
            &[(x, 0.015, z - 0.02), (x, 0.03, hz)],
            0.011,
            M.rosewood,
            verts = 8,
            round = 1
        );
        fsweep!(
            f,
            "stand_post",
            &[(x, 0.05, z - 0.12), (x, 1.0, z - 0.12)],
            0.012,
            M.rosewood,
            verts = 10,
            scale = [1.15, 0.8]
        );
        fcyl!(
            f,
            "stand_collar",
            0.016,
            0.03,
            (x, 0.62, z - 0.12),
            M.rosewood,
            verts = 12,
            bevel = 0.004,
            bevel_segments = 1
        );
        kit::tube!(
            f,
            "stand_arm",
            &[(x, 0.985, z - 0.12), (x, 0.985, z - 0.005)],
            0.008,
            M.rosewood,
            verts = 8,
            round = 1
        );
        kit::tube!(
            f,
            "stand_yoke",
            &kit::fillet!(
                &[
                    (x - 0.03, 0.985, z + 0.045),
                    (x - 0.03, 0.985, z - 0.005),
                    (x + 0.03, 0.985, z - 0.005),
                    (x + 0.03, 0.985, z + 0.045),
                ],
                0.012
            ),
            0.006,
            M.rosewood,
            verts = 6,
            round = 1
        );
        kit::tube!(
            f,
            "stand_cradle",
            &[(x, 0.3, z - 0.12), (x, 0.3, z - 0.075)],
            0.009,
            M.rosewood,
            verts = 8,
            round = 1
        );
        fbox!(
            f,
            "stand_cradle_bar",
            0.13,
            0.022,
            0.016,
            (x, 0.3, z - 0.07),
            M.rosewood,
            bevel = 0.006,
            bevel_segments = 1
        );
        // the bow hung on the stand's post: a cambered stick, its frog and tip, the hair
        let (bx, bz) = (x + 0.035, z - 0.135);
        kit::tube!(
            f,
            "bow_hook",
            &[
                (x, 0.95, z - 0.12),
                (bx, 0.95, bz + 0.005),
                (bx, 0.965, bz + 0.005),
            ],
            0.003,
            M.chrome,
            verts = 5,
            round = 1
        );
        fsweep!(
            f,
            "bow",
            &[(bx, 0.25, bz), (bx, 0.6, bz - 0.008), (bx, 0.95, bz)],
            0.0045,
            M.wick,
            verts = 6,
            resolution = 3,
            scale = [1.0, 0.85, 0.6]
        );
        fbox!(
            f,
            "bow_frog",
            0.012,
            0.05,
            0.026,
            (bx, 0.29, bz + 0.012),
            M.ebony,
            bevel = 0.003,
            bevel_segments = 1
        );
        fbox!(
            f,
            "bow_tip",
            0.008,
            0.025,
            0.018,
            (bx, 0.945, bz + 0.008),
            M.page,
            bevel = 0.002,
            bevel_segments = 1
        );
        fbox!(
            f,
            "bow_hair",
            0.009,
            0.64,
            0.0015,
            (bx, 0.62, bz + 0.022),
            M.page
        );
        // an open book of music on the floor, and a candle on a dish at the left
        open_book(
            f,
            "music",
            -0.188,
            0.0,
            0.24,
            0.135,
            0.2,
            M.plum,
            M.page,
            Some(M.ink),
            0.0,
        );
        candle(f, -0.267, 0.0, -0.2, 0.16, 0.035, true, true, 3, 5);
    });
    join(&still, "body", Some(P3::ZERO));
    // The cello, built upright on its stand with its origin on the end pin's foot: `off_stand` moves it across
    // to between the player's knees and leans it back 14 degrees.
    let body = new_parts(|| cello_body(f));
    let body = join(&body, "cello", Some(f.at(REST.0, REST.1, REST.2)));
    let objs = settle!(centre = false);
    slide!(
        body,
        "off_stand",
        (-(PLAY.0 - REST.0), -(PLAY.2 - REST.2), 0),
        seconds = 0.9
    ); // Blender axes
    swing!(body, "off_stand", -14.0, seconds = 0.9, axis = Axis::X);
    objs
}
