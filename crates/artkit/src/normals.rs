//! Shading normals, as Blender 5.2 works them out, to the bit. A corner's normal is its face's on a flat face; on
//! a smooth one, the mean of the faces round its vertex that it joins without crossing a sharp edge (`Mesh::sharp`,
//! a flat face, an open edge, a side two faces run the same way): its fan. A mesh that is smooth all over has one
//! normal a vertex. A vertex's normal is the mean over all its faces (what the baked shading looks along).
//!
//! What makes them Blender's: a face weighs in by Newell's sum over its corners (a triangle's and a quad's too,
//! not the normal a polygon shows) and by its angle at the vertex, taken with an approximate arc cosine between
//! the unit vectors along its two sides there; those and the sum are made unit vectors by division; a fan is
//! summed in the order Blender walks it.

use crate::math::{V3, cosf_alone, dot_v3, sincosf, sinf_alone};
use crate::mesh::{Mesh, edge, normal_newell};

/// A unit vector by division, as the code that works the normals out makes one (Blender's older code multiplies by
/// the reciprocal, now and then a unit in the last place off it); zero for none.
fn unit(a: V3) -> V3 {
    let d = dot_v3(a, a);
    if d > 1.0e-35 {
        let length = d.sqrt();
        V3::new(a.x / length, a.y / length, a.z / length)
    } else {
        V3::ZERO
    }
}

/// The arc cosine as Blender approximates it for a face's weight (to a few hundred-thousandths of a radian).
pub(crate) fn acos_approx(x: f32) -> f32 {
    let f = x.abs();
    let m = if f < 1.0 { 1.0 - (1.0 - f) } else { 1.0 };
    let a = (1.0 - m).sqrt()
        * (std::f32::consts::FRAC_PI_2
            + m * (-0.213_300_99 + m * (0.077_980_48 + m * -0.021_640_95)));
    if x < 0.0 { std::f32::consts::PI - a } else { a }
}

fn add_scaled(sum: V3, v: V3, k: f32) -> V3 {
    V3::new(sum.x + v.x * k, sum.y + v.y * k, sum.z + v.z * k)
}

/// Each face's normal as the shading normals weigh it: by Newell's sum, whatever its corners.
fn weighed_face_normals(mesh: &Mesh) -> Vec<V3> {
    (0..mesh.faces.len())
        .map(|f| {
            let points: Vec<V3> = mesh
                .corners(f)
                .iter()
                .map(|&v| mesh.verts[v as usize])
                .collect();
            normal_newell(&points)
        })
        .collect()
}

/// A face at a vertex: its corner there, and the vertices before and after it round the face.
struct At {
    face: u32,
    corner: u32,
    prev: u32,
    next: u32,
}

/// Each vertex's faces, in the faces' order.
fn faces_at(mesh: &Mesh) -> Vec<Vec<At>> {
    let mut at: Vec<Vec<At>> = (0..mesh.verts.len()).map(|_| Vec::new()).collect();
    for f in 0..mesh.faces.len() {
        let c = mesh.corners(f);
        let n = c.len();
        for &v in c {
            // (a face that comes by a vertex twice counts twice, at the first corner there)
            let i = c.iter().position(|&x| x == v).unwrap_or(0);
            at[v as usize].push(At {
                face: f as u32,
                corner: mesh.faces[f].start + i as u32,
                prev: c[(i + n - 1) % n],
                next: c[(i + 1) % n],
            });
        }
    }
    at
}

/// The mean at each vertex of what `of` gives for each of its faces, weighed by the face's angle there (a vertex
/// of no face: its direction).
fn mixed(mesh: &Mesh, at: &[Vec<At>], of: impl Fn(&At) -> V3) -> Vec<V3> {
    at.iter()
        .zip(&mesh.verts)
        .map(|(faces, &p)| {
            if faces.is_empty() {
                return unit(p);
            }
            let mut sum = V3::ZERO;
            for face in faces {
                let prev = unit(mesh.verts[face.prev as usize] - p);
                let next = unit(mesh.verts[face.next as usize] - p);
                sum = add_scaled(sum, of(face), acos_approx(dot_v3(prev, next)));
            }
            unit(sum)
        })
        .collect()
}

/// A normal a vertex: its faces' normals weighed by their angles at it; on a mesh with normals of its own
/// (`Mesh::normals`), its corners' there, as they read on the mesh as it is (`corner_normals`).
pub fn vertex_normals(mesh: &Mesh) -> Vec<V3> {
    let at = faces_at(mesh);
    match &mesh.normals {
        Some(_) => {
            let own = corner_normals(mesh);
            mixed(mesh, &at, |face| own[face.corner as usize])
        }
        None => {
            let normals = weighed_face_normals(mesh);
            mixed(mesh, &at, |face| normals[face.face as usize])
        }
    }
}

/// A normal a corner, as the file is written with: the mesh's own where it has them (`Mesh::normals`).
pub fn corner_normals(mesh: &Mesh) -> Vec<V3> {
    if let Some(n) = &mesh.normals {
        let mut out = n.clone();
        // what a tool stored for a corner reads in its fan's frame on the mesh as it is now: Blender keeps the
        // two angles, not the normal, through a join and a move
        if let Some(stored) = mesh.stored.as_ref().filter(|s| s.len() == out.len()) {
            let angles: Vec<[i16; 2]> = stored.iter().map(|s| s.unwrap_or([0, 0])).collect();
            for ((own, read), s) in out.iter_mut().zip(from_stored(mesh, &angles)).zip(stored) {
                if s.is_some() {
                    *own = read;
                }
            }
        }
        return out;
    }
    if mesh.faces.iter().all(|f| f.smooth) && mesh.sharp.is_empty() {
        // smooth all over: one normal a vertex
        let normals = vertex_normals(mesh);
        return mesh.loops.iter().map(|&v| normals[v as usize]).collect();
    }
    fans(mesh, Keeps::Nothing)
}

/// A normal a corner for a part with none of its own that is joined to one that has: its fans', whatever the
/// part (one smooth all over too), and none at all (zero) for a fan Blender can build no frame on (`Frame`).
pub(crate) fn joined_normals(mesh: &Mesh) -> Vec<V3> {
    fans(mesh, Keeps::Frames)
}

/// The normals a mesh has once it is given `normals`, one a vertex, for its own
/// (`normals_split_custom_set_from_vertices`): a normal a corner, each as Blender keeps it, which is as two angles
/// of sixteen bits in its fan's frame, a ten-thousandth or so off what was given.
pub fn custom_from_verts(mesh: &Mesh, normals: &[V3]) -> Vec<V3> {
    let given: Vec<V3> = normals.iter().map(|&n| unit(n)).collect();
    fans(mesh, Keeps::Given(&given))
}

/// The normals a mesh has when Blender reads back what a tool stored for its corners, two angles of sixteen bits
/// each (a bevel's hardened normals, stored in the frames of the mesh it was working on): each fan's corners share
/// the mean of theirs, read in the fan's own frame.
pub(crate) fn from_stored(mesh: &Mesh, stored: &[[i16; 2]]) -> Vec<V3> {
    fans(mesh, Keeps::Stored(stored))
}

/// What a fan's normal is kept as.
enum Keeps<'a> {
    /// As it is worked out.
    Nothing,
    /// As it is worked out on a mesh that has normals of its own somewhere: where a frame can be built.
    Frames,
    /// As these normals, one a vertex, come back out of the fan's frame.
    Given(&'a [V3]),
    /// As what is stored for its corners (two angles of sixteen bits each, whatever frame they were stored in)
    /// reads in the fan's frame: the mean of them.
    Stored(&'a [[i16; 2]]),
}

/// A side at a vertex, as the faces on it join across it or not.
#[derive(Clone, Copy)]
enum Side {
    New,
    /// One face so far (its place among the vertex's), and whether it runs the side towards the vertex.
    One(usize, bool),
    /// Two faces, one each way: they join across it.
    Two(usize, usize),
    /// Marked sharp, on a flat face, run the same way by two faces or shared by three: nothing joins across it.
    Sharp,
}

/// A normal a corner by fans (Blender's `normals_calc_corners`).
fn fans(mesh: &Mesh, keeps: Keeps) -> Vec<V3> {
    let normals = weighed_face_normals(mesh);
    let mut out = vec![V3::ZERO; mesh.loops.len()];
    for (v, faces) in faces_at(mesh).iter().enumerate() {
        if faces.is_empty() {
            continue;
        }
        // the sides at the vertex, each by the vertex at its other end, in the order the faces name them
        let mut sides: Vec<u32> = Vec::new();
        let mut side_of = |other: u32| match sides.iter().position(|&s| s == other) {
            Some(i) => i,
            None => {
                sides.push(other);
                sides.len() - 1
            }
        };
        let ends: Vec<(usize, usize)> = faces
            .iter()
            .map(|face| {
                let prev = side_of(face.prev);
                (prev, side_of(face.next))
            })
            .collect();
        let mut joins = vec![Side::New; sides.len()];
        for (i, face) in faces.iter().enumerate() {
            let (prev, next) = ends[i];
            if !mesh.faces[face.face as usize].smooth {
                (joins[prev], joins[next]) = (Side::Sharp, Side::Sharp);
                continue;
            }
            // (the side before runs towards the vertex, the side after away from it)
            for (side, towards) in [(prev, true), (next, false)] {
                joins[side] = match joins[side] {
                    Side::New if mesh.sharp.contains(&edge(v as u32, sides[side])) => Side::Sharp,
                    Side::New => Side::One(i, towards),
                    Side::One(_, way) if way == towards => Side::Sharp,
                    Side::One(first, _) => Side::Two(first, i),
                    _ => Side::Sharp,
                };
            }
        }
        let along: Vec<V3> = sides
            .iter()
            .map(|&other| unit(mesh.verts[other as usize] - mesh.verts[v]))
            .collect();
        let mut seen = vec![false; faces.len()];
        let (mut done, mut start) = (0, 0);
        loop {
            // the fan from the first face not yet in one: on round the vertex the one way, then the other
            let mut fan = vec![start];
            let (mut at, mut round) = (start, false);
            while let Side::Two(a, b) = joins[ends[at].1] {
                at = if at == a { b } else { a };
                if at == start {
                    round = true;
                    break;
                }
                fan.push(at);
            }
            fan.reverse();
            if round {
                // (all the way round: from its lowest corner)
                let first = (0..fan.len())
                    .min_by_key(|&i| faces[fan[i]].corner)
                    .unwrap_or(0);
                fan.rotate_left(first);
            } else {
                at = start;
                while let Side::Two(a, b) = joins[ends[at].0] {
                    at = if at == a { b } else { a };
                    fan.push(at);
                }
            }
            let mut normal = if fan.len() == 1 {
                normals[faces[fan[0]].face as usize]
            } else {
                let mut sum = V3::ZERO;
                for &i in &fan {
                    let weight = acos_approx(dot_v3(along[ends[i].0], along[ends[i].1]));
                    sum = add_scaled(sum, normals[faces[i].face as usize], weight);
                }
                unit(sum)
            };
            if !matches!(keeps, Keeps::Nothing) {
                let (first, last) = (ends[fan[0]].1, ends[fan[fan.len() - 1]].0);
                let mut spokes: Vec<V3> = Vec::new();
                if fan.len() > 1 {
                    spokes.extend(fan.iter().map(|&i| along[ends[i].1]));
                    if last != first {
                        spokes.push(along[last]);
                    }
                }
                let frame = Frame::new(normal, along[first], along[last], &spokes);
                normal = match keeps {
                    Keeps::Given(given) if fan.len() == 1 => frame.kept(given[v]),
                    Keeps::Given(given) => {
                        // (the mean of the fan's, as Blender sums it: the vertex's, once a corner)
                        let mut mean = V3::ZERO;
                        for _ in &fan {
                            mean += given[v];
                        }
                        frame.kept(crate::math::mul_v3_fl(mean, 1.0 / fan.len() as f32))
                    }
                    Keeps::Stored(stored) => {
                        let (mut a, mut b) = (0_i32, 0_i32);
                        for &i in &fan {
                            let s = stored[faces[i].corner as usize];
                            a += s[0] as i32;
                            b += s[1] as i32;
                        }
                        let n = fan.len() as i32;
                        frame.read((a / n) as i16, (b / n) as i16)
                    }
                    _ => frame.normal,
                };
            }
            for &i in &fan {
                out[faces[i].corner as usize] = normal;
                seen[i] = true;
            }
            done += fan.len();
            if done >= faces.len() {
                break;
            }
            while seen[start] {
                start += 1;
            }
        }
    }
    out
}

/// A side too nearly along a fan's normal to build its frame on, and two directions too nearly one to tell the
/// angle between them: by their cosine.
const ALONG: f32 = 1.0 - 1e-4;
const TURN: f32 = std::f32::consts::PI * 2.0;

/// The frame Blender keeps a fan's own normal in: the fan's worked-out normal, a direction square to it towards
/// the fan's first side and one square to both, the mean angle from the normal to the fan's sides, and the angle
/// round the normal from its first side to its last.
struct Frame {
    normal: V3,
    reference: V3,
    ortho: V3,
    alpha: f32,
    beta: f32,
}

impl Frame {
    /// The frame of a fan with `normal`, from its `first` side round to its `last` (unit vectors along them;
    /// `spokes`: all its sides, for a fan of more than one corner). No frame (all zero, the normal too) on a side
    /// along the normal.
    fn new(normal: V3, first: V3, last: V3, spokes: &[V3]) -> Frame {
        let (dot_first, dot_last) = (dot_v3(first, normal), dot_v3(last, normal));
        if dot_first.abs() >= ALONG || dot_last.abs() >= ALONG {
            return Frame {
                normal: V3::ZERO,
                reference: V3::ZERO,
                ortho: V3::ZERO,
                alpha: 0.0,
                beta: 0.0,
            };
        }
        let alpha = if spokes.is_empty() {
            (acos_approx(dot_v3(first, normal)) + acos_approx(dot_v3(last, normal))) / 2.0
        } else {
            let mut sum = 0.0_f32;
            for &spoke in spokes {
                sum += acos_approx(dot_v3(spoke, normal));
            }
            sum / spokes.len() as f32
        };
        let reference = unit(add_scaled(first, normal, -dot_first));
        let ortho = unit(normal.cross(reference));
        let other = unit(add_scaled(last, normal, -dot_last));
        let turn = dot_v3(reference, other);
        let beta = if turn < ALONG {
            let beta = acos_approx(turn);
            if dot_v3(ortho, other) < 0.0 {
                TURN - beta
            } else {
                beta
            }
        } else {
            TURN
        };
        Frame {
            normal,
            reference,
            ortho,
            alpha,
            beta,
        }
    }

    /// `given` (a unit vector) as it comes back out of the frame: turned into two angles, each a share of the
    /// frame's own in sixteen bits, and back.
    fn kept(&self, given: V3) -> V3 {
        let (a, b) = self.angles(given);
        self.read(a, b)
    }

    /// The normal two stored angles stand for in the frame.
    fn read(&self, a: i16, b: i16) -> V3 {
        if a == 0 || self.alpha == 0.0 || self.beta == 0.0 {
            return self.normal;
        }
        let share = |x: i16| x as f32 / i16::MAX as f32;
        let alpha_share = share(a);
        let alpha = if alpha_share > 0.0 {
            self.alpha
        } else {
            TURN - self.alpha
        } * alpha_share;
        let beta_share = share(b);
        // (the first angle's cosine and sine are asked for apart, the second's together: other routines of the
        // library, which round a little differently)
        let mut out = crate::math::mul_v3_fl(self.normal, cosf_alone(alpha));
        let sin_alpha = sinf_alone(alpha);
        if beta_share == 0.0 {
            out = add_scaled(out, self.reference, sin_alpha);
        } else {
            let beta = if beta_share > 0.0 {
                self.beta
            } else {
                TURN - self.beta
            } * beta_share;
            let (sin_beta, cos_beta) = sincosf(beta);
            out = add_scaled(out, self.reference, sin_alpha * cos_beta);
            out = add_scaled(out, self.ortho, sin_alpha * sin_beta);
        }
        out
    }

    /// The two angles `given` is kept as (zeros: the frame's own normal, which one within a ten-thousandth of it
    /// is taken for).
    fn angles(&self, given: V3) -> (i16, i16) {
        let near = (0..3).all(|i| (self.normal[i] - given[i]).abs() <= 1e-4);
        if given == V3::ZERO || near {
            return (0, 0);
        }
        let short = |x: f32| (x * i16::MAX as f32 + 0.5).floor() as i16;
        let cos_alpha = dot_v3(self.normal, given);
        let alpha = acos_approx(cos_alpha);
        let a = if alpha > self.alpha {
            short(-(TURN - alpha) / (TURN - self.alpha))
        } else {
            short(alpha / self.alpha)
        };
        // the given normal's direction round the frame's
        let flat = unit(add_scaled(given, self.normal, -cos_alpha));
        let cos_beta = dot_v3(self.reference, flat);
        let b = if cos_beta < ALONG {
            let mut beta = acos_approx(cos_beta);
            if dot_v3(self.ortho, flat) < 0.0 {
                beta = TURN - beta;
            }
            if beta > self.beta {
                short(-(TURN - beta) / (TURN - self.beta))
            } else {
                short(beta / self.beta)
            }
        } else {
            0
        };
        (a, b)
    }
}
