//! The round profile a bevel sweeps, and the corners it starts from.

use super::*;

/// A round profile from `start` to `end` about `middle`, its points square to the edge `along` on the plane
/// through `start` with the normal `plane_no`: the points 1 to `ns` - 1.
pub(crate) fn profile(
    start: V3,
    middle: V3,
    end: V3,
    along: V3,
    plane_no: V3,
    ns: usize,
) -> Vec<V3> {
    let (m0, m1) = (middle - start, middle - end);
    let m3 = start + end - middle;
    let step = (PI / 2.0) / ns as f64;
    (1..ns)
        .map(|k| {
            let (x, y) = (
                libm::sin(k as f64 * step) as f32,
                libm::cos(k as f64 * step) as f32,
            );
            let co = V3::new(
                x * m0.x + y * m1.x + m3.x,
                x * m0.y + y * m1.y + m3.y,
                x * m0.z + y * m1.z + m3.z,
            );
            // onto the profile's plane, along the edge
            let h = co - start;
            let dot = dot_v3(plane_no, along);
            if dot.abs() > f32::EPSILON {
                madd(co, along, -dot_v3(plane_no, h) / dot)
            } else {
                co
            }
        })
        .collect()
}

/// A corner patch of a cube of side one, `ns` segments: its points by (bound, ring, step), on the unit sphere.
pub(crate) fn cube_corner(ns: usize) -> Result<BTreeMap<(usize, usize, usize), V3>, String> {
    let unit = |i: usize, a: f32, b: f32, c: f32| {
        // (a along the bound's own axis, b along the next, c along the third)
        let mut p = [0.0_f32; 3];
        (p[i], p[(i + 1) % 3], p[(i + 2) % 3]) = (a, b, c);
        normalize_v3(V3::new(p[0], p[1], p[2])).0
    };
    let mut out = BTreeMap::new();
    let table: &[(usize, usize, [f32; 3])] = match ns {
        2 => &[(0, 1, [0.707_106_77, 0.707_106_77, 0.0])],
        3 => CORNER3,
        4 => CORNER4,
        _ => return Err(format!("a corner patch of {ns} segments isn't built")),
    };
    for i in 0..3 {
        for &(j, k, [a, b, c]) in table {
            out.insert((i, j, k), unit(i, a, b, c));
        }
    }
    if ns.is_multiple_of(2) {
        let s = 0.577_350_26_f32;
        out.insert((0, ns / 2, ns / 2), normalize_v3(V3::new(s, s, s)).0);
    }
    Ok(out)
}

/// The patch of three segments, as Blender's subdivision leaves it on the sphere (measured from its output).
pub(crate) const CORNER3: &[(usize, usize, [f32; 3])] = &[
    (0, 1, [0.866_403_8, 0.499_344, 0.0]),
    (1, 0, [0.866_403_8, 0.0, 0.499_344]),
    (1, 1, [0.736_632_6, 0.478_211_55, 0.478_211_55]),
];
/// The patch of four segments.
pub(crate) const CORNER4: &[(usize, usize, [f32; 3])] = &[
    (0, 1, [0.923_879_5, 0.382_683_43, 0.0]),
    (0, 2, [0.707_106_77, 0.707_106_77, 0.0]),
    (1, 0, [0.923_879_5, 0.0, 0.382_683_43]),
    (1, 1, [0.832_011_1, 0.392_273_8, 0.392_273_8]),
    (1, 2, [0.656_254_55, 0.656_254_55, 0.372_370_53]),
];
