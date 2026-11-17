//! Shading baked into the vertices: how much of the sky each vertex sees (ambient occlusion), written as a grey
//! vertex colour (glTF COLOR_0) that the game multiplies into the part's colour. Creases, seams, the gaps between
//! cushions, drawer reveals and undersides darken; open faces stay as they are. No UVs, no texture, no draw call.
//!
//! A part that moves away from where it rests (a door, a drawer, something that grows or is hidden) is shaded only
//! by itself, so it isn't left with the shadow of what it moved away from; everything else (the still part, a
//! cushion that only sinks) is shaded by all of those together, at rest.
//!
//! A model on the ground (`ground`) is also shaded by the floor it stands on, near it only (`GROUND_REACH`): the
//! feet of its sides darken where they meet it.
//!
//! A tree's leaf cards carry their own shading on their corners (`Mesh::card_shade`, written as they are laid: open
//! on the crown's outside, darker inside it) and shade nothing: they are leaves, not walls.

use crate::bvh::Bvh;
use crate::math::{self, Axis, M3, PI, V3, v3};
use crate::mesh::normal_newell;
use crate::motion;
use crate::parts::measure;
use crate::py;
use crate::scene::Obj;

pub const SAMPLES: usize = 48;
/// How dark a fully enclosed vertex goes: 1 - STRENGTH.
pub const STRENGTH: f64 = 0.8;
/// Rays start this far off the surface (m), so they don't hit their own face.
pub const LIFT: f64 = 0.0015;
/// Greys it is rounded to: neighbours share a value, which the build's compression packs tightly.
pub const LEVELS: f64 = 32.0;
/// m: how far up its sides the floor shades a model standing on it.
pub const GROUND_REACH: f64 = 0.15;

/// `n` directions over the hemisphere about +Z, cosine-weighted (more towards the top), evenly spread (a
/// Fibonacci spiral): the same every build.
pub fn directions(n: usize) -> Vec<V3> {
    let golden = PI * (3.0 - 5.0_f64.sqrt());
    (0..n)
        .map(|i| {
            let r = ((i as f64 + 0.5) / n as f64).sqrt();
            let a = golden * i as f64;
            v3(
                r * math::cos(a),
                r * math::sin(a),
                (1.0 - r * r).max(0.0).sqrt(),
            )
        })
        .collect()
}

/// The triangles rays are cast at, in the world at rest or (`local`) in the one part's own space, with each
/// one's face's normal. Leaf cards shade nothing.
struct Tree {
    bvh: Bvh,
    normals: Vec<V3>,
}

fn tree(objs: &[Obj], local: bool) -> Tree {
    let (mut tris, mut normals) = (Vec::new(), Vec::new());
    for &o in objs {
        let points = if local {
            o.read(|p| p.mesh.verts.clone())
        } else {
            o.world_points()
        };
        o.read(|p| {
            let mesh = &p.mesh;
            for f in 0..mesh.faces.len() {
                let c = mesh.corners(f);
                if mesh
                    .card_shade
                    .as_ref()
                    .is_some_and(|card| c.iter().all(|&v| card[v as usize] > 0.0))
                {
                    continue;
                }
                let poly: Vec<V3> = c.iter().map(|&v| points[v as usize]).collect();
                let normal = normal_newell(&poly);
                let fan: Vec<[u32; 3]> = if poly.len() == 3 {
                    vec![[0, 1, 2]]
                } else {
                    crate::tri::polygon(&poly, normal)
                };
                for t in fan {
                    tris.push([
                        poly[t[0] as usize],
                        poly[t[1] as usize],
                        poly[t[2] as usize],
                    ]);
                    normals.push(normal);
                }
            }
        });
    }
    Tree {
        bvh: Bvh::new(tris),
        normals,
    }
}

/// The share of the sky a point `p` with normal `n` sees, 0 (enclosed) to 1 (open), from rays out over the
/// hemisphere about `n`, a hit counting less the farther away it is (up to `reach`); with `ground`, a ray that
/// reaches down to the floor (z = 0) within `GROUND_REACH` is blocked by it too.
fn seen(tree: &Tree, dirs: &[V3], p: V3, n: V3, twist: usize, reach: f64, ground: bool) -> f64 {
    // a turn taking +Z onto the normal, twisted (by the vertex's index) so neighbours don't band
    let turn =
        V3::Z.rotation_difference(n).to_matrix() * M3::rotation(twist as f64 * 2.399963, Axis::Z);
    let start = p + n * LIFT;
    let (mut blocked, mut inside) = (0.0_f64, 0usize);
    for &d in dirs {
        let ray = turn * d;
        let dir = crate::math::normalize_v3(ray).0;
        match tree.bvh.ray_cast(start, dir, reach as f32) {
            Some((t, dist)) => {
                if tree.normals[t as usize].dot(ray) > 0.0 {
                    // the back of a face: this point is inside another part, not under it
                    inside += 1;
                } else {
                    blocked += 1.0 - math::pow(dist as f64 / reach, 2.0);
                }
            }
            None => {
                if ground && (ray.z as f64) < -1e-6 && start.z > 0.0 {
                    let down = start.z as f64 / -(ray.z as f64);
                    if down < GROUND_REACH {
                        blocked += 1.0 - math::pow(down / GROUND_REACH, 2.0);
                    }
                }
            }
        }
    }
    // A point buried in another part (a panel's corner inside its frame, mirror glass in its rim) is hidden;
    // shaded, it would darken the whole face it shares, so it stays open.
    if inside > dirs.len() / 2 {
        1.0
    } else {
        1.0 - blocked / dirs.len() as f64
    }
}

/// Bakes each of `objs`' (the model at rest, clips made) occlusion into its vertices' shading (`Mesh::ao`), which
/// the file carries as COLOR_0. `reach`: how far a crease is looked for, by default a fifth of the model's largest
/// side (4 cm to 40 cm). `ground`: the model stands on the floor (z = 0), which shades the foot of its sides; a
/// part that moves away is shaded by itself only, without it.
pub fn bake_ao(objs: &[Obj], reach: Option<f64>, ground: bool) {
    let reach = reach.unwrap_or_else(|| {
        let (lo, hi) = measure(objs);
        let largest = (0..3)
            .map(|i| hi[i] - lo[i])
            .fold(f64::NEG_INFINITY, f64::max);
        0.4_f64.min(0.04_f64.max(0.2 * largest))
    });
    let dirs = directions(SAMPLES);
    let away: Vec<bool> = objs.iter().map(|&o| motion::moves_away(o)).collect();
    let together: Vec<Obj> = objs
        .iter()
        .zip(&away)
        .filter(|(_, a)| !**a)
        .map(|(o, _)| *o)
        .collect();
    let shared = if together.is_empty() {
        None
    } else {
        Some(tree(&together, false))
    };
    for (&o, &local) in objs.iter().zip(&away) {
        let own;
        let t = if local {
            own = tree(&[o], true);
            &own
        } else {
            match &shared {
                Some(t) => t,
                None => continue,
            }
        };
        let (points, normals, card) = o.read(|p| {
            (
                p.mesh.verts.clone(),
                crate::normals::vertex_normals(&p.mesh),
                p.mesh.card_shade.clone(),
            )
        });
        let world = o.matrix_world();
        let plain = local || world.to_3x3() == M3::IDENTITY;
        let ao: Vec<f32> = (0..points.len())
            .map(|i| {
                let (p, n) = if local || world.is_identity() {
                    (points[i], normals[i].normalized())
                } else if plain {
                    (world * points[i], normals[i].normalized())
                } else {
                    (
                        world * points[i],
                        normal_through(&world.to_3x3(), normals[i]),
                    )
                };
                let mut s = seen(t, &dirs, p, n, i, reach, ground && !local);
                if let Some(c) = card.as_ref().map(|c| c[i] as f64).filter(|&c| c > 0.0) {
                    // (what it sees, for the grey below to come out as its own shading)
                    s = 1.0 - (1.0 - c) / STRENGTH;
                }
                (py::round((1.0 - STRENGTH * (1.0 - s)) * LEVELS) as f64 / LEVELS) as f32
            })
            .collect();
        o.edit(|p| p.mesh.ao = Some(ao));
    }
}

/// A normal through a part's turn and scale (the inverse transpose), a unit vector again.
fn normal_through(m: &M3, n: V3) -> V3 {
    let a = m.0.map(|c| c.map(|x| x as f64));
    let cof = |i: usize, j: usize| {
        let (i1, i2, j1, j2) = ((i + 1) % 3, (i + 2) % 3, (j + 1) % 3, (j + 2) % 3);
        a[i1][j1] * a[i2][j2] - a[i1][j2] * a[i2][j1]
    };
    // the cofactors are the inverse transpose but for the determinant, which a unit vector doesn't keep
    let v = [n.x as f64, n.y as f64, n.z as f64];
    let out: Vec<f64> = (0..3)
        .map(|r| (0..3).map(|c| cof(c, r) * v[c]).sum())
        .collect();
    v3(out[0], out[1], out[2]).normalized()
}
