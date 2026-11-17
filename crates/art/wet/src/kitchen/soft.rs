//! A stand-in for gamekit's `forms.soft_box` until the kernel has it: the plain soft box the fridge's doors and
//! badge are (edges rounded by `round`, no puff, pinch, sag, tufts, rumple or piping), as forms.py builds it.
//! To go when `artkit` has `soft_box` and `GameFrame::soft`.

use artkit::far::is_low;
use artkit::materials::Mat;
use artkit::math::{P3, PI, V3, cos, tan, v3};
use artkit::mesh::Mesh;
use artkit::place::GameFrame;
use artkit::scene::{self, Obj};
use artkit::shapes::shade;
use std::collections::BTreeMap;

const SOFT_FACET: f64 = 0.0015; // metres: how far a soft thing's rounded edges may stand in from their curve

/// Positions from -h to h: `round_segments` closing in on each end over the rounding `r`, placed so the rounding
/// turns by the same angle from each to the next, `flat` steps across the middle.
fn steps(h: f64, r: f64, round_segments: usize, flat: usize) -> Vec<f64> {
    let edge: Vec<f64> = (0..round_segments)
        .map(|i| -(h - r) - r * tan(PI / 4.0 * (1.0 - i as f64 / round_segments as f64)))
        .collect();
    let inner: Vec<f64> = if h > r + 1e-6 {
        (0..=flat)
            .map(|i| -(h - r) + 2.0 * (h - r) * i as f64 / flat as f64)
            .collect()
    } else {
        vec![0.0]
    };
    let mut out = edge.clone();
    out.extend(inner);
    out.extend(edge.iter().rev().map(|x| -x));
    out
}

fn round_(c: f64) -> i64 {
    (c * 1e6 + 0.5).floor() as i64
}

/// A soft box `size` (x, y, z) across, its edges rounded by `round`.
fn soft_box(
    name: &str,
    size: (f64, f64, f64),
    at: P3,
    material: Mat,
    round: f64,
    round_segments: Option<usize>,
) -> Obj {
    let h = [size.0 / 2.0, size.1 / 2.0, size.2 / 2.0];
    let r = round.min(h[0]).min(h[1]).min(h[2]);
    let low = is_low();
    // the rounding's quarter turn in 2 x round_segments equal parts: each stands r (1 - cos(pi / 8n)) in
    let round_segments = round_segments.unwrap_or(if r < 0.008 || low {
        1
    } else if r * (1.0 - cos(PI / 16.0)) <= SOFT_FACET {
        2
    } else {
        3
    });
    let inner = [h[0] - r, h[1] - r, h[2] - r];
    let steps: Vec<Vec<f64>> = (0..3).map(|i| steps(h[i], r, round_segments, 1)).collect();
    // a point of the box's surface moved onto the rounded one
    let shape = |p: [f64; 3]| -> V3 {
        let centre = v3(
            p[0].clamp(-inner[0], inner[0]),
            p[1].clamp(-inner[1], inner[1]),
            p[2].clamp(-inner[2], inner[2]),
        );
        let d = v3(p[0], p[1], p[2]) - centre;
        if d.length() > 1e-9 {
            centre + d.normalized() * r
        } else {
            v3(p[0], p[1], p[2])
        }
    };
    let mut index: BTreeMap<[i64; 3], u32> = BTreeMap::new();
    let mut vs: Vec<V3> = Vec::new();
    let mut faces: Vec<Vec<u32>> = Vec::new();
    for a in 0..3 {
        // the face pairs across axis a
        let (b, c) = ((a + 1) % 3, (a + 2) % 3);
        for side in [-1.0, 1.0] {
            for i in 0..steps[b].len() - 1 {
                for j in 0..steps[c].len() - 1 {
                    let mut quad = Vec::with_capacity(4);
                    for (di, dj) in [(0, 0), (1, 0), (1, 1), (0, 1)] {
                        let mut p = [0.0; 3];
                        (p[a], p[b], p[c]) = (side * h[a], steps[b][i + di], steps[c][j + dj]);
                        let key = [round_(p[0]), round_(p[1]), round_(p[2])];
                        let at = *index.entry(key).or_insert_with(|| {
                            vs.push(shape(p));
                            vs.len() as u32 - 1
                        });
                        quad.push(at);
                    }
                    if side < 0.0 {
                        quad.reverse();
                    }
                    faces.push(quad);
                }
            }
        }
    }
    let mut mesh = Mesh::from_pydata(vs, &faces);
    mesh.set_material(Some(material));
    mesh.recalc_normals();
    let obj = scene::add(name, mesh);
    shade(obj, true, 75.0);
    obj.set_location(at)
}

/// A soft box of game width, height and depth at a game position.
#[allow(clippy::too_many_arguments)] // (as the scripts called it)
pub fn soft(
    frame: GameFrame,
    name: &str,
    w: f64,
    h: f64,
    d: f64,
    at: (f64, f64, f64),
    material: Mat,
    round: f64,
    round_segments: Option<usize>,
) -> Obj {
    soft_box(
        name,
        (w, d, h),
        frame.at(at.0, at.1, at.2),
        material,
        round,
        round_segments,
    )
}
