//! The quantisation a model may be written with.

/// A material named for a finish of leaves (`name@foliage`, `name@leafcards`, or `foliage` itself): its
/// model's geometry keeps its floats, as does one a manifest maps to such a finish.
pub fn foliage_name(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    if lower == "foliage" {
        return true;
    }
    ["@foliage", "@leafcards"].iter().any(|tag| {
        lower.match_indices(tag).any(|(at, _)| {
            lower[at + tag.len()..]
                .chars()
                .next()
                .is_none_or(|c| !(c.is_ascii_alphanumeric() || c == '_'))
        })
    })
}

/// 14 bits of -1..1 in a 16-bit integer (the top bits repeated below, so 1 is 32767).
pub fn quantize14(x: f32) -> i16 {
    let q = (x.clamp(-1.0, 1.0) * 8191.0).round() as i32;
    let m = q.unsigned_abs();
    let e = ((m << 2) | (m >> 11)) as i32;
    (if q < 0 { -e } else { e }) as i16
}

/// 10 bits of -1..1 in a 16-bit integer.
pub fn quantize10(x: f32) -> i16 {
    let q = (x.clamp(-1.0, 1.0) * 511.0).round() as i32;
    let m = q.unsigned_abs();
    let e = ((m << 6) | (m >> 3)) as i32;
    (if q < 0 { -e } else { e }) as i16
}

/// 12 bits of 0..1 in a 16-bit integer.
pub fn quantize12(x: f32) -> u16 {
    let m = (x.clamp(0.0, 1.0) * 4095.0).round() as u32;
    ((m << 4) | (m >> 8)) as u16
}
