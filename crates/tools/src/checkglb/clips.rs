//! Clips: turning, sweeping and the checks a clip must pass.

use super::*;

/// The product of two quaternions (x, y, z, w).
fn qmul(a: [f64; 4], b: [f64; 4]) -> [f64; 4] {
    let [ax, ay, az, aw] = a;
    let [bx, by, bz, bw] = b;
    [
        aw * bx + ax * bw + ay * bz - az * by,
        aw * by - ax * bz + ay * bw + az * bx,
        aw * bz + ax * by - ay * bx + az * bw,
        aw * bw - ax * bx - ay * by - az * bz,
    ]
}

/// [(clip, node, axis)]: the parts a clip that goes round (`loops`: a loop, `idle`, a clock's hand) turns one
/// way about one axis all through (G.spin: a fan's blades, a wheel; not G.rock, which turns back), so they
/// sweep a whole circle as it repeats. `axis`: the turn's axis in the node's own frame (unit).
fn turning(g: &Glb, motions: &J) -> R<Vec<(String, usize, [f64; 3])>> {
    let mut out = Vec::new();
    let going = loops(motions)?;
    for a in g.gltf.get("animations").and_then(J::list).unwrap_or(&[]) {
        let name = a.get("name").map(J::key).unwrap_or_default();
        if !going.contains(&name) {
            continue;
        }
        for c in key(a, "channels")?.list().unwrap_or(&[]) {
            let target = key(c, "target")?;
            let node = target.opt("node");
            if target.get("path") != Some(&J::Str("rotation".into())) || node.is_none() {
                continue;
            }
            let node = target_node(node.unwrap());
            let keys = values(g, key(sampler(a, c)?, "output")?)?;
            let q = |k: &Vec<f64>| -> R<[f64; 4]> {
                k.get(..4)
                    .and_then(|k| k.try_into().ok())
                    .ok_or_else(|| "not enough values to unpack (expected 4)".into())
            };
            // Each step from one key to the next, in the part's own frame: a turn about one axis, one way.
            let mut steps = Vec::new();
            for (p, k) in keys.iter().zip(keys.iter().skip(1)) {
                let p = q(p)?;
                let st = qmul([-p[0], -p[1], -p[2], p[3]], q(k)?);
                steps.push(if st[3] >= 0.0 { st } else { st.map(|v| -v) });
            }
            let mut big: Option<[f64; 4]> = None;
            for st in &steps {
                if big.is_none_or(|b| hypot(&st[..3]) > hypot(&b[..3])) {
                    big = Some(*st);
                }
            }
            let Some(big) = big else { continue };
            let n = hypot(&big[..3]);
            if n < 1e-6 {
                continue;
            }
            let axis = [big[0] / n, big[1] / n, big[2] / n];
            let one_way = steps.iter().all(|st| {
                let along = 0.0 + st[0] * axis[0] + st[1] * axis[1] + st[2] * axis[2];
                let h = hypot(&st[..3]);
                along > -1e-6 && dist(&st[..3], &axis.map(|v| v * h)) < 1e-3
            });
            if one_way {
                out.push((name.clone(), node, axis));
            }
        }
    }
    Ok(out)
}

/// A part that goes round: its clip, its name, the floor-plan box it sweeps (x, depth), and how far it
/// reaches from its axis.
pub(super) struct Swept {
    pub(super) clip: String,
    pub(super) name: String,
    pub(super) x: (f64, f64),
    pub(super) z: (f64, f64),
    pub(super) radius: f64,
}

/// The floor-plan box each `turning` part sweeps, a whole circle round its pivot (in game axes), and how far
/// it reaches from its axis.
pub(super) fn swept(g: &Glb, motions: &J) -> R<Vec<Swept>> {
    let turns = turning(g, motions)?;
    if turns.is_empty() {
        return Ok(Vec::new());
    }
    let mut worlds = HashMap::new();
    let meshes = node_matrices(&g.gltf, None, Some(&mut worlds))?;
    let empty = J::Arr(Vec::new());
    let nodes = g.gltf.get("nodes").unwrap_or(&empty);
    let mut out = Vec::new();
    for (clip, node, axis) in turns {
        let (mut below, mut todo) = (HashSet::new(), vec![node]);
        while let Some(i) = todo.pop() {
            below.insert(i);
            for c in nodes
                .at(i as i64)
                .ok_or("list index out of range")?
                .get("children")
                .and_then(J::list)
                .unwrap_or(&[])
            {
                todo.push(node_index(nodes, c)?);
            }
        }
        let Some(w) = worlds.get(&node) else { continue };
        let w: Vec<f64> = w.iter().map(|v| v.f()).collect();
        let pivot = [w[12], w[13], w[14]];
        let a = [
            w[0] * axis[0] + w[4] * axis[1] + w[8] * axis[2],
            w[1] * axis[0] + w[5] * axis[1] + w[9] * axis[2],
            w[2] * axis[0] + w[6] * axis[1] + w[10] * axis[2],
        ];
        let n = hypot(&a);
        let a = a.map(|v| v / n);
        // A circle round the axis reaches r * sqrt(1 - a_k^2) either side of its centre along axis k.
        let spread = a.map(|v| f64::max(0.0, 1.0 - v * v).sqrt());
        let (mut lo, mut hi, mut radius) = ([f64::INFINITY; 3], [f64::NEG_INFINITY; 3], 0.0f64);
        for p in &meshes {
            if !below.contains(&p.node) {
                continue;
            }
            for prim in primitives(&g.gltf, &p.mesh)? {
                for v in positions(g, prim)? {
                    let q = applied(&p.m, &v);
                    let d = [q[0] - pivot[0], q[1] - pivot[1], q[2] - pivot[2]];
                    let along = 0.0 + d[0] * a[0] + d[1] * a[1] + d[2] * a[2];
                    let r = dist(&d, &a.map(|k| along * k));
                    if r > radius {
                        radius = r;
                    }
                    for k in 0..3 {
                        let centre = pivot[k] + along * a[k];
                        let (l, h) = (centre - r * spread[k], centre + r * spread[k]);
                        if l < lo[k] {
                            lo[k] = l;
                        }
                        if h > hi[k] {
                            hi[k] = h;
                        }
                    }
                }
            }
        }
        if lo[0] <= hi[0] {
            // glTF x -> game -x
            out.push(Swept {
                clip,
                name: node_name(nodes, node),
                x: (-hi[0], -lo[0]),
                z: (lo[2], hi[2]),
                radius,
            });
        }
    }
    Ok(out)
}

/// (errors, warnings) for a file's animations: moving parts turned or slid by node clips
/// (docs/design/objects.md, "Moving parts"), each starting at rest, each played by the entry's `motions`.
pub(super) fn check_clips(g: &Glb, entry: &J) -> R<(Vec<String>, Vec<String>)> {
    let (mut errors, mut warnings) = (Vec::new(), Vec::new());
    let empty = J::Arr(Vec::new());
    let animations = g.gltf.get("animations").and_then(J::list).unwrap_or(&[]);
    let nodes = g.gltf.get("nodes").unwrap_or(&empty);
    let rest = |path: &str| -> Option<Vec<J>> {
        match path {
            "rotation" => Some(vec![J::Int(0), J::Int(0), J::Int(0), J::Int(1)]),
            "translation" => Some(vec![J::Int(0), J::Int(0), J::Int(0)]),
            "scale" => Some(vec![J::Int(1), J::Int(1), J::Int(1)]),
            _ => None,
        }
    };
    let clip_name = |a: &J| a.get("name").cloned().unwrap_or(J::Str(String::new()));
    for a in animations {
        let name = repr(&clip_name(a));
        // When it starts and how its keys are read: the game plays a cubic spline's keys as straight lines.
        let (mut late, mut cubic) = (0.0_f64, false);
        for s in key(a, "samplers")?.list().unwrap_or(&[]) {
            if let Some(t) = first_value(g, key(s, "input")?, false)? {
                late = late.max(t[0]);
            }
            cubic |= s.get("interpolation") == Some(&J::Str("CUBICSPLINE".into()));
        }
        if late > CLIP_START {
            errors.push(format!(
                "clip {name} starts {} s in: the game holds it at rest until then, a pause each round of a loop (its first key at 0 s, as artkit writes a clip)",
                fixed(late, 3)
            ));
        }
        if cubic {
            warnings.push(strict("cubic", format!(
                "clip {name} has cubic-spline keys: the game plays them as straight lines between the keys (export it sampled, as artkit does)"
            )));
        }
        for c in key(a, "channels")?.list().unwrap_or(&[]) {
            let target = key(c, "target")?;
            let (path, node) = (target.get("path"), target.opt("node"));
            let at_rest = path.and_then(J::str).and_then(rest);
            if at_rest.is_none() || node.is_none() {
                errors.push(format!(
                    "clip {name} animates {} (only a node's rotation, translation or scale)",
                    path.map(str_of).unwrap_or("None".into())
                ));
                continue;
            }
            let path = path.and_then(J::str).unwrap();
            let first = first_value(g, key(sampler(a, c)?, "output")?, false)?;
            let node = node_index(nodes, node.unwrap())?;
            let n = nodes.at(node as i64).unwrap();
            let at_rest = match n.get(path) {
                Some(v) => v.list().ok_or("object of type has no len()")?.to_vec(),
                None => at_rest.unwrap(),
            };
            let away = match &first {
                None => true,
                Some(first) => {
                    let mut away = false;
                    for (k, r) in at_rest.iter().enumerate() {
                        let f = first.get(k).ok_or("tuple index out of range")?;
                        let r = r.num().ok_or("unsupported operand type(s) for -")?.f();
                        if (f - r).abs() > 1e-3 {
                            away = true;
                            break;
                        }
                    }
                    away
                }
            };
            if away {
                errors.push(format!("clip {name} starts {} away from where the file has it (the first key is the rest pose)", node_name(nodes, node)));
            }
            if path == "rotation" {
                let s = sampler(a, c)?;
                let fastest = turn_per_step(g, s)?;
                if fastest > TURN_STEP {
                    warnings.push(format!(
                        "clip {name} turns {} {}° in 1/60 s: the game blends its samples 1/60 s apart, which shrinks a part turning that fast (keep it under two turns a second)",
                        node_name(nodes, node),
                        fixed(fastest, 0)
                    ));
                }
            }
        }
    }
    // (A part inside one that moves, a lever on a door, moves with it and by its own clips too.)
    let names: BTreeSet<String> = animations.iter().map(|a| clip_name(a).key()).collect();
    let none = J::Obj(Vec::new());
    let motions = entry.get("motions").filter(|m| m.truthy()).unwrap_or(&none);
    errors.extend(motions_errors(motions));
    let mut plays = played(motions)?;
    plays.sort_by_key(|(clip, _)| clip.key());
    for (clip, by) in &plays {
        if !names.contains(&clip.key()) {
            errors.push(format!(
                "`motions` plays {} ({by}), which is no clip of the file",
                repr(clip)
            ));
        }
    }
    let played_keys: HashSet<String> = plays.iter().map(|(c, _)| c.key()).collect();
    for name in names.iter().filter(|n| !played_keys.contains(*n)) {
        warnings.push(strict(
            "unplayed",
            format!(
                "clip {} plays for no use (name it in the entry's `motions`)",
                repr_str(name)
            ),
        ));
    }
    // A use's clip plays as the use starts: one much longer than about 1 s reads as slow. Loops, ambient,
    // night, open and growth clips keep their own pace (a merry-go-round's round, a clock's minute).
    let going = loops(motions)?;
    let by_use: HashSet<String> = plays
        .iter()
        .filter(|(_, by)| by.starts_with("use "))
        .map(|(c, _)| c.key())
        .filter(|c| !going.contains(c))
        .collect();
    for a in animations {
        if !by_use.contains(&clip_name(a).key()) {
            continue;
        }
        let mut length: Option<f64> = None;
        for s in key(a, "samplers")?.list().unwrap_or(&[]) {
            let last = first_value(g, key(s, "input")?, true)?
                .map(|v| v[0])
                .unwrap_or(0.0);
            if length.is_none_or(|l| last > l) {
                length = Some(last);
            }
        }
        let length = length.ok_or("max() iterable argument is empty")?;
        if length > USE_CLIP {
            warnings.push(format!(
                "clip {} takes {} s, played by a use: keep a use's clip about 1 s (over 3 s, a loop or an ambient clip instead)",
                repr(key(a, "name")?),
                fixed(length, 1)
            ));
        }
    }
    // A loop goes round from rest to rest: each channel ends where it starts (a turn, either sign).
    // A turn by a whole fraction of a circle (a fan's four blades a quarter round) loops if the part
    // looks the same turned, and a move at an even speed (G.scroll: a belt's slats) if it looks the same
    // moved: warnings to check that.
    for a in animations {
        if !going.contains(&clip_name(a).key()) {
            continue;
        }
        for c in key(a, "channels")?.list().unwrap_or(&[]) {
            let s = sampler(a, c)?;
            let out = key(s, "output")?;
            let (Some(first), Some(last)) =
                (first_value(g, out, false)?, first_value(g, out, true)?)
            else {
                continue;
            };
            let target = key(c, "target")?;
            let node = node_name(nodes, node_index(nodes, key(target, "node")?)?);
            let clip = repr(key(a, "name")?);
            let path = target.get("path");
            if path == Some(&J::Str("rotation".into())) {
                let turned = turn_between(&first, &last);
                if turned < 0.1 {
                    continue;
                }
                let n = py::round_even(360.0 / turned);
                if (2.0..=12.0).contains(&n) && (turned * n - 360.0).abs() < 0.5 {
                    warnings.push(format!(
                        "loop {clip} ends {node} turned {}° from where it starts: right only if it looks the same turned (1/{} of a circle)",
                        fixed(turned, 0),
                        n as i64
                    ));
                    continue;
                }
            } else {
                let diffs: Vec<f64> = first
                    .iter()
                    .zip(&last)
                    .map(|(x, y)| (x - y).abs())
                    .collect();
                let most = diffs
                    .iter()
                    .cloned()
                    .reduce(|m, d| if d > m { d } else { m })
                    .ok_or("max() iterable argument is empty")?;
                if most <= 1e-3 {
                    continue;
                }
                if path == Some(&J::Str("translation".into())) && even(g, s)? {
                    // A scroll (G.scroll: a belt's slats one slat on, its ends hidden) looks the same moved.
                    let part = node_index(nodes, key(target, "node")?)?;
                    if let Some(landed) = lands_on_itself(g, part, &first, &last)?
                        && landed < SCROLLED
                    {
                        errors.push(format!(
                            "loop {clip} ends {node} moved {} cm from where it starts, and only {} of it lands where it was: a scroll moves by whole repeats (a belt's slats one slat on, its ends hidden)",
                            fixed(dist(&first, &last) * 100.0, 0),
                            percent(landed)
                        ));
                    }
                    continue;
                }
            }
            errors.push(format!("loop {clip} ends {node} away from where it starts ({}): a loop goes from rest to rest (G.spin, G.rock, G.scroll)", str_of(key(target, "path")?)));
        }
    }
    Ok((errors, warnings))
}

/// The turn (degrees) from one rotation (a quaternion, x y z w) to another.
fn turn_between(a: &[f64], b: &[f64]) -> f64 {
    let dot = a.iter().zip(b).fold(0.0, |s, (x, y)| s + x * y);
    2.0 * f64::min(1.0, dot.abs()).acos() * (180.0 / PI)
}

/// The most a rotation channel turns its node between two of the game's samples, 1/60 s apart (degrees).
fn turn_per_step(g: &Glb, sampler: &J) -> R<f64> {
    let times = values(g, key(sampler, "input")?)?;
    let keys = values(g, key(sampler, "output")?)?;
    let mut fastest = 0.0_f64;
    for k in 1..times.len().min(keys.len()) {
        let dt = times[k][0] - times[k - 1][0];
        if dt <= 1e-6 || keys[k].len() < 4 || keys[k - 1].len() < 4 {
            continue;
        }
        fastest = fastest.max(turn_between(&keys[k - 1], &keys[k]) / dt / 60.0);
    }
    Ok(fastest)
}

/// [(mesh, faces)] for the meshes with faces over `FLIPPED_AREA` whose stored normals all point against the
/// face's winding: lit as if from behind (dark where the light falls), the way a bevel's arc miter turned a
/// frame's inner corners round. In each mesh's own space.
pub(super) fn flipped_faces(g: &Glb) -> R<Vec<(String, usize)>> {
    let mut out = Vec::new();
    for mesh in g.gltf.get("meshes").and_then(J::list).unwrap_or(&[]) {
        let mut bad = 0;
        for prim in key(mesh, "primitives")?.list().unwrap_or(&[]) {
            if !prim
                .get("mode")
                .is_none_or(|m| m.num().is_some_and(|n| n.f() == 4.0))
            {
                continue;
            }
            let at = key(prim, "attributes")?;
            let Some(normal) = at.get("NORMAL") else {
                continue;
            };
            let (Some(pos), Some(nrm)) = (accessor(g, key(at, "POSITION")?)?, accessor(g, normal)?)
            else {
                continue;
            };
            let idx: Vec<usize> = match prim.get("indices") {
                Some(i) => match accessor(g, i)? {
                    Some(v) => v.iter().map(|x| x[0].f() as usize).collect(),
                    None => continue,
                },
                None => (0..pos.len()).collect(),
            };
            let p = |i: usize| -> Option<[f64; 3]> {
                let v = pos.get(i)?;
                Some([v[0].f(), v[1].f(), v[2].f()])
            };
            for t in idx.as_chunks::<3>().0 {
                let (Some(a), Some(b), Some(c)) = (p(t[0]), p(t[1]), p(t[2])) else {
                    continue;
                };
                let (e1, e2) = (
                    [b[0] - a[0], b[1] - a[1], b[2] - a[2]],
                    [c[0] - a[0], c[1] - a[1], c[2] - a[2]],
                );
                let f = [
                    e1[1] * e2[2] - e1[2] * e2[1],
                    e1[2] * e2[0] - e1[0] * e2[2],
                    e1[0] * e2[1] - e1[1] * e2[0],
                ];
                let len = py::hypot(&f);
                if len / 2.0 < FLIPPED_AREA {
                    continue;
                }
                let against = t.iter().all(|&i| {
                    nrm.get(i).is_some_and(|n| {
                        n.len() >= 3
                            && (n[0].f() * f[0] + n[1].f() * f[1] + n[2].f() * f[2]) / len < -0.5
                    })
                });
                bad += against as usize;
            }
        }
        if bad > 0 {
            out.push((
                mesh.get("name").and_then(J::str).unwrap_or("").to_string(),
                bad,
            ));
        }
    }
    Ok(out)
}

/// The share of a scroll's part that must land where some of it was, moved from its first key to its last (a belt of
/// N slats one slat on: all but one of them, (N - 1) / N).
const SCROLLED: f64 = 0.5;

/// The share of part `node`'s corners (the meshes it moves) that land within a millimetre of one of its corners at
/// rest, moved from translation `first` to `last`; None where its positions can't be read here (compressed).
pub(super) fn lands_on_itself(g: &Glb, node: usize, first: &[f64], last: &[f64]) -> R<Option<f64>> {
    let corners = |t: &[f64]| -> R<Option<Vec<[f64; 3]>>> {
        let pose: Pose = HashMap::from([(
            (node, "translation"),
            t.iter().map(|v| Num::F(*v)).collect(),
        )]);
        let placed: Vec<Placed> = node_matrices(&g.gltf, Some(&pose), None)?
            .into_iter()
            .filter(|p| p.part == Some(node))
            .collect();
        let add = |a: [f64; 3], e: [f64; 3]| [a[0] + e[0], a[1] + e[1], a[2] + e[2]];
        Ok(triangles(g, &placed, &|_| Ok(true))?.map(|tris| {
            tris.iter()
                .flat_map(|t| [t.a, add(t.a, t.e1), add(t.a, t.e2)])
                .collect()
        }))
    };
    let (Some(rest), Some(moved)) = (corners(first)?, corners(last)?) else {
        return Ok(None);
    };
    if moved.is_empty() {
        return Ok(None);
    }
    let cell = |p: &[f64; 3]| p.map(|v| (v * 1000.0).round() as i64);
    let at: HashSet<[i64; 3]> = rest.iter().map(cell).collect();
    let near = |[x, y, z]: [i64; 3]| {
        (-1..=1).any(|i| (-1..=1).any(|j| (-1..=1).any(|k| at.contains(&[x + i, y + j, z + k]))))
    };
    let landed = moved.iter().filter(|p| near(cell(p))).count();
    Ok(Some(landed as f64 / moved.len() as f64))
}
