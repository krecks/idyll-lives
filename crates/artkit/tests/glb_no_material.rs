//! A part with no material, and a face on a slot the part has no material for, are written without a material
//! instead of being dropped. A glTF mesh needs a primitive (`mesh.primitives` has `minItems: 1`), so dropping the
//! faces makes the file invalid for a strict loader, and the part missing from the model with no error anywhere.

use artkit::glb;
use artkit::materials::{Finishes, Mat};
use artkit::math::v3;
use artkit::mesh::Mesh;
use artkit::scene;

/// A square in the XY plane, as `from_pydata` makes it: two triangles on material 0, which is no material at all
/// when the mesh has none.
fn square() -> Mesh {
    Mesh::from_pydata(
        vec![
            v3(0.0, 0.0, 0.0),
            v3(1.0, 0.0, 0.0),
            v3(1.0, 1.0, 0.0),
            v3(0.0, 1.0, 0.0),
        ],
        &[vec![0, 1, 2], vec![0, 2, 3]],
    )
}

#[test]
fn a_part_with_no_material_keeps_its_faces() {
    let o = scene::add("crate", square());
    let model = glb::read(&glb::write(&[o], false, &Finishes::new())).unwrap();
    let primitives = &model.meshes[0].primitives;
    assert_eq!(primitives.len(), 1, "the part is written");
    assert_eq!(
        primitives[0].indices.len(),
        6,
        "its two triangles are written"
    );
    assert_eq!(primitives[0].material, None, "with no material to give it");
    assert!(model.materials.is_empty(), "and none made up for it");
}

#[test]
fn a_face_on_a_slot_past_its_materials_is_written_without_one() {
    let mut mesh = square();
    mesh.set_material(Some(Mat::new("wood", "#8a7050", "matte")));
    mesh.faces[1].mat = 3;
    let o = scene::add("crate", mesh);
    let model = glb::read(&glb::write(&[o], false, &Finishes::new())).unwrap();
    let primitives = &model.meshes[0].primitives;
    assert_eq!(
        primitives.len(),
        2,
        "one primitive a material, one for the stray face"
    );
    assert_eq!(primitives[0].material, Some(0));
    assert_eq!(primitives[0].indices.len(), 3);
    assert_eq!(primitives[1].material, None);
    assert_eq!(primitives[1].indices.len(), 3);
}
