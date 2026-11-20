//! The bevel itself: the mesh with its edges cut back and rounded.
//!
//! Split from `bevel.rs` (see `mod.rs`).

use super::*;

/// `mesh` with its edges that turn more than `angle` degrees bevelled `width` wide in `segments` strips.
pub fn bevelled(
    mesh: &Mesh,
    width: f64,
    segments: usize,
    angle: f64,
    order: &EdgeOrder,
) -> Result<Bevelled, String> {
    let ns = segments.max(1);
    let normals = face_normals(mesh);
    let nf = mesh.faces.len();
    let mut leaving = BTreeMap::new();
    for f in 0..nf {
        let c = mesh.corners(f);
        for (i, &a) in c.iter().enumerate() {
            let b = c[(i + 1) % c.len()];
            if leaving.insert((a, b), (f as u32, i as u32)).is_some() {
                return Err(
                    "an edge with two faces running it the same way can't be bevelled".into(),
                );
            }
        }
    }
    // the edges in the order the mesh numbers them
    let mut rank: BTreeMap<(u32, u32), usize> = BTreeMap::new();
    // (and the way each runs)
    let mut runs: BTreeMap<(u32, u32), (u32, u32)> = BTreeMap::new();
    let mut number = |a: u32, b: u32, way: (u32, u32)| {
        let at = rank.len();
        rank.entry(edge(a, b)).or_insert(at);
        runs.entry(edge(a, b)).or_insert(way);
    };
    if let EdgeOrder::Listed(list) = order {
        list.iter().for_each(|&(a, b)| number(a, b, (a, b)));
    }
    for f in 0..nf {
        let c = mesh.corners(f);
        let n = c.len();
        match order {
            EdgeOrder::Faces => (0..n).for_each(|i| {
                let (a, b) = (c[(i + n - 1) % n], c[i]);
                number(a, b, (a, b))
            }),
            _ => (0..n).for_each(|i| {
                let (a, b) = (c[i], c[(i + 1) % n]);
                number(a, b, edge(a, b))
            }),
        }
    }
    // the edges to bevel: between two faces that turn more than the angle
    let limit = crate::math::radians(angle) as f32;
    let threshold = cosf(limit + 0.000_000_175);
    let mut bev: BTreeSet<(u32, u32)> = BTreeSet::new();
    for (&(a, b), &(f, _)) in &leaving {
        if a < b
            && let Some(&(g, _)) = leaving.get(&(b, a))
            && dot_v3(normals[f as usize], normals[g as usize]) < threshold
        {
            bev.insert((a, b));
        }
    }
    let offset = width as f32;
    let mut work = Work {
        mesh,
        vertex_normals: crate::normals::vertex_normals(mesh),
        normals,
        leaving,
        corners: BTreeMap::new(),
    };
    if bev.is_empty() || offset <= 0.0 {
        return Ok(Bevelled {
            mesh: mesh.clone(),
            kinds: vec![Kind::Kept; nf],
            source: (0..nf)
                .flat_map(|f| std::iter::repeat_n(f as u32, mesh.faces[f].len as usize))
                .collect(),
            picks: Vec::new(),
            segments,
            offset: 0.0,
            made_at: vec![u32::MAX; mesh.verts.len()],
            centres: BTreeMap::new(),
        });
    }
    // each bevelled vertex's edges, anticlockwise from the first bevelled one the mesh numbers
    let mut tagged: BTreeSet<u32> = BTreeSet::new();
    for &(a, b) in &bev {
        tagged.insert(a);
        tagged.insert(b);
    }
    for &v in &tagged {
        let first = bev
            .iter()
            .filter(|e| e.0 == v || e.1 == v)
            .min_by_key(|e| rank[e])
            .map(|e| if e.0 == v { e.1 } else { e.0 })
            .expect("a bevelled edge");
        let mut edges = Vec::new();
        let mut to = first;
        loop {
            let Some(&(f, i)) = work.leaving.get(&(v, to)) else {
                return Err("an open edge can't be bevelled".into());
            };
            edges.push(Half {
                to,
                bev: bev.contains(&edge(v, to)),
                fnext: f,
                offset_l: 0.0,
                offset_r: 0.0,
                left: 0,
                right: 0,
            });
            let c = mesh.corners(f as usize);
            to = c[(i as usize + c.len() - 1) % c.len()];
            if to == first {
                break;
            }
            if edges.len() > 4 * mesh.loops.len() {
                return Err("a vertex whose faces don't close round it can't be bevelled".into());
            }
        }
        if edges.iter().filter(|h| h.bev).count() < 2 {
            return Err(
                "a bevel that ends at a vertex no other bevelled edge reaches isn't built".into(),
            );
        }
        // Blender lists the vertex's edges from that one on through whichever of its two neighbours the mesh
        // numbers first, and turns the list over if that went clockwise, which leaves that edge last; the new
        // points start from the list's last edge if it is bevelled, from its first if not. So they start from
        // that edge where the list was turned over, and from the edge before it, if bevelled, where it was not
        let n = edges.len();
        let mut listed_from = 0;
        if n > 2 {
            let number = |h: &Half| rank[&edge(v, h.to)];
            if number(&edges[1]) > number(&edges[n - 1]) {
                // (turned over, its list starts at the edge after this one)
                listed_from = 1;
            } else if edges[n - 1].bev {
                edges.rotate_right(1);
                listed_from = 1;
            }
        }
        work.corners.insert(
            v,
            Corner {
                v,
                edges,
                listed_from,
                ..Corner::default()
            },
        );
    }
    // no further than the mesh has room for
    let mut limited = offset;
    for corner in work.corners.values() {
        for i in 0..corner.edges.len() {
            limited = limited.min(work.collide_offset(corner, i, offset));
        }
    }
    let factor = if limited < offset {
        limited / offset
    } else {
        1.0
    };
    let mut each = offset;
    if limited < offset {
        each *= factor;
    }
    let keys: Vec<u32> = work.corners.keys().copied().collect();
    for again in [false, true] {
        for &v in &keys {
            let mut corner = work.corners.remove(&v).expect("a corner");
            if !again {
                for h in &mut corner.edges {
                    (h.offset_l, h.offset_r) = if h.bev { (each, each) } else { (0.0, 0.0) };
                }
            }
            work.build_boundary(&mut corner, again);
            work.corners.insert(v, corner);
        }
        if !again {
            adjust_offsets(&mut work.corners);
        }
    }

    // ---- the new mesh ----
    let mut out = Mesh {
        mats: mesh.mats.clone(),
        ..Mesh::default()
    };
    let mut kept = vec![u32::MAX; mesh.verts.len()];
    for (i, &p) in mesh.verts.iter().enumerate() {
        if !tagged.contains(&(i as u32)) {
            kept[i] = out.verts.len() as u32;
            out.verts.push(p);
        }
    }
    let mut kinds = Vec::new();
    let mut source = Vec::new();
    let mut long: BTreeSet<(u32, u32)> = BTreeSet::new(); // (face, corner) whose side runs along a bevelled edge
    // (corners, each one's source face, material, the vertex of a corner's polygon)
    let mut corner_faces: Vec<CornerFace> = Vec::new();
    let mut picks = Vec::new();
    let mut grids: BTreeMap<u32, Numbers> = BTreeMap::new();
    let mut made_at = vec![u32::MAX; out.verts.len()];
    let mut centres: BTreeMap<u32, Centre> = BTreeMap::new();
    for &v in &keys {
        let corner = &work.corners[&v];
        let mut grid = Numbers::new();
        let n = corner.bounds.len();
        for (i, bound) in corner.bounds.iter().enumerate() {
            grid.insert((i, 0, 0), out.verts.len() as u32);
            out.verts.push(bound.co);
        }
        for i in 0..n {
            let next = grid[&((i + 1) % n, 0, 0)];
            grid.insert((i, 0, ns), next);
        }
        let vco = work.co(v);
        let mat = mesh.faces[corner.bounds[0].face as usize].mat;
        if ns > 1 && n == 2 {
            // two bevelled edges: one profile between the two points, in the plane they span with the vertex
            let (d1, d2) = (vco - corner.bounds[0].co, vco - corner.bounds[1].co);
            let (no, l1) = normalize_v3(d1.cross(d2));
            let plane = (l1 > EPSILON).then_some(no);
            let (a, b) = (
                work.profile_of(corner, 0, plane, ns),
                work.profile_of(corner, 1, plane, ns),
            );
            for k in 1..ns {
                let (p, q) = (a[k - 1], b[ns - k - 1]);
                let at = out.verts.len() as u32;
                out.verts.push(V3::new(
                    0.5 * (p.x + q.x),
                    0.5 * (p.y + q.y),
                    0.5 * (p.z + q.z),
                ));
                grid.insert((0, 0, k), at);
                grid.insert((1, 0, ns - k), at);
            }
        } else if ns > 1 {
            let patch = work.patch(corner, ns)?;
            let ns2 = ns / 2;
            for i in 0..n {
                for j in 0..=ns2 {
                    for k in 0..=ns {
                        if (j == 0 && (k == 0 || k == ns)) || canon(n, ns, i, j, k) != (i, j, k) {
                            continue;
                        }
                        grid.insert((i, j, k), out.verts.len() as u32);
                        out.verts.push(patch[&(i, j, k)]);
                    }
                }
            }
            // (each profile's far half is the next bound's near side)
            for i in 0..n {
                for k in 1..ns {
                    let at = grid[&canon(n, ns, i, 0, k)];
                    grid.insert((i, 0, k), at);
                }
            }
            let at = |i: usize, j: usize, k: usize| {
                grid.get(&(i, j, k))
                    .or_else(|| grid.get(&canon(n, ns, i, j, k)))
                    .copied()
                    .expect("a patch point")
            };
            let odd = ns % 2;
            for i in 0..n {
                let (f1, f2) = (corner.bounds[i].face, corner.bounds[(i + 1) % n].face);
                for j in 0..ns2 {
                    for k in 0..ns2 + odd {
                        let quad = vec![
                            at(i, j, k),
                            at(i, j, k + 1),
                            at(i, j + 1, k + 1),
                            at(i, j + 1, k),
                        ];
                        let f = if 2 * k + 1 < ns { f1 } else { f2 };
                        corner_faces.push((quad, vec![f; 4], mat, None, Some((v, i, j, k))));
                    }
                }
            }
            if odd == 1 {
                let ring: Vec<u32> = (0..n).map(|i| at(i, ns2, ns2)).collect();
                let from: Vec<u32> = (0..n).map(|i| corner.bounds[i].face).collect();
                corner_faces.push((ring, from, mat, Some(v), None));
            }
        } else if n >= 3 {
            let ring: Vec<u32> = (0..n).map(|i| grid[&(i, 0, 0)]).collect();
            let from: Vec<u32> = (0..n).map(|i| corner.bounds[i].face).collect();
            corner_faces.push((ring, from, mat, Some(v), None));
        }
        made_at.resize(out.verts.len(), v);
        let (halves, m) = (&corner.edges, corner.edges.len());
        centres.insert(
            v,
            (
                (0..m)
                    .map(|k| (corner.listed_from + k) % m)
                    .filter(|&i| halves[i].bev)
                    .map(|i| (halves[i].to, halves[(i + m - 1) % m].fnext, halves[i].fnext))
                    .collect(),
                (0..n).map(|i| grid[&(i, 0, 0)]).collect(),
            ),
        );
        grids.insert(v, grid);
    }
    // the faces left as they were
    for f in 0..nf {
        if mesh.corners(f).iter().all(|v| !tagged.contains(v)) {
            let c: Vec<u32> = mesh.corners(f).iter().map(|&v| kept[v as usize]).collect();
            out.push_face(&c, mesh.faces[f].mat, mesh.faces[f].smooth);
            kinds.push(Kind::Kept);
            source.extend(std::iter::repeat_n(f as u32, c.len()));
        }
    }
    // each new point's bevelled edge across, with the faces either side of it
    let across = |vert: u32| -> Vec<(u32, u32, u32)> {
        let corner = &work.corners[&vert];
        let (halves, n) = (&corner.edges, corner.edges.len());
        corner
            .bounds
            .iter()
            .map(|b| {
                let h = b.ebev;
                (halves[h].to, halves[(h + n - 1) % n].fnext, halves[h].fnext)
            })
            .collect()
    };
    for (c, from, mat, vert, patch) in corner_faces {
        if let Some(vert) = vert {
            let halves = &work.corners[&vert].edges;
            let n = halves.len();
            picks.push(Pick::Corner {
                at: out.loops.len(),
                vert,
                on: from.clone(),
                edges: (0..n)
                    .filter(|&i| halves[i].bev)
                    .map(|i| (halves[i].to, halves[(i + n - 1) % n].fnext, halves[i].fnext))
                    .collect(),
                across: if ns > 1 { across(vert) } else { Vec::new() },
            });
        }
        if let Some((vert, i, j, k)) = patch {
            picks.push(Pick::Patch {
                at: out.loops.len(),
                vert,
                place: (i, j, k),
                across: across(vert),
            });
        }
        out.push_face(&c, mat, true);
        kinds.push(Kind::Corner);
        source.extend(from);
    }
    // a strip along each bevelled edge, in the mesh's order of them
    let mut by_rank: Vec<(u32, u32)> = bev.iter().copied().collect();
    by_rank.sort_by_key(|e| rank[e]);
    for e in by_rank {
        let (a, b) = runs[&e];
        let (c1, c2) = (&work.corners[&a], &work.corners[&b]);
        let (h1, h2) = (&c1.edges[c1.half_to(b)], &c2.edges[c2.half_to(a)]);
        let (i1, i2) = (h1.left, h2.left);
        // (the faces before and after the edge round its first end: its left point is on the one before)
        let at = c1.half_to(b);
        let (f1, f2) = (
            c1.edges[(at + c1.edges.len() - 1) % c1.edges.len()].fnext,
            h1.fnext,
        );
        let mat = mesh.faces[c1.bounds[h1.left].face as usize].mat;
        let at1 = |k: usize| grids[&a][&(i1, 0, k)];
        let at2 = |k: usize| grids[&b][&(i2, 0, k)];
        for k in 1..=ns {
            let quad = [at1(k - 1), at2(ns - k + 1), at2(ns - k), at1(k)];
            let face = out.faces.len() as u32;
            picks.push(Pick::Strip {
                at: out.loops.len(),
                edge: (a, b),
                faces: (f1, f2),
                k,
            });
            out.push_face(&quad, mat, true);
            kinds.push(Kind::Strip);
            long.insert((face, 0));
            long.insert((face, 2));
            source.extend([if 2 * k <= ns { f1 } else { f2 }; 4]);
        }
    }
    // the mesh's faces round the bevelled vertices, each corner now the new point on it
    let mut done = vec![false; nf];
    // (in the order Blender rebuilds them: vertex by vertex, each vertex's faces as its edges are numbered, a
    // face from the edge it leaves the vertex along; the shading's rays meet the faces in this order)
    for &v in &keys {
        let corner = &work.corners[&v];
        let mut round: Vec<&Half> = corner.edges.iter().collect();
        round.sort_by_key(|h| rank[&edge(v, h.to)]);
        for h in round {
            let f = h.fnext as usize;
            if std::mem::replace(&mut done[f], true) {
                continue;
            }
            let c = mesh.corners(f);
            let mut new: Vec<u32> = Vec::with_capacity(c.len());
            for (i, &w) in c.iter().enumerate() {
                let p = match work.corners.get(&w) {
                    None => kept[w as usize],
                    Some(cw) => {
                        let leaves = &cw.edges[cw.half_to(c[(i + 1) % c.len()])];
                        grids[&w][&(leaves.right, 0, 0)]
                    }
                };
                if new.last() != Some(&p) {
                    new.push(p);
                }
            }
            if new.len() > 1 && new.first() == new.last() {
                new.pop();
            }
            if new.len() < 3 {
                continue;
            }
            source.extend(std::iter::repeat_n(f as u32, new.len()));
            out.push_face(&new, mesh.faces[f].mat, mesh.faces[f].smooth);
            kinds.push(Kind::Cut);
        }
    }
    harden(&mut out, &kinds, &long, mesh, &kept);
    Ok(Bevelled {
        mesh: out,
        kinds,
        source,
        picks,
        segments: ns,
        offset: each,
        made_at,
        centres,
    })
}
