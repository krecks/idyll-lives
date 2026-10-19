//! Writes the character set (rig.json version 4 and its binaries; docs/characters-format.md).
//!
//! Each part in a family's `<family>.bin`: positions f32x3, normals i16x4 (w unused), joints u8x4,
//! weights u8x4 (summing to 255), fields i16x4 (x FIELD_SCALE), tone i16x4 (x TONE_SCALE), indices
//! u16; then, for parts with face morphs, `morphVerts` u16 (the vertices any channel moves), and
//! per channel in order the moved vertices' slots in that list (`morphSlots` u16), position deltas
//! (`morphPositions` i16x4, x MORPH_SCALE) and normal deltas (`morphNormals` i8x4, x
//! NORMAL_SCALE); `morph.ranges[c] .. ranges[c + 1]` are channel c's entries. No UVs.
//!
//! Each life stage other than the base: `<family>.<stage>.bin`, per part int16 position deltas
//! from the base (x STAGE_SCALE) and int8 normals (x 127).
//!
//! Meshes come in counter-clockwise from outside (prims.rs); Babylon's front faces are clockwise
//! in its left-handed space, so the writer turns every triangle round.

use std::io;
use std::path::Path;

use serde::Serialize;

use super::face::Deltas;
use super::json::OMap;
use super::meshops::top4;
use super::np::np_round;
use super::{Mesh, V3};

pub const FIELD_SCALE: f64 = 4000.0;
pub const TONE_SCALE: f64 = 4000.0;
pub const STAGE_SCALE: f64 = 5000.0;
pub const MORPH_SCALE: f64 = 30000.0;
pub const NORMAL_SCALE: f64 = 63.0;
const MORPH_EPS: f64 = 1e-5;

fn unit(n: &V3) -> V3 {
    n / n.norm().max(1e-9)
}

/// A part's sparse morph block.
#[derive(Clone)]
pub struct Morph {
    pub verts: Vec<usize>,
    pub slots: Vec<usize>,
    pub dpos: Vec<V3>,
    pub dnor: Vec<V3>,
    pub ranges: Vec<usize>,
}

/// The sparse morph block of a part of `n` vertices from per channel dense deltas.
pub fn pack_morph(n: usize, deltas: &Deltas, channels: &[&str]) -> Option<Morph> {
    let mut moved = vec![false; n];
    let mut used: Vec<(&str, Vec<bool>)> = Vec::new();
    for &c in channels {
        let Some((dp, dn)) = deltas.get(c) else { continue };
        let m: Vec<bool> = (0..n).map(|i| dp[i].abs().max() > MORPH_EPS || dn[i].abs().max() > MORPH_EPS * 100.0).collect();
        if m.iter().any(|&v| v) {
            for (a, &b) in moved.iter_mut().zip(&m) {
                *a |= b;
            }
            used.push((c, m));
        }
    }
    if !moved.iter().any(|&v| v) {
        return None;
    }
    let verts: Vec<usize> = (0..n).filter(|&i| moved[i]).collect();
    let mut slot_of = vec![usize::MAX; n];
    for (s, &v) in verts.iter().enumerate() {
        slot_of[v] = s;
    }
    let (mut slots, mut dpos, mut dnor, mut ranges) = (Vec::new(), Vec::new(), Vec::new(), vec![0]);
    for &c in channels {
        match used.iter().find(|e| e.0 == c) {
            Some((_, m)) => {
                let (dp, dn) = &deltas[c];
                let sel: Vec<usize> = (0..n).filter(|&i| m[i]).collect();
                slots.extend(sel.iter().map(|&i| slot_of[i]));
                dpos.extend(sel.iter().map(|&i| dp[i]));
                dnor.extend(sel.iter().map(|&i| dn[i]));
                ranges.push(ranges.last().unwrap() + sel.len());
            }
            None => ranges.push(*ranges.last().unwrap()),
        }
    }
    Some(Morph { verts, slots, dpos, dnor, ranges })
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MorphEntry {
    pub count: usize,
    pub ranges: Vec<usize>,
    pub morph_verts: usize,
    pub morph_slots: usize,
    pub morph_positions: usize,
    pub morph_normals: usize,
}

/// A part's entry in rig.json (byte offsets into the family's binary).
#[derive(Clone, Debug, Serialize)]
pub struct PartEntry {
    pub vertices: usize,
    pub indices: usize,
    pub positions: usize,
    pub normals: usize,
    pub joints: usize,
    pub weights: usize,
    pub fields: usize,
    pub tone: usize,
    pub index: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub morph: Option<MorphEntry>,
    pub min: [f64; 3],
    pub max: [f64; 3],
}

/// A part's entry in a stage's table.
#[derive(Clone, Debug, Serialize)]
pub struct StagePart {
    pub positions: usize,
    pub normals: usize,
}

/// A growing binary with 4-byte aligned arrays.
struct Blob(Vec<u8>);

impl Blob {
    fn add(&mut self, bytes: impl IntoIterator<Item = u8>) -> usize {
        while !self.0.len().is_multiple_of(4) {
            self.0.push(0);
        }
        let at = self.0.len();
        self.0.extend(bytes);
        at
    }
}

fn i16s(v: impl IntoIterator<Item = f64>) -> Vec<u8> {
    v.into_iter().flat_map(|x| (x as i16).to_le_bytes()).collect()
}

fn q(x: f64, scale: f64, lim: f64) -> f64 {
    (x * scale).round_ties_even().clamp(-lim, lim)
}

/// The family's main binary; returns the part table for rig.json.
pub fn write_parts(path: &Path, parts: &[(String, Mesh)]) -> io::Result<OMap<PartEntry>> {
    let mut blob = Blob(Vec::new());
    let mut table = OMap::new();
    for (name, part) in parts {
        let n = part.pos.len();
        assert!(n < 65536, "{name}");
        let (mut js, mut wq) = (Vec::with_capacity(n), Vec::with_capacity(n));
        for d in &part.dense {
            let (j, w) = top4(d);
            let mut q = w.map(|v| (v * 255.0).round_ties_even() as i64);
            let am = (0..4).fold(0, |m, i| if q[i] > q[m] { i } else { m });
            q[am] += 255 - q.iter().sum::<i64>();
            js.push(j);
            wq.push(q);
        }
        let positions = blob.add(part.pos.iter().flat_map(|p| [p.x as f32, p.y as f32, p.z as f32]).flat_map(f32::to_le_bytes));
        let normals = blob.add(i16s(part.nor.iter().flat_map(|nv| {
            let u = unit(nv);
            [(u.x * 32767.0).round_ties_even(), (u.y * 32767.0).round_ties_even(), (u.z * 32767.0).round_ties_even(), 0.0]
        })));
        let joints = blob.add(js.iter().flat_map(|j| j.map(|v| v as u8)));
        let weights = blob.add(wq.iter().flat_map(|w| w.map(|v| v as u8)));
        let zeros = vec![[0.0; 4]; n];
        let f = if part.fields.is_empty() { &zeros } else { &part.fields };
        let fields = blob.add(i16s(f.iter().flat_map(|v| v.map(|x| q(x, FIELD_SCALE, 32767.0)))));
        let t = if part.tone.is_empty() { &zeros } else { &part.tone };
        let tone = blob.add(i16s(t.iter().flat_map(|v| v.map(|x| q(x, TONE_SCALE, 32767.0)))));
        // Clockwise for Babylon.
        let index = blob.add(part.idx.chunks(3).flat_map(|t| [t[0], t[2], t[1]]).flat_map(|i| (i as u16).to_le_bytes()));
        let morph = part.morph.as_ref().map(|m| {
            let morph_verts = blob.add(m.verts.iter().flat_map(|&v| (v as u16).to_le_bytes()));
            let morph_slots = blob.add(m.slots.iter().flat_map(|&v| (v as u16).to_le_bytes()));
            let morph_positions = blob.add(i16s(m.dpos.iter().flat_map(|d| [q(d.x, MORPH_SCALE, 32767.0), q(d.y, MORPH_SCALE, 32767.0), q(d.z, MORPH_SCALE, 32767.0), 0.0])));
            let morph_normals = blob.add(m.dnor.iter().flat_map(|d| [q(d.x, NORMAL_SCALE, 127.0), q(d.y, NORMAL_SCALE, 127.0), q(d.z, NORMAL_SCALE, 127.0), 0.0]).map(|x| x as i8 as u8));
            MorphEntry { count: m.verts.len(), ranges: m.ranges.clone(), morph_verts, morph_slots, morph_positions, morph_normals }
        });
        let mn = part.pos.iter().fold(V3::repeat(f64::INFINITY), |a, p| a.inf(p));
        let mx = part.pos.iter().fold(V3::repeat(f64::NEG_INFINITY), |a, p| a.sup(p));
        table.insert(
            name.clone(),
            PartEntry {
                vertices: n,
                indices: part.idx.len(),
                positions,
                normals,
                joints,
                weights,
                fields,
                tone,
                index,
                morph,
                min: [mn.x, mn.y, mn.z].map(|v| np_round(v, 4)),
                max: [mx.x, mx.y, mx.z].map(|v| np_round(v, 4)),
            },
        );
    }
    std::fs::write(path, &blob.0)?;
    Ok(table)
}

/// A life stage: per part, int16 position deltas from the base (x STAGE_SCALE) and int8 normals.
pub fn write_stage(path: &Path, base: &[(String, Mesh)], stage: &[(String, Mesh)]) -> io::Result<OMap<StagePart>> {
    let mut blob = Blob(Vec::new());
    let mut table = OMap::new();
    for (name, part) in stage {
        let b = &base.iter().find(|e| &e.0 == name).expect("base part").1;
        assert_eq!(part.pos.len(), b.pos.len(), "{name}");
        let positions = blob.add(i16s(part.pos.iter().zip(&b.pos).flat_map(|(p, q0)| {
            let d = p - q0;
            [q(d.x, STAGE_SCALE, 32767.0), q(d.y, STAGE_SCALE, 32767.0), q(d.z, STAGE_SCALE, 32767.0), 0.0]
        })));
        let normals = blob.add(part.nor.iter().flat_map(|nv| {
            let u = unit(nv);
            [(u.x * 127.0).round_ties_even(), (u.y * 127.0).round_ties_even(), (u.z * 127.0).round_ties_even(), 0.0]
        }).map(|x| x as i8 as u8));
        table.insert(name.clone(), StagePart { positions, normals });
    }
    std::fs::write(path, &blob.0)?;
    Ok(table)
}

/// Python's `round(float(v), 6)` of each value.
pub fn r4(a: &[f64]) -> Vec<f64> {
    a.iter().map(|&v| super::np::py_round(v, 6)).collect()
}
