//! The taxi's livery on the taxi rank's signs: its word and its chequers.

use crate::materials::Mat;
use crate::math::{Num, atan2, degrees};
use crate::place::GameFrame;
use crate::py;
use crate::scene::Obj;
use crate::shapes::{Cube, options};

options! {
    /// `taxi_word`'s options.
    TaxiWord { depth: f64 = 0.004 }
}

/// TAXI in block letters `h` high, its middle at game (x, height) `centre`, standing `depth` proud of a face at
/// depth `z` that looks `facing` (+1: to the front, +z; -1: to the back), so it reads from that side: seen from
/// the front, the game's +x is on the left (README, "Coordinates"), so the word runs from +x there, from -x
/// behind. The taxi rank's signs and fares board, as the taxi's own roof sign (render/shared/cars.ts `paintTaxi`).
#[allow(clippy::too_many_arguments)] // (as the scripts called it)
pub fn taxi_word(
    frame: &GameFrame,
    name: impl AsRef<str>,
    centre: (impl Num, impl Num),
    h: f64,
    z: f64,
    facing: i32,
    material: Mat,
    o: TaxiWord,
) -> Vec<Obj> {
    let name = name.as_ref();
    let (depth, facing) = (o.depth, facing as f64);
    let (cx, cy) = (centre.0.f(), centre.1.f());
    let (s, w, gap) = (0.2 * h, 0.66 * h, 0.22 * h);
    let total = 4.0 * w + 3.0 * gap;
    let zz = z + facing * depth / 2.0;
    let lean = degrees(atan2(w / 2.0 - s / 2.0, h)); // an A's legs from upright
    let cross = degrees(atan2(w - s, h)); // an X's strokes from upright
    let mut out = Vec::new();
    let turned = |about_y: f64| Cube {
        rot: (0.0, about_y, 0.0).into(),
        ..Cube::default()
    };
    for (i, ch) in "TAXI".chars().enumerate() {
        // (Seen from the side it faces, letter by letter left to right.)
        let x = cx + facing * (total / 2.0 - i as f64 * (w + gap) - w / 2.0);
        let n = format!("{name}_{ch}{i}");
        match ch {
            'T' => {
                out.push(frame.box_(
                    format!("{n}_bar"),
                    w,
                    s,
                    depth,
                    (x, cy + h / 2.0 - s / 2.0, zz),
                    material,
                    Cube::default(),
                ));
                out.push(frame.box_(
                    format!("{n}_stem"),
                    s,
                    h - s,
                    depth,
                    (x, cy - s / 2.0, zz),
                    material,
                    Cube::default(),
                ));
            }
            'A' => {
                for k in [-1, 1] {
                    let leg = py::hypot(w / 2.0 - s / 2.0, h);
                    out.push(frame.box_(
                        format!("{n}_leg{k}"),
                        s,
                        leg,
                        depth,
                        (x + k as f64 * (w / 4.0 - s / 4.0), cy, zz),
                        material,
                        turned(k as f64 * lean),
                    ));
                }
                out.push(frame.box_(
                    format!("{n}_bar"),
                    w * 0.42,
                    s * 0.8,
                    depth,
                    (x, cy - h * 0.12, zz),
                    material,
                    Cube::default(),
                ));
            }
            'X' => {
                for k in [-1, 1] {
                    out.push(frame.box_(
                        format!("{n}_stroke{k}"),
                        s,
                        py::hypot(w - s, h) * 0.97,
                        depth,
                        (x, cy, zz),
                        material,
                        turned(k as f64 * cross),
                    ));
                }
            }
            _ => out.push(frame.box_(
                format!("{n}_stem"),
                s,
                h,
                depth,
                (x, cy, zz),
                material,
                Cube::default(),
            )),
        }
    }
    out
}

options! {
    /// `chequer`'s options.
    Chequer { rows: usize = 2, depth: f64 = 0.003 }
}

/// A chequered band's dark squares (`size` a side, `rows` of them, the band's top at height `y`) from x0 to x1
/// along a face at depth `z` looking `facing` (+1 front, -1 back), `depth` proud of it: the light squares are the
/// face behind. The taxi's livery, on the taxi rank's signs and shelter.
#[allow(clippy::too_many_arguments)] // (as the scripts called it)
pub fn chequer(
    frame: &GameFrame,
    name: impl AsRef<str>,
    x0: f64,
    x1: f64,
    y: f64,
    z: f64,
    size: f64,
    facing: i32,
    material: Mat,
    o: Chequer,
) -> Vec<Obj> {
    let name = name.as_ref();
    let zz = z + facing as f64 * o.depth / 2.0;
    let mut out = Vec::new();
    for r in 0..o.rows {
        let mut k = 0;
        let mut x = x0;
        while x + size <= x1 + 1e-6 {
            if (k + r) % 2 == 0 {
                out.push(frame.box_(
                    format!("{name}{r}_{k}"),
                    size,
                    size,
                    o.depth,
                    (x + size / 2.0, y - size / 2.0 - r as f64 * size, zz),
                    material,
                    Cube::default(),
                ));
            }
            x += size;
            k += 1;
        }
    }
    out
}
