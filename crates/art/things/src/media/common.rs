//! What the media are built from: the helpers media.py shared between its models.

use super::{F, M};
pub use crate::common::P;
use artkit::forms::rounded_rect;
use artkit::materials::Mat;
use artkit::math::{P3, atan2, cosd, degrees, sind};
use artkit::place::Coord;
use artkit::scene::Obj;
use artkit::shapes::Cube;
use artkit::{fbox, fsweep, revolve, soft};

/// The wall's face (game depth).
pub const WALL: f64 = -0.43;

/// `_axle`: the rot for a cylinder whose axis runs along the game direction (gx, 0, gz).
pub fn axle(gx: f64, gz: f64) -> P3 {
    (0, 90, degrees(atan2(-gz, -gx))).into()
}

/// `_rbox`'s options: its edges chamfered (`segs` 1) or rounded (2 or more) by `r`, and the box's turn.
pub struct Rbox {
    pub r: f64,
    pub segs: usize,
    pub rot: P3,
}

impl Default for Rbox {
    fn default() -> Rbox {
        Rbox {
            r: 0.004,
            segs: 1,
            rot: P3::ZERO,
        }
    }
}

/// `_rbox`: a box with its edges chamfered (`segs` 1) or rounded (2 or more).
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn rbox(
    name: impl AsRef<str>,
    w: f64,
    h: f64,
    d: f64,
    at: (impl Coord, impl Coord, impl Coord),
    mat: Mat,
    o: Rbox,
) -> Obj {
    let c = Cube {
        bevel: o.r,
        bevel_segments: o.segs,
        rot: o.rot,
        ..Cube::default()
    };
    F.box_(name, w, h, d, at, mat, c)
}

/// `_capsule`: a bar along x from x0 to x1, its section an h x d rounded rectangle (a soundbar, a rail, a case).
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn capsule(
    name: impl AsRef<str>,
    x0: f64,
    x1: f64,
    y: f64,
    z: f64,
    h: f64,
    d: f64,
    mat: Mat,
    r: Option<f64>,
) -> Obj {
    let r = match r {
        Some(r) if r != 0.0 => r,
        _ => (if d < h { d } else { h }) * 0.45,
    };
    fsweep!(
        F,
        name,
        &[(x0, y, z), (x1, y, z)],
        0,
        mat,
        profile = rounded_rect(h, d, r, 3),
        up = (0, 0, 1),
        angle = 50.0
    )
}

/// `_grille`: a speaker's cloth face: a thin panel on the front at depth `z`, with fine horizontal ribs.
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn grille(
    name: &str,
    x: impl Coord,
    y: f64,
    z: f64,
    w: f64,
    h: f64,
    mat: Mat,
    rows: Option<usize>,
    face: i32,
) {
    rbox!(name, w, h, 0.003, (x, y, z), mat, r = 0.001);
    let n = rows.unwrap_or(0);
    for k in 0..n {
        fbox!(
            F,
            format!("{name}_rib{k}"),
            w - 0.006,
            0.0015,
            0.001,
            (
                x,
                y - h / 2.0 + h * (k as f64 + 0.5) / n as f64,
                z + face as f64 * 0.002
            ),
            M.grille_dark
        );
    }
}

/// `_driver`: a loudspeaker driver facing the front from depth `z`: a rim, a cone sunk into it and a dust cap.
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn driver(
    name: &str,
    x: f64,
    y: f64,
    z: f64,
    r: f64,
    cone: Option<Mat>,
    ring: Option<Mat>,
    verts: usize,
) {
    revolve!(
        name,
        &[
            (0.0, -0.002),
            (r, -0.002),
            (r, 0.003),
            (r * 0.9, 0.004),
            (r * 0.82, 0.0),
            (r * 0.3, -r * 0.35),
            (r * 0.28, -r * 0.3),
            (r * 0.22, -r * 0.18),
            (0.0, -r * 0.14)
        ],
        at = F.at(x, y, z),
        material = cone.unwrap_or(M.cone),
        segments = verts,
        rot = (90, 0, 0)
    );
    if let Some(ring) = ring {
        let pts: Vec<P> = (0..360)
            .step_by(360 / verts)
            .map(|a| {
                let a = a as f64;
                (
                    x + (r + 0.003) * cosd(a),
                    y + (r + 0.003) * sind(a),
                    z + 0.003,
                )
            })
            .collect();
        fsweep!(
            F,
            format!("{name}_ring"),
            &pts,
            0.0035,
            ring,
            verts = 5,
            closed = true
        );
    }
}

/// `_flat_screen`: a flat panel facing the front at depth `z` (its middle): a slim rounded bezel, the screen set a
/// hair in, a deeper rounded hump behind (`back`: its w, h), a logo under the screen.
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn flat_screen(
    prefix: &str,
    x: impl Coord,
    y: f64,
    z: f64,
    w: f64,
    h: f64,
    frame: Mat,
    depth: f64,
    back: Option<(f64, f64)>,
    chin: f64,
) {
    soft!(
        F,
        format!("{prefix}panel"),
        w,
        h,
        depth,
        (x, y, z),
        frame,
        round = 0.006,
        round_segments = 2
    );
    fbox!(
        F,
        format!("{prefix}screen"),
        w - 0.022,
        h - 0.022 - chin,
        0.002,
        (x, y + chin / 2.0, z + depth / 2.0 + 0.0005),
        M.screen
    );
    fbox!(
        F,
        format!("{prefix}logo"),
        0.04,
        0.005,
        0.001,
        (
            x,
            y - h / 2.0 + 0.006 + chin / 3.0,
            z + depth / 2.0 + 0.0004
        ),
        M.steel
    );
    if let Some(back) = back {
        rbox!(
            format!("{prefix}back"),
            back.0,
            back.1,
            0.03,
            (x, y - 0.02, z - depth / 2.0 - 0.015 + 0.004),
            frame,
            r = 0.012,
            segs = 2
        );
        // vents across the hump's back
        for k in 0..5 {
            fbox!(
                F,
                format!("{prefix}vent{k}"),
                back.0 * 0.7,
                0.003,
                0.002,
                (
                    x,
                    y - 0.02 + back.1 / 2.0 - 0.03 - 0.012 * k as f64,
                    z - depth / 2.0 - 0.0305 + 0.004
                ),
                M.grille
            );
        }
    }
}

/// `_records`: the records' sleeves, in the order they stand: rust, black, mustard, teal, cream, black, navy, clay,
/// teal, rust.
pub fn records() -> [Mat; 10] {
    [
        M.rec_rust,
        M.rec_black,
        M.rec_mustard,
        M.rec_teal,
        M.rec_cream,
        M.rec_black,
        M.rec_navy,
        M.rec_clay,
        M.rec_teal,
        M.rec_rust,
    ]
}

/// The helpers called as the script called them, their options by name.
pub mod call {
    /// `rbox` with its options by name: `rbox!(name, w, h, d, at, material, r = 0.006, segs = 2, rot = (0, 0, 14))`.
    macro_rules! rbox {
        ($name:expr, $w:expr, $h:expr, $d:expr, $at:expr, $mat:expr $(, $k:ident = $v:expr)* $(,)?) => {{
            let args = ($name, $w, $h, $d, $at, $mat);
            #[allow(unused_mut)]
            let mut o = $crate::media::common::Rbox::default();
            $(o.$k = artkit::Arg::arg($v);)*
            $crate::media::common::rbox(
                args.0,
                artkit::forms::num(args.1),
                artkit::forms::num(args.2),
                artkit::forms::num(args.3),
                args.4,
                args.5,
                o,
            )
        }};
    }
    pub(crate) use rbox;
}
use call::rbox;
