//! The head: its cage (part of the body's, see cage.rs) and the head interface (`head_frame`):
//! where the skull, face, eyes, mouth, brows and hairline of a family at a life stage are. The body
//! builds the head from the cage here, the face puts its features on the frame and the hair fits
//! over it, so each can be rebuilt on its own.
//!
//! The head cage is 8 rings of 12 vertices (`ROWS`, columns at `COLS` degrees round the head,
//! 0 = the face, 90 = the resident's right) under a crown patch. Its vertices lie where rays from
//! the head's middle leave a smooth head (`head_shape`: a soft union of a cranium, cheeks-and-jaw
//! and chin ellipsoids, rounder on small children); an ear is pulled out of each side, and a small
//! nose, a soft brow and an eye socket round each eyeball (its front a few mm proud of the face;
//! the lids close over it) are offsets on single vertices. There is no mouth hole: the lips and the
//! mouth plate sit on the face.
//!
//! `HeadFrame::sdf` and `surface_z` are measured on the real (subdivided) head and neck without
//! the ears, so the face features and the hair sit on what is drawn; `ears` bound the ears.
//! `centre` / `radii` are a loose ellipsoid round the skull for code that wants a simple one.

use std::sync::Arc;

use super::cage::{Cage, Per, RingTags, Tags, Weights};
use super::kdtree::KdTree;
use super::np::{PI, clip, linspace_open, mean, median, radians, sign};
use super::rig::{b, joints, proportions};
use super::{Cache, V3, key, v3};

/// Columns (degrees round the head's vertical axis; 0 = the face, + = the resident's right, +x).
pub const COLS: [f64; 12] = [0.0, 14.0, 28.0, 50.0, 92.0, 122.0, 180.0, -122.0, -92.0, -50.0, -28.0, -14.0];
/// The socket vertex (and its mirror, 12 - 2).
const EYE_COL: usize = 2;
/// The ear leaves through the face between these columns, rows 3-4.
const EAR_COLS: (usize, usize) = (4, 5);
/// Rows from the jaw up: elevation (degrees) of the ray from the head's middle (`MIDDLE`) at the
/// front and at the back. The smooth head is a soft union of ellipsoids (`Shapes`): the cranium,
/// the cheeks and jaw, the chin; the rows' vertices are where the rays leave it. The features
/// (nose, brow, eye sockets, ears) are cage offsets on top (see cage.rs).
const ROWS: [(f64, f64); 8] = [(-52.0, -35.0), (-42.0, -20.0), (-20.0, -2.0), (-8.0, 10.0), (6.0, 22.0), (24.0, 36.0), (46.0, 52.0), (68.0, 70.0)];
/// (height above the chin, z), fractions of hh.
const MIDDLE: (f64, f64) = (0.40, 0.0);
/// How far the eye socket's vertex moves (the surface follows by about half).
const SOCKET: f64 = 2.2;

/// Per family: (centre, radii) of the cranium, the cheeks and jaw, the chin (fractions of hh from
/// the chin and the Head joint's z), and the blend width.
struct Shapes {
    cranium: (V3, V3),
    cheeks: (V3, V3),
    chin: (V3, V3),
    blend: f64,
}

fn shapes(family: &str) -> Shapes {
    if family == "male" {
        Shapes {
            cranium: (v3(0.0, 0.56, -0.02), v3(0.375, 0.44, 0.445)),
            cheeks: (v3(0.0, 0.32, 0.07), v3(0.335, 0.31, 0.33)),
            chin: (v3(0.0, 0.12, 0.21), v3(0.15, 0.11, 0.12)),
            blend: 0.14,
        }
    } else {
        Shapes {
            cranium: (v3(0.0, 0.56, -0.02), v3(0.37, 0.44, 0.44)),
            cheeks: (v3(0.0, 0.32, 0.07), v3(0.325, 0.30, 0.325)),
            chin: (v3(0.0, 0.13, 0.20), v3(0.13, 0.10, 0.11)),
            blend: 0.14,
        }
    }
}

/// How far each feature's cage vertex moves (fractions of hh; the surface follows by about half).
fn nose(family: &str) -> f64 {
    if family == "male" { 0.085 } else { 0.07 }
}

fn brow(family: &str) -> f64 {
    if family == "male" { 0.025 } else { 0.014 }
}

/// 0 for adult heads (1 / 6.5 of the height), 1 for babies (1 / 3.4).
pub fn youth(stage_hh: f64) -> f64 {
    clip((stage_hh - 1.0 / 6.5) / (1.0 / 3.4 - 1.0 / 6.5), 0.0, 1.0)
}

/// Approximate signed distance to an ellipsoid (exact on its surface).
fn ellipsoid_sd(p: V3, c: V3, r: V3) -> f64 {
    let q = (p - c).component_div(&r);
    let k0 = q.norm();
    let k1 = q.component_div(&r).norm();
    k0 * (k0 - 1.0) / k1.max(1e-9)
}

fn smin(a: f64, b: f64, k: f64) -> f64 {
    let h = clip(0.5 + 0.5 * (b - a) / k, 0.0, 1.0);
    b + (a - b) * h - k * h * (1.0 - h)
}

/// The smooth head (no features) as a signed distance function of points in hh units (origin: the
/// chin's height and the Head joint's z).
pub fn head_shape(family: &str, stage: &str) -> impl Fn(V3) -> f64 {
    let pr = proportions(family, stage);
    let yg = youth(pr.hh);
    let s = shapes(family);
    let (cc, mut cr) = s.cranium;
    let (mut kc, mut kr) = s.cheeks;
    let (nc, mut nr) = s.chin;
    // Small children: a bigger, rounder cranium, chubby cheeks low on a short face, a small chin.
    cr += v3(0.035, 0.0, 0.02) * yg;
    kc += v3(0.0, -0.03, -0.01) * yg;
    kr += v3(0.04, 0.0, 0.0) * yg;
    nr *= 1.0 - 0.3 * yg;
    if stage == "elder" {
        kr += v3(0.008, 0.0, 0.0);
    }
    let bl = s.blend;
    move |p: V3| {
        let d = smin(ellipsoid_sd(p, cc, cr), ellipsoid_sd(p, kc, kr), bl);
        smin(d, ellipsoid_sd(p, nc, nr), 0.09)
    }
}

/// Where a ray from inside (o) along unit direction d leaves the shape f (bisection).
fn cast(f: &impl Fn(V3) -> f64, o: V3, d: V3) -> V3 {
    let (mut lo, mut hi) = (0.0, 1.2);
    for _ in 0..40 {
        let mid = (lo + hi) / 2.0;
        if f(o + d * mid) < 0.0 {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    o + d * lo
}

/// Adds the head to `cage`; returns its jaw ring (12 ids, `COLS` order) for the neck.
pub fn head_cage(cage: &mut Cage, family: &str, stage: &str) -> Vec<usize> {
    let pr = proportions(family, stage);
    let j = joints(family, stage);
    let h = pr.h;
    let hh = pr.hh * h;
    let chin = h - hh;
    let yg = youth(pr.hh);
    let f = head_shape(family, stage);
    let th = COLS.map(radians);
    let s = th.map(|t| (1.0 - t.cos()) / 2.0);
    let o = v3(0.0, MIDDLE.0, MIDDLE.1);
    let eyes = [j[b("eye_l")], j[b("eye_r")]];
    let eye_r = (0.075 + 0.02 * yg) * hh;
    let w: Weights = vec![("Head", 1.0)];
    let origin = v3(0.0, chin, 0.0);
    // (8, 12) the smooth head.
    let grid: Vec<Vec<V3>> = ROWS
        .iter()
        .map(|&(ef, eb)| {
            (0..12)
                .map(|c| {
                    let el = radians(ef + (eb - ef) * s[c]);
                    let d = v3(el.cos() * th[c].sin(), el.sin(), el.cos() * th[c].cos());
                    cast(&f, o, d) * hh + origin
                })
                .collect()
        })
        .collect();
    let mut dd = vec![vec![V3::zeros(); 12]; 8];
    let z = v3(0.0, 0.0, 1.0);
    let young = 1.0 - 0.55 * yg;
    // A small round nose (tip, base, wings, bridge) and a soft brow.
    let n = nose(family) * young * hh * (if stage == "elder" { 1.1 } else { 1.0 });
    dd[3][0] += v3(0.0, 0.05 * n, n);
    dd[2][0] += z * 0.3 * n;
    for c in [1, 11] {
        dd[3][c] += z * 0.06 * n;
    }
    dd[4][0] += z * 0.15 * n;
    let bw = brow(family) * (1.0 - 0.6 * yg) * hh;
    for c in [1, 2, 3, 9, 10, 11] {
        dd[5][c] += z * bw;
    }
    dd[5][0] += z * 0.5 * bw;
    // Eye sockets: the socket vertex pulls the face in behind the eyeball (its front stays proud).
    for (k, cc) in [(0, 12 - EYE_COL), (1, EYE_COL)] {
        let e = eyes[k];
        let sx = sign(e[0]);
        dd[4][cc] = (e + v3(-sx * 0.3 * eye_r, 0.0, 0.16 * eye_r) - grid[4][cc]) * SOCKET;
        let inner = (cc as i64 - sx as i64).rem_euclid(12) as usize;
        dd[4][inner] += v3(-sx * 0.1 * eye_r, 0.0, -0.45 * eye_r);
        dd[3][cc] += v3(0.0, -0.1 * eye_r, -0.15 * eye_r);
    }
    let mut ids = Vec::new();
    for r in 0..ROWS.len() {
        let mut cav = [0.0; 12];
        let mut blush = [0.0; 12];
        // (Tone is the base stage's for every stage; the shader deepens cavities with age.)
        if r == 3 {
            blush[2] = 0.32 + 0.2 * yg;
            blush[10] = blush[2];
            blush[3] = 0.2 + 0.15 * yg;
            blush[9] = blush[3];
            blush[0] = 0.15;
            // Under the eyes.
            cav[2] = 0.08;
            cav[10] = 0.08;
        }
        if r == 2 {
            // Beside the nose; cheek folds.
            cav[1] = 0.15;
            cav[11] = 0.15;
            cav[2] = 0.1;
            cav[10] = 0.1;
        }
        let tags = RingTags { cav: Per::Each(&cav), blush: Per::Each(&blush), ..Default::default() };
        ids.push(cage.ring(&grid[r], Per::All(&w), "head", Some(&dd[r]), tags));
    }
    for r in 0..ROWS.len() - 1 {
        let skip: Vec<usize> = if r == 3 { vec![EAR_COLS.0, 12 - EAR_COLS.1] } else { vec![] };
        let (a, bb) = (ids[r].clone(), ids[r + 1].clone());
        cage.band(&a, &bb, &skip);
    }
    // Ears: from the face left open between columns 92 and 122 (and -122 .. -92), rows 3-4. The
    // smooth head has a low dome there (the skull the hair fits over); the ear is the offset.
    for (c0, c1) in [(EAR_COLS.0, EAR_COLS.1), (12 - EAR_COLS.1, 12 - EAR_COLS.0)] {
        let lp = [ids[3][c0], ids[3][c1], ids[4][c1], ids[4][c0]];
        let p: Vec<V3> = lp.iter().map(|&i| cage.p[i]).collect();
        let m = super::np::mean_rows(&p);
        let sx = sign(m[0]);
        let out = v3(sx, 0.0, 0.0);
        let (mut base, mut offs) = (Vec::new(), Vec::new());
        for (i, &q0) in p.iter().enumerate() {
            let top = i >= 2;
            let front = if sx > 0.0 { i == 0 || i == 3 } else { i == 1 || i == 2 };
            let bpt = m + (q0 - m) * 0.6 + out * 0.012 * hh;
            let mut q = m + (q0 - m).component_mul(&v3(0.6, 1.0, 0.55));
            q += out * (0.04 + 0.015 * yg) * hh * (if front { 0.7 } else { 1.0 });
            q[1] += (if top { 0.085 } else { -0.06 }) * hh * (1.0 - 0.3 * yg);
            q[2] -= 0.015 * hh;
            base.push(bpt);
            offs.push(q - bpt);
        }
        let tags = RingTags { blush: Per::All(0.35), cav: Per::All(0.25), ..Default::default() };
        let e = cage.ring(&base, Per::All(&w), "ear", Some(&offs), tags);
        cage.band(&lp, &e, &[]);
        cage.cap(&e);
    }
    // Crown: a 2 x 4 patch over row 7 (right edge columns 14..130, left edge -14..-130), its middle
    // line on the head's top.
    let top = ids[ROWS.len() - 1].clone();
    let mut mid = Vec::new();
    for (el, az) in [(80.0, 0.0), (90.0, 0.0), (80.0, PI)] {
        let e = radians(el);
        let d = v3(f64::cos(e) * f64::sin(az), e.sin(), e.cos() * f64::cos(az));
        mid.push(cage.add(cast(&f, o, d) * hh + origin, w.clone(), "head", None, Tags::default()));
    }
    let right = [top[1], top[2], top[3], top[4], top[5]];
    let left = [top[11], top[10], top[9], top[8], top[7]];
    let centre = [top[0], mid[0], mid[1], mid[2], top[6]];
    for jj in 0..4 {
        cage.face(&[left[jj], centre[jj], centre[jj + 1], left[jj + 1]]);
        cage.face(&[centre[jj], right[jj], right[jj + 1], centre[jj + 1]]);
    }
    ids[0].clone()
}

/// The head of a family at a life stage (model space at rest, metres).
pub struct HeadFrame {
    pub family: String,
    pub stage: String,
    /// Head height (chin to crown), chin height and the crown.
    pub hh: f64,
    pub chin_y: f64,
    pub top_y: f64,
    /// A loose ellipsoid round the skull: centre and semi-axes (x, y, z). It encloses the cranium
    /// with a margin (more front to back, clearing the brow and chin); fit close things to `sdf`.
    pub centre: V3,
    pub radii: V3,
    /// Face plane: z of the face surface at eye level (round the eye sockets).
    pub face_z: f64,
    /// Eyeball centres (left at -x, then right) and radius.
    pub eyes: [V3; 2],
    pub eye_r: f64,
    /// Iris and pupil radius (m, on the eye's front).
    pub iris_r: f64,
    pub pupil_r: f64,
    /// Mouth: centre of the lip line on the surface, and its width (corner to corner).
    pub mouth: V3,
    pub mouth_w: f64,
    /// Brows: per side (left, right), inner and outer end points on the surface.
    pub brows: [[V3; 2]; 2],
    /// Hairline: a closed loop of points on the skull (front, temples, behind the ears, nape).
    pub hairline: Vec<V3>,
    pub chin: V3,
    /// Ears (left, right): centre and half extents (x, y, z) of a box round each. They stand off
    /// the skull and are not part of `sdf` / `surface_z`; hair should go round or over them.
    pub ears: [[V3; 2]; 2],
    /// The real head and neck surface (dense samples, their normals, and its triangles).
    pub samples: Vec<V3>,
    pub normals: Vec<V3>,
    pub tris: Vec<[V3; 3]>,
    /// The samples' KD-tree, their typical spacing and lowest height (head.py's module caches).
    tree: KdTree,
    spacing: f64,
    floor_y: f64,
}

impl HeadFrame {
    /// Signed distance (m; < 0 inside) of a point to the head and neck as drawn: the nearest
    /// samples' tangent-plane distances blended (moving least squares, Gaussian weights about
    /// the nearest one), so the field is continuous wherever it is evaluated (the ears are not
    /// part of the skull, see `ears`).
    pub fn sdf(&self, p: V3) -> f64 {
        let nb = self.tree.query(&p, 16);
        let sigma = 1.5 * self.spacing;
        let d0 = nb[0].0;
        let mut ws = [0.0; 16];
        let mut wv = [0.0; 16];
        for (j, &(d, i)) in nb.iter().enumerate() {
            let q = p - self.samples[i];
            let n = self.normals[i];
            let side = q.x * n.x + q.y * n.y + q.z * n.z;
            let w = (-(d * d - d0 * d0) / (sigma * sigma)).exp();
            ws[j] = w;
            wv[j] = w * side;
        }
        let mut s = super::np::pairwise_sum(&wv[..nb.len()]) / super::np::pairwise_sum(&ws[..nb.len()]);
        // Farther off, the distance to the nearest sample, with that sign (blended, so still smooth).
        let t = clip((d0 - sigma) / (2.0 * sigma), 0.0, 1.0) * clip(s.abs() / sigma, 0.0, 1.0);
        let t = t * t * (3.0 - 2.0 * t);
        s += t * (sign(s) * d0 - s);
        // The samples stop at the base of the neck: below that is outside.
        s.max(self.floor_y - p.y)
    }

    /// z of the head's front surface at (x, y) (the skull centre's z where (x, y) misses it).
    pub fn surface_z(&self, x: f64, y: f64) -> f64 {
        front_z(&self.tris, x, y, self.centre[2])
    }

    /// Scalp weight (0..1) of a point: 1 well above the hairline, 0 on the face and neck.
    pub fn scalp(&self, p: V3) -> f64 {
        clip((p.y - hairline_y(self, p)) / (0.06 * self.hh) + 0.5, 0.0, 1.0)
    }
}

/// Height of the hairline above a point, by its direction around the head's axis.
pub fn hairline_y(h: &HeadFrame, p: V3) -> f64 {
    // 0 = front, pi = back.
    let a = (p.x - h.centre[0]).atan2(p.z - h.centre[2]);
    let t = a.abs() / PI;
    let (front, side, back) = (h.chin_y + 0.74 * h.hh, h.chin_y + 0.56 * h.hh, h.chin_y + 0.18 * h.hh);
    if t < 0.5 { front + (side - front) * (t / 0.5).powf(1.5) } else { side + (back - side) * ((t - 0.5) / 0.5) }
}

/// The largest z where the vertical line (x, y) crosses triangles; `miss` where none does.
pub fn front_z(tris: &[[V3; 3]], x: f64, y: f64, miss: f64) -> f64 {
    let mut out = f64::NEG_INFINITY;
    for [a, b, c] in tris {
        let lo = (a.x.min(b.x).min(c.x), a.y.min(b.y).min(c.y));
        let hi = (a.x.max(b.x).max(c.x), a.y.max(b.y).max(c.y));
        if !(lo.0 <= x && x <= hi.0 && lo.1 <= y && y <= hi.1) {
            continue;
        }
        let v0 = (b.x - a.x, b.y - a.y);
        let v1 = (c.x - a.x, c.y - a.y);
        let den = v0.0 * v1.1 - v1.0 * v0.1;
        if den.abs() <= 1e-14 {
            continue;
        }
        let v2 = (x - a.x, y - a.y);
        let u = (v2.0 * v1.1 - v1.0 * v2.1) / den;
        let v = (v0.0 * v2.1 - v2.0 * v0.1) / den;
        if u >= -1e-9 && v >= -1e-9 && u + v <= 1.0 + 1e-9 {
            let z = a.z + u * (b.z - a.z) + v * (c.z - a.z);
            out = out.max(z);
        }
    }
    if out.is_finite() { out } else { miss }
}

static HEADS: Cache<(String, String), HeadFrame> = Cache::new();

/// The head of `family` at life stage `stage` (see `HeadFrame`).
pub fn head_frame(family: &str, stage: &str) -> Arc<HeadFrame> {
    HEADS.get(key(family, stage), || make_head_frame(family, stage))
}

fn make_head_frame(family: &str, stage: &str) -> HeadFrame {
    // The head is drawn as part of the body.
    let srf = super::body::surface(family, stage);
    let pr = proportions(family, stage);
    let j = joints(family, stage);
    let (hgt, hh) = (pr.h, pr.hh * pr.h);
    let chin_y = hgt - hh;
    let yg = youth(pr.hh);
    let eyes = [j[b("eye_l")], j[b("eye_r")]];
    let eye_r = (0.075 + 0.02 * yg) * hh;
    // The cranium's ellipsoid: round the head above the ears (ears and face excluded).
    let cran = &srf.cranium;
    let zmin = cran.iter().map(|p| p.z).fold(f64::INFINITY, f64::min);
    let zmax = cran.iter().map(|p| p.z).fold(f64::NEG_INFINITY, f64::max);
    let xmax = cran.iter().map(|p| p.x.abs()).fold(f64::NEG_INFINITY, f64::max);
    let centre = v3(0.0, chin_y + 0.56 * hh, (zmin + zmax) / 2.0);
    let radii = v3(xmax, hgt - centre[1], (zmax - zmin) / 2.0);
    let k = cran.iter().map(|p| (p - centre).component_div(&radii).norm()).fold(f64::NEG_INFINITY, f64::max);
    let radii = (radii * k.max(1.0)).component_mul(&v3(1.05, 1.03, 1.12));
    let samples = srf.head_samples.clone();
    let tree = KdTree::new(&samples);
    // Typical distance between neighbouring surface samples.
    let second: Vec<f64> = samples.iter().map(|p| tree.query(p, 2)[1].0).collect();
    let spacing = median(&second);
    let floor_y = samples.iter().map(|p| p.y).fold(f64::INFINITY, f64::min);
    let mut h = HeadFrame {
        family: family.to_string(),
        stage: stage.to_string(),
        hh,
        chin_y,
        top_y: hgt,
        centre,
        radii,
        face_z: 0.0,
        eyes,
        eye_r,
        iris_r: eye_r * 0.5,
        pupil_r: eye_r * 0.2,
        mouth: V3::zeros(),
        mouth_w: 0.2 * hh,
        brows: [[V3::zeros(); 2]; 2],
        hairline: Vec::new(),
        chin: V3::zeros(),
        ears: srf.ears,
        samples,
        normals: srf.head_normals.clone(),
        tris: srf.head_tris.clone(),
        tree,
        spacing,
        floor_y,
    };
    let a = linspace_open(0.0, 2.0 * PI, 8);
    let rim: Vec<f64> = a.iter().map(|&a| {
        let p = eyes[1] + v3(a.cos(), a.sin(), 0.0) * 1.45 * eye_r;
        h.surface_z(p.x, p.y)
    }).collect();
    h.face_z = mean(&rim);
    let my = chin_y + 0.2 * hh;
    h.mouth = v3(0.0, my, h.surface_z(0.0, my));
    let by = chin_y + 0.6 * hh;
    for (k, sx) in [-1.0, 1.0].into_iter().enumerate() {
        let inner = v3(sx * 0.07 * hh, by, 0.0);
        let outer = v3(sx * 0.3 * hh, by - 0.02 * hh, 0.0);
        for (e, mut pt) in [inner, outer].into_iter().enumerate() {
            pt[2] = h.surface_z(pt[0], pt[1]);
            h.brows[k][e] = pt;
        }
    }
    // Hairline loop: on the surface, at the hairline's height in each direction round the head.
    let a = linspace_open(-PI, PI, 48);
    let mut pts = Vec::new();
    for &a in &a {
        let d = v3(a.sin(), 0.0, a.cos());
        let probe = centre + d.component_mul(&radii);
        let y = hairline_y(&h, probe);
        pts.push(ray(&h, v3(centre[0], y, centre[2]), d));
    }
    h.hairline = pts;
    h.chin = ray(&h, v3(0.0, chin_y + 0.03 * hh, centre[2]), v3(0.0, -0.45, 1.0) / 0.45f64.hypot(1.0));
    h
}

/// Where a ray from inside the head leaves its surface (bisection on the sdf).
fn ray(h: &HeadFrame, o: V3, d: V3) -> V3 {
    let (mut lo, mut hi) = (0.0, 0.5 * h.top_y);
    for _ in 0..40 {
        let mid = (lo + hi) / 2.0;
        if h.sdf(o + d * mid) < 0.0 {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    o + d * lo
}
