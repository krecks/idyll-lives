//! What the home gym is built from: the helpers fitness.py shared between its models.

use super::M;
use artkit::forms::rounded_rect;
use artkit::materials::Mat;
use artkit::math::{Axis, M4, P3, cos, radians, sin};
use artkit::place::{Coord, GameFrame};
use artkit::{fcyl, revolve};

/// A game point (x, height, depth).
pub type P = (f64, f64, f64);

/// A number as the script held it, whole or not: a whole zero negates to a plain zero, a float zero to a negative
/// one, and a part's place keeps that sign (`Coord`).
#[derive(Clone, Copy, Debug)]
pub enum Num {
    Int(i32),
    Float(f64),
}

impl Coord for Num {
    fn f(self) -> f64 {
        match self {
            Num::Int(i) => i as f64,
            Num::Float(x) => x,
        }
    }
    fn negated(self) -> f64 {
        match self {
            Num::Int(i) => -i as f64,
            Num::Float(x) => -x,
        }
    }
}

impl std::ops::Add for Num {
    type Output = Num;
    fn add(self, o: Num) -> Num {
        match (self, o) {
            (Num::Int(a), Num::Int(b)) => Num::Int(a + b),
            _ => Num::Float(self.f() + o.f()),
        }
    }
}

impl From<i32> for Num {
    fn from(i: i32) -> Num {
        Num::Int(i)
    }
}

impl From<f64> for Num {
    fn from(x: f64) -> Num {
        Num::Float(x)
    }
}

/// A float as Python's `str` writes it in a part's name (a whole one with its `.0`).
pub fn py(x: f64) -> String {
    artkit::json::float_repr(x)
}

/// `_rr`: a rounded rectangle outline (`rounded_rect`), for profiles and slabs.
pub fn rr(w: f64, h: f64, r: f64, corner: usize) -> Vec<(f64, f64)> {
    rounded_rect(w, h, r, corner)
}

/// `_on`: a game point `d` (x, y, z) from `c` on a face tilted `tilt` degrees about the game's x axis.
pub fn on(c: (Num, f64, f64), tilt: i32, d: (Num, f64, f64)) -> (Num, f64, f64) {
    let a = radians(tilt as f64);
    (
        c.0 + d.0,
        c.1 + d.1 * cos(a) - d.2 * sin(a),
        c.2 + d.1 * sin(a) + d.2 * cos(a),
    )
}

/// `_plate`: a weight plate on a bar along game x at (x, y, z): a turned disc `r` round and `t` thick, a raised
/// rim, a dished face and a hub, both faces alike; `ring`: a coloured band inside the rim.
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn plate(
    f: &GameFrame,
    name: &str,
    x: f64,
    y: f64,
    z: f64,
    r: f64,
    t: f64,
    material: Mat,
    ring: Option<Mat>,
) {
    let prof = [
        (0.0, -t / 2.0 + 0.006),
        (0.03, -t / 2.0 + 0.006),
        (0.032, -t / 2.0),
        (r - 0.012, -t / 2.0),
        (r, -t / 2.0 + 0.006),
        (r, t / 2.0 - 0.006),
        (r - 0.012, t / 2.0),
        (0.032, t / 2.0),
        (0.03, t / 2.0 - 0.006),
        (0.0, t / 2.0 - 0.006),
    ];
    revolve!(
        name,
        &prof,
        at = f.at(x, y, z),
        material = material,
        segments = 24,
        rot = (0, 90, 0)
    );
    if let Some(ring) = ring {
        for s in [-1, 1] {
            revolve!(
                format!("{name}_ring{s}"),
                &[
                    (r * 0.62, 0.0),
                    (r * 0.78, 0.0),
                    (r * 0.78, 0.0015),
                    (r * 0.62, 0.0015)
                ],
                at = f.at(x + s as f64 * (t / 2.0 - 0.0005), y, z),
                material = ring,
                segments = 24,
                rot = (0, -90 * s, 0)
            );
        }
    }
}

/// Which way a dumbbell lies.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Along {
    X,
    #[allow(dead_code)] // (the script's other way: no model lays one along the depth)
    Z,
}

/// `_dumbbell`: a hex dumbbell: a knurled chrome handle and two hexagonal heads `w` across their flats, chamfered,
/// with chrome end caps, lying along game x (or z).
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn dumbbell(f: &GameFrame, name: &str, x: f64, y: f64, z: f64, w: f64, along: Along) {
    let rot: P3 = if along == Along::X {
        (0, 90, 0).into()
    } else {
        (90, 0, 0).into()
    };
    let off = |k: f64| -> P {
        if along == Along::X {
            (x + k, y, z)
        } else {
            (x, y, z + k)
        }
    };
    fcyl!(
        f,
        format!("{name}_grip"),
        0.014,
        0.1,
        (x, y, z),
        M.chrome,
        rot = rot,
        verts = 8
    );
    fcyl!(
        f,
        format!("{name}_knurl"),
        0.0145,
        0.07,
        (x, y, z),
        M.frame,
        rot = rot,
        verts = 8
    );
    // the hexagon's corner radius
    let r = w / 2.0 / cos(std::f64::consts::PI / 6.0);
    for s in [-1, 1] {
        let sf = s as f64;
        let head = [
            (0.0, -0.0225),
            (r * 0.85, -0.0225),
            (r, -0.016),
            (r, 0.016),
            (r * 0.85, 0.0225),
            (0.0, 0.0225),
        ];
        let (ox, oy, oz) = off(sf * 0.068);
        let obj = revolve!(
            format!("{name}_head{s}"),
            &head,
            at = f.at(ox, oy, oz),
            material = M.plate,
            segments = 6,
            rot = rot,
            smooth = false
        );
        // a flat underneath, not a corner
        if along == Along::X {
            let turn = M4::rotation(radians(30.0), Axis::Z);
            obj.edit(|o| o.mesh.transform(&turn));
        }
        fcyl!(
            f,
            format!("{name}_cap{s}"),
            0.016,
            0.004,
            off(sf * 0.0915),
            M.chrome,
            rot = rot,
            verts = 8
        );
    }
}
