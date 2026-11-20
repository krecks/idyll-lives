//! `Quat`, as `mathutils` has it.

use super::*;

/// `mathutils.Quaternion`: (w, x, y, z), single precision.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Quat {
    pub w: f32,
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Quat {
    pub const IDENTITY: Quat = Quat {
        w: 1.0,
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };

    pub fn array(self) -> [f32; 4] {
        [self.w, self.x, self.y, self.z]
    }

    /// `Quaternion(axis, angle)`: a turn of `angle` radians about `axis` (of any length; none for no axis).
    pub fn from_axis_angle(axis: impl Into<V3>, angle: f64) -> Quat {
        let (unit, len) = normalize_v3(axis.into());
        if len == 0.0 {
            return Quat::IDENTITY;
        }
        axis_angle_normalized_to_quat(unit, angle_wrap(angle as f32))
    }

    /// The quaternion at unit length, as Blender takes an object's before it turns it (`normalize_qt`).
    pub fn normalized(self) -> Quat {
        let len = (self.w * self.w + self.x * self.x + self.y * self.y + self.z * self.z).sqrt();
        if len == 0.0 {
            return Quat {
                w: 0.0,
                x: 1.0,
                y: 0.0,
                z: 0.0,
            };
        }
        let k = 1.0 / len;
        Quat {
            w: self.w * k,
            x: self.x * k,
            y: self.y * k,
            z: self.z * k,
        }
    }

    /// The turn as Euler angles in `order`, radians (`quat_to_eulO`).
    pub fn to_euler(self, order: EulerOrder) -> V3 {
        mat3_normalized_to_eul(&self.to_matrix(), order)
    }

    /// `q.to_matrix()`.
    pub fn to_matrix(self) -> M3 {
        let s = std::f64::consts::SQRT_2;
        let (q0, q1, q2, q3) = (
            s * self.w as f64,
            s * self.x as f64,
            s * self.y as f64,
            s * self.z as f64,
        );
        let (qda, qdb, qdc) = (q0 * q1, q0 * q2, q0 * q3);
        let (qaa, qab, qac) = (q1 * q1, q1 * q2, q1 * q3);
        let (qbb, qbc, qcc) = (q2 * q2, q2 * q3, q3 * q3);
        M3([
            [
                (1.0 - qbb - qcc) as f32,
                (qdc + qab) as f32,
                (-qdb + qac) as f32,
            ],
            [
                (-qdc + qab) as f32,
                (1.0 - qaa - qcc) as f32,
                (qda + qbc) as f32,
            ],
            [
                (qdb + qac) as f32,
                (-qda + qbc) as f32,
                (1.0 - qaa - qbb) as f32,
            ],
        ])
    }
}

/// `q @ v`: the vector turned.
impl Mul<V3> for Quat {
    type Output = V3;
    fn mul(self, r: V3) -> V3 {
        let q = self;
        let t0 = -q.x * r.x - q.y * r.y - q.z * r.z;
        let t1 = q.w * r.x + q.y * r.z - q.z * r.y;
        let t2 = q.w * r.y + q.z * r.x - q.x * r.z;
        let r2 = q.w * r.z + q.x * r.y - q.y * r.x;
        let (r0, r1) = (t1, t2);
        let t1 = t0 * -q.x + r0 * q.w - r1 * q.z + r2 * q.y;
        let t2 = t0 * -q.y + r1 * q.w - r2 * q.x + r0 * q.z;
        let z = t0 * -q.z + r2 * q.w - r0 * q.y + r1 * q.x;
        V3::new(t1, t2, z)
    }
}

/// `a @ b`: the turn `b`, then `a`.
impl Mul<Quat> for Quat {
    type Output = Quat;
    fn mul(self, o: Quat) -> Quat {
        mul_qt_qtqt(self, o)
    }
}
