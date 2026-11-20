//! Points and vectors: `P3` and `V3`.

use super::*;

/// A point or a size as a script wrote it, a tuple of three Python floats: `at=(0.1, 0, 2.5)`.
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct P3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

pub fn p3(x: impl Num, y: impl Num, z: impl Num) -> P3 {
    P3 {
        x: x.f(),
        y: y.f(),
        z: z.f(),
    }
}

impl P3 {
    pub const ZERO: P3 = P3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };
    pub const ONE: P3 = P3 {
        x: 1.0,
        y: 1.0,
        z: 1.0,
    };

    pub fn array(self) -> [f64; 3] {
        [self.x, self.y, self.z]
    }
}

impl<A: Num, B: Num, C: Num> From<(A, B, C)> for P3 {
    fn from(t: (A, B, C)) -> P3 {
        p3(t.0, t.1, t.2)
    }
}
impl<A: Num> From<[A; 3]> for P3 {
    fn from(t: [A; 3]) -> P3 {
        p3(t[0], t[1], t[2])
    }
}
impl From<V3> for P3 {
    fn from(v: V3) -> P3 {
        p3(v.x, v.y, v.z)
    }
}
impl Index<usize> for P3 {
    type Output = f64;
    fn index(&self, i: usize) -> &f64 {
        match i {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            _ => panic!("a point has three coordinates, not a {i}th"),
        }
    }
}

/// `mathutils.Vector` of three: single precision.
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct V3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

/// `Vector((x, y, z))`: each rounded to single precision.
pub fn v3(x: impl Num, y: impl Num, z: impl Num) -> V3 {
    V3 {
        x: x.f() as f32,
        y: y.f() as f32,
        z: z.f() as f32,
    }
}

impl<A: Num, B: Num, C: Num> From<(A, B, C)> for V3 {
    fn from(t: (A, B, C)) -> V3 {
        v3(t.0, t.1, t.2)
    }
}
impl<A: Num> From<[A; 3]> for V3 {
    fn from(t: [A; 3]) -> V3 {
        v3(t[0], t[1], t[2])
    }
}
impl From<P3> for V3 {
    fn from(p: P3) -> V3 {
        v3(p.x, p.y, p.z)
    }
}

impl V3 {
    pub const ZERO: V3 = V3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };
    pub const Z: V3 = V3 {
        x: 0.0,
        y: 0.0,
        z: 1.0,
    };

    pub const fn new(x: f32, y: f32, z: f32) -> V3 {
        V3 { x, y, z }
    }

    pub fn array(self) -> [f32; 3] {
        [self.x, self.y, self.z]
    }

    /// `a.dot(b)`: the products in single precision, summed in double from the last.
    pub fn dot(self, o: V3) -> f64 {
        let mut d = 0.0_f64;
        d += (self.z * o.z) as f64;
        d += (self.y * o.y) as f64;
        d += (self.x * o.x) as f64;
        d
    }

    pub fn cross(self, o: V3) -> V3 {
        V3::new(
            self.y * o.z - self.z * o.y,
            self.z * o.x - self.x * o.z,
            self.x * o.y - self.y * o.x,
        )
    }

    /// `v.length`.
    pub fn length(self) -> f64 {
        self.dot(self).sqrt()
    }

    /// `v.xy.length`.
    pub fn xy_length(self) -> f64 {
        let mut d = 0.0_f64;
        d += (self.y * self.y) as f64;
        d += (self.x * self.x) as f64;
        d.sqrt()
    }

    /// `v.normalized()`: zero for a vector of no length.
    pub fn normalized(self) -> V3 {
        let sq = |x: f32| x as f64 * x as f64;
        let d = sq(self.z) + sq(self.y) + sq(self.x);
        if d > 1.0e-35 {
            let k = 1.0 / (d.sqrt() as f32);
            V3::new(self.x * k, self.y * k, self.z * k)
        } else {
            V3::ZERO
        }
    }

    /// `a.lerp(b, t)`.
    pub fn lerp(self, o: V3, t: f64) -> V3 {
        let t = 1.0 - t as f32;
        let s = 1.0 - t;
        V3::new(
            o.x * s + self.x * t,
            o.y * s + self.y * t,
            o.z * s + self.z * t,
        )
    }

    /// `v.orthogonal()`: some vector square to it.
    pub fn orthogonal(self) -> V3 {
        ortho(self)
    }

    /// `a.rotation_difference(b)`: the turn that takes `a`'s direction onto `b`'s.
    pub fn rotation_difference(self, o: V3) -> Quat {
        rotation_between(normalize_v3(self).0, normalize_v3(o).0)
    }

    /// `v.to_track_quat('Z', 'Y')`: the turn that points +Z along `v`, +Y kept up.
    pub fn to_track_quat_zy(self) -> Quat {
        vec_to_quat(-self, 2, 1)
    }

    /// `v.to_track_quat(track, up)`: the turn that points the `track` axis along `v`, the `up` axis kept up
    /// (`'X', 'Z'`: a chain's links along the chain).
    pub fn to_track_quat(self, track: Axis, up: Axis) -> Quat {
        vec_to_quat(-self, track.index(), up.index())
    }

    /// `a.angle(b)`, radians.
    pub fn angle(self, o: V3) -> f64 {
        let f = |a: f32, b: f32| a as f64 * b as f64;
        let (mut dot, mut da, mut db) = (0.0, 0.0, 0.0);
        for i in 0..3 {
            da += f(self[i], self[i]);
            db += f(o[i], o[i]);
            dot += f(self[i], o[i]);
        }
        saacos((dot / (da.sqrt() * db.sqrt())) as f32) as f64
    }
}

impl Index<usize> for V3 {
    type Output = f32;
    fn index(&self, i: usize) -> &f32 {
        match i {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            _ => panic!("a vector has three coordinates, not a {i}th"),
        }
    }
}
impl IndexMut<usize> for V3 {
    fn index_mut(&mut self, i: usize) -> &mut f32 {
        match i {
            0 => &mut self.x,
            1 => &mut self.y,
            2 => &mut self.z,
            _ => panic!("a vector has three coordinates, not a {i}th"),
        }
    }
}
impl Add for V3 {
    type Output = V3;
    fn add(self, o: V3) -> V3 {
        V3::new(self.x + o.x, self.y + o.y, self.z + o.z)
    }
}
impl Sub for V3 {
    type Output = V3;
    fn sub(self, o: V3) -> V3 {
        V3::new(self.x - o.x, self.y - o.y, self.z - o.z)
    }
}
impl Neg for V3 {
    type Output = V3;
    fn neg(self) -> V3 {
        V3::new(-self.x, -self.y, -self.z)
    }
}
/// `v * k`: the factor cast to single precision first.
impl<N: Num> Mul<N> for V3 {
    type Output = V3;
    fn mul(self, k: N) -> V3 {
        let k = k.f() as f32;
        V3::new(self.x * k, self.y * k, self.z * k)
    }
}
impl Mul<V3> for f64 {
    type Output = V3;
    fn mul(self, v: V3) -> V3 {
        v * self
    }
}
/// `v / k`: times the reciprocal of the divisor in single precision.
impl<N: Num> Div<N> for V3 {
    type Output = V3;
    fn div(self, k: N) -> V3 {
        let k = 1.0 / (k.f() as f32);
        V3::new(self.x * k, self.y * k, self.z * k)
    }
}
impl AddAssign for V3 {
    fn add_assign(&mut self, o: V3) {
        *self = *self + o;
    }
}
impl SubAssign for V3 {
    fn sub_assign(&mut self, o: V3) {
        *self = *self - o;
    }
}
impl<N: Num> MulAssign<N> for V3 {
    fn mul_assign(&mut self, k: N) {
        *self = *self * k;
    }
}

/// `sum(vectors, Vector())`.
pub fn sum(vs: impl IntoIterator<Item = V3>) -> V3 {
    vs.into_iter().fold(V3::ZERO, |a, b| a + b)
}

// ---- Blender's own vector arithmetic (single precision throughout) ----
