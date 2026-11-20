//! The matrices: `M3`, `M4` and the axes.

use super::*;
use std::ops::Mul;

/// `mathutils.Matrix` of 3 x 3, kept as Blender keeps it: `m.0[column][row]`.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct M3(pub [[f32; 3]; 3]);

/// `mathutils.Matrix` of 4 x 4, kept as Blender keeps it: `m.0[column][row]`; the last column is where it moves to.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct M4(pub [[f32; 4]; 4]);

/// 'X', 'Y' or 'Z'.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Axis {
    X,
    Y,
    Z,
}

impl Axis {
    pub fn index(self) -> usize {
        self as usize
    }
}

pub(crate) fn angle_wrap(angle: f32) -> f32 {
    let (pi, tau) = (std::f32::consts::PI, std::f32::consts::PI * 2.0);
    let a = angle + pi;
    (a - tau * (a / tau).floor()) - pi
}

impl M3 {
    pub const IDENTITY: M3 = M3([[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]);

    /// `Matrix.Rotation(angle, 3, axis)`, radians.
    pub fn rotation(angle: f64, axis: Axis) -> M3 {
        let a = angle_wrap(angle as f32);
        let (c, s) = (cosf(a), sinf(a));
        M3(match axis {
            Axis::X => [[1.0, 0.0, 0.0], [0.0, c, s], [0.0, -s, c]],
            Axis::Y => [[c, 0.0, -s], [0.0, 1.0, 0.0], [s, 0.0, c]],
            Axis::Z => [[c, s, 0.0], [-s, c, 0.0], [0.0, 0.0, 1.0]],
        })
    }

    pub fn to_4x4(self) -> M4 {
        let m = self.0;
        M4([
            [m[0][0], m[0][1], m[0][2], 0.0],
            [m[1][0], m[1][1], m[1][2], 0.0],
            [m[2][0], m[2][1], m[2][2], 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    /// Row by row, as `mathutils` prints a matrix.
    pub fn rows(self) -> [f32; 9] {
        let m = self.0;
        [
            m[0][0], m[1][0], m[2][0], m[0][1], m[1][1], m[2][1], m[0][2], m[1][2], m[2][2],
        ]
    }

    pub fn transposed(self) -> M3 {
        let m = self.0;
        M3([
            [m[0][0], m[1][0], m[2][0]],
            [m[0][1], m[1][1], m[2][1]],
            [m[0][2], m[1][2], m[2][2]],
        ])
    }
}

/// `m @ v`: each row's products in single precision, summed in double.
impl Mul<V3> for M3 {
    type Output = V3;
    fn mul(self, v: V3) -> V3 {
        let m = self.0;
        let row = |r: usize| {
            let mut d = 0.0_f64;
            for c in 0..3 {
                d += (m[c][r] * v[c]) as f64;
            }
            d as f32
        };
        V3::new(row(0), row(1), row(2))
    }
}

/// `a @ b`.
impl Mul<M3> for M3 {
    type Output = M3;
    fn mul(self, o: M3) -> M3 {
        let mut out = [[0.0_f32; 3]; 3];
        for (c, col) in out.iter_mut().enumerate() {
            for (r, cell) in col.iter_mut().enumerate() {
                let mut d = 0.0_f64;
                for i in 0..3 {
                    d += (self.0[i][r] * o.0[c][i]) as f64;
                }
                *cell = d as f32;
            }
        }
        M3(out)
    }
}

impl M4 {
    pub const IDENTITY: M4 = M4([
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]);

    /// `Matrix.Translation(v)`.
    pub fn translation(v: V3) -> M4 {
        let mut m = M4::IDENTITY;
        m.0[3] = [v.x, v.y, v.z, 1.0];
        m
    }

    /// `Matrix.Diagonal((x, y, z, 1))`.
    pub fn diagonal(v: V3) -> M4 {
        let mut m = M4::IDENTITY;
        m.0[0][0] = v.x;
        m.0[1][1] = v.y;
        m.0[2][2] = v.z;
        m
    }

    /// `Matrix.Rotation(angle, 4, axis)`, radians.
    pub fn rotation(angle: f64, axis: Axis) -> M4 {
        M3::rotation(angle, axis).to_4x4()
    }

    pub fn to_3x3(self) -> M3 {
        let m = self.0;
        M3([
            [m[0][0], m[0][1], m[0][2]],
            [m[1][0], m[1][1], m[1][2]],
            [m[2][0], m[2][1], m[2][2]],
        ])
    }

    /// `m.translation`.
    pub fn location(self) -> V3 {
        V3::new(self.0[3][0], self.0[3][1], self.0[3][2])
    }

    /// Row by row, as `mathutils` prints a matrix.
    pub fn rows(self) -> [f32; 16] {
        let m = self.0;
        let mut out = [0.0; 16];
        for r in 0..4 {
            for c in 0..4 {
                out[r * 4 + c] = m[c][r];
            }
        }
        out
    }

    pub fn is_identity(self) -> bool {
        self == M4::IDENTITY
    }

    /// `m.inverted()`: the adjugate over the determinant; None for a matrix that flattens space.
    pub fn inverted(self) -> Option<M4> {
        let m = self.0;
        let det2 = |a: f32, b: f32, c: f32, d: f32| a * d - b * c;
        let det3 = |a: [f32; 3], b: [f32; 3], c: [f32; 3]| {
            a[0] * det2(b[1], b[2], c[1], c[2]) - b[0] * det2(a[1], a[2], c[1], c[2])
                + c[0] * det2(a[1], a[2], b[1], b[2])
        };
        // row `r` without column `skip`
        let row = |r: usize, skip: usize| {
            let v: Vec<f32> = (0..4).filter(|&c| c != skip).map(|c| m[c][r]).collect();
            [v[0], v[1], v[2]]
        };
        // the determinant of what is left without row `r` and column `c`
        let minor = |r: usize, c: usize| {
            let rows: Vec<[f32; 3]> = (0..4).filter(|&k| k != r).map(|k| row(k, c)).collect();
            det3(rows[0], rows[1], rows[2])
        };
        let det = m[0][0] * minor(0, 0) - m[0][1] * minor(1, 0) + m[0][2] * minor(2, 0)
            - m[0][3] * minor(3, 0);
        if det == 0.0 {
            return None;
        }
        let mut out = [[0.0_f32; 4]; 4];
        for (c, col) in out.iter_mut().enumerate() {
            for (r, cell) in col.iter_mut().enumerate() {
                let d = minor(c, r);
                *cell = if (c + r) % 2 == 1 { -d } else { d } / det;
            }
        }
        Some(M4(out))
    }
}
