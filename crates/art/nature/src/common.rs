//! What more than one family of the group builds from: the rock's cuts and stones (the garden decor's too) and
//! the part made from points (the ground cover's, the garden things' ponds).

use artkit::edit::{NgonMethod, QuadMethod, holes_fill, triangulate};
use artkit::materials::Mat;
use artkit::math::{M4, V3, v3};
use artkit::mesh::Mesh;
use artkit::noise::noise;
use artkit::normals::custom_from_verts;
use artkit::scene::{self, Obj};
use artkit::{bisect_plane, boulder};

/// A part from points (Blender metres) and faces, normals outward. `sky` bends every face's normal that far
/// towards straight up (normals of its own: the shape stays), as a lawn's grass is shaded, so near-upright blades
/// take the light the ground round them does rather than drawing dark from above. `recalc` false keeps the faces
/// wound as given (a ribbon's two sides, which have no outside to turn to).
pub fn mesh(
    name: impl AsRef<str>,
    verts: Vec<V3>,
    faces: &[Vec<u32>],
    material: Mat,
    smooth: bool,
    sky: f64,
    recalc: bool,
) -> Obj {
    let mut mesh = Mesh::from_pydata(verts, faces);
    if recalc {
        mesh.recalc_normals();
    }
    mesh.set_material(Some(material));
    for f in &mut mesh.faces {
        f.smooth = smooth;
    }
    if sky != 0.0 {
        mesh.normals = Some(sky_normals(&mesh, sky));
    }
    scene::add(name.as_ref(), mesh)
}

/// A normal a corner for a mesh of flat faces, each face's bent `sky` towards straight up, as Blender keeps the
/// normals a mesh is given for its corners.
fn sky_normals(mesh: &Mesh, sky: f64) -> Vec<V3> {
    let up = v3(0, 0, 1);
    // each face on points of its own: a corner of a flat face is kept in that face's frame alone
    let mut apart = Mesh {
        verts: mesh.loops.iter().map(|&v| mesh.verts[v as usize]).collect(),
        ..Mesh::default()
    };
    let mut given = Vec::with_capacity(mesh.loops.len());
    for (f, face) in mesh.faces.iter().enumerate() {
        assert!(!face.smooth, "normals bent to the sky are a flat face's");
        let corners: Vec<u32> = (face.start..face.start + face.len).collect();
        apart.push_face(&corners, face.mat, false);
        let normal = mesh.face_normal(f);
        // (Never past the face's side: the game winds each triangle by its normals, models.ts `pushTri`.)
        let k = if normal.z >= 0.0 {
            sky
        } else {
            sky.min(0.7 / -(normal.z as f64))
        };
        let n = (normal + up * k).normalized();
        given.extend(std::iter::repeat_n(n, face.len as usize));
    }
    custom_from_verts(&apart, &given)
}

/// Cuts `obj` (in place) by the world's plane through `co` facing `no`, dropping what lies in front of it and
/// closing the cut with a face of its own: the ground. The cut face is drawn flat. What is left is all triangles.
pub fn cut(obj: Obj, co: impl Into<V3>, no: impl Into<V3>) -> Obj {
    cut_in(obj, obj.matrix_world(), co.into(), no.into())
}

/// `cut` by a plane in the part's own frame, wherever it stands: a fracture across a stone.
pub fn cut_own(obj: Obj, co: impl Into<V3>, no: impl Into<V3>) -> Obj {
    cut_in(obj, M4::IDENTITY, co.into(), no.into())
}

/// `cut` by the plane through `co` facing `no` in the frame `world` takes the part's points into.
fn cut_in(obj: Obj, world: M4, co: V3, no: V3) -> Obj {
    obj.edit(|o| o.mesh.transform(&world));
    let edges = bisect_plane!(obj, co, no, clear_outer = true);
    if !edges.is_empty() {
        let made = holes_fill(obj, &edges);
        obj.edit(|o| {
            for f in made {
                o.mesh.faces[f].smooth = false;
            }
        });
    }
    triangulate(obj, QuadMethod::Beauty, NgonMethod::Beauty);
    obj.edit(|o| o.mesh.transform(&world.inverted().unwrap_or(M4::IDENTITY)));
    obj
}

/// Roughens a boulder's surface with finer noise than `boulder`'s (its own metres, before its scale).
pub fn lump(obj: Obj, amount: f64, scale: f64, seed: f64) {
    obj.edit(|o| {
        for v in &mut o.mesh.verts {
            let p = *v * scale + v3(seed, -seed, seed * 0.5);
            *v *= 1.0 + amount * noise(p);
        }
    });
}

/// A boulder `top` high (Blender z), roughened (`rough`: its noise and how lumpy it is; `grain`: and pitted finer
/// still, metres), with `fractures` (direction, depth: how far in from its outside, metres) cut flat across it, then
/// cut off flat at `ground` (Blender z), where it goes into the ground (the garden decor's stones too).
#[allow(clippy::too_many_arguments)]
pub fn stone(
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
    rough: (f64, f64),
    ground: f64,
) -> Obj {
    let o = boulder!(
        name,
        radius,
        at = at,
        material = material,
        scale = (1, 1, 1),
        seed = seed,
        noise = rough.0,
        subdivisions = subdivisions
    );
    lump(o, rough.1, 4.0, seed);
    if grain != 0.0 {
        lump(o, grain, 14.0, seed + 5.1);
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
        cut_own(o, n * (reach - depth), n);
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
    cut(o, (0, 0, ground), (0, 0, -1))
}
