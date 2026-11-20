//! A model read whole.
//!
//! Split from `glb.rs` (see `mod.rs`).

use super::*;

/// Reads a .glb: its nodes, meshes (positions, normals, UVs, COLOR_0 and indices of each primitive), materials
/// and clips. Triangles only, in the file's own buffer; a part placed by a matrix, a sparse accessor or another
/// buffer is refused (no file the game has uses one).
pub fn read(data: &[u8]) -> Result<Model, String> {
    let (j, bin) = chunks(data)?;
    let generator = j
        .get("asset")
        .and_then(|a| a.get("generator"))
        .and_then(Json::str)
        .unwrap_or_default()
        .to_string();
    let name = |v: &Json| {
        v.get("name")
            .and_then(Json::str)
            .unwrap_or_default()
            .to_string()
    };
    let mut materials = Vec::new();
    for m in list(&j, "materials") {
        let pbr = m.get("pbrMetallicRoughness");
        let factor = |k: &str| pbr.and_then(|p| p.get(k)).and_then(float);
        materials.push(Material {
            name: name(m),
            base: floats(pbr.and_then(|p| p.get("baseColorFactor")), [1.0; 4])?,
            metallic: factor("metallicFactor").unwrap_or(1.0),
            roughness: factor("roughnessFactor").unwrap_or(1.0),
            double_sided: m.get("doubleSided") == Some(&Json::Bool(true)),
            cutoff: (m.get("alphaMode").and_then(Json::str) == Some("MASK"))
                .then(|| m.get("alphaCutoff").and_then(float).unwrap_or(0.5)),
        });
    }
    let mut meshes = Vec::new();
    for (mi, m) in list(&j, "meshes").iter().enumerate() {
        let primitives = list(m, "primitives")
            .iter()
            .map(|p| primitive(&j, bin, p, mi))
            .collect::<Result<Vec<_>, _>>()?;
        if let Some(bad) = primitives
            .iter()
            .filter_map(|p| p.material)
            .find(|&x| x >= materials.len())
        {
            return Err(format!("mesh {mi}: no material {bad}"));
        }
        meshes.push(Mesh {
            name: name(m),
            primitives,
        });
    }
    let mut nodes = Vec::new();
    for (ni, n) in list(&j, "nodes").iter().enumerate() {
        if n.get("matrix").is_some() {
            return Err(format!("node {ni} is placed by a matrix: not read"));
        }
        let mesh = int(n.get("mesh"));
        if let Some(m) = mesh.filter(|&m| m >= meshes.len()) {
            return Err(format!("node {ni}: no mesh {m}"));
        }
        nodes.push(Node {
            name: name(n),
            mesh,
            children: list(n, "children")
                .iter()
                .filter_map(|c| int(Some(c)))
                .collect(),
            translation: floats(n.get("translation"), [0.0; 3])?,
            rotation: floats(n.get("rotation"), [0.0, 0.0, 0.0, 1.0])?,
            scale: floats(n.get("scale"), [1.0; 3])?,
            clips: Vec::new(),
        });
    }
    if let Some(bad) = nodes
        .iter()
        .flat_map(|n| &n.children)
        .find(|&&c| c >= nodes.len())
    {
        return Err(format!("no node {bad}"));
    }
    let scene = int(j.get("scene")).unwrap_or(0);
    let roots: Vec<usize> = match list(&j, "scenes").get(scene) {
        Some(s) => list(s, "nodes")
            .iter()
            .filter_map(|n| int(Some(n)))
            .collect(),
        None => (0..nodes.len()).collect(),
    };
    if let Some(bad) = roots.iter().find(|&&r| r >= nodes.len()) {
        return Err(format!("the scene has no node {bad}"));
    }
    // (node, animation) by the first of its keys' accessors: the order its clips' keys lie in the file
    let mut first: BTreeMap<(usize, usize), usize> = BTreeMap::new();
    let mut animations = Vec::new();
    for (ai, a) in list(&j, "animations").iter().enumerate() {
        let samplers = list(a, "samplers");
        let mut channels = Vec::new();
        for c in list(a, "channels") {
            let target = c.get("target");
            let Some(node) = int(target.and_then(|t| t.get("node"))) else {
                continue;
            };
            if node >= nodes.len() {
                return Err(format!("animation {ai}: no node {node}"));
            }
            let target = match target.and_then(|t| t.get("path")).and_then(Json::str) {
                Some("translation") => Target::Translation,
                Some("rotation") => Target::Rotation,
                Some("scale") => Target::Scale,
                // (morph weights: no file has them)
                _ => continue,
            };
            let s = int(c.get("sampler"))
                .and_then(|s| samplers.get(s))
                .ok_or_else(|| format!("animation {ai}: a channel without its sampler"))?;
            let (Some(input), Some(output)) = (int(s.get("input")), int(s.get("output"))) else {
                return Err(format!("animation {ai}: a sampler without keys"));
            };
            let interpolation = match s.get("interpolation").and_then(Json::str) {
                None | Some("LINEAR") => Interpolation::Linear,
                Some("STEP") => Interpolation::Step,
                Some("CUBICSPLINE") => Interpolation::CubicSpline,
                Some(other) => return Err(format!("animation {ai}: interpolation {other}")),
            };
            let at = first.entry((node, ai)).or_insert(usize::MAX);
            *at = (*at).min(input.min(output));
            let (times, _) = accessor(&j, bin, input)?;
            let (values, per) = accessor(&j, bin, output)?;
            if per != target.width() {
                return Err(format!(
                    "animation {ai}: {per} numbers a key for a {}",
                    target.name()
                ));
            }
            channels.push(Channel {
                node,
                target,
                interpolation,
                times,
                values,
            });
        }
        animations.push(Animation {
            name: name(a),
            channels,
        });
    }
    for (n, node) in nodes.iter_mut().enumerate() {
        let mut clips: Vec<(usize, usize)> = first
            .iter()
            .filter(|((m, _), _)| *m == n)
            .map(|((_, a), &at)| (at, *a))
            .collect();
        clips.sort_unstable();
        node.clips = clips.into_iter().map(|(_, a)| a).collect();
    }
    Ok(Model {
        generator,
        roots,
        nodes,
        meshes,
        materials,
        animations,
    })
}
