//! Moving parts: a door, lid or drawer that moves while residents use the object (docs/design/objects.md,
//! "Moving parts"). Each part is its own part with its origin on the hinge (`parts::hinged`), and each clip is one
//! track on it, named for the pack's `motions`.
//!
//! A clip's first key (frame 1) is the rest pose: where the file has the part, and the game draws it closed. Only
//! the part's rotation, location or scale move. `grow` and `squash` go from rest to open (the game plays them
//! back); `spin`, `rock`, `turns` and `wobble` come home to rest within the clip, so they also serve as loops
//! (`motions.loops`, `idle`), and `scroll` moves one repeat on, home unseen; `stages` is a crop's growth, held where
//! its stage is; `path` and `keys` key what the others don't. The file's clips are sampled every frame (30 a
//! second, `scene::FPS`) from frame 1 as 0 s, and the game samples them 60 times a second (twice each key).
//!
//! The curves are Blender's F-curves to the bit: auto-clamped keys whose handles are smoothed for continuous
//! acceleration (Blender's defaults), or one of its easings from a key to the next (`Ease`), all in single
//! precision as Blender evaluates them.

use crate::math::{self, Axis, M4, P3, PI, V3};
use crate::py;
use crate::scene::{self, FPS, Obj};
use crate::shapes::options;

/// What a channel keys: the part's place, its turn (Euler XYZ, radians) or its scale.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Path {
    Location,
    Rotation,
    Scale,
}

/// How a clip eases from one key to the next.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Ease {
    /// Blender's default: slow out of each key and into the next.
    #[default]
    Smooth,
    /// An even speed (spins, scrolls, loops).
    Linear,
    /// Quick off, slowing into the end.
    Out,
    In,
    InOut,
    /// A snap off, a long glide in (a drawer let run).
    ExpoOut,
    /// Past the end and back: a door swung open, settling.
    BackOut,
    /// Dropping onto the end and bouncing (a lid let fall).
    BounceOut,
}

/// How far past its end `BackOut` goes, as a share of the move (4°: a door opened 100°).
pub const OVERSHOOT: f64 = 0.04;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Key {
    pub frame: i32,
    pub value: f32,
    /// From this key to the next.
    pub ease: Ease,
}

/// One keyed number: a coordinate of the part's place, turn or scale.
#[derive(Clone, Debug)]
pub struct Channel {
    pub path: Path,
    pub index: usize,
    pub keys: Vec<Key>,
    /// `BackOut`'s overshoot.
    pub back: f64,
}

/// One clip on one part.
#[derive(Clone, Debug)]
pub struct Track {
    pub name: String,
    pub channels: Vec<Channel>,
    /// The frame of its last key.
    pub end: i32,
    /// Where it is most open, for a picture of it.
    pub peak: Option<i32>,
}

/// The frame `seconds` into a clip: frame 1 is 0 s, halves rounded up (a clip is never a frame short).
pub fn frame(seconds: f64) -> i32 {
    1 + (seconds * FPS + 0.5).floor() as i32
}

/// The frame of a clip's last key, for a clip that ends `seconds` in (`frame`, at least the second).
pub fn last_frame(seconds: f64) -> i32 {
    frame(seconds).max(2)
}

impl Channel {
    /// The channel's value at `frame`, as Blender evaluates the F-curve (single precision throughout).
    pub fn at(&self, frame: f64) -> f64 {
        eval(&self.beziers(), frame as f32) as f64
    }

    /// Each key's (left handle, key, right handle), (frame, value) each, as Blender's F-curve holds them.
    pub fn handles(&self) -> Vec<[[f32; 2]; 3]> {
        self.beziers().iter().map(|b| [b.h1, b.co, b.h2]).collect()
    }

    /// The keys as Blender holds them: each with the handles its F-curve works out for it (auto-clamped, smoothed
    /// for continuous acceleration, Blender's defaults) and its interpolation to the next.
    fn beziers(&self) -> Vec<Bez> {
        let back = back_amount(self.back) as f32;
        let mut b: Vec<Bez> = self
            .keys
            .iter()
            .map(|k| Bez {
                h1: [0.0; 2],
                co: [k.frame as f32, k.value],
                h2: [0.0; 2],
                ease: k.ease,
                back,
                locked: false,
            })
            .collect();
        let n = b.len();
        if n < 2 {
            return b;
        }
        for i in 0..n {
            let prev = (i > 0).then(|| b[i - 1].co);
            let next = (i + 1 < n).then(|| b[i + 1].co);
            calc_handles(&mut b[i], prev, next);
            // the first and last keys level, as the curve holds still beyond them
            if i == 0 || i == n - 1 {
                b[i].h1[1] = b[i].co[1];
                b[i].h2[1] = b[i].co[1];
                b[i].locked = true;
            }
        }
        smooth_handles(&mut b);
        b
    }
}

/// A key on an F-curve: its left handle, its place (frame, value), its right handle.
#[derive(Clone, Copy, Debug)]
struct Bez {
    h1: [f32; 2],
    co: [f32; 2],
    h2: [f32; 2],
    ease: Ease,
    back: f32,
    /// Its handles stay as they are (a peak, a trough, an end): Blender's `HD_AUTOTYPE_LOCKED_FINAL`.
    locked: bool,
}

/// Blender's auto-clamped handles of a key (`calchandleNurb_intern` for an F-curve smoothed for continuous
/// acceleration): a third of the way to each neighbour, level at a peak or a trough, never past a neighbour's value.
fn calc_handles(bezt: &mut Bez, prev: Option<[f32; 2]>, next: Option<[f32; 2]>) {
    bezt.locked = false;
    let p2 = bezt.co;
    let p1 = match prev {
        Some(p) => p,
        None => {
            let p3 = next.unwrap_or(p2);
            [2.0 * p2[0] - p3[0], 2.0 * p2[1] - p3[1]]
        }
    };
    let p3 = match next {
        Some(p) => p,
        None => [2.0 * p2[0] - p1[0], 2.0 * p2[1] - p1[1]],
    };
    let dvec_a = [p2[0] - p1[0], p2[1] - p1[1]];
    let dvec_b = [p3[0] - p2[0], p3[1] - p2[1]];
    let mut len_a = if dvec_a[0] == 0.0 { 1.0 } else { dvec_a[0] };
    let mut len_b = if dvec_b[0] == 0.0 { 1.0 } else { dvec_b[0] };
    let tvec = [
        dvec_b[0] / len_b + dvec_a[0] / len_a,
        dvec_b[1] / len_b + dvec_a[1] / len_a,
    ];
    // (the handles a third of the interval along, so time runs evenly)
    let len = 6.0_f32 / 2.5614_f32 * 2.5614_f32;
    if len == 0.0 {
        return;
    }
    let (mut leftviolate, mut rightviolate) = (false, false);
    len_a /= len;
    bezt.h1 = [p2[0] + tvec[0] * -len_a, p2[1] + tvec[1] * -len_a];
    let extreme = |prev: [f32; 2], next: [f32; 2]| {
        let (ydiff1, ydiff2) = (prev[1] - p2[1], next[1] - p2[1]);
        (
            (ydiff1 <= 0.0 && ydiff2 <= 0.0) || (ydiff1 >= 0.0 && ydiff2 >= 0.0),
            ydiff1,
        )
    };
    if let (Some(pv), Some(nx)) = (prev, next) {
        // level at a peak or a trough, else not beyond the neighbour's value
        let (level, ydiff1) = extreme(pv, nx);
        if level {
            bezt.h1[1] = p2[1];
            bezt.locked = true;
        } else if ydiff1 <= 0.0 {
            if pv[1] > bezt.h1[1] {
                bezt.h1[1] = pv[1];
                leftviolate = true;
            }
        } else if pv[1] < bezt.h1[1] {
            bezt.h1[1] = pv[1];
            leftviolate = true;
        }
    }
    len_b /= len;
    bezt.h2 = [p2[0] + tvec[0] * len_b, p2[1] + tvec[1] * len_b];
    if let (Some(pv), Some(nx)) = (prev, next) {
        let (level, ydiff1) = extreme(pv, nx);
        if level {
            bezt.h2[1] = p2[1];
            bezt.locked = true;
        } else if ydiff1 <= 0.0 {
            if nx[1] < bezt.h2[1] {
                bezt.h2[1] = nx[1];
                rightviolate = true;
            }
        } else if nx[1] > bezt.h2[1] {
            bezt.h2[1] = nx[1];
            rightviolate = true;
        }
    }
    if leftviolate || rightviolate {
        // the other handle in line with the one held back
        let h1_x = bezt.h1[0] - p2[0];
        let h2_x = p2[0] - bezt.h2[0];
        if leftviolate {
            bezt.h2[1] = p2[1] + ((p2[1] - bezt.h1[1]) / h1_x) * h2_x;
        } else {
            bezt.h1[1] = p2[1] + ((p2[1] - bezt.h2[1]) / h2_x) * h1_x;
        }
    }
}

/// Blender's second pass over auto handles (`BKE_nurb_handle_smooth_fcurve`): each run of free keys between two locked
/// ones gets the handles that keep the acceleration continuous through it, clamped as auto-clamped handles are.
fn smooth_handles(b: &mut [Bez]) {
    let total = b.len();
    let (mut start, mut count) = (0, 1);
    for j in 1..total {
        if b[j].locked {
            smooth_run(b, start, count + 1);
            start = j;
            count = 1;
        } else {
            count += 1;
        }
    }
    if count > 1 {
        smooth_run(b, start, count);
    }
}

/// `bezier_handle_calc_smooth_fcurve` for the keys `start..start + count`, the first and last locked.
fn smooth_run(b: &mut [Bez], start: usize, count: usize) {
    // (both ends locked: a run of two has nothing to solve)
    if count <= 2 {
        return;
    }
    let mut dx = vec![f32::NAN; count];
    let mut dy = vec![f32::NAN; count];
    for i in 1..count {
        let j = start + i;
        dx[i] = b[j].co[0] - b[j - 1].co[0];
        dy[i] = b[j].co[1] - b[j - 1].co[1];
    }
    let mut l = vec![1.0_f32; count];
    for i in 1..count - 1 {
        l[i] = dx[i + 1] / dx[i];
    }
    let mut hmax = vec![f32::MAX; count];
    let mut hmin = vec![-f32::MAX; count];
    for i in 1..count {
        clamp(&mut hmax, &mut hmin, i - 1, dy[i]);
        clamp(&mut hmax, &mut hmin, i, dy[i] * l[i]);
    }
    let (mut a, mut bb, mut c, mut d) = (
        vec![0.0_f32; count],
        vec![0.0_f32; count],
        vec![0.0_f32; count],
        vec![0.0_f32; count],
    );
    // the ends' handles as they are
    let first = b[start];
    let mut tmp = [first.h2[0] - first.co[0], first.h2[1] - first.co[1]];
    let first_handle_adj = handle_adj(&mut tmp, dx[1]);
    lock(&mut a, &mut bb, &mut c, &mut d, 0, tmp[1]);
    let last = b[start + count - 1];
    let mut tmp = [last.co[0] - last.h1[0], last.co[1] - last.h1[1]];
    let last_handle_adj = handle_adj(&mut tmp, dx[count - 1]);
    lock(&mut a, &mut bb, &mut c, &mut d, count - 1, tmp[1]);
    for i in 1..count - 1 {
        a[i] = l[i] * l[i];
        bb[i] = 2.0 * (l[i] + 1.0);
        c[i] = 1.0 / l[i + 1];
        d[i] = dy[i] * l[i] * l[i] + dy[i + 1];
    }
    bb[1] += l[1] * first_handle_adj;
    bb[count - 2] += last_handle_adj;
    let mut h = vec![0.0_f32; count];
    if solve_with_limits(&mut a, &mut bb, &mut c, &mut d, &mut h, &hmin, &hmax) {
        for i in 1..count - 1 {
            let j = start + i;
            b[j].h1[1] = b[j].co[1] + -h[i] / l[i];
            b[j].h2[1] = b[j].co[1] + h[i];
        }
    }
}

/// An auto-clamped handle neither turns its key back nor overshoots its neighbour (`bezier_clamp`).
fn clamp(hmax: &mut [f32], hmin: &mut [f32], i: usize, dy: f32) {
    if dy > 0.0 {
        hmax[i] = hmax[i].min(dy);
        hmin[i] = 0.0;
    } else if dy < 0.0 {
        hmax[i] = 0.0;
        hmin[i] = hmin[i].max(dy);
    } else {
        hmax[i] = 0.0;
        hmin[i] = 0.0;
    }
}

/// A fixed handle's share of its interval, scaled to fit (`bezier_calc_handle_adj`).
fn handle_adj(hsize: &mut [f32; 2], dx: f32) -> f32 {
    let fac = dx / (hsize[0] + dx / 3.0);
    if fac < 1.0 {
        hsize[0] *= fac;
        hsize[1] *= fac;
    }
    1.0 - 3.0 * hsize[0] / dx
}

fn lock(a: &mut [f32], b: &mut [f32], c: &mut [f32], d: &mut [f32], i: usize, value: f32) {
    a[i] = 0.0;
    c[i] = 0.0;
    b[i] = 1.0;
    d[i] = value;
}

/// The tridiagonal system solved, handles past their limits locked to them and let go again where they pull back
/// (`tridiagonal_solve_with_limits`; its ends are locked, so the system is never cyclic).
fn solve_with_limits(
    a: &mut [f32],
    b: &mut [f32],
    c: &mut [f32],
    d: &mut [f32],
    h: &mut [f32],
    hmin: &[f32],
    hmax: &[f32],
) -> bool {
    let n = a.len();
    let (a0, b0, c0, d0) = (a.to_vec(), b.to_vec(), c.to_vec(), d.to_vec());
    let mut is_locked = vec![false; n];
    let mut num_unlocks = vec![0u8; n];
    loop {
        if !tridiagonal(a, b, c, d, h) {
            return false;
        }
        let (mut all, mut locked) = (false, false);
        let mut overshoot;
        loop {
            overshoot = false;
            for i in 0..n {
                if h[i] >= hmin[i] && h[i] <= hmax[i] {
                    continue;
                }
                overshoot = true;
                let target = if h[i] > hmax[i] { hmax[i] } else { hmin[i] };
                // only those that go the right way, if there are any
                if target != 0.0 || all {
                    is_locked[i] = true;
                    lock(a, b, c, d, i, target);
                    locked = true;
                }
            }
            all = true;
            if !(overshoot && !locked) {
                break;
            }
        }
        let mut unlocked = false;
        if !locked {
            for i in 0..n {
                if !is_locked[i] || num_unlocks[i] >= 2 {
                    continue;
                }
                // a handle that wants to move back within its limits is let go
                let state =
                    a0[i] * h[(i + n - 1) % n] + b0[i] * h[i] + c0[i] * h[(i + 1) % n] - d0[i];
                let relax = -state * b0[i];
                if (relax > 0.0 && h[i] < hmax[i]) || (relax < 0.0 && h[i] > hmin[i]) {
                    a[i] = a0[i];
                    b[i] = b0[i];
                    c[i] = c0[i];
                    d[i] = d0[i];
                    is_locked[i] = false;
                    num_unlocks[i] += 1;
                    unlocked = true;
                }
            }
        }
        if !(overshoot || unlocked) {
            return true;
        }
    }
}

/// `BLI_tridiagonal_solve`: the Thomas algorithm, in double precision over single-precision rows.
fn tridiagonal(a: &[f32], b: &[f32], c: &[f32], d: &[f32], x: &mut [f32]) -> bool {
    let n = a.len();
    let (mut c1, mut d1) = (vec![0.0_f64; n], vec![0.0_f64; n]);
    let mut c_prev = c[0] as f64 / b[0] as f64;
    let mut d_prev = d[0] as f64 / b[0] as f64;
    (c1[0], d1[0]) = (c_prev, d_prev);
    for i in 1..n {
        let denum = b[i] as f64 - a[i] as f64 * c_prev;
        c_prev = c[i] as f64 / denum;
        d_prev = (d[i] as f64 - a[i] as f64 * d_prev) / denum;
        (c1[i], d1[i]) = (c_prev, d_prev);
    }
    let mut x_prev = d_prev;
    x[n - 1] = x_prev as f32;
    for i in (0..n - 1).rev() {
        x_prev = d1[i] - c1[i] * x_prev;
        x[i] = x_prev as f32;
    }
    x_prev.is_finite()
}

/// The F-curve through `b` at `time` (`fcurve_eval_keyframes`), held level beyond its ends.
fn eval(b: &[Bez], time: f32) -> f32 {
    let (first, last) = (b[0], b[b.len() - 1]);
    if time <= first.co[0] {
        return first.co[1];
    }
    if last.co[0] <= time {
        return last.co[1];
    }
    // the key the time is on or before (`BKE_fcurve_bezt_binarysearch_index_ex`)
    let near = |a: f32, b: f32| {
        if a > b {
            a - b <= 0.0001
        } else {
            b - a <= 0.0001
        }
    };
    let (mut lo, mut hi) = (0_i64, b.len() as i64);
    let mut at = None;
    if near(time, first.co[0]) {
        at = Some(0);
    } else if near(time, last.co[0]) {
        at = Some(b.len() - 1);
    } else {
        while lo <= hi {
            let mid = lo + (hi - lo) / 2;
            let midfra = b[mid as usize].co[0];
            if near(time, midfra) {
                at = Some(mid as usize);
                break;
            }
            if time > midfra {
                lo = mid + 1;
            } else {
                hi = mid - 1;
            }
        }
    }
    if let Some(k) = at {
        return b[k].co[1];
    }
    let bezt = b[lo as usize];
    let prev = b[lo as usize - 1];
    if (bezt.co[0] - time).abs() < 1e-8 {
        return bezt.co[1];
    }
    let begin = prev.co[1];
    let change = bezt.co[1] - prev.co[1];
    let duration = bezt.co[0] - prev.co[0];
    let t = time - prev.co[0];
    if duration == 0.0 {
        return prev.co[1];
    }
    match prev.ease {
        Ease::Smooth => {
            let (v1, mut v2, mut v3, v4) = (prev.co, prev.h2, bezt.h1, bezt.co);
            if (v1[1] - v4[1]).abs() < f32::EPSILON
                && (v2[1] - v3[1]).abs() < f32::EPSILON
                && (v3[1] - v4[1]).abs() < f32::EPSILON
            {
                return v1[1];
            }
            correct_bezpart(v1, &mut v2, &mut v3, v4);
            match findzero(time, v1[0], v2[0], v3[0], v4[0]) {
                Some(u) => berekeny(v1[1], v2[1], v3[1], v4[1], u),
                None => 0.0,
            }
        }
        Ease::Linear => change * t / duration + begin,
        Ease::In => {
            let t = t / duration;
            change * t * t + begin
        }
        Ease::Out => {
            let t = t / duration;
            -change * t * (t - 2.0) + begin
        }
        Ease::InOut => {
            let t = t / (duration / 2.0);
            if t < 1.0 {
                change / 2.0 * t * t + begin
            } else {
                let t = t - 1.0;
                -change / 2.0 * (t * (t - 2.0) - 1.0) + begin
            }
        }
        Ease::ExpoOut => {
            if t == 0.0 {
                return begin;
            }
            // (2^-10 at the end made up for, so it lands on the key)
            let (pow_min, pow_scale) = (0.000_976_562_5_f32, 1.0_f32 / (1.0 - 0.000_976_562_5_f32));
            change * (1.0 - (powf(2.0, -10.0 * t / duration) - pow_min) * pow_scale) + begin
        }
        Ease::BackOut => {
            let t = t / duration - 1.0;
            let s = prev.back;
            change * (t * t * ((s + 1.0) * t + s) + 1.0) + begin
        }
        Ease::BounceOut => {
            let mut t = t / duration;
            if t < 1.0 / 2.75 {
                return change * (7.5625 * t * t) + begin;
            }
            let lift = if t < 2.0 / 2.75 {
                t -= 1.5 / 2.75;
                0.75
            } else if t < 2.5 / 2.75 {
                t -= 2.25 / 2.75;
                0.9375
            } else {
                t -= 2.625 / 2.75;
                0.984375
            };
            change * ((7.5625 * t) * t + lift) + begin
        }
    }
}

/// `powf`, as the double rounded.
fn powf(x: f32, y: f32) -> f32 {
    crate::math::pow(x as f64, y as f64) as f32
}

/// Handles that would loop the curve back on itself pulled in to the next key's time (`BKE_fcurve_correct_bezpart`).
fn correct_bezpart(v1: [f32; 2], v2: &mut [f32; 2], v3: &mut [f32; 2], v4: [f32; 2]) {
    let h1 = [v1[0] - v2[0], v1[1] - v2[1]];
    let h2 = [v4[0] - v3[0], v4[1] - v3[1]];
    let len = v4[0] - v1[0];
    let (len1, len2) = (h1[0].abs(), h2[0].abs());
    if len1 + len2 == 0.0 {
        return;
    }
    if len1 > len {
        let fac = len / len1;
        *v2 = [v1[0] - fac * h1[0], v1[1] - fac * h1[1]];
    }
    if len2 > len {
        let fac = len / len2;
        *v3 = [v4[0] - fac * h2[0], v4[1] - fac * h2[1]];
    }
}

/// Where along the Bézier its time is `x`: the first root of the cubic in 0..1 (`findzero`, `solve_cubic`).
fn findzero(x: f32, q0: f32, q1: f32, q2: f32, q3: f32) -> Option<f32> {
    let c0 = (q0 - x) as f64;
    let c1 = (3.0 * (q1 - q0)) as f64;
    let c2 = (3.0 * (q0 - 2.0 * q1 + q2)) as f64;
    let c3 = (q3 - q0 + 3.0 * (q1 - q2)) as f64;
    let small = -1.0e-10_f64 as f32;
    let ok = |o: f32| o >= small && o <= 1.000001;
    let sqrt3d = |d: f64| {
        if d == 0.0 {
            0.0
        } else if d < 0.0 {
            -crate::math::exp(crate::math::ln(-d) / 3.0)
        } else {
            crate::math::exp(crate::math::ln(d) / 3.0)
        }
    };
    let mut roots: Vec<f32> = Vec::new();
    if c3 != 0.0 {
        let mut a = c2 / c3;
        let b = c1 / c3;
        let c = c0 / c3;
        a /= 3.0;
        let p = b / 3.0 - a * a;
        let q = (2.0 * a * a * a - a * b + c) / 2.0;
        let d = q * q + p * p * p;
        if d > 0.0 {
            let t = d.sqrt();
            roots.push((sqrt3d(-q + t) + sqrt3d(-q - t) - a) as f32);
        } else if d == 0.0 {
            let t = sqrt3d(-q);
            roots.push((2.0 * t - a) as f32);
            roots.push((-t - a) as f32);
        } else {
            let phi = crate::math::acos(-q / (-(p * p * p)).sqrt());
            let t = (-p).sqrt();
            let p = crate::math::cos(phi / 3.0);
            let q = (3.0 - 3.0 * p * p).sqrt();
            roots.push((2.0 * t * p - a) as f32);
            roots.push((-t * (p + q) - a) as f32);
            roots.push((-t * (p - q) - a) as f32);
        }
    } else if c2 != 0.0 {
        let (a, b, c) = (c2, c1, c0);
        let p = b * b - 4.0 * a * c;
        if p > 0.0 {
            let p = p.sqrt();
            roots.push(((-b - p) / (2.0 * a)) as f32);
            roots.push(((-b + p) / (2.0 * a)) as f32);
        } else if p == 0.0 {
            roots.push((-b / (2.0 * a)) as f32);
        }
    } else if c1 != 0.0 {
        roots.push((-c0 / c1) as f32);
    } else if c0 == 0.0 {
        return Some(0.0);
    }
    roots.into_iter().find(|&o| ok(o))
}

/// The Bézier's value at `t` along it (`berekeny`).
fn berekeny(f1: f32, f2: f32, f3: f32, f4: f32, t: f32) -> f32 {
    let c0 = f1;
    let c1 = 3.0 * (f2 - f1);
    let c2 = 3.0 * (f1 - 2.0 * f2 + f3);
    let c3 = f4 - f1 + 3.0 * (f2 - f3);
    c0 + t * c1 + t * t * c2 + t * t * t * c3
}

/// Blender's `back` for a back ease-out that goes `overshoot` (a share of the move) past its end: the peak of
/// 1 + (s + 1)u³ + su² (u = t - 1), found by halving.
pub fn back_amount(overshoot: f64) -> f64 {
    let peak = |s: f64| {
        let u = -2.0 * s / (3.0 * (s + 1.0));
        (s + 1.0) * u * u * u + s * u * u
    };
    let (mut lo, mut hi) = (0.0, 10.0);
    for _ in 0..60 {
        let mid = (lo + hi) / 2.0;
        if peak(mid) < overshoot {
            lo = mid
        } else {
            hi = mid
        }
    }
    (lo + hi) / 2.0
}

/// A part's (place, turn, scale) in clip `clip` at `frame`: its rest pose but for what the clip keys.
pub fn pose(obj: Obj, clip: Option<&str>, frame: f64) -> (V3, V3, V3) {
    obj.read(|o| {
        let (mut loc, mut rot, mut scale) = (o.loc, o.rot, o.scale);
        if let Some(track) = clip.and_then(|c| o.tracks.iter().find(|t| t.name == c)) {
            for ch in &track.channels {
                let v = ch.at(frame) as f32;
                match ch.path {
                    Path::Location => loc[ch.index] = v,
                    Path::Rotation => rot[ch.index] = v,
                    Path::Scale => scale[ch.index] = v,
                }
            }
        }
        (loc, rot, scale)
    })
}

/// Keys `obj`'s `path`[`index`] at the frames of `keys`, (frame, value) each, in its clip `clip`: several
/// channels of one clip share the track. `ease` from each key to the next.
pub fn key_channel(
    obj: Obj,
    clip: &str,
    path: Path,
    index: usize,
    keys: &[(i32, f64)],
    ease: Ease,
    overshoot: f64,
) -> Obj {
    let keys: Vec<(i32, f64, Ease)> = keys.iter().map(|&(f, v)| (f, v, ease)).collect();
    key_eased(obj, clip, path, index, &keys, overshoot)
}

/// Keys `obj`'s `path`[`index`] at the frames of `keys`, (frame, value, the ease from it to the next key) each, in
/// its clip `clip` (gamekit's `_clip`). `overshoot`: how far `BackOut` goes past the key it eases into, a share of
/// the move.
pub fn key_eased(
    obj: Obj,
    clip: &str,
    path: Path,
    index: usize,
    keys: &[(i32, f64, Ease)],
    overshoot: f64,
) -> Obj {
    let keys: Vec<Key> = keys
        .iter()
        .map(|&(frame, value, ease)| Key {
            frame,
            value: value as f32,
            ease,
        })
        .collect();
    let end = keys[keys.len() - 1].frame;
    obj.edit(|o| {
        let channel = Channel {
            path,
            index,
            keys,
            back: overshoot,
        };
        match o.tracks.iter_mut().find(|t| t.name == clip) {
            Some(track) => {
                track
                    .channels
                    .retain(|c| !(c.path == path && c.index == index));
                track.channels.push(channel);
                track.end = track.end.max(end);
            }
            None => o.tracks.push(Track {
                name: clip.to_string(),
                channels: vec![channel],
                end,
                peak: None,
            }),
        }
    });
    obj
}

fn rest(obj: Obj, path: Path, index: usize) -> f64 {
    obj.read(|o| match path {
        Path::Location => o.loc[index],
        Path::Rotation => o.rot[index],
        Path::Scale => o.scale[index],
    }) as f64
}

fn mark_peak(obj: Obj, clip: &str, frame: i32) -> Obj {
    obj.edit(|o| {
        if let Some(t) = o.tracks.iter_mut().find(|t| t.name == clip) {
            t.peak = Some(frame);
        }
    });
    obj
}

/// A scale: one number for every axis, or (x, y, z) Blender.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Factors(pub [f64; 3]);

impl From<f64> for Factors {
    fn from(k: f64) -> Factors {
        Factors([k; 3])
    }
}
impl From<i32> for Factors {
    fn from(k: i32) -> Factors {
        Factors([k as f64; 3])
    }
}
impl<A: crate::math::Num, B: crate::math::Num, C: crate::math::Num> From<(A, B, C)> for Factors {
    fn from(t: (A, B, C)) -> Factors {
        Factors([t.0.f(), t.1.f(), t.2.f()])
    }
}

options! {
    /// `squash`'s options.
    Squash { seconds: f64 = 0.6, back: bool = false, split: f64 = 0.5 }
}

/// A clip `clip` scaling `obj` about its own origin from rest to `scale` times its size: a number for every axis,
/// or (x, y, z) Blender. A cushion sinking (0.7 along Z), a bag flattening, a held instrument or weight hidden
/// while in the hands (0.001). The game plays a use's clip back on its own; `back` makes the clip come back to
/// rest by itself (a bounce, for a loop), at `split` of the way through (0.3: down quickly, up slowly, as a
/// cushion recovers).
pub fn squash(obj: Obj, clip: &str, scale: impl Into<Factors>, o: Squash) -> Obj {
    let factors = scale.into().0;
    let end = last_frame(o.seconds);
    let mid = (end - 1).min(2.max(1 + ((end - 1) as f64 * o.split).floor() as i32));
    let mut keyed = false;
    for (i, &k) in factors.iter().enumerate() {
        if k == 1.0 {
            continue;
        }
        let r = rest(obj, Path::Scale, i);
        let keys = if o.back {
            vec![(1, r), (mid, r * k), (end, r)]
        } else {
            vec![(1, r), (end, r * k)]
        };
        key_channel(obj, clip, Path::Scale, i, &keys, Ease::Smooth, OVERSHOOT);
        keyed = true;
    }
    assert!(keyed, "squash {clip:?}: a scale of 1 does nothing");
    mark_peak(obj, clip, if o.back { mid } else { end })
}

options! {
    /// `stages`' options: `sink` is (picked, growing, planted), metres the part goes down into the soil at each.
    Stages { seconds: f64 = 1.5, sink: Option<(f64, f64, f64)> = None }
}

/// A crop's growth clip `clip` (`motions.growth`) for `obj`: the scale of it at each growth stage, times its size
/// at rest, which is ripe. A number for every axis or (x, y, z) Blender; 0.001 hides it. The game holds the clip
/// where the stage is: ripe at rest, picked a third of the way, growing two thirds, planted at the end, and plays
/// it on from stage to stage. `sink`: so plants spread over a bed come up where they stand rather than shrink
/// towards its middle: scale them only in height and sink them, and what shows is each plant's top, in its place.
pub fn stages(
    obj: Obj,
    clip: &str,
    picked: impl Into<Factors>,
    growing: impl Into<Factors>,
    planted: impl Into<Factors>,
    o: Stages,
) -> Obj {
    let end = last_frame(o.seconds);
    let frames = [
        1.0,
        1.0 + (end - 1) as f64 / 3.0,
        1.0 + 2.0 * (end - 1) as f64 / 3.0,
        end as f64,
    ]
    .map(|f| crate::py::round(f) as i32);
    let each = [
        [1.0; 3],
        picked.into().0,
        growing.into().0,
        planted.into().0,
    ];
    let mut keyed = false;
    for i in 0..3 {
        if each.iter().any(|k| k[i] != 1.0) {
            let r = rest(obj, Path::Scale, i);
            let keys: Vec<(i32, f64)> = frames
                .iter()
                .zip(&each)
                .map(|(&f, k)| (f, r * k[i]))
                .collect();
            key_channel(obj, clip, Path::Scale, i, &keys, Ease::Smooth, OVERSHOOT);
            keyed = true;
        }
    }
    if let Some(sink) = o.sink.filter(|s| s.0 != 0.0 || s.1 != 0.0 || s.2 != 0.0) {
        let r = rest(obj, Path::Location, 2);
        let keys: Vec<(i32, f64)> = frames
            .iter()
            .zip([0.0, sink.0, sink.1, sink.2])
            .map(|(&f, d)| (f, r - d))
            .collect();
        key_channel(obj, clip, Path::Location, 2, &keys, Ease::Smooth, OVERSHOOT);
        keyed = true;
    }
    assert!(keyed, "stages {clip:?}: the same at every stage");
    obj
}

/// A part hidden at rest: a thousandth of its size, out of sight.
pub const FLAT: f64 = 0.001;

/// Hides `obj` at rest by flattening it along `axis` (Blender) to `FLAT`: a screen rolled up in its case on the wall, a
/// fire's flames at the foot of the log bed. `grow` runs it out to full size when the object is used, and `squash`
/// from here too.
pub fn flat(obj: Obj, axis: Axis) -> Obj {
    obj.edit(|o| o.scale[axis.index()] = FLAT as f32);
    obj
}

options! {
    /// `grow`'s and `grow_in_turn`'s options.
    Grow { seconds: f64 = 0.4, axis: Axis = Axis::Z }
}

/// A clip `clip` scaling `obj` about its own origin along `axis` (Blender) from where it is (built small and hidden) to
/// full size (scale 1): a part that appears while a resident uses the object (a stream of water).
pub fn grow(obj: Obj, clip: &str, o: Grow) -> Obj {
    let i = o.axis.index();
    let r = rest(obj, Path::Scale, i);
    key_channel(
        obj,
        clip,
        Path::Scale,
        i,
        &[(1, r), (last_frame(o.seconds), 1.0)],
        Ease::Smooth,
        OVERSHOOT,
    )
}

/// The frames that share `end` out among `n` parts, each its turn: from frame 1 to `end`.
fn bounds(end: i32, n: usize) -> Vec<i32> {
    (0..=n)
        .map(|k| 1 + py::round((end - 1) as f64 * k as f64 / n as f64) as i32)
        .collect()
}

/// Keys with one key a frame: a key on the frame of the one before it is left out.
fn one_a_frame(keys: &[(i32, f64)]) -> Vec<(i32, f64)> {
    keys.iter()
        .enumerate()
        .filter(|&(n, kv)| n == 0 || kv.0 != keys[n - 1].0)
        .map(|(_, &kv)| kv)
        .collect()
}

/// A clip `clip` in which `objs` grow one after another, the first first, each in its share of `seconds` (1.5 by
/// default), along `axis` from where it is (built small and hidden) to full size: the washing pegged out line by line.
/// Played along with a use (`over`), they come one at a time through it; played back, the last goes first.
pub fn grow_in_turn(objs: &[Obj], clip: &str, seconds: Option<f64>, axis: Axis) -> Vec<Obj> {
    let i = axis.index();
    let end = last_frame(seconds.unwrap_or(1.5));
    let b = bounds(end, objs.len());
    for (k, &obj) in objs.iter().enumerate() {
        let r = rest(obj, Path::Scale, i);
        let keys = one_a_frame(&[(1, r), (b[k], r), (b[k + 1], 1.0), (end, 1.0)]);
        key_channel(obj, clip, Path::Scale, i, &keys, Ease::Smooth, OVERSHOOT);
    }
    if let Some(&last) = objs.last() {
        mark_peak(last, clip, end);
    }
    objs.to_vec()
}

options! {
    /// `spin`'s options.
    Spin { turns: f64 = 1.0, seconds: f64 = 1.0, axis: Axis = Axis::Z }
}

/// A clip `clip` turning `obj` about its own `axis` (Blender) by whole `turns` (negative: the other way) at an even
/// speed, ending where it started: a wheel, a record platter, a fan. Keep it under about one turn a second (the
/// game blends between its samples); a part that looks the same turned (a fan's blades) can turn by less, e.g. a
/// quarter turn with four blades, and loop.
pub fn spin(obj: Obj, clip: &str, o: Spin) -> Obj {
    let i = o.axis.index();
    let r = rest(obj, Path::Rotation, i);
    let end = last_frame(o.seconds);
    key_channel(
        obj,
        clip,
        Path::Rotation,
        i,
        &[(1, r), (end, r + o.turns * 2.0 * PI)],
        Ease::Linear,
        OVERSHOOT,
    );
    mark_peak(obj, clip, py::round((1 + end) as f64 / 2.0) as i32)
}

/// A clip `clip` moving `obj` by `offset` (Blender x, y, z, metres) at an even speed, as a loop, in `seconds`
/// (1 by default): a treadmill's belt slats, a conveyor. The game jumps it home at the end of each round, unseen when
/// `offset` is the pattern's repeat (one slat on) and the part's ends are hidden under covers.
pub fn scroll(obj: Obj, clip: &str, offset: impl Into<P3>, seconds: Option<f64>) -> Obj {
    let offset: P3 = offset.into();
    let end = last_frame(seconds.unwrap_or(1.0));
    for (i, d) in offset.array().into_iter().enumerate() {
        if d != 0.0 {
            let r = rest(obj, Path::Location, i);
            key_channel(
                obj,
                clip,
                Path::Location,
                i,
                &[(1, r), (end, r + d)],
                Ease::Linear,
                OVERSHOOT,
            );
        }
    }
    mark_peak(obj, clip, py::round((1 + end) as f64 / 2.0) as i32)
}

options! {
    /// `rock`'s options.
    Rock { seconds: f64 = 1.0, axis: Axis = Axis::X }
}

/// A clip `clip` rocking `obj` about its own `axis` (Blender): out by `degrees`, back past rest to the other side
/// and home, as a pendulum (a rocking chair, a swing, the seesaw). It ends where it started at the speed it started
/// with, so it loops smoothly.
pub fn rock(obj: Obj, clip: &str, degrees: f64, o: Rock) -> Obj {
    let i = o.axis.index();
    let r = rest(obj, Path::Rotation, i);
    let end = last_frame(o.seconds);
    let a = math::radians(degrees);
    let keys: Vec<(i32, f64)> = (1..=end)
        .map(|f| {
            (
                f,
                r + a * math::sin(2.0 * PI * (f - 1) as f64 / (end - 1) as f64),
            )
        })
        .collect();
    key_channel(obj, clip, Path::Rotation, i, &keys, Ease::Linear, OVERSHOOT);
    mark_peak(obj, clip, py::round(1.0 + (end - 1) as f64 / 4.0) as i32)
}

/// A clip `clip` in which `objs` take turns: each in its share of `seconds` (1 by default) rises by `offset`
/// (Blender x, y, z, metres) and settles back, the first one first; it ends where it starts, so it loops
/// (`motions.loops`, `idle`). A dance floor's two checkerboards of tiles lifting one after the other.
pub fn turns(objs: &[Obj], clip: &str, offset: impl Into<P3>, seconds: Option<f64>) -> Vec<Obj> {
    let offset: P3 = offset.into();
    let end = last_frame(seconds.unwrap_or(1.0));
    let b = bounds(end, objs.len());
    for (k, &obj) in objs.iter().enumerate() {
        let (lo, hi) = (b[k], b[k + 1]);
        for (i, d) in offset.array().into_iter().enumerate() {
            if d != 0.0 {
                let r = rest(obj, Path::Location, i);
                let keys = one_a_frame(&[
                    (1, r),
                    (lo, r),
                    (py::floordiv((lo + hi) as i64, 2) as i32, r + d),
                    (hi, r),
                    (end, r),
                ]);
                key_channel(obj, clip, Path::Location, i, &keys, Ease::Smooth, OVERSHOOT);
            }
        }
    }
    if let Some(&first) = objs.first() {
        mark_peak(first, clip, py::floordiv((b[0] + b[1]) as i64, 2) as i32);
    }
    objs.to_vec()
}

/// A key of a `path`: (seconds, offset from rest (Blender metres), turn from rest about the part's own X, Y and Z
/// (Blender degrees)).
pub type PathKey = (f64, (f64, f64, f64), (f64, f64, f64));

/// A clip `clip` taking `obj` along a path of `keys`: at each time, its offset from rest and its turn from rest; the
/// first key, at 0 s, is rest. For what moves along more than a line: the pool rack lifted off the balls, carried
/// over the rail and hung on the table's end; the robot vacuum's round (a loop: it ends at rest, its heading a whole
/// turn on, either way; `linear` keeps an even speed between the keys, else each eases).
pub fn path(obj: Obj, clip: &str, keys: &[PathKey], linear: bool) -> Obj {
    let zero = |p: (f64, f64, f64)| p.0 == 0.0 && p.1 == 0.0 && p.2 == 0.0;
    assert!(
        keys[0].0 == 0.0 && zero(keys[0].1) && zero(keys[0].2),
        "path {clip:?}: the first key is rest, at 0 s"
    );
    let frames: Vec<i32> = keys.iter().map(|k| frame(k.0)).collect();
    assert!(
        frames.windows(2).all(|w| w[1] > w[0]),
        "path {clip:?}: keys a frame or more apart, in order"
    );
    let ease = if linear { Ease::Linear } else { Ease::Smooth };
    let at = |p: (f64, f64, f64), i: usize| [p.0, p.1, p.2][i];
    let mut moved = false;
    for i in 0..3 {
        for (path, rot, scale) in [
            (Path::Location, false, 1.0),
            (Path::Rotation, true, math::radians(1.0)),
        ] {
            let of = |k: &PathKey| at(if rot { k.2 } else { k.1 }, i);
            if keys.iter().any(|k| of(k) != 0.0) {
                let r = rest(obj, path, i);
                let channel: Vec<(i32, f64)> = frames
                    .iter()
                    .zip(keys)
                    .map(|(&f, k)| (f, r + of(k) * scale))
                    .collect();
                key_channel(obj, clip, path, i, &channel, ease, OVERSHOOT);
                moved = true;
            }
        }
    }
    assert!(moved, "path {clip:?}: every key at rest");
    mark_peak(obj, clip, frames[frames.len() / 2])
}

options! {
    /// `wobble`'s options.
    Wobble { seconds: f64 = 1.0, axis: Axis = Axis::X, swings: f64 = 3.0, delay: f64 = 0.0 }
}

/// A clip `clip` that knocks `obj` swinging about its own `axis` (Blender) and lets it die away: out by `degrees`,
/// back past rest, `swings` times, each smaller, ending at rest, `delay` seconds in. The give in what hangs or stands
/// loose (a towel on its rail, a lamp's shade, a pan on its hook) as something beside it moves: `attach` it to a part
/// that moves and give both clips the same name.
pub fn wobble(obj: Obj, clip: &str, degrees: f64, o: Wobble) -> Obj {
    let i = o.axis.index();
    let r = rest(obj, Path::Rotation, i);
    let (start, end) = (frame(o.delay), last_frame(o.delay + o.seconds));
    let a = math::radians(degrees);
    let mut keys = vec![(1, r)];
    if start > 1 {
        keys.push((start, r));
    }
    let span = end - start;
    for f in start + 1..end {
        let t = (f - start) as f64 / span as f64;
        keys.push((
            f,
            r + a * math::exp(-3.0 * t) * math::sin(PI * o.swings * t),
        ));
    }
    keys.push((end, r));
    key_channel(obj, clip, Path::Rotation, i, &keys, Ease::Linear, OVERSHOOT);
    let swing = py::round(span as f64 / (2.0 * o.swings)).max(1) as i32;
    mark_peak(obj, clip, start + swing)
}

/// A clip `clip` keying `obj`'s `path`[`index`] (Blender axes) at `points`, (seconds, value, the ease from that key
/// to the next) each; the first at 0 s is rest. Values as Blender has them (metres, radians, scale). For a clip none
/// of the others make: water filling a bath from 0.5 s, a koi surfacing and diving.
pub fn keys(obj: Obj, clip: &str, path: Path, index: usize, points: &[(f64, f64, Ease)]) -> Obj {
    assert!(
        !points.is_empty() && points[0].0 == 0.0,
        "keys {clip:?}: the first key is at 0 s (rest)"
    );
    let frames: Vec<(i32, f64, Ease)> = points.iter().map(|p| (frame(p.0), p.1, p.2)).collect();
    assert!(
        frames.windows(2).all(|w| w[1].0 > w[0].0),
        "keys {clip:?}: keys a frame or more apart, in order"
    );
    key_eased(obj, clip, path, index, &frames, OVERSHOOT)
}

/// Puts moving part `obj` inside moving part `parent`, where it is: it goes with `parent`'s clips and plays its own on
/// top (a fridge's lever turning on its door, a record spinning as it swings up). Call it after `parts::settle` (which
/// moves every part on its own) and before the clips; both stay their own parts, so `obj` still costs its own draw.
/// Its own clips turn it about its own origin, in `parent`'s frame.
pub fn attach(obj: Obj, parent: Obj) -> Obj {
    let world = obj.matrix_world();
    let parent_world = parent.matrix_world();
    // (its place, turn and scale taken apart from where it is in the parent's frame, as Blender's
    // `BKE_object_apply_mat4` does when its world matrix is set)
    let inverse = parent_world.inverted().unwrap_or(M4::IDENTITY);
    let m = crate::math::mul_m4_m4m4(&inverse, &world);
    let (loc, rot, scale) = decompose(&m);
    obj.edit(|o| {
        o.parent = Some(parent);
        o.loc = loc;
        o.rot = rot;
        o.scale = scale;
        o.quat = None;
    });
    obj
}

/// A matrix's place, Euler XYZ turn and scale (`mat4_to_loc_rot_size` and the turn through a quaternion, as an
/// object's are).
fn decompose(m: &M4) -> (V3, V3, V3) {
    let col = |c: usize| V3::new(m.0[c][0], m.0[c][1], m.0[c][2]);
    let (mut rot, mut size) = ([V3::ZERO; 3], [0.0_f32; 3]);
    for c in 0..3 {
        let (n, len) = crate::math::normalize_v3(col(c));
        rot[c] = n;
        size[c] = len;
    }
    if crate::math::dot_v3(rot[0].cross(rot[1]), rot[2]) < 0.0 {
        for c in 0..3 {
            rot[c] = -rot[c];
            size[c] = -size[c];
        }
    }
    let q = mat3_normalized_to_quat([rot[0].array(), rot[1].array(), rot[2].array()]);
    let e = quat_to_eul(q);
    (
        V3::new(m.0[3][0], m.0[3][1], m.0[3][2]),
        e,
        V3::new(size[0], size[1], size[2]),
    )
}

/// Blender's `mat3_normalized_to_quat`: `m[column][row]`, a turned and mirrored one taken as turned.
fn mat3_normalized_to_quat(mut m: [[f32; 3]; 3]) -> [f32; 4] {
    let det = m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
        - m[1][0] * (m[0][1] * m[2][2] - m[0][2] * m[2][1])
        + m[2][0] * (m[0][1] * m[1][2] - m[0][2] * m[1][1]);
    if !det.is_finite() {
        m = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
    } else if det < 0.0 {
        for c in &mut m {
            for x in c.iter_mut() {
                *x = -*x;
            }
        }
    }
    let mut q = [0.0_f32; 4];
    // (Mike Day's method, the larger diagonal first, w kept non-negative)
    let (big, s_index, sign_neg, rest): (f32, usize, bool, [(usize, f32); 3]);
    if m[2][2] < 0.0 {
        if m[0][0] > m[1][1] {
            big = 1.0 + m[0][0] - m[1][1] - m[2][2];
            s_index = 1;
            sign_neg = m[1][2] < m[2][1];
            rest = [
                (0, m[1][2] - m[2][1]),
                (2, m[0][1] + m[1][0]),
                (3, m[2][0] + m[0][2]),
            ];
        } else {
            big = 1.0 - m[0][0] + m[1][1] - m[2][2];
            s_index = 2;
            sign_neg = m[2][0] < m[0][2];
            rest = [
                (0, m[2][0] - m[0][2]),
                (1, m[0][1] + m[1][0]),
                (3, m[1][2] + m[2][1]),
            ];
        }
    } else if m[0][0] < -m[1][1] {
        big = 1.0 - m[0][0] - m[1][1] + m[2][2];
        s_index = 3;
        sign_neg = m[0][1] < m[1][0];
        rest = [
            (0, m[0][1] - m[1][0]),
            (1, m[2][0] + m[0][2]),
            (2, m[1][2] + m[2][1]),
        ];
    } else {
        big = 1.0 + m[0][0] + m[1][1] + m[2][2];
        s_index = 0;
        sign_neg = false;
        rest = [
            (1, m[1][2] - m[2][1]),
            (2, m[2][0] - m[0][2]),
            (3, m[0][1] - m[1][0]),
        ];
    }
    let mut s = 2.0 * big.sqrt();
    if sign_neg {
        s = -s;
    }
    q[s_index] = 0.25 * s;
    let s = 1.0 / s;
    for (i, v) in rest {
        q[i] = v * s;
    }
    if big == 1.0 && rest.iter().all(|&(i, _)| q[i] == 0.0) {
        q[s_index] = 1.0;
    }
    let len_squared = q[0] * q[0] + q[1] * q[1] + q[2] * q[2] + q[3] * q[3];
    if (len_squared - 1.0).abs() >= 0.0002 * 3.0 {
        let k = 1.0 / len_squared.sqrt();
        for x in &mut q {
            *x *= k;
        }
    }
    q
}

/// Blender's `quat_to_eulO` for the XYZ order: through the turn's matrix, the smaller of its two Euler readings.
fn quat_to_eul(q: [f32; 4]) -> V3 {
    let m = crate::math::Quat {
        w: q[0],
        x: q[1],
        y: q[2],
        z: q[3],
    }
    .to_matrix()
    .0;
    let atan2f = |y: f32, x: f32| math::atan2(y as f64, x as f64) as f32;
    let cy = py::hypot(m[0][0] as f64, m[0][1] as f64) as f32;
    let (e1, e2) = if cy > 0.0000375_f64 as f32 {
        (
            [
                atan2f(m[1][2], m[2][2]),
                atan2f(-m[0][2], cy),
                atan2f(m[0][1], m[0][0]),
            ],
            [
                atan2f(-m[1][2], -m[2][2]),
                atan2f(-m[0][2], -cy),
                atan2f(-m[0][1], -m[0][0]),
            ],
        )
    } else {
        let e = [atan2f(-m[2][1], m[1][1]), atan2f(-m[0][2], cy), 0.0];
        (e, e)
    };
    let d = |e: [f32; 3]| e[0].abs() + e[1].abs() + e[2].abs();
    let e = if d(e1) > d(e2) { e2 } else { e1 };
    V3::new(e[0], e[1], e[2])
}

/// [(part, clip name, last frame)] for every clip on `objs`.
pub fn clips(objs: &[Obj]) -> Vec<(Obj, String, i32)> {
    let mut out = Vec::new();
    for &o in objs {
        o.read(|p| out.extend(p.tracks.iter().map(|t| (o, t.name.clone(), t.end))));
    }
    out
}

/// Whether a clip takes `obj` away from its rest: moved over 3 cm, turned over 8 degrees, or scaled to under half
/// or over twice (grown from nothing, hidden while held). A cushion squashed a little stays.
pub fn moves_away(obj: Obj) -> bool {
    let tracks: Vec<(String, i32)> =
        obj.read(|o| o.tracks.iter().map(|t| (t.name.clone(), t.end)).collect());
    let (loc0, rot0, scale0) = pose(obj, None, 1.0);
    let rest_turn = crate::math::object_matrix(V3::ZERO, rot0, crate::math::v3(1, 1, 1));
    for (clip, last) in tracks {
        let step = (last / 12).max(1);
        let mut f = 1;
        while f <= last {
            let (loc, rot, scale) = pose(obj, Some(&clip), f as f64);
            if (loc - loc0).length() > 0.03 {
                return true;
            }
            if turn_between(
                &rest_turn,
                &crate::math::object_matrix(V3::ZERO, rot, crate::math::v3(1, 1, 1)),
            ) > crate::math::radians(8.0)
            {
                return true;
            }
            let ratios: Vec<f64> = (0..3)
                .map(|i| scale[i] as f64 / (scale0[i] as f64).max(1e-9))
                .collect();
            let lowest = ratios.iter().copied().fold(f64::INFINITY, f64::min);
            let highest = ratios.iter().copied().fold(f64::NEG_INFINITY, f64::max);
            if lowest < 0.5
                || highest > 2.0
                || (0..3).map(|i| scale0[i]).fold(f32::INFINITY, f32::min) < 0.01
            {
                return true;
            }
            f += step;
        }
    }
    false
}

/// The angle of the turn from `a` to `b` (two turns, as matrices), radians.
fn turn_between(a: &M4, b: &M4) -> f64 {
    let (a, b) = (a.to_3x3().0, b.to_3x3().0);
    let mut trace = 0.0_f64;
    for (ca, cb) in a.iter().zip(&b) {
        for (x, y) in ca.iter().zip(cb) {
            trace += *x as f64 * *y as f64;
        }
    }
    crate::math::acos(((trace - 1.0) / 2.0).clamp(-1.0, 1.0))
}

/// Whether any of the scene's parts has a clip.
pub fn any_clips() -> bool {
    scene::with(|s| s.objects.iter().any(|o| o.alive_with_tracks()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mesh::Mesh;

    fn part() -> Obj {
        scene::reset();
        scene::add("part", Mesh::default())
    }

    fn scale_at(obj: Obj, clip: &str, frame: i32) -> f64 {
        (pose(obj, Some(clip), frame as f64).2.z as f64 * 1e6).round() / 1e6
    }

    #[test]
    fn a_squash_eases_as_blenders_keys_do() {
        // the apple's `pick` (0.8 s to a thousandth), as Blender sampled it into models/trees/apple.glb
        let obj = squash(
            part(),
            "pick",
            0.001,
            Squash {
                seconds: 0.8,
                ..Squash::default()
            },
        );
        assert_eq!(clips(&[obj]), [(obj, "pick".to_string(), 25)]);
        let got: Vec<f64> = [1, 2, 3, 4, 5, 24, 25]
            .iter()
            .map(|&f| scale_at(obj, "pick", f))
            .collect();
        assert_eq!(
            got,
            [1.0, 0.994941, 0.980344, 0.957074, 0.926, 0.006059, 0.001]
        );
        assert!(moves_away(obj));
    }

    #[test]
    fn growth_stages_hold_a_third_of_the_way_each() {
        // the apple's `ripen`: gone from picked on, 1.5 s
        let obj = stages(part(), "ripen", 0.001, 0.001, 0.001, Stages::default());
        let got: Vec<f64> = [1, 2, 3, 4, 5, 16, 31, 46]
            .iter()
            .map(|&f| scale_at(obj, "ripen", f))
            .collect();
        assert_eq!(
            got,
            [
                1.0, 0.987272, 0.951456, 0.896104, 0.824768, 0.001, 0.001, 0.001
            ]
        );
        // a plant that only shrinks in height and sinks: its place keyed too
        let obj = stages(
            part(),
            "grow",
            1,
            (1, 1, 0.8),
            (1, 1, 0.5),
            Stages {
                sink: Some((0.0, 0.1, 0.32)),
                ..Stages::default()
            },
        );
        let track = obj.read(|o| o.tracks[0].clone());
        assert_eq!(
            track
                .channels
                .iter()
                .map(|c| (c.path, c.index))
                .collect::<Vec<_>>(),
            [(Path::Scale, 2), (Path::Location, 2)]
        );
        assert_eq!(pose(obj, Some("grow"), 46.0).0.z, -0.32);
        assert!((pose(obj, Some("grow"), 31.0).2.z - 0.8).abs() < 1e-7);
        assert!(!moves_away(squash(
            part(),
            "sit",
            (1, 1, 0.7),
            Squash::default()
        )));
    }

    #[test]
    fn frames_count_from_one_and_round_halves_up() {
        assert_eq!(
            (
                frame(0.0),
                frame(0.8),
                frame(1.5),
                frame(0.05),
                last_frame(0.0)
            ),
            (1, 25, 46, 3, 2)
        );
        assert!((back_amount(OVERSHOOT) - 1.0).abs() < 0.2);
    }
}
