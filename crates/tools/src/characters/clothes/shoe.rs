//! Shoes: the styles and the foot they are built on.

use super::*;

/// A shoe type's shape (m, on a body 1.7 m tall and scaled with the body; heights over the ankle
/// joint are on the base body, as the cuts are).
pub(crate) struct ShoeStyle {
    /// The upper's lift off the skin, more on top of the toe box, round the heel counter, in a padded
    /// collar and at a boot's top.
    pub(crate) lift: f64,
    pub(crate) toe: f64,
    pub(crate) heel: f64,
    pub(crate) pad: f64,
    pub(crate) shaft: f64,
    /// How far the toe is drawn out forward: a dress shoe's long toe.
    pub(crate) point: f64,
    /// The collar's height over the ankle joint at the back, the sides and the front (the tongue, or
    /// a dress shoe's low throat, which ends at `throat` along the foot: the vamp covers the toes).
    pub(crate) collar: [f64; 3],
    pub(crate) throat: f64,
    /// The sock's top over the ankle joint: the painted cut (`shoe`), above the collar's sides.
    pub(crate) sock: f64,
    /// The sole's thickness under the ball and under the heel, where the heel's thickness gives way to
    /// the ball's (along the foot, 0 heel .. 1 toe), how far its edge stands out from the upper, the
    /// arch's lift off the floor between the heel and the ball, and the light midsole's share of its edge.
    pub(crate) sole: f64,
    pub(crate) heel_h: f64,
    pub(crate) block: (f64, f64),
    pub(crate) welt: f64,
    pub(crate) arch: f64,
    pub(crate) midsole: f64,
    /// The toe's lift off the floor at the tip: the whole sole rises there, so the shoe's underside is a
    /// curve rather than a slab lying flat on the ground (`spring_at`).
    pub(crate) spring: f64,
    pub(crate) laces: bool,
    /// Polished leather (glossier).
    pub(crate) polish: bool,
}

pub(crate) fn shoe_style(gid: &str) -> ShoeStyle {
    match gid {
        // Chunky: a padded collar, a round toe box, a thick white cup sole on a dark outsole, laces.
        "shoes.sneakers" => ShoeStyle {
            lift: 0.005,
            toe: 0.003,
            heel: 0.003,
            pad: 0.002,
            shaft: 0.0,
            point: 0.0,
            collar: [0.022, 0.012, 0.032],
            throat: 1.0,
            sock: 0.03,
            sole: 0.016,
            heel_h: 0.019,
            block: (0.15, 0.6),
            welt: 0.003,
            arch: 0.0,
            midsole: 0.55,
            spring: 0.008,
            laces: true,
            polish: false,
        },
        // Running shoes: chunkier still, a high heel counter, a thick white midsole rising to the heel.
        "shoes.trainers" => ShoeStyle {
            lift: 0.006,
            toe: 0.004,
            heel: 0.004,
            pad: 0.003,
            shaft: 0.0,
            point: 0.0,
            collar: [0.03, 0.02, 0.042],
            throat: 1.0,
            sock: 0.038,
            sole: 0.018,
            heel_h: 0.028,
            block: (0.1, 0.7),
            welt: 0.004,
            arch: 0.0,
            midsole: 0.7,
            spring: 0.01,
            laces: true,
            polish: false,
        },
        // Up the ankle, a little loose at the top, laced; a dark sole with a heel.
        "shoes.boots" => ShoeStyle {
            lift: 0.005,
            toe: 0.006,
            heel: 0.003,
            pad: 0.0,
            shaft: 0.004,
            point: 0.0,
            collar: [0.13, 0.125, 0.125],
            throat: 1.0,
            sock: 0.12,
            sole: 0.018,
            heel_h: 0.026,
            block: (0.26, 0.31),
            welt: 0.004,
            arch: 0.005,
            midsole: 0.0,
            spring: 0.005,
            laces: true,
            polish: false,
        },
        // Low and sleek: close to the foot, a throat down the instep, a long toe, polished, a thin sole
        // and a heel.
        _ => ShoeStyle {
            lift: 0.003,
            toe: 0.001,
            heel: 0.002,
            pad: 0.0,
            shaft: 0.0,
            point: 0.014,
            collar: [-0.02, -0.03, -0.022],
            throat: 0.45,
            sock: 0.032,
            sole: 0.01,
            heel_h: 0.022,
            block: (0.26, 0.31),
            welt: 0.0025,
            arch: 0.006,
            midsole: 0.0,
            spring: 0.011,
            laces: false,
            polish: true,
        },
    }
}

impl ShoeStyle {
    /// The sole's top (m on a 1.7 m body) at `u` along the foot (0 heel .. 1 toe).
    pub(crate) fn top(&self, u: f64) -> f64 {
        self.sole + (self.heel_h - self.sole) * (1.0 - smoothstep(self.block.0, self.block.1, u))
    }

    /// The toe spring's lift (m on a 1.7 m body) at `u` along the foot: the whole sole's, its top and
    /// its underside alike, and so the upper's cut inside it.
    pub(crate) fn spring_at(&self, u: f64) -> f64 {
        self.spring * smoothstep(0.7, 1.0, u).powi(2)
    }

    /// The collar's height over the ankle joint at a point `dx` to the side of it, `dz` ahead and `u`
    /// along the foot.
    pub(crate) fn collar_at(&self, dx: f64, dz: f64, u: f64) -> f64 {
        let c = dz / dx.hypot(dz).max(1e-9);
        let [back, side, front] = self.collar;
        side + (front - side) * c.max(0.0)
            + (back - side) * (-c).max(0.0)
            + 0.05 * smoothstep(self.throat - 0.03, self.throat + 0.03, u)
    }
}

/// The upper below the sole's top (m on a 1.7 m body): cut there, inside the sole.
pub(crate) const UPPER_IN_SOLE: f64 = 0.005;

static FINER: Cache<(String, String, bool), Vec<V3>> = Cache::new();

/// The next level of a body's surface, on its limit (level 2 under the body, level 1 under the
/// low-detail body): there, the point a cut edge's middle lies on.
pub(crate) fn finer(family: &str, stage: &str, lod: bool) -> Arc<Vec<V3>> {
    FINER.get((family.to_string(), stage.to_string(), lod), || {
        let s = surface(family, stage);
        if lod {
            s.pos.clone()
        } else {
            topology().fine.apply(&s.ctrl)
        }
    })
}

/// Per edge of a surface's triangles (its faces' own edges, and the diagonals they are cut along), the
/// next level's vertex at its middle: an edge's point, or the face's (subdiv.rs: the `edges`' points
/// come after the `n` vertices, then the faces').
pub(crate) fn edge_mids(
    faces: &[Vec<usize>],
    edges: &[(usize, usize)],
    n: usize,
) -> std::collections::BTreeMap<(usize, usize), usize> {
    let mut out: std::collections::BTreeMap<(usize, usize), usize> =
        edges.iter().enumerate().map(|(k, &e)| (e, n + k)).collect();
    for (f, face) in faces.iter().enumerate() {
        for (i, &a) in face.iter().enumerate() {
            for &bv in face.iter().skip(i + 2) {
                out.entry((a.min(bv), a.max(bv)))
                    .or_insert(n + edges.len() + f);
            }
        }
    }
    out
}
