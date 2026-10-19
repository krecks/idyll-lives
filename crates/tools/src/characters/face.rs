//! The residents' faces: small feature shapes on a head that never deforms. Contract:
//!
//! - `build_face(family, stage, head)`: parts `eyes`, `lashes`, `brows`, `teeth`, `tongue` (pos,
//!   nor, idx, dense, fields, tone); `body`, the pieces merged into `body` (lids, lips), skinned
//!   to Head, with their tone; `morphs` per part and channel (dpos, dnor) (`body` indexes the
//!   merged pieces only); `follow`, the lip lines and their morphs (the beard copies them).
//!
//! - Eyes: a sphere per eye on its eye bone, its pole to the front so the rings run round the iris.
//!   Fields xy = offset from the iris centre on the eye's front (m), z = iris radius, w = pupil
//!   radius; the shader draws sclera, iris (the resident's eye colour), pupil and a catchlight.
//! - Lids: upper and lower skin caps hugging the eyeball, their edges almond-shaped curves on it
//!   with a rounded rim, shaded like the face round them (normals leaning to it; build.rs gives
//!   them the tone of the skin they lie on); lashes a dark band along the upper rim (a flick at the
//!   outer corner on women). Blinks meet a little below the middle; squints raise the lower lid's
//!   middle most.
//! - Brows: flattened capsules along the brow line, on the skin.
//! - Mouth: upper and lower lip tubes meeting at the corners (tone.x tints them); between them a
//!   dark plate (tone.y = 1) with the tongue on it, and the upper and lower teeth. Everything inside
//!   the mouth stays between the lips' inner lines, so a closed mouth shows nothing of it.
//!
//! Every piece is made by one function of the face's parameters (`pieces`), with a fixed vertex
//! layout; a channel is the pieces made with its parameters minus the pieces at rest, so units and
//! visemes are authored as shapes (`unit_params`, `VISEME_PARAMS`), not as sums of deltas. Units
//! are at ARKit strength (face.ts uses them unscaled). Morphs are built at the base stage only and
//! shared by the other stages (the runtime adds them to every stage).

use std::collections::BTreeMap;
use std::sync::Arc;

use nalgebra::{DMatrix, DVector};

use super::contract::channels;
use super::head::HeadFrame;
use super::kdtree::KdTree;
use super::meshops::smooth_normals;
use super::np::{PI, clip, interp, linspace, lstsq, radians, sign, smoothstep};
use super::prims::{caps, ellipsoid, grid, loft, orient, rot, rot_x, OPEN};
use super::rig::BASE_STAGE;
use super::{Mesh, V3, merge, skin, v3};

/// The adult head height the millimetres below are measured on; other heads scale them.
const ADULT_HH: f64 = 0.2723;

/// Per family: lip half-heights (upper, lower) and depth (mm), how far the lips stand off the skin,
/// brow thickness, arch and taper, lash length and outer-corner flick, the lids' resting edges.
struct Style {
    lip_u: f64,
    lip_l: f64,
    lip_z: f64,
    proud: f64,
    brow_h: f64,
    brow_z: f64,
    arch: f64,
    brow_tip: f64,
    lash: f64,
    flick: f64,
    upper: f64,
    lower: f64,
    tilt: f64,
}

fn style(family: &str) -> Style {
    if family == "male" {
        Style { lip_u: 2.4, lip_l: 3.2, lip_z: 2.5, proud: 1.6, brow_h: 3.2, brow_z: 1.4, arch: 1.0, brow_tip: 0.5, lash: 1.2, flick: 0.3, upper: 63.0, lower: 121.0, tilt: 0.0 }
    } else {
        Style { lip_u: 2.9, lip_l: 3.8, lip_z: 2.7, proud: 1.8, brow_h: 2.5, brow_z: 1.1, arch: 2.8, brow_tip: 0.35, lash: 2.0, flick: 1.6, upper: 61.0, lower: 120.0, tilt: -3.0 }
    }
}

/// Lid columns: azimuth round the eye (degrees, 0 = front, + towards the outer corner).
fn lid_theta() -> Vec<f64> {
    linspace(-104.0, 104.0, 9)
}

fn lash_theta() -> Vec<f64> {
    linspace(-62.0, 80.0, 9)
}

/// Lip columns along the mouth (-1 = the resident's left corner), denser at the corners.
fn lip_s() -> Vec<f64> {
    linspace(-PI / 2.0, PI / 2.0, 9).iter().map(|v| v.sin()).collect()
}

fn teeth_s() -> Vec<f64> {
    linspace(-0.62, 0.62, 9)
}

fn tongue_s() -> Vec<f64> {
    linspace(-0.6, 0.6, 7)
}

/// How far the lower lip drops with the jaw fully open (mm).
const JAW: f64 = 26.0;

/// Face parameters (all 0 at rest): per side pairs [left, right] or scalars (see `pieces`).
#[derive(Clone, Copy, Default)]
pub struct Params {
    blink: [f64; 2],
    wide: [f64; 2],
    squint: [f64; 2],
    cheek: [f64; 2],
    look_down: [f64; 2],
    look_up: [f64; 2],
    brow_down: [f64; 2],
    brow_inner: f64,
    brow_outer: [f64; 2],
    sneer: [f64; 2],
    smile: [f64; 2],
    frown: [f64; 2],
    dimple: [f64; 2],
    stretch: [f64; 2],
    press: [f64; 2],
    low_down: [f64; 2],
    up_up: [f64; 2],
    jaw: f64,
    jaw_fwd: f64,
    close: f64,
    funnel: f64,
    pucker: f64,
    left: f64,
    roll_l: f64,
    roll_u: f64,
    shrug_l: f64,
    shrug_u: f64,
    tuck: f64,
    tongue: f64,
}

impl Params {
    /// Sets parameter `name` (face.py's names); `side` None sets both sides of a pair.
    fn set(&mut self, name: &str, v: f64, side: Option<usize>) {
        let pair = |p: &mut [f64; 2]| match side {
            Some(s) => p[s] = v,
            None => *p = [v, v],
        };
        match name {
            "blink" => pair(&mut self.blink),
            "wide" => pair(&mut self.wide),
            "squint" => pair(&mut self.squint),
            "cheek" => pair(&mut self.cheek),
            "lookDown" => pair(&mut self.look_down),
            "lookUp" => pair(&mut self.look_up),
            "browDown" => pair(&mut self.brow_down),
            "browOuter" => pair(&mut self.brow_outer),
            "sneer" => pair(&mut self.sneer),
            "smile" => pair(&mut self.smile),
            "frown" => pair(&mut self.frown),
            "dimple" => pair(&mut self.dimple),
            "stretch" => pair(&mut self.stretch),
            "press" => pair(&mut self.press),
            "lowDown" => pair(&mut self.low_down),
            "upUp" => pair(&mut self.up_up),
            "browInner" => self.brow_inner = v,
            "jaw" => self.jaw = v,
            "jawFwd" => self.jaw_fwd = v,
            "close" => self.close = v,
            "funnel" => self.funnel = v,
            "pucker" => self.pucker = v,
            "left" => self.left = v,
            "rollL" => self.roll_l = v,
            "rollU" => self.roll_u = v,
            "shrugL" => self.shrug_l = v,
            "shrugU" => self.shrug_u = v,
            "tuck" => self.tuck = v,
            "tongue" => self.tongue = v,
            _ => panic!("no face parameter {name}"),
        }
    }

    fn of(kw: &[(&str, f64)]) -> Params {
        let mut p = Params::default();
        for &(k, v) in kw {
            p.set(k, v, None);
        }
        p
    }
}

/// The parameters of ARKit unit `c`.
fn unit_params(c: &str) -> Params {
    let (name, side) = match ["Left", "Right"].iter().enumerate().find(|(_, s)| c.ends_with(*s) && c != "mouthLeft") {
        Some((k, s)) => (&c[..c.len() - s.len()], Some(k)),
        None => (c, None),
    };
    let key = match name {
        "eyeBlink" => "blink",
        "eyeWide" => "wide",
        "eyeSquint" => "squint",
        "cheekSquint" => "cheek",
        "eyeLookDown" => "lookDown",
        "eyeLookUp" => "lookUp",
        "browDown" => "browDown",
        "browInnerUp" => "browInner",
        "browOuterUp" => "browOuter",
        "noseSneer" => "sneer",
        "mouthSmile" => "smile",
        "mouthFrown" => "frown",
        "mouthDimple" => "dimple",
        "mouthStretch" => "stretch",
        "mouthPress" => "press",
        "mouthLowerDown" => "lowDown",
        "mouthUpperUp" => "upUp",
        "jawOpen" => "jaw",
        "jawForward" => "jawFwd",
        "mouthClose" => "close",
        "mouthFunnel" => "funnel",
        "mouthPucker" => "pucker",
        "mouthLeft" => "left",
        "mouthRollLower" => "rollL",
        "mouthRollUpper" => "rollU",
        "mouthShrugLower" => "shrugL",
        "mouthShrugUpper" => "shrugU",
        other => panic!("no unit {other}"),
    };
    let mut p = Params::default();
    match side {
        None => p.set(key, 1.0, None),
        Some(s) => {
            p.set(key, 0.0, None);
            p.set(key, 1.0, Some(s));
        }
    }
    p
}

/// Visemes (the voice engine's order) as mouth shapes. Clear, slightly exaggerated shapes read at
/// a distance: open vowels drop the jaw, the wide ones show the teeth, the round ones push the
/// lips out.
fn viseme_params(c: &str) -> Option<&'static [(&'static str, f64)]> {
    Some(match c {
        "sil" => &[],
        "PP" => &[("press", 1.0), ("rollL", 0.25), ("rollU", 0.25), ("shrugL", 0.3)],
        "FF" => &[("jaw", 0.07), ("tuck", 1.0), ("upUp", 0.35)],
        "TH" => &[("jaw", 0.16), ("tongue", 1.0), ("upUp", 0.15)],
        "DD" => &[("jaw", 0.22), ("upUp", 0.25), ("lowDown", 0.15)],
        "kk" => &[("jaw", 0.3), ("upUp", 0.1)],
        "CH" => &[("jaw", 0.12), ("funnel", 0.65), ("upUp", 0.4), ("lowDown", 0.4)],
        "SS" => &[("jaw", 0.05), ("stretch", 0.5), ("upUp", 0.45), ("lowDown", 0.45)],
        "nn" => &[("jaw", 0.16), ("upUp", 0.15)],
        "RR" => &[("jaw", 0.17), ("funnel", 0.45)],
        "aa" => &[("jaw", 0.62), ("upUp", 0.1)],
        "E" => &[("jaw", 0.34), ("stretch", 0.45), ("upUp", 0.25)],
        "I" => &[("jaw", 0.2), ("stretch", 0.6), ("upUp", 0.35), ("lowDown", 0.25)],
        "O" => &[("jaw", 0.45), ("funnel", 0.85)],
        "U" => &[("jaw", 0.16), ("pucker", 0.95)],
        _ => return None,
    })
}

/// A lip line, its half-height and depth per column.
type Lip = (Vec<V3>, Vec<f64>, Vec<f64>);

/// One face (a family at a stage): where its features are, and how they are shaped.
pub struct Face {
    h: Arc<HeadFrame>,
    st: Style,
    mm: f64,
    hw: f64,
    m0: (f64, f64),
    mcoef: Vec<f64>,
    mfloor: f64,
}

impl Face {
    fn new(family: &str, head: Arc<HeadFrame>) -> Self {
        let mm = 0.001 * head.hh / ADULT_HH;
        let hw = head.mouth_w / 2.0 * 0.92;
        let mut f = Face { h: head, st: style(family), mm, hw, m0: (0.0, 0.0), mcoef: Vec::new(), mfloor: 0.0 };
        f.fit_mouth();
        f
    }

    // --- the skin ------------------------------------------------------------------------------

    /// z of the face's skin at (x, y) (refined on the head's distance field).
    fn surf(&self, x: f64, y: f64) -> f64 {
        let h = &self.h;
        let mut p = v3(x, y, h.surface_z(x, y));
        for _ in 0..4 {
            let d = h.sdf(p);
            let dz = (h.sdf(p + v3(0.0, 0.0, 1e-4)) - d) / 1e-4;
            p[2] -= d / dz.max(0.3);
        }
        p[2]
    }

    /// A point moved onto the head's surface along the skull ellipsoid's normal (smooth, whatever
    /// the head's tessellation), then `lift` (m) out along it.
    fn project(&self, p: V3, lift: f64) -> V3 {
        let h = &self.h;
        let n = (p - h.centre).component_div(&h.radii.component_mul(&h.radii));
        let n = n / n.norm();
        let (mut lo, mut hi) = (-0.04, 0.04);
        for _ in 0..30 {
            let mid = (lo + hi) / 2.0;
            if h.sdf(p + n * mid) < 0.0 {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        p + n * ((lo + hi) / 2.0 + lift)
    }

    /// A smooth stand-in for the skin round the mouth (a quadratic in x, y fitted to it), raised
    /// so the real skin is nowhere in front of it: the mouth's pieces are placed on it, so they are
    /// smooth however the head is tessellated and never sink into it.
    fn fit_mouth(&mut self) {
        let (mm, hw) = (self.mm, self.hw);
        let my = self.h.mouth[1];
        let gx: Vec<f64> = linspace(-1.35, 1.35, 15).iter().map(|v| v * hw).collect();
        let gy: Vec<f64> = linspace(-34.0, 12.0, 14).iter().map(|v| my + v * mm).collect();
        self.m0 = (self.h.mouth[0], my);
        let mut z = Vec::new();
        let mut rows = Vec::new();
        for &y in &gy {
            for &x in &gx {
                z.push(self.surf(x, y));
                rows.push(self.feat(x, y));
            }
        }
        let f = DMatrix::from_fn(rows.len(), 7, |i, k| rows[i][k]);
        self.mcoef = lstsq(&f, &DVector::from_vec(z.clone())).iter().copied().collect();
        let worst = rows.iter().zip(&z).map(|(r, z)| z - dot7(r, &self.mcoef)).fold(f64::NEG_INFINITY, f64::max);
        self.mfloor = worst.max(0.0);
    }

    fn feat(&self, x: f64, y: f64) -> [f64; 7] {
        let u = (x - self.m0.0) / self.hw;
        let v = (y - self.m0.1) / self.hw;
        [1.0, u * u, u.powf(4.0), v, v * v, v.powf(3.0), u * u * v]
    }

    /// z of the smoothed skin round the mouth.
    fn mz(&self, x: f64, y: f64) -> f64 {
        dot7(&self.feat(x, y), &self.mcoef) + self.mfloor
    }

    // --- eyes ----------------------------------------------------------------------------------

    fn eyeballs(&self) -> Mesh {
        let h = &self.h;
        let mut out = Vec::new();
        for (k, side) in ["l", "r"].into_iter().enumerate() {
            let c = h.eyes[k];
            let m = ellipsoid(c, V3::repeat(h.eye_r), 12, 16);
            let r = rot_x(PI / 2.0);
            let p: Vec<V3> = m.pos.iter().map(|q| rot(&r, q - c) + c).collect();
            let fields = p
                .iter()
                .map(|q| {
                    let front = q.z - c.z > 0.0;
                    [if front { q.x - c.x } else { 1.0 }, if front { q.y - c.y } else { 1.0 }, h.iris_r * 1.08, h.pupil_r * 1.1]
                })
                .collect();
            let nor = p.iter().map(|q| (q - c) / h.eye_r).collect();
            let n = p.len();
            out.push(Mesh { pos: p, idx: m.idx, nor, fields, dense: skin(n, &format!("eye_{side}")), ..Mesh::default() });
        }
        merge(&out.iter().collect::<Vec<_>>())
    }

    /// Upper and lower lid edges of eye k (polar angle from the top of the eye, degrees) per `lid_theta`.
    fn lid_edges(&self, p: &Params, k: usize) -> (Vec<f64>, Vec<f64>) {
        let st = &self.st;
        let lt = lid_theta();
        let mut uu = Vec::new();
        let mut ll = Vec::new();
        for &th in &lt {
            // + towards the outer corner.
            let t = th / 90.0;
            let to = t;
            let inner = smoothstep(0.1, -0.9, to);
            let u0 = st.upper + 27.0 * t * t + st.tilt * to;
            let l0 = st.lower - 21.0 * t * t + 2.0 * to;
            let mut u = u0 - 12.0 * p.wide[k] - 10.0 * p.look_up[k] + 20.0 * p.look_down[k] + p.brow_down[k] * (3.0 + 12.0 * inner) + 5.0 * p.sneer[k] - 5.0 * p.brow_inner * inner;
            let mut l = l0 - 18.0 * p.squint[k] * (1.0 - 0.45 * t * t) - 8.0 * p.cheek[k] * (1.0 - 0.3 * t * t) + 8.0 * p.look_down[k] + 3.0 * p.wide[k] - 7.0 * p.sneer[k];
            let b = p.blink[k];
            if b != 0.0 {
                let c = 0.52 * l0 + 0.48 * u0;
                u += b * (c + 4.0 - u0);
                l += b * (c - l0);
            }
            uu.push(u);
            ll.push(l);
        }
        (uu, ll)
    }

    /// A point on eye k's sphere frame: polar angle phi from +y, azimuth theta (deg, + outer), radius r.
    fn sph(&self, k: usize, phi: f64, theta: f64, r: f64) -> V3 {
        let c = self.h.eyes[k];
        let out = if k == 1 { 1.0 } else { -1.0 };
        let (ph, th) = (radians(phi), radians(theta) * out);
        c + v3(ph.sin() * th.sin(), ph.cos(), ph.sin() * th.cos()) * r
    }

    #[allow(clippy::type_complexity)]
    fn lids(&self, p: &Params, k: usize) -> (Vec<Vec<V3>>, Vec<Vec<V3>>, Vec<Vec<V3>>) {
        let (h, mm) = (&self.h, self.mm);
        let (u, l) = self.lid_edges(p, k);
        let r = h.eye_r;
        let (ru, rl) = (r * 1.105, r * 1.095);
        let lt = lid_theta();
        let n = lt.len();
        let row = |ph: &dyn Fn(usize) -> f64, rr: f64, th: &[f64]| -> Vec<V3> { (0..th.len()).map(|i| self.sph(k, ph(i), th[i], rr)).collect() };
        // Upper: from close over the top of the eye (so it rises out of the socket rather than
        // standing on the face) down to the edge, then the rim turning in to the eye.
        let up = vec![
            row(&|_| 6.0, r * 1.05, &lt),
            row(&|i| 6.0 + (u[i] - 6.0) * 0.45, r * 1.09, &lt),
            row(&|i| 6.0 + (u[i] - 6.0) * 0.78, r * 1.1, &lt),
            row(&|i| u[i], ru, &lt),
            row(&|i| u[i] + 3.0, r * 1.075, &lt),
        ];
        // Lower: from under the eye up to the edge, then its rim.
        let lo = vec![row(&|_| 168.0, rl, &lt), row(&|i| (168.0 + l[i]) / 2.0, rl, &lt), row(&|i| l[i], rl, &lt), row(&|i| l[i] - 2.5, r * 1.075, &lt)];
        debug_assert_eq!(n, 9);
        // Lashes: a dark band along the upper rim, standing out and up; longer towards the outer corner.
        let st = &self.st;
        let ls = lash_theta();
        let ue: Vec<f64> = ls.iter().map(|&t| interp(t, &lt, &u)).collect();
        let mut lash = vec![Vec::new(), Vec::new(), Vec::new()];
        for (i, &th) in ls.iter().enumerate() {
            let to = th / 90.0;
            let ln = (st.lash * (0.55 + 0.45 * smoothstep(-0.7, 0.6, to)) + st.flick * smoothstep(0.3, 0.95, to)) * mm;
            let lift = 5.0 + 6.0 * smoothstep(0.2, 0.95, to) * (if st.flick > 1.0 { 1.0 } else { 0.0 });
            lash[0].push(self.sph(k, ue[i] + 1.5, th, ru + 0.15 * mm));
            lash[1].push(self.sph(k, ue[i] - 1.0, th, ru + 0.55 * ln));
            lash[2].push(self.sph(k, ue[i] - lift * 0.6 - 1.0, th, ru + ln));
        }
        (up, lo, lash)
    }

    // --- brows ---------------------------------------------------------------------------------

    fn brow(&self, p: &Params, k: usize) -> Lip {
        let (h, mm, st) = (&self.h, self.mm, &self.st);
        let [a, b] = h.brows[k];
        let t = linspace(0.0, 1.0, 6);
        // Towards the middle: +x for the left brow.
        let inward = if k == 0 { 1.0 } else { -1.0 };
        let inner = 11.0 * p.brow_inner + 1.0 * p.brow_outer[k] - 8.0 * p.brow_down[k] - 3.0 * p.sneer[k];
        let mid = 5.0 * p.brow_inner + 4.5 * p.brow_outer[k] - 4.0 * p.brow_down[k] - 1.2 * p.sneer[k];
        let outer = 1.0 * p.brow_inner + 9.0 * p.brow_outer[k] - 1.0 * p.brow_down[k];
        let (mut pts, mut ry, mut rz) = (Vec::new(), Vec::new(), Vec::new());
        for &t in &t {
            let dy = interp(t, &[0.0, 0.45, 1.0], &[inner, mid, outer]) * mm + st.arch * mm * (PI * t.powf(0.8)).sin();
            let dx = (3.5 * p.brow_down[k] + 1.2 * p.sneer[k]) * mm * (1.0 - t).powf(1.5) * inward;
            let x = a[0] + (b[0] - a[0]) * t + dx;
            let y = a[1] + (b[1] - a[1]) * t + dy;
            ry.push(st.brow_h * mm * interp(t, &[0.0, 0.25, 0.7, 1.0], &[0.85, 1.0, 0.8, st.brow_tip]));
            let z_r = st.brow_z * mm * interp(t, &[0.0, 1.0], &[1.0, 0.7]);
            rz.push(z_r);
            let z = self.surf(x, y);
            pts.push(self.project(v3(x, y, z), z_r * 0.75));
        }
        (pts, ry, rz)
    }

    // --- mouth ---------------------------------------------------------------------------------

    /// Lip centre lines and radii, the inner plate, teeth and tongue (positions) for parameters p.
    #[allow(clippy::type_complexity)]
    fn mouth(&self, p: &Params) -> ([Lip; 2], Vec<Vec<V3>>, [Vec<Vec<V3>>; 2], Vec<Vec<V3>>) {
        let (h, mm, st, hw) = (&self.h, self.mm, &self.st, self.hw);
        let s = lip_s();
        let n = s.len();
        let col = |f: &dyn Fn(usize) -> f64| -> Vec<f64> { (0..n).map(f).collect() };
        let ml = col(&|i| smoothstep(0.35, -0.35, s[i]));
        let sv = |v: [f64; 2], i: usize| v[0] * ml[i] + v[1] * (1.0 - ml[i]);
        let cw = col(&|i| s[i] * s[i]);
        let cen = col(&|i| clip(1.0 - s[i] * s[i], 0.0, 1.0));
        let o = col(&|i| cen[i].powf(0.7));
        let rc = col(&|i| cen[i].sqrt());
        let x = col(&|i| {
            let (sm, fr, st_, dim) = (sv(p.smile, i), sv(p.frown, i), sv(p.stretch, i), sv(p.dimple, i));
            let x = s[i] * hw * (1.0 - 0.34 * p.funnel - 0.45 * p.pucker) + sign(s[i]) * s[i].abs().powf(1.5) * mm * (3.5 * sm + 4.0 * st_ + 1.5 * dim - 1.0 * fr);
            x - 4.0 * mm * p.left
        });
        let y = col(&|i| {
            let (sm, fr, st_, dim) = (sv(p.smile, i), sv(p.frown, i), sv(p.stretch, i), sv(p.dimple, i));
            h.mouth[1] + mm * (cw[i] * (0.6 + 6.5 * sm - 7.0 * fr - 1.8 * st_ + 0.8 * dim) + 2.2 * p.shrug_l * cen[i])
        });
        let dz = col(&|i| -mm * cw[i] * (2.0 * sv(p.dimple, i) + 1.0 * sv(p.smile, i)));
        let low = col(&|i| {
            let low = mm * (JAW * p.jaw * o[i] + 4.5 * sv(p.low_down, i) * o[i].powf(0.6) + 5.0 * p.funnel * rc[i] - 0.8 * JAW * p.close * o[i]);
            low - 1.2 * mm * p.tuck * o[i]
        });
        let up = col(&|i| {
            mm * (3.0 * p.jaw * o[i] + 4.5 * sv(p.up_up, i) * o[i].powf(0.6) + 1.5 * p.shrug_u * o[i] + 2.5 * sv(p.sneer, i) * o[i].sqrt() + 3.0 * p.funnel * rc[i] + 1.6 * p.tuck * o[i])
        });
        let taper = col(&|i| 0.22 + 0.78 * cen[i].powf(0.55));
        let ry_u = col(&|i| st.lip_u * mm * taper[i] * (1.0 - 0.4 * sv(p.press, i) - 0.55 * p.roll_u + 0.25 * p.pucker + 0.1 * p.funnel));
        let ry_l = col(&|i| st.lip_l * mm * taper[i] * (1.0 - 0.4 * sv(p.press, i) - 0.55 * p.roll_l + 0.25 * p.pucker + 0.1 * p.funnel + 0.15 * p.shrug_l - 0.3 * p.tuck));
        let rz = col(&|i| st.lip_z * mm * (0.35 + 0.65 * rc[i]) * (1.0 + 0.25 * p.pucker));
        let fwd = col(&|i| mm * rc[i] * (4.5 * p.funnel + 5.5 * p.pucker));
        let proud = col(&|i| st.proud * mm * (0.3 + 0.7 * rc[i]));
        let yu = col(&|i| y[i] + up[i] + 0.75 * ry_u[i]);
        let yl = col(&|i| y[i] - low[i] - 0.75 * ry_l[i]);
        let zu = col(&|i| self.mz(x[i], yu[i]) + proud[i] + fwd[i] + dz[i] - 1.5 * mm * p.roll_u * rc[i]);
        let zl = col(&|i| self.mz(x[i], yl[i]) + proud[i] + fwd[i] + dz[i] + mm * rc[i] * (3.0 * p.jaw_fwd - 2.5 * p.tuck - 1.5 * p.roll_l + 1.0 * p.shrug_l));
        let lips = [
            ((0..n).map(|i| v3(x[i], yu[i], zu[i])).collect(), ry_u.clone(), rz.clone()),
            ((0..n).map(|i| v3(x[i], yl[i], zl[i])).collect(), ry_l.clone(), rz),
        ];
        // The opening: between the lips' inner lines (a little inside each lip, so its edge hides).
        let up_in = col(&|i| y[i] + up[i] + 0.4 * ry_u[i]);
        let low_in = col(&|i| y[i] - low[i] - 0.4 * ry_l[i]);
        // Everything inside sits on the plate's depth (which comes forward with pursed lips).
        let pz = |xx: f64, yy: f64| self.mz(xx, yy) + 0.5 * mm + 0.4 * interp(xx, &x, &fwd);
        let plate = linspace(0.0, 1.0, 4)
            .iter()
            .map(|&v| {
                (0..n)
                    .map(|i| {
                        let py = low_in[i] + (up_in[i] - low_in[i]) * v;
                        v3(x[i], py, pz(x[i], py))
                    })
                    .collect()
            })
            .collect();
        let clamp = |xx: f64, yy: f64| yy.max(interp(xx, &x, &low_in)).min(interp(xx, &x, &up_in));
        // Upper teeth: fixed to the skull; lower teeth: on the jaw.
        let mx = h.mouth[1];
        let ts = teeth_s();
        let tx: Vec<f64> = ts.iter().map(|t| t * h.mouth_w / 2.0).collect();
        let ub: Vec<f64> = ts.iter().map(|t| mx - 0.8 * mm + 1.4 * mm * t * t).collect();
        let drop = 0.86 * JAW * p.jaw * mm;
        let lt: Vec<f64> = ts.iter().map(|t| mx - 1.4 * mm - drop + 0.6 * mm * t * t - 1.2 * mm * p.tuck).collect();
        let teeth_row = |ys: &dyn Fn(usize) -> f64, zf: f64| -> Vec<V3> {
            (0..ts.len())
                .map(|i| {
                    let yy = clamp(tx[i], ys(i));
                    v3(tx[i], yy, pz(tx[i], yy) + (1.5 - 0.8 * ts[i] * ts[i] + zf) * mm)
                })
                .collect()
        };
        let zf_l = 3.0 * p.jaw_fwd - 1.0 * p.tuck;
        let teeth = [
            vec![teeth_row(&|i| ub[i], 0.0), teeth_row(&|i| ub[i] + 7.0 * mm, 0.0)],
            vec![teeth_row(&|i| lt[i] - 6.0 * mm, zf_l), teeth_row(&|i| lt[i], zf_l)],
        ];
        // Tongue: on the floor of the mouth, rising and coming forward between the teeth for TH.
        let gs = tongue_s();
        let gx: Vec<f64> = gs.iter().map(|g| g * hw).collect();
        let bot: Vec<f64> = gx.iter().map(|&g| interp(g, &x, &low_in)).collect();
        let top: Vec<f64> = gs
            .iter()
            .zip(&gx)
            .map(|(g, &gxx)| {
                let q = g / 0.6;
                clamp(gxx, mx - mm * (2.5 + 0.6 * JAW * p.jaw) + mm * (9.0 * p.tongue) - 3.0 * mm * (q * q))
            })
            .collect();
        let tall: Vec<f64> = (0..gs.len()).map(|i| clip((top[i] - bot[i]) / (4.0 * mm), 0.0, 1.0)).collect();
        let tongue = [(0.0, 0.0), (0.55, 0.6), (1.0, 0.25)]
            .iter()
            .map(|&(f, bulge)| {
                (0..gs.len())
                    .map(|i| {
                        let yy = bot[i] + (top[i] - bot[i]) * f;
                        v3(gx[i], yy, pz(gx[i], yy) + (0.4 + (bulge + 2.2 * p.tongue * f) * tall[i]) * mm)
                    })
                    .collect()
            })
            .collect();
        (lips, plate, teeth, tongue)
    }
}

fn dot7(a: &[f64; 7], c: &[f64]) -> f64 {
    let mut s = 0.0;
    for k in 0..7 {
        s += a[k] * c[k];
    }
    s
}

/// Open ends: the corners are thin enough not to show them.
fn lip_tube(lip: &Lip) -> (Vec<V3>, Vec<usize>) {
    let m = loft(&lip.0, &lip.1, &lip.2, 6, v3(0.0, 1.0, 0.0), OPEN);
    (m.pos, m.idx)
}

type Pieces = BTreeMap<&'static str, (Vec<V3>, Vec<usize>)>;

/// Every morphing piece of the face for parameters p: name -> (pos, idx).
fn pieces(face: &Face, p: &Params) -> Pieces {
    let mut out: Pieces = BTreeMap::new();
    for k in 0..2 {
        let (up, lo, lash) = face.lids(p, k);
        let eye = face.h.eyes[k];
        for (name, g) in [(["lidU0", "lidU1"][k], up), (["lidL0", "lidL1"][k], lo), (["lash0", "lash1"][k], lash)] {
            let (pos, idx) = grid(&g, false);
            let outv: Vec<V3> = pos.iter().map(|q| q - eye).collect();
            let idx = orient(&pos, idx, &outv);
            out.insert(name, (pos, idx));
        }
        let b = face.brow(p, k);
        let m = loft(&b.0, &b.1, &b.2, 6, v3(0.0, 1.0, 0.0), caps(1));
        out.insert(["brow0", "brow1"][k], (m.pos, m.idx));
    }
    let (lips, plate, teeth, tongue) = face.mouth(p);
    out.insert("lipU", lip_tube(&lips[0]));
    out.insert("lipL", lip_tube(&lips[1]));
    let [tu, tl] = teeth;
    for (name, g) in [("plate", plate), ("teethU", tu), ("teethL", tl), ("tongue", tongue)] {
        let (pos, idx) = grid(&g, false);
        let fwd = vec![v3(0.0, 0.0, 1.0); pos.len()];
        let idx = orient(&pos, idx, &fwd);
        out.insert(name, (pos, idx));
    }
    out
}

/// Which pieces make which part (in order).
const PARTS: [(&str, &[&str]); 5] = [
    ("body", &["lidU0", "lidL0", "lidU1", "lidL1", "lipU", "lipL"]),
    ("lashes", &["lash0", "lash1"]),
    ("brows", &["brow0", "brow1"]),
    ("tongue", &["plate", "tongue"]),
    ("teeth", &["teethU", "teethL"]),
];

fn smooth_n(name: &str, pos: &[V3], idx: &[usize]) -> Vec<V3> {
    let n = smooth_normals(pos, idx);
    // The lids' shading normals lean towards the face's (they are skin round the eye, not a ball).
    let f = match &name[..4.min(name.len())] {
        "lidU" => Some(v3(0.0, 0.45, 1.0)),
        "lidL" => Some(v3(0.0, 0.35, 1.0)),
        _ => None,
    };
    match f {
        Some(f) => n
            .iter()
            .map(|q| {
                let m = q * 0.2 + f * 0.8 / f.norm();
                m / m.norm()
            })
            .collect(),
        None => n,
    }
}

/// Per piece tone: lips blushed (x), the mouth plate dark (y), the tongue red, teeth shaded at the sides.
fn tone(name: &str, pos: &[V3], idx: &[usize], face: &Face) -> Vec<[f64; 4]> {
    let n = pos.len();
    let mut t = vec![[0.0; 4]; n];
    if name == "lipU" || name == "lipL" {
        // The sides facing the other lip in shade: a dark line between the lips, so the mouth's
        // shape (a smile, a frown) reads even when it is a few pixels wide.
        let nn = smooth_normals(pos, idx);
        for i in 0..n {
            let s = clip((pos[i].x - face.h.mouth[0]).abs() / face.hw, 0.0, 1.0);
            t[i][0] = (if name == "lipU" { 0.75 } else { 0.85 }) * (1.0 - 0.35 * s.powf(3.0));
            let ny = nn[i].y * (if name == "lipU" { -1.0 } else { 1.0 });
            t[i][1] = 0.75 * clip(ny, 0.0, 1.0).powf(1.5);
        }
    } else if name == "plate" {
        for v in &mut t {
            v[1] = 1.0;
        }
    } else if name == "tongue" {
        let k = tongue_s().len();
        for (i, v) in t.iter_mut().enumerate() {
            v[0] = 1.0;
            v[1] = [0.45, 0.15, 0.05][i / k];
        }
    } else if name.starts_with("teeth") {
        for i in 0..n {
            let s = (pos[i].x - face.h.mouth[0]).abs() / face.hw;
            let c = clip(s / 0.62, 0.0, 1.0);
            t[i][1] = 0.05 + 0.5 * (c * c);
        }
    }
    t
}

/// Morph deltas of a part per channel: (dpos, dnor) per vertex.
pub type Deltas = BTreeMap<&'static str, (Vec<V3>, Vec<V3>)>;

/// One shape's deltas per part (`PARTS` order).
type PartDeltas = Vec<(&'static str, (Vec<V3>, Vec<V3>))>;

/// The lip lines (the beard follows them).
pub struct Follow {
    pub pos: Vec<V3>,
    pub upper: Vec<bool>,
    pub morphs: Vec<(&'static str, Vec<V3>)>,
}

pub struct FaceOut {
    /// eyes, lashes, brows, tongue, teeth (in that order).
    pub parts: Vec<(String, Mesh)>,
    /// The pieces merged into `body`: the lids (skin_tone) then the lips.
    pub body: Vec<Mesh>,
    /// Per part (in the order a channel first moves it): channel -> deltas.
    pub morphs: Vec<(String, Deltas)>,
    pub follow: Option<Follow>,
}

pub fn build_face(family: &str, stage: &str, head: Arc<HeadFrame>) -> FaceOut {
    let face = Face::new(family, head);
    let rest = pieces(&face, &Params::default());
    let mut parts = vec![("eyes".to_string(), face.eyeballs())];
    let mut body = Vec::new();
    for (part, names) in PARTS {
        let mut meshes = Vec::new();
        for &name in names {
            let (pos, idx) = &rest[name];
            let n = pos.len();
            meshes.push(Mesh {
                pos: pos.clone(),
                idx: idx.clone(),
                nor: smooth_n(name, pos, idx),
                dense: skin(n, "Head"),
                tone: tone(name, pos, idx, &face),
                fields: vec![[0.0; 4]; n],
                skin_tone: part == "body" && name.starts_with("lid"),
                ..Mesh::default()
            });
        }
        if part == "body" {
            body.extend(meshes);
        } else {
            parts.push((part.to_string(), merge(&meshes.iter().collect::<Vec<_>>())));
        }
    }
    if stage != BASE_STAGE {
        return FaceOut { parts, body, morphs: Vec::new(), follow: None };
    }

    // Morphs: each channel's pieces minus the rest.
    let rest_n: BTreeMap<&str, Vec<V3>> = rest.iter().map(|(&k, (p, i))| (k, smooth_n(k, p, i))).collect();
    let deltas = |pc: &Pieces, base: Option<&Pieces>| -> PartDeltas {
        let (base, base_n): (&Pieces, BTreeMap<&str, Vec<V3>>) = match base {
            Some(b) => (b, b.iter().map(|(&k, (p, i))| (k, smooth_n(k, p, i))).collect()),
            None => (&rest, rest_n.clone()),
        };
        PARTS
            .iter()
            .map(|&(part, names)| {
                let mut dp = Vec::new();
                let mut dn = Vec::new();
                for &nm in names {
                    dp.extend(pc[nm].0.iter().zip(&base[nm].0).map(|(a, b)| a - b));
                    dn.extend(smooth_n(nm, &pc[nm].0, &pc[nm].1).iter().zip(&base_n[nm]).map(|(a, b)| a - b));
                }
                (part, (dp, dn))
            })
            .collect()
    };
    let mut chans: Vec<(&str, PartDeltas)> = Vec::new();
    for c in channels() {
        let d = if let Some(vp) = viseme_params(c) {
            if vp.is_empty() { Vec::new() } else { deltas(&pieces(&face, &Params::of(vp)), None) }
        } else if c == "mouthClose" {
            // A corrective (as in ARKit): the lips closing over an open jaw.
            let open = pieces(&face, &Params::of(&[("jaw", 1.0)]));
            deltas(&pieces(&face, &Params::of(&[("jaw", 1.0), ("close", 1.0)])), Some(&open))
        } else {
            deltas(&pieces(&face, &unit_params(c)), None)
        };
        chans.push((c, d));
    }
    let mut morphs: Vec<(String, Deltas)> = Vec::new();
    for (c, d) in chans {
        for (part, (dp, dn)) in d {
            let mx = dp.iter().map(|v| v.abs().max()).fold(f64::NEG_INFINITY, f64::max);
            if mx > 1e-6 {
                match morphs.iter_mut().find(|e| e.0 == part) {
                    Some(e) => {
                        e.1.insert(c, (dp, dn));
                    }
                    None => {
                        let mut m = Deltas::new();
                        m.insert(c, (dp, dn));
                        morphs.push((part.to_string(), m));
                    }
                }
            }
        }
    }
    // The lip lines (the beard follows them).
    let lip_n: Vec<usize> = PARTS[0].1.iter().map(|n| rest[n].0.len()).collect();
    let k0: usize = lip_n[..4].iter().sum();
    let mut lips = rest["lipU"].0.clone();
    lips.extend(&rest["lipL"].0);
    let upper: Vec<bool> = (0..lips.len()).map(|i| i < rest["lipU"].0.len()).collect();
    let mut fm = Vec::new();
    if let Some((_, bm)) = morphs.iter().find(|e| e.0 == "body") {
        for c in channels() {
            if let Some((dp, _)) = bm.get(c) {
                fm.push((c, dp[k0..k0 + lips.len()].to_vec()));
            }
        }
    }
    FaceOut { parts, body, morphs, follow: Some(Follow { pos: lips, upper, morphs: fm }) }
}

/// The beard's morphs: each vertex copies the morphs of the lip vertices nearest it (the moustache
/// the upper lip's, the rest the lower lip's), fading with distance, so it moves with the mouth.
pub fn follow_mouth(beard: &Mesh, follow: &Follow, head: &HeadFrame) -> Deltas {
    let pos = &beard.pos;
    let above: Vec<bool> = pos.iter().map(|p| p.y > head.mouth[1]).collect();
    let mut fade = vec![0.0; pos.len()];
    // Per vertex: its 6 lip vertices and their weights.
    let mut near: Vec<Vec<(usize, f64)>> = vec![Vec::new(); pos.len()];
    for want_upper in [true, false] {
        let ids: Vec<usize> = (0..follow.pos.len()).filter(|&i| follow.upper[i] == want_upper).collect();
        let tree = KdTree::new(&ids.iter().map(|&i| follow.pos[i]).collect::<Vec<_>>());
        for (v, p) in pos.iter().enumerate() {
            if above[v] != want_upper {
                continue;
            }
            // The lips are mirror-symmetric, so a vertex on the midline has its neighbours in
            // equidistant pairs; a pair straddling the 6th place shares it (the Python took
            // whichever rounding noise put first, so the moustache's middle could slide sideways).
            let nb = tree.query_shared(p, 6, 1e-9);
            let wt: Vec<f64> = nb.iter().map(|&(d, _, share)| 1.0 / (d + 1e-4) * share).collect();
            let s = wt.iter().fold(0.0, |a, w| a + w);
            near[v] = nb.iter().zip(&wt).map(|(&(_, k, _), w)| (ids[k], w / s)).collect();
            fade[v] = clip(1.0 - (nb[0].0 - 0.015 * head.hh) / (0.32 * head.hh), 0.0, 1.0).powf(1.2);
        }
    }
    let mut out = Deltas::new();
    for (c, dp) in &follow.morphs {
        let acc: Vec<V3> = (0..pos.len())
            .map(|v| {
                let mut a = V3::zeros();
                for &(k, w) in &near[v] {
                    a += dp[k] * w;
                }
                a * fade[v]
            })
            .collect();
        let mx = acc.iter().map(|v| v.abs().max()).fold(f64::NEG_INFINITY, f64::max);
        if mx > 1e-6 {
            out.insert(c, (acc, vec![V3::zeros(); pos.len()]));
        }
    }
    out
}
