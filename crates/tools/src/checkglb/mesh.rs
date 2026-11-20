//! Meshes: primitives, samplers, finishes and the clips a model plays.
//!
//! Split from `checkglb.rs` (see `mod.rs`).

use super::*;

/// The primitives of a mesh (`gltf['meshes'][i]['primitives']`).
pub(crate) fn primitives<'a>(gltf: &'a J, mesh: &J) -> R<&'a [J]> {
    Ok(key(item(gltf, "meshes", mesh)?, "primitives")?
        .list()
        .unwrap_or(&[]))
}

/// The positions of a primitive: every vertex, or where they can't be read here (compressed), its box's corners.
pub(crate) fn positions(g: &Glb, prim: &J) -> R<Vec<Vec<Num>>> {
    let index = key(key(prim, "attributes")?, "POSITION")?;
    if let Some(pos) = accessor(g, index)? {
        return Ok(pos);
    }
    let (lo, hi) = box_of(item(&g.gltf, "accessors", index)?)?;
    let mut out = Vec::with_capacity(8);
    for x in [lo[0], hi[0]] {
        for y in [lo[1], hi[1]] {
            for z in [lo[2], hi[2]] {
                out.push(vec![x, y, z]);
            }
        }
    }
    Ok(out)
}

/// Whether a primitive's vertex colour (COLOR_0) is anything but grey. The game reads it as the shading baked into
/// the model's creases (artkit's shade.rs) and multiplies it into the part's colour, so a colour in it would tint
/// the part. One it can't read here (compressed) passes.
pub(crate) fn coloured(g: &Glb, prim: &J) -> R<bool> {
    let Some(index) = key(prim, "attributes")?.get("COLOR_0") else {
        return Ok(false);
    };
    let Some(rows) = accessor(g, index)? else {
        return Ok(false);
    };
    Ok(rows.iter().any(|v| {
        v.len() >= 3 && ((v[0].f() - v[1].f()).abs() > 0.01 || (v[0].f() - v[2].f()).abs() > 0.01)
    }))
}

/// Whether an accessor's values are all the same (or can't be read here: compressed): a plain part's UVs, zero
/// as built, (0, 1) once the writer has turned v round.
pub(crate) fn constant(g: &Glb, index: &J) -> R<bool> {
    Ok(accessor(g, index)?.is_none_or(|rows| {
        rows.first().is_none_or(|first| {
            rows.iter()
                .all(|r| r.iter().zip(first).all(|(a, b)| a.f() == b.f()))
        })
    }))
}

/// An accessor's `min` and `max`.
pub(crate) fn box_of(acc: &J) -> R<(Vec<Num>, Vec<Num>)> {
    let lo = nums(key(acc, "min")?, 0)?;
    let hi = nums(key(acc, "max")?, 0)?;
    if lo.len() < 3 || hi.len() < 3 {
        return Err("list index out of range".into());
    }
    Ok((lo, hi))
}

/// Every element of a float accessor as given by `values`, as an index into a list: `a['samplers'][c['sampler']]`.
pub(crate) fn sampler<'a>(a: &'a J, c: &J) -> R<&'a J> {
    index(key(a, "samplers")?, key(c, "sampler")?)
}

/// Whether a sampler moves at an even speed (each key as far on from the last, per second).
pub(crate) fn even(g: &Glb, sampler: &J) -> R<bool> {
    let times: Vec<f64> = values(g, key(sampler, "input")?)?
        .iter()
        .map(|t| t[0])
        .collect();
    let keys = values(g, key(sampler, "output")?)?;
    if keys.len() < 3 {
        return Ok(false);
    }
    let (t0, t1) = (
        *times.first().ok_or("list index out of range")?,
        *times.last().unwrap(),
    );
    if t1 <= t0 {
        return Ok(false);
    }
    let rate: Vec<f64> = keys[0]
        .iter()
        .zip(keys.last().unwrap())
        .map(|(a, b)| (b - a) / (t1 - t0))
        .collect();
    for (k, t) in keys.iter().zip(&times) {
        for i in 0..rate.len() {
            if (k.get(i).ok_or("tuple index out of range")? - keys[0][i] - rate[i] * (t - t0)).abs()
                > 1e-3
            {
                return Ok(false);
            }
        }
    }
    Ok(true)
}

/// A material named for its finish (`stone@matte`, `tunic@fabric#body`) maps itself to it (models.ts
/// `namedFinish`; a tint slot `#body`, `#skin` or `#hair` in the name aside).
pub(crate) fn named_finish(name: &str) -> Option<J> {
    // re.sub(r'#(body|skin|hair)(?=@|$)', '', name)
    let mut s = String::with_capacity(name.len());
    let mut rest = name;
    while !rest.is_empty() {
        let slot = ["#body", "#skin", "#hair"]
            .iter()
            .find(|w| rest.starts_with(**w));
        if let Some(w) = slot {
            let after = &rest[w.len()..];
            if after.is_empty() || after.starts_with('@') || after == "\n" {
                rest = after;
                continue;
            }
        }
        let c = rest.chars().next().unwrap();
        s.push(c);
        rest = &rest[c.len_utf8()..];
    }
    let (head, tail) = s.rsplit_once('@')?;
    // tail.isalpha() and tail.islower(): letters, some cased, none upper (or title) case.
    let title = |c: char| matches!(c as u32, 0x1c5 | 0x1c8 | 0x1cb | 0x1f2 | 0x1f88..=0x1f8f | 0x1f98..=0x1f9f | 0x1fa8..=0x1faf | 0x1fbc | 0x1fcc | 0x1ffc);
    let lower = !tail.is_empty()
        && tail.chars().all(char::is_alphabetic)
        && tail.chars().any(char::is_lowercase)
        && !tail.chars().any(|c| c.is_uppercase() || title(c));
    if !head.is_empty() && lower {
        Some(J::Obj(vec![(
            "finish".into(),
            J::Str(format!("material.finish.{tail}")),
        )]))
    } else {
        None
    }
}

/// [(use, {clip, hold, delay, every, slot, out, over})] for each clip an entry's `motions.uses` plays (a clip,
/// {clip, ...} or a list of them).
fn uses(motions: &J) -> Vec<(String, J)> {
    let mut out = Vec::new();
    for (name, value) in motions
        .get("uses")
        .filter(|u| u.truthy())
        .and_then(J::entries)
        .unwrap_or(&[])
    {
        let list = match value {
            J::Arr(l) => l.clone(),
            v => vec![v.clone()],
        };
        for u in list {
            out.push((
                name.clone(),
                if matches!(u, J::Obj(_)) {
                    u
                } else {
                    J::Obj(vec![("clip".into(), u)])
                },
            ));
        }
    }
    out
}

/// The `motions` kinds naming one clip each (`played`; `cargo review`'s motions column).
pub(crate) const CLIP_KINDS: [&str; 7] =
    ["open", "idle", "night", "growth", "ride", "running", "burn"];

/// {clip: what plays it} for an entry's `motions` (in the order they're first named): `uses`, `loops`,
/// `open`, `idle`, `night`, `growth`, `ride` (the clip whose part carries its users), `running` (round and
/// round while a machine runs on its own), `burn` (round while a fire burns) and `clock` (`hour`, `minute`:
/// hands set by the time).
pub(crate) fn played(motions: &J) -> R<Vec<(J, String)>> {
    let mut out: Vec<(J, String)> = Vec::new();
    let mut add = |clip: &J, by: String| {
        if !out.iter().any(|(c, _)| c == clip) {
            out.push((clip.clone(), by));
        }
    };
    for (name, u) in uses(motions) {
        add(key(&u, "clip")?, format!("use {}", repr_str(&name)));
    }
    for (name, clip) in motions
        .get("loops")
        .filter(|l| l.truthy())
        .and_then(J::entries)
        .unwrap_or(&[])
    {
        add(clip, format!("loop {}", repr_str(name)));
    }
    for kind in CLIP_KINDS {
        if let Some(clip) = motions.get(kind).filter(|c| c.truthy()) {
            add(clip, kind.into());
        }
    }
    let clock = motions.get("clock").filter(|c| c.truthy());
    for hand in ["hour", "minute"] {
        if let Some(clip) = clock.and_then(|c| c.get(hand)).filter(|c| c.truthy()) {
            add(clip, format!("clock {hand}"));
        }
    }
    Ok(out)
}

/// The clips of `motions` that go round, or from rest to rest (and so must end where they start): loops,
/// `idle`, `running`, `burn`, a use's clip that repeats (`every`) and a clock's hands (a whole turn). As dict keys.
pub(crate) fn loops(motions: &J) -> R<HashSet<String>> {
    let mut out = HashSet::new();
    for (_, clip) in motions
        .get("loops")
        .filter(|l| l.truthy())
        .and_then(J::entries)
        .unwrap_or(&[])
    {
        out.insert(clip.key());
    }
    for kind in ["idle", "running", "burn"] {
        if let Some(clip) = motions.get(kind).filter(|c| c.truthy()) {
            out.insert(clip.key());
        }
    }
    for (_, u) in uses(motions) {
        if u.get("every").is_some_and(J::truthy) {
            out.insert(key(&u, "clip")?.key());
        }
    }
    let clock = motions.get("clock").filter(|c| c.truthy());
    for hand in ["hour", "minute"] {
        if let Some(clip) = clock.and_then(|c| c.get(hand)).filter(|c| c.truthy()) {
            out.insert(clip.key());
        }
    }
    Ok(out)
}

/// What's wrong with the fields of an entry's `motions` (objectMotion.ts): a use's `delay` and `hold` are
/// seconds (0 or more), `every` and `out` more than 0, `slot` a slot number, `over` 1 or -1 (along with the use,
/// forward or back); `clock` names its hands' clips.
pub(crate) fn motions_errors(motions: &J) -> Vec<String> {
    let mut errors = Vec::new();
    for (name, u) in uses(motions) {
        let name = repr_str(&name);
        for k in ["hold", "delay"] {
            if let Some(v) = u.get(k)
                && !v.num().is_some_and(|n| n.f() >= 0.0)
            {
                errors.push(format!(
                    "`motions.uses` {name}: {k} {} is no number of seconds",
                    repr(v)
                ));
            }
        }
        for k in ["every", "out"] {
            if let Some(v) = u.get(k)
                && !v.num().is_some_and(|n| n.f() > 0.0)
            {
                errors.push(format!(
                    "`motions.uses` {name}: {k} {} is no number of seconds above 0",
                    repr(v)
                ));
            }
        }
        if let Some(over) = u.get("over")
            && !matches!(over, J::Int(1 | -1))
        {
            errors.push(format!(
                "`motions.uses` {name}: over {} is neither 1 nor -1",
                repr(over)
            ));
        }
        // (An int, not a bool.)
        if let Some(slot) = u.get("slot")
            && !matches!(slot, J::Int(i) if *i >= 0)
        {
            errors.push(format!(
                "`motions.uses` {name}: slot {} is no slot (0, 1, ...)",
                repr(slot)
            ));
        }
        let mut unknown: Vec<&str> = u
            .entries()
            .unwrap_or(&[])
            .iter()
            .map(|(k, _)| k.as_str())
            .filter(|k| !["clip", "hold", "delay", "every", "slot", "out", "over"].contains(k))
            .collect();
        unknown.sort();
        if !unknown.is_empty() {
            errors.push(format!(
                "`motions.uses` {name}: unknown {}",
                unknown.join(", ")
            ));
        }
    }
    if let Some(clock) = motions.opt("clock") {
        let named = |h: &str| clock.get(h).is_some_and(J::truthy);
        let odd = clock.entries().is_some_and(|e| {
            e.iter()
                .any(|(k, _)| !["hour", "minute", "at"].contains(&k.as_str()))
        });
        if !matches!(clock, J::Obj(_)) || !(named("hour") || named("minute")) || odd {
            errors.push(
                "`motions.clock` names its hands' clips: {\"hour\", \"minute\", \"at\"}".into(),
            );
        } else if let Some(at) = clock.get("at")
            && at.num().is_none()
        {
            errors.push(format!(
                "`motions.clock.at` {} is no time (minutes past twelve)",
                repr(at)
            ));
        }
    }
    errors
}
