//! Hay: the round bale and the stack of square bales, and their far versions.

use super::common::{F, M, fr};
use artkit::forms::rounded_rect;
use artkit::math::{TAU, cos, radians, sin};
use artkit::parts::{join, join_all, new_parts, settle};
use artkit::py;
use artkit::scene::Obj;
use artkit::{cube, cylinder, fsweep, jitter, revolve, soft, sphere, torus};

// --- hay -------------------------------------------------------------------------------------------------
// A round bale as they come off the baler: 1.2 m across and 1.2 m long, up to an adult's chest (it was 0.99 x 0.86 m,
// waist-high), on 2 x 2 tiles. Its rings and twine are spaced for its size.
const BALE_R: f64 = 0.6;
const BALE_L: f64 = 1.2;
const TWINE: [f64; 3] = [-0.35, 0.0, 0.35];

/// A round bale lying on its side: its hay a little lumpy, each end rolled in one long spiral to a tuft at its core,
/// three bands of twine round it, their ends tucked in, and loose wisps of straw on its top and ends.
#[allow(non_snake_case)]
pub fn hayBale() -> Vec<Obj> {
    let y = BALE_R - 0.012; // a little settled into the ground
    let hl = BALE_L / 2.0;
    let bale = revolve!(
        "bale",
        &[
            (0.0, -hl),
            (BALE_R - 0.09, -hl),
            (BALE_R - 0.03, -hl + 0.02),
            (BALE_R, -hl + 0.08),
            (BALE_R + 0.004, -0.3),
            (BALE_R + 0.005, 0.0),
            (BALE_R + 0.004, 0.3),
            (BALE_R, hl - 0.08),
            (BALE_R - 0.03, hl - 0.02),
            (BALE_R - 0.09, hl),
            (0.0, hl)
        ],
        at = F.at(0, y, 0),
        material = M.hay,
        segments = 32,
        rot = (0, 90, 0)
    );
    jitter!(bale, 0.007, size = 0.12, seed = 2.0);
    for s in [-1, 1] {
        let sf = s as f64;
        // the roll's spiral on each end, from the core out
        let mut pts = Vec::new();
        for k in 0..84 {
            let t = k as f64 / 83.0;
            let a = sf * t * 4.2 * TAU;
            let r = 0.05 + 0.45 * t;
            pts.push((sf * (hl - 0.002), y + r * sin(a), r * cos(a)));
        }
        fsweep!(
            F,
            format!("spiral{s}"),
            &pts,
            0.013,
            M.hay_dark,
            verts = 4,
            scale = [0.7, 1.0, 1.0, 1.1]
        );
        sphere!(
            format!("core{s}"),
            0.07,
            at = F.at(sf * (hl - 0.01), y, 0),
            material = M.hay_dark,
            scale = (0.35, 1, 1),
            segments = 8,
            rings = 5
        );
    }
    for (k, x) in TWINE.into_iter().enumerate() {
        torus!(
            format!("twine{k}"),
            BALE_R + 0.006,
            0.008,
            at = F.at(x, y, 0),
            material = M.twine,
            rot = (0, 90, 0),
            major_segments = 32,
            minor_segments = 4
        );
    }
    // the twine's tied ends, hanging a little down the front
    for (k, x) in TWINE[..2].iter().copied().enumerate() {
        fsweep!(
            F,
            format!("twine_end{k}"),
            &[
                (x, y + 0.3, BALE_R - 0.05),
                (x + 0.03, y + 0.15, BALE_R - 0.005),
                (x + 0.05, y - 0.02, BALE_R + 0.004)
            ],
            0.006,
            M.twine,
            verts = 4,
            resolution = 2
        );
    }
    // loose wisps of straw lying on it
    for i in 0..18 {
        let fi = i as f64;
        let a = radians(40.0 + 100.0 * py::fmod(fi * 0.618, 1.0));
        let x = -hl + 0.08 + (BALE_L - 0.16) * py::fmod(fi * 0.381, 1.0);
        let d = radians(-25.0 + 50.0 * py::fmod(fi * 0.7, 1.0));
        let p0 = (x, y + (BALE_R + 0.008) * sin(a), (BALE_R + 0.008) * cos(a));
        let p1 = (
            x + 0.12 * sin(d),
            y + (BALE_R + 0.012) * sin(a + 0.12),
            (BALE_R + 0.012) * cos(a + 0.12),
        );
        fsweep!(
            F,
            format!("wisp{i}"),
            &[p0, p1],
            0.005,
            M.straw,
            verts = 3,
            scale = [1.0, 0.5]
        );
    }
    for s in [-1, 1] {
        let sf = s as f64;
        for i in 0..5 {
            let a = TAU * i as f64 / 5.0 + 0.4 * sf;
            let r = 0.2 + 0.07 * i as f64;
            let p0 = (sf * (hl - 0.004), y + r * sin(a), r * cos(a));
            let p1 = (
                sf * (hl + 0.006),
                y + (r + 0.1) * sin(a + 0.3),
                (r + 0.1) * cos(a + 0.3),
            );
            fsweep!(
                F,
                format!("end_wisp{s}{i}"),
                &[p0, p1],
                0.006,
                M.straw,
                verts = 3,
                scale = [1.0, 0.4]
            );
        }
    }
    settle(&[join_all("hayBale")])
}

/// The bale from afar: a plainer roll, one dark ring on each end, the twine flat bands.
#[allow(non_snake_case)]
pub fn hayBale_far() -> Vec<Obj> {
    let (y, hl) = (BALE_R - 0.012, BALE_L / 2.0);
    revolve!(
        "bale",
        &[
            (0.0, -hl),
            (BALE_R - 0.06, -hl),
            (BALE_R, -hl + 0.07),
            (BALE_R + 0.005, 0.0),
            (BALE_R, hl - 0.07),
            (BALE_R - 0.06, hl),
            (0.0, hl)
        ],
        at = F.at(0, y, 0),
        material = M.hay,
        segments = 14,
        rot = (0, 90, 0)
    );
    for s in [-1, 1] {
        cylinder!(
            format!("end{s}"),
            0.36,
            0.01,
            at = F.at(s as f64 * (hl - 0.002), y, 0),
            material = M.hay_dark,
            rot = (0, 90, 0),
            verts = 10
        );
    }
    for (k, x) in TWINE.into_iter().enumerate() {
        cylinder!(
            format!("twine{k}"),
            BALE_R + 0.02,
            0.016,
            at = F.at(x, y, 0),
            material = M.twine,
            rot = (0, 90, 0),
            verts = 14
        );
    }
    settle(&[join_all("hayBale_far")])
}

/// A small square bale lying flat, `x` across and its middle `y` up: pressed in flakes along its length, each a
/// little proud of the next, tied twice round its length, the cut ends darker.
fn square_bale(name: &str, x: f64, y: f64, detail: bool) {
    if !detail {
        cube!(
            name,
            (0.92, 0.86, 0.44),
            at = F.at(x, y, 0),
            material = M.hay
        );
        for dz in [-0.22, 0.22] {
            cube!(
                format!("{name}_twine{}", fr(dz)),
                (0.935, 0.02, 0.455),
                at = F.at(x, y, dz),
                material = M.twine
            );
        }
        return;
    }
    let n = 9;
    let edges: Vec<f64> = (0..n + 1)
        .map(|i| {
            let fi = i as f64;
            if 0 < i && i < n {
                -0.46 + 0.92 * (fi + 0.18 * sin(fi * 2.3 + x * 7.0)) / n as f64
            } else {
                -0.46 + 0.92 * fi / n as f64
            }
        })
        .collect();
    for i in 0..n {
        let fi = i as f64;
        let (u0, u1) = (edges[i], edges[i + 1]);
        let k = 1.0 + 0.01 * sin(fi * 2.7 + x * 5.0) + 0.006 * cos(fi * 1.3);
        soft!(
            F,
            format!("{name}_flake{i}"),
            u1 - u0 + 0.004,
            0.44 * k - 0.004,
            0.86 * (2.0 - k) - 0.004,
            (
                x + (u0 + u1) / 2.0,
                y + 0.004 * sin(fi * 1.9),
                0.005 * cos(fi * 2.2)
            ),
            if i == 0 || i == n - 1 {
                M.hay_dark
            } else {
                M.hay
            },
            round = 0.014,
            round_segments = 1,
            flat = 1
        );
    }
    for dz in [-0.22, 0.22] {
        let lp: Vec<(f64, f64, f64)> = rounded_rect(0.935, 0.455, 0.04, 2)
            .iter()
            .map(|&(u, v)| (x + u, y + v, dz))
            .collect();
        fsweep!(
            F,
            format!("{name}_twine{}", fr(dz)),
            &lp,
            0.008,
            M.twine,
            verts = 4,
            closed = true
        );
    }
    for i in 0..4 {
        // loose straw on its top
        let fi = i as f64;
        let u = -0.35 + 0.23 * fi;
        fsweep!(
            F,
            format!("{name}_wisp{i}"),
            &[
                (x + u, y + 0.223, -0.3 + 0.17 * fi),
                (x + u + 0.1, y + 0.226, -0.24 + 0.17 * fi)
            ],
            0.005,
            M.straw,
            verts = 3,
            scale = [1.0, 0.5]
        );
    }
}

/// Three square bales: two side by side and one on top, a little askew.
#[allow(non_snake_case)]
pub fn hayStack_detail(detail: bool) -> Vec<Obj> {
    square_bale("bale_l", -0.48, 0.225, detail);
    square_bale("bale_r", 0.48, 0.225, detail);
    let top = new_parts(|| {
        square_bale("bale_top", 0.0, 0.685, detail);
    });
    let joined = join(&top, "bale_top", Some(F.at(0, 0.685, 0)));
    joined.set_rotation_euler_axis(2, radians(3.0));
    settle(&[join_all(if detail { "hayStack" } else { "hayStack_far" })])
}

#[allow(non_snake_case)]
pub fn hayStack() -> Vec<Obj> {
    hayStack_detail(true)
}

/// The stack from afar: three plain bales and their twine.
#[allow(non_snake_case)]
pub fn hayStack_far() -> Vec<Obj> {
    hayStack_detail(false)
}
