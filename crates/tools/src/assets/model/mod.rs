//! Models (`.glb`) made smaller, and read back for the check.
//!
//! - **Quantised geometry** (`KHR_mesh_quantization`): positions to 14 bits of the mesh's box (within a
//!   tenth of a millimetre on a 2 m sofa), normals to 10, texture coordinates within 0..1 to 12, each a 16-bit
//!   integer, not a 32-bit float; the baked shading (COLOR_0, a grey of 32 levels: `artkit`'s `shade`)
//!   to 8 bits, which keep every level. A model with foliage keeps its floats but for the colours (its
//!   leaf cards sit close: quantised, their edges moved).
//! - **A node under each node with a mesh**, named as it is, holds the mesh with the quantisation's
//!   offset and scale: the named nodes keep their own places (the pivots the game takes from them,
//!   `BuildOptions.parts`), the meshes their names. The game bakes each node's place into its vertices.
//! - **Vertices in the order that compresses best** (meshoptimizer's strip order, then first use), and
//!   every buffer view compressed with meshoptimizer (`EXT_meshopt_compression`: lossless on the
//!   quantised data; vertex codec version 0, which every decoder reads). Animations keep their floats.
//!
//! A model this doesn't handle (a skin, morph targets, images of its own, a second buffer) is left as it
//! is: the game loads the source. None of the shipped models is (`cargo assets check` says which).

use serde_json::{Map, Value as J, json};
use std::collections::{BTreeMap, HashMap};

mod accessors;
mod compress;
mod consts;
mod drift;
mod glb;
mod matrix;
mod optimize;
mod quantize;
mod read;
mod types;

use accessors::*;
use compress::*;
use consts::*;
pub use drift::*;
pub use glb::*;
use matrix::*;
pub use optimize::*;
pub use quantize::*;
pub use read::*;
pub use types::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quantised_values_fill_their_sixteen_bits() {
        assert_eq!(quantize14(1.0), 32767);
        assert_eq!(quantize14(-1.0), -32767);
        assert_eq!(quantize14(0.0), 0);
        // (glTF-Transform's own, which the game's models were made with until this tool: 1800 -> 7200, 7651 -> 30607.)
        assert_eq!(quantize14(1800.0 / 8191.0), 7200);
        assert_eq!(quantize14(7651.0 / 8191.0), 30607);
        assert_eq!(quantize10(1.0), 32767);
        assert_eq!(quantize10(-1.0), -32767);
        assert_eq!(quantize12(1.0), 65535);
        assert_eq!(quantize12(0.0), 0);
        // Read back as the spec says (value / 32767), a value is within half a step and what repeating the
        // top bits adds (under one sixteen-bit unit): three quarters of a 14-bit step at most.
        for i in -1000..=1000 {
            let x = i as f32 / 1000.0;
            assert!((quantize14(x) as f32 / 32767.0 - x).abs() <= 0.75 / 8191.0 + 1e-6);
            assert!((quantize10(x) as f32 / 32767.0 - x).abs() <= 0.5 / 511.0 + 4e-5);
            let u = (x + 1.0) / 2.0;
            assert!((quantize12(u) as f32 / 65535.0 - u).abs() <= 0.5 / 4095.0 + 2e-5);
        }
    }

    #[test]
    fn foliage_by_material_name() {
        assert!(foliage_name("leaves@foliage"));
        assert!(foliage_name("crown@leafcards"));
        assert!(foliage_name("Foliage"));
        assert!(foliage_name("a@foliage.dark"));
        assert!(!foliage_name("a@foliageish"));
        assert!(!foliage_name("trunk@bark"));
    }

    /// A quad of two triangles under a moved node, with a turn over two keys.
    fn sample(material: &str) -> Vec<u8> {
        let positions: [[f32; 3]; 4] = [
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 0.5, 0.0],
            [0.0, 0.5, 0.25],
        ];
        let normals: [[f32; 3]; 4] = [[0.0, 0.0, 1.0]; 4];
        let uvs: [[f32; 2]; 4] = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
        let colors: [[u16; 4]; 4] = [
            [65535, 65535, 65535, 65535],
            [32896, 32896, 32896, 65535],
            [0, 0, 0, 65535],
            [8224, 8224, 8224, 65535],
        ];
        let indices: [u16; 6] = [0, 1, 2, 0, 2, 3];
        let times: [f32; 2] = [0.0, 1.0];
        let turns: [[f32; 4]; 2] = [[0.0, 0.0, 0.0, 1.0], [0.0, 0.70710677, 0.0, 0.70710677]];
        let mut bin: Vec<u8> = Vec::new();
        let mut views = Vec::new();
        let mut push = |bytes: Vec<u8>, bin: &mut Vec<u8>| {
            while !bin.len().is_multiple_of(4) {
                bin.push(0);
            }
            views.push(json!({ "buffer": 0, "byteOffset": bin.len(), "byteLength": bytes.len() }));
            bin.extend(bytes);
        };
        push(
            positions
                .iter()
                .flatten()
                .flat_map(|v| v.to_le_bytes())
                .collect(),
            &mut bin,
        );
        push(
            normals
                .iter()
                .flatten()
                .flat_map(|v| v.to_le_bytes())
                .collect(),
            &mut bin,
        );
        push(
            uvs.iter().flatten().flat_map(|v| v.to_le_bytes()).collect(),
            &mut bin,
        );
        push(
            colors
                .iter()
                .flatten()
                .flat_map(|v| v.to_le_bytes())
                .collect(),
            &mut bin,
        );
        push(
            indices.iter().flat_map(|v| v.to_le_bytes()).collect(),
            &mut bin,
        );
        push(
            times.iter().flat_map(|v| v.to_le_bytes()).collect(),
            &mut bin,
        );
        push(
            turns
                .iter()
                .flatten()
                .flat_map(|v| v.to_le_bytes())
                .collect(),
            &mut bin,
        );
        let json = json!({
            "asset": { "version": "2.0", "generator": "test" },
            "scene": 0,
            "scenes": [{ "nodes": [0] }],
            "nodes": [{ "name": "door", "mesh": 0, "translation": [2.0, 0.0, -1.0] }],
            "materials": [{ "name": material }],
            "meshes": [{ "name": "door", "primitives": [{ "attributes": { "POSITION": 0, "NORMAL": 1, "TEXCOORD_0": 2, "COLOR_0": 3 }, "indices": 4, "material": 0 }] }],
            "animations": [{ "name": "open", "channels": [{ "sampler": 0, "target": { "node": 0, "path": "rotation" } }], "samplers": [{ "input": 5, "output": 6, "interpolation": "LINEAR" }] }],
            "accessors": [
                { "bufferView": 0, "componentType": 5126, "count": 4, "type": "VEC3", "min": [0.0, 0.0, 0.0], "max": [1.0, 0.5, 0.25] },
                { "bufferView": 1, "componentType": 5126, "count": 4, "type": "VEC3" },
                { "bufferView": 2, "componentType": 5126, "count": 4, "type": "VEC2" },
                { "bufferView": 3, "componentType": 5123, "normalized": true, "count": 4, "type": "VEC4" },
                { "bufferView": 4, "componentType": 5123, "count": 6, "type": "SCALAR" },
                { "bufferView": 5, "componentType": 5126, "count": 2, "type": "SCALAR", "min": [0.0], "max": [1.0] },
                { "bufferView": 6, "componentType": 5126, "count": 2, "type": "VEC4" },
            ],
            "bufferViews": views,
            "buffers": [{ "byteLength": bin.len() }],
        });
        write(&json, &bin)
    }

    #[test]
    fn a_model_comes_back_as_it_was_within_the_steps() {
        let source = sample("paint");
        let small = optimize(&source, false).unwrap();
        let g = parse(&small).unwrap();
        assert_eq!(g.json["extensionsRequired"], json!([MESHOPT, QUANTIZATION]));
        // The named node keeps its place and gets a node under it with the mesh.
        assert_eq!(g.json["nodes"][0]["translation"], json!([2.0, 0.0, -1.0]));
        assert!(g.json["nodes"][0].get("mesh").is_none());
        assert_eq!(g.json["nodes"][1]["mesh"], json!(0));
        assert_eq!(g.json["nodes"][1]["name"], json!("door"));
        let drift = compare(&read(&source).unwrap(), &read(&small).unwrap()).unwrap();
        assert_eq!(drift.triangles, 2);
        assert!(drift.position <= 0.5 * 0.75 / 8191.0 + 1e-6, "{drift:?}");
        // A corner in the model's space: the node's place is in it.
        let back = read(&small).unwrap();
        assert!(
            back.meshes[0][0]
                .corners
                .iter()
                .any(|c| (c.position[0] - 3.0).abs() < 1e-4)
        );
    }

    #[test]
    fn a_model_with_foliage_keeps_its_floats() {
        let source = sample("crown@leafcards");
        let small = optimize(&source, false).unwrap();
        let g = parse(&small).unwrap();
        assert_eq!(g.json["extensionsRequired"], json!([MESHOPT]));
        assert_eq!(g.json["nodes"][0]["mesh"], json!(0));
        let drift = compare(&read(&source).unwrap(), &read(&small).unwrap()).unwrap();
        assert_eq!((drift.position, drift.normal, drift.uv), (0.0, 0.0, 0.0));
    }

    #[test]
    fn a_change_is_found() {
        let source = read(&sample("paint")).unwrap();
        let mut moved = read(&optimize(&sample("paint"), false).unwrap()).unwrap();
        moved.meshes[0][0].corners[0].position[1] += 0.01;
        assert!(compare(&source, &moved).is_err());
        let mut shaded = read(&optimize(&sample("paint"), false).unwrap()).unwrap();
        shaded.meshes[0][0].corners[0].color[0] ^= 1;
        assert!(compare(&source, &shaded).is_err());
        let mut keyed = read(&optimize(&sample("paint"), false).unwrap()).unwrap();
        keyed.animations[1][0] ^= 1;
        assert!(compare(&source, &keyed).is_err());
    }

    #[test]
    fn what_it_does_not_handle_is_left() {
        let mut g = parse(&sample("paint")).unwrap();
        g.json["skins"] = json!([{ "joints": [0] }]);
        assert_eq!(
            optimize(&write(&g.json, &g.bin), false).unwrap_err(),
            "a skin"
        );
    }
}
