//! What the ground cover is built from: meshes made by hand, a blade and a stem.

use artkit::far::low;
use artkit::materials::Mat;
use artkit::math::{TAU, V3, cos, radians, sin, v3};
use artkit::mesh::Mesh;
use artkit::normals::custom_from_verts;
use artkit::scene::{self, Obj};

/// How far a blade's or leaf's normals bend to the sky (`mesh`).
pub const SKY: f64 = 1.5;

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

/// A grass blade (or a flower's leaf) from `base` (game x, depth) on the ground, `width` across there, leaning
/// `lean` degrees from upright towards `yaw` (degrees round from game +x) and bending `bend` more by its tip,
/// `length` long, with one joint (`joint`; without, a straight spike: a short blade). A flat ribbon drawn from
/// both sides: two faces back to back (each side its own vertices and normals), of which the camera only ever
/// draws the one turned to it, so a tuft costs the GPU what the old part's thin cones did (a closed blade of three
/// sides drew three times the triangles and measured 0.9 ms more over the village's lawns on WebGL). 6 triangles;
/// 2 in a far version (no joint). `fold`: folded along its midrib from the joint up, the midrib that share of its
/// width out of the ribbon (a leaf's channel): 12 triangles.
#[allow(clippy::too_many_arguments)]
pub fn blade(
    name: impl AsRef<str>,
    base: (f64, f64),
    yaw: f64,
    lean: f64,
    bend: f64,
    length: f64,
    width: f64,
    material: Mat,
    mid: f64,
    joint: bool,
    fold: f64,
) -> Obj {
    let h = v3(-cos(radians(yaw)), -sin(radians(yaw)), 0); // game (cos, sin) on the ground
    let up = v3(0, 0, 1);
    let across = up.cross(h).normalized();
    let heading = |a: f64| (h * sin(radians(a)) + up * cos(radians(a))).normalized();
    let p0 = v3(-base.0, -base.1, 0.0);
    let (d0, d1) = (heading(lean), heading(lean + bend));
    let p1 = p0 + d0 * (length * mid);
    let tip = p1 + d1 * (length * (1.0 - mid));
    let (side, front): (Vec<V3>, Vec<Vec<u32>>) = if low(true, false) && fold != 0.0 {
        let out = across.cross(d0).normalized() * (fold * width);
        (
            vec![
                p0 + across * (width / 2.0),
                p0,
                p0 - across * (width / 2.0),
                p1 + across * (width * 0.36),
                p1 + out,
                p1 - across * (width * 0.36),
                tip,
            ],
            vec![
                vec![0, 1, 4, 3],
                vec![1, 2, 5, 4],
                vec![3, 4, 6],
                vec![4, 5, 6],
            ],
        )
    } else if low(true, false) && joint {
        (
            vec![
                p0 + across * (width / 2.0),
                p0 - across * (width / 2.0),
                p1 - across * (width * 0.31),
                p1 + across * (width * 0.31),
                tip,
            ],
            vec![vec![0, 1, 2, 3], vec![3, 2, 4]],
        )
    } else {
        // (A far version goes straight to the same tip.)
        (
            vec![
                p0 + across * (width / 2.0),
                p0 - across * (width / 2.0),
                tip,
            ],
            vec![vec![0, 1, 2]],
        )
    };
    let n = side.len() as u32;
    let mut faces = front.clone();
    faces.extend(
        front
            .iter()
            .map(|f| f.iter().rev().map(|&i| n + i).collect::<Vec<u32>>()),
    );
    // Flat, its normals bent to the sky: upright, its faces drew dark against the lawn from above.
    let both = [side.clone(), side].concat();
    mesh(name, both, &faces, material, false, SKY, false)
}

// ---- the wildflowers -------------------------------------------------------------------------------------------

/// A stem from `base` to `top` (Blender points): a three-sided tube in two lengths, kinked a little at its
/// middle, open at both ends (in the lawn, and under its flower): 12 triangles; 6 in a far version.
pub fn stem(
    name: impl AsRef<str>,
    base: V3,
    top: V3,
    radius: f64,
    material: Mat,
    sway: f64,
) -> Obj {
    let axis = (top - base).normalized();
    let side = axis.cross(v3(0, 0, 1));
    let side = if side.length() > 1e-6 {
        side.normalized()
    } else {
        v3(1, 0, 0)
    };
    let other = axis.cross(side).normalized();
    let centres = if low(true, false) {
        vec![base, (base + top) / 2 + side * sway, top]
    } else {
        vec![base, top]
    };
    let mut verts = Vec::new();
    for &c in &centres {
        for j in 0..3 {
            let a = TAU * j as f64 / 3.0;
            verts.push(c + (side * cos(a) + other * sin(a)) * radius);
        }
    }
    let mut faces: Vec<Vec<u32>> = Vec::new();
    for i in 0..centres.len() as u32 - 1 {
        for j in 0..3 {
            let k = (j + 1) % 3;
            faces.push(vec![i * 3 + j, i * 3 + k, (i + 1) * 3 + k, (i + 1) * 3 + j]);
        }
    }
    mesh(name, verts, &faces, material, true, 0.0, true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use artkit::far::far_of;
    use artkit::materials::Mat;

    const LEAF: Mat = Mat::new("leaf", "#5E8436", "foliage");

    #[test]
    fn a_blade_is_a_ribbon_with_two_sides_and_a_far_one_a_triangle() {
        scene::reset();
        let near = blade(
            "near",
            (0.0, 0.0),
            30.0,
            20.0,
            20.0,
            0.14,
            0.04,
            LEAF,
            0.5,
            true,
            0.0,
        );
        let folded = blade(
            "fold",
            (0.0, 0.0),
            30.0,
            20.0,
            20.0,
            0.14,
            0.04,
            LEAF,
            0.5,
            true,
            0.22,
        );
        let far = far_of(|| {
            blade(
                "far",
                (0.0, 0.0),
                30.0,
                20.0,
                20.0,
                0.14,
                0.04,
                LEAF,
                0.5,
                true,
                0.22,
            )
        });
        let count = |o: Obj| o.read(|p| (p.mesh.verts.len(), p.mesh.faces.len()));
        assert_eq!(
            (count(near), count(folded), count(far)),
            ((10, 4), (14, 8), (6, 2))
        );
        // its tip where the far version's is
        let tip = |o: Obj, i: usize| o.read(|p| p.mesh.verts[i]);
        assert_eq!(tip(folded, 6), tip(far, 2));
    }

    #[test]
    fn a_blades_normals_bend_to_the_sky_on_both_sides() {
        scene::reset();
        let b = blade(
            "b",
            (0.0, 0.0),
            0.0,
            25.0,
            0.0,
            0.14,
            0.04,
            LEAF,
            0.5,
            false,
            0.0,
        );
        b.read(|p| {
            let normals = p.mesh.normals.as_ref().expect("normals of its own");
            let (front, back) = (p.mesh.face_normal(0), p.mesh.face_normal(1));
            assert!(front.dot(back) < -0.999);
            // both look more up than their faces do, the one facing down no further than its own side
            for (f, face) in [front, back].into_iter().enumerate() {
                let n = normals[3 * f];
                assert!(n.z > face.z + 0.2 && n.dot(face) > 0.0, "{n:?} {face:?}");
            }
        });
    }

    #[test]
    fn a_stem_is_kinked_near_and_straight_far() {
        scene::reset();
        let (base, top) = (v3(0, 0, 0), v3(0.01, 0, 0.2));
        let near = stem("near", base, top, 0.0045, LEAF, 0.012);
        let far = far_of(|| stem("far", base, top, 0.0045, LEAF, 0.012));
        assert_eq!(near.read(|p| p.mesh.faces.len()), 6);
        assert_eq!(far.read(|p| p.mesh.faces.len()), 3);
    }
}
