//! What the plants are built from, where artkit's botany hasn't it yet: each as tools/art/blender/families/plants.py
//! has it, for the kit to take.

use artkit::botany::{_mesh, _put, MeshOf, Put};
use artkit::far::{is_low, low};
use artkit::materials::Mat;
use artkit::math::{PI, V3, cos, sin, v3};
use artkit::scene::Obj;
use artkit::sphere;

/// A function's options, as artkit's own are written: a struct with defaults.
macro_rules! options {
    ($(#[$doc:meta])* $name:ident { $($field:ident : $ty:ty = $default:expr),* $(,)? }) => {
        $(#[$doc])*
        #[derive(Clone, Debug)]
        pub struct $name {
            $(pub $field: $ty,)*
        }
        impl Default for $name {
            fn default() -> $name {
                $name { $($field: $default,)* }
            }
        }
    };
}
pub(crate) use options;

options! {
    /// `dome`'s options.
    Dome { yaw: f64 = 0.0, tilt: f64 = 0.0, lift: f64 = 0.004 }
}

// KIT: plants.dome
/// A flower's centre: a low dome on its face (none on a far version: too small to see).
pub fn dome(
    name: impl AsRef<str>,
    at: impl Into<V3>,
    r: f64,
    h: f64,
    material: Mat,
    o: Dome,
) -> Option<Obj> {
    if is_low() {
        return None;
    }
    let obj = sphere!(
        name,
        1.0,
        material = material,
        segments = low(6, 4),
        rings = 3,
        scale = (r, r, h)
    );
    Some(_put(
        obj,
        at,
        Put {
            yaw: o.yaw,
            tilt: o.tilt,
            lift: o.lift,
        },
    ))
}

/// `dome` with its options by name, as the family scripts wrote them: `dome!(name, at, r, h, material, key = value, ...)`.
#[macro_export]
macro_rules! dome {
    ($name:expr, $at:expr, $r:expr, $h:expr, $material:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        #[allow(unused_mut)]
        let mut o = $crate::plants::common::Dome::default();
        $(o.$k = artkit::Arg::arg($v);)*
        $crate::plants::common::dome($name, $at, $r, $h, $material, o)
    }};
}

options! {
    /// `star`'s options.
    Star { yaw: f64 = 0.0, tilt: f64 = 0.0, inner: f64 = 0.5, depth: f64 = 0.008, cup: f64 = 0.2 }
}

// KIT: plants.star
/// A flower's face: a thin star of `n` petals, `r` across its radius, the petals lifted by `cup` (a fraction
/// of their reach), facing up and leaning `tilt` degrees towards `yaw`.
pub fn star(
    name: impl AsRef<str>,
    at: impl Into<V3>,
    r: f64,
    n: usize,
    material: Mat,
    o: Star,
) -> Obj {
    let n = low(n, 3.max(n / 3));
    let (mut top, mut bottom) = (Vec::new(), Vec::new());
    for k in 0..2 * n {
        let rr = if k % 2 == 0 { r } else { r * o.inner };
        let a = PI * k as f64 / n as f64;
        let z = o.cup * rr;
        top.push(v3(rr * cos(a), rr * sin(a), z + o.depth / 2.0));
        bottom.push(v3(rr * cos(a), rr * sin(a), z - o.depth / 2.0));
    }
    let mut verts = top;
    verts.extend(bottom);
    verts.extend([v3(0, 0, o.depth / 2.0), v3(0, 0, -o.depth / 2.0)]);
    let m = 2 * n;
    let (ct, cb) = (2 * m, 2 * m + 1);
    let mut faces: Vec<Vec<usize>> = Vec::new();
    for k in 0..m {
        let j = (k + 1) % m;
        faces.extend([
            vec![ct, k, j],
            vec![cb, m + j, m + k],
            vec![k, m + k, m + j, j],
        ]);
    }
    _put(
        _mesh(name, verts, faces, material, MeshOf { smooth: false }),
        at,
        Put {
            yaw: o.yaw,
            tilt: o.tilt,
            ..Put::default()
        },
    )
}

/// `star` with its options by name, as the family scripts wrote them: `star!(name, at, r, n, material, key = value, ...)`.
#[macro_export]
macro_rules! star {
    ($name:expr, $at:expr, $r:expr, $n:expr, $material:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        #[allow(unused_mut)]
        let mut o = $crate::plants::common::Star::default();
        $(o.$k = artkit::Arg::arg($v);)*
        $crate::plants::common::star($name, $at, $r, $n, $material, o)
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use artkit::far::far_of;
    use artkit::materials::Mat;
    use artkit::scene;

    const PETAL: Mat = Mat::new("petal", "#F2C14E", "satin");

    #[test]
    fn a_star_is_a_closed_slab_of_petals_and_a_far_one_has_a_third_of_them() {
        scene::reset();
        let full = star("full", (0, 0, 0.5), 0.05, 6, PETAL, Star::default());
        full.read(|o| {
            assert_eq!(o.mesh.verts.len(), 4 * 6 + 2);
            assert_eq!(o.mesh.faces.len(), 3 * 2 * 6);
            let top = o.mesh.verts.iter().map(|v| v.z).fold(f32::MIN, f32::max);
            assert!((top - (0.5 + 0.2 * 0.05 + 0.004)).abs() < 1e-6);
        });
        let far = far_of(|| star("far", (0, 0, 0.5), 0.05, 6, PETAL, Star::default()));
        far.read(|o| assert_eq!(o.mesh.faces.len(), 3 * 2 * 3));
    }

    #[test]
    fn a_dome_is_left_out_far_off() {
        scene::reset();
        assert!(dome("eye", (0, 0, 0), 0.01, 0.005, PETAL, Dome::default()).is_some());
        assert!(far_of(|| dome("eye", (0, 0, 0), 0.01, 0.005, PETAL, Dome::default())).is_none());
    }
}
