//! What the big working things are built from: the colours every model may use (each model's own palette adds its
//! `body` and what is its alone: `own_palette!`), the frame, the wall's face, and the shapes several models make:
//! wheel arches, roller shutters and tyres.

use artkit::far::{is_low, low};
use artkit::materials::Mat;
use artkit::math::{PI, asin, cos, sin};
use artkit::place::{Coord, Facing, GameFrame};
use artkit::py;
use artkit::scene::Obj;
use artkit::shapes::sides;
use artkit::{fbox, fextrude, revolve};

/// A model's palette: the colours every model may use, then its own (the script's `_palette(**own)`), so a build
/// reads `M.rubber` and `M.body` alike. `own_palette! { S: }` is the shared colours alone.
macro_rules! own_palette {
    ($name:ident : $($own:tt)*) => {
        artkit::palette! { $name:
            steel = ("#A9AEB2", metal),       // brushed steel and aluminium: shutters, rails, ladders, beams
            chrome = ("#C8CCD0", chrome),     // handles, bezels, trims
            iron = ("#2B302E", metal),        // black steel: chassis, brackets, axles
            dark = ("#1E2120", matte),        // openings, gaps, grilles, the inside of things
            rubber = ("#26282B", matte),      // tyres, mats, feet, seals
            plastic = ("#3A3D41", satin),     // black plastic: housings, buttons, castors
            tinted = ("#2A3439", gloss),      // dark glass (gloss, not see-through: no draw call of its own)
            lamp = ("#F4F1E0", gloss),        // white lenses
            amber = ("#E8902E", gloss),       // indicators
            red_lens = ("#B8221E", gloss),    // tail lamps
            blue_lens = ("#2D62C8", gloss),   // the emergency lights
            white = ("#F1F0EC", satin),
            yellow = ("#F2C14E", gloss),      // stripes and chevrons
            brass = ("#C9A04A", metal, roughness(0.2)),   // polished brass: couplings, the fire pole
            paper = ("#F2F0EA", matte),
            ink = ("#2A2C2E", matte),
            $($own)*
        }
    };
}
pub(crate) use own_palette;

own_palette! { S: }

pub const F: GameFrame = GameFrame::new(0.0);
pub const WALL: f64 = -0.43; // the wall's face, game depth (one tile deep)

// --- helpers (game coordinates) ------------------------------------------------------------------------------
/// The points (depth, height) of a wheel arch: the circle of radius `r` round (zc, yc) above the line at height
/// `y_cut`, from its rear end over the top to its front end.
pub fn arc(zc: f64, yc: f64, r: f64, y_cut: f64, steps: usize) -> Vec<(f64, f64)> {
    let t0 = asin(((y_cut - yc) / r).clamp(-1.0, 1.0));
    (0..steps + 1)
        .map(|i| (PI - t0) - (PI - 2.0 * t0) * i as f64 / steps as f64)
        .map(|t| (zc + r * cos(t), yc + r * sin(t)))
        .collect()
}

/// A roller shutter on a side face at game x `x` (`s` +1 the right side, -1 the left), from depth z0 to z1 and
/// height y0 to y1: its slats rounded ridges running along the depth, `t` proud of the face at most. At low detail
/// (a far version) a plain panel.
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn slats(
    name: &str,
    x: f64,
    s: i32,
    z0: f64,
    z1: f64,
    y0: f64,
    y1: f64,
    material: Mat,
    pitch: f64,
    t: f64,
) -> Obj {
    let s = s as f64;
    if is_low() {
        return fbox!(
            F,
            name,
            t,
            y1 - y0,
            (z1 - z0).abs(),
            (x + s * t / 2.0, (y0 + y1) / 2.0, (z0 + z1) / 2.0),
            material
        );
    }
    let n = py::round((y1 - y0) / pitch).max(1);
    let p = (y1 - y0) / n as f64;
    let mut out = vec![(x, y0)];
    for i in 0..n {
        // each slat: a step out at its foot, then sloping back in to the next
        let ya = y0 + i as f64 * p;
        out.extend([(x + s * t * 0.4, ya), (x + s * t, ya + p * 0.12)]);
    }
    out.extend([(x + s * t * 0.4, y1), (x, y1)]);
    fextrude!(
        F,
        name,
        &out,
        (z1 - z0).abs(),
        (0, 0, (z0 + z1) / 2.0),
        material,
        plane = Facing::Front
    )
}

/// A roller shutter on a face looking to the back at game depth `z`, from x0 to x1 and height y0 to y1.
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn slats_back(
    name: &str,
    z: f64,
    x0: f64,
    x1: f64,
    y0: f64,
    y1: f64,
    material: Mat,
    pitch: f64,
    t: f64,
) -> Obj {
    if is_low() {
        return fbox!(
            F,
            name,
            (x1 - x0).abs(),
            y1 - y0,
            t,
            ((x0 + x1) / 2.0, (y0 + y1) / 2.0, z - t / 2.0),
            material
        );
    }
    let n = py::round((y1 - y0) / pitch).max(1);
    let p = (y1 - y0) / n as f64;
    let mut out = vec![(z, y0)];
    for i in 0..n {
        let ya = y0 + i as f64 * p;
        out.extend([(z - t * 0.4, ya), (z - t, ya + p * 0.12)]);
    }
    out.extend([(z - t * 0.4, y1), (z, y1)]);
    fextrude!(
        F,
        name,
        &out,
        (x1 - x0).abs(),
        ((x0 + x1) / 2.0, 0, 0),
        material,
        plane = Facing::Side
    )
}

/// The axle a tyre turns on: game x, z, or y (lying flat).
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Axle {
    X,
    Z,
    Y,
}

/// A tyre round an axle along game x (or `Axle::Z`; or `Axle::Y`, lying flat) through (x, y, z): rounded
/// shoulders, its tread's grooves round its crown, its bead's hole open (r * 0.64 round), where a rim goes.
/// `grooves` None: plain, one rounded shoulder a side.
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn tyre(
    name: &str,
    x: impl Coord,
    y: impl Coord,
    z: impl Coord,
    r: f64,
    w: f64,
    material: Mat,
    grooves: Option<usize>,
    segments: Option<usize>,
    axis: Axle,
) -> Obj {
    let hw = w / 2.0;
    let sh = f64::min(0.06, r * 0.12);
    let rot = match axis {
        Axle::X => (0, 90, 0),
        Axle::Z => (90, 0, 0),
        Axle::Y => (0, 0, 0),
    };
    let Some(grooves) = grooves else {
        // plain: one rounded shoulder a side
        let mut prof = vec![
            (r * 0.64, -hw + 0.01),
            (r - sh, -hw),
            (r, -hw + w * 0.16),
            (r, hw - w * 0.16),
            (r - sh, hw),
            (r * 0.64, hw - 0.01),
        ];
        prof.push(prof[0]);
        return revolve!(
            name,
            &prof,
            at = F.at(x, y, z),
            material = material,
            rot = rot,
            segments = segments.unwrap_or_else(|| low(sides(r, 24, 44), 16))
        );
    };
    let mut prof = vec![
        (r * 0.64, -hw + 0.01),
        (r - sh, -hw),
        (r - sh * 0.3, -hw + w * 0.07),
        (r, -hw + w * 0.18),
    ];
    if grooves != 0 && !is_low() {
        for k in 0..grooves {
            let g = -hw * 0.6 + (w * 0.6) * (k as f64 + 0.5) / grooves as f64;
            prof.extend([
                (r, g - 0.008),
                (r - 0.01, g - 0.005),
                (r - 0.01, g + 0.005),
                (r, g + 0.008),
            ]);
        }
    }
    prof.extend([
        (r, hw - w * 0.18),
        (r - sh * 0.3, hw - w * 0.07),
        (r - sh, hw),
        (r * 0.64, hw - 0.01),
    ]);
    prof.push(prof[0]);
    revolve!(
        name,
        &prof,
        at = F.at(x, y, z),
        material = material,
        rot = rot,
        segments = segments.unwrap_or_else(|| low(sides(r, 24, 44), 16))
    )
}
