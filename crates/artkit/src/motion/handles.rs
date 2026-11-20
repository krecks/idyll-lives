//! The bezier handles of a curve: how they are found, smoothed and limited.
//!
//! Split from `motion.rs` (see `mod.rs`).

use super::*;

/// A key on an F-curve: its left handle, its place (frame, value), its right handle.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Bez {
    pub(crate) h1: [f32; 2],
    pub(crate) co: [f32; 2],
    pub(crate) h2: [f32; 2],
    pub(crate) ease: Ease,
    pub(crate) back: f32,
    /// Its handles stay as they are (a peak, a trough, an end): Blender's `HD_AUTOTYPE_LOCKED_FINAL`.
    pub(crate) locked: bool,
}

/// Blender's auto-clamped handles of a key (`calchandleNurb_intern` for an F-curve smoothed for continuous
/// acceleration): a third of the way to each neighbour, level at a peak or a trough, never past a neighbour's value.
pub(crate) fn calc_handles(bezt: &mut Bez, prev: Option<[f32; 2]>, next: Option<[f32; 2]>) {
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
pub(crate) fn smooth_handles(b: &mut [Bez]) {
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
