//! The .glb container: its JSON and its binary.
//!
//! Split from `assets/model.rs` (see `mod.rs`).

use super::*;

/// A `.glb`'s two chunks.
pub struct Glb {
    pub json: J,
    pub bin: Vec<u8>,
}

pub fn parse(data: &[u8]) -> Result<Glb, String> {
    let u32_at = |at: usize| -> Result<usize, String> {
        data.get(at..at + 4)
            .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]) as usize)
            .ok_or_else(|| "the file ends early".to_string())
    };
    if data.get(..4) != Some(b"glTF") || u32_at(4)? != 2 {
        return Err("not a glTF 2 binary".into());
    }
    let json_len = u32_at(12)?;
    if data.get(16..20) != Some(b"JSON") {
        return Err("its first chunk isn't JSON".into());
    }
    let json_bytes = data
        .get(20..20 + json_len)
        .ok_or("its JSON chunk ends early")?;
    let json: J = serde_json::from_slice(json_bytes).map_err(|e| format!("its JSON: {e}"))?;
    let at = 20 + json_len;
    let bin = if at + 8 <= data.len() {
        let len = u32_at(at)?;
        if data.get(at + 4..at + 8) != Some(b"BIN\0") {
            return Err("its second chunk isn't BIN".into());
        }
        data.get(at + 8..at + 8 + len)
            .ok_or("its BIN chunk ends early")?
            .to_vec()
    } else {
        Vec::new()
    };
    Ok(Glb { json, bin })
}

pub fn write(json: &J, bin: &[u8]) -> Vec<u8> {
    let mut text = serde_json::to_vec(json).expect("JSON");
    while !text.len().is_multiple_of(4) {
        text.push(b' ');
    }
    let bin_len = bin.len().div_ceil(4) * 4;
    let total = 12 + 8 + text.len() + if bin_len > 0 { 8 + bin_len } else { 0 };
    let mut out = Vec::with_capacity(total);
    out.extend_from_slice(b"glTF");
    out.extend_from_slice(&2u32.to_le_bytes());
    out.extend_from_slice(&(total as u32).to_le_bytes());
    out.extend_from_slice(&(text.len() as u32).to_le_bytes());
    out.extend_from_slice(b"JSON");
    out.extend_from_slice(&text);
    if bin_len > 0 {
        out.extend_from_slice(&(bin_len as u32).to_le_bytes());
        out.extend_from_slice(b"BIN\0");
        out.extend_from_slice(bin);
        out.resize(total, 0);
    }
    out
}
