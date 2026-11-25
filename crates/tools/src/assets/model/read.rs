//! Reading a model.

use super::*;

/// A model's geometry and animations as a loader reads them (compressed or not).
pub fn read(data: &[u8]) -> Result<Model, String> {
    let g = parse(data)?;
    let v = views(&g)?;
    let nodes = list(&g.json, "nodes");
    // Each mesh's place: its first node's, down from the scene.
    let mut parent = vec![usize::MAX; nodes.len()];
    for (i, node) in nodes.iter().enumerate() {
        for child in list(node, "children") {
            if let Some(c) = child.as_u64() {
                parent[c as usize] = i;
            }
        }
    }
    let world = |mut i: usize| -> Mat {
        let mut m = local(&nodes[i]);
        while parent[i] != usize::MAX {
            i = parent[i];
            m = artkit::glb::compose(&local(&nodes[i]), &m);
        }
        m
    };
    let mut meshes = Vec::new();
    let mut spans = Vec::new();
    for (index, mesh) in list(&g.json, "meshes").iter().enumerate() {
        let place = nodes
            .iter()
            .position(|n| uint(n, "mesh") == Some(index))
            .map(world)
            .unwrap_or([
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
            ]);
        let mut prims = Vec::new();
        let mut lo = [f32::INFINITY; 3];
        let mut hi = [f32::NEG_INFINITY; 3];
        for p in list(mesh, "primitives") {
            let attribute = |name: &str| -> Result<Option<Accessor>, String> {
                uint(&p["attributes"], name)
                    .map(|i| accessor(&g, &v, i))
                    .transpose()
            };
            let position = attribute("POSITION")?.ok_or("a primitive without positions")?;
            let normal = attribute("NORMAL")?;
            let uv = attribute("TEXCOORD_0")?;
            let color = attribute("COLOR_0")?;
            let indices: Vec<u32> = match uint(p, "indices") {
                Some(i) => {
                    let a = accessor(&g, &v, i)?;
                    (0..a.count).map(|k| a.index(k)).collect()
                }
                None => (0..position.count as u32).collect(),
            };
            let mut corners = Vec::with_capacity(indices.len());
            for &i in &indices {
                let i = i as usize;
                if i >= position.count {
                    return Err("a triangle past its vertices".into());
                }
                let at = [0, 1, 2].map(|c| position.value(i, c));
                for c in 0..3 {
                    lo[c] = lo[c].min(at[c]);
                    hi[c] = hi[c].max(at[c]);
                }
                corners.push(Corner {
                    position: [0, 1, 2].map(|r| {
                        place[r][0] * at[0]
                            + place[r][1] * at[1]
                            + place[r][2] * at[2]
                            + place[r][3]
                    }),
                    normal: normal
                        .as_ref()
                        .map_or([0.0; 3], |n| [0, 1, 2].map(|c| n.value(i, c))),
                    uv: uv
                        .as_ref()
                        .map_or([0.0; 2], |u| [0, 1].map(|c| u.value(i, c))),
                    color: color.as_ref().map_or([255; 4], |k| {
                        [0, 1, 2, 3].map(|c| {
                            if c < k.width {
                                (k.value(i, c).clamp(0.0, 1.0) * 255.0).round() as u8
                            } else {
                                255
                            }
                        })
                    }),
                });
            }
            prims.push(Primitive {
                material: uint(p, "material"),
                corners,
            });
        }
        meshes.push(prims);
        let half = (0..3).fold(0f32, |m, c| m.max((hi[c] - lo[c]) * 0.5));
        let stretch = (0..3).fold(0f32, |m, c| {
            m.max(
                (place[0][c] * place[0][c] + place[1][c] * place[1][c] + place[2][c] * place[2][c])
                    .sqrt(),
            )
        });
        spans.push(if half.is_finite() {
            half * stretch
        } else {
            0.0
        });
    }
    let mut animations = Vec::new();
    for animation in list(&g.json, "animations") {
        for sampler in list(animation, "samplers") {
            for key in ["input", "output"] {
                let a = accessor(
                    &g,
                    &v,
                    uint(sampler, key).ok_or("an animation sampler without its accessors")?,
                )?;
                animations.push(a.data);
            }
        }
    }
    Ok(Model {
        meshes,
        animations,
        nodes: nodes
            .iter()
            .filter_map(|n| n.get("name").and_then(J::as_str).map(String::from))
            .collect(),
        spans,
    })
}
