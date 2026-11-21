//! Smooth noise, the same every build: `mathutils.noise.noise()` as the family scripts called it (Blender's
//! default basis, Ken Perlin's original gradient noise), to the bit. A point in, a value in about -1..1 out.

use crate::math::V3;
use crate::noise_tables::{G, P};

/// The lattice cell a coordinate lies in (the tables go round every 256) and how far into it.
fn cell(v: f32) -> (usize, usize, f32, f32) {
    let t = v + 10000.0;
    let b0 = (t as i32 & 255) as usize;
    let r0 = t - t.floor();
    (b0, (b0 + 1) & 255, r0, r0 - 1.0)
}

fn fade(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

fn lerp(t: f32, a: f32, b: f32) -> f32 {
    a + t * (b - a)
}

fn perm(i: usize) -> usize {
    P[i & 255] as usize
}

fn grad(i: usize, x: f32, y: f32, z: f32) -> f32 {
    let q = G[i & 255];
    x * q[0] + y * q[1] + z * q[2]
}

/// The noise at `p`, single precision throughout as Blender computes it.
pub fn noise(p: V3) -> f64 {
    let (bx0, bx1, rx0, rx1) = cell(p.x);
    let (by0, by1, ry0, ry1) = cell(p.y);
    let (bz0, bz1, rz0, rz1) = cell(p.z);
    let (i, j) = (perm(bx0), perm(bx1));
    let (b00, b10, b01, b11) = (perm(i + by0), perm(j + by0), perm(i + by1), perm(j + by1));
    let (sx, sy, sz) = (fade(rx0), fade(ry0), fade(rz0));
    let a = lerp(
        sx,
        grad(b00 + bz0, rx0, ry0, rz0),
        grad(b10 + bz0, rx1, ry0, rz0),
    );
    let b = lerp(
        sx,
        grad(b01 + bz0, rx0, ry1, rz0),
        grad(b11 + bz0, rx1, ry1, rz0),
    );
    let c = lerp(sy, a, b);
    let a = lerp(
        sx,
        grad(b00 + bz1, rx0, ry0, rz1),
        grad(b10 + bz1, rx1, ry0, rz1),
    );
    let b = lerp(
        sx,
        grad(b01 + bz1, rx0, ry1, rz1),
        grad(b11 + bz1, rx1, ry1, rz1),
    );
    let d = lerp(sy, a, b);
    let n = 1.5 * lerp(sz, c, d);
    (2.0 * (0.5 + 0.5 * n) - 1.0) as f64
}

/// The noise at (x, y, z), each rounded to single precision first (the scripts' `noise_at`).
pub fn noise_at(x: f64, y: f64, z: f64) -> f64 {
    noise(V3::new(x as f32, y as f32, z as f32))
}
