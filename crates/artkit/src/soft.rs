//! Soft things: a box rounded, puffed and pinched as a stuffed cushion is (`soft_box`), for seat, back and arm
//! cushions, pillows, mattresses, duvets, upholstered arms and backs, with piping along its seams and buttons in
//! its tufts. One mesh part (Blender axes, metres), deterministic, with its material, as the forms are.
//!
//! The surface is the Python's own arithmetic: a point of the box's surface moved onto the soft shape (`Soft`), in
//! double precision where the script had floats and single where it had a `Vector`, so the points are Blender's to
//! the bit.

use crate::far::is_low;
use crate::forms::{self, Sweep};
use crate::materials::Mat;
use crate::math::{self, M4, P3, PI, V3, v3};
use crate::noise::noise;
use crate::py;
use crate::scene::Obj;
use crate::shapes::{self, options, shade};
use std::collections::BTreeMap;

/// Metres: how far a soft thing's rounded edges may stand in from their curve (`cargo review facets`).
pub const SOFT_FACET: f64 = 0.0015;
/// And its crowned faces from theirs: a face's middle isn't an outline, so it may be coarser.
pub const CROWN_FACET: f64 = 0.002;

/// How a soft box's faces puff out, metres in the middle: none, its two largest faces (the ones across its
/// thinnest side) by one number, or each face pair across x, y and z by its own.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum Puff {
    #[default]
    None,
    Faces(f64),
    Each([f64; 3]),
}

impl crate::Arg<Puff> for f64 {
    fn arg(self) -> Puff {
        Puff::Faces(self)
    }
}
impl crate::Arg<Puff> for i32 {
    fn arg(self) -> Puff {
        Puff::Faces(self as f64)
    }
}
impl<A: math::Num, B: math::Num, C: math::Num> crate::Arg<Puff> for (A, B, C) {
    fn arg(self) -> Puff {
        Puff::Each([self.0.f(), self.1.f(), self.2.f()])
    }
}
impl crate::Arg<Puff> for Option<f64> {
    fn arg(self) -> Puff {
        self.map_or(Puff::None, Puff::Faces)
    }
}

/// How a soft box's top sinks: towards its front edge by a number of metres, or towards the edge a vector
/// (x, y, z) points at, its length how far.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Sag {
    Front(f64),
    Towards([f64; 3]),
}

impl Default for Sag {
    fn default() -> Sag {
        Sag::Front(0.0)
    }
}
impl crate::Arg<Sag> for f64 {
    fn arg(self) -> Sag {
        Sag::Front(self)
    }
}
impl crate::Arg<Sag> for i32 {
    fn arg(self) -> Sag {
        Sag::Front(self as f64)
    }
}
impl<A: math::Num, B: math::Num, C: math::Num> crate::Arg<Sag> for (A, B, C) {
    fn arg(self) -> Sag {
        Sag::Towards([self.0.f(), self.1.f(), self.2.f()])
    }
}

options! {
    /// `soft_box`'s options.
    SoftBox { at: P3 = P3::ZERO, material: Option<Mat> = None, rot: P3 = P3::ZERO, round: f64 = 0.04, puff: Puff = Puff::None, round_segments: Option<usize> = None, flat: Option<usize> = None, piping: Option<f64> = None, piping_material: Option<Mat> = None, pinch: Option<f64> = None, waist: f64 = 0.0, sag: Sag = Sag::Front(0.0), overhang: f64 = 0.0, tufts: Option<(usize, usize)> = None, tuft: f64 = 0.012, diamond: bool = false, button: Option<Mat> = None, rumple: f64 = 0.0, rumple_size: f64 = 0.24, seed: f64 = 0.0, calm: Option<(f64, f64)> = None, resting: bool = false }
}

/// `soft_box` with its options by name, as the family scripts wrote them: `soft_box!(name, size, key = value, ...)`.
#[macro_export]
macro_rules! soft_box {
    ($name:expr, $size:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        // (as Python: what is given by place first, then the options in the order written)
        let args = ($name, $size);
        #[allow(unused_mut)]
        let mut o = $crate::soft::SoftBox::default();
        $(o.$k = $crate::Arg::arg($v);)*
        $crate::soft::soft_box(args.0, args.1, o)
    }};
}

// Python's min and max of two: the first unless the second is strictly beyond it.
fn min(a: f64, b: f64) -> f64 {
    if b < a { b } else { a }
}
fn max(a: f64, b: f64) -> f64 {
    if b > a { b } else { a }
}

/// `x ** k` for floats.
fn pw(x: f64, k: f64) -> f64 {
    math::pow(x, k)
}

/// Positions from -h to h: `round_segments` closing in on each end over the rounding `r`, placed so the rounding
/// turns by the same angle from each to the next (its eighth of a turn on this face in equal parts), `flat` steps
/// across the middle.
fn steps(h: f64, r: f64, round_segments: usize, flat: usize) -> Vec<f64> {
    let edge: Vec<f64> = (0..round_segments)
        .map(|i| -(h - r) - r * math::tan(PI / 4.0 * (1.0 - i as f64 / round_segments as f64)))
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

/// A soft box `size` (x, y, z) across: its edges rounded by `round`, its faces puffed out by `puff` metres in the
/// middle (a number for its two largest faces, the ones across its thinnest side; or (x, y, z), each face pair's
/// own), falling to nothing at its edges, as a stuffed cushion is. For seat, back and arm cushions, pillows,
/// mattresses, duvets, upholstered arms and backs. What stuffing and seams do to it, each a few centimetres at most
/// and none of it past the box (`size` stays its size):
/// - `pinch`: its corners pulled in and thinned where the seams meet, so its sides bow out between them, as a boxed
///   cushion's do (metres at the corner; by default 0.6 of its deepest puff, 0 for none and for a box whose sides
///   are crossed in one step). Its corners are then inside the box: give 0 to one turned so that a corner is the
///   model's outermost point (a folded towel askew on a stack), or the model is that much smaller;
/// - `waist`: its sides drawn in between its corners, which stand out as ears, as a pillow sewn flat and stuffed
///   does (metres at the middle of a side);
/// - `sag`: its top (+z) sinking over its front third to this much lower at its front edge (-y), as a seat
///   cushion's does where legs go over it (or a vector (x, y, z) towards the edge it sinks to, its length how far);
///   its middle and back, where the sitter's hips rest, keep their height, and its back corners are pinched a
///   third as much as its front ones (they are held under the back cushions);
/// - `overhang`: its sides drawn in by this much between its two seams, so the top and the bottom stand over them;
/// - `tufts`: (across, along) rows of dents `tuft` deep in the face across its thin side that looks up or to the
///   front (both faces of one thin across x), in a square grid or staggered (`diamond`), each with a button in the
///   material `button`;
/// - `rumple`: its top sunk by up to this much in soft folds about `rumple_size` apart, lying askew, and what hangs
///   down its sides drawn in in flutes, deeper towards the hem, which is square, not rounded under (`seed`: which
///   folds), as bedding shaken out lies; only downwards and inwards; `calm` (from, to) along its own y: where
///   something lies across it (a runner) and it is smooth;
/// - `piping` (a radius): a cord along both seams of its thin side, following them, in `piping_material` (its own
///   material by default).
///
/// `flat` is how many steps cross a puffed face and `round_segments` how many turn each rounded edge on each of the
/// two faces it joins: by default as many as keep the rounded edges within 1.5 mm of their curve and the crowned
/// faces within 2 mm of theirs (and what tufts and a rumple need). `resting`: it lies on its underside (-z) or,
/// thin from front to back, leans on its back (+y), which nobody sees: that face's middle is one flat piece, for a
/// third fewer triangles.
pub fn soft_box(name: impl AsRef<str>, size: impl Into<shapes::Size>, o: SoftBox) -> Obj {
    let name = name.as_ref();
    let size = size.into().0;
    let h = [size.x / 2.0, size.y / 2.0, size.z / 2.0];
    let r = min(min(min(o.round, h[0]), h[1]), h[2]);
    let mut thin = 0;
    for i in 1..3 {
        if h[i] < h[thin] {
            thin = i;
        }
    }
    let plan: [usize; 2] = match thin {
        0 => [1, 2],
        1 => [0, 2],
        _ => [0, 1],
    };
    let puffs = match o.puff {
        Puff::None => [0.0; 3],
        Puff::Faces(p) => [0, 1, 2].map(|i| if i == thin { p } else { 0.0 }),
        Puff::Each(p) => p,
    };
    let low = is_low();
    let own_pinch = o.pinch.is_some();
    let deepest = max(max(puffs[0], puffs[1]), puffs[2]);
    let mut pinch = min(
        o.pinch.unwrap_or(0.6 * deepest),
        0.45 * min(h[plan[0]], h[plan[1]]),
    );
    let sag = match o.sag {
        Sag::Front(s) => v3(0.0, -s, 0.0),
        Sag::Towards(s) => V3::from(s),
    };
    let (tufts, rumple) = if low {
        (None, 0.0)
    } else {
        (o.tufts, o.rumple)
    };
    let round_segments = o.round_segments.unwrap_or_else(|| {
        // the rounding's quarter turn in 2 x round_segments equal parts: each stands r (1 - cos(pi / 8n)) in
        if r < 0.008 || low {
            1
        } else if r * (1.0 - math::cos(PI / 16.0)) <= SOFT_FACET {
            2
        } else {
            3
        }
    });
    let inner = [h[0] - r, h[1] - r, h[2] - r];
    let mut count = [0usize; 3];
    for i in 0..3 {
        // how many steps cross each axis between its rounded ends
        let others: Vec<f64> = (0..3).filter(|&a| a != i).map(|a| puffs[a]).collect();
        let across = max(others[0], others[1]); // the deepest puff on a face this axis crosses
        let mut n = 1;
        if across != 0.0 {
            // a crown `p` deep in n steps stands p / n² in from its curve
            n = match o.flat {
                Some(flat) => flat,
                None if low => 2,
                None => 2.max(5.min((across / CROWN_FACET).sqrt().ceil() as usize)),
            };
        }
        if o.flat.is_none() && !low && sag.length() != 0.0 && (sag[i] as f64).abs() > 1e-9 {
            n = n.max(3);
        }
        if o.overhang != 0.0 && i == thin {
            n = n.max(2);
        }
        if rumple != 0.0 && i != thin {
            n = n.max((2.0 * inner[i] / (o.rumple_size / 2.5)).ceil() as usize);
        }
        if let Some(t) = tufts.filter(|_| i != thin) {
            // a vertex on every tuft and three between two: a dimple, not a pyramid
            let k = 4 * if plan[0] == i { t.0 } else { t.1 };
            n = k * (n as f64 / k as f64).ceil() as usize;
        }
        count[i] = n;
    }
    if !own_pinch && count[plan[0]].min(count[plan[1]]) < 2 {
        pinch = 0.0; // a side crossed in one step has no middle to stay put: pinching its corners would narrow it
    }
    let steps: Vec<Vec<f64>> = (0..3)
        .map(|i| steps(h[i], r, round_segments, count[i]))
        .collect();
    let shape = Soft::new(SoftShape {
        h,
        inner,
        r,
        puffs,
        thin,
        plan,
        pinch,
        waist: o.waist,
        sag,
        overhang: o.overhang,
        tufts,
        tuft: o.tuft,
        diamond: o.diamond,
        rumple,
        size: o.rumple_size,
        seed: o.seed,
        calm: o.calm,
    });
    let mut index: BTreeMap<[i64; 3], u32> = BTreeMap::new();
    let mut vs: Vec<V3> = Vec::new();
    let mut faces: Vec<Vec<u32>> = Vec::new();
    let mut vert = |p: [f64; 3]| -> u32 {
        let key = p.map(round_);
        *index.entry(key).or_insert_with(|| {
            vs.push(shape.at(p));
            vs.len() as u32 - 1
        })
    };
    // the side of the thin axis it rests on
    let hidden: Option<i32> = if o.resting {
        match thin {
            2 => Some(-1),
            1 => Some(1),
            _ => None,
        }
    } else {
        None
    };
    for a in 0..3 {
        // the face pairs across axis a
        let (b, c) = ((a + 1) % 3, (a + 2) % 3);
        for side in [-1, 1] {
            // (an axis no longer than the rounding has one middle point, whatever its count)
            let (i0, i1, j0, j1) = (
                round_segments,
                steps[b].len() - 1 - round_segments,
                round_segments,
                steps[c].len() - 1 - round_segments,
            );
            let plain = a == thin && Some(side) == hidden && i1 - i0 > 1 && j1 - j0 > 1;
            let point = |i: usize, j: usize| {
                let mut p = [0.0; 3];
                p[a] = side as f64 * h[a];
                p[b] = steps[b][i];
                p[c] = steps[c][j];
                p
            };
            if plain {
                // the middle of the face it rests on, in one piece
                let ring: Vec<(usize, usize)> = (i0..i1)
                    .map(|i| (i, j0))
                    .chain((j0..j1).map(|j| (i1, j)))
                    .chain((i0 + 1..=i1).rev().map(|i| (i, j1)))
                    .chain((j0 + 1..=j1).rev().map(|j| (i0, j)))
                    .collect();
                let mut lp: Vec<u32> = ring.iter().map(|&(i, j)| vert(point(i, j))).collect();
                if side < 0 {
                    lp.reverse();
                }
                faces.push(lp);
            }
            for i in 0..steps[b].len() - 1 {
                for j in 0..steps[c].len() - 1 {
                    if plain && i0 <= i && i < i1 && j0 <= j && j < j1 {
                        continue;
                    }
                    let mut quad: Vec<u32> = [(0, 0), (1, 0), (1, 1), (0, 1)]
                        .iter()
                        .map(|&(di, dj)| vert(point(i + di, j + dj)))
                        .collect();
                    if side < 0 {
                        quad.reverse();
                    }
                    faces.push(quad);
                }
            }
        }
    }
    let obj = forms::object(name, vs, &faces, o.material, true, 75.0, false);
    let mut made = vec![obj];
    if let Some(radius) = o.piping.filter(|&p| p != 0.0) {
        let sides: Vec<i32> = [-1, 1].into_iter().filter(|&s| Some(s) != hidden).collect();
        made.extend(piping(
            name,
            h,
            &steps,
            thin,
            plan,
            &shape,
            radius,
            o.piping_material.or(o.material),
            &sides,
        ));
    }
    if let (true, Some(button)) = (shape.tufts.is_some(), o.button) {
        made.extend(buttons(name, &shape, button));
    }
    if made.len() > 1 {
        crate::parts::join_into(obj, &made[1..]);
    }
    obj.set_location(o.at).set_rotation(o.rot)
}

/// A vertex's key: its place to a micrometre.
fn round_(c: f64) -> i64 {
    (c * 1e6 + 0.5).floor() as i64
}

/// What a soft box's shape is made of (`Soft::new`).
struct SoftShape {
    h: [f64; 3],
    inner: [f64; 3],
    r: f64,
    puffs: [f64; 3],
    thin: usize,
    plan: [usize; 2],
    pinch: f64,
    waist: f64,
    sag: V3,
    overhang: f64,
    tufts: Option<(usize, usize)>,
    tuft: f64,
    diamond: bool,
    rumple: f64,
    size: f64,
    seed: f64,
    calm: Option<(f64, f64)>,
}

/// A soft box's shape: a point of the box's surface moved onto the rounded, puffed, pinched one (`soft_box`).
pub struct Soft {
    s: SoftShape,
    /// Where the seams run: half way round the rounding of the thin side's edges.
    seam: f64,
    /// The tufted faces: the one that looks up (thin across z) or to the front (-y); both of one thin across x.
    sides: Vec<i32>,
    /// The tufts' middles (across, along); None for none.
    tufts: Option<Vec<(f64, f64)>>,
    reach: f64,
}

impl Soft {
    fn new(mut s: SoftShape) -> Soft {
        let (h, inner, plan) = (s.h, s.inner, s.plan);
        s.waist = min(s.waist, 0.4 * min(h[plan[0]], h[plan[1]]));
        let seam = max(1e-6, h[s.thin] - s.r * (1.0 - 0.5_f64.sqrt()));
        let sides = match s.thin {
            2 => vec![1],
            1 => vec![-1],
            _ => vec![-1, 1],
        };
        let (mut tufts, mut reach) = (None, 0.0);
        if let Some((nu, nv)) = s.tufts {
            let [b, c] = plan;
            let (pu, pv) = (2.0 * inner[b] / nu as f64, 2.0 * inner[c] / nv as f64);
            let mut at = Vec::new();
            for j in 0..nv {
                let odd = s.diamond && j % 2 == 1;
                for i in 0..if odd { nu - 1 } else { nu } {
                    at.push((
                        -inner[b] + (i as f64 + if odd { 1.0 } else { 0.5 }) * pu,
                        -inner[c] + (j as f64 + 0.5) * pv,
                    ));
                }
            }
            tufts = Some(at);
            reach = 0.22 * min(pu, pv);
        }
        Soft {
            s,
            seam,
            sides,
            tufts,
            reach,
        }
    }

    /// The soft surface's point for `p`, a point of the box's surface.
    pub fn at(&self, p: [f64; 3]) -> V3 {
        let s = &self.s;
        let (h, r, a) = (s.h, s.r, s.thin);
        let [b, c] = s.plan;
        let centre = v3(
            max(-s.inner[0], min(s.inner[0], p[0])),
            max(-s.inner[1], min(s.inner[1], p[1])),
            max(-s.inner[2], min(s.inner[2], p[2])),
        );
        let d = V3::from(p) - centre;
        let mut base = if d.length() > 1e-9 {
            centre + d.normalized() * r
        } else {
            V3::from(p)
        };
        let f = |x: f32| x as f64;
        if s.rumple != 0.0 && p[a] < -s.inner[a] {
            // a cover hangs to a hem: below its sides its edge isn't rounded under, only round its corners
            let across = py::hypot(f(d[b]), f(d[c]));
            base = V3::from(p);
            if across > r {
                let (nb, nc) = (
                    f(centre[b]) + f(d[b]) * r / across,
                    f(centre[c]) + f(d[c]) * r / across,
                );
                base[b] = nb as f32;
                base[c] = nc as f32;
            }
        }
        let mut q = base;
        // q[k] -= x, as a Vector's coordinate takes it: in double, stored single
        let sub = |q: &mut V3, k: usize, x: f64| q[k] = (f(q[k]) - x) as f32;
        for k in 0..3 {
            if s.puffs[k] != 0.0 {
                let (m, n) = ((k + 1) % 3, (k + 2) % 3);
                let fall = max(0.0, 1.0 - pw(f(base[m]) / h[m], 2.0))
                    * max(0.0, 1.0 - pw(f(base[n]) / h[n], 2.0));
                let up =
                    (s.puffs[k] * fall * pw(f(base[k]).abs() / h[k], 4.0)).copysign(f(base[k]));
                q[k] = (f(q[k]) + up) as f32;
            }
        }
        let (u, v, w) = (f(base[b]) / h[b], f(base[c]) / h[c], f(base[a]) / h[a]);
        let face = pw(w.abs(), 4.0); // 1 on the faces across the thin side, falling away round their edges
        if let Some(tufts) = &self.tufts
            && self.sides.contains(&if w > 0.0 { 1 } else { -1 })
        {
            let dent = py::sum(tufts.iter().map(|&(x, y)| {
                math::exp(
                    -(pw(f(base[b]) - x, 2.0) + pw(f(base[c]) - y, 2.0)) / pw(self.reach, 2.0),
                )
            }));
            sub(&mut q, a, (s.tuft * min(1.0, dent) * face).copysign(w));
        }
        if s.rumple != 0.0 {
            let size = s.size;
            let seed = s.seed;
            // the folds lie askew
            let (x, y) = (
                f(base[b]) * 0.83 + f(base[c]) * 0.55,
                f(base[c]) * 0.83 - f(base[b]) * 0.55,
            );
            let fold = noise(v3(x / size + seed, y / (2.4 * size) - seed, seed * 0.5));
            let fine = noise(v3(x / (0.5 * size) - seed, y / (0.8 * size) + 7.3, seed));
            let n = max(0.0, min(1.0, 0.5 + 1.5 * fold + 0.3 * fine));
            let mut still = 1.0;
            if let Some(calm) = s.calm {
                // smooth under what lies across it, the folds coming back over 8 cm either side
                let off = max(max(calm.0 - f(base[1]), f(base[1]) - calm.1), 0.0) / 0.08;
                still = pw(min(1.0, off), 2.0);
            }
            if w > 0.0 {
                sub(
                    &mut q,
                    a,
                    s.rumple * n * n * (3.0 - 2.0 * n) * w * w * still,
                );
            }
            // what hangs down its sides, drawn in in flutes that deepen towards the hem
            let out = if r > 1e-9 {
                py::hypot(f(d[b]), f(d[c])) / r
            } else {
                0.0
            };
            let hang = max(0.0, min(1.0, (1.0 - w) / 2.0)) * out * out;
            if hang != 0.0 {
                // the way along the edge
                let along = if f(d[b]).abs() < f(d[c]).abs() {
                    f(base[b]) - f(base[c])
                } else {
                    f(base[b]) + f(base[c])
                };
                let flute = noise(v3(
                    along / (0.55 * size) + 3.1 + seed,
                    seed * 1.7,
                    f(base[a]) / (4.0 * size),
                ));
                // (not at its corners)
                let n2 = max(0.0, min(1.0, 0.5 + 1.6 * flute)) * (1.0 - pw((u * v).abs(), 3.0));
                for k in [b, c] {
                    sub(&mut q, k, f(d[k]) / r * s.rumple * 0.6 * n2 * hang * still);
                }
            }
        }
        let sag_length = s.sag.length();
        if sag_length != 0.0 && w > 0.0 {
            let to = s.sag.normalized();
            let reach = self.sag_reach(to);
            // (its front third: where the hips rest, and 5 cm round it, stays at its height)
            let t = max(
                0.0,
                min(1.0, (0.5 + 0.5 * base.dot(to) / reach - 0.68) / 0.32),
            );
            sub(&mut q, a, sag_length * t * t * w);
        }
        if s.pinch != 0.0 {
            let mut k = pw(u.abs(), 3.0) * pw(v.abs(), 3.0);
            if sag_length != 0.0 {
                // a seat cushion's back corners, held under the back, are pinched a third as much
                let to = s.sag.normalized();
                let reach = self.sag_reach(to);
                k *= 0.3 + 0.7 * max(0.0, min(1.0, 0.5 + 0.5 * base.dot(to) / reach));
            }
            sub(&mut q, b, (s.pinch * k).copysign(u));
            sub(&mut q, c, (s.pinch * k).copysign(v));
            sub(&mut q, a, min(0.5 * s.pinch, 0.45 * h[a]) * k * w);
        }
        if s.waist != 0.0 {
            // (nothing where a side ends and its corner's rounding begins: its ends are the box's outermost points)
            let along_b = if s.inner[b] > 1e-9 {
                min(1.0, f(base[b]).abs() / s.inner[b])
            } else {
                1.0
            };
            let along_c = if s.inner[c] > 1e-9 {
                min(1.0, f(base[c]).abs() / s.inner[c])
            } else {
                1.0
            };
            sub(
                &mut q,
                b,
                (s.waist * pw(u.abs(), 3.0) * (1.0 - pw(along_c, 2.0))).copysign(u),
            );
            sub(
                &mut q,
                c,
                (s.waist * pw(v.abs(), 3.0) * (1.0 - pw(along_b, 2.0))).copysign(v),
            );
        }
        if s.overhang != 0.0 && r > 1e-9 {
            let between = max(0.0, 1.0 - pw(f(base[a]) / self.seam, 2.0));
            let out = py::hypot(f(d[b]), f(d[c])) / r;
            for k in [b, c] {
                sub(&mut q, k, f(d[k]) / r * s.overhang * between * out);
            }
        }
        q
    }

    /// How far the sag reaches across the box towards `to`.
    fn sag_reach(&self, to: V3) -> f64 {
        let reach = py::sum((0..3).map(|i| (to[i] as f64).abs() * self.s.h[i]));
        if reach != 0.0 { reach } else { 1.0 }
    }
}

/// A closed path with the points left out that lie within `tolerance` of the line between their neighbours kept
/// (a straight run's, a gentle bow's): fewer rings for a cord swept along it.
fn thinned(pts: &[V3], tolerance: f64) -> Vec<V3> {
    let n = pts.len();
    let mut keep = vec![true; n];
    let mut last = 0;
    for i in 1..n {
        let nxt = pts[(i + 1) % n];
        let (a, line) = (pts[last], nxt - pts[last]);
        if line.length() > 1e-9
            && (last + 1..=i).all(|k| (pts[k] - a).cross(line).length() / line.length() < tolerance)
        {
            keep[i] = false;
        } else {
            last = i;
        }
    }
    pts.iter()
        .zip(&keep)
        .filter(|(_, k)| **k)
        .map(|(p, _)| *p)
        .collect()
}

/// Cords along the seams of a soft box's thin side (`sides`: which), where its faces turn into the boxing: through
/// the shape's own seam, so they follow its pinched corners and bowed sides, standing a fifth of their radius proud
/// of it, so a cord cut straight across a gentle bow never sinks into the cushion. They reach as far out as a cord
/// of `radius` centred on the seam would (a model whose outermost point is a welt keeps its size), so they are that
/// much slimmer.
#[allow(clippy::too_many_arguments)]
fn piping(
    name: &str,
    h: [f64; 3],
    steps: &[Vec<f64>],
    thin: usize,
    plan: [usize; 2],
    shape: &Soft,
    radius: f64,
    material: Option<Mat>,
    sides: &[i32],
) -> Vec<Obj> {
    let [b, c] = plan;
    let radius = radius / 1.2;
    let (sb, sc) = (&steps[b], &steps[c]);
    let lp: Vec<(f64, f64)> = sb[..sb.len() - 1]
        .iter()
        .map(|&x| (x, -h[c]))
        .chain(sc[..sc.len() - 1].iter().map(|&y| (h[b], y)))
        .chain(sb[1..].iter().rev().map(|&x| (x, h[c])))
        .chain(sc[1..].iter().rev().map(|&y| (-h[b], y)))
        .collect();
    let inner = shape.s.inner;
    let mut made = Vec::new();
    for &side in sides {
        let pts: Vec<V3> = lp
            .iter()
            .map(|&(x, y)| {
                let mut p = [0.0; 3];
                p[b] = x;
                p[c] = y;
                p[thin] = side as f64 * h[thin];
                let out = v3(
                    p[0] - max(-inner[0], min(inner[0], p[0])),
                    p[1] - max(-inner[1], min(inner[1], p[1])),
                    p[2] - max(-inner[2], min(inner[2], p[2])),
                );
                shape.at(p) + out.normalized() * radius * 0.2
            })
            .collect();
        // (four sides stand 1.5 mm in from a 5 mm cord's round, five from one of 8 mm)
        made.push(forms::sweep(
            format!("{name}_piping{side}"),
            &thinned(&pts, 0.0025),
            radius,
            Sweep {
                material,
                verts: Some(if radius <= 0.0043 { 4 } else { 5 }),
                closed: true,
                ..Sweep::default()
            },
        ));
    }
    made
}

/// A covered button in each of a soft box's tufts: a low dome sitting in the dent, wider for a deeper one.
fn buttons(name: &str, shape: &Soft, material: Mat) -> Vec<Obj> {
    let radius = max(0.009, min(0.014, 0.75 * shape.s.tuft));
    let a = shape.s.thin;
    let [b, c] = shape.s.plan;
    let mut made = Vec::new();
    for &side in &shape.sides {
        for (i, &(x, y)) in shape.tufts.iter().flatten().enumerate() {
            let mut p = [0.0; 3];
            p[b] = x;
            p[c] = y;
            p[a] = side as f64 * shape.s.h[a];
            let mut q = shape.at(p);
            q[a] = (q[a] as f64 + side as f64 * radius * 0.45) as f32; // proud of the dent's floor, so its rim shows
            let mut scale = [1.0; 3];
            scale[a] = 0.55;
            let mut mesh = shapes::primitive::uv_sphere(8, 4, radius as f32);
            mesh.transform(&M4::diagonal(V3::from(scale)));
            mesh.transform(&M4::translation(q));
            mesh.set_material(Some(material));
            let ball =
                crate::scene::add_primitive("Sphere", &format!("{name}_button{side}{i}"), mesh);
            shade(ball, true, 180.0);
            made.push(ball);
        }
    }
    made
}
