//! What the baths are built from: the tubs' superellipse shells, the stream and the bath water and their clips, a
//! duck, towels and the waste.

use super::M;
use super::common::rose;
use artkit::forms::rounded_rect;
use artkit::materials::Mat;
use artkit::math::{Axis, PI, cos, pow, sin, v3};
use artkit::mesh::Mesh;
use artkit::motion::{Ease, Grow, Path, grow, keys};
use artkit::parts::{join, new_parts, set_origin};
use artkit::place::{Coord, GameFrame};
use artkit::py;
use artkit::scene::{self, Obj};
use artkit::shapes::shade;
use artkit::{fcone, fcyl, fsphere, fsweep, soft, sphere};

pub const BATH_TAP: f64 = 0.4;
pub const BATH_FILL: f64 = 2.0;

/// A tub's ring: (a, b, height, n).
pub type Section = (f64, f64, f64, f64);

/// A superellipse |x/a|^n + |y/b|^n = 1 (n = 2: an ellipse; larger: squarer), as `segments` points.
pub fn ring(a: f64, b: f64, n: f64, segments: usize) -> Vec<(f64, f64)> {
    let mut pts = Vec::new();
    for k in 0..segments {
        let t = 2.0 * PI * k as f64 / segments as f64;
        let (c, s) = (cos(t), sin(t));
        pts.push((
            a * pow(c.abs(), 2.0 / n).copysign(c),
            b * pow(s.abs(), 2.0 / n).copysign(s),
        ));
    }
    pts
}

/// A closed body through superellipse rings `sections` [(a, b, height, n)]: half its length (game x) and half its
/// depth at each height, in profile order; both end rings are capped. Edges sharper than `sharp` degrees are
/// shaded flat.
pub fn loft(name: &str, sections: &[Section], material: Mat, segments: usize, sharp: f64) -> Obj {
    let mut verts = Vec::new();
    let mut faces: Vec<Vec<u32>> = Vec::new();
    for &(a, b, y, n) in sections {
        verts.extend(
            ring(a, b, n, segments)
                .into_iter()
                .map(|(x, z)| v3(x, z, y)),
        );
    }
    let m = sections.len();
    for i in 0..m - 1 {
        for k in 0..segments {
            let k2 = (k + 1) % segments;
            faces.push(vec![
                (i * segments + k) as u32,
                (i * segments + k2) as u32,
                ((i + 1) * segments + k2) as u32,
                ((i + 1) * segments + k) as u32,
            ]);
        }
    }
    for i in [0, m - 1] {
        verts.push(v3(0, 0, sections[i].2));
        let centre = (verts.len() - 1) as u32;
        faces.extend((0..segments).map(|k| {
            vec![
                (i * segments + k) as u32,
                (i * segments + (k + 1) % segments) as u32,
                centre,
            ]
        }));
    }
    let mut mesh = Mesh::from_pydata(verts, &faces);
    mesh.set_material(Some(material));
    mesh.recalc_normals();
    let obj = scene::add(name, mesh);
    shade(obj, true, sharp);
    obj.set_location((0, 0, 0))
}

/// A tap's stream from its spout `tip` down to the tub's floor, hidden at rest (the sinks' `water`).
pub fn stream<X: Coord>(f: &GameFrame, tip: (f64, f64, X), floor_y: f64, name: &str) -> Obj {
    let (x, y, z) = tip;
    let parts = new_parts(|| {
        fcyl!(
            f,
            name,
            0.011,
            y - floor_y,
            (x, (y + floor_y) / 2.0, z),
            M.water,
            verts = 10
        );
    });
    let obj = join(&parts, name, Some(f.at(x, y, z)));
    obj.edit(|o| o.scale.z = 0.001);
    obj
}

/// The bath water: the tub's inside from just under its floor ring `floor` (a, b, y, n) up to `top`, a little in
/// from the walls, its origin under the floor; scaled to nothing upright and in to the floor's size across, so it
/// is hidden under the floor at rest and widens with the walls as it rises.
pub fn bath_water(f: &GameFrame, floor: Section, top: Section, inset: f64) -> Obj {
    let (a0, b0, y0, n0) = floor;
    let (a1, b1, y1, n1) = top;
    let water = loft(
        "bathwater",
        &[
            (a0 - inset, b0 - inset, y0 - 0.004, n0),
            (a1 - inset, b1 - inset, y1, n1),
        ],
        M.water,
        40,
        40.0,
    );
    set_origin(water, f.at(0, y0 - 0.006, 0));
    water.set_scale((
        (a0 - inset) / (a1 - inset),
        (b0 - inset) / (b1 - inset),
        0.001,
    ));
    water
}

/// The inside ring (a, b, y, n) at height `y`, between the two inside sections around it (the ones after the rim,
/// going down).
pub fn inside_at(sections: &[Section], y: f64) -> Section {
    let heights: Vec<f64> = sections.iter().map(|s| s.2).collect();
    let top = py::max_of(&heights);
    let at = sections.iter().position(|s| s.2 == top).unwrap();
    let inner = &sections[at..];
    for w in inner.windows(2) {
        let ((a0, b0, y0, n0), (a1, b1, y1, n1)) = (w[0], w[1]);
        if y1 <= y && y <= y0 && y0 > y1 {
            let t = (y0 - y) / (y0 - y1);
            return (
                a0 + (a1 - a0) * t,
                b0 + (b1 - b0) * t,
                y,
                n0 + (n1 - n0) * t,
            );
        }
    }
    panic!("{y}");
}

/// The clips: the stream runs out (`tap_on`), the water waits for it and rises (`fill`).
pub fn fill(stream: Obj, water: Obj) {
    grow(
        stream,
        "tap_on",
        Grow {
            seconds: BATH_TAP,
            axis: Axis::Z,
        },
    );
    for axis in 0..3 {
        let rest = water.read(|o| o.scale[axis]) as f64;
        keys(
            water,
            "fill",
            Path::Scale,
            axis,
            &[
                (0.0, rest, Ease::Smooth),
                (BATH_TAP, rest, Ease::Smooth),
                (BATH_TAP + BATH_FILL, 1.0, Ease::Smooth),
            ],
        );
    }
}

/// The tub's shell; returns its floor ring and the water's top ring, two thirds of the way up.
pub fn tub(sections: &[Section], rim: f64, floor_y: f64) -> (Section, Section) {
    loft("tub", sections, M.porcelain, 48, 40.0);
    let fill = floor_y + (rim - floor_y) * 2.0 / 3.0;
    (sections[sections.len() - 1], inside_at(sections, fill))
}

/// A rubber duck on the deck at game (x, y, z), looking along -x: a plump body rising to a tail, a head, a beak.
pub fn duck(f: &GameFrame, x: f64, y: f64, z: f64) {
    sphere!(
        "duck",
        0.035,
        at = f.at(x, y + 0.028, z),
        material = M.duck,
        scale = (1.25, 1, 0.85),
        segments = 12,
        rings = 8
    );
    fcone!(
        f,
        "duck_tail",
        0.016,
        0.002,
        0.03,
        (x + 0.04, y + 0.045, z),
        M.duck,
        rot = (0, 60, 0),
        verts = 8
    );
    fsphere!(
        f,
        "duck_head",
        0.022,
        (x - 0.022, y + 0.07, z),
        M.duck,
        segments = 10,
        rings = 7
    );
    fcone!(
        f,
        "duck_beak",
        0.009,
        0.002,
        0.02,
        (x - 0.046, y + 0.066, z),
        M.beak,
        rot = (0, -90, 0),
        verts = 8
    );
}

/// A towel folded on a surface at height `y`: soft layers, each a little back from the one under it.
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn folded_towel(
    f: &GameFrame,
    name: &str,
    w: f64,
    d: f64,
    x: f64,
    y: f64,
    z: f64,
    material: Mat,
    layers: usize,
    t: f64,
) {
    for k in 0..layers {
        let kf = k as f64;
        soft!(
            f,
            format!("{name}{k}"),
            w,
            t,
            d - 0.01 * kf,
            (x, y + t * (kf + 0.5), z - 0.005 * kf),
            material,
            round = t * 0.45,
            puff = (0, 0.004, 0),
            round_segments = 2,
            flat = 2
        );
    }
}

/// A towel hanging over an edge through game points (height, depth) at `x`, `w` wide, `t` thick.
pub fn draped_towel(
    f: &GameFrame,
    name: &str,
    x: f64,
    w: f64,
    pts: &[(f64, f64)],
    material: Mat,
    t: f64,
) {
    let path: Vec<(f64, f64, f64)> = pts.iter().map(|&(y, z)| (x, y, z)).collect();
    fsweep!(
        f,
        name,
        &path,
        0.0,
        material,
        profile = rounded_rect(w, t, t * 0.45, 2),
        up = (1, 0, 0),
        resolution = 2
    );
}

/// A bath's plug on its floor and the overflow's round plate high on its end wall at game x `inner_x`.
pub fn waste(f: &GameFrame, end_x: f64, rim: f64, floor_y: f64, inner_x: f64, material: Mat) {
    rose(f, "plug", end_x, floor_y, 0, 0.03, material, 0.006, 16);
    let side = if inner_x > 0.0 { 1 } else { -1 };
    fcyl!(
        f,
        "overflow",
        0.028,
        0.008,
        (inner_x - side as f64 * 0.004, rim - 0.1, 0),
        material,
        rot = (0, 90, 0),
        verts = 16,
        bevel = 0.003,
        bevel_segments = 1
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_inside_ring_between_sections_and_on_one() {
        // the foot, up the outside to the rim's top (the highest), then down the inside
        let s = [
            (0.80, 0.38, 0.0, 2.0),
            (0.86, 0.42, 0.55, 2.0),
            (0.80, 0.36, 0.50, 3.0),
            (0.70, 0.30, 0.10, 4.0),
        ];
        let near = |a: Section, b: Section| {
            [(a.0, b.0), (a.1, b.1), (a.2, b.2), (a.3, b.3)]
                .iter()
                .all(|(x, y)| (x - y).abs() < 1e-12)
        };
        assert!(near(inside_at(&s, 0.30), (0.75, 0.33, 0.30, 3.5)));
        assert!(near(inside_at(&s, 0.50), s[2]));
    }
}
