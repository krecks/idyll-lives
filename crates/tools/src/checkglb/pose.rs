//! Poses: bounds, reach, triangles and what a model warns about.

use super::*;

/// The parts a clip scales up from rest (G.grow: darts, a pot thrown, water), at their largest key, so what
/// is only small or flattened at rest is measured as it shows. Not by the clips `held` names (`open`,
/// `night`: a state of its own, a sofa bed pulled out; the model is at rest).
pub(super) fn grown(g: &Glb, held: &[Option<J>]) -> R<Pose> {
    let empty = J::Arr(Vec::new());
    let nodes = g.gltf.get("nodes").unwrap_or(&empty);
    let mut out: Pose = HashMap::new();
    let abs_sum = |k: &[Num]| {
        k.iter()
            .fold(Num::I(0), |s, v| s + if v.f() < 0.0 { -*v } else { *v })
            .f()
    };
    for a in g.gltf.get("animations").and_then(J::list).unwrap_or(&[]) {
        if held.contains(&a.opt("name").cloned()) {
            continue;
        }
        for c in key(a, "channels")?.list().unwrap_or(&[]) {
            let target = key(c, "target")?;
            let node = target.opt("node");
            if target.get("path") != Some(&J::Str("scale".into())) || node.is_none() {
                continue;
            }
            let node = node_index(nodes, node.unwrap())?;
            let rest = match nodes.at(node as i64).unwrap().get("scale") {
                Some(s) => nums(s, 0)?,
                None => vec![Num::I(1); 3],
            };
            let keys: Vec<Vec<Num>> = values(g, key(sampler(a, c)?, "output")?)?
                .into_iter()
                .map(|k| k.into_iter().map(Num::F).collect())
                .collect();
            let mut best: Option<&Vec<Num>> = None;
            for k in &keys {
                if best.is_none_or(|b| abs_sum(k) > abs_sum(b)) {
                    best = Some(k);
                }
            }
            let best = best.ok_or("max() iterable argument is empty")?;
            if best
                .iter()
                .zip(&rest)
                .any(|(b, r)| b.f().abs() > 1.5 * r.f().abs() + 1e-6)
                && out
                    .get(&(node, "scale"))
                    .is_none_or(|have| abs_sum(best) > abs_sum(have))
            {
                out.insert((node, "scale"), best.clone());
            }
        }
    }
    Ok(out)
}

/// [(clip, what plays it, pose)]: each clip that opens or uses the model (a use's, `open`, `night`) at each of its
/// keys (a door swung open, and a `BackOut`'s overshoot on the way), on top of `rest`.
pub(super) fn opened(g: &Glb, motions: &J, rest: &Pose) -> R<Vec<(String, String, Pose)>> {
    let empty = J::Arr(Vec::new());
    let nodes = g.gltf.get("nodes").unwrap_or(&empty);
    let mut out = Vec::new();
    for (clip, by) in played(motions)? {
        if !(by.starts_with("use ") || by == "open" || by == "night") {
            continue;
        }
        let Some(a) = g
            .gltf
            .get("animations")
            .and_then(J::list)
            .unwrap_or(&[])
            .iter()
            .find(|a| a.get("name") == Some(&clip))
        else {
            continue;
        };
        let mut tracks = Vec::new();
        let mut times: Vec<f64> = Vec::new();
        for c in key(a, "channels")?.list().unwrap_or(&[]) {
            let target = key(c, "target")?;
            let path = match target.get("path").and_then(J::str) {
                Some("rotation") => "rotation",
                Some("translation") => "translation",
                Some("scale") => "scale",
                _ => continue,
            };
            let Some(node) = target.opt("node") else {
                continue;
            };
            let s = sampler(a, c)?;
            let track = super::near::Track {
                times: values(g, key(s, "input")?)?
                    .into_iter()
                    .map(|t| t[0])
                    .collect(),
                keys: values(g, key(s, "output")?)?,
                step: s.get("interpolation") == Some(&J::Str("STEP".into())),
            };
            times.extend(&track.times);
            tracks.push((node_index(nodes, node)?, path, track));
        }
        times.sort_by(f64::total_cmp);
        times.dedup();
        for t in times {
            let mut pose = rest.clone();
            for (node, path, track) in &tracks {
                if let Some(v) = track.at(t, *path == "rotation") {
                    pose.insert((*node, path), v.into_iter().map(Num::F).collect());
                }
            }
            out.push((clip.key(), by.clone(), pose));
        }
    }
    Ok(out)
}

pub(super) type Bounds = ([Num; 3], [Num; 3]);
/// Each moving part's bounds (None: the still body), in the order they're met.
type PartBounds = Vec<(Option<usize>, Bounds)>;

/// A model's nodes drawn but not part of its own size (`artkit::parts::aside`): a desk's chair, a chest of
/// drawers' picture, the blocks left out in front of a toy box, the flag on a climbing frame, the block and
/// bottle beside a rolled mat. The game draws them; `model()` leaves them out of the size it measures.
pub const ASIDE: &str = "_aside";

/// The nodes of the file named as `aside` (their names end with `ASIDE`).
pub(super) fn aside_nodes(gltf: &J) -> HashSet<usize> {
    gltf.get("nodes")
        .and_then(J::list)
        .unwrap_or(&[])
        .iter()
        .enumerate()
        .filter(|(_, n)| {
            n.get("name")
                .map(J::key)
                .is_some_and(|name| name.ends_with(ASIDE))
        })
        .map(|(i, _)| i)
        .collect()
}

/// The file's bounds in game axes (glTF x -> game -x, as `render/pc/models.ts` bakes a file), from
/// each primitive's box through its node, and each moving part's (None: the still body), in the order
/// they're met; the nodes in `skip` left out.
pub(super) fn bounds(gltf: &J, pose: &Pose, skip: &HashSet<usize>) -> R<(Bounds, PartBounds)> {
    let accessors = gltf.get("accessors").cloned().unwrap_or(J::Arr(Vec::new()));
    let (inf, ninf) = (Num::F(f64::INFINITY), Num::F(f64::NEG_INFINITY));
    let (mut lo, mut hi) = ([inf; 3], [ninf; 3]);
    let mut parts: PartBounds = Vec::new();
    for p in node_matrices(gltf, Some(pose), None)? {
        if skip.contains(&p.node) {
            continue;
        }
        let at = match parts.iter().position(|(part, _)| *part == p.part) {
            Some(at) => at,
            None => {
                parts.push((p.part, ([inf; 3], [ninf; 3])));
                parts.len() - 1
            }
        };
        for prim in primitives(gltf, &p.mesh)? {
            let (bl, bh) = box_of(index(
                &accessors,
                key(key(prim, "attributes")?, "POSITION")?,
            )?)?;
            for cx in [bl[0], bh[0]] {
                for cy in [bl[1], bh[1]] {
                    for cz in [bl[2], bh[2]] {
                        let mut q = apply(&p.m, &[cx, cy, cz]);
                        q[0] = -q[0];
                        let (plo, phi) = &mut parts[at].1;
                        for k in 0..3 {
                            lo[k] = py::min(lo[k], q[k]);
                            hi[k] = py::max(hi[k], q[k]);
                            plo[k] = py::min(plo[k], q[k]);
                            phi[k] = py::max(phi[k], q[k]);
                        }
                    }
                }
            }
        }
    }
    Ok(((lo, hi), parts))
}

/// How far the model reaches from its origin across the floor (m), its parts where `pose` puts them (`grown`:
/// as it shows): its farthest vertex's hypot(x, depth) (a round tray reaches its radius, a square one its
/// corners), or where the positions can't be read here (compressed), its boxes' farthest corner.
fn reach(g: &Glb, pose: &Pose) -> R<f64> {
    let mut far = 0.0f64;
    for p in node_matrices(&g.gltf, Some(pose), None)? {
        for prim in primitives(&g.gltf, &p.mesh)? {
            for v in positions(g, prim)? {
                let q = applied(&p.m, &v);
                let r = hypot(&[q[0], q[2]]);
                if r > far {
                    far = r;
                }
            }
        }
    }
    Ok(far)
}

/// `x ** 2` as Python has it: the C library's `pow`, which isn't always `x * x` (macOS's differs in the last
/// bit for about 1 in 1000). (`black_box`: LLVM would make `pow(x, 2.0)` a multiplication.)
fn square(x: f64) -> f64 {
    x.abs().powf(std::hint::black_box(2.0))
}

/// An opaque triangle of the model at rest, with the node it belongs to.
pub(super) struct Tri {
    node: usize,
    pub(super) a: [f64; 3],
    pub(super) e1: [f64; 3],
    pub(super) e2: [f64; 3],
    pub(super) nrm: [f64; 3],
}

/// Every face the line through p along d crosses (Möller-Trumbore), in the order given: (how far along d,
/// negative behind p; the face's normal).
pub(super) fn hits<'a>(
    p: &'a [f64; 3],
    d: &'a [f64; 3],
    faces: impl IntoIterator<Item = &'a Tri>,
) -> impl Iterator<Item = (f64, &'a [f64; 3])> {
    faces.into_iter().filter_map(move |f| {
        let (a, e1, e2) = (&f.a, &f.e1, &f.e2);
        let h = [
            d[1] * e2[2] - d[2] * e2[1],
            d[2] * e2[0] - d[0] * e2[2],
            d[0] * e2[1] - d[1] * e2[0],
        ];
        let det = e1[0] * h[0] + e1[1] * h[1] + e1[2] * h[2];
        if det.abs() < 1e-12 {
            return None;
        }
        let s = [p[0] - a[0], p[1] - a[1], p[2] - a[2]];
        let u = (s[0] * h[0] + s[1] * h[1] + s[2] * h[2]) / det;
        #[allow(clippy::manual_range_contains)] // (not the same for a NaN)
        if u < 0.0 || u > 1.0 {
            return None;
        }
        let q = [
            s[1] * e1[2] - s[2] * e1[1],
            s[2] * e1[0] - s[0] * e1[2],
            s[0] * e1[1] - s[1] * e1[0],
        ];
        let v = (d[0] * q[0] + d[1] * q[1] + d[2] * q[2]) / det;
        if v < 0.0 || u + v > 1.0 {
            return None;
        }
        Some(((e2[0] * q[0] + e2[1] * q[1] + e2[2] * q[2]) / det, &f.nrm))
    })
}

/// The nearest face each way along d from p: [forward, backward], each (distance, whether it is a face leaving
/// a solid that way).
fn cast(p: &[f64; 3], d: &[f64; 3], faces: &[&Tri]) -> [(f64, bool); 2] {
    let mut near = [(f64::INFINITY, false), (f64::INFINITY, false)];
    for (t, nrm) in hits(p, d, faces.iter().copied()) {
        let (slot, way) = if t > 0.0 { (0, 1.0) } else { (1, -1.0) };
        if 1e-4 < t.abs() && t.abs() < near[slot].0 {
            near[slot] = (
                t.abs(),
                (nrm[0] * d[0] + nrm[1] * d[1] + nrm[2] * d[2]) * way > 0.0,
            );
        }
    }
    near
}

/// The triangles of the placed meshes' primitives that `keep` keeps (by their material index), with the node
/// each belongs to and its normal from the winding; None if some positions can't be read here (compressed).
pub(super) fn triangles(
    g: &Glb,
    placed: &[Placed],
    keep: &dyn Fn(Option<&J>) -> R<bool>,
) -> R<Option<Vec<Tri>>> {
    let det = |m: &[f64]| {
        m[0] * (m[5] * m[10] - m[9] * m[6]) - m[4] * (m[1] * m[10] - m[9] * m[2])
            + m[8] * (m[1] * m[6] - m[5] * m[2])
    };
    let mut tris = Vec::new();
    for p in placed {
        let mf: Vec<f64> = p.m.iter().map(|v| v.f()).collect();
        let sign = if det(&mf) >= 0.0 { 1.0 } else { -1.0 };
        for prim in primitives(&g.gltf, &p.mesh)? {
            let triangles = prim
                .get("mode")
                .is_none_or(|m| m.num().is_some_and(|n| n.f() == 4.0));
            if !triangles || !keep(prim.opt("material"))? {
                continue;
            }
            let pos = accessor(g, key(key(prim, "attributes")?, "POSITION")?)?;
            let idx = match prim.get("indices") {
                Some(i) => accessor(g, i)?,
                None => Some(
                    (0..pos.as_ref().map_or(0, |p| p.len()))
                        .map(|k| vec![Num::I(k as i64)])
                        .collect(),
                ),
            };
            let (Some(pos), Some(idx)) = (pos, idx) else {
                return Ok(None);
            };
            let world: Vec<[f64; 3]> = pos.iter().map(|v| applied(&p.m, v)).collect();
            let corner = |k: usize| -> R<[f64; 3]> {
                let i = idx[k][0];
                let i = match i {
                    Num::I(i) => i,
                    Num::F(_) => {
                        return Err("list indices must be integers or slices, not float".into());
                    }
                };
                let i = if i < 0 { i + world.len() as i64 } else { i };
                world
                    .get(i as usize)
                    .copied()
                    .ok_or_else(|| "list index out of range".into())
            };
            let mut k = 0;
            while k + 2 < idx.len() {
                let (a, b, c) = (corner(k)?, corner(k + 1)?, corner(k + 2)?);
                let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
                let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
                let nrm = [
                    sign * (e1[1] * e2[2] - e1[2] * e2[1]),
                    sign * (e1[2] * e2[0] - e1[0] * e2[2]),
                    sign * (e1[0] * e2[1] - e1[1] * e2[0]),
                ];
                tris.push(Tri {
                    node: p.node,
                    a,
                    e1,
                    e2,
                    nrm,
                });
                k += 3;
            }
        }
    }
    Ok(Some(tris))
}

/// [(part, share of it in view)] for the parts flattened to nothing at rest (a node scaled under 0.01
/// along an axis, G.grow from 0.001): it still shows as a flat shape, if it's 3 cm across or more, where its
/// flattened plane isn't inside something opaque (`opaque(material index)`), as the dartboard's flights did
/// as crosses on its face. Each of its points is looked from along the flattened axis both ways: inside a
/// solid, the nearest face either way is one leaving it (its normal, from the winding, pointing on).
pub(super) fn flattened_in_view(
    g: &Glb,
    opaque: &dyn Fn(Option<&J>) -> R<bool>,
) -> R<Vec<(String, f64)>> {
    let empty = J::Arr(Vec::new());
    let nodes = g.gltf.get("nodes").unwrap_or(&empty);
    let mut flat: BTreeMap<usize, usize> = BTreeMap::new();
    for i in moved_nodes(&g.gltf)? {
        let n = nodes.at(i as i64).ok_or("list index out of range")?;
        let scale = match n.get("scale") {
            Some(s) => nums(s, 0)?,
            None => vec![Num::I(1); 3],
        };
        let smallest = scale
            .iter()
            .map(|v| v.f().abs())
            .fold(f64::INFINITY, |m, v| if v < m { v } else { m });
        if n.has("mesh") && smallest < 0.01 {
            if scale.len() < 3 {
                return Err("list index out of range".into());
            }
            let mut axis = 0;
            for k in 1..3 {
                if scale[k].f().abs() < scale[axis].f().abs() {
                    axis = k;
                }
            }
            flat.insert(i, axis);
        }
    }
    if flat.is_empty() {
        return Ok(Vec::new());
    }
    let placed = node_matrices(&g.gltf, None, None)?;
    // Every opaque triangle of the model at rest, with the node it belongs to.
    let Some(tris) = triangles(g, &placed, opaque)? else {
        return Ok(Vec::new());
    };

    let mut out = Vec::new();
    for (&node, &axis) in &flat {
        let Some(p) = placed.iter().find(|p| p.node == node) else {
            return Err("StopIteration".into());
        };
        let m: Vec<f64> = p.m.iter().map(|v| v.f()).collect();
        let d = [m[axis * 4], m[axis * 4 + 1], m[axis * 4 + 2]];
        let length = (0.0 + d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
        let length = if length == 0.0 { 1.0 } else { length };
        let d = d.map(|v| v / length);
        let mut points = Vec::new();
        for prim in primitives(&g.gltf, key(nodes.at(node as i64).unwrap(), "mesh")?)? {
            let Some(pos) = accessor(g, key(key(prim, "attributes")?, "POSITION")?)? else {
                return Ok(out);
            };
            points.extend(pos.iter().map(|v| applied(&p.m, v)));
        }
        if points.is_empty() {
            continue;
        }
        // Its flat shape's two sides (across its plane, along its own main lines): one under 3 cm across, or
        // a sliver under 1 cm (a stream's end, a painting's edge before it spreads), hardly shows.
        let u0 = if d[1].abs() < 0.9 {
            [0.0, 1.0, 0.0]
        } else {
            [1.0, 0.0, 0.0]
        };
        let along = 0.0 + u0[0] * d[0] + u0[1] * d[1] + u0[2] * d[2];
        let u = [
            u0[0] - d[0] * along,
            u0[1] - d[1] * along,
            u0[2] - d[2] * along,
        ];
        let ul = (0.0 + u[0] * u[0] + u[1] * u[1] + u[2] * u[2]).sqrt();
        let u = u.map(|v| v / ul);
        let w = [
            d[1] * u[2] - d[2] * u[1],
            d[2] * u[0] - d[0] * u[2],
            d[0] * u[1] - d[1] * u[0],
        ];
        let flat2: Vec<[f64; 2]> = points
            .iter()
            .map(|q| {
                [
                    0.0 + q[0] * u[0] + q[1] * u[1] + q[2] * u[2],
                    0.0 + q[0] * w[0] + q[1] * w[1] + q[2] * w[2],
                ]
            })
            .collect();
        let count = flat2.len() as f64;
        let mu = [
            flat2.iter().fold(0.0, |s, c| s + c[0]) / count,
            flat2.iter().fold(0.0, |s, c| s + c[1]) / count,
        ];
        let cxx = flat2.iter().fold(0.0, |s, c| s + square(c[0] - mu[0]));
        let cyy = flat2.iter().fold(0.0, |s, c| s + square(c[1] - mu[1]));
        let cxy = flat2
            .iter()
            .fold(0.0, |s, c| s + (c[0] - mu[0]) * (c[1] - mu[1]));
        let angle = 0.5 * (2.0 * cxy).atan2(cxx - cyy);
        let axes = [(angle.cos(), angle.sin()), (-angle.sin(), angle.cos())];
        let mut side: Vec<f64> = axes
            .iter()
            .map(|a| {
                let along = flat2.iter().map(|c| c[0] * a.0 + c[1] * a.1);
                let hi = along
                    .clone()
                    .reduce(|m, v| if v > m { v } else { m })
                    .unwrap();
                let lo = along.reduce(|m, v| if v < m { v } else { m }).unwrap();
                hi - lo
            })
            .collect();
        side.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        if side[0] < 0.01 || side[1] < 0.03 {
            continue;
        }
        let step = (points.len() / 24).max(1);
        let points: Vec<[f64; 3]> = points.into_iter().step_by(step).collect();
        let others: Vec<&Tri> = tris.iter().filter(|t| t.node != node).collect();
        let mut shown = 0;
        for p in &points {
            // Hidden inside a solid (the nearest face either way along its flattened axis leaves one), or,
            // lying level, under something (the game looks from above: glTF +y; a shower head's rain).
            let near = cast(p, &d, &others);
            if near[0].1 && near[1].1 {
                continue;
            }
            if d[1].abs() > 0.7 && cast(p, &[0.0, 1.0, 0.0], &others)[0].0 < f64::INFINITY {
                continue;
            }
            shown += 1;
        }
        let share = shown as f64 / points.len() as f64;
        if share > 0.25 {
            out.push((node_name(nodes, node), share));
        }
    }
    Ok(out)
}

/// `overrides.get(name, named_finish(name) or overrides.get('*'))`: a material's entry in `materials`
/// (None: unmapped).
pub(super) fn override_of(overrides: &J, name: &str) -> J {
    match overrides.get(name) {
        Some(ov) => ov.clone(),
        None => named_finish(name)
            .or_else(|| overrides.get("*").cloned())
            .unwrap_or(J::Null),
    }
}

/// Warnings for a model (at rest `lo`; as it shows, its parts where `pose` puts them: `slo`, `shi`,
/// `sparts`) against one content object's facts (None: none, a floor thing).
#[allow(clippy::too_many_arguments)]
pub(super) fn content_warnings(
    facts: Option<&Facts>,
    lo: &[Num; 3],
    (slo, shi): &Bounds,
    sparts: &[(Option<usize>, Bounds)],
    scale: Num,
    g: &Glb,
    pose: &Pose,
    entry: &J,
    in_hand: bool,
) -> R<Vec<String>> {
    let mut warnings = Vec::new();
    let floor = J::Str("floor".into());
    let layer = facts.map(|f| &f.layer).unwrap_or(&floor);
    let is = |l: &str| layer.str() == Some(l);
    let foot = |f: &Facts| -> R<(Num, Num)> {
        let l = nums(&f.footprint, 2)?;
        Ok((l[0], l[1]))
    };
    // Against a wall: the wall's face is `WALL` into the footprint's back tile. A wall thing, or a floor
    // thing indoors that stands against one, has its back on it.
    // (A rug lies under the wall: it may reach in. Stairs reach their footprint's back edge, where the floor
    // above goes on from the top step: no wall.)
    if let Some(f) = facts
        && (is("wall") || (is("floor") && !f.outside && !f.stairs))
    {
        let depth = f
            .footprint
            .at(1)
            .ok_or("list index out of range")?
            .num()
            .ok_or("bad operand type for unary -")?;
        let face = ((-depth) / Num::I(2)).f() + WALL;
        if slo[2].f() < face - 0.01 {
            warnings.push(strict(
                "wall",
                format!(
                    "reaches {} cm into the wall (its back at depth {}, the wall's face at {})",
                    fixed((face - slo[2].f()) * 100.0, 1),
                    fixed(slo[2].f(), 3),
                    fixed(face, 2)
                ),
            ));
        }
        // Opened or used: a door swung back or a lid tipped past upright reaches no farther (a clip's deepest key).
        // Not where someone stands behind it (content `staff.at`: a shop counter's till opens to the shopkeeper).
        let staffed = f
            .object
            .get("staff")
            .and_then(|s| s.get("at"))
            .and_then(|a| a.at(1))
            .and_then(J::num)
            .is_some_and(|z| z.f() < ((-depth) / Num::I(2)).f());
        let motions = match entry.get("motions") {
            Some(m) if !staffed => m,
            _ => &J::Null,
        };
        let mut deepest: Vec<(String, String, f64)> = Vec::new();
        for (clip, by, p) in opened(g, motions, pose)? {
            let back = (bounds(&g.gltf, &p, &HashSet::new())?.0.0[2] * scale).f();
            match deepest.iter_mut().find(|d| d.0 == clip) {
                Some(d) => d.2 = d.2.min(back),
                None => deepest.push((clip, by, back)),
            }
        }
        for (clip, by, back) in deepest {
            if back < face - 0.01 && back < slo[2].f() - 0.001 {
                warnings.push(strict(
                    "opened",
                    format!(
                        "opened ({}, {by}), reaches {} cm into the wall (its back at depth {}, the wall's face at {})",
                        repr_str(&clip),
                        fixed((face - back) * 100.0, 1),
                        fixed(back, 3),
                        fixed(face, 2)
                    ),
                ));
            }
        }
        // (Whether it should stand against the wall, its back on the face, nothing in the content says: a
        // counter and an island have the same top, and `fillsWall` only keeps the wall over it clear.)
    }
    // Within its footprint: a thing indoors on the floor, a wall or the ceiling that reaches past its tiles
    // overlaps what stands next to it (a fan's blades another's). Outside, trees, hedges and parasols
    // spread over their neighbours, and a model with `vary` is drawn larger or smaller anyway; a rug
    // may reach under the wall, and a thing on a top takes its disc (below).
    if let Some(f) = facts
        && (is("floor") || is("wall") || is("ceiling"))
        && !f.outside
        && !entry.has("vary")
    {
        let (w, d) = foot(f)?;
        let half = |n: Num| (n / Num::I(2)).f();
        let (nw, nd) = (((-w) / Num::I(2)).f(), ((-d) / Num::I(2)).f());
        // The tiles beside its own it reaches over at head height (content `overhang`, `[right, front]`: an arc
        // lamp's shade over the next tile) are its too, at rest.
        let (mut xlo, mut xhi, mut zlo, mut zhi) = (nw, half(w), nd, half(d));
        for t in f.object.get("overhang").and_then(J::list).unwrap_or(&[]) {
            let (r, fw) = (
                t.at(0).and_then(J::num).map_or(0.0, Num::f),
                t.at(1).and_then(J::num).map_or(0.0, Num::f),
            );
            xlo = xlo.min(r - 0.5);
            xhi = xhi.max(r + 0.5);
            zlo = zlo.min(fw - 0.5);
            zhi = zhi.max(fw + 0.5);
        }
        let over = [
            xlo - slo[0].f(),
            shi[0].f() - xhi,
            zlo - slo[2].f(),
            shi[2].f() - zhi,
        ]
        .into_iter()
        .reduce(|m, v| if v > m { v } else { m })
        .unwrap();
        if over > OVERHANG {
            warnings.push(strict("footprint", format!(
                "reaches {} cm past its {} x {} footprint (x {} to {}, depth {} to {}): it overlaps what stands next to it",
                fixed(over * 100.0, 1),
                py::num_str(w),
                py::num_str(d),
                fixed(slo[0].f(), 2),
                fixed(shi[0].f(), 2),
                fixed(slo[2].f(), 2),
                fixed(shi[2].f(), 2)
            )));
        }
        // A part that goes round (a fan's blades) sweeps a whole circle about its pivot, not just its rest pose.
        let none = J::Obj(Vec::new());
        for s in swept(
            g,
            entry.get("motions").filter(|m| m.truthy()).unwrap_or(&none),
        )? {
            let sc = scale.f();
            let (xlo, xhi, zlo, zhi, radius) = (
                s.x.0 * sc,
                s.x.1 * sc,
                s.z.0 * sc,
                s.z.1 * sc,
                s.radius * sc,
            );
            let over = [nw - xlo, xhi - half(w), nd - zlo, zhi - half(d)]
                .into_iter()
                .reduce(|m, v| if v > m { v } else { m })
                .unwrap();
            if over > SWEEP {
                warnings.push(strict("sweep", format!(
                    "turning ({}), {} sweeps {} cm past its {} x {} footprint (x {} to {}, depth {} to {}; {} m from its axis): it overlaps what stands next to it (keep its axis on the middle, or make it smaller)",
                    repr_str(&s.clip),
                    s.name,
                    fixed(over * 100.0, 1),
                    py::num_str(w),
                    py::num_str(d),
                    fixed(xlo, 3),
                    fixed(xhi, 3),
                    fixed(zlo, 3),
                    fixed(zhi, 3),
                    fixed(radius, 3)
                )));
            }
        }
    }
    // On a top: the game keeps the disc of its content `radius` round its origin clear on its host, so the
    // model reaches no farther (a square tray's corners, a model off its origin).
    if let Some(f) = facts
        && is("surface")
    {
        let out = reach(g, pose)? * scale.f();
        if out > 1.1 * f.radius {
            warnings.push(strict("radius", format!(
                "reaches {} m from its origin, past the {} m disc its content `radius` keeps on a top: it overlaps what stands next to it (give it a larger radius)",
                fixed(out, 3),
                fixed(f.radius, 2)
            )));
        }
    }
    // Thin on the floor: a house's floor hides what lies under `FLOOR_TOP` (a mat, a belt), the whole
    // model or a moving part (as it shows, grown). Not a tool, which is only ever drawn in a hand.
    if (is("floor") || is("rug")) && lo[1].f() <= 0.005 && !in_hand {
        let mut flat: Vec<(Option<usize>, f64)> = Vec::new();
        if shi[1].f() < FLOOR_TOP {
            flat.push((None, shi[1].f()));
        }
        for (part, b) in sparts {
            let top = (b.1[1] * scale).f();
            if part.is_some() && top < FLOOR_TOP {
                flat.push((*part, top));
            }
        }
        let empty = J::Arr(Vec::new());
        let nodes = g.gltf.get("nodes").unwrap_or(&empty);
        for (part, top) in flat {
            let what = match part {
                None => "it".to_string(),
                Some(p) => format!("moving part {}", node_name(nodes, p)),
            };
            warnings.push(strict("thin", format!("{what} reaches only {} cm up: a house's floor is drawn 1 cm up and hides it (make it reach 1.2 cm)", fixed(top * 100.0, 1))));
        }
    }
    Ok(warnings)
}
