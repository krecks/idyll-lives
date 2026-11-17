//! What the dining things are built from.

use super::F;
use artkit::forms::rounded_rect;
use artkit::materials::Mat;
use artkit::math::{P3, PI, pow, sin};
use artkit::parts::{Settle, join_all, settle_with};
use artkit::place::Facing;
use artkit::scene::{Obj, meshes};
use artkit::{fextrude, loft, revolve};

/// A turned part (a leg, a finial, a column, a foot, a plate), `profile` [(radius, height), ...] from the
/// bottom, standing at game point `at`; edges turning more than `angle` stay crisp.
pub fn turned(name: &str, profile: &[(f64, f64)], at: (f64, f64, f64), material: Mat, segments: usize, angle: f64) -> Obj {
    let obj = revolve!(name, profile, at = F.at(at.0, at.1, at.2), material = material, segments = segments);
    obj.edit(|o| o.mesh.shade_smooth(angle));
    obj
}

/// A table top or a seat board, `w` by `d` with corners of radius `r`, its top at `y_top`, `t` thick, its edge
/// moulded: `edge` [(inset, fraction of the thickness up), ...] from the bottom (by default a rounded nose with
/// a chamfer underneath). One piece, lofted round.
#[allow(clippy::too_many_arguments)]
pub fn slab(name: &str, w: f64, d: f64, y_top: f64, t: f64, material: Mat, r: f64, edge: Option<&[(f64, f64)]>, corner: usize, at: (f64, f64)) -> Obj {
    let edge = edge.unwrap_or(&[(0.006, 0.0), (0.002, 0.3), (0.0, 0.62), (0.0015, 0.88), (0.0045, 1.0)]);
    let mut sections: Vec<Vec<P3>> = Vec::new();
    for &(inset, f) in edge {
        let y = y_top - t + f * t;
        let outline = rounded_rect(w - 2.0 * inset, d - 2.0 * inset, (r - inset).max(0.002), corner);
        sections.push(outline.iter().map(|&(u, v)| F.at(at.0 + u, y, at.1 + v)).collect());
    }
    loft!(name, &sections, material = material, angle = 50)
}

/// A square leg tapering from `s_foot` across at `y0` to `s_top` at `y1`, its corners rounded by `r`; `foot`:
/// a small chamfer at the floor. `at_top` (x, z): where its top is, for a splayed leg.
#[allow(clippy::too_many_arguments)]
pub fn sq_leg(name: &str, x: f64, z: f64, y0: f64, y1: f64, s_foot: f64, s_top: f64, material: Mat, r: f64, foot: f64, at_top: Option<(f64, f64)>) -> Obj {
    let (xt, zt) = at_top.unwrap_or((x, z));
    let mut rings = Vec::new();
    if foot != 0.0 {
        rings.push((s_foot - 2.0 * foot, y0, x, z));
        rings.push((s_foot, y0 + foot, x, z));
    } else {
        rings.push((s_foot, y0, x, z));
    }
    rings.push((s_top, y1, xt, zt));
    let sections: Vec<Vec<P3>> = rings
        .iter()
        .map(|&(s, y, cx, cz)| {
            rounded_rect(s, s, r.min(s * 0.45), 1)
                .iter()
                .map(|&(u, v)| F.at(cx + u, y, cz + v))
                .collect()
        })
        .collect();
    loft!(name, &sections, material = material, angle = 50)
}

/// An apron rail `length` long and `h` high (its top at at.1), `t` thick, its lower edge arched up by `arch`
/// in the middle between the legs: along x (`plane` front) or along the depth (side).
#[allow(clippy::too_many_arguments)]
pub fn apron(name: &str, length: f64, h: f64, t: f64, at: (f64, f64, f64), material: Mat, plane: Facing, arch: f64, steps: usize, bevel: f64) -> Obj {
    let half = length / 2.0;
    let lower: Vec<(f64, f64)> = (0..=steps)
        .map(|i| {
            (
                -half + length * i as f64 / steps as f64,
                -h + arch * pow(sin(PI * i as f64 / steps as f64), 1.5),
            )
        })
        .collect();
    let outline: Vec<(f64, f64)> = if arch != 0.0 {
        [vec![(half, 0.0), (-half, 0.0)], lower].concat()
    } else {
        vec![(half, 0.0), (-half, 0.0), (-half, -h), (half, -h)]
    };
    fextrude!(F, name, &outline, t, at, material, plane = plane, bevel = bevel, bevel_segments = 1)
}

/// Joins the model into one, its feet cut flat on the floor (a leg swept up from it at a slant dips a few mm
/// below: cut, it stands where it was built, so its seat stays at its height), and settles it there.
pub fn finish(name: &str) -> Vec<Obj> {
    let obj = join_all(name);
    obj.edit(|o| {
        for v in &mut o.mesh.verts {
            v.z = v.z.max(0.0);
        }
    });
    settle_with(&meshes(), Settle { centre: false, floor: true });
    meshes()
}
