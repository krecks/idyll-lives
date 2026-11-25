//! What the dining things are built from.

use super::F;
use artkit::forms::rounded_rect;
use artkit::materials::Mat;
use artkit::math::{P3, PI, TAU, V3, cos, pow, sin, tan};
use artkit::mesh::Mesh;
use artkit::parts::join_all;
use artkit::place::{Facing, Spot};
use artkit::scene::{Obj, meshes};
use artkit::shapes::recalc_normals;
use artkit::{fextrude, fsweep, loft, revolve, solidify};
use artkit::{kit, scene, settle};

/// A game position or direction as a Blender vector.
pub fn v(p: impl Spot) -> V3 {
    p.through(&F).into()
}

/// Game points of a polyline whose corners are bent round, about `bend` in radius (a curve through each corner,
/// `steps` segments, never past half way along a side): the path of a tube bent from one length.
pub fn fillet(
    pts: &[(f64, f64, f64)],
    bend: f64,
    steps: usize,
    closed: bool,
) -> Vec<(f64, f64, f64)> {
    let p: Vec<V3> = pts.iter().map(|&q| V3::from(q)).collect();
    let n = p.len();
    let mut out = if closed { Vec::new() } else { vec![p[0]] };
    let corners: Vec<usize> = if closed {
        (0..n).collect()
    } else {
        (1..n - 1).collect()
    };
    for i in corners {
        let (a, q, b) = (p[(i + n - 1) % n], p[i], p[(i + 1) % n]);
        let (u, w) = (a - q, b - q);
        let (lu, lw) = (u.length(), w.length());
        let (u, w) = (u.normalized(), w.normalized());
        let ang = u.angle(w);
        if ang > PI - 1e-3 {
            out.push(q);
            continue;
        }
        let t = (bend / tan(ang / 2.0)).min(0.45 * lu).min(0.45 * lw);
        let (t1, t2) = (q + u * t, q + w * t);
        for k in 0..=steps {
            let s = k as f64 / steps as f64;
            out.push(t1 * pow(1.0 - s, 2.0) + q * (2.0 * (1.0 - s) * s) + t2 * (s * s));
        }
    }
    if !closed {
        out.push(p[n - 1]);
    }
    out.iter()
        .map(|q| (q.x as f64, q.y as f64, q.z as f64))
        .collect()
}

/// A round tube of radius `r` through game points `pts`, bent round each corner (`fillet`): one length of bent
/// rod, rail or bentwood. Open ones are `tube`; a closed loop is swept round on itself.
#[allow(clippy::too_many_arguments)]
pub fn rail(
    name: &str,
    pts: &[(f64, f64, f64)],
    r: f64,
    material: Mat,
    bend: f64,
    verts: usize,
    steps: usize,
    closed: bool,
) -> Obj {
    let path = fillet(pts, bend, steps, closed);
    if closed {
        return fsweep!(F, name, &path, r, material, verts = verts, closed = true);
    }
    kit::tube!(F, name, &path, r, material, verts = verts, round = 1)
}

/// A flat ring of tube in the floor's plane, `rx` by `rz` about (x, z) at height `y`: a foot ring, a rim.
#[allow(clippy::too_many_arguments)]
pub fn ring_loop(
    name: &str,
    rx: f64,
    rz: f64,
    y: f64,
    minor: f64,
    material: Mat,
    x: f64,
    z: f64,
    verts: usize,
    n: usize,
) -> Obj {
    let pts: Vec<(f64, f64, f64)> = (0..n)
        .map(|i| {
            let a = TAU * i as f64 / n as f64;
            (x + rx * cos(a), y, z + rz * sin(a))
        })
        .collect();
    fsweep!(F, name, &pts, minor, material, verts = verts, closed = true)
}

/// A turned part (a leg, a finial, a column, a foot, a plate), `profile` [(radius, height), ...] from the
/// bottom, standing at game point `at`; edges turning more than `angle` stay crisp.
pub fn turned(
    name: &str,
    profile: &[(f64, f64)],
    at: impl Spot,
    material: Mat,
    segments: usize,
    angle: f64,
) -> Obj {
    let obj = revolve!(
        name,
        profile,
        at = at.through(&F),
        material = material,
        segments = segments
    );
    obj.edit(|o| o.mesh.shade_smooth(angle));
    obj
}

/// A table top or a seat board, `w` by `d` with corners of radius `r`, its top at `y_top`, `t` thick, its edge
/// moulded: `edge` [(inset, fraction of the thickness up), ...] from the bottom (by default a rounded nose with
/// a chamfer underneath). One piece, lofted round.
#[allow(clippy::too_many_arguments)]
pub fn slab(
    name: &str,
    w: f64,
    d: f64,
    y_top: f64,
    t: f64,
    material: Mat,
    r: f64,
    edge: Option<&[(f64, f64)]>,
    corner: usize,
    at: (f64, f64),
) -> Obj {
    let edge = edge.unwrap_or(&[
        (0.006, 0.0),
        (0.002, 0.3),
        (0.0, 0.62),
        (0.0015, 0.88),
        (0.0045, 1.0),
    ]);
    let mut sections: Vec<Vec<P3>> = Vec::new();
    for &(inset, f) in edge {
        let y = y_top - t + f * t;
        let outline = rounded_rect(
            w - 2.0 * inset,
            d - 2.0 * inset,
            (r - inset).max(0.002),
            corner,
        );
        sections.push(
            outline
                .iter()
                .map(|&(u, v)| F.at(at.0 + u, y, at.1 + v))
                .collect(),
        );
    }
    loft!(name, &sections, material = material, angle = 50)
}

/// A round top `rad` across its radius, its top at `y_top`, `t` thick, moulded as `slab`.
#[allow(clippy::too_many_arguments)]
pub fn round_slab(
    name: &str,
    rad: f64,
    y_top: f64,
    t: f64,
    material: Mat,
    segments: usize,
    edge: Option<&[(f64, f64)]>,
    at: (f64, f64),
) -> Obj {
    let edge = edge.unwrap_or(&[
        (0.006, 0.0),
        (0.002, 0.3),
        (0.0, 0.62),
        (0.0015, 0.88),
        (0.0045, 1.0),
    ]);
    let mut prof = vec![(0.0, 0.0)];
    prof.extend(edge.iter().map(|&(i, f)| (rad - i, f * t)));
    prof.push((0.0, t));
    turned(
        name,
        &prof,
        (at.0, y_top - t, at.1),
        material,
        segments,
        50.0,
    )
}

/// A square leg tapering from `s_foot` across at `y0` to `s_top` at `y1`, its corners rounded by `r`; `foot`:
/// a small chamfer at the floor. `at_top` (x, z): where its top is, for a splayed leg.
#[allow(clippy::too_many_arguments)]
pub fn sq_leg(
    name: &str,
    x: f64,
    z: f64,
    y0: f64,
    y1: f64,
    s_foot: f64,
    s_top: f64,
    material: Mat,
    r: f64,
    foot: f64,
    at_top: Option<(f64, f64)>,
) -> Obj {
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
pub fn apron(
    name: &str,
    length: f64,
    h: f64,
    t: f64,
    at: impl Spot,
    material: Mat,
    plane: Facing,
    arch: f64,
    steps: usize,
    bevel: f64,
) -> Obj {
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
    fextrude!(
        F,
        name,
        &outline,
        t,
        at,
        material,
        plane = plane,
        bevel = bevel,
        bevel_segments = 1
    )
}

/// A turned stretcher from game point `a` to `b`, swelling to `swell` times `r` in the middle.
pub fn stretcher(
    name: &str,
    a: (f64, f64, f64),
    b: (f64, f64, f64),
    r: f64,
    material: Mat,
    swell: f64,
    verts: usize,
) -> Obj {
    let m = ((a.0 + b.0) / 2.0, (a.1 + b.1) / 2.0, (a.2 + b.2) / 2.0);
    kit::tube!(
        F,
        name,
        &[a, m, b],
        r,
        material,
        verts = verts,
        round = 3,
        scale = [0.8, swell, 0.8]
    )
}

/// A back rail from -x to x at height `y`, bowed back by `bow` in the middle (round the sitter's back), `h` high
/// and `t` thick, taller by `arch` in the middle.
#[allow(clippy::too_many_arguments)]
pub fn crest(
    name: &str,
    x: f64,
    y: f64,
    z: f64,
    h: f64,
    t: f64,
    material: Mat,
    bow: f64,
    arch: f64,
    up: (f64, f64, f64),
    resolution: usize,
) -> Obj {
    fsweep!(
        F,
        name,
        &[(-x, y, z), (0.0, y, z - bow), (x, y, z)],
        0.0,
        material,
        profile = rounded_rect(h, t, (t * 0.45).min(0.01), 1),
        up = up,
        resolution = resolution,
        scale = [1.0, arch, 1.0],
        angle = 60
    )
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
    settle!(centre = false);
    meshes()
}

/// A curved panel `t` thick through a grid of game points (rows of the same length): a moulded shell, a bent back, a
/// slatted seat. Its faces are shaded smooth.
pub fn surface(
    name: &str,
    grid: &[Vec<(f64, f64, f64)>],
    t: f64,
    material: Mat,
    angle: f64,
) -> Obj {
    let (rows, cols) = (grid.len(), grid[0].len());
    let vs: Vec<V3> = grid
        .iter()
        .flatten()
        .map(|&(x, y, z)| F.at(x, y, z).into())
        .collect();
    let mut faces = Vec::new();
    for i in 0..rows - 1 {
        for j in 0..cols - 1 {
            faces.push(
                [
                    i * cols + j,
                    i * cols + j + 1,
                    (i + 1) * cols + j + 1,
                    (i + 1) * cols + j,
                ]
                .map(|v| v as u32)
                .to_vec(),
            );
        }
    }
    let mut mesh = Mesh::from_pydata(vs, &faces);
    mesh.set_material(Some(material));
    let obj = scene::add(name, mesh);
    solidify!(obj, t, offset = 0.0, use_even_offset = true);
    recalc_normals(obj); // the grid's winding may face either way: outwards
    obj.edit(|o| o.mesh.shade_smooth(angle));
    obj
}
