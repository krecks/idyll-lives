//! The topology of the mesh: which vertices are which, and their tags.
//!
//! Split from `characters/body.rs` (see `mod.rs`).

use super::*;

/// The cage's faces (oriented), its subdivision and limit operators (shared by every body).
pub struct Topology {
    pub faces: Vec<Vec<usize>>,
    pub n: usize,
    pub s: Subdivision,
    /// Cage -> level 1 on the limit surface.
    pub body: Csr,
    /// Cage -> its own limit points, and its LU (factored once).
    pub a: Csr,
    pub(crate) lu: LU<f64, Dyn, Dyn>,
    /// Per level-1 and level-2 point, how far it is snapped onto the smooth head (`snapped`).
    pub(crate) snap1: Vec<f64>,
    pub(crate) snap2: Vec<f64>,
    pub lod_tris: Vec<[usize; 3]>,
    /// A second level, only to sample the head densely (head_frame).
    pub s2: Subdivision,
    pub fine: Csr,
    pub fine_m: Csr,
}

impl Topology {
    fn new(cage: &Cage) -> Self {
        let pos = &cage.p;
        let n = pos.len();
        let faces = orient(&cage.faces, pos);
        if let Err(e) = check_closed(&faces, Some(n)) {
            panic!("{e}");
        }
        // The LOD leaves the face's finer cage out (its vertices stay, unused).
        let lod_faces = orient(&cage.lod_faces, pos);
        if let Err(e) = check_closed(&lod_faces, None) {
            panic!("LOD: {e}");
        }
        let s = subdivision(&faces, n);
        let n1 = s.m.rows;
        let l1 = limit_matrix(&s.quads, n1);
        let body = l1.matmul(&s.m);
        let a = body.head(n);
        let lu = a.to_dense().lu();
        let q1: Vec<Vec<usize>> = s.quads.iter().map(|q| q.to_vec()).collect();
        let s2 = subdivision(&q1, n1);
        let l2 = limit_matrix(&s2.quads, s2.m.rows);
        let fine = l2.matmul(&s2.m).matmul(&s.m);
        let fine_m = s2.m.matmul(&s.m);
        // Snapped: points of the head's cage alone, fading out where the neck's share comes in.
        let share = |m: &Csr| -> Vec<f64> {
            m.data
                .iter()
                .map(|row| {
                    let head: f64 = row
                        .iter()
                        .filter(|&&(c, _)| cage.region[c] == "head")
                        .map(|&(_, v)| v)
                        .sum();
                    smoothstep(0.8, 1.0, head)
                })
                .collect()
        };
        let (snap1, snap2) = (share(&body), share(&fine));
        let lod_tris = polygon_triangles(&lod_faces);
        Topology {
            faces,
            n,
            s,
            body,
            a,
            lu,
            snap1,
            snap2,
            lod_tris,
            s2,
            fine,
            fine_m,
        }
    }

    /// Cage positions whose limit points are `t` (splu's solve, here a dense LU).
    pub(crate) fn solve(&self, t: &[V3]) -> Vec<V3> {
        let mut out = vec![V3::zeros(); t.len()];
        for k in 0..3 {
            let rhs = DVector::from_iterator(t.len(), t.iter().map(|p| p[k]));
            let x = self.lu.solve(&rhs).expect("singular cage");
            for (o, v) in out.iter_mut().zip(x.iter()) {
                o[k] = *v;
            }
        }
        out
    }

    /// Points of level 1 (`fine`: level 2) of the features' cage `ctrl`, with its smooth part (`base`,
    /// the cage fitted to the smooth shape) moved onto that shape (`onto`) on the head, then the eye
    /// sockets carved (`sockets`). The cage's limit passes through the smooth head only at its vertices:
    /// between them the face grid's uneven spacing and the ends of its partial lines (3-valent vertices,
    /// pentagons) dent it by up to 4 mm and turn its normals by up to 17 degrees, which showed as patches
    /// on the cheeks; snapped, the head is the smooth one plus the features.
    pub(crate) fn snapped(
        &self,
        fine: bool,
        base: &[V3],
        ctrl: &[V3],
        onto: &impl Fn(V3) -> V3,
        sockets: &impl Fn(V3) -> V3,
    ) -> Vec<V3> {
        let (m, w) = if fine {
            (&self.fine, &self.snap2)
        } else {
            (&self.body, &self.snap1)
        };
        m.apply(base)
            .iter()
            .zip(m.apply(ctrl))
            .zip(w)
            .map(|((b, p), &w)| sockets(if w > 0.0 { p + (onto(*b) - b) * w } else { p }))
            .collect()
    }
}

pub fn topology() -> &'static Topology {
    static TOPO: OnceLock<Topology> = OnceLock::new();
    TOPO.get_or_init(|| Topology::new(&body_cage("male", BASE_STAGE).0))
}

/// Tags per vertex, as separate arrays.
#[derive(Clone)]
pub struct TagArrays {
    pub arm: Vec<f64>,
    pub blush: Vec<f64>,
}

impl TagArrays {
    pub(crate) fn of(t: &[Tags]) -> Self {
        TagArrays {
            arm: t.iter().map(|t| t.arm).collect(),
            blush: t.iter().map(|t| t.blush).collect(),
        }
    }

    pub(crate) fn subdivided(&self, m: &Csr) -> Self {
        TagArrays {
            arm: m.apply1(&self.arm),
            blush: m.apply1(&self.blush),
        }
    }
}
