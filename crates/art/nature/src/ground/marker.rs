//! The selection marker: the sparkle star over the selected resident.

use super::M;
use super::common::mesh;
use artkit::edit::{NgonMethod, QuadMethod, triangulate};
use artkit::math::{PI, TAU, V3, cos, pow, sin, sqrt, v3};
use artkit::normals::custom_from_verts;
use artkit::py;
use artkit::scene::Obj;
use std::collections::BTreeSet;

/// The old part's star: rays (half-length, half-width at the middle, half-depth at the middle) up, down and to the
/// sides, as lathes whose radius eases to the tip as (1 - u) ** 1.6, round a core 5 cm round and 3 cm deep.
const RAYS: [(f64, f64, f64); 2] = [(0.17, 0.0425, 0.025), (0.115, 0.0375, 0.0225)];
const CORE: (f64, f64) = (0.05, 0.03);

/// The star's half-depth at (a across, b up) in its plane: a smooth union of the rays' and the core's.
fn depth(a: f64, b: f64) -> f64 {
    let mut parts = Vec::new();
    for ((length, w0, d0), (u, l)) in [(RAYS[0], (b.abs(), a)), (RAYS[1], (a.abs(), b))] {
        if u < length {
            let f = pow(1.0 - u / length, 1.6);
            let (w, d) = (w0 * f, d0 * f);
            if l.abs() < w {
                parts.push(d * sqrt(1.0 - pow(l / w, 2.0)));
            }
        }
    }
    let r = py::hypot(a, b);
    if r < CORE.0 {
        parts.push(CORE.1 * sqrt(1.0 - pow(r / CORE.0, 2.0)));
    }
    if parts.is_empty() {
        0.0
    } else {
        pow(py::sum(parts.iter().map(|&p| pow(p, 6.0))), 1.0 / 6.0)
    }
}

/// The star's outline as `keep` points evenly spaced along it (so the tips get as many as the flanks), each the
/// furthest point inside it along a direction from the middle.
fn outline(samples: usize, keep: usize) -> Vec<(f64, f64)> {
    let mut pts = Vec::with_capacity(samples);
    for i in 0..samples {
        let t = TAU * i as f64 / samples as f64;
        let (c, s) = (cos(t), sin(t));
        let (mut lo, mut hi) = (0.0, 0.18);
        for _ in 0..30 {
            let m = (lo + hi) / 2.0;
            (lo, hi) = if depth(c * m, s * m) > 0.0 {
                (m, hi)
            } else {
                (lo, m)
            };
        }
        pts.push((c * lo, s * lo));
    }
    // the tips exactly: the outline's samples there fall a little short
    let tip = |k: usize| {
        let length = if k.is_multiple_of(2) {
            RAYS[1].0
        } else {
            RAYS[0].0
        };
        (
            cos(k as f64 * PI / 2.0) * length,
            sin(k as f64 * PI / 2.0) * length,
        )
    };
    for k in 0..4 {
        pts[k * samples / 4] = tip(k);
    }
    let mut lengths = vec![0.0];
    for i in 1..=samples {
        let (p, q) = (pts[i - 1], pts[i % samples]);
        lengths.push(lengths[i - 1] + py::hypot(q.0 - p.0, q.1 - p.1));
    }
    let (mut out, mut j) = (Vec::with_capacity(keep), 0);
    for k in 0..keep {
        let want = lengths[samples] * k as f64 / keep as f64;
        while lengths[j + 1] < want {
            j += 1;
        }
        let f = (want - lengths[j]) / (lengths[j + 1] - lengths[j]).max(1e-9);
        let (p, q) = (pts[j], pts[(j + 1) % samples]);
        out.push((p.0 + (q.0 - p.0) * f, p.1 + (q.1 - p.1) * f));
    }
    // each tip on a point of its own
    for k in 0..4 {
        let tip = tip(k);
        let mut nearest = 0;
        for n in 1..keep {
            if py::hypot(out[n].0 - tip.0, out[n].1 - tip.1)
                < py::hypot(out[nearest].0 - tip.0, out[nearest].1 - tip.1)
            {
                nearest = n;
            }
        }
        out[nearest] = tip;
    }
    out
}

/// The selection marker: a soft four-point sparkle star, tall rays up and down and shorter ones to the sides,
/// their flanks concave, round a rounder core, as the old part's lathes and sphere but one closed, smooth shell
/// (its front and back meet at the outline), which gives the game's pushed-out outline no seams. In the game's
/// frame it lies across x (right) and y (up), thin in z, which the game turns to the camera; its origin is its
/// middle, as the old part's. Its normals come from the shell's shape (`normal`). 1,344 triangles (the old part:
/// 1,936).
pub fn marker() -> Vec<Obj> {
    let outline = outline(4000, 96);
    let (n, rings) = (outline.len(), 4);
    let (mut verts, mut normals): (Vec<V3>, Vec<V3>) = (Vec::new(), Vec::new());
    let h = 1e-4;
    // The shell's normal (game) from its depth's slope: the game pushes its outline out along it, so it is worked
    // out, not left to the triangles (thin ones along the rays folded the outline through the front).
    let normal = |a: f64, b: f64, side: f64| {
        let da = (depth(a + h, b) - depth(a - h, b)) / (2.0 * h);
        let db = (depth(a, b + h) - depth(a, b - h)) / (2.0 * h);
        v3(-da, -db, side).normalized()
    };
    // Game (a, b, depth) -> Blender (-a, -depth, b).
    let to_blender = |v: V3| v3(-v.x, -v.z, v.y);
    for side in [1.0, -1.0] {
        verts.push(v3(0, -side * depth(0.0, 0.0), 0));
        normals.push(to_blender(v3(0, 0, side)));
        for j in 1..=rings {
            // closer together towards the outline, where the shell turns
            let f = 1.0 - pow(1.0 - j as f64 / rings as f64, 2.0);
            for (i, &(a, b)) in outline.iter().enumerate() {
                let d = if j == rings { 0.0 } else { depth(a * f, b * f) };
                verts.push(v3(-a * f, -side * d, b * f));
                if j == rings {
                    // On the outline: square to it, in the star's plane.
                    let ((pa, pb), (qa, qb)) = (outline[(i + n - 1) % n], outline[(i + 1) % n]);
                    let mut out = v3(qb - pb, -(qa - pa), 0);
                    if out.length() < 1e-9 || out.dot(v3(a, b, 0)) <= 0.0 {
                        out = v3(a, b, 0);
                    }
                    normals.push(to_blender(out.normalized()));
                } else {
                    normals.push(to_blender(normal(a * f, b * f, side)));
                }
            }
        }
    }
    let per = 1 + rings * n;
    // the back shares the outline (the last ring) with the front
    let rim: Vec<usize> = (0..n).map(|i| 1 + (rings - 1) * n + i).collect();
    let at = |side: usize, j: usize, i: usize| {
        if j == rings {
            rim[i % n]
        } else {
            side * per + 1 + (j - 1) * n + i % n
        }
    };
    let mut faces: Vec<Vec<usize>> = Vec::new();
    for side in [0, 1] {
        let c = side * per;
        for i in 0..n {
            faces.push(vec![c, at(side, 1, i), at(side, 1, i + 1)]);
        }
        for j in 1..rings {
            for i in 0..n {
                faces.push(vec![
                    at(side, j, i),
                    at(side, j + 1, i),
                    at(side, j + 1, i + 1),
                    at(side, j, i + 1),
                ]);
            }
        }
    }
    let used: Vec<usize> = faces
        .iter()
        .flatten()
        .copied()
        .collect::<BTreeSet<usize>>()
        .into_iter()
        .collect();
    let mut index = vec![u32::MAX; verts.len()];
    for (k, &v) in used.iter().enumerate() {
        index[v] = k as u32;
    }
    let remapped: Vec<Vec<u32>> = faces
        .iter()
        .map(|f| f.iter().map(|&v| index[v]).collect())
        .collect();
    let obj = mesh(
        "marker",
        used.iter().map(|&v| verts[v]).collect(),
        &remapped,
        M.star,
        true,
        0.0,
        true,
    );
    let given: Vec<V3> = used.iter().map(|&v| normals[v]).collect();
    // Quads to triangles along their shorter diagonal, so the outline and the rays' flanks stay smooth.
    triangulate(obj, QuadMethod::ShortEdge, NgonMethod::Beauty);
    obj.edit(|o| {
        for f in &mut o.mesh.faces {
            f.smooth = true;
        }
        o.mesh.normals = Some(custom_from_verts(&o.mesh, &given));
    });
    vec![obj]
}
