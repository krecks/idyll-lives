//! The boolean cuts, made as Blender's exact boolean made them for these two calls, without one: the retro
//! fridge's lofted shell opened through its front (`cut_shell`), and the retro mini fridge's bevelled cabinet
//! (`cut_cabinet`). Each leaves the pieces of the body's faces outside the cutter, in the body's own face order,
//! and adds the cutter's faces inside the body, taking the cutter's material (the modifier's `TRANSFER`), its points
//! numbered among the body's as Blender numbered them.

use artkit::materials::Mat;
use artkit::math::V3;
use artkit::mesh::{Mesh, edge};
use artkit::scene::Obj;

/// The cutter's box in `obj`'s own frame (its bevel made first, as the modifier would see it): its corners'
/// lowest and highest x, y and z.
fn bounds(obj: Obj, cutter: Obj) -> ([f32; 3], [f32; 3]) {
    artkit::bevel::apply(obj);
    let inv = obj
        .matrix_world()
        .inverted()
        .expect("a body is not flattened");
    let corners: Vec<V3> = cutter.world_points().iter().map(|&p| inv * p).collect();
    let lo = |i: usize| corners.iter().map(|c| c[i]).fold(f32::INFINITY, f32::min);
    let hi = |i: usize| {
        corners
            .iter()
            .map(|c| c[i])
            .fold(f32::NEG_INFINITY, f32::max)
    };
    ([lo(0), lo(1), lo(2)], [hi(0), hi(1), hi(2)])
}

/// The point at `t` of the way from `a` to `b`.
fn lerp(a: [f64; 3], b: [f64; 3], t: f64) -> [f64; 3] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

fn f64s(v: V3) -> [f64; 3] {
    [v.x as f64, v.y as f64, v.z as f64]
}

fn v3(p: [f64; 3]) -> V3 {
    V3::new(p[0] as f32, p[1] as f32, p[2] as f32)
}

/// Where the line through (x, z) along y meets a quad `q`, as Blender's exact boolean cuts it: on the triangle of
/// the two its first-to-third diagonal makes that the line passes through.
fn on_quad(q: [[f64; 3]; 4], x: f64, z: f64) -> [f64; 3] {
    let side = |p: [f64; 3], a: [f64; 3]| (a[0] - p[0]) * (z - p[2]) - (a[2] - p[2]) * (x - p[0]);
    let inside = |t: [[f64; 3]; 3]| {
        let (d0, d1, d2) = (side(t[0], t[1]), side(t[1], t[2]), side(t[2], t[0]));
        (d0 >= 0.0 && d1 >= 0.0 && d2 >= 0.0) || (d0 <= 0.0 && d1 <= 0.0 && d2 <= 0.0)
    };
    let first = [q[0], q[1], q[2]];
    let t = if inside(first) {
        first
    } else {
        [q[0], q[2], q[3]]
    };
    let (u, w) = (
        [t[1][0] - t[0][0], t[1][1] - t[0][1], t[1][2] - t[0][2]],
        [t[2][0] - t[0][0], t[2][1] - t[0][1], t[2][2] - t[0][2]],
    );
    let n = [
        u[1] * w[2] - u[2] * w[1],
        u[2] * w[0] - u[0] * w[2],
        u[0] * w[1] - u[1] * w[0],
    ];
    let y = t[0][1] - (n[0] * (x - t[0][0]) + n[2] * (z - t[0][2])) / n[1];
    [x, y, z]
}

/// The difference of the retro fridge's shell (a loft: rings of `m` points, bottom first, then its two caps) and a
/// box that opens it through its front (Blender -y): the box's floor lies between two rings (a and b), its ceiling
/// between the next two (b and c), its sides cut the front's strips between two of the rings' points. What is left
/// of the front strips is cut round the opening, the points of ring b inside the box go, and the box's back, sides,
/// floor and ceiling line the cut, flat, in the box's material.
pub fn cut_shell(obj: Obj, cutter: Obj) -> Obj {
    let (lo, hi) = bounds(obj, cutter);
    let mat = cutter.read(|o| o.mesh.mats[0]);
    obj.edit(|o| loft_cut(&mut o.mesh, mat, lo, hi));
    obj
}

fn loft_cut(mesh: &mut Mesh, mat: Mat, lo: [f32; 3], hi: [f32; 3]) {
    let (x0, x1, back, z0, z1) = (
        lo[0] as f64,
        hi[0] as f64,
        hi[1] as f64,
        lo[2] as f64,
        hi[2] as f64,
    );
    let m = mesh.faces[mesh.faces.len() - 1].len as usize;
    let rings = mesh.verts.len() / m;
    let p = |i: usize| f64s(mesh.verts[i]);
    let height = |r: usize| p(r * m)[2];
    let a = (0..rings - 1)
        .find(|&r| height(r) < z0 && z0 < height(r + 1))
        .expect("the box's floor lies between two rings");
    let (b, c) = (a + 1, a + 2);
    assert!(
        height(b) < z1 && z1 < height(c),
        "the box's ceiling lies between the next two rings"
    );
    let inside = |q: [f64; 3]| x0 < q[0] && q[0] < x1 && (lo[1] as f64) < q[1] && q[1] < back;
    // the front's points of ring a within the box's sides: i0..=ir
    let front: Vec<usize> = (0..m).filter(|&i| inside(p(a * m + i))).collect();
    let (i0, ir) = (front[0], front[front.len() - 1]);
    assert!(
        i0 > 0 && ir + 1 < m && front.len() == ir - i0 + 1,
        "the opening is a run of the front's points"
    );
    let il = i0 - 1;
    let gone: Vec<usize> = (0..m)
        .filter(|&i| inside(p(b * m + i)))
        .map(|i| b * m + i)
        .collect();
    assert!(
        gone == (i0..=ir).map(|i| b * m + i).collect::<Vec<_>>(),
        "ring b's front points go"
    );
    let (va, vb, vc) = (
        |i: usize| a * m + i,
        |i: usize| b * m + i,
        |i: usize| c * m + i,
    );
    let (za, zb, zc) = (height(a), height(b), height(c));
    // the new points
    let e0 = |i: usize| {
        let q = lerp(p(va(i)), p(vb(i)), (z0 - za) / (zb - za));
        [q[0], q[1], z0]
    };
    let e1 = |i: usize| {
        let q = lerp(p(vb(i)), p(vc(i)), (z1 - zb) / (zc - zb));
        [q[0], q[1], z1]
    };
    let on_ring_b = |i: usize, x: f64| {
        let (s, e) = (p(vb(i)), p(vb(i + 1)));
        let q = lerp(s, e, (x - s[0]) / (e[0] - s[0]));
        [x, q[1], q[2]]
    };
    let quad = |r: [usize; 4]| [p(r[0]), p(r[1]), p(r[2]), p(r[3])];
    let l0 = on_quad(quad([va(il), va(il + 1), vb(il + 1), vb(il)]), x0, z0);
    let r0 = on_quad(quad([va(ir), va(ir + 1), vb(ir + 1), vb(ir)]), x1, z0);
    let l1 = on_quad(quad([vb(il), vb(il + 1), vc(il + 1), vc(il)]), x0, z1);
    let r1 = on_quad(quad([vb(ir), vb(ir + 1), vc(ir + 1), vc(ir)]), x1, z1);
    let (lb, rb) = (on_ring_b(il, x0), on_ring_b(ir, x1));
    // the points' numbers: the old ones but ring b's that go, the box's back corners among them where Blender put
    // them, then the cut's points in the order Blender met them
    let corners = [
        [x0, back, z0],
        [x0, back, z1],
        [x1, back, z0],
        [x1, back, z1],
    ];
    let at = [2usize, 4, 8, 10];
    let n_old = mesh.verts.len();
    let mut remap = vec![u32::MAX; n_old];
    let mut verts: Vec<V3> = Vec::with_capacity(n_old + 24);
    let mut next = 0;
    let mut corner_at = [0u32; 4];
    for (k, &place) in at.iter().enumerate() {
        while verts.len() < place {
            if !gone.contains(&next) {
                remap[next] = verts.len() as u32;
                verts.push(mesh.verts[next]);
            }
            next += 1;
        }
        corner_at[k] = verts.len() as u32;
        verts.push(v3(corners[k]));
    }
    for i in next..n_old {
        if !gone.contains(&i) {
            remap[i] = verts.len() as u32;
            verts.push(mesh.verts[i]);
        }
    }
    let mut add = |q: [f64; 3]| {
        verts.push(v3(q));
        (verts.len() - 1) as u32
    };
    let n_l0 = add(l0);
    let mut n_e0 = vec![0u32; m];
    n_e0[i0] = add(e0(i0));
    let n_lb = add(lb);
    for (i, slot) in n_e0.iter_mut().enumerate().take(ir + 1).skip(i0 + 1) {
        *slot = add(e0(i));
    }
    let n_r0 = add(r0);
    let n_rb = add(rb);
    let n_l1 = add(l1);
    let mut n_e1 = vec![0u32; m];
    for (i, slot) in n_e1.iter_mut().enumerate().take(ir + 1).skip(i0) {
        *slot = add(e1(i));
    }
    let n_r1 = add(r1);
    let [bk00, bk01, bk10, bk11] = corner_at;
    let (pa, pb, pc) = (
        |i: usize| remap[va(i)],
        |i: usize| remap[vb(i)],
        |i: usize| remap[vc(i)],
    );
    // the front strips' faces round the opening, as Blender left them
    let mut cut: Vec<(usize, Vec<u32>)> = Vec::new();
    cut.push((
        a * m + il,
        vec![n_lb, pb(il), pa(il), pa(il + 1), n_e0[il + 1], n_l0],
    ));
    for k in i0..ir {
        cut.push((a * m + k, vec![n_e0[k], pa(k), pa(k + 1), n_e0[k + 1]]));
    }
    cut.push((
        a * m + ir,
        vec![n_e0[ir], pa(ir), pa(ir + 1), pb(ir + 1), n_rb, n_r0],
    ));
    cut.push((
        b * m + il,
        vec![n_e1[il + 1], pc(il + 1), pc(il), pb(il), n_lb, n_l1],
    ));
    for k in i0..ir {
        cut.push((b * m + k, vec![n_e1[k + 1], pc(k + 1), pc(k), n_e1[k]]));
    }
    cut.push((
        b * m + ir,
        vec![pb(ir + 1), pc(ir + 1), pc(ir), n_e1[ir], n_r1, n_rb],
    ));
    let old = std::mem::take(mesh);
    let mut new = Mesh {
        verts,
        sharp: old
            .sharp
            .iter()
            .filter(|&&(u, w)| remap[u as usize] != u32::MAX && remap[w as usize] != u32::MAX)
            .map(|&(u, w)| edge(remap[u as usize], remap[w as usize]))
            .collect(),
        mats: old.mats.clone(),
        ..Mesh::default()
    };
    for (f, face) in old.faces.iter().enumerate() {
        match cut.iter().find(|(g, _)| *g == f) {
            Some((_, c)) => new.push_face(c, face.mat, face.smooth),
            None => {
                let c: Vec<u32> = old.corners(f).iter().map(|&v| remap[v as usize]).collect();
                new.push_face(&c, face.mat, face.smooth);
            }
        }
    }
    // the box's faces inside the shell: its left side, back, right side, floor and ceiling
    let slot = new.slot(mat);
    let mut floor = vec![n_r0, bk10, bk00, n_l0];
    floor.extend((i0..=ir).map(|i| n_e0[i]));
    let mut ceiling = vec![n_e1[i0], n_l1, bk01, bk11, n_r1];
    ceiling.extend((i0 + 1..=ir).rev().map(|i| n_e1[i]));
    for c in [
        vec![bk00, bk01, n_l1, n_lb, n_l0],
        vec![bk00, bk10, bk11, bk01],
        vec![bk11, bk10, n_r0, n_rb, n_r1],
        floor,
        ceiling,
    ] {
        new.push_face(&c, slot, false);
    }
    *mesh = new;
}

/// Where Blender put the cutter's points that are kept (those behind the cabinet's front), among the cabinet's.
const CABINET_KEPT_AT: [usize; 28] = [
    14, 17, 19, 20, 22, 24, 27, 28, 30, 33, 35, 36, 39, 40, 56, 58, 60, 62, 64, 66, 69, 71, 73, 74,
    77, 79, 80, 83,
];
/// The cutter's points in front of the cabinet's face, on its edges that run through it, in the order Blender
/// numbered the points where those edges cross the face.
const CABINET_CROSSING: [usize; 12] = [2, 9, 37, 29, 8, 28, 41, 31, 35, 0, 6, 12];
/// The cutter's faces the front cuts, by their corners, with the corner Blender began each one's piece at.
const CABINET_STARTS: [([u32; 4], usize); 12] = [
    ([14, 0, 6, 17], 1),
    ([17, 6, 2, 15], 1),
    ([8, 21, 27, 12], 2),
    ([12, 27, 23, 9], 3),
    ([28, 42, 48, 31], 3),
    ([31, 48, 44, 29], 3),
    ([49, 35, 41, 52], 1),
    ([52, 41, 37, 50], 1),
    ([14, 42, 28, 0], 2),
    ([2, 9, 23, 15], 3),
    ([49, 21, 8, 35], 2),
    ([44, 50, 37, 29], 1),
];

/// The difference of the retro mini fridge's bevelled cabinet and its bevelled cavity (its bevel made), which
/// opens it through the flat of its front (Blender -y) and is as wide as that flat: the front's face is split
/// into the piece over the opening and the piece under it, the two faces beside it take the opening's corners on
/// their shared edges, and the cutter's faces behind the front line the cut, turned inward, in the cutter's
/// material.
pub fn cut_cabinet(obj: Obj, cutter: Obj) -> Obj {
    artkit::bevel::apply(obj);
    let inv = obj
        .matrix_world()
        .inverted()
        .expect("a cabinet is not flattened");
    let pts: Vec<V3> = cutter.world_points().iter().map(|&p| inv * p).collect();
    let inner = cutter.read(|o| o.mesh.clone());
    obj.edit(|o| box_cut(&mut o.mesh, &inner, &pts));
    obj
}

fn box_cut(mesh: &mut Mesh, inner: &Mesh, pts: &[V3]) {
    let front_y = mesh.verts.iter().map(|v| v.y).fold(f32::INFINITY, f32::min);
    let front = (0..mesh.faces.len())
        .find(|&f| {
            mesh.corners(f)
                .iter()
                .all(|&v| mesh.verts[v as usize].y == front_y)
        })
        .expect("the cabinet has a flat front");
    let kept: Vec<usize> = (0..pts.len()).filter(|&i| pts[i].y > front_y).collect();
    assert_eq!(
        kept.len(),
        CABINET_KEPT_AT.len(),
        "the cutter is the cavity Blender cut"
    );
    // the front's corners, and the points where the cutter's edges cross it
    let corner = |left: bool, low: bool| -> u32 {
        let c = mesh.corners(front);
        let key = |v: &u32| {
            let p = mesh.verts[*v as usize];
            (if left { p.x } else { -p.x }) + (if low { p.z } else { -p.z })
        };
        *c.iter()
            .min_by(|a, b| key(a).partial_cmp(&key(b)).unwrap())
            .unwrap()
    };
    let (bl, br, tl, tr) = (
        corner(true, true),
        corner(false, true),
        corner(true, false),
        corner(false, false),
    );
    let n_old = mesh.verts.len();
    let mut remap = vec![0u32; n_old];
    let mut inner_at = vec![u32::MAX; pts.len()];
    let mut verts: Vec<V3> = Vec::with_capacity(n_old + 40);
    let mut next = 0;
    for (k, &place) in CABINET_KEPT_AT.iter().enumerate() {
        while verts.len() < place {
            remap[next] = verts.len() as u32;
            verts.push(mesh.verts[next]);
            next += 1;
        }
        inner_at[kept[k]] = verts.len() as u32;
        verts.push(pts[kept[k]]);
    }
    for (i, slot) in remap.iter_mut().enumerate().skip(next) {
        *slot = verts.len() as u32;
        verts.push(mesh.verts[i]);
    }
    for &i in &CABINET_CROSSING {
        assert!(pts[i].y < front_y, "a crossing edge's front point");
        inner_at[i] = verts.len() as u32;
        verts.push(V3::new(pts[i].x, front_y, pts[i].z));
    }
    // the crossings by where they lie round the opening
    let (x0, x1) = CABINET_CROSSING
        .iter()
        .fold((f32::INFINITY, f32::NEG_INFINITY), |(a, b), &i| {
            (a.min(pts[i].x), b.max(pts[i].x))
        });
    let (z0, z1) = CABINET_CROSSING
        .iter()
        .fold((f32::INFINITY, f32::NEG_INFINITY), |(a, b), &i| {
            (a.min(pts[i].z), b.max(pts[i].z))
        });
    let (xm, zm) = ((x0 + x1) / 2.0, (z0 + z1) / 2.0);
    let find = |test: &dyn Fn(V3) -> bool| -> u32 {
        let i = *CABINET_CROSSING
            .iter()
            .find(|&&i| test(pts[i]))
            .expect("the opening has its corners");
        inner_at[i]
    };
    let on_side = |p: V3| p.x == x0 || p.x == x1;
    let on_cap = |p: V3| p.z == z0 || p.z == z1;
    let side =
        |left: bool, low: bool| find(&|p| on_side(p) && (p.x < xm) == left && (p.z < zm) == low);
    let cap =
        |left: bool, low: bool| find(&|p| on_cap(p) && (p.x < xm) == left && (p.z < zm) == low);
    let mid = |left: bool, low: bool| {
        find(&|p| !on_side(p) && !on_cap(p) && (p.x < xm) == left && (p.z < zm) == low)
    };
    let (bl, br, tl, tr) = (
        remap[bl as usize],
        remap[br as usize],
        remap[tl as usize],
        remap[tr as usize],
    );
    let top = vec![
        side(false, false),
        tr,
        tl,
        side(true, false),
        mid(true, false),
        cap(true, false),
        cap(false, false),
        mid(false, false),
    ];
    let bottom = vec![
        side(true, true),
        bl,
        br,
        side(false, true),
        mid(false, true),
        cap(false, true),
        cap(true, true),
        mid(true, true),
    ];
    // the front's two side edges, each with the two corners of the opening on it, from its first end
    let edges = [
        ((bl, tl), [side(true, true), side(true, false)]),
        ((tr, br), [side(false, false), side(false, true)]),
    ];
    let split = |c: &[u32]| -> Option<Vec<u32>> {
        for &((p, q), on) in &edges {
            for (i, &v) in c.iter().enumerate() {
                let w = c[(i + 1) % c.len()];
                let (first, second) = if (v, w) == (p, q) {
                    (on[0], on[1])
                } else if (v, w) == (q, p) {
                    (on[1], on[0])
                } else {
                    continue;
                };
                // (the face starts at the second of the two)
                let mut out = vec![second, w];
                out.extend((2..c.len()).map(|k| c[(i + k) % c.len()]));
                out.extend([v, first]);
                return Some(out);
            }
        }
        None
    };
    let old = std::mem::take(mesh);
    let mut sharp = std::collections::BTreeSet::new();
    for &(a, b) in &old.sharp {
        let (a, b) = (remap[a as usize], remap[b as usize]);
        match edges.iter().find(|((p, q), _)| edge(*p, *q) == edge(a, b)) {
            Some(&((p, q), on)) => {
                sharp.extend([edge(p, on[0]), edge(on[0], on[1]), edge(on[1], q)]);
            }
            None => {
                sharp.insert(edge(a, b));
            }
        }
    }
    for &(a, b) in &inner.sharp {
        // (an edge of the cutter's front is gone, though its two crossings may be joined by the rim)
        let behind = |v: u32| pts[v as usize].y > front_y;
        let (from, to) = (inner_at[a as usize], inner_at[b as usize]);
        if from != u32::MAX && to != u32::MAX && (behind(a) || behind(b)) {
            sharp.insert(edge(from, to));
        }
    }
    let mut new = Mesh {
        verts,
        sharp,
        mats: old.mats.clone(),
        ..Mesh::default()
    };
    for (f, face) in old.faces.iter().enumerate() {
        let c: Vec<u32> = old.corners(f).iter().map(|&v| remap[v as usize]).collect();
        if f == front {
            new.push_face(&top, face.mat, face.smooth);
            new.push_face(&bottom, face.mat, face.smooth);
        } else {
            new.push_face(&split(&c).unwrap_or(c), face.mat, face.smooth);
        }
    }
    // the cutter's faces behind the front, turned inward, in its order and its material
    let slot = new.slot(inner.mats[0]);
    for (f, face) in inner.faces.iter().enumerate() {
        let c = inner.corners(f);
        if !c.iter().any(|&v| pts[v as usize].y > front_y) {
            continue;
        }
        let start = CABINET_STARTS
            .iter()
            .find(|(k, _)| k[..] == *c)
            .map_or(0, |&(_, s)| s);
        let n = c.len();
        let turned: Vec<u32> = (0..n)
            .map(|k| {
                let v = inner_at[c[(start + n - k) % n] as usize];
                assert!(
                    v != u32::MAX,
                    "a cut face's point in front crosses the front"
                );
                v
            })
            .collect();
        new.push_face(&turned, slot, face.smooth);
    }
    *mesh = new;
}
