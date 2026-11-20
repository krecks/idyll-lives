//! A model encoded as a .glb.

use super::*;

/// Writes `model` as a .glb laid out as `write` lays one out, so that a file `write` made, read, is written again to
/// the same bytes. The clips' keys go into the buffer a node at a time, its clips in the order `Node::clips` gives
/// (then the model's), its channels by what they move; each clip lists its channels in its own order.
pub fn encode(model: &Model) -> Vec<u8> {
    let mut file = File::default();
    let meshes: Vec<String> = model
        .meshes
        .iter()
        .map(|mesh| {
            let primitives: Vec<String> = mesh
                .primitives
                .iter()
                .map(|p| primitive_json(&mut file, p))
                .collect();
            format!(
                "{{\"name\":{},\"primitives\":[{}]}}",
                quoted(&mesh.name),
                primitives.join(",")
            )
        })
        .collect();
    let nodes: Vec<String> = model.nodes.iter().map(node_json).collect();
    // the keys' accessors (input, output) of each channel of each clip
    let mut keys: Vec<Vec<(usize, usize)>> = model
        .animations
        .iter()
        .map(|a| vec![(0, 0); a.channels.len()])
        .collect();
    // (the nodes as the clips are met: each root, then its children, as the writer and Blender's exporter go)
    let mut met: Vec<usize> = Vec::new();
    fn walk(i: usize, nodes: &[Node], met: &mut Vec<usize>) {
        if i < nodes.len() && !met.contains(&i) {
            met.push(i);
            for &child in &nodes[i].children {
                walk(child, nodes, met);
            }
        }
    }
    for &root in &model.roots {
        walk(root, &model.nodes, &mut met);
    }
    for i in 0..model.nodes.len() {
        walk(i, &model.nodes, &mut met);
    }
    for i in met {
        let n = &model.nodes[i];
        let mut order: Vec<usize> = n.clips.clone();
        for (a, anim) in model.animations.iter().enumerate() {
            if !order.contains(&a) && anim.channels.iter().any(|c| c.node == i) {
                order.push(a);
            }
        }
        for a in order {
            let Some(anim) = model.animations.get(a) else {
                continue;
            };
            let mut channels: Vec<(usize, &Channel)> = anim
                .channels
                .iter()
                .enumerate()
                .filter(|(_, c)| c.node == i)
                .collect();
            channels.sort_by_key(|(_, c)| c.target);
            for (k, c) in channels {
                let kind = if c.target.width() == 4 {
                    "VEC4"
                } else {
                    "VEC3"
                };
                let input = file.floats(&c.times, "SCALAR", 1, true, None);
                let output = file.floats(&c.values, kind, c.target.width(), false, None);
                keys[a][k] = (input, output);
            }
        }
    }
    while !file.bin.len().is_multiple_of(4) {
        file.bin.push(0);
    }
    let roots: Vec<String> = model.roots.iter().map(|r| r.to_string()).collect();
    let mut json = format!(
        "{{\"asset\":{{\"generator\":{},\"version\":\"2.0\"}},\"scene\":0,",
        quoted(&model.generator)
    );
    json.push_str(&format!(
        "\"scenes\":[{{\"name\":\"Scene\",\"nodes\":[{}]}}],",
        roots.join(",")
    ));
    json.push_str(&format!("\"nodes\":[{}],", nodes.join(",")));
    if !model.animations.is_empty() {
        let all: Vec<String> = model
            .animations
            .iter()
            .zip(&keys)
            .map(|(a, keys)| {
                let (mut channels, mut samplers) = (Vec::new(), Vec::new());
                for (k, (c, (input, output))) in a.channels.iter().zip(keys).enumerate() {
                    channels.push(format!(
                        "{{\"sampler\":{k},\"target\":{{\"node\":{},\"path\":\"{}\"}}}}",
                        c.node,
                        c.target.name()
                    ));
                    samplers.push(format!(
                        "{{\"input\":{input},\"interpolation\":\"{}\",\"output\":{output}}}",
                        c.interpolation.name()
                    ));
                }
                format!(
                    "{{\"channels\":[{}],\"name\":{},\"samplers\":[{}]}}",
                    channels.join(","),
                    quoted(&a.name),
                    samplers.join(",")
                )
            })
            .collect();
        json.push_str(&format!("\"animations\":[{}],", all.join(",")));
    }
    let mats: Vec<String> = model.materials.iter().map(material_json).collect();
    json.push_str(&format!("\"materials\":[{}],", mats.join(",")));
    json.push_str(&format!("\"meshes\":[{}],", meshes.join(",")));
    json.push_str(&format!("\"accessors\":[{}],", file.accessors.join(",")));
    json.push_str(&format!("\"bufferViews\":[{}],", file.views.join(",")));
    json.push_str(&format!(
        "\"buffers\":[{{\"byteLength\":{}}}]}}",
        file.bin.len()
    ));
    while !json.len().is_multiple_of(4) {
        json.push(' ');
    }
    let total = 12 + 8 + json.len() + 8 + file.bin.len();
    let mut out = Vec::with_capacity(total);
    out.extend(b"glTF");
    out.extend(2u32.to_le_bytes());
    out.extend((total as u32).to_le_bytes());
    out.extend((json.len() as u32).to_le_bytes());
    out.extend(b"JSON");
    out.extend(json.as_bytes());
    out.extend((file.bin.len() as u32).to_le_bytes());
    out.extend(b"BIN\0");
    out.extend(&file.bin);
    out
}
