//! What the farm is built from: the palette, the frame, and the slabs, shells and bars the buildings are made of,
//! each drawn as an outline on a plane and thickened across it.

pub use artkit::bevel::eased;
use artkit::forms;
use artkit::materials::Mat;
use artkit::math::{V3, cosd, sind};
use artkit::palette;
use artkit::place::{Coord, GameFrame, N};
use artkit::py;
use artkit::scene::Obj;
use artkit::shapes::SHARP;

/// Built in the game's axes (x, height, depth; front +depth).
pub const F: GameFrame = GameFrame::new(0.0);

palette! { M:
    // the barn
    barn = ("#A8322D", matte),          // walls and door leaves (Paint)
    barn_roof = ("#4B4F55", satin),     // (Roof)
    barn_trim = ("#F1EEE6", satin),     // corner boards, door frames and braces, window frames (Trim)
    stone = ("#8E8C88", matte),         // the plinth
    shadow = ("#2A2422", matte),        // behind the doors, window panes, the cupola's louvres
    iron = ("#2B302E", metal),          // door rail, weathervane, hooks
    // the silo
    silo = ("#C3C8CB", metal),          // the corrugated drum (Paint)
    silo_cap = ("#A8322D", satin),      // the domed cap (Roof)
    concrete = ("#9A958C", matte),
    steel = ("#5C6168", metal),         // ladder, cage, chute
    // the tractor
    tractor = ("#3E7D3A", gloss),       // hood, fenders, canopy (Paint)
    rims = ("#F2C14E", gloss),          // (Wheels)
    tyre = ("#26282B", matte),
    seat = ("#2B2C2E", satin),
    lamp = ("#F4F1E0", gloss),
    tail_lamp = ("#C8322A", gloss),
    // hay
    hay = ("#D9B95E", fabric),
    hay_dark = ("#B8964A", fabric),     // the rolled spiral on a round bale's ends
    straw = ("#D9B95E", matte),         // loose straw, the scarecrow's stuffing
    twine = ("#8C6A3A", matte),         // (Twine)
    // the scarecrow
    post = ("#7A5A3A", wood),
    shirt = ("#B8322B", fabric),        // (Shirt)
    shirt_check = ("#7E1F1B", fabric),  // the plaid's dark lines
    trousers = ("#2E4E7A", fabric),     // (Trousers)
    patch = ("#D9A64A", fabric),
    hat = ("#6E5A3A", fabric),          // (Hat)
    sack = ("#C9A86A", matte),
    stitch = ("#2A2624", matte),
    // the trough
    trough = ("#A9AFB3", metal),        // (Paint)
    water = ("#5B93A8", gloss),
    // the coop
    coop = ("#C8A46E", wood),           // walls, ramp, legs (Paint)
    coop_roof = ("#A8322D", satin),     // (Roof)
    coop_trim = ("#F1EEE6", satin),     // corner boards, runners, the pop-hole hatch (Trim)
    coop_dark = ("#3A2A1E", matte),     // the pop hole behind the hatch, the window's wire
}

/// A point (u, v) on a plane, each as the script wrote it.
pub type P2 = (N, N);

/// Float points on a plane.
pub fn pts(list: &[(f64, f64)]) -> Vec<P2> {
    list.iter().map(|&(u, v)| (N::F(u), N::F(v))).collect()
}

// --- helpers (game coordinates) ------------------------------------------------------------------------------

/// A game point from a 2D point (u, v) on a plane and `w` across it: 'z' (x, y) at depth w, 'x' (depth, y) at x w,
/// 'y' (x, depth) at height w.
pub fn lift(axis: char, u: N, v: N, w: N) -> (N, N, N) {
    match axis {
        'z' => (u, v, w),
        'x' => (w, v, u),
        'y' => (u, w, v),
        _ => panic!("no plane {axis:?}"),
    }
}

/// A flat-shaded mesh from game points, its normals made to face out.
pub fn mesh(name: &str, verts: &[(N, N, N)], faces: &[Vec<u32>], material: Mat) -> Obj {
    let vs: Vec<V3> = verts
        .iter()
        .map(|&(x, y, z)| F.at(x, y, z).into())
        .collect();
    forms::object(name, vs, faces, Some(material), false, SHARP, false)
}

/// A bar lying on a surface from game point `a` to `b`, standing `height` off it towards `up` (a game direction
/// square to it), `base` wide on the surface and `top` at its face, the face's four edges cut across `c` (a batten
/// on its wall, a tyre's lug): 20 triangles where a bevelled box takes 44, its edges on the surface covered and
/// those up its ends short.
#[allow(clippy::too_many_arguments)] // (a bar's every measure)
pub fn bar_on(
    name: &str,
    a: [f64; 3],
    b: [f64; 3],
    up: [f64; 3],
    base: f64,
    top: f64,
    height: f64,
    c: f64,
    material: Mat,
) -> Obj {
    let unit = |v: [f64; 3]| {
        let k = py::hypot(py::hypot(v[0], v[1]), v[2]);
        v.map(|x| x / k)
    };
    let cross = |p: [f64; 3], q: [f64; 3]| {
        [
            p[1] * q[2] - p[2] * q[1],
            p[2] * q[0] - p[0] * q[2],
            p[0] * q[1] - p[1] * q[0],
        ]
    };
    let along = unit([b[0] - a[0], b[1] - a[1], b[2] - a[2]]);
    let across = unit(cross(along, up));
    let up = cross(across, along);
    let at = |e: [f64; 3], fwd: f64, out: f64, side: f64| {
        let p: [f64; 3] =
            std::array::from_fn(|i| e[i] + along[i] * fwd + up[i] * out + across[i] * side);
        (N::F(p[0]), N::F(p[1]), N::F(p[2]))
    };
    let mut verts = Vec::with_capacity(12);
    for (e, inward) in [(a, c), (b, -c)] {
        verts.extend([
            at(e, 0.0, 0.0, -base / 2.0),
            at(e, 0.0, 0.0, base / 2.0),
            at(e, 0.0, height - c, -top / 2.0),
            at(e, 0.0, height - c, top / 2.0),
            at(e, inward, height, c - top / 2.0),
            at(e, inward, height, top / 2.0 - c),
        ]);
    }
    let faces = [
        [0, 1, 7, 6],
        [1, 7, 9, 3],
        [0, 6, 8, 2],
        [3, 9, 11, 5],
        [2, 8, 10, 4],
        [4, 5, 11, 10],
        [0, 1, 3, 2],
        [2, 3, 5, 4],
        [6, 7, 9, 8],
        [8, 9, 11, 10],
    ]
    .map(|f| f.to_vec());
    mesh(name, &verts, &faces, material)
}

/// A convex outline `pts` on a plane (see `lift`), extruded from `w0` to `w1` across it.
pub fn prism(name: &str, pts: &[P2], w0: f64, w1: f64, material: Mat, axis: char) -> Obj {
    let n = pts.len();
    let mut verts: Vec<(N, N, N)> = pts
        .iter()
        .map(|&(u, v)| lift(axis, u, v, N::F(w0)))
        .collect();
    verts.extend(pts.iter().map(|&(u, v)| lift(axis, u, v, N::F(w1))));
    let n32 = n as u32;
    let mut faces: Vec<Vec<u32>> = vec![(0..n32).collect(), (n32..2 * n32).rev().collect()];
    for i in 0..n32 {
        faces.push(vec![i, (i + 1) % n32, n32 + (i + 1) % n32, n32 + i]);
    }
    eased(mesh(name, &verts, &faces, material))
}

/// `line` (a polyline on a plane, u rising left to right) moved `t` to its left (up, for a roof), mitred.
pub fn offset(line: &[P2], t: f64) -> Vec<P2> {
    fn normal(a: P2, b: P2) -> (f64, f64) {
        let (dx, dy) = (b.0.f() - a.0.f(), b.1.f() - a.1.f());
        let k = py::hypot(dx, dy);
        (-dy / k, dx / k)
    }
    let mut out = Vec::with_capacity(line.len());
    for (i, &p) in line.iter().enumerate() {
        let mut ns = Vec::new();
        if i > 0 {
            ns.push(normal(line[i - 1], p));
        }
        if i < line.len() - 1 {
            ns.push(normal(p, line[i + 1]));
        }
        let (nx, ny) = (
            py::sum(ns.iter().map(|q| q.0)),
            py::sum(ns.iter().map(|q| q.1)),
        );
        let k = py::hypot(nx, ny);
        let (nx, ny) = (nx / k, ny / k);
        let s = t / (nx * ns[0].0 + ny * ns[0].1);
        out.push((N::F(p.0.f() + nx * s), N::F(p.1.f() + ny * s)));
    }
    out
}

/// A roof or fender: the polyline `line` thickened `t` to its left, from `w0` to `w1` across.
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn shell(name: &str, line: &[P2], t: f64, w0: f64, w1: f64, material: Mat, axis: char) -> Obj {
    let (inner, outer) = (line.to_vec(), offset(line, t));
    let n = line.len();
    let mut verts = Vec::with_capacity(4 * n);
    for w in [w0, w1] {
        verts.extend(inner.iter().map(|&(u, v)| lift(axis, u, v, N::F(w))));
        verts.extend(outer.iter().map(|&(u, v)| lift(axis, u, v, N::F(w))));
    }
    let n = n as u32;
    let (i0, o0, i1, o1) = (0, n, 2 * n, 3 * n);
    let mut faces: Vec<Vec<u32>> = Vec::new();
    for i in 0..n - 1 {
        faces.push(vec![o0 + i, o0 + i + 1, o1 + i + 1, o1 + i]);
        faces.push(vec![i0 + i, i1 + i, i1 + i + 1, i0 + i + 1]);
        faces.push(vec![i0 + i, i0 + i + 1, o0 + i + 1, o0 + i]);
        faces.push(vec![i1 + i, o1 + i, o1 + i + 1, i1 + i + 1]);
    }
    faces.push(vec![i0, o0, o1, i1]);
    faces.push(vec![i0 + n - 1, i1 + n - 1, o1 + n - 1, o0 + n - 1]);
    eased(mesh(name, &verts, &faces, material))
}

/// A flat bar from `a` to `b` on a plane (a door's brace), `width` wide, from `w0` to `w1` across.
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn strut(
    name: &str,
    a: (f64, f64),
    b: (f64, f64),
    width: f64,
    w0: f64,
    w1: f64,
    material: Mat,
    axis: char,
) -> Obj {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let k = py::hypot(dx, dy);
    let (nx, ny) = (-dy / k * width / 2.0, dx / k * width / 2.0);
    let pts = pts(&[
        (a.0 - nx, a.1 - ny),
        (b.0 - nx, b.1 - ny),
        (b.0 + nx, b.1 + ny),
        (a.0 + nx, a.1 + ny),
    ]);
    prism(name, &pts, w0, w1, material, axis)
}

/// (cx, angle) samples round a stadium (a straight middle `2 * half` long along x, round ends), `n` + 1 per end.
pub fn stadium(half: f64, n: usize) -> Vec<(f64, f64)> {
    let mut out: Vec<(f64, f64)> = (0..n + 1)
        .map(|i| (half, -90.0 + 180.0 * i as f64 / n as f64))
        .collect();
    out.extend((0..n + 1).map(|i| (-half, 90.0 + 180.0 * i as f64 / n as f64)));
    out
}

/// A closed profile [(r, height)] swept round a stadium (a stock tank's wall): x along, depth across.
pub fn stadium_sweep(
    name: &str,
    profile: &[(f64, f64)],
    half: f64,
    material: Mat,
    n: usize,
) -> Obj {
    let ring = stadium(half, n);
    let m = profile.len();
    let mut verts = Vec::with_capacity(ring.len() * m);
    for &(cx, a) in &ring {
        for &(r, y) in profile {
            verts.push((N::F(cx + r * cosd(a)), N::F(y), N::F(r * sind(a))));
        }
    }
    let mut faces: Vec<Vec<u32>> = Vec::new();
    for s in 0..ring.len() {
        let s2 = (s + 1) % ring.len();
        for i in 0..m {
            let i2 = (i + 1) % m;
            faces.push(
                [s * m + i, s * m + i2, s2 * m + i2, s2 * m + i]
                    .map(|k| k as u32)
                    .to_vec(),
            );
        }
    }
    eased(mesh(name, &verts, &faces, material))
}

pub fn stadium_slab(
    name: &str,
    r: f64,
    half: f64,
    y0: f64,
    y1: f64,
    material: Mat,
    n: usize,
) -> Obj {
    let pts: Vec<P2> = stadium(half, n)
        .iter()
        .map(|&(cx, a)| (N::F(cx + r * cosd(a)), N::F(r * sind(a))))
        .collect();
    prism(name, &pts, y0, y1, material, 'y')
}

/// A rectangular frame of bars `bar` wide on a face at `z` (see `lift`), standing `depth` proud.
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn frame(
    name: &str,
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
    z: f64,
    bar: f64,
    depth: f64,
    material: Mat,
    axis: char,
) {
    let h = bar / 2.0;
    let bars = [
        ("b", (x0 - h, y0 - h, x1 + h, y0 + h)),
        ("t", (x0 - h, y1 - h, x1 + h, y1 + h)),
        ("l", (x0 - h, y0 + h, x0 + h, y1 - h)),
        ("r", (x1 - h, y0 + h, x1 + h, y1 - h)),
    ];
    for (side, (u0, v0, u1, v1)) in bars {
        prism(
            &format!("{name}_{side}"),
            &pts(&[(u0, v0), (u1, v0), (u1, v1), (u0, v1)]),
            z,
            z + depth,
            material,
            axis,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `offset` moves each segment of a polyline `t` to its left, mitred where two meet.
    #[test]
    fn an_offset_line_stands_its_thickness_off_every_segment() {
        let line = pts(&[(-2.0, 0.0), (-1.0, 1.0), (0.0, 1.5), (1.0, 1.0), (2.0, 0.0)]);
        let t = 0.12;
        let out = offset(&line, t);
        assert_eq!(out.len(), line.len());
        for i in 0..line.len() - 1 {
            let (a, b) = (
                (line[i].0.f(), line[i].1.f()),
                (line[i + 1].0.f(), line[i + 1].1.f()),
            );
            let (dx, dy) = (b.0 - a.0, b.1 - a.1);
            let k = py::hypot(dx, dy);
            for p in [out[i], out[i + 1]] {
                // the signed distance to the segment's line, to its left
                let d = ((p.0.f() - a.0) * -dy + (p.1.f() - a.1) * dx) / k;
                assert!((d - t).abs() < 1e-12, "segment {i}: {d}");
            }
        }
    }
}
