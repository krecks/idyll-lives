//! Nodes: their places, matrices and poses.

use super::*;

type Matrix = [Num; 16];
const IDENTITY: Matrix = {
    let (o, z) = (Num::I(1), Num::I(0));
    [o, z, z, z, z, o, z, z, z, z, o, z, z, z, z, o]
};

/// A point `v` through a column-major 4x4 matrix `m` (glTF axes: the game's x is its -x).
pub(super) fn apply(m: &Matrix, v: &[Num]) -> [Num; 3] {
    [
        m[0] * v[0] + m[4] * v[1] + m[8] * v[2] + m[12],
        m[1] * v[0] + m[5] * v[1] + m[9] * v[2] + m[13],
        m[2] * v[0] + m[6] * v[1] + m[10] * v[2] + m[14],
    ]
}

pub(crate) fn applied(m: &Matrix, v: &[Num]) -> [f64; 3] {
    apply(m, v).map(Num::f)
}

pub(super) fn nums(v: &J, what: usize) -> R<Vec<Num>> {
    let l = v
        .list()
        .ok_or_else(|| format!("cannot unpack non-iterable {} object", glb::type_name(v)))?;
    let out: Option<Vec<Num>> = l.iter().map(J::num).collect();
    let out = out.ok_or("unsupported operand type(s)")?;
    if out.len() != what && what != 0 {
        return Err(if out.len() > what {
            format!("too many values to unpack (expected {what})")
        } else {
            format!(
                "not enough values to unpack (expected {what}, got {})",
                out.len()
            )
        });
    }
    Ok(out)
}

/// A node's index (`nodes[i]`, an int of the file, which must be one of its nodes).
pub(super) fn node_index(nodes: &J, i: &J) -> R<usize> {
    index(nodes, i)?;
    match i {
        J::Int(i) if *i >= 0 => Ok(*i as usize),
        _ => Err("list index out of range".into()),
    }
}

/// A node an animation channel names (`c['target']['node']`), as an index.
pub(super) fn target_node(node: &J) -> usize {
    match node {
        J::Int(i) if *i >= 0 => *i as usize,
        _ => usize::MAX,
    }
}

/// `nodes[i].get('name', i)`, as printed.
pub(super) fn node_name(nodes: &J, i: usize) -> String {
    match nodes.at(i as i64).and_then(|n| n.get("name")) {
        Some(name) => str_of(name),
        None => i.to_string(),
    }
}

/// A mesh node of the default scene: its mesh, its world matrix, its moving part (the nearest node above
/// it, or itself, that an animation moves; None: the still body) and the node.
pub(crate) struct Placed {
    pub(crate) mesh: J,
    pub(crate) m: Matrix,
    pub(super) part: Option<usize>,
    pub(crate) node: usize,
}

/// Translations, rotations and scales (by node and path) that put nodes elsewhere than at rest (a part grown,
/// `grown`; a bed pulled out, `open_pose`).
pub(super) type Pose = HashMap<(usize, &'static str), Vec<Num>>;

/// The mesh nodes of the default scene (`Placed`), their parts where `pose` puts them; `worlds`, if given,
/// gets every node's world matrix.
pub(crate) fn node_matrices(
    gltf: &J,
    pose: Option<&Pose>,
    worlds: Option<&mut HashMap<usize, Matrix>>,
) -> R<Vec<Placed>> {
    placed_in(gltf, &moved_nodes(gltf)?, pose, worlds)
}

/// `node_matrices` with `parts` as the nodes that are parts (`Placed::part`: the nearest of them above a mesh,
/// or its own node), in place of the nodes the animations move (a bird's, by name).
fn placed_in(
    gltf: &J,
    parts: &HashSet<usize>,
    pose: Option<&Pose>,
    mut worlds: Option<&mut HashMap<usize, Matrix>>,
) -> R<Vec<Placed>> {
    let empty = J::Arr(Vec::new());
    let nodes = gltf.get("nodes").unwrap_or(&empty);
    let scenes = gltf
        .get("scenes")
        .cloned()
        .unwrap_or(J::Arr(vec![J::Obj(Vec::new())]));
    let scene = index(&scenes, gltf.get("scene").unwrap_or(&J::Int(0)))?;

    fn local(n: &J, i: usize, pose: Option<&Pose>) -> R<Matrix> {
        if let Some(m) = n.get("matrix") {
            let m = nums(m, 0)?;
            return m
                .get(..16)
                .and_then(|m| m.try_into().ok())
                .ok_or_else(|| "list index out of range".into());
        }
        let (z, o) = (Num::I(0), Num::I(1));
        let t = match (
            pose.and_then(|p| p.get(&(i, "translation"))),
            n.get("translation"),
        ) {
            (Some(t), _) if t.len() == 3 => t.clone(),
            (_, Some(t)) => nums(t, 3)?,
            _ => vec![z, z, z],
        };
        let q = match (
            pose.and_then(|p| p.get(&(i, "rotation"))),
            n.get("rotation"),
        ) {
            (Some(q), _) if q.len() == 4 => q.clone(),
            (_, Some(q)) => nums(q, 4)?,
            _ => vec![z, z, z, o],
        };
        let s = match pose.and_then(|p| p.get(&(i, "scale"))) {
            Some(s) if s.len() != 3 => {
                return Err(format!(
                    "too many values to unpack (expected 3, got {})",
                    s.len()
                ));
            }
            Some(s) => s.clone(),
            None => match n.get("scale") {
                Some(s) => nums(s, 3)?,
                None => vec![o, o, o],
            },
        };
        let (qx, qy, qz, qw) = (q[0], q[1], q[2], q[3]);
        let two = Num::I(2);
        let r = [
            o - two * (qy * qy + qz * qz),
            two * (qx * qy + qz * qw),
            two * (qx * qz - qy * qw),
            two * (qx * qy - qz * qw),
            o - two * (qx * qx + qz * qz),
            two * (qy * qz + qx * qw),
            two * (qx * qz + qy * qw),
            two * (qy * qz - qx * qw),
            o - two * (qx * qx + qy * qy),
        ];
        Ok([
            r[0] * s[0],
            r[1] * s[0],
            r[2] * s[0],
            z,
            r[3] * s[1],
            r[4] * s[1],
            r[5] * s[1],
            z,
            r[6] * s[2],
            r[7] * s[2],
            r[8] * s[2],
            z,
            t[0],
            t[1],
            t[2],
            o,
        ])
    }

    fn mul(a: &Matrix, b: &Matrix) -> Matrix {
        let mut out = [Num::I(0); 16];
        for c in 0..4 {
            for r in 0..4 {
                let mut sum = Num::I(0);
                for k in 0..4 {
                    sum = sum + a[k * 4 + r] * b[c * 4 + k];
                }
                out[c * 4 + r] = sum;
            }
        }
        out
    }

    #[allow(clippy::too_many_arguments)]
    fn walk(
        nodes: &J,
        i: usize,
        parent: &Matrix,
        part: Option<usize>,
        pose: Option<&Pose>,
        parts: &HashSet<usize>,
        worlds: &mut Option<&mut HashMap<usize, Matrix>>,
        out: &mut Vec<Placed>,
        depth: usize,
    ) -> R<()> {
        if depth > 990 {
            return Err("maximum recursion depth exceeded".into());
        }
        let n = nodes.at(i as i64).ok_or("list index out of range")?;
        let m = mul(parent, &local(n, i, pose)?);
        if let Some(w) = worlds.as_deref_mut() {
            w.insert(i, m);
        }
        let part = if parts.contains(&i) { Some(i) } else { part };
        if let Some(mesh) = n.get("mesh") {
            out.push(Placed {
                mesh: mesh.clone(),
                m,
                part,
                node: i,
            });
        }
        for c in n.get("children").and_then(J::list).unwrap_or(&[]) {
            walk(
                nodes,
                node_index(nodes, c)?,
                &m,
                part,
                pose,
                parts,
                worlds,
                out,
                depth + 1,
            )?;
        }
        Ok(())
    }

    let mut out = Vec::new();
    for i in scene.get("nodes").and_then(J::list).unwrap_or(&[]) {
        walk(
            nodes,
            node_index(nodes, i)?,
            &IDENTITY,
            None,
            pose,
            parts,
            &mut worlds,
            &mut out,
            0,
        )?;
    }
    Ok(out)
}

/// The nodes the file's animations move.
pub(super) fn moved_nodes(gltf: &J) -> R<HashSet<usize>> {
    let mut out = HashSet::new();
    for a in gltf.get("animations").and_then(J::list).unwrap_or(&[]) {
        for c in key(a, "channels")?.list().unwrap_or(&[]) {
            if let Some(node) = key(c, "target")?.get("node") {
                out.insert(target_node(node));
            }
        }
    }
    Ok(out)
}

/// A JSON number (not a bool), as f64.
fn json_number(v: &J) -> Option<f64> {
    match v {
        J::Int(i) => Some(*i as f64),
        J::Float(f) => Some(*f),
        _ => None,
    }
}

/// What is wrong with a bird entry's `bird` itself (`BIRD_KEYS`): no file needed.
pub(super) fn bird_fields(bird: &J) -> Vec<String> {
    let Some(fields) = bird.entries() else {
        return vec![format!(
            "`bird` is {{\"length\", \"wingspan\", ...}} (metres), not {}",
            repr(bird)
        )];
    };
    let mut errors = Vec::new();
    let mut unknown: Vec<&str> = fields
        .iter()
        .map(|(k, _)| k.as_str())
        .filter(|k| !BIRD_KEYS.contains(k))
        .collect();
    unknown.sort();
    if !unknown.is_empty() {
        errors.push(format!(
            "`bird`: unknown {} (it takes {})",
            unknown.join(", "),
            BIRD_KEYS.join(", ")
        ));
    }
    for k in BIRD_KEYS {
        let Some(v) = bird.get(k) else {
            if k == "length" || k == "wingspan" {
                errors.push(format!("`bird` has no `{k}`"));
            }
            continue;
        };
        let ok = match (k, json_number(v)) {
            ("pitch", Some(p)) => (-90.0..=90.0).contains(&p),
            (_, Some(m)) => m > 0.0,
            _ => false,
        };
        if !ok {
            let what = if k == "pitch" {
                "angle (degrees, -90 to 90)"
            } else {
                "length (metres, over 0)"
            };
            errors.push(format!("`bird.{k}` {} is no {what}", repr(v)));
        }
    }
    errors
}

/// A bird's file as `bird_errors` needs it: which of `BIRD_PARTS` it has, each part's vertices at rest (game axes),
/// and the parts saved at scale 0.
pub(super) struct BirdParts {
    pub(super) names: BTreeSet<String>,
    pub(super) points: HashMap<String, Vec<[f64; 3]>>,
    pub(super) flat: Vec<String>,
}

pub(super) fn bird_parts(g: &Glb) -> R<BirdParts> {
    let nodes = g.gltf.get("nodes").and_then(J::list).unwrap_or(&[]);
    let name = |i: usize| {
        nodes
            .get(i)
            .and_then(|n| n.get("name"))
            .and_then(J::str)
            .unwrap_or("")
    };
    let mut parts = BirdParts {
        names: BTreeSet::new(),
        points: HashMap::new(),
        flat: Vec::new(),
    };
    let mut named = HashSet::new();
    for (i, n) in nodes.iter().enumerate() {
        if BIRD_PARTS.contains(&name(i)) {
            named.insert(i);
            parts.names.insert(name(i).to_string());
            if n.get("scale").and_then(J::list).is_some_and(|s| {
                s.iter()
                    .any(|v| json_number(v).is_some_and(|v| v.abs() < 1e-6))
            }) {
                parts.flat.push(name(i).to_string());
            }
        }
    }
    // Each mesh in the part it is in: its own node, or the nearest one above it named as a part.
    for p in placed_in(&g.gltf, &named, None, None)? {
        let Some(part) = p.part else { continue };
        let out = parts.points.entry(name(part).to_string()).or_default();
        for prim in primitives(&g.gltf, &p.mesh)? {
            for v in positions(g, prim)? {
                let q = apply(&p.m, &v);
                out.push([-q[0].f(), q[1].f(), q[2].f()]);
            }
        }
    }
    Ok(parts)
}

/// What is wrong with a bird (its `bird`, already sound by `bird_fields`) against its file: its parts, its wings
/// spread to its sides at scale 1, its wingspan across them, its length from bill (`head`) to tail tip (`tail`),
/// and its body's middle within its height. `dims` is the file's size (game width, height, depth).
pub(super) fn bird_errors(bird: &J, parts: &BirdParts, dims: [f64; 3]) -> Vec<String> {
    let mut errors = Vec::new();
    let missing: Vec<&str> = BIRD_PARTS
        .iter()
        .copied()
        .filter(|p| !parts.names.contains(*p))
        .collect();
    if !missing.is_empty() {
        errors.push(format!(
            "a bird's parts are nodes named {}: it has no {}",
            BIRD_PARTS.join(", "),
            missing.join(", ")
        ));
    }
    for name in &parts.flat {
        errors.push(format!("{name} is saved at scale 0: the renderer bakes each part as the file has it (the wings spread, at scale 1) and hides them itself"));
    }
    let wanted = |k: &str| bird.get(k).and_then(json_number).unwrap_or(0.0);
    let off = |got: f64, want: f64| (got - want).abs() > BIRD_FIT * want;
    let (span, length) = (wanted("wingspan"), wanted("length"));
    if off(dims[0], span) {
        errors.push(format!(
            "{} m across its spread wings, its `bird.wingspan` says {} m (within {})",
            fixed(dims[0], 3),
            fixed(span, 3),
            percent(BIRD_FIT)
        ));
    }
    let (head, tail) = (parts.points.get("head"), parts.points.get("tail"));
    if let (Some(head), Some(tail)) = (head, tail) {
        let far = head
            .iter()
            .flat_map(|a| tail.iter().map(move |b| dist(a, b)))
            .fold(0.0, f64::max);
        if off(far, length) {
            errors.push(format!(
                "{} m from bill to tail tip, its `bird.length` says {} m (within {})",
                fixed(far, 3),
                fixed(length, 3),
                percent(BIRD_FIT)
            ));
        }
    }
    for (name, side, where_) in [
        ("wing.L", -1.0, "its left, game -x (glTF +x)"),
        ("wing.R", 1.0, "its right, game +x (glTF -x)"),
    ] {
        if let Some(points) = parts.points.get(name).filter(|p| !p.is_empty()) {
            let mid = points.iter().map(|p| p[0]).sum::<f64>() / points.len() as f64;
            if mid * side <= 0.0 {
                errors.push(format!(
                    "{name} reaches out the wrong way: a bird's {name} is spread to {where_}"
                ));
            }
        }
    }
    if let Some(h) = bird.get("perchHeight").and_then(json_number)
        && h > dims[1]
    {
        errors.push(format!(
            "`bird.perchHeight` {} m is over the bird's height, {} m",
            fixed(h, 3),
            fixed(dims[1], 3)
        ));
    }
    errors
}

/// What is wrong with an entry's `perches` (where the game's birds sit: [[x, y, z], ...] in the model's game axes,
/// metres, before the entry's `scale`) against the model's box (`lo`, `hi`, game axes, at rest, scaled by `scale`).
pub(super) fn perch_errors(perches: &J, lo: [f64; 3], hi: [f64; 3], scale: f64) -> Vec<String> {
    let Some(list) = perches.list() else {
        return vec![format!(
            "`perches` is a list of [x, y, z] (metres), not {}",
            repr(perches)
        )];
    };
    if list.is_empty() {
        return vec!["`perches` is empty: leave it out".into()];
    }
    let mut errors = Vec::new();
    let mut outside = Vec::new();
    for (i, p) in list.iter().enumerate() {
        let xyz: Vec<f64> = p
            .list()
            .unwrap_or(&[])
            .iter()
            .filter_map(json_number)
            .collect();
        if xyz.len() != 3 || p.list().is_none_or(|l| l.len() != 3) {
            errors.push(format!("`perches` {i}: {} is no [x, y, z]", repr(p)));
            continue;
        }
        if (0..3).any(|k| xyz[k] * scale < lo[k] - PERCH_OUT || xyz[k] * scale > hi[k] + PERCH_OUT)
        {
            outside.push(i);
        }
    }
    if let Some(&first) = outside.first() {
        let box_ = (0..3)
            .map(|k| format!("{} to {}", fixed(lo[k] / scale, 2), fixed(hi[k] / scale, 2)))
            .collect::<Vec<_>>()
            .join(", ");
        errors.push(format!(
            "{} of {} perches outside the model (the first, {first}: {}; its box x, y, z: {box_})",
            outside.len(),
            list.len(),
            repr(&list[first])
        ));
    }
    errors
}

/// What is wrong with an entry's `habitat` (the kind of plant its `perches` are on, which sets the birds that sit
/// there: one of `HABITATS`), if it has one.
pub(super) fn habitat_errors(entry: &J) -> Vec<String> {
    let Some(habitat) = entry.get("habitat") else {
        return Vec::new();
    };
    let mut errors = Vec::new();
    if !habitat.str().is_some_and(|h| HABITATS.contains(&h)) {
        errors.push(format!(
            "`habitat` {} is none of {}",
            repr(habitat),
            HABITATS.join(", ")
        ));
    }
    if entry.get("perches").is_none() {
        errors.push("`habitat` without `perches`: it says what the perches are on".into());
    }
    errors
}
