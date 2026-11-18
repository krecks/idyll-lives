//! Bevelled edges, as Blender's bevel modifier rounds them the way gamekit set it (shapes.py `bevel_edges`: by
//! angle, an even offset, a round profile, hardened normals): the method is Blender's own (its bevel tool's,
//! bmesh_bevel.cc, GPL-2.0-or-later), worked through again here for the meshes the builders make: a box, a
//! cylinder or prism, an outline's slab, and those after a build has moved their points. Every edge between two
//! faces that turn more than the angle is cut back by the width on both faces and bridged in `segments` strips;
//! where three or more such edges meet, the corner is a polygon (one segment) or a patch (more).
//!
//! What a part's look hangs on is kept as Blender has it, in single precision and in its order:
//! - the width is cut back for the whole part where a face has no room for it (a box thinner than two widths, and
//!   a slab's face by the triangles it is cut into: `tri::fill`);
//! - a new point between two bevelled edges with one other edge between them slides along that edge, and the
//!   widths along a run of such points are then evened (least squares), so a slab's rim is not one width all round;
//! - a corner of three edges at right angles, all cut back alike, takes an eighth of a sphere; any other takes
//!   a patch subdivided from its sides' profiles;
//! - the new points come in the order the modifier leaves them (the baked shading twists its rays by a point's
//!   number), on a box, a cylinder and a slab: each corner's from the edge Blender starts it at, which hangs on
//!   how the mesh's edges are numbered (`EdgeOrder`).
//!
//! The normals are hardened: the faces the bevel joins stay flat, and each strip's corners take the normal of
//! the face they touch (or of the strips either side, between two), so the bevel shades as a rounded edge.
//! Blender stores each of those normals in the frame of its corner's fan on the mesh the bevel was working on (two
//! angles, 16 bits each) and the mesh it leaves reads them back in its own fans' frames, the mean of them where
//! a fan's corners were given unlike ones: so are they here, which is why a normal is now and then not the one
//! given (round a profile's middle point it leans to one side).
//!
//! A bevel is asked for (`bevel_edges`, a builder's `bevel`) and given later, as Blender's modifier is: the part
//! keeps its mesh, which a build can still push about, scale or cut, and is rounded as it is when it is joined
//! (`parts::join`), measured or written. A textured part's UVs are made on the mesh as it is then and carried
//! onto the bevel as Blender carries them (`carried`).
//!
//! What isn't built says so: an edge whose bevel ends at a vertex no other bevelled edge reaches, and a patch of
//! five segments or more.

use crate::math::{PI, V3, cosf, dot_v3, len_v3, mul_v3_fl, normalize_v3, sinf};
use crate::mesh::{Mesh, edge, normal_newell, normal_tri};
use crate::scene::Obj;
use crate::shapes::{SHARP, shade};
use std::collections::{BTreeMap, BTreeSet};

const EPSILON: f32 = 1e-6;
const EPSILON_BIG: f32 = 1e-4;
/// Two edges within two degrees of one line are parallel.
const EPSILON_ANG: f32 = 0.034_906_585;
/// Under ten degrees between two edges, the face's own normal says which plane they meet in.
const SMALL_ANG: f32 = 0.174_532_92;
const GOOD_ANGLE: f32 = 0.1;
/// How far towards the corner a patch's middle stands, by its segments (a round profile).
const FULLNESS: [f32; 4] = [0.0, 0.559, 0.642, 0.551];

/// How a mesh's edges are numbered and which way each runs, which decides the edge a corner's new points start
/// from and the corner each strip starts at.
#[derive(Clone, Debug)]
pub enum EdgeOrder {
    /// As the faces' corners name them, each face from its first side, each from its lower vertex to its higher
    /// (a mesh made of points and faces).
    Corners,
    /// Each face's closing side first, then its others, each the way that face runs it (a primitive of Blender's
    /// own).
    Faces,
    /// These first, in this order, each from its first vertex to its second; the others after them, as `Corners`
    /// has them (an outline's rims, every edge of a cylinder).
    Listed(Vec<(u32, u32)>),
}

/// What a face of the bevelled mesh is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// A face no bevel touched.
    Kept,
    /// A face of the mesh, its corners cut back.
    Cut,
    /// A strip along a bevelled edge.
    Strip,
    /// A corner's polygon or patch.
    Corner,
}

/// A new face whose texture hangs on the old mesh's UVs (`carried`): which old face's it takes is settled once
/// they are known.
#[derive(Clone, Debug)]
pub enum Pick {
    /// A strip's quad (its first corner's place among the new mesh's), step `k` of the strips along the old edge
    /// `edge` from the face `faces.0`, which its first two corners are nearer, to `faces.1`.
    Strip {
        at: usize,
        edge: (u32, u32),
        faces: (u32, u32),
        k: usize,
    },
    /// A corner's polygon at the old vertex `vert`: the face each of its corners lies on, and round the vertex
    /// each bevelled edge's other end with the faces before and after it.
    Corner {
        at: usize,
        vert: u32,
        on: Vec<u32>,
        edges: Vec<(u32, u32, u32)>,
        /// (Of an odd number of segments over one, the polygon in the middle of a patch: as a patch quad's.)
        across: Vec<(u32, u32, u32)>,
    },
    /// A quad of a corner's patch of an odd number of segments at the old vertex `vert`: the one at `place` (new
    /// point, ring, step). `across`: for each new point round the vertex, the bevelled edge its profile to the next
    /// crosses (its other end) and the faces before and after that edge.
    Patch {
        at: usize,
        vert: u32,
        place: (usize, usize, usize),
        across: Vec<(u32, u32, u32)>,
    },
}

/// A mesh with its edges bevelled.
#[derive(Clone, Debug)]
pub struct Bevelled {
    pub mesh: Mesh,
    pub kinds: Vec<Kind>,
    /// For each corner of the new mesh, the face of the old one whose texture it takes (but for `picks`).
    pub source: Vec<u32>,
    pub picks: Vec<Pick>,
    pub segments: usize,
    /// The offset the edges were cut back by: the width, or less where the mesh has no room for it.
    pub offset: f32,
}

/// The faces' normals as the mesh the bevel works on keeps them (Blender's BMesh): a triangle or a quad of no
/// area has none, where a mesh's own is straight up.
fn face_normals(mesh: &Mesh) -> Vec<V3> {
    (0..mesh.faces.len())
        .map(|f| {
            let p: Vec<V3> = mesh
                .corners(f)
                .iter()
                .map(|&v| mesh.verts[v as usize])
                .collect();
            match p.len() {
                3 => normal_tri(p[0], p[1], p[2]),
                4 => normalize_v3((p[0] - p[2]).cross(p[1] - p[3])).0,
                _ => normal_newell(&p),
            }
        })
        .collect()
}

fn mid(a: V3, b: V3) -> V3 {
    V3::new(0.5 * (a.x + b.x), 0.5 * (a.y + b.y), 0.5 * (a.z + b.z))
}

fn madd(a: V3, b: V3, k: f32) -> V3 {
    V3::new(a.x + b.x * k, a.y + b.y * k, a.z + b.z * k)
}

fn saasin(f: f32) -> f32 {
    if f <= -1.0 {
        -std::f32::consts::FRAC_PI_2
    } else if f >= 1.0 {
        std::f32::consts::FRAC_PI_2
    } else {
        // (the single-precision routine: nearer Blender's C library's than the double one rounded)
        libm::asinf(f)
    }
}

/// The angle between two unit vectors, as Blender takes it.
fn angle_normalized(a: V3, b: V3) -> f32 {
    if dot_v3(a, b) >= 0.0 {
        2.0 * saasin(len_v3(b - a) / 2.0)
    } else {
        std::f32::consts::PI - 2.0 * saasin(len_v3(-b - a) / 2.0)
    }
}

fn angle(a: V3, b: V3) -> f32 {
    angle_normalized(normalize_v3(a).0, normalize_v3(b).0)
}

/// Where two lines pass nearest each other: (how they lie: 0 one line, 1 they meet, 2 skew; the point on the
/// first; the point on the second).
fn isect_line_line(v1: V3, v2: V3, v3: V3, v4: V3) -> (u8, V3, V3) {
    let c = v3 - v1;
    let a = v2 - v1;
    let b = v4 - v3;
    let ab = a.cross(b);
    let d = dot_v3(c, ab);
    let div = dot_v3(ab, ab);
    if div == 0.0 {
        return (0, v1, v3);
    }
    if d.abs() <= 0.000_001 {
        let cb = c.cross(b);
        let i1 = v1 + mul_v3_fl(a, dot_v3(cb, ab) / div);
        return (1, i1, i1);
    }
    let t = v1 - v3;
    let n = a.cross(b);
    // the offset between the planes the lines lie in
    let t = mul_v3_fl(n, dot_v3(t, n) / dot_v3(n, n));
    let (v3t, v4t) = (v3 + t, v4 + t);
    let c = v3t - v1;
    let a = v2 - v1;
    let b = v4t - v3t;
    let ab = a.cross(b);
    let cb = c.cross(b);
    let i1 = v1 + mul_v3_fl(a, dot_v3(cb, ab) / dot_v3(ab, ab));
    (2, i1, i1 - t)
}

/// One end of an edge at a vertex.
#[derive(Clone, Debug)]
struct Half {
    to: u32,
    bev: bool,
    /// The face between this edge and the next round the vertex.
    fnext: u32,
    /// How far it is cut back on the face before it and on the face after it.
    offset_l: f32,
    offset_r: f32,
    /// The new points either side of it (places among the vertex's).
    left: usize,
    right: usize,
}

/// A new point at a vertex, between two bevelled edges.
#[derive(Clone, Debug)]
struct Bound {
    co: V3,
    /// The bevelled edge before it round the vertex.
    efirst: usize,
    /// The bevelled edge after it, which its profile to the next point crosses.
    ebev: usize,
    /// The edge it slid along, if it did, and the sines of that edge's angles to the bevelled edges after
    /// and before it, one over the other.
    eon: Option<usize>,
    sinratio: f32,
    /// The face it lies on, for the texture.
    face: u32,
    visited: bool,
    /// The point along the next bevelled edge whose width is evened with this one's.
    chain: Option<(u32, usize)>,
}

#[derive(Clone, Debug, Default)]
struct Corner {
    v: u32,
    edges: Vec<Half>,
    bounds: Vec<Bound>,
}

/// A face a corner makes: its corners, each one's source face, its material, for the corner's polygon the vertex,
/// and for a patch's quad the vertex and its place in the patch (new point, ring, step).
type CornerFace = (
    Vec<u32>,
    Vec<u32>,
    u16,
    Option<u32>,
    Option<(u32, usize, usize, usize)>,
);

/// Each edge's faces, with the places of its two ends among the mesh's corners.
type Sides = BTreeMap<(u32, u32), Vec<(u32, usize, usize)>>;

/// A vertex's new points' numbers by (bound, ring, step).
type Numbers = BTreeMap<(usize, usize, usize), u32>;

impl Corner {
    fn half_to(&self, to: u32) -> usize {
        self.edges
            .iter()
            .position(|h| h.to == to)
            .expect("an edge of the vertex")
    }
}

/// The patch point that stands for (i, j, k): each is made once.
fn canon(n: usize, ns: usize, i: usize, j: usize, k: usize) -> (usize, usize, usize) {
    let ns2 = ns / 2;
    if ns % 2 == 1 {
        if j <= ns2 && k <= ns2 {
            (i, j, k)
        } else if k <= ns2 {
            ((i + n - 1) % n, k, ns - j)
        } else {
            ((i + 1) % n, ns - k, j)
        }
    } else if j < ns2 && k <= ns2 {
        (i, j, k)
    } else if j == ns2 && k == ns2 {
        (0, j, k)
    } else if k <= ns2 {
        ((i + n - 1) % n, k, ns - j)
    } else {
        ((i + 1) % n, ns - k, j)
    }
}

struct Work<'a> {
    mesh: &'a Mesh,
    normals: Vec<V3>,
    vertex_normals: Vec<V3>,
    /// The face and corner a directed edge leaves from.
    leaving: BTreeMap<(u32, u32), (u32, u32)>,
    corners: BTreeMap<u32, Corner>,
}

impl Work<'_> {
    fn co(&self, v: u32) -> V3 {
        self.mesh.verts[v as usize]
    }

    /// The angle at `b` between `a` and `c`.
    fn angle3(&self, a: u32, b: u32, c: u32) -> f32 {
        angle(self.co(a) - self.co(b), self.co(c) - self.co(b))
    }

    /// The largest offset the edge `i` of `corner` has room for before the cut faces collide (Blender's
    /// `geometry_collide_offset`, in its arithmetic: a slab's clamped corners close to the last bit or don't).
    fn collide_offset(&self, corner: &Corner, i: usize, offset: f32) -> f32 {
        let no_collide = offset + 1e6;
        let mut limit = no_collide;
        let eb = &corner.edges[i];
        let ea = &corner.edges[(i + 1) % corner.edges.len()];
        let k = |bev: bool| if bev { offset } else { 0.0 };
        let (ka, kb) = (k(ea.bev), k(eb.bev));
        let (va, vb, vc) = (ea.to, corner.v, eb.to);
        let Some(&(f, at)) = self.leaving.get(&(vb, vc)) else {
            return limit;
        };
        // the face between the two edges, round from the corner before this vertex
        let c = self.mesh.corners(f as usize);
        let n = c.len();
        let round = |j: usize| c[(at as usize + n - 1 + j) % n];
        let vd = round(3);
        let ec = self.corners.get(&vc).map(|c| {
            let other = c.half_to(vb);
            &c.edges[(other + c.edges.len() - 1) % c.edges.len()]
        });
        let kc = ec.map(|e| k(e.bev)).unwrap_or(0.0);
        if va == vc || vd == vb {
            return limit;
        }
        let (th1, th2) = (self.angle3(va, vb, vc), self.angle3(vb, vc, vd));
        let (sin1, sin2, cos1, cos2) = (sinf(th1), sinf(th2), cosf(th1), cosf(th2));
        let safe = |a: f32, b: f32| if b == 0.0 { 0.0 } else { a / b };
        // where the edge closes up: its two ends' cuts meeting on it
        let on_b = safe(ka + cos1 * kb, sin1) + safe(kc + cos2 * kb, sin2);
        if on_b > EPSILON {
            let room = offset * (len_v3(self.co(vb) - self.co(vc)) / on_b);
            if room > EPSILON {
                limit = room;
            }
        }
        // where a point sliding along an edge left as it is reaches that edge's end (on along the edges that
        // run straight on from it)
        if kb > f32::EPSILON && (ka == 0.0 || kc == 0.0) {
            let kb = offset / kb;
            let pi = std::f32::consts::PI;
            let side = |j: usize| len_v3(self.co(round(j)) - self.co(round(j + 1)));
            if ka == 0.0 && round(0) == va {
                // (back from the edge before the vertex: corner n - k is k corners before it)
                let (mut back, mut exterior) = (0, pi - th1);
                let mut slide = side(0) * sinf(exterior);
                while exterior < 0.0001 && back < n {
                    back += 1;
                    let j = n - back;
                    exterior += pi - self.angle3(round(j), round(j + 1), round(j + 2));
                    slide += side(j) * sinf(exterior);
                }
                limit = (slide * kb).min(limit);
            }
            if kc == 0.0 {
                let (mut j, mut exterior) = (2, pi - th2);
                let mut slide = side(j) * sinf(exterior);
                while exterior < 0.0001 && j < n + 2 {
                    j += 1;
                    exterior += pi - self.angle3(round(j - 1), round(j), round(j + 1));
                    slide += side(j) * sinf(exterior);
                }
                limit = (slide * kb).min(limit);
            }
        }
        limit
    }
    /// Where the offset lines of two bevelled edges of a vertex meet, on the face (or faces) between them.
    fn offset_meet(
        &self,
        corner: &Corner,
        i1: usize,
        i2: usize,
        face: Option<u32>,
        between: bool,
    ) -> V3 {
        let n = corner.edges.len();
        let (e1, e2) = (&corner.edges[i1], &corner.edges[i2]);
        let v = self.co(corner.v);
        let dir1 = v - self.co(e1.to);
        let dir2 = self.co(e2.to) - v;
        let ang = angle(dir1, dir2);
        let face_no = |f: Option<u32>| match f {
            Some(f) => self.normals[f as usize],
            None => self.vertex_normals[corner.v as usize],
        };
        if ang < EPSILON_ANG {
            // one line: square to both, from the vertex
            let norm_v = face_no(face);
            let (perp, _) = normalize_v3((dir1 + dir2).cross(norm_v));
            let d = e1.offset_r.max(e2.offset_l) / cosf(ang / 2.0);
            return madd(v, perp, d);
        }
        if (ang - std::f32::consts::PI).abs() < EPSILON_ANG {
            // back along itself: on the line, the offset away
            let d = e1.offset_r.max(e2.offset_l);
            let (dir, _) = normalize_v3(self.co(e2.to) - v);
            return madd(v, dir, d);
        }
        let (norm_v1, norm_v2);
        if let (Some(f), true) = (face, ang < SMALL_ANG) {
            norm_v1 = self.normals[f as usize];
            norm_v2 = norm_v1;
        } else if !between {
            let (mut nv, _) = normalize_v3(dir2.cross(dir1));
            if dot_v3(nv, face_no(face)) < 0.0 {
                nv = -nv;
            }
            norm_v1 = nv;
            norm_v2 = nv;
        } else {
            // other faces between: each edge's own face at the corner
            let e1next = &corner.edges[(i1 + 1) % n];
            let e2prev = &corner.edges[(i2 + n - 1) % n];
            let dir1n = self.co(e1next.to) - v;
            let dir2p = v - self.co(e2prev.to);
            let (mut n1, _) = normalize_v3(dir1n.cross(dir1));
            if dot_v3(n1, self.normals[e1.fnext as usize]) < 0.0 {
                n1 = -n1;
            }
            let (mut n2, _) = normalize_v3(dir2.cross(dir2p));
            if dot_v3(n2, self.normals[e2prev.fnext as usize]) < 0.0 {
                n2 = -n2;
            }
            norm_v1 = n1;
            norm_v2 = n2;
        }
        // square to each edge in its face, pointing into the face
        let (perp1, _) = normalize_v3(dir1.cross(norm_v1));
        let (perp2, _) = normalize_v3(dir2.cross(norm_v2));
        let off1a = madd(v, perp1, e1.offset_r);
        let off1b = off1a + dir1;
        let off2a = madd(v, perp2, e2.offset_l);
        let off2b = off2a + dir2;
        match isect_line_line(off1a, off1b, off2a, off2b) {
            (0, _, _) => off1a,
            // (lines that pass each other, with other faces between: half way between their nearest points)
            (2, a, b) if between => mid(a, b),
            (_, a, _) => a,
        }
    }

    /// Where the offset line of the bevelled edge `bev` meets the edge `mid` left as it is (`first`: the bevelled
    /// one comes before it round the vertex), if it does, and the angle between the two.
    fn offset_meet_edge(
        &self,
        corner: &Corner,
        bev: usize,
        mid: usize,
        first: bool,
    ) -> Option<(V3, f32)> {
        let v = self.co(corner.v);
        let (e1, e2) = if first { (bev, mid) } else { (mid, bev) };
        let (dir1, _) = normalize_v3(self.co(corner.edges[e1].to) - v);
        let (dir2, _) = normalize_v3(self.co(corner.edges[e2].to) - v);
        let ang = angle_normalized(dir1, dir2);
        if ang.abs() < GOOD_ANGLE {
            return None;
        }
        let fno = dir1.cross(dir2);
        if dot_v3(fno, self.vertex_normals[corner.v as usize]) < 0.0 {
            return None; // a reflex angle
        }
        if (ang - std::f32::consts::PI).abs() < GOOD_ANGLE {
            return None;
        }
        let sinang = sinf(ang);
        Some(if first {
            (madd(v, dir2, corner.edges[bev].offset_r / sinang), ang)
        } else {
            (madd(v, dir1, corner.edges[bev].offset_l / sinang), ang)
        })
    }

    /// The new points round a vertex, one between each bevelled edge and the next: made, or (`again`) moved to
    /// where the edges' offsets now put them.
    fn build_boundary(&self, corner: &mut Corner, again: bool) {
        let n = corner.edges.len();
        if !again {
            corner.bounds.clear();
        }
        let mut e = 0;
        let mut at = 0;
        loop {
            // the edges left as they are up to the next bevelled one: between two faces of one plane, or not
            // (the last of each kind stands for it)
            let mut e2 = (e + 1) % n;
            let (mut in_plane, mut not_in_plane, mut eip, mut enip) = (0, 0, None, None);
            while !corner.edges[e2].bev {
                let before = self.normals[corner.edges[(e2 + n - 1) % n].fnext as usize];
                let after = self.normals[corner.edges[e2].fnext as usize];
                let dot = dot_v3(before, after);
                if (dot + 1.0).abs() <= EPSILON_BIG || (dot - 1.0).abs() <= EPSILON_BIG {
                    in_plane += 1;
                    eip = Some(e2);
                } else {
                    not_in_plane += 1;
                    enip = Some(e2);
                }
                e2 = (e2 + 1) % n;
            }
            let face = corner.edges[e].fnext;
            // a single edge between them takes the point, slid along it, if both offset lines reach it
            let on_edge = |mid_edge: Option<usize>| {
                let m = mid_edge?;
                let (a, ang1) = self.offset_meet_edge(corner, e, m, true)?;
                let (b, ang2) = self.offset_meet_edge(corner, e2, m, false)?;
                let ratio = if ang1 == 0.0 {
                    1.0
                } else {
                    sinf(ang2) / sinf(ang1)
                };
                Some((mid(a, b), m, ratio))
            };
            let slid = if in_plane == 0 && not_in_plane == 0 {
                None
            } else if not_in_plane > 0 {
                on_edge(enip.filter(|_| not_in_plane == 1))
            } else {
                on_edge(eip.filter(|_| in_plane == 1))
            };
            let co = match slid {
                Some((co, _, _)) => co,
                None if in_plane == 0 && not_in_plane == 0 => {
                    self.offset_meet(corner, e, e2, Some(face), false)
                }
                None if not_in_plane > 0 => self.offset_meet(corner, e, e2, None, true),
                None => self.offset_meet(corner, e, e2, Some(face), true),
            };
            if again {
                corner.bounds[at].co = co;
            } else {
                corner.bounds.push(Bound {
                    co,
                    efirst: e,
                    ebev: e2,
                    eon: slid.map(|s| s.1),
                    sinratio: slid.map(|s| s.2).unwrap_or(1.0),
                    face,
                    visited: false,
                    chain: None,
                });
                corner.edges[e].right = at;
                corner.edges[e2].left = at;
                let mut between = (e + 1) % n;
                while between != e2 {
                    (corner.edges[between].left, corner.edges[between].right) = (at, at);
                    between = (between + 1) % n;
                }
            }
            at += 1;
            e = e2;
            if e == 0 {
                break;
            }
        }
    }
}

/// Evens the widths along each run of new points that slid along an edge: where a point slides, the cut on
/// one side of it is tied to the cut on the other, so the offsets asked for can't all be met; each run (a chain
/// between two points that didn't slide, or a ring) is given the offsets that fit together.
fn adjust_offsets(corners: &mut BTreeMap<u32, Corner>) {
    let keys: Vec<u32> = corners.keys().copied().collect();
    // the point at the far end of bevelled edge `half` of `v`, on the same side of it
    let other = |corners: &BTreeMap<u32, Corner>,
                 v: u32,
                 half: usize,
                 left: bool|
     -> Option<(u32, usize)> {
        let to = corners[&v].edges[half].to;
        let c = corners.get(&to)?;
        let h = &c.edges[c.half_to(v)];
        Some((to, if left { h.left } else { h.right }))
    };
    for &bv in &keys {
        for anchor in 0..corners[&bv].bounds.len() {
            let b = &corners[&bv].bounds[anchor];
            if b.visited || b.eon.is_none() {
                continue;
            }
            // along the edges after each point, then back along those before the first
            let mut v = (bv, anchor);
            let (mut start, mut end) = (v, v);
            let (mut ring, mut len) = (false, 1);
            loop {
                let b = &corners[&v.0].bounds[v.1];
                if b.eon.is_none() || b.visited || ring {
                    break;
                }
                let efirst = b.efirst;
                corners.get_mut(&v.0).expect("a corner").bounds[v.1].visited = true;
                let Some(next) = other(corners, v.0, efirst, true) else {
                    break;
                };
                corners.get_mut(&v.0).expect("a corner").bounds[v.1].chain = Some(next);
                end = next;
                len += 1;
                if corners[&next.0].bounds[next.1].visited {
                    if next != start {
                        break;
                    }
                    adjust_run(corners, start, true);
                    ring = true;
                }
                v = next;
            }
            if !ring {
                corners.get_mut(&v.0).expect("a corner").bounds[v.1].chain = None;
                v = start;
                loop {
                    corners.get_mut(&v.0).expect("a corner").bounds[v.1].visited = true;
                    let elast = corners[&v.0].bounds[v.1].ebev;
                    let Some(next) = other(corners, v.0, elast, false) else {
                        break;
                    };
                    corners.get_mut(&next.0).expect("a corner").bounds[next.1].chain = Some(v);
                    start = next;
                    len += 1;
                    v = next;
                    let b = &corners[&v.0].bounds[v.1];
                    if b.visited || b.eon.is_none() {
                        break;
                    }
                }
                let slid = |p: (u32, usize)| corners[&p.0].bounds[p.1].eon.is_some();
                if len >= 3 && !slid(start) && !slid(end) {
                    adjust_run(corners, start, false);
                }
            }
        }
    }
}

/// One run's offsets: the least squares of each edge's two ends differing in width and, a fifth as heavy, of
/// each offset differing from what it was.
fn adjust_run(corners: &mut BTreeMap<u32, Corner>, start: (u32, usize), ring: bool) {
    let mut run = vec![start];
    while let Some(next) = corners[&run[run.len() - 1].0].bounds[run[run.len() - 1].1].chain {
        if next == start {
            break;
        }
        run.push(next);
    }
    let np = run.len();
    let bound = |p: (u32, usize)| &corners[&p.0].bounds[p.1];
    let ratio: Vec<f64> = run.iter().map(|&p| bound(p).sinratio as f64).collect();
    let right = |p: (u32, usize)| corners[&p.0].edges[bound(p).efirst].offset_r as f64;
    let left = |p: (u32, usize)| corners[&p.0].edges[bound(p).ebev].offset_l as f64;
    let rows = if ring { 3 * np } else { 3 * np - 3 };
    let mut a = vec![vec![0.0_f64; np]; rows];
    let mut rhs = vec![0.0_f64; rows];
    let weight = 0.2_f64;
    for i in 0..np {
        if ring || i < np - 1 {
            a[i][i] += 1.0;
            if ring {
                a[if i > 0 { i - 1 } else { np - 1 }][i] += -ratio[i];
            } else if i > 0 {
                a[i - 1][i] += -ratio[i];
            }
            let row = if ring { np + 2 * i } else { np - 1 + 2 * i };
            a[row][i] += weight;
            rhs[row] += weight * right(run[i]);
            let next = if i == np - 1 { 0 } else { i + 1 };
            a[row + 1][next] += weight * ratio[next];
            rhs[row + 1] += weight * left(run[next]);
        } else {
            a[i - 1][i] += -1.0;
        }
    }
    // the normal equations, by elimination
    let mut m = vec![vec![0.0_f64; np + 1]; np];
    for r in 0..rows {
        for i in 0..np {
            if a[r][i] == 0.0 {
                continue;
            }
            for j in 0..np {
                m[i][j] += a[r][i] * a[r][j];
            }
            m[i][np] += a[r][i] * rhs[r];
        }
    }
    for c in 0..np {
        let pivot = (c..np)
            .max_by(|&x, &y| m[x][c].abs().total_cmp(&m[y][c].abs()))
            .expect("a row");
        if m[pivot][c].abs() < 1e-300 {
            return;
        }
        m.swap(c, pivot);
        for r in 0..np {
            if r != c && m[r][c] != 0.0 {
                let k = m[r][c] / m[c][c];
                for j in c..=np {
                    m[r][j] -= k * m[c][j];
                }
            }
        }
    }
    let x: Vec<f64> = (0..np).map(|i| m[i][np] / m[i][i]).collect();
    let ends: Vec<(usize, usize, f32)> = run
        .iter()
        .map(|&p| (bound(p).efirst, bound(p).ebev, bound(p).sinratio))
        .collect();
    for (i, &p) in run.iter().enumerate() {
        let (efirst, elast, r) = ends[i];
        let corner = corners.get_mut(&p.0).expect("a corner");
        if ring || i < np - 1 {
            corner.edges[efirst].offset_r = x[i] as f32;
            if ring || i > 0 {
                corner.edges[elast].offset_l = (r as f64 * x[i]) as f32;
            }
        } else {
            corner.edges[elast].offset_l = x[i] as f32;
        }
    }
}

/// A round profile from `start` to `end` about `middle`, its points square to the edge `along` on the plane
/// through `start` with the normal `plane_no`: the points 1 to `ns` - 1.
fn profile(start: V3, middle: V3, end: V3, along: V3, plane_no: V3, ns: usize) -> Vec<V3> {
    let (m0, m1) = (middle - start, middle - end);
    let m3 = start + end - middle;
    let step = (PI / 2.0) / ns as f64;
    (1..ns)
        .map(|k| {
            let (x, y) = (
                libm::sin(k as f64 * step) as f32,
                libm::cos(k as f64 * step) as f32,
            );
            let co = V3::new(
                x * m0.x + y * m1.x + m3.x,
                x * m0.y + y * m1.y + m3.y,
                x * m0.z + y * m1.z + m3.z,
            );
            // onto the profile's plane, along the edge
            let h = co - start;
            let dot = dot_v3(plane_no, along);
            if dot.abs() > f32::EPSILON {
                madd(co, along, -dot_v3(plane_no, h) / dot)
            } else {
                co
            }
        })
        .collect()
}

/// A corner patch of a cube of side one, `ns` segments: its points by (bound, ring, step), on the unit sphere.
fn cube_corner(ns: usize) -> Result<BTreeMap<(usize, usize, usize), V3>, String> {
    let unit = |i: usize, a: f32, b: f32, c: f32| {
        // (a along the bound's own axis, b along the next, c along the third)
        let mut p = [0.0_f32; 3];
        (p[i], p[(i + 1) % 3], p[(i + 2) % 3]) = (a, b, c);
        normalize_v3(V3::new(p[0], p[1], p[2])).0
    };
    let mut out = BTreeMap::new();
    let table: &[(usize, usize, [f32; 3])] = match ns {
        2 => &[(0, 1, [0.707_106_77, 0.707_106_77, 0.0])],
        3 => CORNER3,
        4 => CORNER4,
        _ => return Err(format!("a corner patch of {ns} segments isn't built")),
    };
    for i in 0..3 {
        for &(j, k, [a, b, c]) in table {
            out.insert((i, j, k), unit(i, a, b, c));
        }
    }
    if ns.is_multiple_of(2) {
        let s = 0.577_350_26_f32;
        out.insert((0, ns / 2, ns / 2), normalize_v3(V3::new(s, s, s)).0);
    }
    Ok(out)
}

/// The patch of three segments, as Blender's subdivision leaves it on the sphere (measured from its output).
const CORNER3: &[(usize, usize, [f32; 3])] = &[
    (0, 1, [0.866_403_8, 0.499_344, 0.0]),
    (1, 0, [0.866_403_8, 0.0, 0.499_344]),
    (1, 1, [0.736_632_6, 0.478_211_55, 0.478_211_55]),
];
/// The patch of four segments.
const CORNER4: &[(usize, usize, [f32; 3])] = &[
    (0, 1, [0.923_879_5, 0.382_683_43, 0.0]),
    (0, 2, [0.707_106_77, 0.707_106_77, 0.0]),
    (1, 0, [0.923_879_5, 0.0, 0.382_683_43]),
    (1, 1, [0.832_011_1, 0.392_273_8, 0.392_273_8]),
    (1, 2, [0.656_254_55, 0.656_254_55, 0.372_370_53]),
];

/// `mesh` with its edges that turn more than `angle` degrees bevelled `width` wide in `segments` strips.
pub fn bevelled(
    mesh: &Mesh,
    width: f64,
    segments: usize,
    angle: f64,
    order: &EdgeOrder,
) -> Result<Bevelled, String> {
    let ns = segments.max(1);
    let normals = face_normals(mesh);
    let nf = mesh.faces.len();
    let mut leaving = BTreeMap::new();
    for f in 0..nf {
        let c = mesh.corners(f);
        for (i, &a) in c.iter().enumerate() {
            let b = c[(i + 1) % c.len()];
            if leaving.insert((a, b), (f as u32, i as u32)).is_some() {
                return Err(
                    "an edge with two faces running it the same way can't be bevelled".into(),
                );
            }
        }
    }
    // the edges in the order the mesh numbers them
    let mut rank: BTreeMap<(u32, u32), usize> = BTreeMap::new();
    // (and the way each runs)
    let mut runs: BTreeMap<(u32, u32), (u32, u32)> = BTreeMap::new();
    let mut number = |a: u32, b: u32, way: (u32, u32)| {
        let at = rank.len();
        rank.entry(edge(a, b)).or_insert(at);
        runs.entry(edge(a, b)).or_insert(way);
    };
    if let EdgeOrder::Listed(list) = order {
        list.iter().for_each(|&(a, b)| number(a, b, (a, b)));
    }
    for f in 0..nf {
        let c = mesh.corners(f);
        let n = c.len();
        match order {
            EdgeOrder::Faces => (0..n).for_each(|i| {
                let (a, b) = (c[(i + n - 1) % n], c[i]);
                number(a, b, (a, b))
            }),
            _ => (0..n).for_each(|i| {
                let (a, b) = (c[i], c[(i + 1) % n]);
                number(a, b, edge(a, b))
            }),
        }
    }
    // the edges to bevel: between two faces that turn more than the angle
    let limit = crate::math::radians(angle) as f32;
    let threshold = cosf(limit + 0.000_000_175);
    let mut bev: BTreeSet<(u32, u32)> = BTreeSet::new();
    for (&(a, b), &(f, _)) in &leaving {
        if a < b
            && let Some(&(g, _)) = leaving.get(&(b, a))
            && dot_v3(normals[f as usize], normals[g as usize]) < threshold
        {
            bev.insert((a, b));
        }
    }
    let offset = width as f32;
    let mut work = Work {
        mesh,
        vertex_normals: crate::normals::vertex_normals(mesh),
        normals,
        leaving,
        corners: BTreeMap::new(),
    };
    if bev.is_empty() || offset <= 0.0 {
        return Ok(Bevelled {
            mesh: mesh.clone(),
            kinds: vec![Kind::Kept; nf],
            source: (0..nf)
                .flat_map(|f| std::iter::repeat_n(f as u32, mesh.faces[f].len as usize))
                .collect(),
            picks: Vec::new(),
            segments,
            offset: 0.0,
        });
    }
    // each bevelled vertex's edges, anticlockwise from the first bevelled one the mesh numbers
    let mut tagged: BTreeSet<u32> = BTreeSet::new();
    for &(a, b) in &bev {
        tagged.insert(a);
        tagged.insert(b);
    }
    for &v in &tagged {
        let first = bev
            .iter()
            .filter(|e| e.0 == v || e.1 == v)
            .min_by_key(|e| rank[e])
            .map(|e| if e.0 == v { e.1 } else { e.0 })
            .expect("a bevelled edge");
        let mut edges = Vec::new();
        let mut to = first;
        loop {
            let Some(&(f, i)) = work.leaving.get(&(v, to)) else {
                return Err("an open edge can't be bevelled".into());
            };
            edges.push(Half {
                to,
                bev: bev.contains(&edge(v, to)),
                fnext: f,
                offset_l: 0.0,
                offset_r: 0.0,
                left: 0,
                right: 0,
            });
            let c = mesh.corners(f as usize);
            to = c[(i as usize + c.len() - 1) % c.len()];
            if to == first {
                break;
            }
            if edges.len() > 4 * mesh.loops.len() {
                return Err("a vertex whose faces don't close round it can't be bevelled".into());
            }
        }
        if edges.iter().filter(|h| h.bev).count() < 2 {
            return Err(
                "a bevel that ends at a vertex no other bevelled edge reaches isn't built".into(),
            );
        }
        // Blender lists the vertex's edges from that one on through whichever of its two neighbours the mesh
        // numbers first, and turns the list over if that went clockwise, which leaves that edge last; the new
        // points start from the list's last edge if it is bevelled, from its first if not. So they start from
        // that edge where the list was turned over, and from the edge before it, if bevelled, where it was not
        let n = edges.len();
        if n > 2 {
            let number = |h: &Half| rank[&edge(v, h.to)];
            if number(&edges[1]) < number(&edges[n - 1]) && edges[n - 1].bev {
                edges.rotate_right(1);
            }
        }
        work.corners.insert(
            v,
            Corner {
                v,
                edges,
                ..Corner::default()
            },
        );
    }
    // no further than the mesh has room for
    let mut limited = offset;
    for corner in work.corners.values() {
        for i in 0..corner.edges.len() {
            limited = limited.min(work.collide_offset(corner, i, offset));
        }
    }
    let factor = if limited < offset {
        limited / offset
    } else {
        1.0
    };
    let mut each = offset;
    if limited < offset {
        each *= factor;
    }
    let keys: Vec<u32> = work.corners.keys().copied().collect();
    for again in [false, true] {
        for &v in &keys {
            let mut corner = work.corners.remove(&v).expect("a corner");
            if !again {
                for h in &mut corner.edges {
                    (h.offset_l, h.offset_r) = if h.bev { (each, each) } else { (0.0, 0.0) };
                }
            }
            work.build_boundary(&mut corner, again);
            work.corners.insert(v, corner);
        }
        if !again {
            adjust_offsets(&mut work.corners);
        }
    }

    // ---- the new mesh ----
    let mut out = Mesh {
        mats: mesh.mats.clone(),
        ..Mesh::default()
    };
    let mut kept = vec![u32::MAX; mesh.verts.len()];
    for (i, &p) in mesh.verts.iter().enumerate() {
        if !tagged.contains(&(i as u32)) {
            kept[i] = out.verts.len() as u32;
            out.verts.push(p);
        }
    }
    let mut kinds = Vec::new();
    let mut source = Vec::new();
    let mut long: BTreeSet<(u32, u32)> = BTreeSet::new(); // (face, corner) whose side runs along a bevelled edge
    // (corners, each one's source face, material, the vertex of a corner's polygon)
    let mut corner_faces: Vec<CornerFace> = Vec::new();
    let mut picks = Vec::new();
    let mut grids: BTreeMap<u32, Numbers> = BTreeMap::new();
    for &v in &keys {
        let corner = &work.corners[&v];
        let mut grid = Numbers::new();
        let n = corner.bounds.len();
        for (i, bound) in corner.bounds.iter().enumerate() {
            grid.insert((i, 0, 0), out.verts.len() as u32);
            out.verts.push(bound.co);
        }
        for i in 0..n {
            let next = grid[&((i + 1) % n, 0, 0)];
            grid.insert((i, 0, ns), next);
        }
        let vco = work.co(v);
        let mat = mesh.faces[corner.bounds[0].face as usize].mat;
        if ns > 1 && n == 2 {
            // two bevelled edges: one profile between the two points, in the plane they span with the vertex
            let (d1, d2) = (vco - corner.bounds[0].co, vco - corner.bounds[1].co);
            let (no, l1) = normalize_v3(d1.cross(d2));
            let plane = (l1 > EPSILON).then_some(no);
            let (a, b) = (
                work.profile_of(corner, 0, plane, ns),
                work.profile_of(corner, 1, plane, ns),
            );
            for k in 1..ns {
                let (p, q) = (a[k - 1], b[ns - k - 1]);
                let at = out.verts.len() as u32;
                out.verts.push(V3::new(
                    0.5 * (p.x + q.x),
                    0.5 * (p.y + q.y),
                    0.5 * (p.z + q.z),
                ));
                grid.insert((0, 0, k), at);
                grid.insert((1, 0, ns - k), at);
            }
        } else if ns > 1 {
            let patch = work.patch(corner, ns)?;
            let ns2 = ns / 2;
            for i in 0..n {
                for j in 0..=ns2 {
                    for k in 0..=ns {
                        if (j == 0 && (k == 0 || k == ns)) || canon(n, ns, i, j, k) != (i, j, k) {
                            continue;
                        }
                        grid.insert((i, j, k), out.verts.len() as u32);
                        out.verts.push(patch[&(i, j, k)]);
                    }
                }
            }
            // (each profile's far half is the next bound's near side)
            for i in 0..n {
                for k in 1..ns {
                    let at = grid[&canon(n, ns, i, 0, k)];
                    grid.insert((i, 0, k), at);
                }
            }
            let at = |i: usize, j: usize, k: usize| {
                grid.get(&(i, j, k))
                    .or_else(|| grid.get(&canon(n, ns, i, j, k)))
                    .copied()
                    .expect("a patch point")
            };
            let odd = ns % 2;
            for i in 0..n {
                let (f1, f2) = (corner.bounds[i].face, corner.bounds[(i + 1) % n].face);
                for j in 0..ns2 {
                    for k in 0..ns2 + odd {
                        let quad = vec![
                            at(i, j, k),
                            at(i, j, k + 1),
                            at(i, j + 1, k + 1),
                            at(i, j + 1, k),
                        ];
                        let f = if 2 * k + 1 < ns { f1 } else { f2 };
                        corner_faces.push((quad, vec![f; 4], mat, None, Some((v, i, j, k))));
                    }
                }
            }
            if odd == 1 {
                let ring: Vec<u32> = (0..n).map(|i| at(i, ns2, ns2)).collect();
                let from: Vec<u32> = (0..n).map(|i| corner.bounds[i].face).collect();
                corner_faces.push((ring, from, mat, Some(v), None));
            }
        } else if n >= 3 {
            let ring: Vec<u32> = (0..n).map(|i| grid[&(i, 0, 0)]).collect();
            let from: Vec<u32> = (0..n).map(|i| corner.bounds[i].face).collect();
            corner_faces.push((ring, from, mat, Some(v), None));
        }
        grids.insert(v, grid);
    }
    // the faces left as they were
    for f in 0..nf {
        if mesh.corners(f).iter().all(|v| !tagged.contains(v)) {
            let c: Vec<u32> = mesh.corners(f).iter().map(|&v| kept[v as usize]).collect();
            out.push_face(&c, mesh.faces[f].mat, mesh.faces[f].smooth);
            kinds.push(Kind::Kept);
            source.extend(std::iter::repeat_n(f as u32, c.len()));
        }
    }
    // each new point's bevelled edge across, with the faces either side of it
    let across = |vert: u32| -> Vec<(u32, u32, u32)> {
        let corner = &work.corners[&vert];
        let (halves, n) = (&corner.edges, corner.edges.len());
        corner
            .bounds
            .iter()
            .map(|b| {
                let h = b.ebev;
                (halves[h].to, halves[(h + n - 1) % n].fnext, halves[h].fnext)
            })
            .collect()
    };
    for (c, from, mat, vert, patch) in corner_faces {
        if let Some(vert) = vert {
            let halves = &work.corners[&vert].edges;
            let n = halves.len();
            picks.push(Pick::Corner {
                at: out.loops.len(),
                vert,
                on: from.clone(),
                edges: (0..n)
                    .filter(|&i| halves[i].bev)
                    .map(|i| (halves[i].to, halves[(i + n - 1) % n].fnext, halves[i].fnext))
                    .collect(),
                across: if ns > 1 { across(vert) } else { Vec::new() },
            });
        }
        if let Some((vert, i, j, k)) = patch {
            picks.push(Pick::Patch {
                at: out.loops.len(),
                vert,
                place: (i, j, k),
                across: across(vert),
            });
        }
        out.push_face(&c, mat, true);
        kinds.push(Kind::Corner);
        source.extend(from);
    }
    // a strip along each bevelled edge, in the mesh's order of them
    let mut by_rank: Vec<(u32, u32)> = bev.iter().copied().collect();
    by_rank.sort_by_key(|e| rank[e]);
    for e in by_rank {
        let (a, b) = runs[&e];
        let (c1, c2) = (&work.corners[&a], &work.corners[&b]);
        let (h1, h2) = (&c1.edges[c1.half_to(b)], &c2.edges[c2.half_to(a)]);
        let (i1, i2) = (h1.left, h2.left);
        // (the faces before and after the edge round its first end: its left point is on the one before)
        let at = c1.half_to(b);
        let (f1, f2) = (
            c1.edges[(at + c1.edges.len() - 1) % c1.edges.len()].fnext,
            h1.fnext,
        );
        let mat = mesh.faces[c1.bounds[h1.left].face as usize].mat;
        let at1 = |k: usize| grids[&a][&(i1, 0, k)];
        let at2 = |k: usize| grids[&b][&(i2, 0, k)];
        for k in 1..=ns {
            let quad = [at1(k - 1), at2(ns - k + 1), at2(ns - k), at1(k)];
            let face = out.faces.len() as u32;
            picks.push(Pick::Strip {
                at: out.loops.len(),
                edge: (a, b),
                faces: (f1, f2),
                k,
            });
            out.push_face(&quad, mat, true);
            kinds.push(Kind::Strip);
            long.insert((face, 0));
            long.insert((face, 2));
            source.extend([if 2 * k <= ns { f1 } else { f2 }; 4]);
        }
    }
    // the mesh's faces round the bevelled vertices, each corner now the new point on it
    let mut done = vec![false; nf];
    // (in the order Blender rebuilds them: vertex by vertex, each vertex's faces as its edges are numbered, a
    // face from the edge it leaves the vertex along; the shading's rays meet the faces in this order)
    for &v in &keys {
        let corner = &work.corners[&v];
        let mut round: Vec<&Half> = corner.edges.iter().collect();
        round.sort_by_key(|h| rank[&edge(v, h.to)]);
        for h in round {
            let f = h.fnext as usize;
            if std::mem::replace(&mut done[f], true) {
                continue;
            }
            let c = mesh.corners(f);
            let mut new: Vec<u32> = Vec::with_capacity(c.len());
            for (i, &w) in c.iter().enumerate() {
                let p = match work.corners.get(&w) {
                    None => kept[w as usize],
                    Some(cw) => {
                        let leaves = &cw.edges[cw.half_to(c[(i + 1) % c.len()])];
                        grids[&w][&(leaves.right, 0, 0)]
                    }
                };
                if new.last() != Some(&p) {
                    new.push(p);
                }
            }
            if new.len() > 1 && new.first() == new.last() {
                new.pop();
            }
            if new.len() < 3 {
                continue;
            }
            source.extend(std::iter::repeat_n(f as u32, new.len()));
            out.push_face(&new, mesh.faces[f].mat, mesh.faces[f].smooth);
            kinds.push(Kind::Cut);
        }
    }
    harden(&mut out, &kinds, &long, mesh, &kept);
    Ok(Bevelled {
        mesh: out,
        kinds,
        source,
        picks,
        segments: ns,
        offset: each,
    })
}

impl Work<'_> {
    /// The profile from a vertex's new point `i` to the next, across the bevelled edge between them: its points
    /// between the two, on `plane` (a normal) if given.
    fn profile_of(&self, corner: &Corner, i: usize, plane: Option<V3>, ns: usize) -> Vec<V3> {
        let n = corner.bounds.len();
        let (start, end) = (corner.bounds[i].co, corner.bounds[(i + 1) % n].co);
        let to = corner.edges[corner.bounds[i].ebev].to;
        let (vco, other) = (self.co(corner.v), self.co(to));
        // the point of the edge nearest the line between the two
        let (lo, hi) = if corner.v < to {
            (vco, other)
        } else {
            (other, vco)
        };
        let middle = match isect_line_line(lo, hi, start, end) {
            (0, _, _) => lo,
            (_, on, _) => on,
        };
        let (along, _) = normalize_v3(vco - other);
        let plane_no = plane.unwrap_or_else(|| {
            let (d1, _) = normalize_v3(middle - start);
            let (d2, _) = normalize_v3(middle - end);
            normalize_v3(d1.cross(d2)).0
        });
        profile(start, middle, end, along, plane_no, ns)
    }

    /// Whether a corner of three bevelled edges is near enough a cube's to take a cube corner's patch.
    fn cube_like(&self, corner: &Corner) -> bool {
        if corner.bounds.len() != 3 {
            return false;
        }
        let v = corner.v;
        let mut total = 0.0_f32;
        let mut in_plane = 0;
        for h in &corner.edges {
            let (&(f, i), &(g, _)) = (&self.leaving[&(v, h.to)], &self.leaving[&(h.to, v)]);
            let (n1, n2) = (self.normals[f as usize], self.normals[g as usize]);
            let mut ang = angle_normalized(n1, n2);
            if n1 != n2 {
                let c = self.mesh.corners(f as usize);
                let dir = self.co(c[(i as usize + 1) % c.len()]) - self.co(c[i as usize]);
                if dot_v3(dir, n1.cross(n2)) <= 0.0 {
                    ang = -ang;
                }
            }
            let abs = ang.abs();
            // (against a quarter of pi in double precision, as Blender has it: just under the single)
            if abs as f64 <= std::f64::consts::FRAC_PI_4 {
                in_plane += 1;
            } else if abs >= 3.0 * std::f32::consts::FRAC_PI_4 {
                return false;
            }
            // (a cube corner's patch is for edges all cut back alike)
            if h.bev && (h.offset_l - corner.edges[0].offset_l).abs() > EPSILON {
                return false;
            }
            total += ang;
        }
        if in_plane != corner.edges.len() - 3 {
            return false;
        }
        // (with other edges lying in its faces too, a slab's corner where its face's triangles meet: Blender
        // tells such a corner apart, and gives it the cube corner's patch all the same)
        let off = (total.abs() - 3.0 * std::f32::consts::FRAC_PI_2).abs();
        off <= std::f32::consts::FRAC_PI_4
    }

    /// The points of the patch where three or more bevelled edges meet, by (bound, ring, step).
    fn patch(
        &self,
        corner: &Corner,
        ns: usize,
    ) -> Result<BTreeMap<(usize, usize, usize), V3>, String> {
        let n = corner.bounds.len();
        let v = self.co(corner.v);
        let mut out = BTreeMap::new();
        if self.cube_like(corner) {
            // a cube corner's patch, its three axes laid onto the three new points and the vertex
            let (va, vb, vc) = (
                corner.bounds[0].co,
                corner.bounds[1].co,
                corner.bounds[2].co,
            );
            let half = |p: V3, q: V3, r: V3| mul_v3_fl(p - q - r + v, 0.5);
            let (m0, m1, m2) = (half(va, vb, vc), half(vb, va, vc), half(vc, va, vb));
            let m3 = mul_v3_fl(va + vb + vc - v, 0.5);
            for (key, p) in cube_corner(ns)? {
                out.insert(
                    key,
                    V3::new(
                        p.x * m0.x + p.y * m1.x + m2.x * p.z + m3.x,
                        p.x * m0.y + p.y * m1.y + m2.y * p.z + m3.y,
                        p.x * m0.z + p.y * m1.z + m2.z * p.z + m3.z,
                    ),
                );
            }
            return Ok(out);
        }
        // any other corner: a patch of two segments (each side's middle on its profile, one point in the middle,
        // towards the vertex by the profile's fullness), subdivided, and read off at the segments asked for
        if ns > 4 {
            return Err(format!(
                "a patch of {ns} segments where {n} bevelled edges meet isn't built"
            ));
        }
        let mut vm = Grid::new(n, 2);
        let mut middle = V3::ZERO;
        for i in 0..n {
            *vm.raw(i, 0, 0) = corner.bounds[i].co;
            *vm.raw(i, 0, 1) = self.profile_of(corner, i, None, 2)[0];
            middle += corner.bounds[i].co;
        }
        let middle = mul_v3_fl(middle, 1.0 / n as f32);
        let towards = v - middle;
        *vm.raw(0, 1, 1) = if dot_v3(towards, towards) > 1e-12 {
            madd(middle, towards, FULLNESS[ns - 1])
        } else {
            middle
        };
        vm.copy_equiv();
        loop {
            vm = self.subdivided(corner, vm);
            if vm.ns >= ns {
                break;
            }
        }
        if vm.ns != ns {
            vm = self.read_off(corner, &vm, ns);
        }
        let ns2 = ns / 2;
        for i in 0..n {
            for j in 0..=ns2 {
                for k in 0..=ns {
                    if canon(n, ns, i, j, k) == (i, j, k) {
                        out.insert((i, j, k), *vm.raw(i, j, k));
                    }
                }
            }
        }
        Ok(out)
    }

    /// One step of Blender's cubic subdivision of a patch (Catmull-Clark, its sides kept on their profiles).
    fn subdivided(&self, corner: &Corner, mut vm_in: Grid) -> Grid {
        let (n, ns_in) = (vm_in.n, vm_in.ns);
        let (ns_in2, ns_out) = (ns_in / 2, 2 * ns_in);
        let mut out = Grid::new(n, ns_out);
        let avg4 = |a: V3, b: V3, c: V3, d: V3| mul_v3_fl(a + b + c + d, 0.25);
        let smooth = |co: V3, co1: V3, co2: V3| {
            let acc = madd(co1 + co2, co, -2.0);
            madd(co, acc, -1.0 / 6.0)
        };
        // the sides: the points there, smoothed
        for i in 0..n {
            *out.raw(i, 0, 0) = *vm_in.raw(i, 0, 0);
            for k in 1..ns_in {
                let co = smooth(
                    *vm_in.raw(i, 0, k),
                    *vm_in.raw(i, 0, k - 1),
                    *vm_in.raw(i, 0, k + 1),
                );
                out.set(i, 0, 2 * k, co);
            }
        }
        // and the new ones between them, from the profile
        let profiles: Vec<Vec<V3>> = (0..n)
            .map(|i| self.profile_of(corner, i, None, ns_out))
            .collect();
        for (i, profile) in profiles.iter().enumerate() {
            for k in (1..ns_out).step_by(2) {
                let co = smooth(profile[k - 1], out.get(i, 0, k - 1), out.get(i, 0, k + 1));
                out.set(i, 0, k, co);
            }
        }
        out.copy_equiv();
        for i in 0..n {
            for k in 0..ns_in {
                *vm_in.raw(i, 0, k) = *out.raw(i, 0, 2 * k);
            }
        }
        vm_in.copy_equiv();
        // the new points in the faces
        for i in 0..n {
            for j in 0..ns_in2 {
                for k in 0..ns_in2 {
                    let co = avg4(
                        *vm_in.raw(i, j, k),
                        *vm_in.raw(i, j, k + 1),
                        *vm_in.raw(i, j + 1, k),
                        *vm_in.raw(i, j + 1, k + 1),
                    );
                    *out.raw(i, 2 * j + 1, 2 * k + 1) = co;
                }
            }
        }
        // on the edges across the rings
        for i in 0..n {
            for j in 0..ns_in2 {
                for k in 1..=ns_in2 {
                    let co = avg4(
                        *vm_in.raw(i, j, k),
                        *vm_in.raw(i, j + 1, k),
                        out.get(i, 2 * j + 1, 2 * k - 1),
                        out.get(i, 2 * j + 1, 2 * k + 1),
                    );
                    *out.raw(i, 2 * j + 1, 2 * k) = co;
                }
            }
        }
        // on the edges along them
        for i in 0..n {
            for j in 1..ns_in2 {
                for k in 0..ns_in2 {
                    let co = avg4(
                        *vm_in.raw(i, j, k),
                        *vm_in.raw(i, j, k + 1),
                        out.get(i, 2 * j - 1, 2 * k + 1),
                        out.get(i, 2 * j + 1, 2 * k + 1),
                    );
                    *out.raw(i, 2 * j, 2 * k + 1) = co;
                }
            }
        }
        // the old points inside
        let (gamma, beta) = (0.25_f32, -0.25_f32);
        for i in 0..n {
            for j in 1..ns_in2 {
                for k in 1..=ns_in2 {
                    let co1 = avg4(
                        out.get(i, 2 * j, 2 * k - 1),
                        out.get(i, 2 * j, 2 * k + 1),
                        out.get(i, 2 * j - 1, 2 * k),
                        out.get(i, 2 * j + 1, 2 * k),
                    );
                    let co2 = avg4(
                        out.get(i, 2 * j - 1, 2 * k - 1),
                        out.get(i, 2 * j + 1, 2 * k - 1),
                        out.get(i, 2 * j - 1, 2 * k + 1),
                        out.get(i, 2 * j + 1, 2 * k + 1),
                    );
                    let co = madd(madd(co1, co2, beta), *vm_in.raw(i, j, k), gamma);
                    *out.raw(i, 2 * j, 2 * k) = co;
                }
            }
        }
        out.copy_equiv();
        // the middle
        let gamma = sabin_gamma(n);
        let beta = -gamma;
        let (mut co1, mut co2) = (V3::ZERO, V3::ZERO);
        for i in 0..n {
            co1 += *out.raw(i, ns_in, ns_in - 1);
            co2 += *out.raw(i, ns_in - 1, ns_in - 1);
            co2 += *out.raw(i, ns_in - 1, ns_in + 1);
        }
        let mut co = mul_v3_fl(co1, 1.0 / n as f32);
        co = madd(co, co2, beta / (2.0 * n as f32));
        co = madd(co, *vm_in.raw(0, ns_in2, ns_in2), gamma);
        for i in 0..n {
            *out.raw(i, ns_in, ns_in) = co;
        }
        // the sides back on their profiles
        for (i, profile) in profiles.iter().enumerate() {
            let inext = (i + 1) % n;
            for k in 1..ns_out {
                let co = profile[k - 1];
                *out.raw(i, 0, k) = co;
                if k >= ns_in {
                    *out.raw(inext, ns_out - k, 0) = co;
                }
            }
            *out.raw(i, 0, ns_out) = *out.raw(inext, 0, 0);
        }
        out
    }

    /// A patch of `nseg` segments read off one of more: each point where its share of the way along the sides
    /// falls among the finer patch's.
    fn read_off(&self, corner: &Corner, vm_in: &Grid, nseg: usize) -> Grid {
        let (n, ns_in) = (vm_in.n, vm_in.ns);
        let (nseg2, odd) = (nseg / 2, nseg % 2);
        let mut out = Grid::new(n, nseg);
        // how far along a side each of its points is, 0 to 1
        let shares = |points: &[V3]| {
            let mut frac = vec![0.0_f32; points.len()];
            let mut total = 0.0_f32;
            for k in 1..points.len() {
                total += len_v3(points[k - 1] - points[k]);
                frac[k] = total;
            }
            let last = points.len() - 1;
            if total > 0.0 {
                frac[1..].iter_mut().for_each(|f| *f /= total);
            } else {
                frac[last] = 1.0;
            }
            frac
        };
        let side_in =
            |i: usize| shares(&(0..=ns_in).map(|k| vm_in.at(i, 0, k)).collect::<Vec<_>>());
        let side_new = |i: usize| {
            let mut points = vec![corner.bounds[i].co];
            points.extend(self.profile_of(corner, i, None, nseg));
            points.push(corner.bounds[(i + 1) % n].co);
            shares(&points)
        };
        // the finer point at or before a share, and the rest of the way to the next
        let range = |frac: &[f32], f: f32| -> (usize, f32) {
            let count = frac.len() - 1;
            for i in 0..count {
                if f <= frac[i + 1] {
                    let rest = f - frac[i];
                    let rest = if rest == 0.0 {
                        0.0
                    } else {
                        rest / (frac[i + 1] - frac[i])
                    };
                    if i == count - 1 && rest == 1.0 {
                        return (count, 0.0);
                    }
                    return (i, rest);
                }
            }
            (count, 0.0)
        };
        let mut prev_frac = side_in(n - 1);
        let mut prev_new_frac = side_new(n - 1);
        for i in 0..n {
            let frac = side_in(i);
            let new_frac = side_new(i);
            for j in 0..nseg2 + odd {
                for (k, &share) in new_frac.iter().enumerate().take(nseg2 + 1) {
                    let (k_in, restk) = range(&frac, share);
                    let (k_in_prev, restkprev) = range(&prev_frac, prev_new_frac[nseg - j]);
                    let mut j_in = ns_in - k_in_prev;
                    let mut restj = -restkprev;
                    if restj > -EPSILON {
                        restj = 0.0;
                    } else {
                        j_in -= 1;
                        restj += 1.0;
                    }
                    let co = if restj < EPSILON && restk < EPSILON {
                        vm_in.get(i, j_in, k_in)
                    } else {
                        let j0inc = if restj < EPSILON || j_in == ns_in {
                            0
                        } else {
                            1
                        };
                        let k0inc = if restk < EPSILON || k_in == ns_in {
                            0
                        } else {
                            1
                        };
                        let quad = [
                            vm_in.get(i, j_in, k_in),
                            vm_in.get(i, j_in, k_in + k0inc),
                            vm_in.get(i, j_in + j0inc, k_in + k0inc),
                            vm_in.get(i, j_in + j0inc, k_in),
                        ];
                        let (u, v) = (restj, restk);
                        let mut res = V3::ZERO;
                        res += mul_v3_fl(quad[0], (1.0 - u) * (1.0 - v));
                        res += mul_v3_fl(quad[1], (1.0 - u) * v);
                        res += mul_v3_fl(quad[2], u * v);
                        res += mul_v3_fl(quad[3], u * (1.0 - v));
                        res
                    };
                    *out.raw(i, j, k) = co;
                }
            }
            prev_frac = frac;
            prev_new_frac = new_frac;
        }
        if odd == 0 {
            *out.raw(0, nseg2, nseg2) = vm_in.at(0, ns_in / 2, ns_in / 2);
        }
        out.copy_equiv();
        out
    }
}

/// How much of its old place a patch's middle keeps when it is subdivided, by its sides (Sabin's weights).
fn sabin_gamma(n: usize) -> f32 {
    match n {
        0..=2 => 0.0,
        3 => 0.065_247_58,
        4 => 0.25,
        5 => 0.401_983_44,
        6 => 0.523_423_3,
        _ => {
            let k = libm::cos(PI / n as f64);
            let (k2, k4, k6) = (k * k, k * k * k * k, k * k * k * k * k * k);
            let y = libm::pow(
                3.0_f64.sqrt() * (64.0 * k6 - 144.0 * k4 + 135.0 * k2 - 27.0).sqrt() + 9.0 * k,
                1.0 / 3.0,
            );
            let x = 0.480_749_856_769_136 * y - (0.231_120_424_783_545 * (12.0 * k2 - 9.0)) / y;
            ((k * x + 2.0 * k2 - 1.0) / (x * x * (k * x + 1.0))) as f32
        }
    }
}

/// A patch's points as Blender keeps them: a block for each side by (ring, step), the points the blocks share
/// kept in one of them and copied to the others.
#[derive(Clone)]
struct Grid {
    n: usize,
    ns: usize,
    co: Vec<V3>,
}

impl Grid {
    fn new(n: usize, ns: usize) -> Grid {
        Grid {
            n,
            ns,
            co: vec![V3::ZERO; n * (ns / 2 + 1) * (ns + 1)],
        }
    }
    fn index(&self, i: usize, j: usize, k: usize) -> usize {
        (i * (self.ns / 2 + 1) + j) * (self.ns + 1) + k
    }
    /// The slot itself.
    fn raw(&mut self, i: usize, j: usize, k: usize) -> &mut V3 {
        let at = self.index(i, j, k);
        &mut self.co[at]
    }
    fn at(&self, i: usize, j: usize, k: usize) -> V3 {
        self.co[self.index(i, j, k)]
    }
    /// The point that stands for the slot.
    fn get(&self, i: usize, j: usize, k: usize) -> V3 {
        let (i, j, k) = canon(self.n, self.ns, i, j, k);
        self.at(i, j, k)
    }
    fn set(&mut self, i: usize, j: usize, k: usize, co: V3) {
        let (i, j, k) = canon(self.n, self.ns, i, j, k);
        *self.raw(i, j, k) = co;
    }
    fn copy_equiv(&mut self) {
        for i in 0..self.n {
            for j in 0..=self.ns / 2 {
                for k in 0..=self.ns {
                    let c = canon(self.n, self.ns, i, j, k);
                    if c != (i, j, k) {
                        let co = self.at(c.0, c.1, c.2);
                        *self.raw(i, j, k) = co;
                    }
                }
            }
        }
    }
}

/// Hardened normals: a face of the mesh keeps its own (smooth across the edges left smooth), a strip's corners
/// take the face they touch or the mean of the strips either side, a corner patch's the face, strips or
/// vertex they stand on.
fn harden(out: &mut Mesh, kinds: &[Kind], long: &BTreeSet<(u32, u32)>, old: &Mesh, kept: &[u32]) {
    let nf = out.faces.len();
    let flat = |k: Kind| matches!(k, Kind::Kept | Kind::Cut);
    let mut leaving: BTreeMap<(u32, u32), (u32, u32)> = BTreeMap::new();
    for f in 0..nf {
        let c = out.corners(f);
        for (i, &a) in c.iter().enumerate() {
            leaving.insert((a, c[(i + 1) % c.len()]), (f as u32, i as u32));
        }
    }
    // sharp: where a face of the mesh meets the bevel, and the mesh's own sharp edges that are left
    out.sharp = old
        .sharp
        .iter()
        .filter(|e| kept[e.0 as usize] != u32::MAX && kept[e.1 as usize] != u32::MAX)
        .map(|e| edge(kept[e.0 as usize], kept[e.1 as usize]))
        .collect();
    for (&(a, b), &(f, _)) in &leaving {
        if let Some(&(g, _)) = leaving.get(&(b, a))
            && flat(kinds[f as usize])
            && !flat(kinds[g as usize])
            && out.faces[f as usize].smooth
        {
            out.sharp.insert(edge(a, b));
        }
    }
    out.normals = None;
    let face_normals = face_normals(out);
    // a vertex's normal: its faces' weighed by their angles at it (by Blender's approximate arc cosine, as
    // every angle here is: the fans' weights, their frames and the angles stored in them)
    let mut vertex_normals = vec![V3::ZERO; out.verts.len()];
    for (f, &normal) in face_normals.iter().enumerate() {
        let c = out.corners(f);
        let n = c.len();
        for i in 0..n {
            let p = out.verts[c[i] as usize];
            let (e1, _) = normalize_v3(out.verts[c[(i + n - 1) % n] as usize] - p);
            let (e2, _) = normalize_v3(out.verts[c[(i + 1) % n] as usize] - p);
            let k = crate::normals::acos_approx(dot_v3(e1, e2));
            let s = &mut vertex_normals[c[i] as usize];
            *s = madd(*s, normal, k);
        }
    }
    // the corner of the face across the side leaving (or entering) corner (f, i), at the same vertex
    let across_leaving = |f: u32, i: u32| -> Option<(u32, u32)> {
        let c = out.corners(f as usize);
        let (a, b) = (c[i as usize], c[(i as usize + 1) % c.len()]);
        let &(g, j) = leaving.get(&(b, a))?;
        Some((g, (j + 1) % out.faces[g as usize].len))
    };
    let across_entering = |f: u32, i: u32| -> Option<(u32, u32)> {
        let c = out.corners(f as usize);
        let (p, a) = (c[(i as usize + c.len() - 1) % c.len()], c[i as usize]);
        leaving.get(&(a, p)).copied()
    };
    let kind = |l: Option<(u32, u32)>| l.map(|(f, _)| kinds[f as usize]);
    let no = |l: Option<(u32, u32)>| face_normals[l.expect("a face").0 as usize];
    let mean = |a: V3, b: V3| normalize_v3(a + b).0;
    // each bevel face's corners: the normal it is given, or none to give
    let mut given: Vec<Option<Option<V3>>> = vec![None; out.loops.len()];
    let prev_corner = |f: u32, i: u32| {
        let n = out.faces[f as usize].len;
        (f, (i + n - 1) % n)
    };
    for f in 0..nf as u32 {
        let k = kinds[f as usize];
        if flat(k) {
            continue;
        }
        let own = face_normals[f as usize];
        for i in 0..out.faces[f as usize].len {
            let lprev = across_leaving(f, i);
            let lnext = across_entering(f, i);
            let (kprev, knext) = (kind(lprev), kind(lnext));
            let is_flat = |k: Option<Kind>| k.is_some_and(flat);
            let pnorm = if k == Kind::Strip {
                if kprev == Some(Kind::Strip) && long.contains(&(f, i)) {
                    Some(mean(own, no(lprev)))
                } else if knext == Some(Kind::Strip) && lnext.is_some_and(|l| long.contains(&l)) {
                    Some(mean(own, no(lnext)))
                } else if is_flat(kprev) && long.contains(&(f, i)) {
                    Some(no(lprev))
                } else if is_flat(knext) && long.contains(&prev_corner(f, i)) {
                    Some(no(lnext))
                } else {
                    None
                }
            } else if kprev == Some(Kind::Corner) && knext == Some(Kind::Corner) {
                let v = out.corners(f as usize)[i as usize];
                Some(normalize_v3(vertex_normals[v as usize]).0)
            } else if is_flat(kprev) {
                Some(no(lprev))
            } else if is_flat(knext) {
                Some(no(lnext))
            } else {
                let lprevprev = lprev.and_then(|(g, j)| across_leaving(g, j));
                let lnextnext = lnext.and_then(|(g, j)| across_entering(g, j));
                let (kpp, knn) = (kind(lprevprev), kind(lnextnext));
                if kprev == Some(Kind::Strip) && is_flat(kpp) {
                    Some(no(lprevprev))
                } else if kprev == Some(Kind::Strip)
                    && knext == Some(Kind::Corner)
                    && kpp == Some(Kind::Strip)
                {
                    Some(mean(no(lprev), no(lprevprev)))
                } else if knext == Some(Kind::Strip)
                    && kprev == Some(Kind::Corner)
                    && knn == Some(Kind::Strip)
                {
                    Some(mean(no(lnext), no(lnextnext)))
                } else {
                    None
                }
            };
            // (a face of no area has no normal to give: the corner keeps its own)
            let l = (out.faces[f as usize].start + i) as usize;
            given[l] = Some(pnorm.filter(|p| dot_v3(*p, *p) > 0.0));
        }
    }
    // Blender's bevel stores each given normal in the frame of its corner's fan on the mesh it is working on (a
    // fan: the corners of one vertex joined across smooth edges; two angles, from the fan's normal and round it
    // from the edge the fan starts at, each as a share of the frame's own angle in 16 bits). The mesh it leaves
    // reads them back in its own fans' frames (`normals::from_stored`), which start elsewhere now and then, and
    // where a fan's corners were given unlike normals it reads the mean of what was stored: the normals are
    // Blender's, not always the ones given.
    let mut stored = vec![[0_i16; 2]; out.loops.len()];
    for fan in fans(out, &face_normals) {
        if !fan.corners.iter().any(|&l| given[l].flatten().is_some()) {
            continue;
        }
        let space = Space::new(&fan);
        for &l in &fan.corners {
            if let Some(p) = given[l].flatten() {
                let s = space.store(p);
                stored[l] = [s[0] as i16, s[1] as i16];
            }
        }
    }
    out.normals = Some(crate::normals::from_stored(out, &stored));
    out.stored = Some(stored.into_iter().map(Some).collect());
}

/// The corners of one vertex that shade as one, in the order Blender walks them round the vertex.
struct Fan {
    corners: Vec<usize>,
    /// The fan's own normal: its faces' weighed by their angles at the vertex.
    normal: V3,
    /// The directions of the edges it crosses, from the one it starts at to the one it ends at.
    edges: Vec<V3>,
    /// A corner with a sharp edge either side: its frame is its own two sides'.
    single: bool,
}

/// The fans of `mesh`'s corners: round each vertex from a sharp edge to the next (or all the way round, if none
/// is sharp), started as Blender starts them.
fn fans(mesh: &Mesh, face_normals: &[V3]) -> Vec<Fan> {
    let nl = mesh.loops.len();
    let mut face_of = vec![0usize; nl];
    let mut on_edge: BTreeMap<(u32, u32), Vec<usize>> = BTreeMap::new();
    for f in 0..mesh.faces.len() {
        let face = mesh.faces[f];
        for i in 0..face.len {
            let l = (face.start + i) as usize;
            face_of[l] = f;
            let (a, b) = (
                mesh.loops[l],
                mesh.loops[(face.start + (i + 1) % face.len) as usize],
            );
            on_edge.entry(edge(a, b)).or_default().push(l);
        }
    }
    let next = |l: usize| {
        let face = mesh.faces[face_of[l]];
        (face.start + (l as u32 - face.start + 1) % face.len) as usize
    };
    let prev = |l: usize| {
        let face = mesh.faces[face_of[l]];
        (face.start + (l as u32 - face.start + face.len - 1) % face.len) as usize
    };
    // a corner's edge: from its vertex to the next corner's
    let edge_of = |l: usize| edge(mesh.loops[l], mesh.loops[next(l)]);
    let sharp = |e: (u32, u32)| {
        let on = &on_edge[&e];
        on.len() != 2
            || mesh.sharp.contains(&e)
            || !mesh.faces[face_of[on[0]]].smooth
            || !mesh.faces[face_of[on[1]]].smooth
            || mesh.loops[on[0]] == mesh.loops[on[1]]
    };
    let dir = |pivot: u32, e: (u32, u32)| {
        let other = if e.0 == pivot { e.1 } else { e.0 };
        normalize_v3(mesh.verts[other as usize] - mesh.verts[pivot as usize]).0
    };
    // from the corner whose edge comes into the vertex (`curr`) and the vertex's own corner, to the same in the
    // next face round
    let step = |curr: usize, pivot: u32| -> (usize, usize) {
        let on = &on_edge[&edge_of(curr)];
        let other = if on[0] == curr { on[1] } else { on[0] };
        let (orig, across) = (mesh.loops[curr], mesh.loops[other]);
        if (orig == across && orig == pivot) || (orig != across && orig != pivot) {
            (prev(other), other)
        } else {
            (next(other), next(other))
        }
    };
    let mut skip = vec![false; nl];
    let mut out = Vec::new();
    for l in 0..nl {
        let (e_curr, e_prev) = (edge_of(l), edge_of(prev(l)));
        let pivot = mesh.loops[l];
        if !sharp(e_curr) {
            // all smooth round the vertex? then the fan starts at its first corner; else at its sharp edge
            if skip[l] || sharp(e_prev) {
                continue;
            }
            skip[l] = true;
            let mut curr = prev(l);
            let whole = loop {
                let (across, vert) = step(curr, pivot);
                curr = across;
                if sharp(edge_of(curr)) {
                    break false;
                }
                if skip[vert] {
                    break vert == l;
                }
                skip[vert] = true;
            };
            if !whole {
                continue;
            }
        }
        if sharp(e_curr) && sharp(e_prev) {
            out.push(Fan {
                corners: vec![l],
                normal: face_normals[face_of[l]],
                edges: vec![dir(pivot, e_curr), dir(pivot, e_prev)],
                single: true,
            });
            continue;
        }
        let vec_org = dir(pivot, e_curr);
        let mut fan = Fan {
            corners: Vec::new(),
            normal: V3::ZERO,
            edges: vec![vec_org],
            single: false,
        };
        let (mut curr, mut vert) = (prev(l), l);
        let mut vec_prev = vec_org;
        loop {
            let e = edge_of(curr);
            let vec_curr = dir(pivot, e);
            let k = crate::normals::acos_approx(dot_v3(vec_curr, vec_prev));
            fan.normal = madd(fan.normal, face_normals[face_of[curr]], k);
            fan.corners.push(vert);
            if e != e_curr {
                fan.edges.push(vec_curr);
            }
            if sharp(e) || e == e_curr {
                if e == e_curr {
                    // (all the way round: it ends at the edge it started at)
                    fan.edges.push(vec_curr);
                }
                break;
            }
            vec_prev = vec_curr;
            (curr, vert) = step(curr, pivot);
            if fan.corners.len() > nl {
                break;
            }
        }
        let (unit, len) = normalize_v3(fan.normal);
        fan.normal = if len == 0.0 {
            face_normals[face_of[l]]
        } else {
            unit
        };
        out.push(fan);
    }
    out
}

/// A fan's frame for the normals stored in it, as Blender defines it.
struct Space {
    lnor: V3,
    vec_ref: V3,
    vec_ortho: V3,
    ref_alpha: f32,
    ref_beta: f32,
}

impl Space {
    const TRIGO: f32 = 0.9999;
    const TWO_PI: f32 = std::f32::consts::TAU;

    fn new(fan: &Fan) -> Space {
        let lnor = fan.normal;
        let (vec_ref, vec_other) = (fan.edges[0], fan.edges[fan.edges.len() - 1]);
        let (dtp_ref, dtp_other) = (dot_v3(vec_ref, lnor), dot_v3(vec_other, lnor));
        if dtp_ref.abs() >= Space::TRIGO || dtp_other.abs() >= Space::TRIGO {
            // (an edge along the normal: no frame, the normal as it is)
            return Space {
                lnor,
                vec_ref: V3::ZERO,
                vec_ortho: V3::ZERO,
                ref_alpha: 0.0,
                ref_beta: 0.0,
            };
        }
        let ref_alpha = if fan.single {
            (crate::normals::acos_approx(dtp_ref) + crate::normals::acos_approx(dtp_other)) / 2.0
        } else {
            // (the first edge again at the end of a fan all the way round is not counted twice)
            let whole = fan.edges.len() > 2 && fan.edges[0] == fan.edges[fan.edges.len() - 1];
            let counted = if whole {
                &fan.edges[..fan.edges.len() - 1]
            } else {
                &fan.edges[..]
            };
            let sum: f32 = counted.iter().fold(0.0, |s, &v| {
                s + crate::normals::acos_approx(dot_v3(v, lnor))
            });
            sum / counted.len() as f32
        };
        let (vec_ref, _) = normalize_v3(vec_ref - mul_v3_fl(lnor, dtp_ref));
        let (vec_ortho, _) = normalize_v3(lnor.cross(vec_ref));
        let (other, _) = normalize_v3(vec_other - mul_v3_fl(lnor, dtp_other));
        let dtp = dot_v3(vec_ref, other);
        let ref_beta = if dtp < Space::TRIGO {
            let beta = crate::normals::acos_approx(dtp);
            if dot_v3(vec_ortho, other) < 0.0 {
                Space::TWO_PI - beta
            } else {
                beta
            }
        } else {
            Space::TWO_PI
        };
        Space {
            lnor,
            vec_ref,
            vec_ortho,
            ref_alpha,
            ref_beta,
        }
    }

    fn short(val: f32) -> i32 {
        if val >= 1.0 {
            32767
        } else if val <= -1.0 {
            -32768
        } else {
            (val * 32767.0 + 0.5).floor() as i32
        }
    }

    /// A normal as the two shares stored for it.
    fn store(&self, custom: V3) -> [i32; 2] {
        let close = (0..3).all(|i| (self.lnor[i] - custom[i]).abs() <= 1e-4);
        if dot_v3(custom, custom) == 0.0 || close {
            return [0, 0];
        }
        let cos_alpha = dot_v3(self.lnor, custom);
        let alpha = crate::normals::acos_approx(cos_alpha);
        let a = if alpha > self.ref_alpha {
            Space::short(-(Space::TWO_PI - alpha) / (Space::TWO_PI - self.ref_alpha))
        } else {
            Space::short(alpha / self.ref_alpha)
        };
        let (vec, _) = normalize_v3(mul_v3_fl(self.lnor, -cos_alpha) + custom);
        let cos_beta = dot_v3(self.vec_ref, vec);
        let b = if cos_beta < Space::TRIGO {
            let mut beta = crate::normals::acos_approx(cos_beta);
            if dot_v3(self.vec_ortho, vec) < 0.0 {
                beta = Space::TWO_PI - beta;
            }
            if beta > self.ref_beta {
                Space::short(-(Space::TWO_PI - beta) / (Space::TWO_PI - self.ref_beta))
            } else {
                Space::short(beta / self.ref_beta)
            }
        } else {
            0
        };
        [a, b]
    }
}

/// A bevel a part is to be given: Blender's modifier, which rounds the mesh as it is when the part is joined,
/// measured or written, not as it was when the bevel was asked for.
#[derive(Clone, Debug)]
pub struct Pending {
    pub width: f64,
    pub segments: usize,
    pub angle: f64,
    pub order: EdgeOrder,
    /// A slab's side faces, from this one on: Blender numbers them by where its memory put their edges, an
    /// order that can't be told from here (`Charts::groups` reads it for the textures).
    pub sides_from: Option<usize>,
}

/// Rounds `obj`'s edges turning more than `angle` degrees by `width` (metres), in `segments` strips, shaded
/// smooth with hardened normals, so the faces it joins stay flat and the bevel reads as a rounded edge, not a
/// facet. As Blender's modifier, it is only asked for here: the part keeps its mesh, which a build can still move,
/// scale or cut, and is rounded when it is joined (`parts::join`), measured or written (`apply`).
pub fn bevel_edges(obj: Obj, width: f64, segments: usize, angle: f64, order: &EdgeOrder) -> Obj {
    if obj.read(|o| o.mesh.faces.first().is_none_or(|f| !f.smooth)) {
        shade(obj, true, SHARP);
    }
    // (a second bevel rounds what the first made)
    apply(obj);
    obj.edit(|o| {
        o.bevel = Some(Pending {
            width,
            segments,
            angle,
            order: order.clone(),
            sides_from: None,
        })
    });
    obj
}

/// The mesh of `obj` with the bevel it was to be given (`None`: it was to be given none), and what each of its
/// faces came from.
fn rounded(o: &crate::scene::Object) -> Option<Bevelled> {
    let p = o.bevel.as_ref()?;
    match bevelled(&o.mesh, p.width, p.segments, p.angle, &p.order) {
        Ok(made) => Some(made),
        Err(why) => panic!("{}: {why}", o.name),
    }
}

/// The points of `obj` as it will be written: with its bevel, if it is still to be given one.
pub fn points(obj: Obj) -> Vec<V3> {
    obj.read(|o| rounded(o).map_or_else(|| o.mesh.verts.clone(), |made| made.mesh.verts))
}

/// Gives `obj` the bevel it was to be given, on its mesh as it is now (once: nothing is left to give after).
pub fn apply(obj: Obj) -> Obj {
    if obj.read(|o| o.bevel.is_none()) {
        return obj;
    }
    let textured = crate::uv::textured(obj);
    obj.edit(|o| {
        // (a textured part's UVs are made on the mesh as it was, as the modifier carried them)
        if textured && !o.mesh.uv_metres {
            match o.uv {
                crate::mesh::UvWay::Box => crate::uv::box_(&mut o.mesh),
                crate::mesh::UvWay::Round => crate::uv::round(&mut o.mesh),
                crate::mesh::UvWay::Done => {}
            }
        }
        let Some(made) = rounded(o) else { return };
        let mut made = made;
        if let Some(uvs) = &o.mesh.uvs {
            made.mesh.uvs = Some(if textured {
                let sides_from = o.bevel.as_ref().and_then(|b| b.sides_from);
                carried(&o.mesh, uvs, &made.mesh, &made, sides_from)
            } else {
                // (a part on plain finishes: its UVs are zero, or will be before they are read)
                vec![[0.0, 0.0]; made.mesh.loops.len()]
            });
            // the mesh is still the one whose UVs were made: joined first, it says so for the whole
            made.mesh.uv_metres = o.mesh.uv_metres;
        }
        o.mesh = made.mesh;
        o.bevel = None;
    });
    obj
}

/// Gives each of `objs` (parts about to be joined, or a model's as written) its UVs and then its bevel, as Blender
/// applies a part's modifiers.
pub fn apply_all(objs: &[Obj]) {
    crate::uv::ensure(objs);
    for &o in objs {
        apply(o);
    }
}

/// Which old face a strip or a corner's polygon takes its texture from where the faces round it don't share one
/// (Blender's `choose_rep_face`): each old face's number among the groups of faces whose UVs run on across their
/// shared edges, its material and its middle.
struct Charts<'a> {
    old: &'a Mesh,
    uvs: &'a [[f32; 2]],
    /// (Only for an odd number of strips, as in Blender; none otherwise.)
    group: Option<Vec<u32>>,
    sides: Sides,
}

impl<'a> Charts<'a> {
    fn new(old: &'a Mesh, uvs: &'a [[f32; 2]], odd: bool, sides_from: Option<usize>) -> Charts<'a> {
        let mut sides = Sides::new();
        for f in 0..old.faces.len() {
            let c = old.corners(f);
            let at = old.faces[f].start as usize;
            for i in 0..c.len() {
                let j = (i + 1) % c.len();
                let (a, b) = if c[i] < c[j] { (i, j) } else { (j, i) };
                sides
                    .entry(edge(c[i], c[j]))
                    .or_default()
                    .push((f as u32, at + a, at + b));
            }
        }
        let mut charts = Charts {
            old,
            uvs,
            group: None,
            sides,
        };
        if odd {
            charts.group = Some(charts.groups(sides_from));
        }
        charts
    }

    /// Whether the UVs run on from face `f` to face `g` across the edge `e` they share: the same at both its ends.
    fn runs_on(&self, e: (u32, u32), f: u32, g: u32) -> bool {
        let Some(faces) = self.sides.get(&edge(e.0, e.1)) else {
            return false;
        };
        if faces.len() != 2 {
            return false;
        }
        let of = |face: u32| faces.iter().find(|s| s.0 == face);
        match (of(f), of(g)) {
            (Some(a), Some(b)) if f != g => {
                self.uvs[a.1] == self.uvs[b.1] && self.uvs[a.2] == self.uvs[b.2]
            }
            _ => false,
        }
    }

    /// The middle of a face's bounds.
    fn middle(&self, f: u32) -> V3 {
        let c = self.old.corners(f as usize);
        let (mut lo, mut hi) = (self.old.verts[c[0] as usize], self.old.verts[c[0] as usize]);
        for &v in c {
            let p = self.old.verts[v as usize];
            lo = V3::new(lo.x.min(p.x), lo.y.min(p.y), lo.z.min(p.z));
            hi = V3::new(hi.x.max(p.x), hi.y.max(p.y), hi.z.max(p.z));
        }
        V3::new(
            0.5 * (lo.x + hi.x),
            0.5 * (lo.y + hi.y),
            0.5 * (lo.z + hi.z),
        )
    }

    /// Each face's group, numbered in the order the faces come to them; then the topmost face's group takes the
    /// first face's number and the bottom-most's the second's, as Blender has it. The groups that start among a
    /// slab's side faces (`sides_from`) come in an order of Blender's that can't be told: they are numbered the
    /// larger first, which is the likelier (a group of more faces is likelier to hold the first of them).
    fn groups(&self, sides_from: Option<usize>) -> Vec<u32> {
        let nf = self.old.faces.len();
        let mut group = vec![u32::MAX; nf];
        let mut count = 0;
        for start in 0..nf {
            if group[start] != u32::MAX {
                continue;
            }
            let mut stack = vec![start as u32];
            while let Some(f) = stack.pop() {
                if group[f as usize] != u32::MAX {
                    continue;
                }
                group[f as usize] = count;
                let c = self.old.corners(f as usize);
                for i in 0..c.len() {
                    let e = (c[i], c[(i + 1) % c.len()]);
                    for &(g, _, _) in &self.sides[&edge(e.0, e.1)] {
                        if g != f && group[g as usize] == u32::MAX && self.runs_on(e, f, g) {
                            stack.push(g);
                        }
                    }
                }
            }
            count += 1;
        }
        if let Some(from) = sides_from {
            let mut first = vec![usize::MAX; count as usize];
            let mut size = vec![0_usize; count as usize];
            for (f, &g) in group.iter().enumerate() {
                first[g as usize] = first[g as usize].min(f);
                size[g as usize] += 1;
            }
            let fixed = first.iter().filter(|&&f| f < from).count();
            let mut loose: Vec<usize> = (0..count as usize).filter(|&g| first[g] >= from).collect();
            loose.sort_by_key(|&g| (std::cmp::Reverse(size[g]), first[g]));
            let mut renumbered = vec![0_u32; count as usize];
            let mut next = 0;
            for g in 0..count as usize {
                if first[g] < from {
                    renumbered[g] = next;
                    next += 1;
                }
            }
            debug_assert_eq!(next as usize, fixed);
            for (i, &g) in loose.iter().enumerate() {
                renumbered[g] = (fixed + i) as u32;
            }
            for g in group.iter_mut() {
                *g = renumbered[*g as usize];
            }
        }
        if count <= 1 {
            return group;
        }
        let (mut top, mut bottom) = ((f32::MIN, 0), (f32::MAX, 0));
        for (f, &g) in group.iter().enumerate() {
            let z = self.middle(f as u32).z;
            if z > top.0 {
                top = (z, g);
            }
            if z < bottom.0 {
                bottom = (z, g);
            }
        }
        let swap = |group: &mut Vec<u32>, a: u32, b: u32| {
            for g in group.iter_mut() {
                *g = if *g == a {
                    b
                } else if *g == b {
                    a
                } else {
                    *g
                };
            }
        };
        let first = group[0];
        swap(&mut group, first, top.1);
        if bottom.1 != top.1 {
            // (the first face's group, if it was the bottom-most, has the topmost's number now)
            let lowest = if bottom.1 == 0 { top.1 } else { bottom.1 };
            let second = group[1.min(nf - 1)];
            swap(&mut group, second, lowest);
        }
        group
    }

    /// The one of `faces` a texture is taken from: the lowest group, then material, then middle (z, x, y), the
    /// first of them where all are alike.
    fn pick(&self, faces: &[u32]) -> u32 {
        let values: Vec<[f32; 5]> = faces
            .iter()
            .map(|&f| {
                let m = self.middle(f);
                [
                    self.group.as_ref().map_or(0.0, |g| g[f as usize] as f32),
                    self.old.faces[f as usize].mat as f32,
                    m.z,
                    m.x,
                    m.y,
                ]
            })
            .collect();
        let mut viable = vec![true; faces.len()];
        let mut left = faces.len();
        let mut best: Option<usize> = None;
        for value in 0..5 {
            if left <= 1 {
                break;
            }
            for f in 0..faces.len() {
                if !viable[f] || Some(f) == best {
                    continue;
                }
                let Some(b) = best else {
                    best = Some(f);
                    continue;
                };
                if values[f][value] < values[b][value] {
                    best = Some(f);
                    for earlier in viable.iter_mut().take(f) {
                        if std::mem::replace(earlier, false) {
                            left -= 1;
                        }
                    }
                } else if values[f][value] > values[b][value] {
                    viable[f] = false;
                    left -= 1;
                }
            }
        }
        faces[best.unwrap_or(0)]
    }
}

/// A corner's patch of an odd number of segments, for its texture (Blender's bevel_build_rings): which old face
/// each corner of its quads reads, and the old edge a corner on the patch's middle lines is read on.
struct Patch<'a> {
    vert: u32,
    across: &'a [(u32, u32, u32)],
    /// For each new point, whether its face is taken before the next point's where the two meet.
    beats: Vec<bool>,
    /// Whether the UVs break across each new point's bevelled edge.
    seam: Vec<bool>,
    /// The face the patch's middle takes its texture from.
    centre: u32,
}

impl<'a> Patch<'a> {
    fn new(charts: &Charts, vert: u32, across: &'a [(u32, u32, u32)]) -> Patch<'a> {
        let n = across.len();
        let beats = (0..n)
            .map(|i| charts.pick(&[across[i].1, across[(i + 1) % n].1]) == across[i].1)
            .collect();
        let seam = across
            .iter()
            .map(|&(to, before, after)| !charts.runs_on((vert, to), before, after))
            .collect();
        let mut choices: Vec<u32> = Vec::new();
        for &(_, before, after) in across {
            let f = charts.pick(&[before, after]);
            if !choices.contains(&f) {
                choices.push(f);
            }
        }
        Patch {
            vert,
            across,
            beats,
            seam,
            centre: charts.pick(&choices),
        }
    }

    fn any_seam(&self) -> bool {
        self.seam.iter().any(|&s| s)
    }

    /// The face of the new point `i`.
    fn rep(&self, i: usize) -> u32 {
        self.across[i % self.across.len()].1
    }

    /// The bevelled edge across from the new point `i`.
    fn old_edge(&self, i: usize) -> (u32, u32) {
        (self.vert, self.across[i % self.across.len()].0)
    }

    /// That edge, if the UVs break across it.
    fn edge(&self, i: usize) -> Option<(u32, u32)> {
        self.seam[i % self.across.len()].then(|| self.old_edge(i))
    }

    /// The face corner `corner` of the quad at step `k` from the new point `i` reads.
    fn face(&self, i: usize, k: usize, corner: usize, half: usize) -> u32 {
        if k < half {
            self.rep(i)
        } else if self.seam[i] {
            self.rep(if self.beats[i] { i } else { i + 1 })
        } else {
            self.rep(if corner == 0 || corner == 3 { i } else { i + 1 })
        }
    }

    /// The edge a point on the patch's middle lines is read on: the point at ring `jj`, step `kk` from `i`.
    fn snap(&self, i: usize, jj: usize, kk: usize, half: usize) -> Option<(u32, u32)> {
        let n = self.across.len();
        let prev = (i + n - 1) % n;
        let centre = |i: usize| {
            let (prev, next) = ((i + n - 1) % n, (i + 1) % n);
            if self.beats[prev] && self.rep(prev) == self.centre {
                self.edge(prev)
            } else if !self.beats[i] && self.rep(next) == self.centre {
                self.edge(i)
            } else {
                None
            }
        };
        match (jj < half, kk) {
            (true, kk) if kk < half => None,
            (true, kk) if kk == half => (!self.beats[i]).then(|| self.edge(i)).flatten(),
            (true, _) => self.beats[i].then(|| self.edge(i)).flatten(),
            (false, kk) if kk < half => self.beats[prev].then(|| self.edge(prev)).flatten(),
            (false, kk) if kk == half => centre(i),
            (false, _) => centre((i + 1) % n),
        }
    }
}

/// The point of the segment a-b nearest to `p`.
fn on_segment(p: V3, a: V3, b: V3) -> V3 {
    let d = b - a;
    let length = dot_v3(d, d);
    if length == 0.0 {
        return a;
    }
    let t = (dot_v3(p - a, d) / length).clamp(0.0, 1.0);
    V3::new(a.x + d.x * t, a.y + d.y * t, a.z + d.z * t)
}

/// Each new corner's UV, as Blender's bevel carries them: its source face's own way across the face, at the
/// corner's point. A strip between two faces whose UVs don't run on across the edge takes one face's texture
/// (the middle strip of an odd number; of an even number, half the strips each), its points beyond that face
/// read on the old edge; so does a corner's polygon, its points off the face read on the face's nearer edge at
/// the vertex.
pub fn carried(
    old: &Mesh,
    uvs: &[[f32; 2]],
    new: &Mesh,
    made: &Bevelled,
    sides_from: Option<usize>,
) -> Vec<[f32; 2]> {
    /// A face's UVs as a point of it, two of its sides and how the UV changes along each.
    struct Frame {
        p0: V3,
        u0: [f64; 2],
        sides: [V3; 2],
        /// The inverse of the sides' dot products with each other.
        inverse: [f64; 3],
        rates: [[f64; 2]; 2],
    }
    let frames: Vec<Option<Frame>> = (0..old.faces.len())
        .map(|f| {
            let c = old.corners(f);
            let at = old.faces[f].start as usize;
            let p0 = old.verts[c[0] as usize];
            let uv = |i: usize| [uvs[at + i][0] as f64, uvs[at + i][1] as f64];
            let e1 = old.verts[c[1] as usize] - p0;
            // with the corner farthest from the first side's line
            let height = |i: usize| e1.cross(old.verts[c[i] as usize] - p0).length();
            let far = (2..c.len()).max_by(|&a, &b| height(a).total_cmp(&height(b)))?;
            let e2 = old.verts[c[far] as usize] - p0;
            let (g11, g12, g22) = (e1.dot(e1), e1.dot(e2), e2.dot(e2));
            let det = g11 * g22 - g12 * g12;
            if det.abs() < 1e-30 {
                return None;
            }
            let (u0, u1, u2) = (uv(0), uv(1), uv(far));
            Some(Frame {
                p0,
                u0,
                sides: [e1, e2],
                inverse: [g22 / det, -g12 / det, g11 / det],
                rates: [
                    [u1[0] - u0[0], u1[1] - u0[1]],
                    [u2[0] - u0[0], u2[1] - u0[1]],
                ],
            })
        })
        .collect();
    let ns = made.segments;
    let (odd, half) = (ns % 2 == 1, ns / 2);
    let charts = Charts::new(old, uvs, odd, sides_from);
    let mut source = made.source.clone();
    let mut read_at: Vec<Option<(u32, u32)>> = vec![None; source.len()];
    for pick in &made.picks {
        match pick {
            Pick::Strip { at, edge, faces, k } => {
                let (f1, f2) = *faces;
                // (corners 0 and 1 are the nearer to f1, 2 and 3 to f2)
                let (from, far): (u32, &[usize]) = if odd && *k == half + 1 {
                    if charts.runs_on(*edge, f1, f2) {
                        source[*at..*at + 2].fill(f1);
                        source[*at + 2..*at + 4].fill(f2);
                        continue;
                    }
                    let from = charts.pick(&[f1, f2]);
                    (from, if from == f1 { &[2, 3] } else { &[0, 1] })
                } else if !odd && *k == half {
                    (f1, &[2, 3])
                } else if !odd && *k == half + 1 {
                    (f2, &[0, 1])
                } else if odd
                    && ns > 1
                    && (*k == half || *k == half + 2)
                    && !charts.runs_on(*edge, f1, f2)
                    && (charts.pick(&[f1, f2]) == f1) == (*k == half + 2)
                {
                    // the strip beside the middle one, on the face the middle one doesn't read: its points on
                    // the middle strip's side are read on the old edge too, so its face's texture runs up to it
                    if *k == half {
                        (f1, &[2, 3])
                    } else {
                        (f2, &[0, 1])
                    }
                } else {
                    (if *k <= half { f1 } else { f2 }, &[])
                };
                source[*at..*at + 4].fill(from);
                for &i in far {
                    read_at[*at + i] = Some(*edge);
                }
            }
            Pick::Patch {
                at,
                vert,
                place,
                across,
            } => {
                let patch = Patch::new(&charts, *vert, across);
                let (i, j, k) = *place;
                if !odd {
                    // an even number: the quads that touch the patch's middle lines read their points there on
                    // the old edges
                    let n = across.len();
                    let on_next = (k + 1 == half).then(|| patch.old_edge(i));
                    let on_prev = (j + 1 == half).then(|| patch.old_edge(i + n - 1));
                    source[*at..*at + 4].fill(patch.rep(i));
                    read_at[*at + 1] = on_next;
                    read_at[*at + 2] = on_next.or(on_prev);
                    read_at[*at + 3] = on_prev;
                    continue;
                }
                for corner in 0..4 {
                    let jj = j + (corner >= 2) as usize;
                    let kk = k + (corner == 1 || corner == 2) as usize;
                    source[*at + corner] = patch.face(i, k, corner, half);
                    read_at[*at + corner] = patch.snap(i, jj, kk, half);
                }
            }
            Pick::Corner {
                at,
                vert,
                on,
                edges,
                across,
            } => {
                if !across.is_empty() && odd {
                    // the polygon in the middle of a patch: its corners as the quads round it have them
                    let patch = Patch::new(&charts, *vert, across);
                    for i in 0..on.len() {
                        source[*at + i] = if patch.any_seam() {
                            patch.centre
                        } else {
                            patch.rep(i)
                        };
                        read_at[*at + i] = patch.snap(i, half, half, half);
                    }
                    continue;
                }
                if edges
                    .iter()
                    .all(|&(to, before, after)| charts.runs_on((*vert, to), before, after))
                {
                    continue;
                }
                let mut choices: Vec<u32> = Vec::new();
                for &(_, before, after) in edges {
                    let f = charts.pick(&[before, after]);
                    if !choices.contains(&f) {
                        choices.push(f);
                    }
                }
                let from = charts.pick(&choices);
                // the face's two sides at the vertex, in the face's order of them
                let c = old.corners(from as usize);
                let Some(p) = c.iter().position(|&v| v == *vert) else {
                    continue;
                };
                let n = c.len();
                let sides = if p == 0 {
                    [(c[0], c[1]), (c[n - 1], c[0])]
                } else {
                    [(c[p - 1], c[p]), (c[p], c[(p + 1) % n])]
                };
                for (i, &face) in on.iter().enumerate() {
                    source[*at + i] = from;
                    if face != from {
                        let point = new.verts[new.loops[*at + i] as usize];
                        let apart = |s: (u32, u32)| {
                            let q =
                                on_segment(point, old.verts[s.0 as usize], old.verts[s.1 as usize]);
                            dot_v3(point - q, point - q)
                        };
                        read_at[*at + i] = Some(if apart(sides[1]) < apart(sides[0]) {
                            sides[1]
                        } else {
                            sides[0]
                        });
                    }
                }
            }
        }
    }
    let mut out = vec![[0.0_f32; 2]; new.loops.len()];
    for (l, &f) in source.iter().enumerate() {
        if let Some(frame) = &frames[f as usize] {
            let mut point = new.verts[new.loops[l] as usize];
            if let Some((a, b)) = read_at[l] {
                point = on_segment(point, old.verts[a as usize], old.verts[b as usize]);
            }
            let d = point - frame.p0;
            let (x, y) = (d.dot(frame.sides[0]), d.dot(frame.sides[1]));
            let a = frame.inverse[0] * x + frame.inverse[1] * y;
            let b = frame.inverse[1] * x + frame.inverse[2] * y;
            out[l] = [
                (frame.u0[0] + a * frame.rates[0][0] + b * frame.rates[1][0]) as f32,
                (frame.u0[1] + a * frame.rates[0][1] + b * frame.rates[1][1]) as f32,
            ];
        }
    }
    out
}
