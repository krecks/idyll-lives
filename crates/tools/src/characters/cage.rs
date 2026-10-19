//! The residents' control cage: a closed polygon mesh of cross-section rings (superellipses) joined
//! by fixed patches at the shoulders, the crotch and the neck, plus the palm, fingers, thumb,
//! shoe-like feet and the head (head.rs). subdiv.rs subdivides it once for the `body` part (body.rs
//! fits it so its limit surface passes through the ring points); the cage at its limit is the LOD.
//!
//! `Cage::add(point, weights, region, tags) -> vertex id`; `ring(points, ...) -> ids`;
//! `band(a, b)` joins two rings with quads; `join(a, b, match)` joins rings of different sizes
//! (some faces become pentagons).
//!
//! Every life stage and both families build the same vertices in the same order, so the topology
//! is fixed and stages are deltas. Per vertex the cage carries its target point (on the designed
//! surface), its skin weights (bone -> weight), a region name and tags (`arm`: 0 torso .. 1 arm,
//! used by the sleeve field; `cav`: cavity darkening; `blush`), which subdivide like positions.
//!
//! Torso rings have 12 vertices at 30 degree steps (0 = front, 90 = the resident's right, +x): the
//! arms leave through the 2 x 2 patches at 60..120 / 240..300 degrees of the chest rings T5-T7, the
//! legs through the halves of the hip ring T1 and the crotch vertex, the neck through the middle
//! of the shoulder cap. Limbs and the neck have 8-vertex rings, the palm 10, fingers and thumbs 4.

use super::np::sign;
use super::rig::{NB, b};
use super::{Dense, M3, V3};

/// Bone weights of a cage vertex, in the order they were given.
pub type Weights = Vec<(&'static str, f64)>;

/// Superellipse coordinate: sign(t) |t|^(2 / p).
pub fn se(t: f64, p: f64) -> f64 {
    sign(t) * t.abs().powf(2.0 / p)
}

pub fn unit(v: V3) -> V3 {
    v / v.norm()
}

/// The smallest rotation (3x3) taking unit vector `a` to unit vector `b` (not opposite).
pub fn turn(a: V3, b: V3) -> M3 {
    let v = a.cross(&b);
    let cs = a.dot(&b);
    let k = M3::new(0.0, -v[2], v[1], v[2], 0.0, -v[0], -v[1], v[0], 0.0);
    M3::identity() + k + k * k / (1.0 + cs)
}

/// Per-vertex tags (`arm`, `cav`, `blush`).
#[derive(Clone, Copy, Default)]
pub struct Tags {
    pub arm: f64,
    pub cav: f64,
    pub blush: f64,
}

/// A tag (or a displacement) shared by a ring or given per vertex.
pub enum Per<'a, T> {
    All(T),
    Each(&'a [T]),
}

impl<T: Copy> Per<'_, T> {
    fn at(&self, i: usize) -> T {
        match self {
            Per::All(v) => *v,
            Per::Each(v) => v[i],
        }
    }
}

pub struct RingTags<'a> {
    pub arm: Per<'a, f64>,
    pub cav: Per<'a, f64>,
    pub blush: Per<'a, f64>,
}

impl Default for RingTags<'_> {
    fn default() -> Self {
        RingTags { arm: Per::All(0.0), cav: Per::All(0.0), blush: Per::All(0.0) }
    }
}

#[derive(Default)]
pub struct Cage {
    pub p: Vec<V3>,
    pub d: Vec<V3>,
    pub w: Vec<Weights>,
    pub region: Vec<&'static str>,
    pub tags: Vec<Tags>,
    pub faces: Vec<Vec<usize>>,
    /// The shoulder cap's loop round the neck (body.rs sets it; the necklines are cut from it).
    pub neck_loop: Vec<usize>,
}

/// The cage's per-vertex arrays: targets, feature offsets, normalised dense weights, tags.
pub struct CageArrays {
    pub t: Vec<V3>,
    pub d: Vec<V3>,
    pub dense: Vec<Dense>,
    pub tags: Vec<Tags>,
}

impl Cage {
    pub fn new() -> Self {
        Cage::default()
    }

    /// A vertex: `p` on the smooth surface (fitted exactly), `disp` a feature offset added to the
    /// fitted cage as it is (a soft bump of about 0.45 x its size, no ringing round it).
    pub fn add(&mut self, p: V3, w: Weights, region: &'static str, disp: Option<V3>, tags: Tags) -> usize {
        let k = self.p.len();
        self.p.push(p);
        self.d.push(disp.unwrap_or_else(V3::zeros));
        self.w.push(w);
        self.region.push(region);
        self.tags.push(tags);
        k
    }

    /// Vertices for `pts`; `w`, `disp` and tags may be per vertex or shared.
    pub fn ring(&mut self, pts: &[V3], w: Per<'_, &Weights>, region: &'static str, disp: Option<&[V3]>, tags: RingTags<'_>) -> Vec<usize> {
        (0..pts.len())
            .map(|i| {
                let wi = match &w {
                    Per::All(v) => (*v).clone(),
                    Per::Each(v) => v[i].clone(),
                };
                let t = Tags { arm: tags.arm.at(i), cav: tags.cav.at(i), blush: tags.blush.at(i) };
                self.add(pts[i], wi, region, disp.map(|d| d[i]), t)
            })
            .collect()
    }

    pub fn face(&mut self, ids: &[usize]) {
        for (i, a) in ids.iter().enumerate() {
            assert!(!ids[i + 1..].contains(a), "{ids:?}");
        }
        self.faces.push(ids.to_vec());
    }

    /// Quads between rings a and b (same size, same direction, cyclic); `skip`: edges left open.
    pub fn band(&mut self, a: &[usize], b: &[usize], skip: &[usize]) {
        let n = a.len();
        for i in 0..n {
            if skip.contains(&i) {
                continue;
            }
            let j = (i + 1) % n;
            self.face(&[a[i], a[j], b[j], b[i]]);
        }
    }

    /// Faces between ring a and a larger ring b: a[i] meets b[match[i]] (increasing, cyclic).
    pub fn join(&mut self, a: &[usize], b: &[usize], m: &[usize]) {
        let (n, mm) = (a.len(), b.len());
        for i in 0..n {
            let j = (i + 1) % n;
            let (s, e) = (m[i], m[j]);
            let mut f = vec![a[i], a[j]];
            let mut k = e;
            loop {
                f.push(b[k]);
                if k == s {
                    break;
                }
                k = (k + mm - 1) % mm;
            }
            self.face(&f);
        }
    }

    /// Closes a 4-ring with one quad.
    pub fn cap(&mut self, ring: &[usize]) {
        self.face(ring);
    }

    pub fn arrays(&self) -> CageArrays {
        let dense = self
            .w
            .iter()
            .map(|w| {
                let mut d = [0.0; NB];
                for &(name, v) in w {
                    d[b(name)] += v;
                }
                let s = super::np::pairwise_sum(&d);
                d.map(|v| v / s)
            })
            .collect();
        CageArrays { t: self.p.clone(), d: self.d.clone(), dense, tags: self.tags.clone() }
    }
}
