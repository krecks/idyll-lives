//! Blender's own tools as a family script uses them on a mesh by hand, against Blender 5.2
//! (tests/golden/api_edit.txt): the SOLIDIFY modifier (dining's curved panels, half each way and evenly thick;
//! the energetic pack's canopy, inwards; a box open at a corner, outwards) and bmesh's delete of vertices.

mod common;

use artkit::edit::delete_verts;
use artkit::materials::Mat;
use artkit::math::V3;
use artkit::mesh::Mesh;
use artkit::parts::join;
use artkit::place::GameFrame;
use artkit::scene::{self, Obj};
use artkit::shapes::{recalc_normals, shade};
use artkit::{cube, fsphere, solidify};
use common::{GoldenMesh, as_set, f64s, golden_meshes, points_off};

const BODY: Mat = Mat::new("body", "#F3F3F1", "gloss");
const GLASS: Mat = Mat::new("glass", "#DCE6EA", "gloss");

/// How far a solidified point may be from Blender's, metres. The shell's points are the surface's moved along the
/// vertices' normals by a share of the thickness, and two things come out a unit or two in the last place from
/// Blender's: that share where the shell is evenly thick (at about a quarter of the vertices; the same sums written
/// in C against the system's library give the kernel's bits, so what Blender's build does otherwise there is not
/// known), and a triangle's face normal, which Blender's mesh divides by its length and the kernel multiplies by
/// one over it (the canopy's pole). Under a twentieth of a micrometre either way.
const SHELL: f32 = 5e-8;

/// The part against a golden mesh: its points in order, to the bit (or within `reach` metres), its faces as a
/// set (each wound as Blender's), which are smooth, and which edges are sharp.
fn same_but(what: &str, obj: Obj, g: &GoldenMesh, reach: f32) {
    let mesh = obj.read(|o| o.mesh.clone());
    let (off, _) = points_off(&mesh.verts, &g.verts);
    let far = mesh
        .verts
        .iter()
        .zip(&g.verts)
        .flat_map(|(v, w)| (0..3).map(move |k| (v[k] - w[k]).abs()))
        .fold(0.0_f32, f32::max);
    println!(
        "{what}: {} points, {off} not Blender's to the bit, the farthest {far:e} m off",
        mesh.verts.len()
    );
    assert!(far <= reach, "{what}: {off} points off, by {far:e} m");
    let faces: Vec<Vec<u32>> = (0..mesh.faces.len())
        .map(|f| mesh.corners(f).to_vec())
        .collect();
    assert_eq!(as_set(&faces), as_set(&g.faces), "{what}: faces");
    // each face's shading and material, by its corners
    let key = |f: &[u32]| as_set(&[f.to_vec()]).remove(0);
    let want: std::collections::BTreeMap<Vec<u32>, (char, &str)> = g
        .faces
        .iter()
        .zip(g.lines["smooth"].chars())
        .zip(g.lines["mat"].split(' '))
        .map(|((f, s), m)| (key(f), (s, m)))
        .collect();
    for (f, face) in mesh.faces.iter().enumerate() {
        let (smooth, mat) = want[&key(mesh.corners(f))];
        assert_eq!(face.smooth, smooth == '1', "{what}: face {f} smooth");
        assert_eq!(face.mat.to_string(), mat, "{what}: face {f} material");
    }
    let sharp: Vec<String> = mesh.sharp.iter().map(|(a, b)| format!("{a}-{b}")).collect();
    let mut wanted: Vec<(u32, u32)> = g.lines["sharp"]
        .split(' ')
        .filter(|w| !w.is_empty())
        .map(|w| {
            let (a, b) = w.split_once('-').unwrap();
            (a.parse().unwrap(), b.parse().unwrap())
        })
        .collect();
    wanted.sort_unstable();
    let wanted: Vec<String> = wanted.iter().map(|(a, b)| format!("{a}-{b}")).collect();
    // (a flat mesh's sharp edges are not read: Blender keeps none on it, the kernel what it had)
    if mesh.faces.iter().any(|f| f.smooth) {
        assert_eq!(sharp, wanted, "{what}: sharp edges");
    }
}

fn same(what: &str, obj: Obj, g: &GoldenMesh) {
    same_but(what, obj, g, 0.0);
}

#[test]
fn a_surface_is_made_a_shell_as_blenders_solidify_makes_it() {
    let all = golden_meshes("api_edit.txt");
    let of = |what: &str, name: &str| -> &GoldenMesh {
        all.iter()
            .find(|m| m.what.starts_with(what) && m.name == name)
            .unwrap_or_else(|| panic!("no {what} {name}"))
    };
    // dining's panels: a grid of quads through the script's points, half the thickness each way
    for what in ["chairModern shell", "chairWindsor seat"] {
        scene::reset();
        let first = of(what, "grid");
        let word = |key: &str| first.lines[key].clone();
        let (rows, cols): (usize, usize) =
            (word("rows").parse().unwrap(), word("cols").parse().unwrap());
        let (t, angle) = (f64s(&[&word("t")])[0], f64s(&[&word("angle")])[0]);
        let grid = word("grid");
        let coords = f64s(&grid.split(' ').collect::<Vec<_>>());
        let vs: Vec<V3> = coords
            .chunks(3)
            .map(|c| V3::new(c[0] as f32, c[1] as f32, c[2] as f32))
            .collect();
        let mut faces = Vec::new();
        for i in 0..rows - 1 {
            for j in 0..cols - 1 {
                faces.push(vec![
                    (i * cols + j) as u32,
                    (i * cols + j + 1) as u32,
                    ((i + 1) * cols + j + 1) as u32,
                    ((i + 1) * cols + j) as u32,
                ]);
            }
        }
        let mut mesh = Mesh::from_pydata(vs, &faces);
        mesh.set_material(Some(BODY));
        let obj = scene::add("panel", mesh);
        same(&format!("{what} grid"), obj, first);
        solidify!(obj, t, offset = 0.0, use_even_offset = true);
        same_but(&format!("{what} solid"), obj, of(what, "solid"), SHELL);
        recalc_normals(obj); // the grid's winding may face either way: outwards
        shade(obj, true, angle);
        same_but(&format!("{what} final"), obj, of(what, "final"), SHELL);
    }
    // the pod's canopy: the lower half of a sphere deleted, a shell grown inwards (Blender's defaults), joined
    scene::reset();
    let frame = GameFrame::new(0.0);
    let (hw, hl, hh) = (0.36, 0.9, 0.34);
    let shell = fsphere!(
        frame,
        "canopy",
        1.0,
        (0, 0.5, 0.62),
        GLASS,
        scale = (hw, hl, hh),
        segments = 28,
        rings = 14
    );
    same("canopy sphere", shell, of("canopy", "sphere"));
    delete_verts(shell, |v| (v.z as f64) < -1e-4);
    same("canopy half", shell, of("canopy", "half"));
    solidify!(shell, 0.008);
    same_but(
        "canopy solid",
        join(&[shell], "canopy", None),
        of("canopy", "solid"),
        SHELL,
    );
    // a box open at a corner: a shell grown outwards, evenly thick
    scene::reset();
    let open = cube!("box", (0.4, 0.3, 0.2), material = BODY);
    delete_verts(open, |v| v.x > 0.0 && v.y > 0.0 && v.z > 0.0);
    same("box cornerless", open, of("a box", "cornerless"));
    solidify!(open, 0.02, offset = 1.0, use_even_offset = true);
    same_but("box solid", open, of("a box", "solid"), SHELL);
}
