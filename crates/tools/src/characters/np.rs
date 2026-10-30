//! The numpy and scipy pieces the generator used, with numpy's edge rules (so the port draws what
//! the Python did): `linspace`, `interp` (with `period=`), `gradient`, `percentile`, `median`,
//! `unique` by rows, a stable `argsort`, `lstsq`, pairwise sums, Python's `%` and `round`.

use nalgebra::{DMatrix, DVector};

use super::V3;

pub const TAU: f64 = 2.0 * std::f64::consts::PI;
pub const PI: f64 = std::f64::consts::PI;

/// `np.radians`: `x * (pi / 180)`.
pub fn radians(x: f64) -> f64 {
    x * (PI / 180.0)
}

/// `np.degrees`.
pub fn degrees(x: f64) -> f64 {
    x * (180.0 / PI)
}

/// `np.sign` (0 at 0, unlike `f64::signum`).
pub fn sign(x: f64) -> f64 {
    if x > 0.0 {
        1.0
    } else if x < 0.0 {
        -1.0
    } else {
        x
    }
}

/// `np.clip` of a scalar.
pub fn clip(x: f64, lo: f64, hi: f64) -> f64 {
    x.max(lo).min(hi)
}

/// `t * t * (3 - 2 t)` of `(x - a) / (b - a)` clipped to 0..1 (garments.py and prims.py).
pub fn smoothstep(a: f64, b: f64, x: f64) -> f64 {
    let t = clip((x - a) / (b - a), 0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Python's float `%` and `np.remainder`: the result takes the divisor's sign.
pub fn py_mod(a: f64, b: f64) -> f64 {
    let m = a % b;
    if m != 0.0 {
        if (b < 0.0) != (m < 0.0) { m + b } else { m }
    } else {
        0.0f64.copysign(b)
    }
}

/// `np.linspace(a, b, n)` (endpoint included): `i * step + a`, the last exactly `b`.
pub fn linspace(a: f64, b: f64, n: usize) -> Vec<f64> {
    let step = (b - a) / (n as f64 - 1.0);
    let mut out: Vec<f64> = (0..n).map(|i| i as f64 * step + a).collect();
    if n > 1 {
        out[n - 1] = b;
    }
    out
}

/// `np.linspace(a, b, n, endpoint=False)`.
pub fn linspace_open(a: f64, b: f64, n: usize) -> Vec<f64> {
    let step = (b - a) / n as f64;
    (0..n).map(|i| i as f64 * step + a).collect()
}

/// numpy's pairwise summation (`np.sum` of a 1-D array or along a contiguous axis).
pub fn pairwise_sum(a: &[f64]) -> f64 {
    let n = a.len();
    if n < 8 {
        let mut res = 0.0;
        for &v in a {
            res += v;
        }
        res
    } else if n <= 128 {
        let mut r = [0.0; 8];
        r.copy_from_slice(&a[..8]);
        let mut i = 8;
        while i < n - n % 8 {
            for j in 0..8 {
                r[j] += a[i + j];
            }
            i += 8;
        }
        let mut res = ((r[0] + r[1]) + (r[2] + r[3])) + ((r[4] + r[5]) + (r[6] + r[7]));
        while i < n {
            res += a[i];
            i += 1;
        }
        res
    } else {
        let mut n2 = n / 2;
        n2 -= n2 % 8;
        pairwise_sum(&a[..n2]) + pairwise_sum(&a[n2..])
    }
}

/// `np.mean` of a 1-D array.
pub fn mean(a: &[f64]) -> f64 {
    pairwise_sum(a) / a.len() as f64
}

/// `x.mean(0)` of points (n, 3): numpy sums the rows in order.
pub fn mean_rows(p: &[V3]) -> V3 {
    let mut s = V3::zeros();
    for q in p {
        s += q;
    }
    s / p.len() as f64
}

/// `np.interp(x, xp, fp)` (xp increasing): fp[0] / fp[-1] outside, numpy's slope form inside.
pub fn interp(x: f64, xp: &[f64], fp: &[f64]) -> f64 {
    let n = xp.len();
    if x.is_nan() {
        return x;
    }
    if n == 1 {
        return fp[0];
    }
    if x > xp[n - 1] {
        return fp[n - 1];
    }
    if x < xp[0] {
        return fp[0];
    }
    // The largest j with xp[j] <= x.
    let j = xp.partition_point(|&v| v <= x) - 1;
    if j == n - 1 || xp[j] == x {
        return fp[j];
    }
    let slope = (fp[j + 1] - fp[j]) / (xp[j + 1] - xp[j]);
    // numpy's arm64 build fuses this multiply-add.
    let r = slope.mul_add(x - xp[j], fp[j]);
    if r.is_nan() {
        let r2 = slope.mul_add(x - xp[j + 1], fp[j + 1]);
        if r2.is_nan() && fp[j] == fp[j + 1] {
            return fp[j];
        }
        return r2;
    }
    r
}

/// `np.interp(x, xp, fp, period=p)`, its table prepared once: xp taken mod p, sorted, and padded
/// with the last point a period down and the first a period up.
pub struct Periodic {
    xp: Vec<f64>,
    fp: Vec<f64>,
    period: f64,
}

impl Periodic {
    pub fn new(xp: &[f64], fp: &[f64], period: f64) -> Self {
        let period = period.abs();
        let xm: Vec<f64> = xp.iter().map(|&v| py_mod(v, period)).collect();
        let o = argsort(&xm);
        let xs: Vec<f64> = o.iter().map(|&i| xm[i]).collect();
        let fs: Vec<f64> = o.iter().map(|&i| fp[i]).collect();
        let n = xs.len();
        let mut x = vec![xs[n - 1] - period];
        x.extend(&xs);
        x.push(xs[0] + period);
        let mut f = vec![fs[n - 1]];
        f.extend(&fs);
        f.push(fs[0]);
        Periodic {
            xp: x,
            fp: f,
            period,
        }
    }

    pub fn at(&self, x: f64) -> f64 {
        interp(py_mod(x, self.period), &self.xp, &self.fp)
    }
}

/// `np.gradient(P, axis=0)` of points: central differences inside, one-sided at the ends.
pub fn gradient(p: &[V3]) -> Vec<V3> {
    let n = p.len();
    let mut out = vec![V3::zeros(); n];
    for i in 1..n - 1 {
        out[i] = (p[i + 1] - p[i - 1]) / 2.0;
    }
    out[0] = p[1] - p[0];
    out[n - 1] = p[n - 1] - p[n - 2];
    out
}

/// A stable argsort (ties keep their order; numpy's default quicksort doesn't promise that, but
/// nothing here sorts ties whose order shows).
pub fn argsort(a: &[f64]) -> Vec<usize> {
    let mut o: Vec<usize> = (0..a.len()).collect();
    o.sort_by(|&i, &j| a[i].total_cmp(&a[j]));
    o
}

/// `np.argsort(v)` exactly as numpy orders ties: its default (unstable) introsort, the scalar
/// `aquicksort` of npysort/quicksort.cpp (numpy has no SIMD argsort on arm64). Used where the order
/// of exact ties shows in the output (`meshops::top4`: equal weights on mirrored bones).
pub fn argsort_numpy(v: &[f64]) -> Vec<usize> {
    let less = |a: f64, b: f64| a < b || (b.is_nan() && !a.is_nan());
    let num = v.len();
    let mut t: Vec<usize> = (0..num).collect();
    if num < 2 {
        return t;
    }
    let (mut pl, mut pr) = (0usize, num - 1);
    let mut stack: Vec<(usize, usize, i32)> = Vec::new();
    let mut cdepth = 2 * (usize::BITS - 1 - num.leading_zeros()) as i32;
    loop {
        if cdepth < 0 {
            aheapsort(v, &mut t[pl..=pr], less);
        } else {
            while pr - pl > 15 {
                let pm = pl + ((pr - pl) >> 1);
                if less(v[t[pm]], v[t[pl]]) {
                    t.swap(pm, pl);
                }
                if less(v[t[pr]], v[t[pm]]) {
                    t.swap(pr, pm);
                }
                if less(v[t[pm]], v[t[pl]]) {
                    t.swap(pm, pl);
                }
                let vp = v[t[pm]];
                let (mut pi, mut pj) = (pl, pr - 1);
                t.swap(pm, pj);
                loop {
                    pi += 1;
                    while less(v[t[pi]], vp) {
                        pi += 1;
                    }
                    pj -= 1;
                    while less(vp, v[t[pj]]) {
                        pj -= 1;
                    }
                    if pi >= pj {
                        break;
                    }
                    t.swap(pi, pj);
                }
                t.swap(pi, pr - 1);
                // Push the larger partition, go on with the smaller.
                cdepth -= 1;
                if pi - pl < pr - pi {
                    stack.push((pi + 1, pr, cdepth));
                    pr = pi - 1;
                } else {
                    stack.push((pl, pi - 1, cdepth));
                    pl = pi + 1;
                }
            }
            // Insertion sort.
            for pi in pl + 1..=pr {
                let vi = t[pi];
                let vp = v[vi];
                let mut pj = pi;
                while pj > pl && less(vp, v[t[pj - 1]]) {
                    t[pj] = t[pj - 1];
                    pj -= 1;
                }
                t[pj] = vi;
            }
        }
        match stack.pop() {
            Some((l, r, d)) => {
                pl = l;
                pr = r;
                cdepth = d;
            }
            None => break,
        }
    }
    t
}

/// numpy's `aheapsort` (introsort's fallback), on a slice of indices.
fn aheapsort(v: &[f64], a: &mut [usize], less: impl Fn(f64, f64) -> bool) {
    let n = a.len();
    // 1-based, as numpy's.
    let sift = |a: &mut [usize], mut i: usize, n: usize, tmp: usize| {
        let mut j = i << 1;
        while j <= n {
            if j < n && less(v[a[j - 1]], v[a[j]]) {
                j += 1;
            }
            if less(v[tmp], v[a[j - 1]]) {
                a[i - 1] = a[j - 1];
                i = j;
                j += j;
            } else {
                break;
            }
        }
        a[i - 1] = tmp;
    };
    let mut l = n >> 1;
    while l > 0 {
        let tmp = a[l - 1];
        sift(a, l, n, tmp);
        l -= 1;
    }
    let mut n = n;
    while n > 1 {
        let tmp = a[n - 1];
        a[n - 1] = a[0];
        n -= 1;
        sift(a, 1, n, tmp);
    }
}

/// `np.percentile(a, q)`, linear: numpy's lerp between the neighbouring order statistics.
pub fn percentile(a: &[f64], q: f64) -> f64 {
    let mut s = a.to_vec();
    s.sort_by(f64::total_cmp);
    let n = s.len();
    let vi = (n as f64 - 1.0) * (q / 100.0);
    let prev = vi.floor();
    let lo = (prev as usize).min(n - 1);
    let hi = (lo + 1).min(n - 1);
    let t = vi - prev;
    let (a, b) = (s[lo], s[hi]);
    let d = b - a;
    if t >= 0.5 {
        b - d * (1.0 - t)
    } else {
        a + d * t
    }
}

/// `np.median`.
pub fn median(a: &[f64]) -> f64 {
    let mut s = a.to_vec();
    s.sort_by(f64::total_cmp);
    let n = s.len();
    if n % 2 == 1 {
        s[n / 2]
    } else {
        (s[n / 2 - 1] + s[n / 2]) / 2.0
    }
}

/// `np.unique(rows, axis=0, return_inverse=True, return_counts=True)`: the distinct rows in
/// lexicographic order, each row's index among them, and how often each occurs.
pub fn unique_rows<T: Ord + Copy, const K: usize>(
    rows: &[[T; K]],
) -> (Vec<[T; K]>, Vec<usize>, Vec<usize>) {
    let mut o: Vec<usize> = (0..rows.len()).collect();
    o.sort_by(|&i, &j| rows[i].cmp(&rows[j]));
    let mut uniq: Vec<[T; K]> = Vec::new();
    let mut counts = Vec::new();
    let mut inv = vec![0; rows.len()];
    for &i in &o {
        if uniq.last() != Some(&rows[i]) {
            uniq.push(rows[i]);
            counts.push(0);
        }
        inv[i] = uniq.len() - 1;
        *counts.last_mut().unwrap() += 1;
    }
    (uniq, inv, counts)
}

/// `np.unique` of a flat array: sorted, distinct.
pub fn unique(a: &[usize]) -> Vec<usize> {
    let mut v = a.to_vec();
    v.sort_unstable();
    v.dedup();
    v
}

/// `np.linalg.lstsq(A, b)` (A full column rank), by SVD as LAPACK's gelsd.
pub fn lstsq(a: &DMatrix<f64>, b: &DVector<f64>) -> DVector<f64> {
    let eps = f64::EPSILON * a.nrows().max(a.ncols()) as f64;
    let svd = a.clone().svd(true, true);
    let smax = svd.singular_values.max();
    svd.solve(b, eps * smax).expect("lstsq")
}

/// Python's `round(x, nd)`: the correctly rounded decimal (ties to even) read back.
pub fn py_round(x: f64, nd: usize) -> f64 {
    format!("{x:.nd$}").parse().unwrap()
}

/// `np.round(x, nd)`: `rint(x * 10^nd) / 10^nd`.
pub fn np_round(x: f64, nd: i32) -> f64 {
    let f = 10f64.powi(nd);
    (x * f).round_ties_even() / f
}

/// Python's float `repr`: the shortest digits that read back, fixed between 1e-4 and 1e16,
/// otherwise scientific with a two-digit exponent (`1e-05`), and `.0` on whole numbers.
pub fn py_repr(x: f64) -> String {
    if x.is_nan() {
        return "NaN".into();
    }
    if x.is_infinite() {
        return if x > 0.0 {
            "Infinity".into()
        } else {
            "-Infinity".into()
        };
    }
    if x == 0.0 {
        return if x.is_sign_negative() {
            "-0.0".into()
        } else {
            "0.0".into()
        };
    }
    let e = format!("{x:e}");
    let (mant, exp) = e.split_once('e').unwrap();
    let exp: i32 = exp.parse().unwrap();
    let neg = mant.starts_with('-');
    let digits: String = mant.chars().filter(|c| c.is_ascii_digit()).collect();
    let sign = if neg { "-" } else { "" };
    if (-4..16).contains(&exp) {
        let n = digits.len() as i32;
        let s = if exp >= 0 {
            if n <= exp + 1 {
                format!("{digits}{}.0", "0".repeat((exp + 1 - n) as usize))
            } else {
                format!(
                    "{}.{}",
                    &digits[..(exp + 1) as usize],
                    &digits[(exp + 1) as usize..]
                )
            }
        } else {
            format!("0.{}{digits}", "0".repeat((-exp - 1) as usize))
        };
        format!("{sign}{s}")
    } else {
        let m = if digits.len() > 1 {
            format!("{}.{}", &digits[..1], &digits[1..])
        } else {
            digits
        };
        let es = if exp < 0 { '-' } else { '+' };
        format!("{sign}{m}e{es}{:02}", exp.abs())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interp_edges() {
        let xp = [0.0, 1.0, 3.0];
        let fp = [1.0, 3.0, -1.0];
        assert_eq!(interp(-1.0, &xp, &fp), 1.0);
        assert_eq!(interp(5.0, &xp, &fp), -1.0);
        assert_eq!(interp(1.0, &xp, &fp), 3.0);
        assert_eq!(interp(3.0, &xp, &fp), -1.0);
        assert_eq!(interp(0.5, &xp, &fp), 2.0);
        assert_eq!(interp(2.0, &xp, &fp), 1.0);
        // np.interp(x, [-3, -1, 1, 3], [0, 1, 2, 3], period=8) at x = 3.5, -3.5, 7, 4.
        let p = Periodic::new(&[-3.0, -1.0, 1.0, 3.0], &[0.0, 1.0, 2.0, 3.0], 8.0);
        assert_eq!(
            [p.at(3.5), p.at(-3.5), p.at(7.0), p.at(4.0), p.at(0.0)],
            [2.25, 0.75, 1.0, 1.5, 1.5]
        );
    }

    #[test]
    fn unique_by_rows() {
        let rows = [[2, 1], [0, 5], [2, 1], [0, 3]];
        let (u, inv, cnt) = unique_rows(&rows);
        assert_eq!(u, vec![[0, 3], [0, 5], [2, 1]]);
        assert_eq!(inv, vec![2, 1, 2, 0]);
        assert_eq!(cnt, vec![1, 1, 2]);
        assert_eq!(unique(&[3, 1, 3, 2]), vec![1, 2, 3]);
    }

    #[test]
    fn gradient_edges() {
        let p: Vec<V3> = [0.0, 1.0, 4.0, 9.0]
            .iter()
            .map(|&x| V3::new(x, 0.0, -x))
            .collect();
        let g: Vec<f64> = gradient(&p).iter().map(|v| v.x).collect();
        assert_eq!(g, [1.0, 2.0, 4.0, 5.0]);
    }

    #[test]
    fn percentile_median() {
        // np.percentile([5, 1, 4, 2, 3, 10], 40) = 3.0; at 30: 2.5; np.median: 3.5.
        let a = [5.0, 1.0, 4.0, 2.0, 3.0, 10.0];
        assert_eq!(percentile(&a, 40.0), 3.0);
        assert_eq!(percentile(&a, 30.0), 2.5);
        assert_eq!(percentile(&a, 90.0), 7.5);
        assert_eq!(median(&a), 3.5);
        assert_eq!(median(&a[..5]), 3.0);
    }

    #[test]
    fn rounding() {
        assert_eq!(
            [0.5f64, 1.5, 2.5, -0.5, -1.5].map(f64::round_ties_even),
            [0.0, 2.0, 2.0, -0.0, -2.0]
        );
        assert_eq!(py_round(0.123456789, 4), 0.1235);
        assert_eq!(py_round(2.675, 2), 2.67); // 2.675 is just below the tie
        assert_eq!(np_round(0.00125, 4), 0.0012);
        assert_eq!(py_repr(1e-5), "1e-05");
        assert_eq!(py_repr(4000.0), "4000.0");
        assert_eq!(py_repr(0.0001), "0.0001");
        assert_eq!(py_repr(-1.2345e-7), "-1.2345e-07");
        assert_eq!(py_repr(1.77), "1.77");
        assert_eq!(py_repr(1e16), "1e+16");
        assert_eq!(py_repr(123456.5), "123456.5");
    }

    #[test]
    fn mods_and_sums() {
        assert_eq!(py_mod(-1.0, 3.0), 2.0);
        assert_eq!(py_mod(1.0, -3.0), -2.0);
        assert_eq!(sign(0.0), 0.0);
        let a: Vec<f64> = (0..300).map(|i| (i as f64 * 0.37).sin()).collect();
        let naive: f64 = a.iter().sum();
        assert!((pairwise_sum(&a) - naive).abs() < 1e-12);
        assert_eq!(linspace(0.0, 1.0, 5), [0.0, 0.25, 0.5, 0.75, 1.0]);
        assert_eq!(argsort(&[3.0, 1.0, 3.0, 0.0]), [3, 1, 0, 2]);
    }

    #[test]
    fn argsort_like_numpy() {
        // np.argsort of these in numpy 2.4 (ties in numpy's own order).
        let mut a = vec![0.0; 33];
        a[10] = -0.5;
        a[12] = -0.25;
        a[13] = -0.25;
        assert_eq!(
            argsort_numpy(&a),
            [
                10, 13, 12, 0, 20, 21, 22, 19, 23, 25, 26, 27, 28, 29, 30, 24, 18, 16, 31, 15, 14,
                11, 9, 8, 7, 6, 5, 4, 3, 2, 1, 17, 32
            ]
        );
        let b: Vec<f64> = (0..33).map(|i| -((i % 3) as f64) * 0.1).collect();
        assert_eq!(
            argsort_numpy(&b),
            [
                32, 29, 2, 26, 5, 23, 8, 20, 11, 17, 14, 28, 25, 22, 19, 31, 16, 13, 10, 7, 4, 1,
                18, 12, 21, 9, 24, 6, 27, 3, 30, 15, 0
            ]
        );
        // Any order it gives is sorted.
        let c: Vec<f64> = (0..200).map(|i| ((i * 37) % 11) as f64).collect();
        let o = argsort_numpy(&c);
        assert!(o.windows(2).all(|w| c[w[0]] <= c[w[1]]));
    }
}
