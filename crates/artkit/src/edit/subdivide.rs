//! Splitting chosen edges and the faces along them.
//!
//! Split from `edit.rs` (see `mod.rs`).

use super::*;

options! {
    /// The options of subdivide_edges, Blender's own defaults: cuts is how many points each chosen edge takes;
    /// use_grid_fill fills a quad whose four edges are all chosen with a grid of quads.
    SubdivideEdges { cuts: usize = 0, use_grid_fill: bool = false }
}

/// The mesh's edges in the order bmesh has them for a mesh it built face by face (a primitive: a box, a
/// cylinder), each from its first vertex to its second: every face in turn gives its sides, from the one that
/// comes into its first corner and then as it runs them, an edge once. (A mesh from points and faces,
/// `forms::object`, has its edges in this order too while it is small, each from its lower vertex.)
pub fn edges(obj: Obj) -> Vec<(u32, u32)> {
    obj.read(|part| {
        let mut seen = std::collections::BTreeSet::new();
        let mut out = Vec::new();
        for f in 0..part.mesh.faces.len() {
            let c = part.mesh.corners(f);
            for i in 0..c.len() {
                let (a, b) = (c[(i + c.len() - 1) % c.len()], c[i]);
                if seen.insert(edge(a, b)) {
                    out.push((a, b));
                }
            }
        }
        out
    })
}

/// bmesh.ops.subdivide_edges(bm, edges, cuts, use_grid_fill) on the mesh of obj: each chosen edge takes `cuts`
/// points, evenly, from its first vertex to its second, and the faces are cut to them: a quad with two opposite
/// edges chosen into strips across, one with all four chosen into a grid of quads (with use_grid_fill; without,
/// it keeps its new points on its sides and is not cut, as does any face with one edge chosen). `edges` are
/// (first vertex, second vertex) as the mesh has them and are gone through in the mesh's own order, whatever
/// order a script lists them in: pass `edges(obj)`, picked from. The new points come after the old, the edges'
/// first and then each grid's; the new faces after the old.
///
/// The other ways Blender fills a face (two chosen edges that meet, three of a quad, a triangle's three) are not
/// built: no script uses them.
pub fn subdivide_edges(obj: Obj, edges: &[(u32, u32)], o: SubdivideEdges) -> Obj {
    obj.edit(|part| part.mesh = subdivided(&part.mesh, edges, &o));
    obj
}

/// What subdivide_edges does with a face.
enum Fill {
    /// A grid, read from the corner after this vertex.
    Grid(u32),
    Strips,
}

/// A mesh while subdivide_edges cuts it.
struct Subdivided {
    verts: Vec<V3>,
    /// The points the cuts made.
    made: Vec<bool>,
    faces: Vec<Cut>,
    sharp: std::collections::BTreeSet<(u32, u32)>,
}

impl Subdivided {
    /// bmesh's BM_edge_split from the edge's first vertex: a point `fac` of the way from `from` to `to`, in every
    /// face that runs the edge.
    fn split_edge(&mut self, from: u32, to: u32, fac: f32) -> u32 {
        let (a, b) = (self.verts[from as usize], self.verts[to as usize]);
        let v = self.verts.len() as u32;
        self.verts.push(V3::new(
            a.x + (b.x - a.x) * fac,
            a.y + (b.y - a.y) * fac,
            a.z + (b.z - a.z) * fac,
        ));
        self.made.push(true);
        for face in &mut self.faces {
            let c = &mut face.corners;
            let n = c.len();
            if let Some(i) = (0..n).find(|&i| edge(c[i], c[(i + 1) % n]) == edge(from, to)) {
                c.insert(i + 1, v);
            }
        }
        if self.sharp.remove(&edge(from, to)) {
            self.sharp.insert(edge(from, v));
            self.sharp.insert(edge(v, to));
        }
        v
    }

    /// `cuts` points along an edge, each split off what is left of it (bm_subdivide_multicut).
    fn cut_edge(&mut self, from: u32, to: u32, cuts: usize) -> Vec<u32> {
        let mut from = from;
        (0..cuts)
            .map(|i| {
                from = self.split_edge(from, to, 1.0 / (cuts + 1 - i) as f32);
                from
            })
            .collect()
    }

    /// bmesh's BM_face_split of face f from the corner at vertex a to the one at b: f keeps its place.
    fn split_face(&mut self, f: usize, a: u32, b: u32) {
        let c = &self.faces[f].corners;
        let at = |v: u32| c.iter().position(|&x| x == v).unwrap();
        let (kept, other) = split_face(c, at(a), at(b));
        let new = Cut {
            corners: other,
            ..self.faces[f].clone()
        };
        self.faces[f].corners = kept;
        self.faces.push(new);
    }

    /// connect_smallest_face: the face with both vertices (the one of fewest corners) split between them.
    fn connect(&mut self, a: u32, b: u32) {
        let f = (0..self.faces.len())
            .filter(|&f| self.faces[f].corners.contains(&a) && self.faces[f].corners.contains(&b))
            .min_by_key(|&f| self.faces[f].corners.len())
            .expect("subdivide_edges: two points of a grid share no face");
        self.split_face(f, a, b);
    }

    /// quad_4edge_subdivide: the face's corners read from the first new point after `start`; lines from its
    /// first side to its third, each cut from the points of one side to those of the side across and given
    /// `n` points; then every line joined to the one before at each of them.
    fn grid(&mut self, f: usize, start: u32, n: usize) {
        let c = &self.faces[f].corners;
        let a = c.iter().position(|&x| x == start).unwrap() + 1;
        let verts: Vec<u32> = (0..c.len()).map(|b| c[(b + a) % c.len()]).collect();
        let s = n + 2;
        let mut lines = vec![0u32; s * s];
        for i in 0..s {
            lines[i] = verts[n * 3 + 2 + (n + 1 - i)];
            lines[(s - 1) * s + i] = verts[n + i];
        }
        for i in 0..n {
            let (a, b) = (verts[i], verts[n + 1 + n + 1 + (n - i - 1)]);
            self.connect(a, b);
            lines[(i + 1) * s] = a;
            lines[(i + 1) * s + s - 1] = b;
            for (k, v) in self.cut_edge(a, b, n).into_iter().enumerate() {
                lines[(i + 1) * s + k + 1] = v;
            }
        }
        for i in 1..n + 2 {
            for j in 1..=n {
                self.connect(lines[i * s + j], lines[(i - 1) * s + j]);
            }
        }
    }

    /// A face with two chosen edges and no pattern of its own: the points of the one joined to those of the
    /// other, the first of one to the last of the other.
    fn strips(&mut self, f: usize, n: usize) {
        let c = self.faces[f].corners.clone();
        let len = c.len();
        let made = |i: usize| self.made[c[i % len] as usize];
        // where a run of new points starts
        let starts = |i: usize| !made(i + len - 1) && made(i);
        let mut a = (0..len).find(|&i| starts(i)).unwrap();
        let mut b = (a + n + 1) % len;
        if !made(b) {
            b = (0..len)
                .map(|j| (j + a + n + 1) % len)
                .find(|&b| starts(b))
                .unwrap();
        }
        b += n - 1;
        assert!(b < len, "subdivide_edges: a face's two chosen edges meet");
        let mut pairs = Vec::new();
        for _ in 0..n {
            // (not where another face has both points too: Blender leaves that pair alone)
            let shared = (0..self.faces.len()).any(|g| {
                g != f
                    && self.faces[g].corners.contains(&c[a])
                    && self.faces[g].corners.contains(&c[b])
            });
            if !shared {
                pairs.push((c[a], c[b]));
            }
            b = (b + len - 1) % len;
            a = (a + 1) % len;
        }
        for (a, b) in pairs {
            self.split_face(f, a, b);
        }
    }
}

fn subdivided(mesh: &Mesh, edges: &[(u32, u32)], o: &SubdivideEdges) -> Mesh {
    assert!(
        mesh.normals.is_none()
            && mesh.stored.is_none()
            && mesh.uvs.is_none()
            && mesh.ao.is_none()
            && mesh.card_shade.is_none(),
        "subdivide_edges: a mesh with normals, UVs or shading of its own is not built"
    );
    let n = o.cuts;
    if n == 0 || edges.is_empty() {
        return mesh.clone();
    }
    let chosen: std::collections::BTreeSet<(u32, u32)> =
        edges.iter().map(|&(a, b)| edge(a, b)).collect();
    // what is to be done with each face, found on the mesh as it comes
    let mut fills: Vec<(usize, Fill)> = Vec::new();
    for f in 0..mesh.faces.len() {
        let c = mesh.corners(f);
        let on: Vec<bool> = (0..c.len())
            .map(|i| chosen.contains(&edge(c[i], c[(i + 1) % c.len()])))
            .collect();
        match (c.len(), on.iter().filter(|&&x| x).count()) {
            (_, 0 | 1) => {}
            (4, 4) if o.use_grid_fill => fills.push((f, Fill::Grid(c[0]))),
            (4, 4) => {}
            (4, 2) if on[0] == on[2] => fills.push((f, Fill::Strips)),
            (len, sel) => panic!(
                "subdivide_edges: a face of {len} corners with {sel} edges chosen is not built"
            ),
        }
    }
    let mut m = Subdivided {
        verts: mesh.verts.clone(),
        made: vec![false; mesh.verts.len()],
        faces: (0..mesh.faces.len())
            .map(|f| Cut {
                corners: mesh.corners(f).to_vec(),
                mat: mesh.faces[f].mat,
                smooth: mesh.faces[f].smooth,
                normal: V3::ZERO,
            })
            .collect(),
        sharp: mesh.sharp.clone(),
    };
    for &(a, b) in edges {
        m.cut_edge(a, b, n);
    }
    // (Blender keeps the faces on a stack: the last found is cut first)
    for (f, fill) in fills.into_iter().rev() {
        match fill {
            Fill::Grid(start) => m.grid(f, start, n),
            Fill::Strips => m.strips(f, n),
        }
    }
    let mut out = Mesh {
        verts: m.verts,
        sharp: m.sharp,
        mats: mesh.mats.clone(),
        ..Mesh::default()
    };
    for face in &m.faces {
        out.push_face(&face.corners, face.mat, face.smooth);
    }
    out
}

/// subdivide_edges with its options by name: subdivide_edges!(obj, &edges, cuts = 5, use_grid_fill = true).
#[macro_export]
macro_rules! subdivide_edges {
    ($obj:expr, $edges:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        // (as Python: what is given by place first, then the options in the order written)
        let args = ($obj, $edges);
        #[allow(unused_mut)]
        let mut o = $crate::edit::SubdivideEdges::default();
        $(o.$k = $crate::Arg::arg($v);)*
        $crate::edit::subdivide_edges(args.0, args.1, o)
    }};
}
