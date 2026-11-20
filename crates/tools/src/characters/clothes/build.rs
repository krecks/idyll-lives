//! Building every shell of a family at a stage.
//!
//! Split from `characters/clothes.rs` (see `mod.rs`).

use super::*;

/// Every shell of `family` at `stage`, each with a low-detail copy (`lod.<id>`).
pub fn build_clothes(family: &str, stage: &str, cuts: &Cuts) -> Vec<(String, Mesh)> {
    let mut out = Vec::new();
    for gid in ["top.jacket", "top.suit"] {
        if cuts.contains(gid) {
            out.push((gid.to_string(), jacket(family, stage, gid, false)));
            out.push((format!("lod.{gid}"), jacket(family, stage, gid, true)));
        }
    }
    for gid in ["top.polo", "top.shirt"] {
        if cuts.contains(gid) {
            let m = collar(family, stage, gid, cuts);
            out.push((gid.to_string(), m.clone()));
            out.push((format!("lod.{gid}"), m));
        }
    }
    if cuts.contains("top.apron") {
        out.push(("top.apron".into(), apron(family, stage, false)));
        out.push(("lod.top.apron".into(), apron(family, stage, true)));
    }
    if cuts.contains("top.long") {
        out.push(("top.long".into(), tops(family, stage)));
    }
    if cuts.contains("bottom.trousers") {
        out.push(("bottom.trousers".into(), trousers(family, stage)));
    }
    if cuts.contains("bottom.skirt") {
        out.push((
            "bottom.skirt".into(),
            skirt_shell(family, stage, cuts, SKIRT_SEGS, SKIRT_ROWS),
        ));
        out.push((
            "lod.bottom.skirt".into(),
            skirt_shell(family, stage, cuts, 12, 4),
        ));
    }
    for gid in cuts.keys().filter(|g| g.starts_with("shoes.")) {
        let full = upper(family, stage, gid, false);
        out.push((gid.to_string(), shoe(family, stage, gid, &full, false)));
        out.push((format!("lod.{gid}"), shoe(family, stage, gid, &full, true)));
    }
    out
}
