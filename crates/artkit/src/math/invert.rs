//! Inverting a matrix, and the products that need the reciprocal.

use super::*;
use std::ops::Mul;

/// Blender's own inverse of an object's matrix (`invert_m4_m4`: Eigen's, for a 4 x 4 of singles): the matrix in
/// four 2 x 2 blocks, their adjugates and determinants, every step in single precision in Eigen's order. What a
/// join takes the active part's matrix back with; not `m.inverted()`, which is mathutils' and differs from it in
/// the last places. None for a matrix that flattens space.
pub fn invert_m4(m: &M4) -> Option<M4> {
    type P = [f32; 4];
    m.inverted()?; // (Eigen checks the determinant apart)
    let (l1, l2, l3, l4) = (m.0[0], m.0[1], m.0[2], m.0[3]);
    let sw = |a: P, b: P, p: usize, q: usize, r: usize, s: usize| -> P { [a[p], a[q], b[r], b[s]] };
    let movelh = |a: P, b: P| -> P { [a[0], a[1], b[0], b[1]] };
    let movehl = |a: P, b: P| -> P { [b[2], b[3], a[2], a[3]] };
    let mul = |a: P, b: P| -> P { std::array::from_fn(|i| a[i] * b[i]) };
    let sub = |a: P, b: P| -> P { std::array::from_fn(|i| a[i] - b[i]) };
    let add = |a: P, b: P| -> P { std::array::from_fn(|i| a[i] + b[i]) };
    let dup = |a: P, p: usize| -> P { [a[p]; 4] };
    // the four 2 x 2 blocks of the matrix
    let a = movelh(l1, l2);
    let b = movehl(l2, l1);
    let c = movelh(l3, l4);
    let d = movehl(l4, l3);
    // A# B and D# C (# the adjugate)
    let ab = mul(sw(a, a, 3, 3, 0, 0), b);
    let ab = sub(ab, mul(sw(a, a, 1, 1, 2, 2), sw(b, b, 2, 3, 0, 1)));
    let dc = mul(sw(d, d, 3, 3, 0, 0), c);
    let dc = sub(dc, mul(sw(d, d, 1, 1, 2, 2), sw(c, c, 2, 3, 0, 1)));
    // the blocks' determinants
    let det2 = |x: P| -> P {
        let t = mul(sw(x, x, 3, 3, 1, 1), x);
        sub(t, movehl(t, t))
    };
    let (da, db, dcc, dd) = (det2(a), det2(b), det2(c), det2(d));
    let t = mul(sw(dc, dc, 0, 2, 1, 3), ab);
    let t = add(t, movehl(t, t));
    let t = add(t, sw(t, t, 1, 0, 0, 0));
    // |A||D| + |B||C| - trace(A# B D# C), and one over it
    let det = dup(sub(add(mul(da, dd), mul(db, dcc)), t), 0);
    let rd: P = [reciprocal(det[0]); 4];
    let id = mul(sw(c, c, 0, 0, 2, 2), movelh(ab, ab));
    let id = add(id, mul(sw(c, c, 1, 1, 3, 3), movehl(ab, ab)));
    let id = sub(mul(d, dup(da, 0)), id);
    let ia = mul(sw(b, b, 0, 0, 2, 2), movelh(dc, dc));
    let ia = add(ia, mul(sw(b, b, 1, 1, 3, 3), movehl(dc, dc)));
    let ia = sub(mul(a, dup(dd, 0)), ia);
    let ib = mul(d, sw(ab, ab, 3, 0, 3, 0));
    let ib = sub(ib, mul(sw(d, d, 1, 0, 3, 2), sw(ab, ab, 2, 1, 2, 1)));
    let ib = sub(mul(c, dup(db, 0)), ib);
    let ic = mul(a, sw(dc, dc, 3, 0, 3, 0));
    let ic = sub(ic, mul(sw(a, a, 1, 0, 3, 2), sw(dc, dc, 2, 1, 2, 1)));
    let ic = sub(mul(b, dup(dcc, 0)), ic);
    let rd: P = [rd[0], -rd[1], -rd[2], rd[3]];
    let (ia, ib, ic, id) = (mul(ia, rd), mul(ib, rd), mul(ic, rd), mul(id, rd));
    Some(M4([
        sw(ia, ib, 3, 1, 3, 1),
        sw(ia, ib, 2, 0, 2, 0),
        sw(ic, id, 3, 1, 3, 1),
        sw(ic, id, 2, 0, 2, 0),
    ]))
}

/// One over `a` as Eigen works it out on this processor: the processor's own estimate (eight bits, from its
/// table), made finer twice by Newton's step, each step's `2 - a x` rounded once. Within a unit in the last place of
/// the quotient, not always it (one over the single just under a quarter is four).
fn reciprocal(a: f32) -> f32 {
    let bits = a.to_bits();
    let (sign, exp, fraction) = (bits & 0x8000_0000, (bits >> 23) & 0xff, bits & 0x007f_ffff);
    if exp == 0 || exp >= 253 {
        // (nothing a matrix that is turned back has: the plain quotient)
        return 1.0 / a;
    }
    // the estimate: from the nine leading bits of the number, as the processor's instruction gives it
    let scaled = 256 + (fraction >> 15);
    let estimate = ((1 << 19) / (scaled * 2 + 1)).div_ceil(2);
    let mut x = f32::from_bits(sign | ((253 - exp) << 23) | ((estimate & 0xff) << 15));
    for _ in 0..2 {
        let step = (2.0 - a as f64 * x as f64) as f32;
        x *= step;
    }
    x
}

/// Whether Blender leaves a mesh's points as they are for this matrix (`math::transform_points`): every entry
/// within a millionth of the identity's.
pub fn skips_transform(m: &M4) -> bool {
    (0..4).all(|c| (0..4).all(|r| (m.0[c][r] - M4::IDENTITY.0[c][r]).abs() <= 1e-6))
}

/// `m @ v`: the point moved; each row's products in single precision, summed in double.
impl Mul<V3> for M4 {
    type Output = V3;
    fn mul(self, v: V3) -> V3 {
        let m = self.0;
        let p = [v.x, v.y, v.z, 1.0];
        let row = |r: usize| {
            let mut d = 0.0_f64;
            for c in 0..4 {
                d += (m[c][r] * p[c]) as f64;
            }
            d as f32
        };
        V3::new(row(0), row(1), row(2))
    }
}

/// `a @ b`.
impl Mul<M4> for M4 {
    type Output = M4;
    fn mul(self, o: M4) -> M4 {
        let mut out = [[0.0_f32; 4]; 4];
        for (c, col) in out.iter_mut().enumerate() {
            for (r, cell) in col.iter_mut().enumerate() {
                let mut d = 0.0_f64;
                for i in 0..4 {
                    d += (self.0[i][r] * o.0[c][i]) as f64;
                }
                *cell = d as f32;
            }
        }
        M4(out)
    }
}

// ---- Blender's own matrix arithmetic: what moves a mesh's vertices ----
