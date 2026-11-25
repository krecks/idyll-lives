//! What the bathroom's things are built from: tubes, lathed shells, taps, pulls, panels, bottles, the water.

use super::M;
use artkit::forms::rounded_rect;
use artkit::kit;
use artkit::materials::Mat;
use artkit::math::{P3, PI, TAU, V3, cos, pow, sin};
use artkit::place::{Coord, GameFrame};
use artkit::py;
use artkit::scene::Obj;
use artkit::{fbox, fcyl, fextrude, loft};

/// The wall's face, game depth: the toilets' cisterns and the storage stand on it.
pub use artkit::kit::WALL;

/// A closed loop of Blender points round game (cx, cz) at height `y`: `rx` across, `rz` to the front (`back`
/// behind, if given), squarer towards a rounded rectangle as `square` grows, narrower to the front by `egg`.
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn loop_(
    f: &GameFrame,
    cx: f64,
    cz: f64,
    y: f64,
    rx: f64,
    rz: f64,
    verts: usize,
    square: f64,
    egg: f64,
    back: Option<f64>,
) -> Vec<P3> {
    let mut out = Vec::new();
    let e = 2.0 / square;
    for i in 0..verts {
        let a = TAU * i as f64 / verts as f64;
        let (c, s) = (cos(a), sin(a));
        let (u, v) = (pow(c.abs(), e).copysign(c), pow(s.abs(), e).copysign(s));
        let dz = v * match back {
            Some(back) if v < 0.0 => back,
            _ => rz,
        };
        out.push(f.at(cx + u * rx * (1.0 - egg * v), y, cz + dz));
    }
    out
}

/// A ring of a `lathe`: (y, rx, rz[, square[, back[, dz]]]).
#[derive(Clone, Copy, Debug)]
pub struct Ring {
    pub y: f64,
    pub rx: f64,
    pub rz: f64,
    pub sq: Option<f64>,
    pub back: Option<f64>,
    pub dz: Option<f64>,
}

impl From<(f64, f64, f64)> for Ring {
    fn from((y, rx, rz): (f64, f64, f64)) -> Ring {
        Ring {
            y,
            rx,
            rz,
            sq: None,
            back: None,
            dz: None,
        }
    }
}
impl From<(f64, f64, f64, f64)> for Ring {
    fn from((y, rx, rz, sq): (f64, f64, f64, f64)) -> Ring {
        Ring {
            sq: Some(sq),
            ..Ring::from((y, rx, rz))
        }
    }
}
impl From<(f64, f64, f64, f64, f64)> for Ring {
    fn from((y, rx, rz, sq, back): (f64, f64, f64, f64, f64)) -> Ring {
        Ring {
            back: Some(back),
            ..Ring::from((y, rx, rz, sq))
        }
    }
}
impl From<(f64, f64, f64, f64, f64, f64)> for Ring {
    fn from((y, rx, rz, sq, back, dz): (f64, f64, f64, f64, f64, f64)) -> Ring {
        Ring {
            dz: Some(dz),
            ..Ring::from((y, rx, rz, sq, back))
        }
    }
}

/// `lathe`'s options.
#[derive(Clone, Copy, Debug)]
pub struct Lathe {
    pub verts: usize,
    pub square: f64,
    pub egg: f64,
    pub caps: bool,
    pub angle: f64,
}

/// `lathe`'s options as the script's defaults.
pub const LATHE: Lathe = Lathe {
    verts: 24,
    square: 2.0,
    egg: 0.0,
    caps: true,
    angle: 40.0,
};

/// A body lofted through game rings [(y, rx, rz[, square[, back[, dz]]]), ...] round (cx, cz) (each moved `dz` to
/// the front), in profile order: a basin's shell (up the outside, over the rim, down the inside to its floor), a
/// cistern, a bowl, a bottle.
pub fn lathe<R: Into<Ring> + Copy>(
    f: &GameFrame,
    name: impl AsRef<str>,
    cx: f64,
    cz: f64,
    rings: &[R],
    material: Mat,
    o: Lathe,
) -> Obj {
    let mut sections = Vec::new();
    for &ring in rings {
        let ring: Ring = ring.into();
        let (y, rx, rz) = (ring.y, ring.rx, ring.rz);
        let sq = ring.sq.unwrap_or(o.square);
        let back = ring.back;
        let dz = ring.dz.unwrap_or(0.0);
        sections.push(loop_(f, cx, cz + dz, y, rx, rz, o.verts, sq, o.egg, back));
    }
    loft!(
        name,
        &sections,
        material = material,
        caps = o.caps,
        angle = o.angle
    )
}

/// A tap's or a fitting's round base plate, domed: from `r` on the deck at height `y` up `h`.
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn rose(
    f: &GameFrame,
    name: impl AsRef<str>,
    x: impl Coord,
    y: f64,
    z: impl Coord,
    r: f64,
    material: Mat,
    h: f64,
    verts: usize,
) -> Obj {
    lathe(
        f,
        name,
        x.f(),
        z.f(),
        &[
            (y, r, r),
            (y + h * 0.45, r * 0.97, r * 0.97),
            (y + h, r * 0.62, r * 0.62),
        ],
        material,
        Lathe {
            verts,
            angle: 60.0,
            ..LATHE
        },
    )
}

/// A deck tap in one piece: a riser from (x, base_y, z) up and over in a half circle whose top is `top`, down to
/// a spout at depth `spout_z` with an aerator; a domed rose under it and a side lever. Returns the spout tip's
/// game point (where the water starts).
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn gooseneck<X: Coord>(
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
    lever: bool,
) -> (X, f64, f64) {
    let tip_y = tip_y.unwrap_or(base_y + 0.16);
    let big_r = (spout_z - z) / 2.0;
    let (cz, cy) = (z + big_r, top - big_r);
    let end = py::min(tip_y, cy);
    let arc = (0..9).map(|k| {
        (
            x,
            cy + big_r * sin(PI * k as f64 / 8.0),
            cz - big_r * cos(PI * k as f64 / 8.0),
        )
    });
    let mut pts = vec![(x, base_y, z), (x, cy - 0.04, z)];
    pts.extend(arc);
    if end < cy - 0.01 {
        pts.push((x, end, spout_z));
    }
    kit::tube!(f, name, &pts, r, material, verts = 10, round = 2);
    fcyl!(
        f,
        format!("{name}_aerator"),
        r * 1.12,
        0.02,
        (x, end + 0.006, spout_z),
        material,
        verts = 10
    );
    rose(
        f,
        format!("{name}_rose"),
        x,
        base_y,
        z,
        r * 2.2,
        material,
        0.012,
        16,
    );
    if lever {
        let ly = base_y + 0.07;
        kit::tube!(
            f,
            format!("{name}_lever"),
            &[
                (x.f() - r * 0.5, ly, z),
                (x.f() - r - 0.022, ly + 0.004, z),
                (x.f() - r - 0.028, ly + 0.05, z - 0.012),
            ],
            0.0065,
            material,
            verts = 8,
            round = 2,
            scale = [1.3, 1.0, 0.75]
        );
    }
    (x, end - 0.014, spout_z)
}

/// A bar pull in one piece, its middle at game (x, y) on a face at depth `z`: out on rounded legs, along
/// `length` (up, or across), and back in.
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn pull(
    f: &GameFrame,
    name: impl AsRef<str>,
    x: f64,
    y: f64,
    z: f64,
    length: f64,
    material: Mat,
    vertical: bool,
    r: f64,
    stand: f64,
) -> Obj {
    let (a, b) = (-length / 2.0, length / 2.0);
    let p = |t: f64, out: f64| {
        if vertical {
            (x, y + t, z + out)
        } else {
            (x + t, y, z + out)
        }
    };
    let pts = [
        p(a, -0.002),
        p(a, stand - 0.009),
        p(a + 0.009, stand),
        p(b - 0.009, stand),
        p(b, stand - 0.009),
        p(b, -0.002),
    ];
    kit::tube!(f, name, &pts, r, material, verts = 8, round = 2)
}

/// A shaker door or drawer front, its back on game depth `z`: a frame (an outline with a hole, its edges eased)
/// round a panel set 6 mm back inside it.
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn shaker(
    f: &GameFrame,
    name: &str,
    w: f64,
    h: f64,
    x: impl Coord,
    y: f64,
    z: f64,
    material: Mat,
    t: f64,
    rail: f64,
) {
    fextrude!(
        f,
        name,
        &rounded_rect(w, h, 0.004, 1),
        t,
        (x, y, z + t / 2.0),
        material,
        holes = vec![rounded_rect(w - 2.0 * rail, h - 2.0 * rail, 0.003, 1)],
        bevel = 0.003,
        bevel_segments = 1
    );
    let (pw, ph) = (w - 2.0 * rail + 0.004, h - 2.0 * rail + 0.004);
    panel(
        f,
        &format!("{name}_panel"),
        pw,
        ph,
        x,
        y,
        z,
        t - 0.006,
        material,
        py::min(py::min(0.025, pw / 5.0), ph / 5.0),
    );
}

/// A flat panel set in a frame, its back on game depth `z`: a band round its edge and its middle, so the shading
/// baked into its vertices darkens only the band in the frame's crease, not the whole face.
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn panel(
    f: &GameFrame,
    name: &str,
    w: f64,
    h: f64,
    x: impl Coord,
    y: f64,
    z: f64,
    t: f64,
    material: Mat,
    band: f64,
) {
    let inner = rounded_rect(w - 2.0 * band, h - 2.0 * band, 0.002, 1);
    fextrude!(
        f,
        format!("{name}_band"),
        &rounded_rect(w, h, 0.002, 1),
        t,
        (x, y, z + t / 2.0),
        material,
        holes = vec![inner.clone()]
    );
    fextrude!(f, name, &inner, t, (x, y, z + t / 2.0), material);
}

/// A bottle standing at game point `foot`: a round body, a shoulder, a neck and its cap (or a pump).
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn bottle(
    f: &GameFrame,
    name: &str,
    foot: (f64, f64, f64),
    r: f64,
    h: f64,
    material: Mat,
    cap: Mat,
    verts: usize,
    pump: bool,
) {
    let (x, y, z) = foot;
    lathe(
        f,
        name,
        x,
        z,
        &[
            (y, r * 0.9, r * 0.9),
            (y + h * 0.03, r, r),
            (y + h * 0.7, r, r),
            (y + h * 0.8, r * 0.6, r * 0.6),
            (y + h * 0.86, r * 0.4, r * 0.4),
            (y + h * 0.9, r * 0.4, r * 0.4),
        ],
        material,
        Lathe { verts, ..LATHE },
    );
    if pump {
        fcyl!(
            f,
            format!("{name}_pump"),
            r * 0.45,
            h * 0.1,
            (x, y + h * 0.95, z),
            cap,
            verts = 8
        );
        fbox!(
            f,
            format!("{name}_nozzle"),
            0.008,
            0.01,
            r * 1.3,
            (x, y + h, z + r * 0.5),
            cap,
            bevel = 0.003,
            bevel_segments = 1
        );
    } else {
        fcyl!(
            f,
            format!("{name}_cap"),
            r * 0.48,
            h * 0.1,
            (x, y + h * 0.95, z),
            cap,
            verts = verts
        );
    }
}

/// A plug's flange (chrome, or the tap's metal) and its dark hole, on a basin's floor at height `y`.
pub fn drain(f: &GameFrame, x: impl Coord, y: f64, z: f64, r: f64, material: Option<Mat>) {
    rose(
        f,
        "drain_flange",
        x,
        y - 0.002,
        z,
        r,
        material.unwrap_or(M.chrome),
        0.006,
        16,
    );
    fcyl!(
        f,
        "drain",
        r * 0.55,
        0.004,
        (x, y + 0.003, z),
        M.dark,
        verts = 12
    );
}

/// A vessel bowl in one shell (up the outside from its foot, over a rounded rim, down the inside to a floor),
/// `w` x `d` across the rim, standing on `floor_y`. `foot` is the foot's share of the width, `belly` how full
/// its sides are.
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn bowl_shell(
    f: &GameFrame,
    name: &str,
    x: impl Coord,
    z: f64,
    floor_y: f64,
    rim_y: f64,
    w: f64,
    d: f64,
    material: Mat,
    wall: f64,
    foot: f64,
    belly: f64,
    verts: usize,
    square: f64,
) -> Obj {
    let (rx, rz, h) = (w / 2.0, d / 2.0, rim_y - floor_y);
    let k = rz / rx;
    let ring = |y: f64, r: f64| (y, r, r * k, square);
    lathe(
        f,
        name,
        x.f(),
        z,
        &[
            ring(floor_y, rx * foot),
            ring(floor_y + h * 0.12, rx * (foot + 0.18 * belly)),
            ring(floor_y + h * 0.45, rx * (0.86 + 0.06 * belly)),
            ring(floor_y + h * 0.8, rx * 0.985),
            ring(rim_y - 0.004, rx),
            ring(rim_y, rx - wall * 0.35),
            ring(rim_y - 0.002, rx - wall),
            ring(floor_y + h * 0.6, rx * (0.92 + 0.03 * belly) - wall),
            ring(floor_y + h * 0.25, rx * 0.72 - wall),
            ring(floor_y + wall + 0.006, rx * 0.4),
            ring(floor_y + wall, rx * 0.25),
        ],
        material,
        Lathe {
            verts,
            angle: 55.0,
            ..LATHE
        },
    )
}

/// A round plate on a wall (or a cistern's front) at game depth `z`, its edge eased.
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn wall_rose(
    f: &GameFrame,
    name: impl AsRef<str>,
    x: impl Coord,
    y: f64,
    z: f64,
    r: f64,
    material: Mat,
    h: f64,
    verts: usize,
) -> Obj {
    fcyl!(
        f,
        name,
        r,
        h,
        (x, y, z + h / 2.0),
        material,
        rot = (90, 0, 0),
        verts = verts,
        bevel = h * 0.4,
        bevel_segments = 1
    )
}

/// The points of a path with each inner corner turned in a round bend of about `r` (`steps` points on it), as
/// bent tube or a welded frame turns: sweep them as they are (resolution 1), so long runs don't overshoot.
pub fn bends(pts: &[(f64, f64, f64)], r: f64, steps: usize) -> Vec<(f64, f64, f64)> {
    let mut out = vec![pts[0]];
    for w in pts.windows(3) {
        let (a_, b_, c_) = (V3::from(w[0]), V3::from(w[1]), V3::from(w[2]));
        let p0 = b_ + (a_ - b_).normalized() * py::min(r, (a_ - b_).length() / 2.0);
        let p2 = b_ + (c_ - b_).normalized() * py::min(r, (c_ - b_).length() / 2.0);
        for k in 0..=steps {
            let t = k as f64 / steps as f64;
            let p = p0 * pow(1.0 - t, 2.0) + b_ * 2 * (1.0 - t) * t + p2 * t * t;
            out.push((p.x as f64, p.y as f64, p.z as f64));
        }
    }
    out.push(pts[pts.len() - 1]);
    out
}
