//! What differs between two models: nodes, materials, clips.
//!
//! Split from `against.rs` (see `mod.rs`).

use super::*;

/// The nodes: the same paths (names under the same parents), each with or without a mesh as before, at rest where
/// it was.
pub(crate) fn nodes_differ(old: &Model, new: &Model, out: &mut Vec<Finding>) {
    only(
        "nodes",
        "nodes",
        old.tree.paths.iter(),
        new.tree.paths.iter(),
        out,
    );
    for (i, path) in old.tree.paths.iter().enumerate() {
        let (a, Some(b)) = (&old.rests[i], new.rest(path)) else {
            continue;
        };
        if old.meshes.contains(path) != new.meshes.contains(path) {
            let side = if old.meshes.contains(path) {
                "old"
            } else {
                "new"
            };
            out.push(found(
                "nodes",
                format!("{path}: has a mesh only in the {side} file"),
            ));
        }
        let moved = py::dist(&a.translation, &b.translation);
        if moved > ORIGIN {
            out.push(found(
                "nodes",
                format!(
                    "{path}: its origin moved {}, at most {} (from {} to {})",
                    mm(moved),
                    mm(ORIGIN),
                    point(a.translation),
                    point(b.translation)
                ),
            ));
        }
        let turned = near::turn(a.rotation, b.rotation);
        if turned > REST_TURN {
            out.push(found(
                "nodes",
                format!("{path}: turned {turned:.3}° at rest, at most {REST_TURN}"),
            ));
        }
        let scaled = (0..3)
            .map(|k| (a.scale[k] - b.scale[k]).abs())
            .fold(0.0, f64::max);
        if scaled > SCALE {
            out.push(found(
                "nodes",
                format!(
                    "{path}: its scale at rest {} is {}, at most {SCALE} off",
                    point(a.scale),
                    point(b.scale)
                ),
            ));
        }
    }
}

/// The differences between two JSON values, each with its path: a number by its value, an object whatever the
/// order of its keys.
pub(crate) fn json_differ(a: Option<&J>, b: Option<&J>, path: &str, out: &mut Vec<String>) {
    let short = |v: &J| {
        let text = digest::canon(v);
        if text.chars().count() > 60 {
            format!("{}…", text.chars().take(60).collect::<String>())
        } else {
            text
        }
    };
    match (a, b) {
        (None, None) => {}
        (Some(a), None) => out.push(format!("{path} ({}) is gone", short(a))),
        (None, Some(b)) => out.push(format!("{path} ({}) is new", short(b))),
        (Some(J::Obj(a)), Some(J::Obj(b))) => {
            let mut keys: Vec<&String> = a.iter().map(|(k, _)| k).collect();
            keys.extend(
                b.iter()
                    .map(|(k, _)| k)
                    .filter(|k| !a.iter().any(|(x, _)| x == *k)),
            );
            for k in keys {
                let of =
                    |o: &'_ [(String, J)]| o.iter().find(|(x, _)| x == k).map(|(_, v)| v.clone());
                let inner = if path.is_empty() {
                    k.clone()
                } else {
                    format!("{path}.{k}")
                };
                json_differ(of(a).as_ref(), of(b).as_ref(), &inner, out);
            }
        }
        (Some(J::Arr(a)), Some(J::Arr(b))) if a.len() == b.len() => {
            for (i, (x, y)) in a.iter().zip(b).enumerate() {
                json_differ(Some(x), Some(y), &format!("{path}[{i}]"), out);
            }
        }
        (Some(a), Some(b)) => {
            if digest::canon(a) != digest::canon(b) {
                out.push(format!("{path}: {} is now {}", short(a), short(b)));
            }
        }
    }
}

/// An object without some of its keys.
pub(crate) fn without(v: &J, keys: &[&str]) -> J {
    match v {
        J::Obj(o) => J::Obj(
            o.iter()
                .filter(|(k, _)| !keys.contains(&k.as_str()))
                .cloned()
                .collect(),
        ),
        other => other.clone(),
    }
}

/// The materials: the same names, each with its base colour within a step and everything else it says.
pub(crate) fn materials_differ(o: &Glb, n: &Glb, out: &mut Vec<Finding>) {
    let named = |g: &Glb| -> BTreeMap<String, J> {
        g.gltf
            .get("materials")
            .and_then(J::list)
            .unwrap_or(&[])
            .iter()
            .map(|m| (m.get("name").map(J::key).unwrap_or_default(), m.clone()))
            .collect()
    };
    let (old, new) = (named(o), named(n));
    only("materials", "materials", old.keys(), new.keys(), out);
    for (name, a) in &old {
        let Some(b) = new.get(name) else { continue };
        let colour = |m: &J| -> [f64; 4] {
            let list: Vec<f64> = m
                .get("pbrMetallicRoughness")
                .and_then(|p| p.get("baseColorFactor"))
                .and_then(J::list)
                .unwrap_or(&[])
                .iter()
                .filter_map(|v| v.num().map(|x| x.f()))
                .collect();
            list.try_into().unwrap_or([1.0; 4])
        };
        let (ca, cb) = (colour(a), colour(b));
        let off = (0..4).map(|k| (ca[k] - cb[k]).abs()).fold(0.0, f64::max);
        if off > COLOUR {
            let text =
                |c: [f64; 4]| format!("({:.3}, {:.3}, {:.3}, {:.3})", c[0], c[1], c[2], c[3]);
            out.push(found(
                "materials",
                format!(
                    "material {name}: its base colour {} is now {}, {:.1} of 255 off, at most 1",
                    text(ca),
                    text(cb),
                    off * 255.0
                ),
            ));
        }
        let rest = |m: &J| {
            let mut m = without(m, &["name"]);
            if let Some(pbr) = m
                .get("pbrMetallicRoughness")
                .map(|p| without(p, &["baseColorFactor"]))
            {
                m.set("pbrMetallicRoughness", pbr);
            }
            m
        };
        let mut differs = Vec::new();
        json_differ(Some(&rest(a)), Some(&rest(b)), "", &mut differs);
        if !differs.is_empty() {
            out.push(found(
                "materials",
                format!("material {name}: {}", names(&differs)),
            ));
        }
    }
}

/// The clips: the same names, as long, and every part they move where it was at each sixtieth of a second (a part
/// one build's clip leaves alone is at rest there). How many keys a clip has is free.
pub(crate) fn clips_differ(old: &Model, new: &Model, out: &mut Vec<Finding>) {
    only(
        "clips",
        "clips",
        old.clips.iter().map(|c| &c.name),
        new.clips.iter().map(|c| &c.name),
        out,
    );
    for a in &old.clips {
        let Some(b) = new.clips.iter().find(|c| c.name == a.name) else {
            continue;
        };
        for (what, x, y) in [("starts", a.start, b.start), ("ends", a.end, b.end)] {
            if (x - y).abs() > CLIP_LENGTH {
                out.push(found(
                    "clips",
                    format!(
                        "clip {}: {what} at {y:.4} s, was {x:.4} (at most 1 ms off)",
                        a.name
                    ),
                ));
            }
        }
        let end = a.end.max(b.end);
        let mut times: Vec<f64> = (0..=(end * CLIP_HZ).floor() as usize)
            .map(|k| k as f64 / CLIP_HZ)
            .collect();
        times.push(end);
        let tracks: BTreeSet<&(String, String)> = a.tracks.keys().chain(b.tracks.keys()).collect();
        for track in tracks {
            let (node, path) = track;
            let (Some(ra), Some(rb)) = (old.rest(node), new.rest(node)) else {
                continue;
            };
            let rest = |r: &Rest| -> Vec<f64> {
                match path.as_str() {
                    "translation" => r.translation.to_vec(),
                    "rotation" => r.rotation.to_vec(),
                    "scale" => r.scale.to_vec(),
                    _ => Vec::new(),
                }
            };
            let rotation = path == "rotation";
            // (off by, at, limit, unit)
            let mut worst: Option<(f64, f64)> = None;
            let (limit, unit) = match path.as_str() {
                "translation" => (CLIP_MOVE * 1000.0, "mm"),
                "rotation" => (CLIP_TURN, "°"),
                _ => (SCALE, ""),
            };
            for &t in &times {
                let va = a
                    .tracks
                    .get(track)
                    .and_then(|k| k.at(t, rotation))
                    .unwrap_or_else(|| rest(ra));
                let vb = b
                    .tracks
                    .get(track)
                    .and_then(|k| k.at(t, rotation))
                    .unwrap_or_else(|| rest(rb));
                let off = if va.len() != vb.len() {
                    f64::INFINITY
                } else {
                    match path.as_str() {
                        "translation" => py::dist(&va, &vb) * 1000.0,
                        "rotation" if va.len() == 4 => {
                            near::turn([va[0], va[1], va[2], va[3]], [vb[0], vb[1], vb[2], vb[3]])
                        }
                        _ => va
                            .iter()
                            .zip(&vb)
                            .map(|(x, y)| (x - y).abs())
                            .fold(0.0, f64::max),
                    }
                };
                if worst.is_none_or(|(w, _)| off > w) {
                    worst = Some((off, t));
                }
            }
            if let Some((off, t)) = worst.filter(|(off, _)| *off > limit) {
                out.push(found(
                    "clips",
                    format!(
                        "clip {}: {node}'s {path} is {off:.3}{unit} off at {t:.3} s, at most {limit}{unit}",
                        a.name
                    ),
                ));
            }
        }
    }
}
