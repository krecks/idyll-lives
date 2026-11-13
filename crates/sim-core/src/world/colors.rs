//! The colours a player gives an object’s parts.
//!
//! Split from `world.rs` (see `mod.rs`).

use super::*;

/// The colours a player gives an object's parts, 0xRRGGBB by the name of the model's material
/// they go on (`body`, `stone`, `chrome`); a part not named is as it's made. The sim doesn't know
/// the parts: the renderer colours only those the model lets the player paint.
pub type Colors = BTreeMap<String, u32>;

/// Parts one object may colour at most.
pub const MAX_COLORS: usize = 16;

/// An object's colours as a command gives them: at most `MAX_COLORS` parts, each named by a
/// material (`[a-z0-9_]`, up to 32 characters), each 0xRRGGBB.
pub(crate) fn check_colors(colors: Colors) -> Result<Colors, Error> {
    if colors.len() > MAX_COLORS {
        return Err(Error::new(format!(
            "too many colours: {} (at most {MAX_COLORS})",
            colors.len()
        )));
    }
    for (part, &color) in &colors {
        if !part_name(part) {
            return Err(Error::new(format!("no such part: {part:?}")));
        }
        if color > 0xFF_FFFF {
            return Err(Error::new(format!("no such colour: {color:#x}")));
        }
    }
    Ok(colors)
}

pub(crate) fn part_name(part: &str) -> bool {
    (1..=32).contains(&part.len())
        && part
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
}

/// An object's colours as saves and the view write them: `#rrggbb` by part.
pub fn colors_hex(colors: &Colors) -> BTreeMap<String, String> {
    colors
        .iter()
        .map(|(part, &c)| (part.clone(), color_hex(c)))
        .collect()
}

/// `colors_hex` read back, with an older file's one `color` on the `body` (the part every
/// paintable model's colour went on); what isn't a part or a colour is left out.
pub fn parse_colors(colors: &BTreeMap<String, String>, color: Option<&str>) -> Colors {
    let old = color
        .and_then(parse_color)
        .filter(|_| colors.is_empty())
        .map(|c| ("body".to_string(), c));
    let parts = colors
        .iter()
        .filter(|(part, _)| part_name(part))
        .filter_map(|(part, hex)| Some((part.clone(), parse_color(hex)?)));
    old.into_iter().chain(parts).take(MAX_COLORS).collect()
}

/// An object's colour as saves and the view write it: `#rrggbb`.
pub fn color_hex(color: u32) -> String {
    format!("#{color:06x}")
}

/// `color_hex` read back; None if it isn't one.
pub fn parse_color(hex: &str) -> Option<u32> {
    let digits = hex.strip_prefix('#')?;
    if digits.len() == 6 {
        u32::from_str_radix(digits, 16).ok()
    } else {
        None
    }
}
