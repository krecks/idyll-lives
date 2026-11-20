//! Carving a hole out of a face.

use super::*;
use crate::math::V3;
use crate::mesh::{Mesh, edge, normal_newell};
use std::collections::BTreeMap;

/// What a recess leaves of a body: `tool` (its points `pts` in the body's frame) pushed in along axis `k` from
/// the side `side` (-1: from below, +1: from above).
pub(super) fn carve(
    body: &Mesh,
    tool: &Mesh,
    pts: &[V3],
    k: usize,
    side: f32,
    lined: bool,
    name: &str,
) -> Mesh {
    let (a, b) = ((k + 1) % 3, (k + 2) % 3);
    let flat = |p: V3| -> P2 { [p[a] as f64, p[b] as f64] };
    // how deep a point lies: growing from the side the cutter reaches out of
    let depth = |p: V3| -> f64 { -(side as f64) * p[k] as f64 };

    // the cutter's outline seen along the axis: its points gathered into corners, the hull of those
    let mut reps: Vec<usize> = Vec::new();
    let mut cluster = vec![0usize; pts.len()];
    for (i, &p) in pts.iter().enumerate() {
        let q = flat(p);
        let found = reps.iter().position(|&r| {
            let other = flat(pts[r]);
            (other[0] - q[0]).abs() <= SNAP && (other[1] - q[1]).abs() <= SNAP
        });
        match found {
            Some(c) => {
                cluster[i] = c;
                // (the corner stands where its foremost point does: the cutter's edge the body's face cuts)
                if depth(p) < depth(pts[reps[c]]) {
                    reps[c] = i;
                }
            }
            None => {
                cluster[i] = reps.len();
                reps.push(i);
            }
        }
    }
    let mut order: Vec<usize> = (0..reps.len()).collect();
    order.sort_by(|&x, &y| {
        let (p, q) = (flat(pts[reps[x]]), flat(pts[reps[y]]));
        p[0].total_cmp(&q[0]).then(p[1].total_cmp(&q[1]))
    });
    let mut hull: Vec<usize> = Vec::new();
    for pass in 0..2 {
        let start = hull.len();
        let run: Vec<usize> = if pass == 0 {
            order.clone()
        } else {
            order.iter().rev().copied().collect()
        };
        for c in run {
            let p = flat(pts[reps[c]]);
            while hull.len() >= start + 2
                && turn(
                    flat(pts[reps[hull[hull.len() - 2]]]),
                    flat(pts[reps[hull[hull.len() - 1]]]),
                    p,
                ) <= 0.0
            {
                hull.pop();
            }
            hull.push(c);
        }
        hull.pop();
    }
    let m = hull.len();
    assert!(m >= 3, "{name}: the cutter has an outline");
    let q: Vec<P2> = hull.iter().map(|&c| flat(pts[reps[c]])).collect();
    let mut hull_at: Vec<Option<usize>> = vec![None; reps.len()];
    for (j, &c) in hull.iter().enumerate() {
        hull_at[c] = Some(j);
    }
    let (t0, t1) = pts
        .iter()
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(l, h), &p| {
            (l.min(depth(p)), h.max(depth(p)))
        });
    let (qlo, qhi) = q
        .iter()
        .fold(([f64::INFINITY; 2], [f64::NEG_INFINITY; 2]), |(l, h), p| {
            (
                [l[0].min(p[0]), l[1].min(p[1])],
                [h[0].max(p[0]), h[1].max(p[1])],
            )
        });

    let n_old = body.verts.len() as u32;
    let mut new_pts: Vec<V3> = Vec::new();
    let mut made: BTreeMap<Made, u32> = BTreeMap::new();
    // the new points on each of the body's edges, by how far along it from its lower-numbered end
    let mut on_edge: BTreeMap<(u32, u32), Vec<(f64, u32)>> = BTreeMap::new();
    // a new point's edge of the body, and how far along it
    let mut edge_of: BTreeMap<u32, (u32, u32, f64)> = BTreeMap::new();
    // the points along each side of the outline, on the faces the cutter enters by (0) and leaves by (1)
    let mut rim: [Vec<Vec<(f64, u32)>>; 2] = [vec![Vec::new(); m], vec![Vec::new(); m]];
    let mut corner: [Vec<Option<u32>>; 2] = [vec![None; m], vec![None; m]];
    // each cut face: what is left of it, and its corners seen along the axis (anticlockwise)
    let mut pieces: BTreeMap<usize, Vec<Vec<u32>>> = BTreeMap::new();
    let mut sheets: [Vec<(usize, Vec<P2>)>; 2] = [Vec::new(), Vec::new()];
    let mut by_face: Vec<Vec<[u32; 3]>> = vec![Vec::new(); body.faces.len()];
    for (f, t) in body.triangles() {
        by_face[f as usize].push(t);
    }
    // where the line along the axis through a point seen from the front meets a face: on the triangle Blender
    // cuts the face into that it passes through
    let lift = |f: usize, at: P2| -> f32 {
        let c = body.corners(f);
        let first = body.verts[c[0] as usize][k];
        if c.iter().all(|&v| body.verts[v as usize][k] == first) {
            return first;
        }
        let tris = &by_face[f];
        let holds = |t: &[u32; 3]| {
            let p = t.map(|i| flat(body.verts[c[i as usize] as usize]));
            let (d0, d1, d2) = (
                turn(p[0], p[1], at),
                turn(p[1], p[2], at),
                turn(p[2], p[0], at),
            );
            (d0 >= 0.0 && d1 >= 0.0 && d2 >= 0.0) || (d0 <= 0.0 && d1 <= 0.0 && d2 <= 0.0)
        };
        let t = tris.iter().find(|t| holds(t)).unwrap_or(&tris[0]);
        let p = t.map(|i| {
            let v = body.verts[c[i as usize] as usize];
            [v[a] as f64, v[b] as f64, v[k] as f64]
        });
        let (u, w) = (
            [p[1][0] - p[0][0], p[1][1] - p[0][1], p[1][2] - p[0][2]],
            [p[2][0] - p[0][0], p[2][1] - p[0][1], p[2][2] - p[0][2]],
        );
        let n = [
            u[1] * w[2] - u[2] * w[1],
            u[2] * w[0] - u[0] * w[2],
            u[0] * w[1] - u[1] * w[0],
        ];
        (p[0][2] - (n[0] * (at[0] - p[0][0]) + n[1] * (at[1] - p[0][1])) / n[2]) as f32
    };

    for f in 0..body.faces.len() {
        let ids: Vec<u32> = body.corners(f).to_vec();
        let mut p: Vec<P2> = ids.iter().map(|&v| flat(body.verts[v as usize])).collect();
        let twice = area2(&p);
        if twice.abs() < 1e-14 {
            continue; // (a face seen on edge has nothing cut out of it)
        }
        let (plo, phi) =
            p.iter()
                .fold(([f64::INFINITY; 2], [f64::NEG_INFINITY; 2]), |(l, h), c| {
                    (
                        [l[0].min(c[0]), l[1].min(c[1])],
                        [h[0].max(c[0]), h[1].max(c[1])],
                    )
                });
        if plo[0] >= qhi[0] || phi[0] <= qlo[0] || plo[1] >= qhi[1] || phi[1] <= qlo[1] {
            continue;
        }
        let (dmin, dmax) = ids
            .iter()
            .fold((f64::INFINITY, f64::NEG_INFINITY), |(l, h), &v| {
                let d = depth(body.verts[v as usize]);
                (l.min(d), h.max(d))
            });
        if dmin >= t1 || dmax <= t0 {
            continue; // (beyond the cutter's end)
        }
        // anticlockwise, as the outline is
        let mut ids = ids;
        let turned = twice < 0.0;
        if turned {
            p.reverse();
            ids.reverse();
        }
        let n = p.len();
        let convex = (0..n).all(|i| turn(p[i], p[(i + 1) % n], p[(i + 2) % n]) >= 0.0);
        // each of the face's corners against the outline's sides, and the outline's against the face's
        let sp: Vec<Vec<f64>> = (0..m)
            .map(|j| (0..n).map(|i| turn(q[j], q[(j + 1) % m], p[i])).collect())
            .collect();
        let sq: Vec<Vec<f64>> = (0..n)
            .map(|i| (0..m).map(|j| turn(p[i], p[(i + 1) % n], q[j])).collect())
            .collect();
        let mut meets: Vec<Meet> = Vec::new();
        for i in 0..n {
            if (0..m).all(|j| sp[j][i] >= 0.0) {
                meets.push(Meet {
                    at: p[i],
                    corner: Some(i),
                    crossing: None,
                    outline: (0..m).find(|&j| q[j] == p[i]),
                    on_face: vec![(i + n - 1) % n, i],
                    on_outline: (0..m).filter(|&j| sp[j][i] == 0.0).collect(),
                });
            }
        }
        for j in 0..m {
            if (0..n).all(|i| sq[i][j] >= 0.0) && !p.contains(&q[j]) {
                meets.push(Meet {
                    at: q[j],
                    corner: None,
                    crossing: None,
                    outline: Some(j),
                    on_face: (0..n).filter(|&i| sq[i][j] == 0.0).collect(),
                    on_outline: vec![(j + m - 1) % m, j],
                });
            }
        }
        for i in 0..n {
            let i1 = (i + 1) % n;
            for j in 0..m {
                let j1 = (j + 1) % m;
                if sp[j][i] * sp[j][i1] < 0.0 && sq[i][j] * sq[i][j1] < 0.0 {
                    let t = sp[j][i] / (sp[j][i] - sp[j][i1]);
                    meets.push(Meet {
                        at: [
                            p[i][0] + (p[i1][0] - p[i][0]) * t,
                            p[i][1] + (p[i1][1] - p[i][1]) * t,
                        ],
                        corner: None,
                        crossing: Some((i, j)),
                        outline: None,
                        on_face: vec![i],
                        on_outline: vec![j],
                    });
                }
            }
        }
        if meets.len() < 3 {
            continue;
        }
        let middle = meets.iter().fold([0.0; 2], |s, x| {
            [
                s[0] + x.at[0] / meets.len() as f64,
                s[1] + x.at[1] / meets.len() as f64,
            ]
        });
        meets.sort_by(|x, y| {
            let angle = |z: &Meet| (z.at[1] - middle[1]).atan2(z.at[0] - middle[0]);
            angle(x).total_cmp(&angle(y))
        });
        let inside: Vec<P2> = meets.iter().map(|x| x.at).collect();
        if area2(&inside) < 1e-12 {
            continue; // (the outline only touches the face)
        }
        assert!(
            convex,
            "{name}: the cutter opens a face that turns in on itself (face {f}): cut it before that face is"
        );
        assert!(
            dmax < t1 && dmin > t0,
            "{name}: the cutter's end lies in a face of the body (face {f})"
        );
        let entering = (side > 0.0) != turned;
        let kind = usize::from(!entering);

        // the points' numbers in the cut body
        let mut number = |key: Made, at: &mut dyn FnMut() -> V3| -> u32 {
            *made.entry(key).or_insert_with(|| {
                new_pts.push(at());
                n_old + new_pts.len() as u32 - 1
            })
        };
        let mut gid: Vec<u32> = Vec::with_capacity(meets.len());
        for x in &meets {
            let id = if let Some(i) = x.corner {
                ids[i]
            } else if let Some((i, j)) = x.crossing {
                let (v, w) = (ids[i], ids[(i + 1) % n]);
                let (lo, hi, slo, shi) = if v < w {
                    (v, w, sp[j][i], sp[j][(i + 1) % n])
                } else {
                    (w, v, sp[j][(i + 1) % n], sp[j][i])
                };
                let t = slo / (slo - shi);
                let id = number(Made::Crossing(lo, hi, j), &mut || {
                    let (from, to) = (body.verts[lo as usize], body.verts[hi as usize]);
                    let mut at = V3::ZERO;
                    for c in 0..3 {
                        at[c] = (from[c] as f64 + (to[c] as f64 - from[c] as f64) * t) as f32;
                    }
                    // (on a side of the outline that runs along an axis, the point is on it to the bit)
                    let j1 = (j + 1) % m;
                    if q[j][0] == q[j1][0] {
                        at[a] = q[j][0] as f32;
                    }
                    if q[j][1] == q[j1][1] {
                        at[b] = q[j][1] as f32;
                    }
                    at
                });
                let list = on_edge.entry((lo, hi)).or_default();
                if !list.iter().any(|e| e.1 == id) {
                    list.push((t, id));
                    edge_of.insert(id, (lo, hi, t));
                }
                id
            } else {
                let j = x.outline.unwrap();
                if let Some(&i) = x.on_face.first() {
                    let (v, w) = (ids[i], ids[(i + 1) % n]);
                    let (lo, hi) = (v.min(w), v.max(w));
                    let (from, to) = (body.verts[lo as usize], body.verts[hi as usize]);
                    let t = along(flat(from), flat(to), q[j]);
                    let id = number(Made::OnEdge(lo, hi, j), &mut || {
                        let mut at = V3::ZERO;
                        at[k] = (from[k] as f64 + (to[k] as f64 - from[k] as f64) * t) as f32;
                        (at[a], at[b]) = (q[j][0] as f32, q[j][1] as f32);
                        at
                    });
                    let list = on_edge.entry((lo, hi)).or_default();
                    if !list.iter().any(|e| e.1 == id) {
                        list.push((t, id));
                        edge_of.insert(id, (lo, hi, t));
                    }
                    id
                } else {
                    number(Made::InFace(f, j), &mut || {
                        let mut at = V3::ZERO;
                        at[k] = lift(f, q[j]);
                        (at[a], at[b]) = (q[j][0] as f32, q[j][1] as f32);
                        at
                    })
                }
            };
            gid.push(id);
            if let Some(j) = x.outline {
                corner[kind][j] = Some(id);
            }
            for &j in &x.on_outline {
                let t = match x.outline {
                    Some(c) if c == j => 0.0,
                    Some(c) if c == (j + 1) % m => 1.0,
                    _ => along(q[j], q[(j + 1) % m], x.at),
                };
                if !rim[kind][j].iter().any(|e| e.1 == id) {
                    rim[kind][j].push((t, id));
                }
            }
        }

        // what is left of the face: its own outline with the cut's points on it, and the cut's the other way
        // round; a stretch that is both is the opening reaching the face's side, and goes
        let mut next: BTreeMap<u32, Vec<u32>> = BTreeMap::new();
        let mut join = |from: u32, to: u32| {
            if let Some(back) = next.get_mut(&to)
                && let Some(i) = back.iter().position(|&v| v == from)
            {
                back.remove(i);
                return;
            }
            next.entry(from).or_default().push(to);
        };
        let mut place: BTreeMap<u32, P2> = BTreeMap::new();
        for (i, &v) in ids.iter().enumerate() {
            place.insert(v, p[i]);
        }
        for (x, &id) in meets.iter().zip(&gid) {
            place.insert(id, x.at);
        }
        for i in 0..n {
            let i1 = (i + 1) % n;
            let mut on: Vec<(f64, u32)> = meets
                .iter()
                .zip(&gid)
                .filter(|(x, _)| x.corner.is_none() && x.on_face.contains(&i))
                .map(|(x, &id)| (along(p[i], p[i1], x.at), id))
                .collect();
            on.sort_by(|x, y| x.0.total_cmp(&y.0));
            let mut from = ids[i];
            for (_, id) in on {
                join(from, id);
                from = id;
            }
            join(from, ids[i1]);
        }
        for i in 0..gid.len() {
            join(gid[(i + 1) % gid.len()], gid[i]);
        }
        let mut outer: Vec<Vec<u32>> = Vec::new();
        let mut holes: Vec<Vec<u32>> = Vec::new();
        while let Some(start) = next.iter().find(|(_, to)| !to.is_empty()).map(|(&v, _)| v) {
            let mut ring = Vec::new();
            let mut v = start;
            loop {
                let to = next.get_mut(&v).filter(|to| !to.is_empty());
                let Some(to) = to else {
                    panic!("{name}: the cut leaves face {f} open");
                };
                assert!(
                    to.len() == 1,
                    "{name}: the opening touches a side of face {f} at one point"
                );
                ring.push(v);
                v = to.pop().unwrap();
                if v == start {
                    break;
                }
            }
            let flat_ring: Vec<P2> = ring.iter().map(|v| place[v]).collect();
            if area2(&flat_ring) > 0.0 {
                outer.push(ring);
            } else {
                holes.push(ring);
            }
        }
        let mut left: Vec<Vec<u32>> = Vec::new();
        if holes.is_empty() {
            left = outer;
        } else {
            assert!(
                outer.len() == 1 && holes.len() == 1,
                "{name}: the opening lies whole in face {f}"
            );
            let (rest, beside) = round_hole(&outer[0], &holes[0], &place);
            left.push(rest);
            left.push(beside);
        }
        if turned {
            for ring in &mut left {
                ring.reverse();
            }
        }
        pieces.insert(f, left);
        sheets[kind].push((f, p));
    }
    assert!(
        !sheets[0].is_empty(),
        "{name}: the cutter passes no face of the body on the side it reaches out of"
    );
    let through = !sheets[1].is_empty();
    let point = |id: u32, new_pts: &[V3]| -> V3 {
        if id < n_old {
            body.verts[id as usize]
        } else {
            new_pts[(id - n_old) as usize]
        }
    };
    for kind in 0..2 {
        for (j, side_rim) in rim[kind].iter_mut().enumerate() {
            side_rim.sort_by(|x, y| x.0.total_cmp(&y.0));
            if kind == 0 || through {
                assert!(
                    corner[kind][j].is_some()
                        && corner[kind][(j + 1) % m].is_some()
                        && side_rim.first().map(|e| e.1) == corner[kind][j]
                        && side_rim.last().map(|e| e.1) == corner[kind][(j + 1) % m],
                    "{name}: the cutter's outline lies on the body's faces all round"
                );
            }
        }
    }

    // the cutter's points inside the body: behind the faces it enters by, before those it leaves by
    let surface = |kind: usize, i: usize, new_pts: &[V3]| -> Option<f64> {
        if let Some(j) = hull_at[cluster[i]] {
            return corner[kind][j].map(|id| depth(point(id, new_pts)));
        }
        let at = flat(pts[i]);
        sheets[kind].iter().find_map(|(f, p)| {
            let n = p.len();
            (0..n)
                .all(|c| turn(p[c], p[(c + 1) % n], at) >= 0.0)
                .then(|| {
                    let mut on = V3::ZERO;
                    on[k] = lift(*f, at);
                    depth(on)
                })
        })
    };
    let reach: Vec<(f64, Option<f64>)> = (0..pts.len())
        .map(|i| {
            let enter = surface(0, i, &new_pts).unwrap_or_else(|| {
                panic!("{name}: the cutter's outline lies on the body's faces all round")
            });
            (
                enter,
                if through {
                    surface(1, i, &new_pts)
                } else {
                    None
                },
            )
        })
        .collect();
    let kept: Vec<bool> = (0..pts.len())
        .map(|i| {
            let d = depth(pts[i]);
            d > reach[i].0 && reach[i].1.is_none_or(|leave| d < leave)
        })
        .collect();
    let mut tool_id: Vec<Option<u32>> = vec![None; pts.len()];
    for i in 0..pts.len() {
        if kept[i] {
            new_pts.push(pts[i]);
            tool_id[i] = Some(n_old + new_pts.len() as u32 - 1);
            made.insert(Made::Tool(i), n_old + new_pts.len() as u32 - 1);
        }
    }

    // the cut body: its faces in their order, each cut one's pieces in its place
    let mut verts = body.verts.clone();
    verts.extend(new_pts.iter().copied());
    let mut new = Mesh {
        verts,
        mats: body.mats.clone(),
        ..Mesh::default()
    };
    let old_normals = body.normals.as_ref();
    let mut normals: Vec<V3> = Vec::new();
    // What a bevel stored for the body's corners stays theirs, to be read on the cut mesh as Blender reads it (a
    // corner whose fan now runs on into the cut's smooth faces turns with them); the cut's own corners have
    // nothing stored: two zeros, their fan's own normal.
    let old_stored = body.stored.as_ref().filter(|_| old_normals.is_some());
    let mut stored: Vec<Option<[i16; 2]>> = Vec::new();
    for f in 0..body.faces.len() {
        let face = body.faces[f];
        let c = body.corners(f);
        let own = |v: u32| -> Option<V3> {
            let at = c.iter().position(|&x| x == v)?;
            old_normals.map(|n| n[face.start as usize + at])
        };
        // a point on one of the face's edges takes the nearer end's normal; one inside it, the face's
        let stored_at = |v: u32| -> Option<[i16; 2]> {
            match (c.iter().position(|&x| x == v), old_stored) {
                (Some(at), Some(s)) => s[face.start as usize + at],
                _ => Some([0, 0]),
            }
        };
        let normal_at = |v: u32| -> V3 {
            if let Some(n) = own(v) {
                return n;
            }
            match edge_of.get(&v) {
                Some(&(lo, hi, t)) if own(lo).is_some() && own(hi).is_some() => {
                    own(if t < 0.5 { lo } else { hi }).unwrap()
                }
                _ => body.face_normal(f),
            }
        };
        match pieces.get(&f) {
            Some(left) => {
                for ring in left {
                    new.push_face(ring, face.mat, face.smooth);
                    if old_normals.is_some() {
                        normals.extend(ring.iter().map(|&v| normal_at(v)));
                        stored.extend(ring.iter().map(|&v| stored_at(v)));
                    }
                }
            }
            None => {
                // (a face beside a cut one takes the points the cut put on the edge they share)
                let mut ring: Vec<u32> = Vec::with_capacity(c.len());
                for (i, &v) in c.iter().enumerate() {
                    let w = c[(i + 1) % c.len()];
                    ring.push(v);
                    if let Some(on) = on_edge.get(&edge(v, w)) {
                        let mut on = on.clone();
                        on.sort_by(|x, y| x.0.total_cmp(&y.0));
                        if v > w {
                            on.reverse();
                        }
                        ring.extend(on.iter().map(|e| e.1));
                    }
                }
                if old_normals.is_some() {
                    normals.extend(ring.iter().map(|&v| normal_at(v)));
                    stored.extend(ring.iter().map(|&v| stored_at(v)));
                }
                new.push_face(&ring, face.mat, face.smooth);
            }
        }
    }

    // the cutter's faces inside the body, turned inward: whole, or from the rim back
    let slot_of = |new: &mut Mesh, mat: u16| -> u16 {
        if lined && !tool.mats.is_empty() {
            new.slot(tool.mats[(mat as usize).min(tool.mats.len() - 1)])
        } else if (mat as usize) < body.mats.len() {
            mat
        } else {
            0
        }
    };
    let chain = |kind: usize, j: usize| -> Vec<u32> { rim[kind][j].iter().map(|e| e.1).collect() };
    for f in 0..tool.faces.len() {
        let face = tool.faces[f];
        let c = tool.corners(f);
        let tool_normal = {
            let p: Vec<V3> = c.iter().map(|&v| pts[v as usize]).collect();
            normal_newell(&p)
        };
        if c.iter().all(|&v| kept[v as usize]) {
            let ring: Vec<u32> = (0..c.len())
                .map(|i| tool_id[c[(c.len() - i) % c.len()] as usize].unwrap())
                .collect();
            let slot = slot_of(&mut new, face.mat);
            new.push_face(&ring, slot, face.smooth);
            if old_normals.is_some() {
                normals.extend((0..c.len()).map(|i| {
                    let at = face.start as usize + (c.len() - i) % c.len();
                    match &tool.normals {
                        Some(n) => V3::new(-n[at].x, -n[at].y, -n[at].z),
                        None => V3::new(-tool_normal.x, -tool_normal.y, -tool_normal.z),
                    }
                }));
                stored.extend((0..c.len()).map(|i| {
                    let at = face.start as usize + (c.len() - i) % c.len();
                    tool.stored.as_ref().map_or(Some([0, 0]), |s| s[at])
                }));
            }
            continue;
        }
        // a face along the axis, between two corners of the outline next to each other: a wall of the cut
        let ends: Vec<Option<usize>> = c.iter().map(|&v| hull_at[cluster[v as usize]]).collect();
        let Some(j0) = ends[0] else {
            assert!(
                !c.iter().any(|&v| kept[v as usize]),
                "{name}: the body's face cuts the cutter's end aslant"
            );
            continue;
        };
        let other = ends.iter().flatten().copied().find(|&j| j != j0);
        let (Some(j1), true) = (
            other,
            ends.iter()
                .all(|e| e.is_some_and(|j| j == j0 || Some(j) == other)),
        ) else {
            assert!(
                !c.iter().any(|&v| kept[v as usize]),
                "{name}: the body's face cuts the cutter's end aslant"
            );
            continue;
        };
        let j = if (j0 + 1) % m == j1 {
            j0
        } else if (j1 + 1) % m == j0 {
            j1
        } else {
            assert!(
                !c.iter().any(|&v| kept[v as usize]),
                "{name}: the body's face cuts the cutter's end aslant"
            );
            continue;
        };
        let at_end = |end: usize| -> Vec<u32> {
            let mut v: Vec<u32> = c
                .iter()
                .copied()
                .filter(|&v| hull_at[cluster[v as usize]] == Some(end))
                .collect();
            v.sort_by(|&x, &y| depth(pts[x as usize]).total_cmp(&depth(pts[y as usize])));
            v
        };
        let (near, far) = (at_end(j), at_end((j + 1) % m));
        let before = |end: &[u32]| {
            end.iter()
                .any(|&v| depth(pts[v as usize]) < reach[v as usize].0)
        };
        let beyond = |end: &[u32]| {
            end.iter().any(|&v| {
                reach[v as usize]
                    .1
                    .is_some_and(|leave| depth(pts[v as usize]) > leave)
            })
        };
        let (enters, leaves) = (before(&near) && before(&far), beyond(&near) && beyond(&far));
        let inside = |end: &[u32]| -> Vec<u32> {
            end.iter()
                .filter(|&&v| kept[v as usize])
                .map(|&v| tool_id[v as usize].unwrap())
                .collect()
        };
        let mut ring: Vec<u32> = Vec::new();
        if enters {
            ring.extend(chain(0, j));
        }
        ring.extend(inside(&far));
        if leaves {
            ring.extend(chain(1, j).into_iter().rev());
        }
        ring.extend(inside(&near).into_iter().rev());
        ring.dedup();
        if ring.len() < 3 || !(enters || leaves) {
            assert!(
                !c.iter().any(|&v| kept[v as usize]),
                "{name}: the body's face cuts the cutter's end aslant"
            );
            continue;
        }
        let p: Vec<V3> = ring.iter().map(|&v| new.verts[v as usize]).collect();
        let mut normal = normal_newell(&p);
        if normal.dot(tool_normal) > 0.0 {
            ring.reverse();
            normal = V3::new(-normal.x, -normal.y, -normal.z);
        }
        let slot = slot_of(&mut new, face.mat);
        new.push_face(&ring, slot, face.smooth);
        if old_normals.is_some() {
            normals.extend(ring.iter().map(|_| normal));
            stored.extend(ring.iter().map(|_| Some([0, 0])));
        }
    }

    // the sharp edges: the body's, in as many stretches as the cut left them in, and the cutter's inside it
    let mut present: std::collections::BTreeSet<(u32, u32)> = std::collections::BTreeSet::new();
    for f in 0..new.faces.len() {
        let c = new.corners(f);
        for (i, &v) in c.iter().enumerate() {
            present.insert(edge(v, c[(i + 1) % c.len()]));
        }
    }
    for &(v, w) in &body.sharp {
        let mut run = vec![v];
        if let Some(on) = on_edge.get(&(v, w)) {
            let mut on = on.clone();
            on.sort_by(|x, y| x.0.total_cmp(&y.0));
            run.extend(on.iter().map(|e| e.1));
        }
        run.push(w);
        for pair in run.windows(2) {
            if present.contains(&edge(pair[0], pair[1])) {
                new.sharp.insert(edge(pair[0], pair[1]));
            }
        }
    }
    for &(v, w) in &tool.sharp {
        let stands = |v: u32| -> Option<u32> {
            tool_id[v as usize].or_else(|| hull_at[cluster[v as usize]].and_then(|j| corner[0][j]))
        };
        if let (Some(x), Some(y), true) =
            (stands(v), stands(w), kept[v as usize] || kept[w as usize])
            && present.contains(&edge(x, y))
        {
            new.sharp.insert(edge(x, y));
        }
    }
    if old_normals.is_some() {
        new.normals = Some(normals);
        if old_stored.is_some() {
            new.stored = Some(stored);
        }
    }

    // the body's points inside the cutter are no face's any more
    // and the points are numbered as Blender's solver mostly leaves them: the body's and the cutter's by the
    // numbers they had, one of the body's before the cutter's of the same number, then the points the cut made
    let mut used = vec![false; new.verts.len()];
    for &v in &new.loops {
        used[v as usize] = true;
    }
    let mut renumber = vec![u32::MAX; new.verts.len()];
    let mut verts = Vec::with_capacity(new.verts.len());
    let mut take = |v: u32, verts: &mut Vec<V3>| {
        if used[v as usize] && renumber[v as usize] == u32::MAX {
            renumber[v as usize] = verts.len() as u32;
            verts.push(new.verts[v as usize]);
        }
    };
    for i in 0..(n_old as usize).max(pts.len()) {
        if i < n_old as usize {
            take(i as u32, &mut verts);
        }
        if let Some(id) = tool_id.get(i).copied().flatten() {
            take(id, &mut verts);
        }
    }
    for v in n_old..new.verts.len() as u32 {
        take(v, &mut verts);
    }
    new.verts = verts;
    for v in &mut new.loops {
        *v = renumber[*v as usize];
    }
    new.sharp = new
        .sharp
        .iter()
        .map(|&(v, w)| edge(renumber[v as usize], renumber[w as usize]))
        .collect();
    new
}
