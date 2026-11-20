//! Optimising a model: its accessors quantised and its meshes deduplicated.

use super::*;

/// The model made smaller (the module's header), or why it is left as it is.
pub fn optimize(data: &[u8], foliage: bool) -> Result<Vec<u8>, String> {
    let g = parse(data)?;
    if let Some(why) = unsupported(&g.json) {
        return Err(why);
    }
    let v = views(&g)?;
    let foliage = foliage
        || list(&g.json, "materials")
            .iter()
            .any(|m| m.get("name").and_then(J::as_str).is_some_and(foliage_name));
    // (The exporter gives primitives with the same triangles one index accessor: each gets its own here, in
    // its own vertices' new order. Vertex data is never shared; it couldn't be put in two orders.)
    let mut uses: HashMap<usize, usize> = HashMap::new();
    for mesh in list(&g.json, "meshes") {
        for p in list(mesh, "primitives") {
            let attributes = p
                .get("attributes")
                .and_then(J::as_object)
                .ok_or("a primitive without attributes")?;
            for a in attributes.values() {
                *uses
                    .entry(a.as_u64().ok_or("an accessor index that isn't a number")? as usize)
                    .or_default() += 1;
            }
        }
    }
    if uses.values().any(|&n| n > 1) {
        return Err("vertex data two primitives share".into());
    }

    let mut b = Builder {
        streams: Vec::new(),
        accessors: Vec::new(),
    };
    let mut json = g.json.clone();
    let mut meshes = list(&g.json, "meshes").to_vec();
    // Every primitive's triangles, one after another: one view for all, as one stream compresses best.
    let mut indices: Vec<u32> = Vec::new();
    let mut index_accessors: Vec<(usize, usize, usize)> = Vec::new();
    let mut frames: Vec<Option<([f32; 3], f32)>> = Vec::new();
    let mut most_vertices = 0usize;

    for mesh in &mut meshes {
        let prims = mesh
            .get("primitives")
            .and_then(J::as_array)
            .cloned()
            .unwrap_or_default();
        // The mesh's box: one offset and one scale for all its primitives, so they still meet.
        let mut lo = [f32::INFINITY; 3];
        let mut hi = [f32::NEG_INFINITY; 3];
        let mut positions = Vec::new();
        for p in &prims {
            let at = uint(&p["attributes"], "POSITION").ok_or("a primitive without positions")?;
            let a = accessor(&g, &v, at)?;
            if a.width != 3 || a.component != FLOAT {
                return Err("positions that aren't three floats".into());
            }
            for i in 0..a.count {
                for c in 0..3 {
                    lo[c] = lo[c].min(a.value(i, c));
                    hi[c] = hi[c].max(a.value(i, c));
                }
            }
            positions.push(a);
        }
        let frame = (!foliage && !prims.is_empty()).then(|| {
            let centre = [0, 1, 2].map(|c| (lo[c] + hi[c]) * 0.5);
            let half = (0..3).fold(0f32, |m, c| m.max((hi[c] - lo[c]) * 0.5));
            (centre, if half > 0.0 { half } else { 1.0 })
        });
        frames.push(frame);

        let mut out_prims = Vec::new();
        for (p, position) in prims.iter().zip(positions) {
            let count = position.count;
            let attributes = p["attributes"].as_object().expect("checked above");
            let source: Vec<u32> = match uint(p, "indices") {
                Some(i) => {
                    let a = accessor(&g, &v, i)?;
                    (0..a.count).map(|k| a.index(k)).collect()
                }
                None => (0..count as u32).collect(),
            };
            if !source.len().is_multiple_of(3) || source.iter().any(|&i| i as usize >= count) {
                return Err("triangles that don't fit their vertices".into());
            }
            let strip = meshopt::optimize_vertex_cache_strip(&source, count);
            // First use first: the order the triangles reach the vertices in.
            let mut place = vec![u32::MAX; count];
            let mut order: Vec<u32> = Vec::new();
            let triangles: Vec<u32> = strip
                .iter()
                .map(|&old| {
                    if place[old as usize] == u32::MAX {
                        place[old as usize] = order.len() as u32;
                        order.push(old);
                    }
                    place[old as usize]
                })
                .collect();
            let n = order.len();
            most_vertices = most_vertices.max(n);

            let mut out_attributes = Map::new();
            for (name, at) in attributes {
                let a = if name == "POSITION" {
                    position.clone()
                } else {
                    accessor(&g, &v, at.as_u64().expect("checked above") as usize)?
                };
                if a.count != count {
                    return Err(format!("{name} with another count than the positions"));
                }
                let made = if name == "POSITION" {
                    match frame {
                        Some((centre, scale)) => {
                            let mut q = Vec::with_capacity(n * 3);
                            for &old in &order {
                                for (c, centre) in centre.iter().enumerate() {
                                    q.push(quantize14((a.value(old as usize, c) - centre) / scale));
                                }
                            }
                            let (min, max) = bounds_i16(&q, 3);
                            let mut bytes = vec![0u8; n * 8];
                            for (i, v3) in q.as_chunks::<3>().0.iter().enumerate() {
                                for c in 0..3 {
                                    bytes[i * 8 + c * 2..i * 8 + c * 2 + 2]
                                        .copy_from_slice(&v3[c].to_le_bytes());
                                }
                            }
                            let s = b.stream(bytes, 8, Some(ARRAY_BUFFER));
                            b.accessor(s, 0, SHORT, true, "VEC3", n, Some(min), Some(max))
                        }
                        None => {
                            let mut lo = [f32::INFINITY; 3];
                            let mut hi = [f32::NEG_INFINITY; 3];
                            for &old in &order {
                                for c in 0..3 {
                                    lo[c] = lo[c].min(a.value(old as usize, c));
                                    hi[c] = hi[c].max(a.value(old as usize, c));
                                }
                            }
                            let s = b.stream(reordered(&a, &order, 12), 12, Some(ARRAY_BUFFER));
                            b.accessor(
                                s,
                                0,
                                FLOAT,
                                false,
                                "VEC3",
                                n,
                                Some(json!(lo)),
                                Some(json!(hi)),
                            )
                        }
                    }
                } else if name == "NORMAL"
                    && frame.is_some()
                    && a.width == 3
                    && a.component == FLOAT
                {
                    let mut bytes = vec![0u8; n * 8];
                    for (i, &old) in order.iter().enumerate() {
                        for c in 0..3 {
                            bytes[i * 8 + c * 2..i * 8 + c * 2 + 2].copy_from_slice(
                                &quantize10(a.value(old as usize, c)).to_le_bytes(),
                            );
                        }
                    }
                    let s = b.stream(bytes, 8, Some(ARRAY_BUFFER));
                    b.accessor(s, 0, SHORT, true, "VEC3", n, None, None)
                } else if name.starts_with("TEXCOORD_")
                    && frame.is_some()
                    && a.width == 2
                    && a.component == FLOAT
                    && (0..count).all(|i| (0..2).all(|c| (0.0..=1.0).contains(&a.value(i, c))))
                {
                    let mut bytes = vec![0u8; n * 4];
                    for (i, &old) in order.iter().enumerate() {
                        for c in 0..2 {
                            bytes[i * 4 + c * 2..i * 4 + c * 2 + 2].copy_from_slice(
                                &quantize12(a.value(old as usize, c)).to_le_bytes(),
                            );
                        }
                    }
                    let s = b.stream(bytes, 4, Some(ARRAY_BUFFER));
                    b.accessor(s, 0, USHORT, true, "VEC2", n, None, None)
                } else if name.starts_with("COLOR_")
                    && (a.width == 3 || a.width == 4)
                    && (a.component == FLOAT || a.normalized)
                {
                    let mut bytes = vec![0u8; n * 4];
                    for (i, &old) in order.iter().enumerate() {
                        for c in 0..a.width {
                            bytes[i * 4 + c] =
                                (a.value(old as usize, c).clamp(0.0, 1.0) * 255.0).round() as u8;
                        }
                    }
                    let s = b.stream(bytes, 4, Some(ARRAY_BUFFER));
                    b.accessor(s, 0, UBYTE, true, &a.kind, n, None, None)
                } else {
                    let size = a.data.len() / a.count.max(1);
                    let stride = size.div_ceil(4) * 4;
                    let s = b.stream(reordered(&a, &order, stride), stride, Some(ARRAY_BUFFER));
                    b.accessor(
                        s,
                        0,
                        a.component,
                        a.normalized,
                        &a.kind,
                        n,
                        a.min.clone(),
                        a.max.clone(),
                    )
                };
                out_attributes.insert(name.clone(), json!(made));
            }
            // (Its place among the accessors is set once the index stream's width is known.)
            index_accessors.push((indices.len(), triangles.len(), b.accessors.len()));
            b.accessors.push(Out {
                stream: usize::MAX,
                offset: 0,
                component: USHORT,
                normalized: false,
                kind: "SCALAR".into(),
                count: triangles.len(),
                min: None,
                max: None,
            });
            let mut out = p.as_object().cloned().unwrap_or_default();
            out.insert("attributes".into(), J::Object(out_attributes));
            out.insert("indices".into(), json!(b.accessors.len() - 1));
            out.insert("mode".into(), json!(4));
            out_prims.push(J::Object(out));
            indices.extend(triangles);
        }
        mesh["primitives"] = J::Array(out_prims);
    }

    // The triangles: 16-bit unless a primitive has more vertices than that holds.
    if !index_accessors.is_empty() {
        let wide = most_vertices > 65536;
        let stride = if wide { 4 } else { 2 };
        let mut bytes = Vec::with_capacity(indices.len() * stride);
        for &i in &indices {
            if wide {
                bytes.extend_from_slice(&i.to_le_bytes());
            } else {
                bytes.extend_from_slice(&(i as u16).to_le_bytes());
            }
        }
        b.streams.push(Stream {
            bytes,
            stride,
            triangles: true,
            target: Some(ELEMENT_ARRAY_BUFFER),
        });
        let stream = b.streams.len() - 1;
        for &(start, _, at) in &index_accessors {
            b.accessors[at].stream = stream;
            b.accessors[at].offset = start * stride;
            b.accessors[at].component = if wide { UINT } else { USHORT };
        }
    }

    // Animations: their keys and values as they are (an accessor two samplers share stays one).
    let mut animations = list(&g.json, "animations").to_vec();
    let mut moved: BTreeMap<usize, usize> = BTreeMap::new();
    for animation in &mut animations {
        let Some(samplers) = animation.get_mut("samplers").and_then(J::as_array_mut) else {
            continue;
        };
        for sampler in samplers {
            for key in ["input", "output"] {
                let old = uint(sampler, key).ok_or("an animation sampler without its accessors")?;
                let new = match moved.get(&old) {
                    Some(&new) => new,
                    None => {
                        let a = accessor(&g, &v, old)?;
                        let size = a.data.len() / a.count.max(1);
                        if !size.is_multiple_of(4) {
                            return Err("animation data that isn't in whole 4-byte steps".into());
                        }
                        let s = b.stream(a.data.clone(), size, None);
                        let new = b.accessor(
                            s,
                            0,
                            a.component,
                            a.normalized,
                            &a.kind,
                            a.count,
                            a.min.clone(),
                            a.max.clone(),
                        );
                        moved.insert(old, new);
                        new
                    }
                };
                sampler[key] = json!(new);
            }
        }
    }

    // The compressed data, and the file's two buffers: it, and what it decodes to (never loaded).
    let mut bin: Vec<u8> = Vec::new();
    let mut raw = 0usize;
    let mut buffer_views = Vec::new();
    for stream in &b.streams {
        let packed = compress(stream)?;
        while !bin.len().is_multiple_of(4) {
            bin.push(0);
        }
        let count = stream.bytes.len() / stream.stride;
        let mut view = Map::new();
        view.insert("buffer".into(), json!(1));
        view.insert("byteOffset".into(), json!(raw));
        view.insert("byteLength".into(), json!(stream.bytes.len()));
        if let Some(target) = stream.target {
            view.insert("target".into(), json!(target));
            if target == ARRAY_BUFFER {
                view.insert("byteStride".into(), json!(stream.stride));
            }
        }
        view.insert(
            "extensions".into(),
            json!({ MESHOPT: {
                "buffer": 0,
                "byteOffset": bin.len(),
                "byteLength": packed.len(),
                "byteStride": stream.stride,
                "count": count,
                "mode": if stream.triangles { "TRIANGLES" } else { "ATTRIBUTES" },
            } }),
        );
        buffer_views.push(J::Object(view));
        bin.extend_from_slice(&packed);
        raw += stream.bytes.len().div_ceil(4) * 4;
    }
    let accessors: Vec<J> = b
        .accessors
        .iter()
        .map(|a| {
            let mut out = Map::new();
            out.insert("bufferView".into(), json!(a.stream));
            out.insert("byteOffset".into(), json!(a.offset));
            out.insert("componentType".into(), json!(a.component));
            if a.normalized {
                out.insert("normalized".into(), json!(true));
            }
            out.insert("count".into(), json!(a.count));
            out.insert("type".into(), json!(a.kind));
            if let Some(min) = &a.min {
                out.insert("min".into(), min.clone());
            }
            if let Some(max) = &a.max {
                out.insert("max".into(), max.clone());
            }
            J::Object(out)
        })
        .collect();

    // A node under each node with a quantised mesh: the mesh, at the quantisation's offset and scale.
    let mut nodes = list(&g.json, "nodes").to_vec();
    let mut quantized = false;
    for i in 0..nodes.len() {
        let Some(mesh) = uint(&nodes[i], "mesh") else {
            continue;
        };
        let Some((centre, scale)) = frames.get(mesh).copied().flatten() else {
            continue;
        };
        quantized = true;
        let mut child = Map::new();
        if let Some(name) = nodes[i].get("name") {
            child.insert("name".into(), name.clone());
        }
        child.insert("mesh".into(), json!(mesh));
        child.insert("translation".into(), json!(centre));
        child.insert("scale".into(), json!([scale, scale, scale]));
        let at = nodes.len();
        nodes.push(J::Object(child));
        let node = nodes[i].as_object_mut().expect("a node is an object");
        node.remove("mesh");
        match node.get_mut("children").and_then(J::as_array_mut) {
            Some(children) => children.push(json!(at)),
            None => {
                node.insert("children".into(), json!([at]));
            }
        }
    }

    let root = json.as_object_mut().ok_or("its JSON isn't an object")?;
    root.insert("accessors".into(), J::Array(accessors));
    root.insert("bufferViews".into(), J::Array(buffer_views));
    root.insert(
        "buffers".into(),
        json!([
            { "byteLength": bin.len() },
            { "byteLength": raw, "extensions": { MESHOPT: { "fallback": true } } },
        ]),
    );
    if !meshes.is_empty() {
        root.insert("meshes".into(), J::Array(meshes));
    }
    if !nodes.is_empty() {
        root.insert("nodes".into(), J::Array(nodes));
    }
    if !animations.is_empty() {
        root.insert("animations".into(), J::Array(animations));
    }
    for key in ["extensionsUsed", "extensionsRequired"] {
        let mut names: Vec<String> = root
            .get(key)
            .and_then(J::as_array)
            .map(|a| {
                a.iter()
                    .filter_map(|n| n.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();
        for name in [MESHOPT, QUANTIZATION] {
            if (name == MESHOPT || quantized) && !names.iter().any(|n| n == name) {
                names.push(name.to_string());
            }
        }
        root.insert(key.into(), json!(names));
    }
    if let Some(asset) = root.get_mut("asset").and_then(J::as_object_mut) {
        let from = asset
            .get("generator")
            .and_then(J::as_str)
            .unwrap_or("unknown")
            .to_string();
        asset.insert(
            "generator".into(),
            json!(format!("cargo assets (from {from})")),
        );
    }
    Ok(write(&json, &bin))
}

// ---- reading a model back, to compare one with another ---------------------------------------
