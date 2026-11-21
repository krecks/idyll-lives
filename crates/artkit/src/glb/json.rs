//! The JSON a model is written as.

use super::*;

pub(super) fn material_json(m: &Material) -> String {
    let mut pbr = Vec::new();
    if m.base[..3] != [1.0, 1.0, 1.0] {
        pbr.push(format!("\"baseColorFactor\":{}", nums(&m.base)));
    }
    if m.metallic != 1.0 {
        pbr.push(format!("\"metallicFactor\":{}", num(m.metallic as f64)));
    }
    if m.roughness != 1.0 {
        pbr.push(format!("\"roughnessFactor\":{}", num(m.roughness as f64)));
    }
    let mut out = format!("{{\"name\":{}", quoted(&m.name));
    if !pbr.is_empty() {
        out.push_str(&format!(",\"pbrMetallicRoughness\":{{{}}}", pbr.join(",")));
    }
    if m.double_sided {
        out.push_str(",\"doubleSided\":true");
    }
    if let Some(cut) = m.cutoff {
        out.push_str(&format!(
            ",\"alphaMode\":\"MASK\",\"alphaCutoff\":{}",
            num(cut as f64)
        ));
    }
    out.push('}');
    out
}

pub(super) fn primitive_json(file: &mut File, p: &Primitive) -> String {
    let flat = |v: &[[f32; 3]]| -> Vec<f32> { v.iter().flatten().copied().collect() };
    let mut attributes = Vec::new();
    attributes.push(format!(
        "\"POSITION\":{}",
        file.floats(&flat(&p.positions), "VEC3", 3, true, Some(ARRAY_BUFFER))
    ));
    if !p.normals.is_empty() {
        attributes.push(format!(
            "\"NORMAL\":{}",
            file.floats(&flat(&p.normals), "VEC3", 3, false, Some(ARRAY_BUFFER))
        ));
    }
    if let Some(uvs) = &p.uvs {
        let uvs: Vec<f32> = uvs.iter().flatten().copied().collect();
        attributes.push(format!(
            "\"TEXCOORD_0\":{}",
            file.floats(&uvs, "VEC2", 2, false, Some(ARRAY_BUFFER))
        ));
    }
    if let Some(colours) = &p.colours {
        let bytes: Vec<u8> = colours
            .iter()
            .flatten()
            .flat_map(|&c| {
                let c = (c.clamp(0.0, 1.0) as f64 * 65535.0).round_ties_even() as u16;
                c.to_le_bytes()
            })
            .collect();
        let view = file.view(&bytes, Some(ARRAY_BUFFER));
        file.accessors.push(format!(
            "{{\"bufferView\":{view},\"componentType\":{USHORT},\"count\":{},\"normalized\":true,\"type\":\"VEC4\"}}",
            colours.len()
        ));
        attributes.push(format!("\"COLOR_0\":{}", file.accessors.len() - 1));
    }
    let (bytes, kind): (Vec<u8>, u32) = if p.positions.len() < 65535 {
        (
            p.indices
                .iter()
                .flat_map(|&i| (i as u16).to_le_bytes())
                .collect(),
            USHORT,
        )
    } else {
        (
            p.indices.iter().flat_map(|i| i.to_le_bytes()).collect(),
            UINT,
        )
    };
    let view = file.view(&bytes, Some(ELEMENT_ARRAY_BUFFER));
    file.accessors.push(format!(
        "{{\"bufferView\":{view},\"componentType\":{kind},\"count\":{},\"type\":\"SCALAR\"}}",
        p.indices.len()
    ));
    let material = p
        .material
        .map(|m| format!(",\"material\":{m}"))
        .unwrap_or_default();
    format!(
        "{{\"attributes\":{{{}}},\"indices\":{}{material}}}",
        attributes.join(","),
        file.accessors.len() - 1
    )
}

pub(super) fn node_json(n: &Node) -> String {
    let mut node = Vec::new();
    if !n.children.is_empty() {
        let kids: Vec<String> = n.children.iter().map(|k| k.to_string()).collect();
        node.push(format!("\"children\":[{}]", kids.join(",")));
    }
    if let Some(m) = n.mesh {
        node.push(format!("\"mesh\":{m}"));
    }
    node.push(format!("\"name\":{}", quoted(&n.name)));
    if n.rotation != [0.0, 0.0, 0.0, 1.0] {
        node.push(format!("\"rotation\":{}", nums(&n.rotation)));
    }
    if n.scale != [1.0; 3] {
        node.push(format!("\"scale\":{}", nums(&n.scale)));
    }
    if n.translation != [0.0; 3] {
        node.push(format!("\"translation\":{}", nums(&n.translation)));
    }
    format!("{{{}}}", node.join(","))
}
