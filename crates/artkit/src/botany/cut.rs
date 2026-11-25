//! Leaves cut from an outline (`cut_leaf`: lobes, slits, leaflets, a wavy or toothed edge) and the outlines they are
//! cut from: families/plants.py's detail pass.

use crate::botany::leaf::profile;
use crate::botany::polyfill;
use crate::far::is_low;
use crate::materials::Mat;
use crate::math::{
    Axis, M4, PI, TAU, V3, cos, dot_v3, exp, len_v3, mul_v3_fl, normalize_v3, pow, radians, sin,
    sqrt, v3,
};
use crate::mesh::{Mesh, edge, normal_of};
use crate::scene::{self, Obj};
use crate::shapes::options;
use std::collections::{BTreeMap, BTreeSet};

/// A leaf's shape: how wide it is (0..1 of its width) at `t` of the way along it. One of `PROFILES` by name, or
/// any function of t.
pub trait Shape {
    fn at(&self, t: f64) -> f64;
}

impl Shape for &str {
    fn at(&self, t: f64) -> f64 {
        profile(self)(t)
    }
}

impl<F: Fn(f64) -> f64> Shape for F {
    fn at(&self, t: f64) -> f64 {
        self(t)
    }
}

options! {
    /// `cut_leaf`'s options (`left` None: the right half's mirror).
    CutLeaf { droop: f64 = 0.0, fold: f64 = 0.25, thick: f64 = 0.006, ribs: usize = 5, twist: f64 = 0.0, left: Option<Vec<(f64, f64)>> = None }
}

/// A leaf cut from an outline: `edge` is its right half [(t, w), ...] from the stalk (t 0) to the tip (t 1), `w`
/// metres out from the midrib (a lobe, a slit, a leaflet: t may go back), the left half its mirror (or `left`). The
/// blade is closed and thin: the top and the underside meet at the edge and stand `thick` apart along the midrib
/// (`ribs` points), with no walls, so it costs two triangles a point of its outline and four a midrib point. It
/// bends down along its length (`droop`, as `leaf`), lifts its halves into a V (`fold`) and turns about its midrib
/// by the tip (`twist`, degrees); it starts at `base` (Blender metres), heading `yaw` round from the back and `tilt`
/// up. A far version takes every third point of the outline and a third of the midrib.
#[allow(clippy::too_many_arguments)]
pub fn cut_leaf(
    name: impl AsRef<str>,
    base: impl Into<V3>,
    yaw: f64,
    tilt: f64,
    length: f64,
    edge: &[(f64, f64)],
    material: Mat,
    o: CutLeaf,
) -> Obj {
    let mut right = edge.to_vec();
    let mut lft = o.left.clone().unwrap_or_else(|| edge.to_vec());
    let mut ribs = o.ribs;
    if is_low() {
        // every third point from the second on, or the first of fewer than two
        let third = |side: &[(f64, f64)]| -> Vec<(f64, f64)> {
            let kept: Vec<(f64, f64)> = side.iter().skip(1).step_by(3).copied().collect();
            if kept.is_empty() {
                side[..side.len().min(1)].to_vec()
            } else {
                kept
            }
        };
        (right, lft) = (third(&right), third(&lft));
        ribs = 1.max(ribs / 3);
    }
    let bend = 2.0 * o.droop;
    let place =
        |x: f64, t: f64, z: f64| crate::botany::leaf::on_midrib(length, bend, o.twist, x, t, z);
    // each point drawn flat (where the faces are cut into triangles), and where it goes
    let (mut flat, mut placed): (Vec<V3>, Vec<V3>) = (Vec::new(), Vec::new());
    let mut vert = |x: f64, t: f64, z: f64| -> u32 {
        flat.push(v3(x, t * length, 0.0));
        placed.push(place(x, t, z));
        flat.len() as u32 - 1
    };
    let ts: Vec<f64> = (0..ribs)
        .map(|i| (i + 1) as f64 / (ribs + 1) as f64)
        .collect();
    let ht = (o.thick / 2.0).max(0.0008);
    let (b, tip) = (vert(0.0, 0.0, 0.0), vert(0.0, 1.0, 0.0));
    let rise = |t: f64| pow(sin(PI * 1.0_f64.min(t * 1.6)), 0.5);
    let top: Vec<u32> = ts.iter().map(|&t| vert(0.0, t, ht * rise(t))).collect();
    let under: Vec<u32> = ts.iter().map(|&t| vert(0.0, t, -ht * rise(t))).collect();
    let rv: Vec<u32> = right.iter().map(|&(t, w)| vert(w, t, o.fold * w)).collect();
    let lv: Vec<u32> = lft.iter().map(|&(t, w)| vert(-w, t, o.fold * w)).collect();
    // the four faces: each half of the top and of the underside, from the stalk up the midrib and back down the edge
    let mut rings: Vec<Vec<u32>> = Vec::new();
    let mut made: BTreeSet<Vec<u32>> = BTreeSet::new();
    for (mid, up) in [(&top, true), (&under, false)] {
        for (side, left) in [(&rv, false), (&lv, true)] {
            let mut ring = vec![b];
            ring.extend(mid);
            ring.push(tip);
            ring.extend(side.iter().rev());
            if left {
                ring.reverse();
            }
            if !up {
                ring.reverse();
            }
            // (a face on the points of one there already isn't made: a leaf with no midrib point has no underside)
            let mut key = ring.clone();
            key.sort_unstable();
            if ring.len() < 3 || !made.insert(key) {
                continue;
            }
            rings.push(ring);
        }
    }
    // cut into triangles as Blender cuts them: each face's last triangle takes its place, the others follow
    let (mut firsts, mut rest): (Vec<Vec<u32>>, Vec<Vec<u32>>) = (Vec::new(), Vec::new());
    let (mut stale, mut stale_rest): (Vec<V3>, Vec<V3>) = (Vec::new(), Vec::new());
    for ring in &rings {
        let points: Vec<V3> = ring.iter().map(|&v| flat[v as usize]).collect();
        let normal = normal_of(&points);
        let tris = polyfill::triangulate(&points, normal);
        let corners = |t: &[u32; 3]| t.map(|i| ring[i as usize]).to_vec();
        let (last, others) = tris.split_last().expect("a face has three corners");
        firsts.push(corners(last));
        rest.extend(others.iter().map(corners));
        stale.push(normal);
        stale_rest.extend(others.iter().map(|_| normal));
    }
    stale.extend(stale_rest);
    firsts.extend(rest);
    // (each triangle keeps its face's normal as it was drawn flat: Blender doesn't work them out again before it
    // turns the faces to look outwards)
    let mut faces = firsts;
    turn_outwards(&placed, &mut faces, &stale);
    let mut mesh = Mesh::from_pydata(placed, &faces);
    let m = M4::translation(base.into())
        * M4::rotation(radians(-yaw), Axis::Z)
        * M4::rotation(radians(tilt), Axis::X);
    mesh.transform(&m);
    mesh.set_material(Some(material));
    sharp_from_angle(&mut mesh, 70.0);
    scene::add(name.as_ref(), mesh)
}

/// `mesh.shade_smooth()` then `mesh.set_sharp_from_angle(angle)` on a mesh whose edges may have more than two
/// faces (a leaf's two sheets share their cuts where both were cut alike): smooth, but for the edges whose first
/// two faces, in the mesh's order, turn more than `degrees`; a third face on an edge takes no mark off it.
fn sharp_from_angle(mesh: &mut Mesh, degrees: f64) {
    for f in &mut mesh.faces {
        f.smooth = true;
    }
    mesh.sharp.clear();
    let angle = radians(degrees) as f32;
    if angle as f64 >= PI {
        return;
    }
    let split = cos(angle as f64) as f32;
    let normals = mesh.face_normals();
    // each edge's first face and the vertex it ran the edge from, until a second face came
    let mut first: BTreeMap<(u32, u32), Option<(usize, u32)>> = BTreeMap::new();
    for f in 0..mesh.faces.len() {
        let c = mesh.corners(f).to_vec();
        for (i, &v) in c.iter().enumerate() {
            let e = edge(v, c[(i + 1) % c.len()]);
            match first.get_mut(&e) {
                None => {
                    first.insert(e, Some((f, v)));
                }
                Some(open) => {
                    if let Some((g, _)) = open.take()
                        && dot_v3(normals[g], normals[f]) < split
                    {
                        mesh.sharp.insert(e);
                    }
                }
            }
        }
    }
}

/// `bmesh.ops.recalc_face_normals` on a mesh a script built face by face: each piece of faces joined across
/// edges of two faces is wound one way, the way that has the face at its outermost corner look outwards. A leaf's
/// two sheets share their cuts where both were cut alike, so it falls into several pieces, some of no thickness;
/// which way those come out hangs on the order Blender goes through their faces and on the normals the faces had
/// before (`stale`, one a face), so both are its own here.
fn turn_outwards(verts: &[V3], faces: &mut [Vec<u32>], stale: &[V3]) {
    // the faces on each edge, in the mesh's order
    let mut on_edge: BTreeMap<(u32, u32), Vec<usize>> = BTreeMap::new();
    for (f, c) in faces.iter().enumerate() {
        for (i, &v) in c.iter().enumerate() {
            on_edge
                .entry(edge(v, c[(i + 1) % c.len()]))
                .or_default()
                .push(f);
        }
    }
    // the face across the side of `f` from its corner `i`, where that edge has two faces
    let across = |faces: &[Vec<u32>], f: usize, i: usize| -> Option<usize> {
        let c = &faces[f];
        let on = &on_edge[&edge(c[i], c[(i + 1) % c.len()])];
        if on.len() != 2 || on[0] == on[1] {
            return None;
        }
        Some(if on[0] == f { on[1] } else { on[0] })
    };
    // the pieces: from each face not yet in one, the last face found goes first
    let mut tagged = vec![false; faces.len()];
    let mut groups: Vec<Vec<usize>> = Vec::new();
    for seed in 0..faces.len() {
        if tagged[seed] {
            continue;
        }
        tagged[seed] = true;
        let (mut stack, mut group) = (vec![seed], Vec::new());
        while let Some(f) = stack.pop() {
            group.push(f);
            for i in 0..faces[f].len() {
                if let Some(other) = across(faces, f, i)
                    && !tagged[other]
                {
                    tagged[other] = true;
                    stack.push(other);
                }
            }
        }
        groups.push(group);
    }
    let mut flips = vec![false; faces.len()];
    for group in &groups {
        let (start, flip) = outermost(verts, faces, stale, group);
        let mut seen = vec![false; faces.len()];
        (seen[start], flips[start]) = (true, flip);
        let mut stack = vec![start];
        while let Some(f) = stack.pop() {
            let c = &faces[f];
            for i in 0..c.len() {
                let Some(other) = across(faces, f, i) else {
                    continue;
                };
                if seen[other] {
                    continue;
                }
                seen[other] = true;
                // a neighbour running the shared edge the same way is the other way up
                let o = &faces[other];
                let same_way = (0..o.len())
                    .any(|k| o[k] == c[i] && o[(k + 1) % o.len()] == c[(i + 1) % c.len()]);
                flips[other] = same_way != flips[f];
                stack.push(other);
            }
        }
    }
    for (face, flip) in faces.iter_mut().zip(flips) {
        if flip {
            face[1..].reverse();
        }
    }
}

/// Of a piece's faces (in Blender's order): the one at the corner farthest from the piece's middle, and whether it
/// looks inwards there.
fn outermost(verts: &[V3], faces: &[Vec<u32>], stale: &[V3], group: &[usize]) -> (usize, bool) {
    let eps = f32::EPSILON;
    let points = |f: usize| -> Vec<V3> { faces[f].iter().map(|&v| verts[v as usize]).collect() };
    let k = 1.0 / group.len() as f32;
    let (mut cent, mut area_sum) = (V3::ZERO, 0.0_f32);
    for &f in group {
        let p = points(f);
        // (the face's area by Newell's sum, and its middle with each corner weighed by its two sides)
        let mut n = V3::ZERO;
        for (i, &cur) in p.iter().enumerate() {
            let next = p[(i + 1) % p.len()];
            n.x += (cur.y - next.y) * (cur.z + next.z);
            n.y += (cur.z - next.z) * (cur.x + next.x);
            n.z += (cur.x - next.x) * (cur.y + next.y);
        }
        let area = len_v3(n) * 0.5;
        let (mut c, mut total) = (V3::ZERO, 0.0_f32);
        let mut w_prev = len_v3(p[p.len() - 1] - p[0]);
        for i in 0..p.len() {
            let w_cur = len_v3(p[i] - p[(i + 1) % p.len()]);
            let w = w_cur + w_prev;
            c += mul_v3_fl(p[i], w);
            total += w;
            w_prev = w_cur;
        }
        if total != 0.0 {
            c = mul_v3_fl(c, 1.0 / total);
        }
        cent += mul_v3_fl(c, k * area);
        area_sum += area;
    }
    if area_sum != 0.0 {
        cent = mul_v3_fl(cent, 1.0 / area_sum);
    }
    let (mut best_dist, mut best_edge, mut best_loop) = (eps, f32::MIN, f32::MIN);
    let (mut start, mut flip) = (group[0], false);
    for &f in group {
        let p = points(f);
        let n = p.len();
        for i in 0..n {
            let dir = p[i] - cent;
            let dist = dot_v3(dir, dir);
            let far = dist > best_dist;
            if !(far || dist == best_dist) {
                continue;
            }
            let dir = mul_v3_fl(dir, 1.0 / dist.sqrt());
            let (e0, l0) = normalize_v3(p[(i + 1) % n] - p[i]);
            let (e1, l1) = normalize_v3(p[(i + n - 1) % n] - p[i]);
            if !(l0 > eps && l1 > eps) {
                continue;
            }
            let edge_dot = dot_v3(dir, e0).max(dot_v3(dir, e1));
            let better_edge = edge_dot > best_edge;
            if !(far || better_edge || edge_dot == best_edge) {
                continue;
            }
            let (mut loop_dir, lc) = normalize_v3(e0.cross(e1));
            if lc > eps {
                if dot_v3(loop_dir, stale[f]) < 0.0 {
                    loop_dir = -loop_dir;
                }
                let loop_dot = dot_v3(dir, loop_dir);
                if far || better_edge || loop_dot.abs() > best_loop {
                    (best_dist, best_edge, best_loop) = (dist, edge_dot, loop_dot.abs());
                    start = f;
                    flip = loop_dot < 0.0;
                }
            }
        }
    }
    (start, flip)
}

options! {
    /// `outline`'s options.
    OutlineOf { n: usize = 6, wave: f64 = 0.0, waves: f64 = 0.0, teeth: f64 = 0.0 }
}

/// A leaf's right half for `cut_leaf`: `n` points of `shape` (`PROFILES`, or a function of t) `width` across, its
/// edge waving (`wave` of the width, `waves` times along) or toothed (`teeth`: every other point drawn in by that
/// share).
pub fn outline(shape: impl Shape, width: f64, o: OutlineOf) -> Vec<(f64, f64)> {
    let mut out = Vec::with_capacity(o.n);
    for i in 0..o.n {
        let t = (i as f64 + 0.5) / o.n as f64;
        let mut w = width / 2.0 * shape.at(t) * (1.0 + o.wave * sin(PI * o.waves * t));
        if o.teeth != 0.0 && i % 2 != 0 {
            w *= 1.0 - o.teeth;
        }
        out.push((t, w.max(0.0005)));
    }
    out
}

options! {
    /// `pinnate`'s options.
    Pinnate { lean: f64 = 0.8, fill: f64 = 0.45, start: f64 = 0.06, round_tip: bool = false, stalk: f64 = 0.0012 }
}

/// A frond's right half for `cut_leaf`: `pairs` leaflets along it, each from the midrib (`stalk` wide there) out to
/// a tip swept forward by `lean` of a leaflet's spacing, back to the midrib `fill` of the spacing on, so the leaflets
/// stand apart; their reach follows `shape` (the script's default: "lance"). `round_tip`: two more points round
/// each (broad leaflets).
pub fn pinnate(pairs: usize, width: f64, shape: impl Shape, o: Pinnate) -> Vec<(f64, f64)> {
    let (lean, fill, start, stalk) = (o.lean, o.fill, o.start, o.stalk);
    let step = (1.0 - start) / pairs as f64;
    let mut out = Vec::new();
    for i in 0..pairs {
        let t0 = start + i as f64 * step;
        let tt = 0.985_f64.min(t0 + step * lean);
        let reach = width / 2.0 * 0.05_f64.max(shape.at(tt));
        out.push((t0, stalk));
        if o.round_tip {
            out.push((t0 + step * lean * 0.35, reach * 0.75));
        }
        out.push((tt, reach));
        if o.round_tip {
            out.push((
                0.99_f64.min(t0 + step * (fill * 0.5 + lean * 0.6)),
                reach * 0.7,
            ));
        }
        out.push((t0 + step * fill, stalk));
    }
    out
}

options! {
    /// `slits`' options.
    Slits { depth: f64 = 0.7, gap: f64 = 0.25 }
}

/// A split leaf's right half (a monstera's): `n` lobes, each cut from the edge `depth` of the way in to the
/// midrib, the cuts `gap` of a lobe wide.
pub fn slits(shape: impl Shape, width: f64, n: usize, o: Slits) -> Vec<(f64, f64)> {
    let (depth, gap) = (o.depth, o.gap);
    let mut out = Vec::new();
    for i in 0..n {
        let (t0, t1) = ((i as f64 + 0.08) / n as f64, (i + 1) as f64 / n as f64);
        let w = width / 2.0 * shape.at((t0 + t1) / 2.0);
        out.extend([
            (t0, w * 0.92),
            (t0 * 0.4 + t1 * 0.6, w),
            (t1 - gap / n as f64, w * 0.9),
        ]);
        if i < n - 1 {
            out.push((t1 - gap / n as f64 * 0.4, 0.004_f64.max(w * (1.0 - depth))));
        }
    }
    out
}

/// A fiddle-leaf fig's leaf: broad near its tip, waisted below the middle, narrowing to the stalk.
pub fn fiddle(t: f64) -> f64 {
    let w = pow(sin(PI * 1.0_f64.min(t * 1.08)), 0.45)
        * (0.55 + 0.45 * t)
        * (1.0 - 0.22 * exp(-pow((t - 0.42) / 0.12, 2.0)));
    if w > 0.0 { w } else { 0.0 }
}

/// A snake plant's blade: full width from low down, narrowing to a sharp point.
pub fn sword(t: f64) -> f64 {
    pow(1.0_f64.min(t * 6.0), 0.5) * pow(1.0 - t, 0.45)
}

options! {
    /// `lobed`'s options.
    Lobed { depth: f64 = 0.35 }
}

/// A palmate leaf's right half, as a width profile: `lobes` rounded lobes (on both halves together), cut in by
/// `depth`.
pub fn lobed(lobes: impl crate::math::Num, o: Lobed) -> impl Fn(f64) -> f64 + Copy {
    let (lobes, depth) = (lobes.f(), o.depth);
    move |t: f64| {
        pow(sin(PI * 1.0_f64.min(0.15 + t * 0.85)), 0.6)
            * (1.0 - depth * pow(sin(PI * lobes / 2.0 * t).abs(), 3.0))
    }
}

options! {
    /// `sinuate`'s options.
    Sinuate { depth: f64 = 0.45 }
}

/// An oak leaf's right half, as a width profile: `pairs` rounded lobes a side with narrow bays between them, cut in
/// by `depth`, on a blade widest above its middle that narrows to two small ears at the stalk.
pub fn sinuate(pairs: impl crate::math::Num, o: Sinuate) -> impl Fn(f64) -> f64 + Copy {
    let (pairs, depth) = (pairs.f(), o.depth);
    move |t: f64| {
        let blade = pow(sin(PI * pow(t, 1.35)), 0.75) + 0.2 * exp(-pow((t - 0.05) / 0.045, 2.0));
        let lobe = 0.5 - 0.5 * cos(TAU * pairs * t - 0.9);
        blade * (1.0 - depth * (1.0 - pow(lobe, 0.6)))
    }
}

options! {
    /// `palmate`'s options.
    Palmate { cut: f64 = 0.4, bulge: f64 = 0.07, teeth: usize = 1, spread: f64 = 105.0, fall: f64 = 0.84 }
}

/// A palmately lobed leaf's right half for `cut_leaf` (a maple's), `length` metres from the stalk's end to the
/// middle lobe's tip: `lobes` pointed lobes in all (an odd number: one down the middle) fanning from the stalk's
/// end, the outermost `spread` degrees round from the tip, each `fall` as long as the one before it, the bays
/// between them `cut` of the way in to the stalk's end; each side of a lobe stands out by `bulge` of its length:
/// as a point with a notch behind it (`teeth`: a Norway maple's) or as a shoulder (none: a Japanese maple's
/// narrow lobes). The lowest lobes reach back past the stalk's end (t under 0).
pub fn palmate(lobes: usize, length: f64, o: Palmate) -> Vec<(f64, f64)> {
    let side = (lobes.max(3) - 1) / 2;
    let at = |r: f64, a: f64| (r * cos(a), r * sin(a));
    let turn = radians(o.spread) / side as f64;
    let long = |k: usize| pow(o.fall, k as f64);
    // the bay between lobe k and the one before it (nearer the tip), and the leaf's foot past the last
    let bay = |k: usize| {
        at(
            (1.0 - o.cut) * (long(k) + long(k - 1)) / 2.0,
            turn * (k as f64 - 0.5),
        )
    };
    let foot = at(
        (1.0 - o.cut) * long(side) * 0.55,
        (turn * side as f64 + radians(38.0)).min(radians(172.0)),
    );
    // a lobe's side from `from` (a bay) out to its `tip`, standing out to the side away from `axis` (the lobe's)
    let flank = |from: (f64, f64), tip: (f64, f64), axis: f64, long: f64| -> Vec<(f64, f64)> {
        let (dx, dy) = (tip.0 - from.0, tip.1 - from.1);
        let d = sqrt(dx * dx + dy * dy).max(1e-9);
        let mut out = (-dy / d, dx / d);
        let mid = ((from.0 + tip.0) / 2.0, (from.1 + tip.1) / 2.0);
        // (away from the lobe's own line)
        let (ax, ay) = (cos(axis), sin(axis));
        let off = (
            mid.0 - ax * (mid.0 * ax + mid.1 * ay),
            mid.1 - ay * (mid.0 * ax + mid.1 * ay),
        );
        if out.0 * off.0 + out.1 * off.1 < 0.0 {
            out = (-out.0, -out.1);
        }
        let on = |f: f64, by: f64| {
            (
                from.0 + dx * f + out.0 * by * long,
                from.1 + dy * f + out.1 * by * long,
            )
        };
        if o.teeth > 0 {
            vec![
                on(0.3, o.bulge * 0.45),
                on(0.5, o.bulge),
                on(0.58, o.bulge * 0.2),
                on(0.8, o.bulge * 0.3),
            ]
        } else {
            vec![
                on(0.25, o.bulge * 0.7),
                on(0.5, o.bulge),
                on(0.78, o.bulge * 0.6),
            ]
        }
    };
    let mut points = vec![foot];
    for k in (1..=side).rev() {
        let (axis, tip) = (turn * k as f64, at(long(k), turn * k as f64));
        let below = if k == side { foot } else { bay(k + 1) };
        points.extend(flank(below, tip, axis, long(k)));
        points.push(tip);
        let mut above = flank(bay(k), tip, axis, long(k));
        above.reverse();
        points.extend(above);
        points.push(bay(k));
    }
    points.extend(flank(bay(1), (1.0, 0.0), 0.0, 1.0));
    points
        .into_iter()
        .map(|(t, w)| (t, (w * length).max(0.0005)))
        .collect()
}

options! {
    /// `rose_leaf`'s options.
    RoseLeaf { pairs: usize = 1, droop: f64 = 0.25 }
}

/// A rose's (or a strawberry's, a tomato's) compound leaf: `pairs` pairs of broad leaflets and one at the tip, cut
/// from one outline (`cut_leaf`), `size` long.
pub fn rose_leaf(
    name: impl AsRef<str>,
    base: impl Into<V3>,
    yaw: f64,
    tilt: f64,
    size: f64,
    material: Mat,
    o: RoseLeaf,
) -> Obj {
    let edge = pinnate(
        o.pairs,
        size * 0.75,
        "oval",
        Pinnate {
            lean: 0.75,
            fill: 0.85,
            start: 0.1,
            round_tip: true,
            ..Pinnate::default()
        },
    );
    cut_leaf(
        name,
        base,
        yaw,
        tilt,
        size,
        &edge,
        material,
        CutLeaf {
            droop: o.droop,
            fold: 0.2,
            ribs: 3,
            thick: 0.003,
            ..CutLeaf::default()
        },
    )
}
