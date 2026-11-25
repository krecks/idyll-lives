//! Checking one file at a time, and counting what the checks find.

use super::*;

/// The numbers of one file, for the summary and the far version's checks.
struct Info {
    size: usize,
    tris: i64,
    draws: usize,
    materials: usize,
    dims: [Num; 3],
    lo: [Num; 3],
    clips: usize,
    moving: usize,
    /// What each counted check run on the file (`FileCheck`) found.
    counted: Vec<(&'static str, Found)>,
}

fn dims_text(dims: &[Num; 3]) -> String {
    dims.iter()
        .map(|d| fixed(d.f(), 2))
        .collect::<Vec<_>>()
        .join(" x ")
}

/// Whether the model `key` is only ever drawn in a resident's hand (a held pack's `tool.*`: a fork, a coin, a phone),
/// never on a floor or a top. (The pack's `item.*` are set down too: a plate on a table, a parcel on the doorstep.)
pub(super) fn in_hand(key: &str) -> bool {
    key.starts_with("tool.")
}

/// A model file as a counted check gets it: the model's key and entry, the file, the manifests' finishes, and
/// whether it is the model's far version.
pub(super) struct ModelFile<'a> {
    pub(super) key: &'a str,
    pub(super) entry: &'a J,
    pub(super) glb: &'a Glb,
    pub(super) fin: &'a Finishes,
    pub(super) far: bool,
}

/// A counted check (`COUNTED`) run on each model file as the gate reads it, the full model's and its far
/// version's (whose findings are the model's): what it finds in the file.
pub(super) type FileCheck = fn(&ModelFile) -> R<Found>;

/// The counted checks run on each model file, by name.
pub(super) const FILE_CHECKS: &[(&str, FileCheck)] =
    &[("edges", meshes::hard_edges), ("texels", meshes::texels)];

/// A counted check (`COUNTED`) run over every model file at once (an outside program, `--validate`) before the
/// models are judged: given each model's key, its file and its far version's, what it found in each model it read.
pub type BatchCheck =
    fn(&[(String, String, Option<String>)]) -> Result<BTreeMap<String, Found>, String>;

/// The counted checks of a gate run and the baseline they are judged by.
pub(super) struct Counting<'a> {
    pub(super) baseline: &'a Baseline,
    pub(super) file: &'a [(&'static str, FileCheck)],
    /// What the batch checks (`BatchCheck`) found: {check: {key: …}}.
    pub(super) batch: BTreeMap<&'static str, BTreeMap<String, Found>>,
}

/// One model's findings of a counted check by the baseline (`baseline::sort`), as (errors, warnings) for
/// `strictly`, and the count the baseline may keep. Over it or new, the findings are strict: errors unless the
/// model's entry `accept`s the check. An entry that accepts it takes its findings whatever the baseline holds
/// (it may keep none), so a count the baseline still has for the model is stale.
fn ratchet(
    baseline: &Baseline,
    entry: &J,
    key: &str,
    check: &'static str,
    found: Found,
) -> (Vec<String>, Vec<String>, u64) {
    let accepted = entry
        .get("accept")
        .and_then(|a| a.get(check))
        .and_then(J::str)
        .is_some_and(|r| !r.trim().is_empty());
    let count = found.count;
    let verdict = match (accepted, count) {
        (false, _) => baseline.judge(check, key, count),
        (true, 0) => Verdict::Clean,
        (true, _) => Verdict::New(count),
    };
    let sorted = baseline::sort(check, verdict, found);
    let mut errors = sorted.errors;
    let base = baseline.get(check, key);
    if accepted && base > 0 {
        errors.push(format!(
            "{check}: its entry accepts it, and the baseline still has {base}: take it out (`cargo check-glb --update-baseline`)"
        ));
    }
    let mut warnings: Vec<String> = sorted
        .strict
        .into_iter()
        .map(|t| strict(check, t))
        .collect();
    warnings.extend(sorted.warnings);
    (errors, warnings, if accepted { 0 } else { count })
}

/// (errors, warnings, summary) for one model entry and its file (and its far version's, at `far`);
/// `facts`: those of each content object that draws it (`content_objects`); `centre`: where it's built
/// centred (centres.json; None: not built here, not checked); `counting`: the counted checks.
#[allow(clippy::too_many_arguments)]
pub(super) fn check(
    entry_key: &str,
    fin: &Finishes,
    entry: &J,
    path: &str,
    far: Option<&str>,
    facts: Option<&Vec<Facts>>,
    centre: Option<&J>,
    furniture: Option<&Furniture>,
    counting: &Counting,
) -> R<Checked> {
    let (mut errors, mut warnings, info) = check_file(
        entry_key,
        fin,
        entry,
        path,
        false,
        facts,
        centre,
        furniture,
        counting.file,
    )?;
    // `opened` came in after the models it finds: counted against the baseline.
    let mut opened = Found::default();
    warnings.retain(|w| match untag(w) {
        Some(("opened", text)) => {
            opened.count += 1;
            opened.text.push(text.to_string());
            false
        }
        _ => true,
    });
    let mut summary = format!(
        "{:>6} KB {:>6} tris {} draws {:>2} materials  {} m",
        fixed(info.size as f64 / 1024.0, 1),
        info.tris,
        info.draws,
        info.materials,
        dims_text(&info.dims)
    );
    if info.clips > 0 {
        summary += &format!("  {} moving parts, {} clips", info.moving, info.clips);
    }
    let mut far_counted = Vec::new();
    if let Some(far) = far {
        // Its far version (`far`): the same model, still, with far fewer triangles.
        // (Without what the full model is checked for: its clips, colours and centre, a bird's parts and the perches.)
        let far_entry = J::Obj(
            entry
                .entries()
                .unwrap_or(&[])
                .iter()
                .filter(|(k, _)| {
                    !["motions", "paint", "perches", "habitat", "bird"].contains(&k.as_str())
                })
                .cloned()
                .collect(),
        );
        let (mut far_errors, mut far_warnings, far_info) = match check_file(
            entry_key,
            fin,
            &far_entry,
            far,
            true,
            None,
            None,
            None,
            counting.file,
        ) {
            Ok((e, w, i)) => (e, w, Some(i)),
            Err(err) => (vec![format!("cannot read {far}: {err}")], Vec::new(), None),
        };
        if let Some(fi) = far_info {
            if fi.clips > 0 {
                far_errors.push("has clips: a far version stands still".into());
            }
            if fi.tris as f64 > info.tris as f64 / 2.0 {
                far_warnings.push(format!(
                    "{} triangles, over half the full model's {} (aim for a quarter)",
                    fi.tris, info.tris
                ));
            }
            // It stands where the model stands: decimating can drop the lowest points and lift it off the floor.
            let off = fi.lo[1] - info.lo[1];
            if off.f().abs() > 0.005 {
                far_errors.push(format!(
                    "its lowest point is {} cm off the full model's: a far version stands on the floor as the model does (G.far_of, or far= keeps the floor)",
                    signed((off * Num::I(100)).f(), 1)
                ));
            }
            if fi
                .dims
                .iter()
                .zip(&info.dims)
                .any(|(a, b)| (a.f() - b.f()).abs() > f64::max(0.05 * b.f(), 0.02))
            {
                far_warnings.push(strict(
                    "far",
                    format!(
                        "{} m, the full model {} m",
                        dims_text(&fi.dims),
                        dims_text(&info.dims)
                    ),
                ));
            }
            summary += &format!("  (far: {} tris)", fi.tris);
            far_counted = fi.counted;
        }
        // (A strict finding of the far version stays one: `strict`'s mark comes first.)
        let far = |w: String| match untag(&w) {
            Some((kind, text)) => strict(kind, format!("far: {text}")),
            None => format!("far: {w}"),
        };
        errors.extend(far_errors.into_iter().map(far));
        // (What it says of the far version, it says of the full model too.)
        warnings.extend(far_warnings.into_iter().map(far));
    }
    // The counted checks: what they found in the file and its far version's (`far: …`), and the batch checks'.
    let mut found: BTreeMap<&'static str, Found> = BTreeMap::new();
    // (Counted where it finds any, or the baseline or the entry names it: none found is then stale, or unused.)
    if opened.count > 0
        || counting.baseline.get("opened", entry_key) > 0
        || entry.get("accept").and_then(|a| a.get("opened")).is_some()
    {
        found.insert("opened", opened);
    }
    let far_found = far_counted.into_iter().map(|(c, f)| {
        let text = f.text.into_iter().map(|t| format!("far: {t}")).collect();
        (
            c,
            Found {
                count: f.count,
                text,
            },
        )
    });
    let batch = counting
        .batch
        .iter()
        .filter_map(|(c, by_key)| Some((*c, by_key.get(entry_key)?.clone())));
    for (c, f) in info.counted.into_iter().chain(far_found).chain(batch) {
        let all = found.entry(c).or_default();
        all.count += f.count;
        all.text.extend(f.text);
    }
    let mut counts = Vec::new();
    for (c, f) in found {
        let (e, w, n) = ratchet(counting.baseline, entry, entry_key, c, f);
        errors.extend(e);
        warnings.extend(w);
        counts.push((c, n));
    }
    let ran: Vec<&str> = counts.iter().map(|(c, _)| *c).collect();
    let (errors, warnings, accepted) = strictly(entry, errors, warnings, &ran);
    Ok(Checked {
        errors,
        warnings,
        accepted,
        summary,
        counts,
    })
}

/// What `check` found of one model: its errors, its warnings, the strict findings its entry `accept`s (each with
/// its reason), and its numbers.
pub(super) struct Checked {
    pub(super) errors: Vec<String>,
    pub(super) warnings: Vec<String>,
    pub(super) accepted: Vec<String>,
    pub(super) summary: String,
    /// What the counted checks that ran counted, as the baseline may keep it (`ratchet`).
    pub(super) counts: Vec<(&'static str, u64)>,
}

/// (errors, warnings, numbers) for one model entry and a file of it (`far`: its far version); `facts`: those of
/// each content object that draws it (`content_objects`), each checked; `centre`: where it's built centred
/// (centres.json), or None; `counted`: the counted checks to run on it.
#[allow(clippy::too_many_arguments)]
fn check_file(
    entry_key: &str,
    fin: &Finishes,
    entry: &J,
    path: &str,
    far: bool,
    facts: Option<&Vec<Facts>>,
    centre: Option<&J>,
    furniture: Option<&Furniture>,
    counted: &[(&'static str, FileCheck)],
) -> R<(Vec<String>, Vec<String>, Info)> {
    let (mut errors, mut warnings) = (Vec::new(), Vec::new());
    let data = std::fs::read(path).map_err(|e| py::os_error(&e, path))?;
    let g = parse_glb(&data)?;
    let gltf = &g.gltf;
    let empty = J::Arr(Vec::new());
    let required: BTreeSet<String> = gltf
        .get("extensionsRequired")
        .and_then(J::list)
        .unwrap_or(&[])
        .iter()
        .map(J::key)
        .collect();
    for ext in required
        .iter()
        .filter(|e| !EXTENSIONS.contains(&e.as_str()))
    {
        errors.push(format!("requires {ext}, which the game cannot read"));
    }
    if gltf.get("skins").is_some_and(J::truthy) {
        errors.push("has a skin: placed things are drawn as thin instances, without bones".into());
    }
    let meshes = gltf.get("meshes").and_then(J::list).unwrap_or(&[]);
    let mut morphs = false;
    'm: for mesh in meshes {
        for prim in key(mesh, "primitives")?.list().unwrap_or(&[]) {
            if prim.get("targets").is_some_and(J::truthy) {
                morphs = true;
                break 'm;
            }
        }
    }
    if morphs {
        errors.push(
            "has morph targets: placed things are drawn as thin instances, without them".into(),
        );
    }
    if gltf.get("cameras").is_some_and(J::truthy) {
        warnings.push(strict("cameras", "has cameras; the game drops them".into()));
    }
    for (mesh, faces) in flipped_faces(&g)? {
        warnings.push(strict("flipped", format!(
            "mesh {}: {faces} faces over 1 cm² have normals turned against their winding, lit as if from behind (a face built inside out, or a bevel's arc miter)",
            repr_str(&mesh)
        )));
    }
    let (clip_errors, clip_warnings) = check_clips(&g, entry)?;
    errors.extend(clip_errors);
    warnings.extend(clip_warnings);
    // The parts the player may colour (`paint`: a material's name, or {material, label}): materials of the file.
    let materials = gltf.get("materials").and_then(J::list).unwrap_or(&[]);
    let names: Vec<J> = materials
        .iter()
        .map(|m| m.get("name").cloned().unwrap_or(J::Str(String::new())))
        .collect();
    for slot in entry.get("paint").and_then(J::list).unwrap_or(&[]) {
        let name = if matches!(slot, J::Obj(_)) {
            slot.get("material").cloned().unwrap_or(J::Null)
        } else {
            slot.clone()
        };
        if matches!(slot, J::Obj(_)) && !slot.get("label").is_some_and(J::truthy) {
            errors.push(format!("`paint` part {} has no label", repr(&name)));
        }
        if !names.contains(&name) {
            errors.push(format!(
                "`paint` names {}, which is no material of the file",
                repr(&name)
            ));
        }
    }
    if gltf
        .get("extensionsUsed")
        .and_then(J::list)
        .unwrap_or(&[])
        .contains(&J::Str("KHR_lights_punctual".into()))
    {
        warnings.push(strict(
            "lights",
            "has lights; the game drops them (a lamp's light is the content's `light`)".into(),
        ));
    }

    let accessors = gltf.get("accessors").unwrap_or(&empty);
    let no_overrides = J::Obj(Vec::new());
    let overrides = entry.get("materials").unwrap_or(&no_overrides);
    // Mirror glass (a material with `mirror`): flat, on one part (models.ts `fileMirror`).
    let mut glass: BTreeMap<String, (Bounds, Option<usize>)> = BTreeMap::new();
    for (name, ov) in overrides.entries().unwrap_or(&[]) {
        let mirror = ov.opt("mirror");
        let truthy = mirror.is_some_and(J::truthy);
        if !matches!(mirror.and_then(J::str), None | Some("front") | Some("back"))
            || mirror.is_some_and(|m| m.str().is_none())
        {
            errors.push(format!(
                "material {}: `mirror` is front or back, not {}",
                repr_str(name),
                repr(mirror.unwrap())
            ));
        } else if truthy && !names.contains(&J::Str(name.clone())) {
            errors.push(format!(
                "material {} is mirror glass, but no material of the file",
                repr_str(name)
            ));
        } else if truthy && fin.see_through(ov.opt("finish")) {
            warnings.push(strict("mirror", format!(
                "mirror glass {} is see-through ({}): where nothing reflects (Medium, Low) it shows what is behind it; map it to gloss",
                repr_str(name),
                str_of(key(ov, "finish")?)
            )));
        }
        if let Some(glow) = ov.opt("glow") {
            let ok = glow.str().is_some_and(|s| {
                s.chars().count() == 7
                    && s.starts_with('#')
                    && s.chars().skip(1).all(|c| c.is_ascii_hexdigit())
            });
            if !ok {
                errors.push(format!(
                    "material {}: `glow` is a colour, #rrggbb, not {}",
                    repr_str(name),
                    repr(glow)
                ));
            }
        }
        if let Some(screen) = ov.get("screen") {
            if !matches!(screen, J::Bool(_)) {
                errors.push(format!(
                    "material {}: `screen` is true or false, not {}",
                    repr_str(name),
                    repr(screen)
                ));
            } else if screen.truthy() && ov.opt("glow").is_none() {
                errors.push(format!(
                    "material {}: `screen` needs a `glow` colour",
                    repr_str(name)
                ));
            }
        }
    }
    let (inf, ninf) = (Num::F(f64::INFINITY), Num::F(f64::NEG_INFINITY));
    let (mut lo, mut hi) = ([inf; 3], [ninf; 3]);
    let mut tris = 0i64;
    let mut groups: BTreeSet<(Option<usize>, String)> = BTreeSet::new();
    // Per problem, the materials that have it.
    let mut flags: HashMap<&str, BTreeSet<String>> = HashMap::new();
    // Draw calls: one per finish group, the still body's and each moving part's apart.
    for p in node_matrices(gltf, None, None)? {
        for prim in primitives(gltf, &p.mesh)? {
            if !prim
                .get("mode")
                .is_none_or(|m| m.num().is_some_and(|n| n.f() == 4.0))
            {
                errors.push("a primitive that is not triangles".into());
                continue;
            }
            let pos = index(accessors, key(key(prim, "attributes")?, "POSITION")?)?;
            let counted = match prim.get("indices") {
                Some(i) => index(accessors, i)?,
                None => pos,
            };
            let n = key(counted, "count")?.num().map_or(0, |n| n.f() as i64);
            tris += n.div_euclid(3);
            // Bounds: the accessor's box through the node matrix, then glTF x -> game -x (as `render/pc/models.ts` bakes it).
            let (bl, bh) = box_of(pos)?;
            for cx in [bl[0], bh[0]] {
                for cy in [bl[1], bh[1]] {
                    for cz in [bl[2], bh[2]] {
                        let mut q = apply(&p.m, &[cx, cy, cz]);
                        q[0] = -q[0];
                        for k in 0..3 {
                            lo[k] = py::min(lo[k], q[k]);
                            hi[k] = py::max(hi[k], q[k]);
                        }
                    }
                }
            }
            let unnamed = J::Obj(vec![("name".into(), J::Str(String::new()))]);
            let mat = match prim.get("material") {
                Some(i) => index(&J::Arr(materials.to_vec()), i)?.clone(),
                None => unnamed,
            };
            let name = mat.get("name").map(J::key).unwrap_or_default();
            let ov = override_of(overrides, &name);
            let ov_on = ov.truthy();
            if ov_on && ov.get("mirror").is_some_and(J::truthy) {
                let ((glo, ghi), part) = glass
                    .entry(name.clone())
                    .or_insert((([inf; 3], [ninf; 3]), p.part));
                let (bl, bh) = box_of(pos)?;
                for k in 0..3 {
                    let mut l = Num::I(0);
                    let mut h = Num::I(0);
                    for j in 0..3 {
                        let mj = p.m[j * 4 + k];
                        l = l + mj * if mj.f() >= 0.0 { bl[j] } else { bh[j] };
                        h = h + mj * if mj.f() >= 0.0 { bh[j] } else { bl[j] };
                    }
                    glo[k] = py::min(glo[k], l + p.m[12 + k]);
                    ghi[k] = py::max(ghi[k], h + p.m[12 + k]);
                }
                if *part != p.part {
                    errors.push(format!(
                        "mirror glass {} is on two parts: one material per mirror",
                        repr_str(&name)
                    ));
                }
            }
            let pbr = mat.get("pbrMetallicRoughness");
            let textured = ["baseColorTexture", "metallicRoughnessTexture"]
                .iter()
                .any(|k| pbr.is_some_and(|p| p.has(k)))
                || ["normalTexture", "occlusionTexture", "emissiveTexture"]
                    .iter()
                    .any(|k| mat.has(k));
            let finish = if ov_on {
                ov.get("finish").filter(|f| f.truthy())
            } else {
                None
            };
            // UVs a textured finish draws with (artkit's uv.rs) or the file's own textures read; a plain part joined
            // with textured ones carries one UV throughout, which costs next to nothing compressed.
            if let Some(uvs) = key(prim, "attributes")?.get("TEXCOORD_0") {
                let used = finish.map_or(textured, |f| fin.textured(f));
                if !used && !constant(&g, uvs)? {
                    flags.entry("uvs").or_default().insert(name.clone());
                }
            }
            if coloured(&g, prim)? {
                flags.entry("coloured").or_default().insert(name.clone());
            }
            if ov_on && ov.get("hide").is_some_and(J::truthy) {
                continue;
            }
            if let Some(finish) = finish {
                let f = finish.key();
                if !fin.plain.contains_key(&f) {
                    errors.push(format!(
                        "material {} maps to {f}, which is no finish in the manifests",
                        repr_str(&name)
                    ));
                }
                groups.insert((
                    p.part,
                    if fin.plain.get(&f) == Some(&true) {
                        "plain".into()
                    } else {
                        f
                    },
                ));
                continue;
            }
            groups.insert((p.part, format!("file:{name}")));
            flags.entry("unmapped").or_default().insert(name.clone());
            if mat.get("doubleSided").is_some_and(J::truthy) {
                flags.entry("double").or_default().insert(name.clone());
            }
            if mat.get("alphaMode") == Some(&J::Str("BLEND".into())) {
                flags.entry("blend").or_default().insert(name.clone());
            }
            if mat
                .get("emissiveFactor")
                .map(|e| e.list().unwrap_or(&[]).iter().any(J::truthy))
                .unwrap_or(false)
            {
                flags.entry("emissive").or_default().insert(name.clone());
            }
        }
    }
    for (name, ((glo, ghi), _part)) in &glass {
        // (not `size`: that is the file's, for the summary)
        let mut extent: Vec<Num> = (0..3).map(|k| ghi[k] - glo[k]).collect();
        extent.sort_by(|a, b| {
            a.f()
                .partial_cmp(&b.f())
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        if extent[0].f() > 0.1 * extent[1].f() {
            warnings.push(strict("mirror", format!(
                "mirror glass {} isn't flat ({} cm thick, {} cm across): the reflection lies on its box's thinnest side",
                repr_str(name),
                fixed((extent[0] * Num::I(100)).f(), 1),
                fixed((extent[1] * Num::I(100)).f(), 0)
            )));
        }
    }
    for (flag, text) in [
        (
            "unmapped",
            "not mapped to a finish in the entry's `materials` (each its own draw call and look)",
        ),
        ("double", "double-sided (every back face drawn too)"),
        (
            "blend",
            "see-through (BLEND: drawn after everything else, unsorted between instances)",
        ),
        (
            "emissive",
            "glowing by day too (emissive; map it to a finish, the lamp lights it at night)",
        ),
        ("uvs", "carrying UVs nothing reads (export without them)"),
    ] {
        if let Some(names) = flags.get(flag).filter(|n| !n.is_empty()) {
            warnings.push(strict(
                "materials",
                format!(
                    "materials {text}: {}",
                    names.iter().cloned().collect::<Vec<_>>().join(", ")
                ),
            ));
        }
    }
    if let Some(names) = flags.get("coloured").filter(|n| !n.is_empty()) {
        let names = names.iter().cloned().collect::<Vec<_>>().join(", ");
        errors.push(format!("materials with a coloured COLOR_0: {names} (the game multiplies it in as baked shading, grey: artkit's shade.rs)"));
    }

    // Units and origin: on the floor, centred on the footprint (models.ts lines files up by neither).
    let fit = entry.get("fit").is_some_and(J::truthy);
    let scale = if fit {
        Num::I(1)
    } else {
        entry
            .get("scale")
            .map(|s| s.num().ok_or("can't multiply sequence by non-int"))
            .transpose()?
            .unwrap_or(Num::I(1))
    };
    let lo = lo.map(|v| v * scale);
    let hi = hi.map(|v| v * scale);
    let dims = [hi[0] - lo[0], hi[1] - lo[1], hi[2] - lo[2]];
    // As it shows: the parts a clip grows (hidden or small at rest: darts, a pot thrown) grown, for what
    // is compared with the content (the footprint, the wall, what the floor hides, a top's disc).
    let none = J::Obj(Vec::new());
    let motions = entry.get("motions").filter(|m| m.truthy()).unwrap_or(&none);
    let pose = grown(
        &g,
        &[motions.opt("open").cloned(), motions.opt("night").cloned()],
    )?;
    let ((slo, shi), sparts) = bounds(gltf, &pose, &HashSet::new())?;
    let (slo, shi) = (slo.map(|v| v * scale), shi.map(|v| v * scale));
    // (Hung at its real height, a thing on a wall or the ceiling floats as it should: every content object
    // drawing it is one. A model no content object draws, a held tool or a far version, is checked as on the floor.)
    let on_wall_or_ceiling = facts.is_some_and(|f| {
        !f.is_empty()
            && f.iter()
                .all(|f| matches!(f.layer.str(), Some("wall") | Some("ceiling")))
    });
    if !fit && !entry.get("offset").is_some_and(J::truthy) {
        // On the floor (y = 0). A model the game sinks may reach below it by its `sink` (the woods' rock, the marker).
        let sink = entry
            .get("sink")
            .map(|s| s.num().ok_or("unsupported operand type(s) for -"))
            .transpose()?
            .unwrap_or(Num::I(0));
        if lo[1].f() < (Num::F(-0.005) - sink).f() {
            errors.push(format!(
                "reaches {} cm below the floor (origin on the floor, y = 0)",
                fixed((-lo[1] * Num::I(100)).f(), 1)
            ));
        } else if lo[1].f() > 0.005 && !on_wall_or_ceiling {
            warnings.push(strict(
                "float",
                format!(
                    "floats {} cm above its origin (fine only on a wall or ceiling)",
                    fixed((lo[1] * Num::I(100)).f(), 1)
                ),
            ));
        }
        // Centred where its `Model` says (centres.json, which the build writes: the middle, or its `centre`, a
        // fridge against the wall, a wall bed at the back of its tiles), at rest as `G.settle` centres it (grown,
        // the dartboard's darts and the unrolled yoga mat reach forward). A file not built here isn't checked.
        if let Some(centre) = centre.filter(|c| **c != J::Null) {
            let at = (
                ((lo[0] + hi[0]) / Num::I(2)).f(),
                ((lo[2] + hi[2]) / Num::I(2)).f(),
            );
            let c0 = centre
                .at(0)
                .ok_or("list index out of range")?
                .num()
                .ok_or("unsupported operand type(s) for -")?
                .f();
            let c1 = centre
                .at(1)
                .ok_or("list index out of range")?
                .num()
                .ok_or("unsupported operand type(s) for -")?
                .f();
            if f64::max((at.0 - c0).abs(), (at.1 - c1).abs()) > OFF_CENTRE {
                warnings.push(strict("centre", format!(
                    "centred on (x {}, depth {}), its Model's `centre` is ({}, {}) (centres.json): rebuild it, or `cargo art --centres` if the Model changed",
                    fixed(at.0, 3),
                    fixed(at.1, 3),
                    fixed(c0, 3),
                    fixed(c1, 3)
                )));
            }
        }
        // Against what each content object that draws it says of it (two objects may share a model).
        let mut seen = HashSet::new();
        let each: Vec<Option<&Facts>> = match facts.filter(|f| !f.is_empty()) {
            Some(f) => f.iter().map(Some).collect(),
            None => vec![None],
        };
        let many = facts.is_some_and(|f| f.len() > 1);
        for fact in each {
            // Its functional heights against the model (`heights`: warnings only).
            let mut found = content_warnings(
                fact,
                &lo,
                &(slo, shi),
                &sparts,
                scale,
                &g,
                &pose,
                entry,
                in_hand(entry_key),
            )?;
            if let (Some(f), Some(furniture)) = (fact, furniture) {
                found.extend(
                    heights(&g, entry, &f.object, furniture, true)?
                        .iter()
                        .flat_map(Height::warnings),
                );
            }
            for w in found {
                let w = if many {
                    format!("{w} (object {})", str_of(&fact.unwrap().id))
                } else {
                    w
                };
                if seen.insert(w.clone()) {
                    warnings.push(w);
                }
            }
        }
    }
    // Parts flattened to nothing at rest whose flat shape shows (README trap "Grown from nothing").
    let opaque = |index_: Option<&J>| -> R<bool> {
        let mat = match index_ {
            Some(i) => index(&J::Arr(materials.to_vec()), i)?.clone(),
            None => J::Obj(vec![("name".into(), J::Str(String::new()))]),
        };
        let ov = override_of(overrides, &mat.get("name").map(J::key).unwrap_or_default());
        if ov.truthy() && ov.get("hide").is_some_and(J::truthy) {
            return Ok(false);
        }
        Ok(
            match ov.get("finish").filter(|f| ov.truthy() && f.truthy()) {
                Some(f) => !fin.see_through(Some(f)),
                None => mat.get("alphaMode") != Some(&J::Str("BLEND".into())),
            },
        )
    };
    for (name, share) in flattened_in_view(&g, &opaque)? {
        warnings.push(strict("flat", format!(
            "{name} lies flattened at rest with {} of it outside anything opaque, so its flat shape shows: put its origin inside something opaque (README trap \"Grown from nothing\")",
            percent(share)
        )));
    }

    // A bird the renderer flies (`bird`): its parts and size against the file; where birds sit on the model
    // (`perches`): inside it, and what they are on (`habitat`).
    if let Some(bird) = entry.get("bird") {
        let fields = bird_fields(bird);
        if fields.is_empty() {
            errors.extend(bird_errors(bird, &bird_parts(&g)?, dims.map(|d| d.f())));
        }
        errors.extend(fields);
    }
    if let Some(perches) = entry.get("perches") {
        errors.extend(perch_errors(
            perches,
            lo.map(|v| v.f()),
            hi.map(|v| v.f()),
            scale.f(),
        ));
    }
    errors.extend(habitat_errors(entry));

    // Budgets.
    let side = dims
        .iter()
        .copied()
        .reduce(|m, v| if v.f() > m.f() { v } else { m })
        .unwrap();
    let (target, most) = TRIANGLES
        .iter()
        .find(|(up, _, _)| side.f() <= *up)
        .map(|(_, t, m)| (*t, *m))
        .ok_or("StopIteration")?;
    let texture_most = texture_most(side.f());
    if tris > most {
        errors.push(format!(
            "{tris} triangles, at most {most} for a model {} m across (aim for {target})",
            fixed(side.f(), 2)
        ));
    } else if tris > target {
        warnings.push(format!(
            "{tris} triangles, aim for {target} for a model {} m across (at most {most})",
            fixed(side.f(), 2)
        ));
    }
    let most_draws = match draws_allowed(entry) {
        Ok(n) => n,
        Err(e) => {
            errors.push(e);
            DRAWS.1
        }
    };
    if groups.len() > most_draws {
        errors.push(format!(
            "{} draw calls, at most {} (one per finish group: map materials to finishes)",
            groups.len(),
            most_draws
        ));
    } else if groups.len() > DRAWS.0 {
        warnings.push(format!(
            "{} draw calls, aim for {} at most",
            groups.len(),
            DRAWS.0
        ));
    }
    for (i, img) in gltf
        .get("images")
        .and_then(J::list)
        .unwrap_or(&[])
        .iter()
        .enumerate()
    {
        let wh = match img.get("bufferView") {
            Some(v) => {
                let view = item(gltf, "bufferViews", v)?;
                let start = glb::int(view, "byteOffset", 0)?.max(0) as usize;
                let len = key(view, "byteLength")?
                    .num()
                    .map_or(0, |n| n.f() as i64)
                    .max(0) as usize;
                image_size(&g.bin[start.min(g.bin.len())..(start + len).min(g.bin.len())])
            }
            None => None,
        };
        if let Some((w, h)) = wh
            && w.max(h) > texture_most
        {
            errors.push(format!(
                "texture {i} is {w}x{h}, at most {texture_most} px a side for a model {} m across",
                fixed(side.f(), 2)
            ));
        }
        let label = img
            .get("name")
            .or(img.get("uri"))
            .map(str_of)
            .unwrap_or("embedded".into());
        warnings.push(format!(
            "texture {i} ({label}): a texture of its own, prefer the game's finishes"
        ));
    }

    let info = Info {
        size: g.size,
        tris,
        draws: groups.len(),
        materials: materials.len(),
        dims,
        lo,
        clips: gltf
            .get("animations")
            .and_then(J::list)
            .map_or(0, |a| a.len()),
        moving: moved_nodes(gltf)?.len(),
        counted: counted
            .iter()
            .map(|(c, run)| {
                let file = ModelFile {
                    key: entry_key,
                    entry,
                    glb: &g,
                    fin,
                    far,
                };
                Ok((*c, run(&file)?))
            })
            .collect::<R<_>>()?,
    };
    Ok((errors, warnings, info))
}
