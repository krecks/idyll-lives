//! Drawing each face in from its edges.

use super::*;

options! {
    /// The options of inset_individual, Blender's own defaults: thickness is how far in from its edges each face
    /// is drawn, depth how far along its normal; use_even_offset pushes a corner further in the sharper it is,
    /// so the strip is as wide all round; use_relative_offset scales both by the lengths of the edges at a corner.
    InsetIndividual { thickness: f64 = 0.0, depth: f64 = 0.0, use_even_offset: bool = false, use_relative_offset: bool = false }
}

/// bmesh.ops.inset_individual(bm, faces=all, thickness, depth, use_even_offset, use_relative_offset) on the mesh of
/// obj: each face drawn in from its own edges and moved along its normal, joined to where its edges were by a strip
/// of quads (a mosaic's tiles, each proud of the grout between them). Every face keeps its place and takes new
/// points, in its corners' order; the old points stay as the strips' outer side; the strips come after the faces,
/// each face's in its edges' order.
pub fn inset_individual(obj: Obj, o: InsetIndividual) -> Obj {
    let (thickness, depth) = (o.thickness as f32, o.depth as f32);
    obj.edit(|part| {
        let mesh = &part.mesh;
        let mut out = Mesh {
            verts: mesh.verts.clone(),
            mats: mesh.mats.clone(),
            sharp: mesh.sharp.clone(),
            ..Mesh::default()
        };
        let mut strips: Vec<(Vec<u32>, u16, bool)> = Vec::new();
        for (f, face) in mesh.faces.iter().enumerate() {
            let c = mesh.corners(f);
            let n = c.len();
            let p: Vec<V3> = c.iter().map(|&v| mesh.verts[v as usize]).collect();
            let no = mesh.face_normal(f);
            // each edge's way into the face, and its length
            let edge_nors: Vec<V3> = (0..n)
                .map(|i| normalize_v3((p[i] - p[(i + 1) % n]).cross(no)).0)
                .collect();
            let length = |i: usize| crate::math::len_v3(p[(i + 1) % n] - p[i]);
            let mut length_prev = if depth != 0.0 { length(n - 1) } else { 0.0 };
            let first = out.verts.len() as u32;
            for i in 0..n {
                let (eno_prev, eno_next) =
                    (edge_nors[(if i != 0 { i } else { n }) - 1], edge_nors[i]);
                let mut tvec = normalize_v3(eno_prev + eno_next).0;
                if o.use_even_offset {
                    // (Blender's shell_v3v3_mid_normalized_to_dist: further in the sharper the corner)
                    let (mid, len) = normalize_v3(eno_prev + eno_next);
                    let cos = if len != 0.0 {
                        dot_v3(eno_prev, mid).abs()
                    } else {
                        0.0
                    };
                    tvec = crate::math::mul_v3_fl(tvec, if cos < 1.0e-8 { 1.0 } else { 1.0 / cos });
                }
                if o.use_relative_offset {
                    tvec =
                        crate::math::mul_v3_fl(tvec, (length(i) + length((i + n - 1) % n)) / 2.0);
                }
                let mut co = p[i];
                co.x += tvec.x * thickness;
                co.y += tvec.y * thickness;
                co.z += tvec.z * thickness;
                if depth != 0.0 {
                    let e_length = length(i);
                    let fac = depth
                        * if o.use_relative_offset {
                            (length_prev + e_length) * 0.5
                        } else {
                            1.0
                        };
                    length_prev = e_length;
                    co.x += no.x * fac;
                    co.y += no.y * fac;
                    co.z += no.z * fac;
                }
                out.verts.push(co);
            }
            let inner: Vec<u32> = (0..n as u32).map(|i| first + i).collect();
            out.push_face(&inner, face.mat, face.smooth);
            for i in 0..n {
                let j = (i + 1) % n;
                strips.push((vec![c[i], c[j], inner[j], inner[i]], face.mat, face.smooth));
            }
        }
        for (corners, mat, smooth) in &strips {
            out.push_face(corners, *mat, *smooth);
        }
        part.mesh = out;
    });
    obj
}

/// inset_individual with its options by name, as the scripts gave them to the operator:
/// inset_individual!(obj, thickness = grout, depth = raised, use_even_offset = true).
#[macro_export]
macro_rules! inset_individual {
    ($obj:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        // (as Python: what is given by place first, then the options in the order written)
        let args = ($obj,);
        #[allow(unused_mut)]
        let mut o = $crate::edit::InsetIndividual::default();
        $(o.$k = $crate::Arg::arg($v);)*
        $crate::edit::inset_individual(args.0, o)
    }};
}
