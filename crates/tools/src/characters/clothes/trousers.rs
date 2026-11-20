//! Trousers: the bottoms with legs.

use super::*;

/// The bottoms with legs (`contract::garments`): one shell, `bottom.trousers`, cut to each one's
/// waist and length by the shader (shorts and capri are the same mesh).
pub const LEGGED: [&str; 4] = [
    "bottom.trousers",
    "bottom.suit",
    "bottom.shorts",
    "bottom.capri",
];
/// The trousers' shell reaches this far (m) past the highest waist and the lowest hem of the
/// bottoms with legs: the shader's cut lies inside the mesh.
pub(super) const CUT_ROOM: f64 = 0.006;

/// The trousers' piece of the base body: the surface from the highest waistband down to the lowest
/// hem, the arms and hands left out (the bottom's predicate, as the shader paints it).
fn trousers_layout(family: &str) -> Arc<Layout> {
    LAYOUTS.get(
        (family.to_string(), "bottom.trousers".to_string(), false),
        || {
            let s = surface(family, BASE_STAGE);
            let fl = base_fields(family);
            let cuts = garment_cuts(family, &s.p);
            let legged: Vec<&Cut> = LEGGED.iter().filter_map(|g| cuts.get(g)).collect();
            let waist = legged
                .iter()
                .map(|c| c.waist.unwrap())
                .fold(f64::MIN, f64::max)
                + CUT_ROOM;
            let leg = legged
                .iter()
                .map(|c| c.leg.unwrap())
                .fold(f64::MAX, f64::min)
                - CUT_ROOM;
            let g: Vec<f64> =
                fl.0.iter()
                    .map(|f| (waist - f[1]).min(f[1] - leg).min((-0.5 - f[0]) * ARM_M))
                    .collect();
            let (idx, src) = clip_by_field(&tris(family), &g);
            let f1: Vec<f64> = fl.0.iter().map(|f| f[1]).collect();
            let hem_y = lerp_src(&src, &f1);
            Layout {
                src,
                idx,
                hem_y,
                sleeve: Vec::new(),
                cut: Cut::default(),
            }
        },
    )
}

/// `bottom.trousers`: the body's own surface under every bottom with legs, as it is (the shader
/// lifts it off the skin by the resident's ease, as it lifts the cloth it paints, and cuts it at the
/// resident's waist and hem: fields.w is the body's height there). No `lod.` copy: far off, the
/// low-detail body is painted as before.
pub(super) fn trousers(family: &str, stage: &str) -> Mesh {
    let lay = trousers_layout(family);
    let (pos, nor, dense) = shell_on(family, stage, &lay.src, false);
    let n = pos.len();
    Mesh {
        pos,
        nor,
        idx: lay.idx.clone(),
        dense,
        fields: lay
            .hem_y
            .iter()
            .map(|&y| [NO_HEM, FLAG_CUT, BOTTOM, y])
            .collect(),
        tone: vec![[0.0; 4]; n],
        ..Mesh::default()
    }
}
