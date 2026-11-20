//! The texel rule over the built world.
//!
//! Split from `built.rs` (see `mod.rs`).

use super::*;

// ---- texels -------------------------------------------------------------------------------------------

/// The textured materials among `surfaces` as the texel rule takes them, each with its texture's size.
pub(crate) fn textures(surfaces: &Surfaces, pictures: &mut Pictures) -> Textures {
    surfaces
        .iter()
        .filter_map(|(k, s)| {
            let file = s.texture.as_ref()?;
            let texture = Texture {
                uv_scale: s.uv_scale,
                px: pictures.side(file),
            };
            Some((k.clone(), texture))
        })
        .collect()
}

/// A sample's parts on textured materials as the texel rule reads them: a part is the rule's material (what the
/// findings name), its material the rule's finish. A material projected from above has x and z for UVs.
pub(crate) fn textured(parts: &[Part], surfaces: &Surfaces) -> Vec<Mesh> {
    parts
        .iter()
        .filter(|p| {
            !p.shadow_only
                && surfaces
                    .get(&p.mesh.material)
                    .is_some_and(|s| s.texture.is_some())
        })
        .map(|p| {
            let planar = surfaces[&p.mesh.material].planar;
            Mesh {
                material: p.mesh.part.clone(),
                uvs: if planar {
                    p.mesh.positions.iter().map(|at| [at[0], at[2]]).collect()
                } else {
                    p.mesh.uvs.clone()
                },
                ..p.mesh.clone()
            }
        })
        .collect()
}
