//! Fans: the corners of a vertex that shade as one.

use super::*;

/// The corners of one vertex that shade as one, in the order Blender walks them round the vertex.
pub(crate) struct Fan {
    pub(crate) corners: Vec<usize>,
    /// The fan's own normal: its faces' weighed by their angles at the vertex.
    pub(crate) normal: V3,
    /// The directions of the edges it crosses, from the one it starts at to the one it ends at.
    pub(crate) edges: Vec<V3>,
    /// A corner with a sharp edge either side: its frame is its own two sides'.
    pub(crate) single: bool,
}

/// The fans of `mesh`'s corners: round each vertex from a sharp edge to the next (or all the way round, if none
/// is sharp), started as Blender starts them.
pub(crate) fn fans(mesh: &Mesh, face_normals: &[V3]) -> Vec<Fan> {
    let nl = mesh.loops.len();
    let mut face_of = vec![0usize; nl];
    let mut on_edge: BTreeMap<(u32, u32), Vec<usize>> = BTreeMap::new();
    for f in 0..mesh.faces.len() {
        let face = mesh.faces[f];
        for i in 0..face.len {
            let l = (face.start + i) as usize;
            face_of[l] = f;
            let (a, b) = (
                mesh.loops[l],
                mesh.loops[(face.start + (i + 1) % face.len) as usize],
            );
            on_edge.entry(edge(a, b)).or_default().push(l);
        }
    }
    let next = |l: usize| {
        let face = mesh.faces[face_of[l]];
        (face.start + (l as u32 - face.start + 1) % face.len) as usize
    };
    let prev = |l: usize| {
        let face = mesh.faces[face_of[l]];
        (face.start + (l as u32 - face.start + face.len - 1) % face.len) as usize
    };
    // a corner's edge: from its vertex to the next corner's
    let edge_of = |l: usize| edge(mesh.loops[l], mesh.loops[next(l)]);
    let sharp = |e: (u32, u32)| {
        let on = &on_edge[&e];
        on.len() != 2
            || mesh.sharp.contains(&e)
            || !mesh.faces[face_of[on[0]]].smooth
            || !mesh.faces[face_of[on[1]]].smooth
            || mesh.loops[on[0]] == mesh.loops[on[1]]
    };
    let dir = |pivot: u32, e: (u32, u32)| {
        let other = if e.0 == pivot { e.1 } else { e.0 };
        normalize_v3(mesh.verts[other as usize] - mesh.verts[pivot as usize]).0
    };
    // from the corner whose edge comes into the vertex (`curr`) and the vertex's own corner, to the same in the
    // next face round
    let step = |curr: usize, pivot: u32| -> (usize, usize) {
        let on = &on_edge[&edge_of(curr)];
        let other = if on[0] == curr { on[1] } else { on[0] };
        let (orig, across) = (mesh.loops[curr], mesh.loops[other]);
        if (orig == across && orig == pivot) || (orig != across && orig != pivot) {
            (prev(other), other)
        } else {
            (next(other), next(other))
        }
    };
    let mut skip = vec![false; nl];
    let mut out = Vec::new();
    for l in 0..nl {
        let (e_curr, e_prev) = (edge_of(l), edge_of(prev(l)));
        let pivot = mesh.loops[l];
        if !sharp(e_curr) {
            // all smooth round the vertex? then the fan starts at its first corner; else at its sharp edge
            if skip[l] || sharp(e_prev) {
                continue;
            }
            skip[l] = true;
            let mut curr = prev(l);
            let whole = loop {
                let (across, vert) = step(curr, pivot);
                curr = across;
                if sharp(edge_of(curr)) {
                    break false;
                }
                if skip[vert] {
                    break vert == l;
                }
                skip[vert] = true;
            };
            if !whole {
                continue;
            }
        }
        if sharp(e_curr) && sharp(e_prev) {
            out.push(Fan {
                corners: vec![l],
                normal: face_normals[face_of[l]],
                edges: vec![dir(pivot, e_curr), dir(pivot, e_prev)],
                single: true,
            });
            continue;
        }
        let vec_org = dir(pivot, e_curr);
        let mut fan = Fan {
            corners: Vec::new(),
            normal: V3::ZERO,
            edges: vec![vec_org],
            single: false,
        };
        let (mut curr, mut vert) = (prev(l), l);
        let mut vec_prev = vec_org;
        loop {
            let e = edge_of(curr);
            let vec_curr = dir(pivot, e);
            let k = crate::normals::acos_approx(dot_v3(vec_curr, vec_prev));
            fan.normal = madd(fan.normal, face_normals[face_of[curr]], k);
            fan.corners.push(vert);
            if e != e_curr {
                fan.edges.push(vec_curr);
            }
            if sharp(e) || e == e_curr {
                if e == e_curr {
                    // (all the way round: it ends at the edge it started at)
                    fan.edges.push(vec_curr);
                }
                break;
            }
            vec_prev = vec_curr;
            (curr, vert) = step(curr, pivot);
            if fan.corners.len() > nl {
                break;
            }
        }
        let (unit, len) = normalize_v3(fan.normal);
        fan.normal = if len == 0.0 {
            face_normals[face_of[l]]
        } else {
            unit
        };
        out.push(fan);
    }
    out
}
