//! The surface of a body: its positions, normals and indices, cached.
//!
//! Split from `characters/body.rs` (see `mod.rs`).

use super::*;

/// The body of a family at a stage: cage, level-1 mesh, LOD and the dense head samples.
pub struct Surface {
    pub p: [V3; NB],
    pub cage: Cage,
    pub ctrl: Vec<V3>,
    pub lod_pos: Vec<V3>,
    pub lod_dense: Vec<Dense>,
    pub lod_tags: TagArrays,
    pub lod_region: Vec<[f64; NR]>,
    pub lod_nor: Vec<V3>,
    pub pos: Vec<V3>,
    pub dense: Vec<Dense>,
    pub tags: TagArrays,
    pub region: Vec<[f64; NR]>,
    pub nor: Vec<V3>,
    /// Level 1 as it would be without the eye sockets (`nor` is its normals): what the lids are shaded as.
    pub unsocketed: Vec<V3>,
    pub head_samples: Vec<V3>,
    pub head_normals: Vec<V3>,
    /// Per sample, the head's share of it: 1 on the head, falling to 0 down the neck.
    pub head_share: Vec<f64>,
    /// The samples' triangles: each one's corners among the first samples (the vertices; a centre sample
    /// follows per triangle, in this order).
    pub head_tri_ix: Vec<[usize; 3]>,
    pub head_tris: Vec<[V3; 3]>,
    /// The same triangles without the eye sockets (where the closed lids lie, face.rs).
    pub bare_tris: Vec<[V3; 3]>,
    pub cranium: Vec<V3>,
    /// The close-up body (`hd.body`): the whole body at level 2. Positions (snapped,
    /// sockets carved), the same without the sockets, their smooth normals,
    /// and the weights, tags and regions subdivided to level 2 (`Topology::fine_m`).
    pub fine_pos: Vec<V3>,
    pub fine_bare: Vec<V3>,
    pub fine_nor: Vec<V3>,
    pub fine_dense: Vec<Dense>,
    pub fine_tags: TagArrays,
    pub fine_region: Vec<[f64; NR]>,
}

static SURFACES: Cache<(String, String), Surface> = Cache::new();

/// The body of `family` at `stage` (cached).
pub fn surface(family: &str, stage: &str) -> Arc<Surface> {
    SURFACES.get(key(family, stage), || {
        make_surface(family, stage, &Identity::NONE)
    })
}

/// The body with a face of its own (`id`; identity.rs): the same topology, another head. Not kept.
pub fn surface_of(family: &str, stage: &str, id: &Identity) -> Surface {
    make_surface(family, stage, id)
}

fn make_surface(family: &str, stage: &str, id: &Identity) -> Surface {
    let topo = topology();
    let (cage, p) = body_cage_of(family, stage, id);
    let arr = cage.arrays();
    assert_eq!(arr.t.len(), topo.n);
    // The smooth shape is interpolated at the cage's vertices and snapped onto between them; features
    // are offsets on the fitted cage, the eye sockets carved after (`snapped`).
    let base = topo.solve(&arr.t);
    let ctrl: Vec<V3> = base.iter().zip(&arr.d).map(|(c, d)| c + d).collect();
    let (onto, holes) = (
        onto_head(family, stage, id),
        sockets(family, stage, false, id),
    );
    let s = &topo.s;
    let pos = topo.snapped(false, &base, &ctrl, &onto, &holes);
    let region: Vec<[f64; NR]> = cage
        .region
        .iter()
        .map(|r| {
            let mut v = [0.0; NR];
            v[super::body::region(r)] = 1.0;
            v
        })
        .collect();
    let lod_pos: Vec<V3> = topo
        .a
        .apply(&ctrl)
        .into_iter()
        .map(sockets(family, stage, true, id))
        .collect();
    let tags = TagArrays::of(&arr.tags);
    let dense1 = rigid_fingertips(s.m.apply_n(&arr.dense), &pos, &p);
    let tags1 = tags.subdivided(&s.m);
    let region1 = s.m.apply_n(&region);
    // Head and neck, densely (for the head interface's sdf and surface_z).
    let fpos = topo.snapped(true, &base, &ctrl, &onto, &holes);
    // The close-up body (`hd.body`): level 2 as a mesh of its own.
    let fine_dense = rigid_fingertips(topo.fine_m.apply_n(&arr.dense), &fpos, &p);
    let fine_tags = tags.subdivided(&topo.fine_m);
    // Normals from the finer (level-2) limit surface: smooth shading on the coarse level 1 (whose
    // vertices are the first of level 2's), as it is without the eye sockets: the lids cover them
    // (face.rs), and the skin round a lid is shaded as the face it carries on.
    let bare = topo.snapped(true, &base, &ctrl, &onto, &|p| p);
    let bare_nor = smooth_normals(&bare, &triangles(&topo.s2.quads, &bare).concat());
    let nor = bare_nor[..pos.len()].to_vec();
    let lod_nor = bare_nor[..topo.n].to_vec();
    let unsocketed = bare[..pos.len()].to_vec();
    let freg = topo.fine_m.apply_n(&region);
    let (ri_head, ri_neck) = (super::body::region("head"), super::body::region("neck"));
    let hmask: Vec<bool> = freg.iter().map(|r| r[ri_head] + r[ri_neck] > 0.5).collect();
    let q = &topo.s2.quads;
    let qm: Vec<[usize; 4]> = q
        .iter()
        .filter(|q| q.iter().all(|&v| hmask[v]))
        .copied()
        .collect();
    let tri = triangles(&qm, &fpos);
    let nor2 = smooth_normals(&fpos, &triangles(q, &fpos).concat());
    let used = unique(&tri.concat());
    // Samples: the vertices and the triangles' centres (about 4 mm apart on an adult head).
    let mut head_samples: Vec<V3> = used.iter().map(|&i| fpos[i]).collect();
    let mut head_normals: Vec<V3> = used.iter().map(|&i| nor2[i]).collect();
    let share = |i: usize| freg[i][ri_head];
    let mut head_share: Vec<f64> = used.iter().map(|&i| share(i)).collect();
    for t in &tri {
        let cn = nor2[t[0]] + nor2[t[1]] + nor2[t[2]];
        head_normals.push(cn / cn.norm());
        head_samples.push((fpos[t[0]] + fpos[t[1]] + fpos[t[2]]) / 3.0);
        head_share.push((share(t[0]) + share(t[1]) + share(t[2])) / 3.0);
    }
    let head_tri_ix = tri
        .iter()
        .map(|t| t.map(|i| used.binary_search(&i).expect("sample")))
        .collect();
    let head_tris = tri
        .iter()
        .map(|t| [fpos[t[0]], fpos[t[1]], fpos[t[2]]])
        .collect();
    let bare_tris = tri
        .iter()
        .map(|t| [bare[t[0]], bare[t[1]], bare[t[2]]])
        .collect();
    let pr = proportions(family, stage);
    let hh = pr.hh * pr.h;
    let cranium = fpos
        .iter()
        .zip(&freg)
        .filter(|(p, r)| r[ri_head] > 0.99 && p.y > pr.h - hh + 0.6 * hh)
        .map(|(p, _)| *p)
        .collect();
    Surface {
        p,
        cage,
        ctrl,
        lod_pos,
        lod_dense: arr.dense,
        lod_tags: tags,
        lod_region: region,
        lod_nor,
        pos,
        dense: dense1,
        tags: tags1,
        region: region1,
        nor,
        unsocketed,
        head_samples,
        head_normals,
        head_share,
        head_tri_ix,
        head_tris,
        bare_tris,
        cranium,
        fine_pos: fpos,
        fine_bare: bare,
        fine_nor: bare_nor,
        fine_dense,
        fine_tags,
        fine_region: freg,
    }
}

static TRIS_HD: Cache<String, Vec<usize>> = Cache::new();

/// The close-up body's triangles (level 2), split as `tris` splits level 1's.
pub fn tris_hd(family: &str) -> Arc<Vec<usize>> {
    TRIS_HD.get(family.to_string(), || {
        triangles(&topology().s2.quads, &surface(family, BASE_STAGE).fine_pos).concat()
    })
}

static TRIS: Cache<String, Vec<usize>> = Cache::new();

/// Level-1 triangles: quads split along their shorter diagonal on the family's base body.
pub fn tris(family: &str) -> Arc<Vec<usize>> {
    TRIS.get(family.to_string(), || {
        triangles(&topology().s.quads, &surface(family, BASE_STAGE).pos).concat()
    })
}
