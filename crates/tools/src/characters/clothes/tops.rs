//! Tops, worn as they are painted.

use super::*;

/// The tops worn as they are painted (`contract::garments`: not the jacket, the suit or the apron,
/// which are shells of their own over a painted shirt): one shell, `top.long`, cut to each one's
/// sleeves, hem and neckline by the shader (a tee, a tank top and a shirt are the same mesh).
pub const REGULAR: [&str; 7] = [
    "top.long",
    "top.shirt",
    "top.tee",
    "top.vneck",
    "top.polo",
    "top.tank",
    "top.blouse",
];

/// The regular tops' piece of the base body: the surface any of them covers (the longest sleeve,
/// the lowest hem, the widest neckline), with `CUT_ROOM` to spare.
fn tops_layout(family: &str) -> Arc<Layout> {
    LAYOUTS.get((family.to_string(), "top.long".to_string(), false), || {
        let s = surface(family, BASE_STAGE);
        let fl = base_fields(family);
        let cuts = garment_cuts(family, &s.p);
        let worn: Vec<&Cut> = REGULAR.iter().filter_map(|g| cuts.get(g)).collect();
        let g: Vec<f64> =
            fl.0.iter()
                .map(|f| {
                    worn.iter()
                        .map(|c| top_coverage(f, c))
                        .fold(f64::MIN, f64::max)
                        + CUT_ROOM
                })
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
    })
}

/// `top.long`: the body's own surface under every regular top, as it is (the shader lifts it off
/// the skin by the resident's ease and cuts it at the resident's sleeves, hem and neckline). The
/// cuts are on all four of the body's fields: the height in fields.w, as the trousers', and the
/// sleeve parameter and the two necklines in the tone's x, y and z, where the other shells have
/// their seated corrective (it has none). No `lod.` copy.
pub(super) fn tops(family: &str, stage: &str) -> Mesh {
    let lay = tops_layout(family);
    let (pos, nor, dense) = shell_on(family, stage, &lay.src, false);
    let fl = base_fields(family);
    let of = |k: usize| -> Vec<f64> {
        let f: Vec<f64> = fl.0.iter().map(|f| f[k]).collect();
        lerp_src(&lay.src, &f)
    };
    let (sleeve, crew, scoop) = (of(0), of(2), of(3));
    Mesh {
        pos,
        nor,
        idx: lay.idx.clone(),
        dense,
        fields: lay
            .hem_y
            .iter()
            .map(|&y| [NO_HEM, FLAG_CUT, TOP, y])
            .collect(),
        tone: (0..sleeve.len())
            .map(|i| [sleeve[i], crew[i], scoop[i], 0.0])
            .collect(),
        ..Mesh::default()
    }
}
