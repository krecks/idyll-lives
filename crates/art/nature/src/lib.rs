//! The game's item models, generated: trees, plants, the garden, the ground and the street. One folder a family, each
//! listed in `families()`.

use artkit::family::Family;

pub mod garden_decor;
pub mod garden_things;
pub mod ground;
pub mod plants;
pub mod street;
pub mod trees;

/// Lays the textured finishes on the parts whose names start with one of `names` a box a face (`uv::box_`, at the
/// join), not round their axis or along their path: a turned part tilted after it was turned, a bent spray or a
/// jittered stump, whose own UVs stretch the texture over 2x. A face's box never stretches it past 1.7x.
pub fn box_uvs(names: &[&str]) {
    for part in artkit::scene::meshes() {
        let name = part.name();
        if names.iter().any(|n| name.starts_with(n)) {
            part.edit(|p| p.uv = artkit::mesh::UvWay::Box);
        }
    }
}

/// Eases the edges over `shapes::SHARP` of the parts whose names start with one of `names` by a 4 mm chamfer
/// (`bevel::bevel_edges`, one strip: a cut stone's or a sawn board's arris, the play camera's pixel), none in a far
/// version.
pub fn chamfer(names: &[&str]) {
    if artkit::far::is_low() {
        return;
    }
    for part in artkit::scene::meshes() {
        let name = part.name();
        if names.iter().any(|n| name.starts_with(n)) {
            artkit::bevel::bevel_edges(
                part,
                0.004,
                1,
                artkit::shapes::SHARP,
                &artkit::bevel::EdgeOrder::Corners,
            );
        }
    }
}

/// Rounds the parts whose names start with one of `names` now, before they are joined, so their textured finish
/// is laid on the rounded mesh a box a face: a one-strip chamfer joined as asked has its UVs carried from the faces
/// it cut, and its strips' have no area (the texture smeared along them).
pub fn bevel_first(names: &[&str]) {
    for part in artkit::scene::meshes() {
        let name = part.name();
        if names.iter().any(|n| name.starts_with(n)) {
            artkit::bevel::apply(part);
        }
    }
}

/// Bakes the scale of the parts whose names start with one of `names` into their meshes now, as the join would,
/// so their textured finish is laid on the shape as drawn: a squashed boulder's UVs made before its scale stretch
/// the texture by as much as it was squashed.
pub fn bake_scale(names: &[&str]) {
    for part in artkit::scene::meshes() {
        let name = part.name();
        if names.iter().any(|n| name.starts_with(n)) {
            let scale = part.scale();
            part.edit(|p| p.mesh.transform(&artkit::math::M4::diagonal(scale)));
            part.set_scale((1.0, 1.0, 1.0));
        }
    }
}

/// The group's families by their names on the command line (`cargo art <name>`).
pub fn families() -> Vec<(&'static str, Family)> {
    vec![
        ("trees", trees::family()),
        ("plants", plants::family()),
        ("garden_things", garden_things::family()),
        ("garden_decor", garden_decor::family()),
        ("ground", ground::family()),
        ("street", street::family()),
    ]
}
