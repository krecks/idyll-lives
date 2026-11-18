//! The woods' rock: boulders roughened, split and cut off flat under the ground.

use super::M;
use artkit::boulder;
use artkit::far::low;
use artkit::materials::Mat;
use artkit::math::{M4, V3, v3};
use artkit::mesh::{self, Mesh};
use artkit::noise::noise;
use artkit::parts::{join_all, measure};
use artkit::scene::Obj;
use artkit::tri;
use std::collections::BTreeMap;

/// Game (x, depth): the old part's middle.
pub const ROCK_CENTRE: (f64, f64) = (0.085, 0.01);
/// The rock's lumps reach this far under the ground: it is cut flat there.
pub const GROUND: f64 = -0.115;

/// Cuts `obj` (in place) by the plane through `co` facing `no`, dropping what lies in front of it and closing
/// the cut with a face of its own: a fracture, or the ground. The cut face is drawn flat. What is left is all
/// triangles. `world`: the part's matrix as Blender had it when it was cut (a part made and placed since Blender
/// last looked at the scene was still where it was made, at the origin, unturned).
pub fn _cut(obj: Obj, world: &M4, co: impl Into<V3>, no: impl Into<V3>) -> Obj {
    let (co, no) = (co.into(), no.into());
    obj.edit(|o| {
        o.mesh.transform(world);
        bisect_plane(&mut o.mesh, co, no);
        triangulate(&mut o.mesh);
        o.mesh.transform(&world.inverted().unwrap_or(M4::IDENTITY));
    });
    obj
}

// KIT: bmesh.ops.bisect_plane (clear_outer, its `dist` left at none) and bmesh.ops.holes_fill on its cut
/// What of `mesh` lies behind the plane through `co` facing `no`, the cut closed with a flat face of its own. A
/// point is in front of the plane or behind it, none on it (artkit's `forms::bisect` takes one within a tenth of a
/// millimetre for on it, and leaves the cut's face that far out of flat).
fn bisect_plane(mesh: &mut Mesh, co: V3, no: V3) {
    let dot = |a: V3, b: V3| a.x * b.x + a.y * b.y + a.z * b.z;
    let offset = -dot(no, co);
    let d: Vec<f32> = mesh.verts.iter().map(|&p| dot(p, no) + offset).collect();
    let front: Vec<bool> = d.iter().map(|&x| x > 0.0).collect();
    let mut new = Mesh {
        mats: mesh.mats.clone(),
        ..Mesh::default()
    };
    let mut kept = vec![u32::MAX; mesh.verts.len()];
    for (i, &p) in mesh.verts.iter().enumerate() {
        if !front[i] {
            kept[i] = new.verts.len() as u32;
            new.verts.push(p);
        }
    }
    // the point where an edge crosses the plane, made once
    let mut crossings: BTreeMap<(u32, u32), u32> = BTreeMap::new();
    let mut cross = |new: &mut Mesh, a: u32, b: u32| -> u32 {
        let key = mesh::edge(a, b);
        *crossings.entry(key).or_insert_with(|| {
            let (da, db) = (d[key.0 as usize], d[key.1 as usize]);
            let t = da / (da - db);
            let (p, q) = (mesh.verts[key.0 as usize], mesh.verts[key.1 as usize]);
            new.verts.push(V3::new(
                p.x + (q.x - p.x) * t,
                p.y + (q.y - p.y) * t,
                p.z + (q.z - p.z) * t,
            ));
            new.verts.len() as u32 - 1
        })
    };
    // each face: its corners behind the plane, with the crossings where it goes through
    let mut cut_edges: Vec<(u32, u32)> = Vec::new();
    for f in 0..mesh.faces.len() {
        let c = mesh.corners(f);
        let n = c.len();
        let mut out: Vec<u32> = Vec::new();
        let mut on_plane: Vec<u32> = Vec::new();
        for i in 0..n {
            let (a, b) = (c[i], c[(i + 1) % n]);
            if !front[a as usize] {
                out.push(kept[a as usize]);
            }
            if front[a as usize] != front[b as usize] {
                let x = cross(&mut new, a, b);
                out.push(x);
                on_plane.push(x);
            }
        }
        if out.len() >= 3 {
            // the side of it that lies along the plane, as the face runs it
            for i in 0..out.len() {
                let (a, b) = (out[i], out[(i + 1) % out.len()]);
                if on_plane.contains(&a) && on_plane.contains(&b) && a != b {
                    cut_edges.push((a, b));
                }
            }
            new.push_face(&out, mesh.faces[f].mat, mesh.faces[f].smooth);
        }
    }
    // the cut's edges chained into loops, each run against the faces' way so the face that closes it looks out
    let mut next: BTreeMap<u32, u32> = BTreeMap::new();
    for &(a, b) in &cut_edges {
        next.insert(b, a);
    }
    while let Some((&start, _)) = next.iter().next() {
        let mut ring = vec![start];
        let mut v = start;
        while let Some(w) = next.remove(&v) {
            if w == start {
                break;
            }
            ring.push(w);
            v = w;
        }
        if ring.len() >= 3 {
            new.push_face(&ring, 0, false);
        }
    }
    new.sharp = mesh
        .sharp
        .iter()
        .filter(|e| kept[e.0 as usize] != u32::MAX && kept[e.1 as usize] != u32::MAX)
        .map(|e| mesh::edge(kept[e.0 as usize], kept[e.1 as usize]))
        .collect();
    *mesh = new;
}

/// Every face of four corners or more cut into triangles (`bmesh.ops.triangulate` as it is by default): a quad
/// along the diagonal that leaves the better pair, a polygon by clipping its ears and turning their sides for the
/// better (`beautify`).
fn triangulate(mesh: &mut Mesh) {
    let mut new = Mesh {
        verts: mesh.verts.clone(),
        mats: mesh.mats.clone(),
        sharp: mesh.sharp.clone(),
        ..Mesh::default()
    };
    for f in 0..mesh.faces.len() {
        let c = mesh.corners(f);
        let face = mesh.faces[f];
        let at = |i: usize| mesh.verts[c[i] as usize];
        match c.len() {
            3 => new.push_face(c, face.mat, face.smooth),
            4 => {
                // (the diagonal between the first and third corners is kept where turning it would gain nothing)
                if rotate_beauty(at(1), at(2), at(3), at(0)) > 0.0 {
                    new.push_face(&[c[0], c[1], c[2]], face.mat, face.smooth);
                    new.push_face(&[c[0], c[2], c[3]], face.mat, face.smooth);
                } else {
                    new.push_face(&[c[0], c[1], c[3]], face.mat, face.smooth);
                    new.push_face(&[c[1], c[2], c[3]], face.mat, face.smooth);
                }
            }
            n => {
                let points: Vec<V3> = (0..n).map(at).collect();
                let normal = mesh.face_normal(f);
                let mut tris = tri::polygon(&points, normal);
                let axes = flat_axes([normal.x, normal.y, normal.z]);
                let xy: Vec<[f32; 2]> = points.iter().map(|&p| flat(&axes, p)).collect();
                beautify(&xy, &mut tris);
                for t in tris {
                    let corners = [c[t[0] as usize], c[t[1] as usize], c[t[2] as usize]];
                    new.push_face(&corners, face.mat, face.smooth);
                }
            }
        }
    }
    *mesh = new;
}

/// Two axes square to the unit vector `no`: a face seen along its normal (Blender's `axis_dominant_v3_to_m3`).
fn flat_axes(no: [f32; 3]) -> ([f32; 3], [f32; 3]) {
    let f = no[0] * no[0] + no[1] * no[1];
    if f > f32::EPSILON {
        let d = 1.0 / f.sqrt();
        let n1 = [no[1] * d, -no[0] * d, 0.0];
        (
            n1,
            [-no[2] * n1[1], no[2] * n1[0], no[0] * n1[1] - no[1] * n1[0]],
        )
    } else {
        (
            [if no[2] < 0.0 { -1.0 } else { 1.0 }, 0.0, 0.0],
            [0.0, 1.0, 0.0],
        )
    }
}

fn flat(axes: &([f32; 3], [f32; 3]), p: V3) -> [f32; 2] {
    let (n1, n2) = axes;
    [
        n1[0] * p.x + n1[1] * p.y + n1[2] * p.z,
        n2[0] * p.x + n2[1] * p.y + n2[2] * p.z,
    ]
}

/// Twice a flat triangle's area, signed by the way it winds.
fn area_2x(a: [f32; 2], b: [f32; 2], c: [f32; 2]) -> f32 {
    (a[0] - b[0]) * (b[1] - c[1]) + (a[1] - b[1]) * (c[0] - b[0])
}

/// What turning a quad's diagonal from v2-v4 to v1-v3 would gain (Blender's `BM_verts_calc_rotate_beauty`, by
/// area: GPL-2.0-or-later, worked through again here). Negative where v1-v3 is the better cut; single precision
/// throughout, as Blender's.
fn rotate_beauty(v1: V3, v2: V3, v3_: V3, v4: V3) -> f32 {
    let cross_tri = |a: V3, b: V3, c: V3| {
        let (n1, n2) = (
            [a.x - b.x, a.y - b.y, a.z - b.z],
            [b.x - c.x, b.y - c.y, b.z - c.z],
        );
        [
            n1[1] * n2[2] - n1[2] * n2[1],
            n1[2] * n2[0] - n1[0] * n2[2],
            n1[0] * n2[1] - n1[1] * n2[0],
        ]
    };
    let (no_a, no_b) = (cross_tri(v2, v3_, v4), cross_tri(v2, v4, v1));
    let mut no = [no_a[0] + no_b[0], no_a[1] + no_b[1], no_a[2] + no_b[2]];
    let no_scale = (no[0] * no[0] + no[1] * no[1] + no[2] * no[2]).sqrt();
    if no_scale <= 1.0e-35 {
        return f32::MAX;
    }
    no = no.map(|x| x * (1.0 / no_scale));
    let axes = flat_axes(no);
    let (p1, p2, p3, p4) = (
        flat(&axes, v1),
        flat(&axes, v2),
        flat(&axes, v3_),
        flat(&axes, v4),
    );
    let sign = |x: f32| {
        if x > 1e-5 {
            1
        } else if x < -1e-5 {
            -1
        } else {
            0
        }
    };
    // (wound opposite ways, or both without area: not a quad to turn a diagonal in)
    if sign(area_2x(p2, p3, p4) / no_scale) + sign(area_2x(p2, p4, p1) / no_scale) == 0 {
        return f32::MAX;
    }
    quad_rotate(p1, p2, p3, p4)
}

/// The same of a flat quad (Blender's `BLI_polyfill_beautify_quad_rotate_calc`): the two triangles' areas over
/// their perimeters, as they are less as they would be.
fn quad_rotate(p1: [f32; 2], p2: [f32; 2], p3: [f32; 2], p4: [f32; 2]) -> f32 {
    let len = |a: [f32; 2], b: [f32; 2]| {
        let (x, y) = (a[0] - b[0], a[1] - b[1]);
        (x * x + y * y).sqrt()
    };
    let (area_234, area_241) = (area_2x(p2, p3, p4), area_2x(p2, p4, p1));
    let (area_123, area_134) = (area_2x(p1, p2, p3), area_2x(p1, p3, p4));
    let eps = 1e-12_f32;
    // (turned, its two faces would look opposite ways or one would have no area: left)
    if (area_123 >= 0.0) != (area_134 >= 0.0) || area_123.abs() <= eps || area_134.abs() <= eps {
        return f32::MAX;
    }
    // (as it is they do: always turned)
    if (area_234 >= 0.0) != (area_241 >= 0.0) || area_234.abs() <= eps || area_241.abs() <= eps {
        return -f32::MAX;
    }
    let (len_12, len_23, len_34, len_41) = (len(p1, p2), len(p2, p3), len(p3, p4), len(p4, p1));
    let (len_13, len_24) = (len(p1, p3), len(p2, p4));
    let fac_24 =
        area_234.abs() / (len_23 + len_34 + len_24) + area_241.abs() / (len_41 + len_12 + len_24);
    let fac_13 =
        area_123.abs() / (len_12 + len_23 + len_13) + area_134.abs() / (len_34 + len_41 + len_13);
    fac_24 - fac_13
}

/// A polygon's triangles (`tris`, corners among `xy`) with every inner side turned that is the better for it,
/// the one that gains most first (Blender's `BLI_polyfill_beautify`): no slivers along a straight side.
fn beautify(xy: &[[f32; 2]], tris: &mut [[u32; 3]]) {
    loop {
        // each side's triangle and the corner of it across from the side, as the triangle runs it
        let mut sides: BTreeMap<(u32, u32), (usize, u32)> = BTreeMap::new();
        for (t, tri) in tris.iter().enumerate() {
            for i in 0..3 {
                sides.insert((tri[i], tri[(i + 1) % 3]), (t, tri[(i + 2) % 3]));
            }
        }
        let mut best: Option<(f32, usize, usize, [u32; 4])> = None;
        for (&(a, b), &(t, c)) in &sides {
            if a > b {
                continue;
            }
            let Some(&(u, d)) = sides.get(&(b, a)) else {
                continue;
            };
            let cost = quad_rotate(
                xy[c as usize],
                xy[a as usize],
                xy[d as usize],
                xy[b as usize],
            );
            if cost < 0.0 && best.is_none_or(|(least, ..)| cost < least) {
                best = Some((cost, t, u, [a, b, c, d]));
            }
        }
        let Some((_, t, u, [a, b, c, d])) = best else {
            break;
        };
        // (a, b, c) and (b, a, d) become (c, a, d) and (d, b, c)
        tris[t] = [c, a, d];
        tris[u] = [d, b, c];
    }
}

/// Roughens a boulder's surface with finer noise than `boulder`'s (its own metres, before its scale).
pub fn _lump(obj: Obj, amount: f64, scale: f64, seed: f64) {
    obj.edit(|o| {
        for v in &mut o.mesh.verts {
            let p = *v * scale + v3(seed, -seed, seed * 0.5);
            *v *= 1.0 + amount * noise(p);
        }
    });
}

/// A boulder `top` high (Blender z), roughened (`grain`: and pitted finer still, metres), with `fractures`
/// (direction, depth: how far in from its outside, metres) cut flat across it, then cut off under the ground.
#[allow(clippy::too_many_arguments)]
fn _stone(
    name: &str,
    radius: f64,
    at: (f64, f64, f64),
    material: Mat,
    scale: (f64, f64, f64),
    seed: f64,
    top: f64,
    subdivisions: usize,
    fractures: &[((f64, f64, f64), f64)],
    grain: f64,
) -> Obj {
    let o = boulder!(
        name,
        radius,
        at = at,
        material = material,
        scale = (1, 1, 1),
        seed = seed,
        noise = 0.26,
        subdivisions = subdivisions
    );
    _lump(o, 0.08, 4.0, seed);
    if grain != 0.0 {
        _lump(o, grain, 14.0, seed + 5.1);
    }
    o.edit(|p| p.mesh.transform(&M4::diagonal(scale.into())));
    for &(direction, depth) in fractures {
        let n = V3::from(direction).normalized();
        let reach = o.read(|p| {
            p.mesh
                .verts
                .iter()
                .map(|v| v.dot(n))
                .fold(f64::NEG_INFINITY, f64::max)
        });
        // (the stone's matrix is still the one it was made with, at the origin, while these are cut)
        _cut(o, &M4::IDENTITY, n * (reach - depth), n);
    }
    o.edit(|p| {
        let highest = p
            .mesh
            .verts
            .iter()
            .map(|v| v.z)
            .fold(f32::NEG_INFINITY, f32::max);
        p.loc.z = (top - highest as f64) as f32;
    });
    _cut(o, &o.matrix_world(), (0, 0, GROUND), (0, 0, -1))
}

/// Colours a stone's weathered faces (its rounded ones, not the fresh flat breaks): lichen in patches on what
/// faces the sky (smooth noise over `cover`), and the damp, earthy band where it goes into the ground.
fn _weather(obj: Obj, seed: f64, lichen: Mat, soil: Mat, cover: f64) {
    let world = obj.matrix_world();
    obj.edit(|o| {
        let (lichen, soil) = (o.mesh.slot(lichen), o.mesh.slot(soil));
        for f in 0..o.mesh.faces.len() {
            if !o.mesh.faces[f].smooth {
                continue;
            }
            let corners = o.mesh.corners(f);
            let middle = if let [a, b, c] = *corners {
                let (a, b, c) = (
                    o.mesh.verts[a as usize],
                    o.mesh.verts[b as usize],
                    o.mesh.verts[c as usize],
                );
                V3::new(
                    (a.x + b.x + c.x) / 3.0,
                    (a.y + b.y + c.y) / 3.0,
                    (a.z + b.z + c.z) / 3.0,
                )
            } else {
                panic!("a cut stone is all triangles");
            };
            let c = world * middle;
            if (c.z as f64) < 0.004 {
                o.mesh.faces[f].mat = soil;
            } else if o.mesh.face_normal(f).z as f64 > 0.45
                && noise(v3(
                    c.x as f64 * 7.0 + seed,
                    c.y as f64 * 7.0 - seed,
                    c.z as f64 * 7.0,
                )) > cover
            {
                o.mesh.faces[f].mat = lichen;
            }
        }
    });
}

/// A grey boulder and a lighter one leaning on it, lumpy and pitted (the old part's two blobs, where they stand),
/// faces split off flat, lichen in patches on their tops and the damp soil line round their foot, cut off under
/// the ground, two pebbles at the foot, all plain matte: one draw call. Its far version, what the woods draw (some
/// hundreds of them), is two icosahedra lumped the same way, without the split faces (each cut adds triangles) or
/// the pebbles, lichen or soil line (a patch would take a whole face): 56 triangles, few enough for the woods to
/// keep drawing them as one mesh (nature.ts `SPLIT_TRIANGLES`), as the old part's.
pub fn rock() -> Vec<Obj> {
    let fractures = [
        ((0.35, -0.2, 1.0), 0.07),
        ((-0.9, 0.3, 0.35), 0.06),
        ((0.2, 1.0, 0.1), 0.05),
        ((0.95, -0.15, 0.25), 0.035),
        ((-0.3, -1.0, 0.45), 0.03),
    ];
    // (Far off, an icosahedron, its corners on the round and its faces inside it: a little wider to fill the same
    // box.)
    let main = _stone(
        "rock_main",
        0.5,
        (0.0, 0.0, 0.0),
        M.stone,
        low((1.0, 0.85, 0.8), (1.08, 0.95, 0.8)),
        1.3,
        0.435,
        low(4, 1),
        low(&fractures[..], &[][..]),
        low(0.025, 0.0),
    );
    let side = _stone(
        "rock_side",
        0.25,
        (-0.42, -0.22, 0.0),
        M.stone_light,
        (1.0, 0.9, 0.9),
        3.7,
        0.24,
        low(3, 1),
        low(
            &[((-0.5, -0.6, 1.0), 0.04), ((-1.0, 0.2, 0.3), 0.025)][..],
            &[][..],
        ),
        low(0.03, 0.0),
    );
    if low(true, false) {
        _weather(main, 1.3, M.lichen, M.soil, 0.25);
        _weather(side, 3.7, M.lichen, M.soil, 0.1);
    }
    let (lo, hi) = measure(&[main, side]);
    if low(true, false) {
        // Two pebbles broken off it, half sunk at its foot (inside its box, so it is centred as it was).
        for (k, (x, y, r, sd)) in [(0.36, -0.3, 0.07, 6.2), (-0.56, 0.08, 0.055, 8.9)]
            .into_iter()
            .enumerate()
        {
            let peb = _stone(
                &format!("pebble{k}"),
                r,
                (x, y, 0.0),
                if k != 0 { M.stone_light } else { M.stone },
                (1.0, 0.8, 0.6),
                sd,
                0.035,
                2,
                &[],
                0.0,
            );
            _weather(peb, sd, M.lichen, M.soil, 0.6);
        }
    }
    // Centred where the old part's lumps are (game x 8.5 cm, depth 1 cm), by hand: a far version is built
    // without the Model's `centre` set.
    let obj = join_all("rock");
    obj.edit(|o| {
        o.loc += v3(
            -ROCK_CENTRE.0 - (lo[0] + hi[0]) / 2.0,
            -ROCK_CENTRE.1 - (lo[1] + hi[1]) / 2.0,
            0,
        )
    });
    vec![obj]
}

#[cfg(test)]
mod tests {
    use super::*;
    use artkit::boulder;
    use artkit::scene;
    use std::collections::BTreeMap;

    /// How many faces run each side, by its two points.
    fn sides(mesh: &Mesh) -> BTreeMap<(u32, u32), usize> {
        let mut sides = BTreeMap::new();
        for f in 0..mesh.faces.len() {
            let c = mesh.corners(f);
            for i in 0..c.len() {
                *sides
                    .entry(mesh::edge(c[i], c[(i + 1) % c.len()]))
                    .or_insert(0) += 1;
            }
        }
        sides
    }

    #[test]
    fn a_cut_stone_is_closed_triangles_behind_the_plane() {
        scene::reset();
        let stone = boulder!("stone", 0.3, seed = 1.5, subdivisions = 2);
        _lump(stone, 0.08, 4.0, 1.5);
        let no = v3(0.3, -0.2, 1).normalized();
        _cut(stone, &M4::IDENTITY, no * 0.4, no);
        _cut(stone, &M4::IDENTITY, (0, 0, 0.1), (0, 0, -1));
        stone.read(|o| {
            assert!(o.mesh.faces.iter().all(|f| f.len == 3));
            assert!(
                sides(&o.mesh).values().all(|&n| n == 2),
                "the cuts are closed"
            );
            for v in &o.mesh.verts {
                assert!(v.dot(no) <= 0.4 + 1e-6 && v.z >= 0.1 - 1e-6);
            }
            // the faces that close the cuts are flat, the stone's own smooth
            let flat = o.mesh.faces.iter().filter(|f| !f.smooth).count();
            assert!(flat > 0 && flat < o.mesh.faces.len());
        });
    }

    #[test]
    fn a_polygon_with_points_along_a_straight_side_is_cut_without_slivers() {
        // a square with three more points along its bottom side: clipped ear by ear, some triangles lie on that side
        let xy = [
            [0.0, 0.0],
            [0.25, 0.0],
            [0.5, 0.0],
            [0.75, 0.0],
            [1.0, 0.0],
            [1.0, 1.0],
            [0.0, 1.0],
        ];
        let mut tris = vec![[0, 1, 2], [0, 2, 3], [0, 3, 4], [0, 4, 5], [0, 5, 6]];
        beautify(&xy, &mut tris);
        for t in &tris {
            let area = area_2x(xy[t[0] as usize], xy[t[1] as usize], xy[t[2] as usize]);
            assert!(area > 0.1, "{t:?} is a sliver");
        }
        assert_eq!(tris.len(), 5);
    }

    #[test]
    fn a_quad_is_cut_along_its_shorter_better_diagonal() {
        // a kite, long from the first corner to the third: turning the diagonal gains
        let (a, b, c, d) = (v3(0, 0, 0), v3(1, 0.2, 0), v3(2, 0, 0), v3(1, -0.2, 0));
        assert!(rotate_beauty(b, c, d, a) < 0.0);
        assert!(rotate_beauty(c, d, a, b) > 0.0);
    }
}
