//! The ears: each a small, dense piece of its own set into the side of the smooth head, as the lids
//! and lips are pieces on the face (build.rs appends them to the `body` part after those, the left
//! one then the right; `lod.body` gets a plain flap of 24 triangles).
//!
//! An ear is laid out on a sheet hinged on the head along the ear's front edge and turned out from
//! it towards the back (`OUT` off the head at its back edge), leaning back a little at the top: a
//! point of it is (a, b, h), a from the front edge to the back (0..1 of the ear's breadth), b from
//! the lobe to the top (0..1 of its height), h out of the sheet. Its outline (`OUTLINE`, a closed
//! curve through a few points) says where the ear is attached (the front edge, where it runs into
//! the cheek) and where its rim is rolled (the helix, from its root at the front round the top and
//! down the back to the lobe, which has none).
//!
//! The mesh is a polar grid round the bowl's middle: the bowl's floor (a little over the head's
//! skin), its rim (the antihelix behind and above it, the tragus in front), the groove inside the
//! helix (the scapha), the helix's crest, the ear's edge between two rings close to it, then round
//! to the back: under the helix, under the bowl's rim (so the ear is a shell a few millimetres
//! thick), and a foot ring buried in the head, so nothing is stitched. Only the four rings of the
//! rolled edge have all the edge's points; the others half.
//! Where the piece runs into the head (the foot, the front edge) its normals turn to the head's and
//! build.rs gives it the skin's tone (`seam`), so the join shows no crease and no line.
//!
//! It is built from the family's smooth head at the stage (`head::head_shape`, the shared face:
//! the identity channels leave the ears alone, identity.rs `Mask`), sized by the head's height.

use std::sync::Arc;

use super::head::{cast, head_shape, youth};
use super::identity::Identity;
use super::np::{clip, smoothstep};
use super::rig::proportions;
use super::{Cache, Mesh, V3, key, skin, v3};

/// A millimetre on a grown man's head (238 mm high), of hh: the relief below is in these.
const MM: f64 = 1.0 / 238.0;
/// The ear's height and breadth (of hh; a baby's a tenth smaller) and its middle: above the chin and
/// behind the head joint's z.
const SIZE: (f64, f64) = (0.262, 0.14);
const MIDDLE: (f64, f64) = (0.455, -0.115);
/// How far its top lies behind its bottom (of its height).
const LEAN: f64 = 0.14;
/// How far off the head the sheet's back edge stands, two thirds of the way up (mm).
const OUT: f64 = 13.5;
/// The outline, from where the lobe joins the cheek, round the lobe, up the back, over the top and
/// down the front edge: (a, b), how much it is attached there (1: it runs into the head) and how much
/// of a rolled rim it has (1: the helix).
const OUTLINE: [(f64, f64, f64, f64); 16] = [
    (0.08, 0.19, 1.0, 0.0),
    (0.17, 0.055, 0.15, 0.0),
    (0.37, 0.0, 0.0, 0.0),
    (0.59, 0.065, 0.0, 0.0),
    (0.76, 0.22, 0.0, 0.5),
    (0.90, 0.39, 0.0, 1.0),
    (0.98, 0.56, 0.0, 1.0),
    (1.0, 0.72, 0.0, 1.0),
    (0.94, 0.87, 0.0, 1.0),
    (0.78, 0.97, 0.0, 1.0),
    (0.54, 1.0, 0.0, 1.0),
    (0.30, 0.96, 0.0, 1.0),
    (0.12, 0.86, 0.3, 0.9),
    (0.03, 0.72, 1.0, 0.3),
    (0.0, 0.55, 1.0, 0.0),
    (0.02, 0.35, 1.0, 0.0),
];
/// The edge's points (the rings in from the crest and on the back have half of them; the low-detail ear
/// a third).
const SEGS: usize = 24;
/// The bowl (the concha): its middle on the sheet (a, b), how far its rim is from it towards the
/// front, the back, the top and the bottom (mm), and how far over the skin its floor lies (more than
/// the occlusion bake's lift, ao.rs, so the skin under it is not shaded as the bowl is).
const BOWL: (f64, f64) = (0.36, 0.48);
const BOWL_RIM: [f64; 4] = [8.0, 9.0, 13.5, 10.5];
const BOWL_FLOOR: f64 = 2.0;
/// The bowl's rim above the sheet (mm): behind and above the bowl (the antihelix), more at the back,
/// less at the bottom (the notch over the lobe); in front, the tragus over the skin: its foot, how
/// much higher its top is, and where up the ear (b, and over how much of it) the top is.
const ANTIHELIX: (f64, f64, f64) = (3.8, 0.6, 1.4);
const TRAGUS: (f64, f64, f64, f64) = (1.5, 2.0, 0.44, 0.1);
/// The rings from the groove out, each on the lobe and on the helix: how far in from the edge (mm)
/// and how far above the sheet. The groove (the scapha; on the lobe the pad's middle), the crest, the
/// edge with a ring close either side of it (the edge is where the ear's outline is from most sides: its
/// turn is in two narrow bands, so no broad one is shaded as turning away from the eye), and under the
/// edge on the back.
const GROOVE: [(f64, f64); 2] = [(7.0, 2.8), (6.5, 0.4)];
const CREST: [(f64, f64); 2] = [(3.4, 2.6), (3.0, 3.7)];
const BEFORE: [(f64, f64); 2] = [(1.1, 1.7), (0.9, 3.0)];
const EDGE: [f64; 2] = [0.3, 1.5];
const AFTER: [(f64, f64); 2] = [(1.0, -1.1), (0.9, 0.0)];
const UNDER: [(f64, f64); 2] = [(3.2, -1.8), (3.0, -0.6)];
/// Where the ear is attached, those rings over the skin instead (real mm, on any head; the edge and
/// what is after it in the head).
const ATTACHED: [f64; 6] = [1.3, 0.4, -0.6, -2.0, -2.6, -3.0];
/// The back under the bowl's rim: how far out from the rim (mm) and how far under it; then the foot,
/// further out by this and by this more above and below the bowl, and buried this deep in the head.
const BACK: (f64, f64) = (1.0, 4.2);
const FOOT: (f64, f64, f64) = (3.0, 2.0, 2.0);
/// Blush (tone.x): the lobe, the rim, the bowl; and the shade in the bowl's middle, the ear's canal
/// (tone.y).
const BLUSH: (f64, f64, f64) = (0.26, 0.2, 0.14);
const CANAL: (f64, f64) = (0.3, 0.12);
/// How much of the baked occlusion (tone.w, build.rs) the ear keeps: its front, its edge, its back. All
/// of it, the back, which faces the head a centimetre off, is as dark as the inside of a mouth; an ear
/// is thin and lit through.
const SHADE: (f64, f64, f64) = (0.7, 0.5, 0.4);

/// An ear's vertices and triangles, and the low-detail ear's.
pub const VERTS: usize = 1 + 6 * (SEGS / 2) + 4 * SEGS;
pub const TRIS: usize = SEGS / 2 + 4 * (SEGS / 2) * 2 + 2 * (SEGS / 2) * 3 + 3 * SEGS * 2;
pub const LOD_VERTS: usize = 1 + 2 * (SEGS / 3);
pub const LOD_TRIS: usize = 3 * (SEGS / 3);

/// One ear: its mesh (positions, normals, triangles counter-clockwise from outside, the `Head` bone,
/// its blush and canal shade in `tone`) and, per vertex, how much it is the head's there (`seam`,
/// 0..1: build.rs gives it that much of the skin's tone) and how much of its baked occlusion it keeps
/// (`shade`).
pub struct Ear {
    pub mesh: Mesh,
    pub seam: Vec<f64>,
    pub shade: Vec<f64>,
}

/// A body's ears, the left (-x) then the right: on `body` (and `hd.body`), and on `lod.body`.
pub struct Ears {
    pub full: [Ear; 2],
    pub lod: [Ear; 2],
}

impl Ears {
    /// A box round each ear (left, right): centre and half extents.
    pub fn boxes(&self) -> [[V3; 2]; 2] {
        [0, 1].map(|e| {
            let p = &self.full[e].mesh.pos;
            let mx = p
                .iter()
                .fold(V3::repeat(f64::NEG_INFINITY), |a, q| a.sup(q));
            let mn = p.iter().fold(V3::repeat(f64::INFINITY), |a, q| a.inf(q));
            [(mx + mn) / 2.0, (mx - mn) / 2.0]
        })
    }

    /// The ears' faces (left, right), and the faces that close each over its foot (a fan over the foot
    /// ring, in the head): as built, or on the positions `on` of a `body` part, which the ears end
    /// (`VERTS` each, the left one first: the ears as a built set has them, check.rs).
    pub fn faces(&self, on: Option<&[V3]>) -> [[Vec<[V3; 3]>; 2]; 2] {
        [0, 1].map(|e| {
            let m = &self.full[e].mesh;
            let at = match on {
                Some(pos) => {
                    let first = pos.len() - (2 - e) * VERTS;
                    &pos[first..first + VERTS]
                }
                None => &m.pos[..],
            };
            let own = m
                .idx
                .chunks(3)
                .map(|t| [at[t[0]], at[t[1]], at[t[2]]])
                .collect();
            let foot = &at[VERTS - SEGS / 2..];
            let mid = foot.iter().sum::<V3>() / foot.len() as f64;
            let cap = (0..foot.len())
                .map(|i| [mid, foot[i], foot[(i + 1) % foot.len()]])
                .collect();
            [own, cap]
        })
    }
}

static EARS: Cache<(String, String), Ears> = Cache::new();

/// The ears of `family` at `stage` (cached).
pub fn ears(family: &str, stage: &str) -> Arc<Ears> {
    EARS.get(key(family, stage), || {
        let (full, lod) = build(family, stage);
        Ears {
            full: [mirrored(&full), full],
            lod: [mirrored(&lod), lod],
        }
    })
}

/// The left ear of a right one.
fn mirrored(e: &Ear) -> Ear {
    let flip = |v: &V3| v3(-v.x, v.y, v.z);
    Ear {
        mesh: Mesh {
            pos: e.mesh.pos.iter().map(flip).collect(),
            nor: e.mesh.nor.iter().map(flip).collect(),
            idx: e
                .mesh
                .idx
                .chunks(3)
                .flat_map(|t| [t[0], t[2], t[1]])
                .collect(),
            ..e.mesh.clone()
        },
        seam: e.seam.clone(),
        shade: e.shade.clone(),
    }
}

/// The sheet a right ear is laid out on (head units: hh, from the chin's height and the head joint's z).
struct Sheet<F: Fn(V3) -> f64> {
    f: F,
    high: f64,
    wide: f64,
    /// Along the sheet towards the back, and out of it.
    back: V3,
    out: V3,
}

impl<F: Fn(V3) -> f64> Sheet<F> {
    /// The sheet turned `phi` out from the head's middle plane.
    fn turned(self, phi: f64) -> Self {
        Sheet {
            back: v3(phi.sin(), 0.0, -phi.cos()),
            out: v3(phi.cos(), 0.0, phi.sin()),
            ..self
        }
    }

    /// The front edge at b: on the head's side.
    fn hinge(&self, b: f64) -> V3 {
        let y = MIDDLE.0 + (b - 0.5) * self.high;
        let z = MIDDLE.1 + 0.5 * self.wide - LEAN * (b - 0.5) * self.high;
        cast(&self.f, v3(0.0, y, z), v3(1.0, 0.0, 0.0))
    }

    fn at(&self, a: f64, b: f64, h: f64) -> V3 {
        self.hinge(b) + self.back * (a * self.wide) + self.out * h
    }

    /// How far out of the sheet the head's skin is at (a, b) (under it: negative).
    fn skin(&self, a: f64, b: f64) -> f64 {
        let (mut lo, mut hi) = (-0.3, 0.1);
        for _ in 0..40 {
            let mid = (lo + hi) / 2.0;
            if (self.f)(self.at(a, b, mid)) < 0.0 {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        lo
    }

    /// The head's outward normal at a point.
    fn normal(&self, p: V3) -> V3 {
        let e = 1e-4;
        let d = |k: usize| {
            let mut dv = V3::zeros();
            dv[k] = e;
            (self.f)(p + dv) - (self.f)(p - dv)
        };
        v3(d(0), d(1), d(2)).normalize()
    }
}

/// A point of the outline: where on the sheet, how attached, how much helix, and from the bowl's
/// middle its direction on the sheet (in lengths, not in a and b) and its distance (of hh).
#[derive(Clone, Copy)]
struct Spoke {
    att: f64,
    helix: f64,
    dir: (f64, f64),
    len: f64,
}

/// `SEGS` points evenly along the outline (a closed Catmull-Rom curve through `OUTLINE`; half as
/// close together where it is attached, which no one sees), from its first point.
fn outline(wide: f64, high: f64) -> Vec<[f64; 4]> {
    const FINE: usize = 32;
    let n = OUTLINE.len();
    let p = |i: usize| {
        let (a, b, t, h) = OUTLINE[i % n];
        [a, b, t, h]
    };
    let mut fine: Vec<[f64; 4]> = Vec::with_capacity(n * FINE + 1);
    for i in 0..n {
        let (p0, p1, p2, p3) = (p(i + n - 1), p(i), p(i + 1), p(i + 2));
        for s in 0..FINE {
            let t = s as f64 / FINE as f64;
            let mut q = [0.0; 4];
            for k in 0..4 {
                q[k] = 0.5
                    * (2.0 * p1[k]
                        + (p2[k] - p0[k]) * t
                        + (2.0 * p0[k] - 5.0 * p1[k] + 4.0 * p2[k] - p3[k]) * t * t
                        + (3.0 * p1[k] - p0[k] - 3.0 * p2[k] + p3[k]) * t * t * t);
            }
            q[2] = clip(q[2], 0.0, 1.0);
            q[3] = clip(q[3], 0.0, 1.0);
            fine.push(q);
        }
    }
    fine.push(fine[0]);
    let mut s = vec![0.0];
    for w in fine.windows(2) {
        let d = ((w[1][0] - w[0][0]) * wide).hypot((w[1][1] - w[0][1]) * high);
        s.push(s[s.len() - 1] + d * (1.0 - 0.5 * (w[0][2] + w[1][2]) / 2.0));
    }
    let total = s[s.len() - 1];
    (0..SEGS)
        .map(|j| {
            let at = total * j as f64 / SEGS as f64;
            let i = (s.partition_point(|&v| v <= at) - 1).min(fine.len() - 2);
            let t = (at - s[i]) / (s[i + 1] - s[i]).max(1e-12);
            let mut q = [0.0; 4];
            for k in 0..4 {
                q[k] = fine[i][k] + (fine[i + 1][k] - fine[i][k]) * t;
            }
            q
        })
        .collect()
}

fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

/// A ring of the mesh being built: its vertices' ids.
type Ring = Vec<usize>;

#[derive(Default)]
struct Builder {
    pos: Vec<V3>,
    /// Per vertex, how much it is the head's: its tone (`Ear::seam`), and its normal.
    seam: Vec<f64>,
    turn: Vec<f64>,
    tone: Vec<[f64; 4]>,
    /// How much of the baked occlusion the vertices added from here on keep.
    keeps: f64,
    shade: Vec<f64>,
    idx: Vec<usize>,
}

impl Builder {
    fn add(&mut self, p: V3, (seam, turn): (f64, f64), blush: f64, shade: f64) -> usize {
        self.pos.push(p);
        self.seam.push(clip(seam, 0.0, 1.0));
        self.turn.push(clip(turn, 0.0, 1.0));
        self.tone.push([blush * (1.0 - seam), shade, 0.0, 0.0]);
        self.shade.push(self.keeps);
        self.pos.len() - 1
    }

    /// A quad a b c d, split along the diagonal it is flatter across (the shorter one, if as flat).
    fn quad(&mut self, a: usize, b: usize, c: usize, d: usize) {
        let p = &self.pos;
        let n = |i: usize, j: usize, k: usize| {
            (p[j] - p[i])
                .cross(&(p[k] - p[i]))
                .try_normalize(1e-18)
                .unwrap_or_else(V3::zeros)
        };
        let (ac, bd) = (n(a, b, c).dot(&n(a, c, d)), n(a, b, d).dot(&n(b, c, d)));
        let shorter = (p[a] - p[c]).norm_squared() <= (p[b] - p[d]).norm_squared();
        if ac > bd + 0.02 || (ac > bd - 0.02 && shorter) {
            self.idx.extend([a, b, c, a, c, d]);
        } else {
            self.idx.extend([a, b, d, b, c, d]);
        }
    }

    /// The band from ring `a` out to ring `b`: as many points, or twice or half as many (a point of
    /// the sparser ring at every other of the denser one's, from its first).
    fn band(&mut self, a: &Ring, b: &Ring) {
        let (na, nb) = (a.len(), b.len());
        if na == nb {
            for i in 0..na {
                let j = (i + 1) % na;
                self.quad(a[i], b[i], b[j], a[j]);
            }
        } else if nb == 2 * na {
            for i in 0..na {
                let j = (i + 1) % na;
                let (o0, o1, o2) = (b[2 * i], b[2 * i + 1], b[(2 * i + 2) % nb]);
                self.idx
                    .extend([a[i], o0, o1, a[i], o1, a[j], a[j], o1, o2]);
            }
        } else {
            assert_eq!(na, 2 * nb, "ear: rings that do not fit");
            for i in 0..nb {
                let j = (i + 1) % nb;
                let (o0, o1, o2) = (a[2 * i], a[2 * i + 1], a[(2 * i + 2) % na]);
                self.idx
                    .extend([o0, b[i], o1, o1, b[i], b[j], o1, b[j], o2]);
            }
        }
    }

    fn fan(&mut self, centre: usize, ring: &Ring) {
        for i in 0..ring.len() {
            self.idx
                .extend([centre, ring[i], ring[(i + 1) % ring.len()]]);
        }
    }

    /// Vertex normals: its triangles' weighted by their angle at the vertex, not by their area (the ear's
    /// folds are sharp and its triangles of very different sizes: by area a fold's normal is its larger
    /// side's), then leant towards any of its triangles it still faces away from until it faces them all:
    /// no corner is shaded as the back of its triangle.
    fn normals(&self) -> Vec<V3> {
        let mut acc = vec![V3::zeros(); self.pos.len()];
        let mut faces: Vec<Vec<V3>> = vec![Vec::new(); self.pos.len()];
        for t in self.idx.chunks(3) {
            let p = [self.pos[t[0]], self.pos[t[1]], self.pos[t[2]]];
            let Some(n) = (p[1] - p[0]).cross(&(p[2] - p[0])).try_normalize(1e-18) else {
                continue;
            };
            for k in 0..3 {
                let (u, v) = (p[(k + 1) % 3] - p[k], p[(k + 2) % 3] - p[k]);
                acc[t[k]] += n * u.angle(&v);
                faces[t[k]].push(n);
            }
        }
        acc.iter()
            .zip(&faces)
            .map(|(n, faces)| {
                let mut n = n / n.norm().max(1e-12);
                for _ in 0..64 {
                    let lean: V3 = faces.iter().map(|f| f * (FACING - n.dot(f)).max(0.0)).sum();
                    if lean == V3::zeros() {
                        break;
                    }
                    n = (n + lean).normalize();
                }
                n
            })
            .collect()
    }
}

/// A vertex's normal faces each of its triangles by at least this (the cosine between them).
const FACING: f64 = 0.1;

/// The right ear of `family` at `stage` and its low-detail copy (model space, metres).
fn build(family: &str, stage: &str) -> (Ear, Ear) {
    let pr = proportions(family, stage);
    let hh = pr.hh * pr.h;
    let origin = v3(0.0, pr.h - hh, 0.0);
    let k = 1.0 - 0.1 * youth(pr.hh);
    // The relief's millimetre (the ear's own size) and the skin's (a real one: what keeps the ear's
    // faces off the skin's, or under them, on every head).
    let (mm, real) = (MM * k, 0.001 / hh);
    let sheet = Sheet {
        f: head_shape(family, stage, &Identity::NONE),
        high: SIZE.0 * k,
        wide: SIZE.1 * k,
        back: V3::zeros(),
        out: V3::zeros(),
    };
    // Turned out until its back edge stands `OUT` off the head.
    let (mut lo, mut hi) = (0.0f64, 0.9f64);
    let mut sheet = sheet.turned(lo);
    for _ in 0..30 {
        let mid = (lo + hi) / 2.0;
        sheet = sheet.turned(mid);
        if (sheet.f)(sheet.at(1.0, 0.66, 0.0)) < OUT * mm {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    let sheet = sheet.turned(lo);
    let (wide, high) = (sheet.wide, sheet.high);
    let spokes: Vec<Spoke> = outline(wide, high)
        .iter()
        .map(|&[a, b, att, helix]| {
            let d = ((a - BOWL.0) * wide, (b - BOWL.1) * high);
            let len = d.0.hypot(d.1);
            Spoke {
                att,
                helix,
                dir: (d.0 / len, d.1 / len),
                len,
            }
        })
        .collect();
    // A spoke's point `d` out from the bowl's middle, on the sheet.
    let on = |s: &Spoke, d: f64| (BOWL.0 + d * s.dir.0 / wide, BOWL.1 + d * s.dir.1 / high);
    // The bowl's rim along a spoke: how far out, and how far above the sheet.
    let rim = |s: &Spoke| {
        let (ux, uy) = s.dir;
        let rx = if ux < 0.0 { BOWL_RIM[0] } else { BOWL_RIM[1] };
        let ry = if uy > 0.0 { BOWL_RIM[2] } else { BOWL_RIM[3] };
        let d = (mm / (ux / rx).hypot(uy / ry)).min(0.6 * s.len);
        let (a, b) = on(s, d);
        let skin = sheet.skin(a, b);
        let behind = (ANTIHELIX.0 + ANTIHELIX.1 * ux.max(0.0) - ANTIHELIX.2 * (-uy).max(0.0)) * mm;
        let tragus =
            skin + (TRAGUS.0 + TRAGUS.1 * (-((b - TRAGUS.2) / TRAGUS.3).powi(2)).exp()) * mm;
        let h = lerp(behind, tragus, smoothstep(0.2, 0.8, -ux)).max(skin + TRAGUS.0 * mm);
        (d, h)
    };
    // A ring from the groove out: `of` on the lobe and on the helix (how far in from the edge, how far
    // above the sheet), no nearer the bowl's rim than `least` of the way from it to the edge, and
    // `over` the skin where the ear is attached.
    let outer = |s: &Spoke, of: [(f64, f64); 2], least: f64, over: f64| {
        let (r, _) = rim(s);
        let inset = lerp(of[0].0, of[1].0, s.helix) * mm;
        let d = (s.len - inset).max(r + least * (s.len - r));
        let (a, b) = on(s, d);
        let free = lerp(of[0].1, of[1].1, s.helix) * mm;
        let h = lerp(free, sheet.skin(a, b) + over * real, s.att);
        (a, b, h)
    };
    let edge = |s: &Spoke| outer(s, [(0.0, EDGE[0]), (0.0, EDGE[1])], 0.0, ATTACHED[3]);
    // The back along a spoke: under the bowl's rim (in the head where the ear is attached: the tragus
    // is low), then the foot on the head, buried.
    let foot = |s: &Spoke| {
        let (r, h) = rim(s);
        let d2 = r + BACK.0 * mm;
        let (a2, b2) = on(s, d2);
        let under = h - BACK.1 * mm;
        let h2 = lerp(under, under.min(sheet.skin(a2, b2) - FOOT.2 * real), s.att);
        // (Where the ear is attached the foot is straight under it, deeper: the back's rings go on
        // into the head the way they go round the ear, and nowhere fold back on themselves.)
        let d3 = d2 + (FOOT.0 + FOOT.1 * s.dir.1.abs()) * mm * (1.0 - s.att);
        let (a3, b3) = on(s, d3);
        let deep = FOOT.2 * (1.0 + 1.5 * s.att) * real;
        ((a2, b2, h2), (a3, b3, sheet.skin(a3, b3) - deep))
    };
    let rim_blush = |s: &Spoke| lerp(BLUSH.0, BLUSH.1, s.helix);
    // How much a vertex of the edge is the head's: its tone as it nears the head, its normal only once
    // it and the edge either side of it are in it (the edge is the front's and the back's, and the back
    // faces the head).
    let sunk: Vec<(f64, f64)> = (0..SEGS)
        .map(|j| {
            let near = [j + SEGS - 1, j, j + 1].map(|i| spokes[i % SEGS].att);
            (
                spokes[j].att,
                smoothstep(0.7, 1.0, near.into_iter().fold(1.0, f64::min)),
            )
        })
        .collect();

    let make = |full: bool| -> Ear {
        let mut m = Builder::default();
        let put = |m: &mut Builder,
                   (a, b, h): (f64, f64, f64),
                   head: (f64, f64),
                   blush: f64,
                   shade: f64| { m.add(sheet.at(a, b, h), head, blush, shade) };
        let every = |step: usize| spokes.iter().step_by(step);
        let edges = |m: &mut Builder, step: usize| -> Ring {
            (0..SEGS)
                .step_by(step)
                .map(|j| put(m, edge(&spokes[j]), sunk[j], rim_blush(&spokes[j]), 0.0))
                .collect()
        };
        let none = (0.0, 0.0);
        m.keeps = SHADE.0;
        if full {
            let floor = |a: f64, b: f64| (a, b, sheet.skin(a, b) + BOWL_FLOOR * real);
            let centre = put(&mut m, floor(BOWL.0, BOWL.1), none, BLUSH.2, CANAL.0);
            let r1: Ring = every(2)
                .map(|s| {
                    let (a, b) = on(s, 0.55 * rim(s).0);
                    put(&mut m, floor(a, b), none, BLUSH.2, CANAL.1)
                })
                .collect();
            let r2: Ring = every(2)
                .map(|s| {
                    let (d, h) = rim(s);
                    let (a, b) = on(s, d);
                    put(&mut m, (a, b, h), none, BLUSH.2, 0.0)
                })
                .collect();
            let r3: Ring = every(2)
                .map(|s| {
                    let head = (0.25 * s.att, 0.25 * s.att);
                    put(
                        &mut m,
                        outer(s, GROOVE, 0.42, ATTACHED[0]),
                        head,
                        BLUSH.2,
                        0.0,
                    )
                })
                .collect();
            let r4: Ring = every(1)
                .map(|s| {
                    let head = (0.6 * s.att, 0.6 * s.att);
                    put(
                        &mut m,
                        outer(s, CREST, 0.72, ATTACHED[1]),
                        head,
                        rim_blush(s),
                        0.0,
                    )
                })
                .collect();
            let ra: Ring = every(1)
                .map(|s| {
                    let head = (0.8 * s.att, 0.8 * s.att);
                    put(
                        &mut m,
                        outer(s, BEFORE, 0.9, ATTACHED[2]),
                        head,
                        rim_blush(s),
                        0.0,
                    )
                })
                .collect();
            m.keeps = SHADE.1;
            let r5 = edges(&mut m, 1);
            m.keeps = SHADE.2;
            let rb: Ring = every(1)
                .map(|s| {
                    let p = outer(s, AFTER, 0.9, ATTACHED[4]);
                    put(&mut m, p, (s.att, 0.0), rim_blush(s), 0.0)
                })
                .collect();
            let b1: Ring = every(2)
                .map(|s| {
                    let p = outer(s, UNDER, 0.66, ATTACHED[5]);
                    put(&mut m, p, (s.att, 0.0), rim_blush(s), 0.0)
                })
                .collect();
            let b2: Ring = every(2)
                .map(|s| put(&mut m, foot(s).0, (s.att, 0.0), BLUSH.2, 0.0))
                .collect();
            let b3: Ring = every(2)
                .map(|s| put(&mut m, foot(s).1, (1.0, 0.8 * (1.0 - s.att)), 0.0, 0.0))
                .collect();
            m.fan(centre, &r1);
            for (a, b) in [
                (&r1, &r2),
                (&r2, &r3),
                (&r3, &r4),
                (&r4, &ra),
                (&ra, &r5),
                (&r5, &rb),
                (&rb, &b1),
                (&b1, &b2),
                (&b2, &b3),
            ] {
                m.band(a, b);
            }
        } else {
            // The low-detail ear: a plate from the bowl's middle to the edge, and down to the foot.
            let centre = put(&mut m, (BOWL.0, BOWL.1, EDGE[1] * mm), none, BLUSH.2, 0.0);
            m.keeps = SHADE.1;
            let rim = edges(&mut m, 3);
            m.keeps = SHADE.2;
            let base: Ring = every(3)
                .map(|s| put(&mut m, foot(s).1, (1.0, 0.8 * (1.0 - s.att)), 0.0, 0.0))
                .collect();
            m.fan(centre, &rim);
            m.band(&rim, &base);
        }
        // Counter-clockwise from outside: the bowl's middle faces out of the sheet.
        let t = &m.idx[..3];
        if (m.pos[t[1]] - m.pos[t[0]])
            .cross(&(m.pos[t[2]] - m.pos[t[0]]))
            .dot(&sheet.out)
            < 0.0
        {
            for t in m.idx.chunks_mut(3) {
                t.swap(1, 2);
            }
        }
        // Its own smooth normals (the edge is shared by the front and the back, so it shades round),
        // turned to the head's where it runs into the head.
        let own = m.normals();
        let nor = (0..m.pos.len())
            .map(|i| {
                let w = m.turn[i];
                (own[i] * (1.0 - w) + sheet.normal(m.pos[i]) * w).normalize()
            })
            .collect();
        let pos: Vec<V3> = m.pos.iter().map(|p| p * hh + origin).collect();
        let n = pos.len();
        Ear {
            mesh: Mesh {
                pos,
                nor,
                idx: m.idx,
                dense: skin(n, "Head"),
                fields: vec![[0.0; 4]; n],
                tone: m.tone,
                arm: vec![0.0; n],
                ..Mesh::default()
            },
            seam: m.seam,
            shade: m.shade,
        }
    };
    (make(true), make(false))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::characters::meshops::boundary_edges;

    #[test]
    fn counts_and_one_open_ring() {
        let e = ears("female", "adult");
        for side in 0..2 {
            let (full, lod) = (&e.full[side].mesh, &e.lod[side].mesh);
            assert_eq!((full.pos.len(), full.idx.len() / 3), (VERTS, TRIS));
            assert_eq!((lod.pos.len(), lod.idx.len() / 3), (LOD_VERTS, LOD_TRIS));
            // Open only at the foot, which is in the head.
            assert_eq!(boundary_edges(&full.idx).len(), SEGS / 2);
            assert_eq!(boundary_edges(&lod.idx).len(), SEGS / 3);
            let x = if side == 0 { -1.0 } else { 1.0 };
            assert!(full.pos.iter().all(|p| p.x * x > 0.0));
        }
    }
}
