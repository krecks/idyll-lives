//! A fan's frame for the normals stored in it.

use super::*;

/// A fan's frame for the normals stored in it, as Blender defines it.
pub(super) struct Space {
    pub(super) lnor: V3,
    pub(super) vec_ref: V3,
    pub(super) vec_ortho: V3,
    pub(super) ref_alpha: f32,
    pub(super) ref_beta: f32,
}

impl Space {
    const TRIGO: f32 = 0.9999;
    const TWO_PI: f32 = std::f32::consts::TAU;

    pub(super) fn new(fan: &Fan) -> Space {
        let lnor = fan.normal;
        let (vec_ref, vec_other) = (fan.edges[0], fan.edges[fan.edges.len() - 1]);
        let (dtp_ref, dtp_other) = (dot_v3(vec_ref, lnor), dot_v3(vec_other, lnor));
        if dtp_ref.abs() >= Space::TRIGO || dtp_other.abs() >= Space::TRIGO {
            // (an edge along the normal: no frame, the normal as it is)
            return Space {
                lnor,
                vec_ref: V3::ZERO,
                vec_ortho: V3::ZERO,
                ref_alpha: 0.0,
                ref_beta: 0.0,
            };
        }
        let ref_alpha = if fan.single {
            (crate::normals::acos_approx(dtp_ref) + crate::normals::acos_approx(dtp_other)) / 2.0
        } else {
            // (the first edge again at the end of a fan all the way round is not counted twice)
            let whole = fan.edges.len() > 2 && fan.edges[0] == fan.edges[fan.edges.len() - 1];
            let counted = if whole {
                &fan.edges[..fan.edges.len() - 1]
            } else {
                &fan.edges[..]
            };
            let sum: f32 = counted.iter().fold(0.0, |s, &v| {
                s + crate::normals::acos_approx(dot_v3(v, lnor))
            });
            sum / counted.len() as f32
        };
        // (unit vectors by division here, as the frame's own code makes them; the fan's sides and normal, which
        // Blender's bevel hands it, are made by the reciprocal)
        let unit = crate::normals::unit;
        let vec_ref = unit(vec_ref - mul_v3_fl(lnor, dtp_ref));
        let vec_ortho = unit(lnor.cross(vec_ref));
        let other = unit(vec_other - mul_v3_fl(lnor, dtp_other));
        let dtp = dot_v3(vec_ref, other);
        let ref_beta = if dtp < Space::TRIGO {
            let beta = crate::normals::acos_approx(dtp);
            if dot_v3(vec_ortho, other) < 0.0 {
                Space::TWO_PI - beta
            } else {
                beta
            }
        } else {
            Space::TWO_PI
        };
        Space {
            lnor,
            vec_ref,
            vec_ortho,
            ref_alpha,
            ref_beta,
        }
    }

    fn short(val: f32) -> i32 {
        if val >= 1.0 {
            32767
        } else if val <= -1.0 {
            -32768
        } else {
            (val * 32767.0 + 0.5).floor() as i32
        }
    }

    /// A normal as the two shares stored for it.
    pub(super) fn store(&self, custom: V3) -> [i32; 2] {
        let close = (0..3).all(|i| (self.lnor[i] - custom[i]).abs() <= 1e-4);
        if dot_v3(custom, custom) == 0.0 || close {
            return [0, 0];
        }
        let cos_alpha = dot_v3(self.lnor, custom);
        let alpha = crate::normals::acos_approx(cos_alpha);
        let a = if alpha > self.ref_alpha {
            Space::short(-(Space::TWO_PI - alpha) / (Space::TWO_PI - self.ref_alpha))
        } else {
            Space::short(alpha / self.ref_alpha)
        };
        let vec = crate::normals::unit(mul_v3_fl(self.lnor, -cos_alpha) + custom);
        let cos_beta = dot_v3(self.vec_ref, vec);
        let b = if cos_beta < Space::TRIGO {
            let mut beta = crate::normals::acos_approx(cos_beta);
            if dot_v3(self.vec_ortho, vec) < 0.0 {
                beta = Space::TWO_PI - beta;
            }
            if beta > self.ref_beta {
                Space::short(-(Space::TWO_PI - beta) / (Space::TWO_PI - self.ref_beta))
            } else {
                Space::short(beta / self.ref_beta)
            }
        } else {
            0
        };
        [a, b]
    }
}
