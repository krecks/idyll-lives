//! The file a model is written into: its buffers and its JSON.

use super::*;

impl File {
    pub(super) fn view(&mut self, bytes: &[u8], target: Option<u32>) -> usize {
        while !self.bin.len().is_multiple_of(4) {
            self.bin.push(0);
        }
        let offset = self.bin.len();
        self.bin.extend_from_slice(bytes);
        let target = target
            .map(|t| format!(",\"target\":{t}"))
            .unwrap_or_default();
        self.views.push(format!(
            "{{\"buffer\":0,\"byteLength\":{},\"byteOffset\":{offset}{target}}}",
            bytes.len()
        ));
        self.views.len() - 1
    }

    pub(super) fn floats(
        &mut self,
        values: &[f32],
        kind: &'static str,
        per: usize,
        bounds: bool,
        target: Option<u32>,
    ) -> usize {
        let bytes: Vec<u8> = values.iter().flat_map(|v| v.to_le_bytes()).collect();
        if target.is_none()
            && let Some(&i) = self.shared.get(&(bytes.clone(), kind))
        {
            return i;
        }
        let view = self.view(&bytes, target);
        // (no bounds for no values: they'd be written as Infinity, which isn't JSON)
        let range = if bounds && !values.is_empty() {
            let (mut lo, mut hi) = (vec![f32::INFINITY; per], vec![f32::NEG_INFINITY; per]);
            for v in values.chunks(per) {
                for i in 0..per {
                    lo[i] = lo[i].min(v[i]);
                    hi[i] = hi[i].max(v[i]);
                }
            }
            format!(",\"max\":{},\"min\":{}", nums(&hi), nums(&lo))
        } else {
            String::new()
        };
        self.accessors.push(format!(
            "{{\"bufferView\":{view},\"componentType\":{FLOAT},\"count\":{}{range},\"type\":\"{kind}\"}}",
            values.len() / per
        ));
        if target.is_none() {
            self.shared.insert((bytes, kind), self.accessors.len() - 1);
        }
        self.accessors.len() - 1
    }
}
