//! Ears, and the hair kept clear of them.

use super::*;

/// How round the hair's edge is where it meets the ovals round the ears (m: the smoothing's width).
pub(super) const RIM_ROUND: f64 = 0.011;

/// How far hair keeps off the ears (m); the check wants more than `EAR_MIN`.
pub(super) const EAR_CLEAR: f64 = 0.003;
pub const EAR_MIN: f64 = 0.0015;

/// The ears as drawn (ear.rs: pieces set into the head, which the head's `sdf` knows nothing of), to
/// keep hair off them: how far a point is into one.
pub struct Ears {
    pub(super) ears: Vec<Ear>,
}

pub(super) struct Ear {
    /// Its side (sign of x) and its box grown by 2 cm.
    pub(super) side: f64,
    pub(super) lo: V3,
    pub(super) hi: V3,
    /// The ear's own faces, and the same closed over its foot in the head (a ray out to the side
    /// crosses these an odd number of times from inside the ear).
    pub(super) own: Vec<[V3; 3]>,
    pub(super) closed: Vec<[V3; 3]>,
}

impl Ears {
    /// From each ear's faces and the faces closing its foot (left, right: `ear::Ears::faces`), round
    /// the head interface's `ears` boxes.
    pub fn new(boxes: &[[V3; 2]; 2], faces: [[Vec<[V3; 3]>; 2]; 2]) -> Self {
        let ears = boxes
            .iter()
            .zip(faces)
            .map(|([c, e], [own, cap])| Ear {
                side: c.x.signum(),
                lo: c - e - V3::repeat(0.02),
                hi: c + e + V3::repeat(0.02),
                closed: [own.as_slice(), cap.as_slice()].concat(),
                own,
            })
            .collect();
        Ears { ears }
    }

    /// The ears of a family at a stage (as built).
    pub fn of(family: &str, stage: &str, h: &HeadFrame) -> Self {
        Ears::new(&h.ears, super::ear::ears(family, stage).faces(None))
    }

    /// How far a point is into an ear (m; < 0: that far out of the nearest), and the way out of it;
    /// None away from the ears.
    pub fn depth(&self, p: V3) -> Option<(f64, V3)> {
        let mut best: Option<(f64, V3)> = None;
        for e in &self.ears {
            if !(0..3).all(|k| e.lo[k] <= p[k] && p[k] <= e.hi[k]) || e.own.is_empty() {
                continue;
            }
            let ray = v3(e.side, 0.0123, 0.0071).normalize();
            let inside = e
                .closed
                .iter()
                .filter(|t| ray_hit(p, ray, t).is_some())
                .count()
                % 2
                == 1;
            let q = e
                .own
                .iter()
                .map(|t| nearest_point(p, t))
                .min_by(|a, b| (a - p).norm_squared().total_cmp(&(b - p).norm_squared()))
                .unwrap();
            let d = (q - p).norm();
            let out = if inside { q - p } else { p - q } / d.max(1e-9);
            let v = if inside { d } else { -d };
            if best.is_none_or(|b| v > b.0) {
                best = Some((v, out));
            }
        }
        best
    }

    /// Whether the segment p..q passes through an ear's surface.
    pub fn cut(&self, p: V3, q: V3) -> bool {
        let l = (q - p).norm();
        let d = (q - p) / l.max(1e-12);
        self.ears.iter().any(|e| {
            (0..3).all(|k| e.lo[k] <= p[k].max(q[k]) && p[k].min(q[k]) <= e.hi[k])
                && e.own
                    .iter()
                    .any(|t| ray_hit(p, d, t).is_some_and(|s| s < l))
        })
    }
}

/// The least distance to the head (sdf) at and round a point.
pub(super) fn near_sdf(h: &HeadFrame, p: V3) -> f64 {
    let r = 0.0004;
    let mut d = h.sdf(p);
    for e in [
        v3(r, 0.0, 0.0),
        v3(0.0, r, 0.0),
        v3(0.0, 0.0, r),
        v3(-r, 0.0, 0.0),
        v3(0.0, -r, 0.0),
        v3(0.0, 0.0, -r),
    ] {
        d = d.min(h.sdf(p + e));
    }
    d
}

pub(super) fn grad(h: &HeadFrame, p: V3, e: f64) -> V3 {
    let mut g = V3::zeros();
    for k in 0..3 {
        let mut d = V3::zeros();
        d[k] = e;
        g[k] = (h.sdf(p + d) - h.sdf(p - d)) / (2.0 * e);
    }
    g / g.norm().max(1e-9)
}
