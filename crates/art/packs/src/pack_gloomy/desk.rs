//! The candlelit writing desk (2x1): the writer's chair in the middle, the desk before them.

use super::common::{
    M, book, cabriole, candle, lathe, name, open_book, scroll, tube, turned, vase,
};
use super::music::settle_in_place;
use artkit::forms::rounded_rect;
use artkit::math::{Axis, P3, PI, cos, cosd, pow, sin, sind, sqrt};
use artkit::motion::{Grow, grow};
use artkit::parts::{join, new_parts};
use artkit::place::{Facing, GameFrame};
use artkit::scene::Obj;
use artkit::{fbox, fextrude, fsphere, fsweep, soft, torus};

const WICK: (f64, f64, f64) = (-0.25, 0.75 + 0.14, 0.38); // the lit candle's wick top

/// A pointed (gothic) arch opening `w` wide from height y0 to its apex at y1, centred on cx.
fn arch_hole(cx: f64, w: f64, y0: f64, y1: f64) -> Vec<(f64, f64)> {
    let r = w * 0.8; // each side's arc, centred across the opening
    let spring = y1 - sqrt(r * r - pow(r - w / 2.0, 2.0));
    let mut pts = vec![(cx - w / 2.0, y0), (cx + w / 2.0, y0)];
    let a0 = libm::acos((r - w / 2.0) / r); // where each arc meets the middle
    for k in 0..5 {
        // up the right side from the spring to the apex
        let a = a0 * k as f64 / 5.0;
        pts.push((cx + w / 2.0 - r + r * cos(a), spring + r * sin(a)));
    }
    pts.push((cx, y1));
    for k in (0..5).rev() {
        let a = a0 * k as f64 / 5.0;
        pts.push((cx - w / 2.0 + r - r * cos(a), spring + r * sin(a)));
    }
    pts
}

pub fn journal_desk() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let still = new_parts(|| {
        fbox!(
            f,
            "top",
            1.5,
            0.04,
            0.26,
            (0, 0.73, 0.32),
            M.rosewood,
            bevel = 0.012,
            bevel_segments = 2
        );
        // cabriole legs, and an apron only along the far edge and the ends (so the writer's hands have room under the
        // top), its lower edge a row of scallops
        for x in [-0.68, 0.68] {
            for z in [0.22, 0.42] {
                cabriole(
                    f,
                    &name("leg"),
                    x,
                    z,
                    0.712,
                    M.walnut,
                    0.025_f64.copysign(x),
                    0.014_f64.copysign(z - 0.32),
                    0.021,
                    12,
                );
            }
        }
        let edge: Vec<(f64, f64)> = (0..49)
            .map(|i| -0.66 + 1.32 * i as f64 / 48.0)
            .map(|u| (u, 0.625 + 0.016 * sin(PI * u / 0.165).abs()))
            .collect();
        fextrude!(
            f,
            "apron",
            &[vec![(0.66, 0.712), (-0.66, 0.712)], edge].concat(),
            0.02,
            (0, 0, 0.43),
            M.walnut,
            bevel = 0.003,
            bevel_segments = 1
        );
        for x in [-0.69, 0.69] {
            let side: Vec<(f64, f64)> = (0..11)
                .map(|i| {
                    (
                        0.4 - 0.18 * i as f64 / 10.0,
                        0.64 + 0.012 * sin(PI * i as f64 / 5.0).abs(),
                    )
                })
                .collect();
            fextrude!(
                f,
                name("apron_end"),
                &[vec![(0.22, 0.712), (0.4, 0.712)], side].concat(),
                0.02,
                (x, 0, 0),
                M.walnut,
                plane = Facing::Side,
                bevel = 0.003,
                bevel_segments = 1
            );
        }
        // the gallery along the far edge: a board pierced by a row of pointed arches, a moulded shelf over it on
        // scrolled brackets
        let holes: Vec<Vec<(f64, f64)>> = (0..9)
            .map(|k| arch_hole(-0.6 + 0.15 * k as f64, 0.08, -0.045, 0.045))
            .collect();
        fextrude!(
            f,
            "gallery",
            &rounded_rect(1.5, 0.14, 0.01, 2),
            0.03,
            (0, 0.82, 0.435),
            M.rosewood,
            holes = holes
        );
        fbox!(
            f,
            "shelf",
            1.44,
            0.02,
            0.12,
            (0, 0.90, 0.4),
            M.walnut,
            bevel = 0.006,
            bevel_segments = 2
        );
        fsweep!(
            f,
            "shelf_nose",
            &[(-0.72, 0.902, 0.341), (0.72, 0.902, 0.341)],
            0.011,
            M.walnut,
            verts = 8
        );
        for x in [-0.66, 0.66] {
            scroll(
                f,
                &name("bracket"),
                (x, 0.888, 0.35),
                (0.86, 0.38),
                M.walnut,
                -0.9,
                0.3,
                0.018,
                0.02,
                7,
            );
        }
        // on the desk: the open journal, an inkwell and quill, books, a wilted rose, candles on a dish
        open_book(
            f,
            "journal",
            0.0,
            0.75,
            0.3,
            0.2,
            0.24,
            M.plum,
            M.page,
            Some(M.ink),
            0.0,
        );
        lathe(
            f,
            "inkwell",
            0.3,
            0.75,
            0.32,
            &[
                (0.03, 0.0),
                (0.033, 0.004),
                (0.033, 0.034),
                (0.024, 0.044),
                (0.013, 0.05),
                (0.013, 0.058),
                (0.016, 0.06),
                (0.011, 0.06),
                (0.009, 0.052),
            ],
            M.inkwell,
            16,
            4.0,
        );
        torus!(
            "inkwell_collar",
            0.0145,
            0.003,
            at = f.at(0.3, 0.802, 0.32),
            material = M.brass,
            major_segments = 14,
            minor_segments = 4
        );
        fsweep!(
            f,
            "quill",
            &[(0.3, 0.765, 0.32), (0.33, 0.88, 0.342), (0.36, 0.99, 0.36)],
            0.0022,
            M.page,
            verts = 5
        );
        fsweep!(
            f,
            "quill_vane",
            &[
                (0.307, 0.79, 0.325),
                (0.322, 0.85, 0.338),
                (0.338, 0.91, 0.349),
                (0.355, 0.975, 0.358)
            ],
            0,
            M.page,
            profile = rounded_rect(0.0015, 0.034, 0.0007, 1),
            up = (0, 1, 0),
            scale = [0.15, 1.0, 0.95, 0.3],
            resolution = 2
        );
        let mut y = 0.75;
        for (w, h, m, r) in [
            (0.22, 0.05, M.sleeve_a, 0.0),
            (0.2, 0.04, M.sleeve_b, 10.0),
            (0.21, 0.045, M.sleeve_d, -6.0),
        ] {
            book(f, &name("book"), -0.5, y, 0.36, w, h, 0.15, m, r, None);
            y += h;
        }
        let top = vase(f, 0.6, 0.75, 0.38, 0.7);
        fsweep!(
            f,
            "rose_stem",
            &[
                (0.6, top - 0.04, 0.38),
                (0.61, top + 0.06, 0.375),
                (0.64, top + 0.1, 0.365),
                (0.665, top + 0.075, 0.352)
            ],
            0.0035,
            M.stem,
            verts = 5,
            resolution = 3
        );
        for (k, (dx, dy, dz, s)) in [
            (0.0, 0.0, 0.0, 1.0),
            (0.012, 0.004, 0.006, 0.8),
            (-0.01, 0.006, -0.004, 0.75),
            (0.004, -0.012, 0.002, 0.7),
        ]
        .into_iter()
        .enumerate()
        {
            let k = k as i32;
            fsphere!(
                f,
                name("rose_petal"),
                0.02 * s,
                (0.672 + dx, top + 0.062 + dy, 0.35 + dz),
                M.rose,
                scale = (1, 0.8, 0.75),
                rot = (30 * k, 20, 15 * k),
                segments = 8,
                rings = 5
            );
        }
        fsphere!(
            f,
            "rose_leaf",
            0.016,
            (0.62, top + 0.04, 0.378),
            M.stem,
            scale = (1.6, 0.2, 0.7),
            rot = (0, 0, 25),
            segments = 6,
            rings = 4
        );
        for (px, pz, a) in [(0.69, 0.3, 20), (0.55, 0.28, -40), (0.71, 0.36, 70)] {
            fsphere!(
                f,
                name("fallen_petal"),
                0.014,
                (px, 0.752, pz),
                M.rose,
                scale = (1.0, 0.75, 0.12),
                rot = (0, 0, a),
                segments = 8,
                rings = 4
            );
        }
        turned(
            f,
            "dish",
            -0.215,
            0.75,
            0.39,
            &[
                (0.0, 0.0),
                (0.06, 0.0),
                (0.07, 0.006),
                (0.068, 0.012),
                (0.062, 0.009),
                (0.0, 0.008),
            ],
            M.brass,
            22,
        );
        candle(
            f,
            WICK.0,
            0.758,
            WICK.2,
            WICK.1 - 0.758 - 0.008,
            0.022,
            false,
            false,
            3,
            11,
        );
        candle(f, -0.18, 0.758, 0.4, 0.09, 0.02, false, false, 2, 12);
        candle(f, 0.5, 0.91, 0.4, 0.12, 0.022, false, false, 2, 13);
        // the chair: a buttoned plum back in a show-wood frame under a pointed crest, sabre back legs, turned front
        // legs, an H stretcher; its seat at 0.45 under the writer
        fbox!(
            f,
            "seat",
            0.48,
            0.04,
            0.44,
            (0, 0.395, -0.06),
            M.walnut,
            bevel = 0.01,
            bevel_segments = 2
        );
        soft!(
            f,
            "seat_pad",
            0.44,
            0.025,
            0.4,
            (0, 0.4325, -0.05),
            M.plum,
            round = 0.012,
            puff = 0.006,
            flat = 2
        );
        let lean = 6.0;
        soft!(
            f,
            "back_pad",
            0.36,
            0.6,
            0.05,
            (0, 0.8, -0.288),
            M.plum,
            rot = (-lean, 0.0, 0.0),
            round = 0.02,
            puff = 0.016,
            flat = 3
        );
        for i in 0..3 {
            for j in 0..3 {
                let (u, v) = ((i - 1) as f64 * 0.1, (j - 1) as f64 * 0.17 + 0.02);
                let out =
                    0.025 + 0.016 * (1.0 - pow(u / 0.18, 2.0)) * (1.0 - pow(v / 0.3, 2.0)) - 0.004;
                fsphere!(
                    f,
                    name("button"),
                    0.008,
                    (
                        u,
                        0.8 + v * cosd(lean),
                        -0.288 - v * sind(lean) + out * cosd(lean)
                    ),
                    M.ink,
                    scale = (1, 0.5, 1),
                    segments = 6,
                    rings = 3
                );
            }
        }
        for x in [-0.21, 0.21] {
            fsweep!(
                f,
                name("back_leg"),
                &[(x, 0.0, -0.228), (x, 0.42, -0.27), (x, 1.165, -0.355)],
                0,
                M.walnut,
                profile = rounded_rect(0.04, 0.042, 0.01, 1),
                up = (1, 0, 0),
                resolution = 2,
                scale = [0.8, 1.0, 0.85]
            );
            fsphere!(
                f,
                name("finial"),
                0.017,
                (x, 1.18, -0.356),
                M.walnut,
                segments = 10,
                rings = 6
            );
            fsweep!(
                f,
                name("front_leg"),
                &[(x, 0.39, 0.13), (x, 0.36, 0.13), (x, 0.0, 0.14)],
                0.02,
                M.walnut,
                verts = 10,
                scale = [1.0, 0.85, 0.62]
            );
            tube(
                f,
                &name("side_stretcher"),
                &[(x, 0.13, 0.13), (x, 0.13, -0.246)],
                0.008,
                M.walnut,
                Some(6),
                1,
            );
        }
        tube(
            f,
            "cross_stretcher",
            &[(-0.21, 0.13, -0.06), (0.21, 0.13, -0.06)],
            0.008,
            M.walnut,
            Some(6),
            1,
        );
        let crest: Vec<(f64, f64, f64)> = [-0.21, -0.14, -0.07, 0.0, 0.07, 0.14, 0.21]
            .iter()
            .map(|&x: &f64| {
                (
                    x,
                    1.11 + 0.065 * pow(1.0 - x.abs() / 0.21, 1.4),
                    -0.35 - 0.003 * (1.0 - x.abs() / 0.21),
                )
            })
            .collect();
        fsweep!(
            f,
            "crest",
            &crest,
            0,
            M.walnut,
            profile = rounded_rect(0.034, 0.03, 0.01, 1),
            up = (0, 0, 1)
        );
        fsweep!(
            f,
            "back_rail",
            &[(-0.21, 0.47, -0.272), (0.21, 0.47, -0.272)],
            0,
            M.walnut,
            profile = rounded_rect(0.03, 0.03, 0.01, 1),
            up = (0, 0, 1)
        );
    });
    join(&still, "body", Some(P3::ZERO));
    // The flame on the tall candle, built full size on its wick and scaled to nothing there: `candle` lights it.
    let (wx, wy, wz) = WICK;
    let fl = new_parts(|| {
        fsphere!(
            f,
            "flame_outer",
            0.014,
            (wx, wy + 0.026, wz),
            M.flame,
            scale = (1, 1, 2.2),
            segments = 10,
            rings = 6
        );
        fsphere!(
            f,
            "flame_core",
            0.008,
            (wx, wy + 0.018, wz),
            M.candle_flame,
            scale = (1, 1, 1.8),
            segments = 8,
            rings = 5
        );
    });
    let flame = join(&fl, "flame", Some(f.at(wx, wy, wz)));
    flame.set_scale((0.001, 0.001, 0.001));
    let objs = settle_in_place();
    for axis in [Axis::X, Axis::Y, Axis::Z] {
        grow(flame, "candle", Grow { seconds: 0.6, axis });
    }
    objs
}
