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
