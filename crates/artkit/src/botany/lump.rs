//! The lumps a crown, a bush or a pompom is made of.

use crate::far::low;
use crate::materials::Mat;
use crate::math::{P3, V3, v3};
use crate::noise::noise_at;
use crate::py;
use crate::scene::Obj;
use crate::shapes::{Sphere, options, sphere};

options! {
    /// `lump`'s options.
    Lump { scale: P3 = P3::ONE, seed: f64 = 0.0, noise: f64 = 0.25, segments: usize = 10, rings: usize = 6 }
}

/// A soft, lumpy ball (a leafy clump, a pompom, a cluster of florets): a smooth sphere pushed about by noise, its
/// bottom at `at`.
pub fn lump(name: impl AsRef<str>, r: f64, at: impl Into<V3>, material: Mat, o: Lump) -> Obj {
    let (seed, noise) = (o.seed, o.noise);
    let (segments, rings) = (
        low(
            o.segments,
            6.max(py::round(o.segments as f64 * 0.6) as usize),
        ),
        low(o.rings, 3.max(py::round(o.rings as f64 * 0.6) as usize)),
    );
    let obj = sphere(
        name,
        r,
        Sphere {
            material: Some(material),
            segments,
            rings,
            scale: o.scale,
            ..Sphere::default()
        },
    );
    let at: V3 = at.into();
    obj.edit(|part| {
        for v in &mut part.mesh.verts {
            let p = *v;
            let n = noise_at(
                p.x as f64 / r * 1.3 + seed,
                p.y as f64 / r * 1.3 - seed,
                p.z as f64 / r * 1.3 + seed * 0.5,
            );
            *v = p * (1.0 + noise * n);
        }
        let bottom = part
            .mesh
            .verts
            .iter()
            .map(|v| v.z)
            .fold(f32::INFINITY, f32::min);
        part.mesh.translate(at - v3(0, 0, bottom));
    });
    obj
}
