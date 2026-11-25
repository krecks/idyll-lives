//! Materials. A model's colours are a palette: one material a colour, named for what it is, each with the game
//! finish it is drawn with (docs/design/objects.md, "Materials, finishes ..."). The game takes only a material's
//! colour from the file; the finish decides how it shines, and is said once: the pack's `materials` map is
//! written from the same table (`Palette::finishes`).
//!
//!   palette! { M: body = ("#F3F3F1", gloss), handle = ("#D0D3D6", chrome) }
//!   cube!("door", (0.7, 0.04, 1.1), material = M.body);

use crate::json::Json;
use std::collections::BTreeMap;

/// The side a mirror's face reflects to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mirror {
    Front,
    Back,
}

/// What a palette colour says after its finish (the pack entry's material fields).
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Extra {
    pub mirror: Option<Mirror>,
    /// A lamp's shade or a screen that glows, in this colour.
    pub glow: Option<[u8; 3]>,
    /// A screen lit only while it is used (needs `glow`).
    pub screen: Option<bool>,
    /// A plain finish shining more or less than the finish does, 0..1.
    pub roughness: Option<f64>,
    pub metallic: Option<f64>,
}

/// One material: its name is the key the pack maps to a finish.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mat {
    pub name: &'static str,
    /// sRGB, as the game's manifests write colours.
    pub colour: [u8; 3],
    pub finish: &'static str,
    pub extra: Extra,
}

const fn hex_digit(c: u8) -> u8 {
    match c {
        b'0'..=b'9' => c - b'0',
        b'a'..=b'f' => c - b'a' + 10,
        b'A'..=b'F' => c - b'A' + 10,
        _ => panic!("a colour is #rrggbb"),
    }
}

/// `#rrggbb` as its three bytes.
pub const fn rgb_of(hex: &str) -> [u8; 3] {
    let b = hex.as_bytes();
    let at = if !b.is_empty() && b[0] == b'#' { 1 } else { 0 };
    if b.len() - at != 6 {
        panic!("a colour is #rrggbb");
    }
    [
        hex_digit(b[at]) * 16 + hex_digit(b[at + 1]),
        hex_digit(b[at + 2]) * 16 + hex_digit(b[at + 3]),
        hex_digit(b[at + 4]) * 16 + hex_digit(b[at + 5]),
    ]
}

/// A colour as the manifests write it.
pub fn hex_of(rgb: [u8; 3]) -> String {
    format!("#{:02X}{:02X}{:02X}", rgb[0], rgb[1], rgb[2])
}

/// sRGB 0..1 to linear 0..1 (the sRGB transfer function): glTF's colour factors are linear.
pub fn linear(c: f64) -> f64 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        crate::math::pow((c + 0.055) / 1.055, 2.4)
    }
}

impl Mat {
    pub const fn new(name: &'static str, hex: &str, finish: &'static str) -> Mat {
        Mat {
            name,
            colour: rgb_of(hex),
            finish,
            extra: Extra {
                mirror: None,
                glow: None,
                screen: None,
                roughness: None,
                metallic: None,
            },
        }
    }

    /// Under another name than its field in `palette!`: a name the language keeps for itself (`crate`, `box`,
    /// `type`, `move`) is written `crate_ = ("#A87547", wood, named("crate"))`. The material is `M.crate_`; in the
    /// file, the pack and the palette it is `crate`.
    pub const fn named(mut self, name: &'static str) -> Mat {
        self.name = name;
        self
    }

    /// Mirror glass, reflecting to `side`.
    pub const fn mirror(mut self, side: Mirror) -> Mat {
        self.extra.mirror = Some(side);
        self
    }

    /// Glowing in `hex`.
    pub const fn glow(mut self, hex: &str) -> Mat {
        self.extra.glow = Some(rgb_of(hex));
        self
    }

    /// A screen: its glow shows only while it is used.
    pub const fn screen(mut self) -> Mat {
        self.extra.screen = Some(true);
        self
    }

    pub const fn roughness(mut self, r: f64) -> Mat {
        self.extra.roughness = Some(r);
        self
    }

    pub const fn metallic(mut self, m: f64) -> Mat {
        self.extra.metallic = Some(m);
        self
    }

    /// Linear RGB, 0..1.
    pub fn linear(self) -> [f64; 3] {
        [
            linear(self.colour[0] as f64 / 255.0),
            linear(self.colour[1] as f64 / 255.0),
            linear(self.colour[2] as f64 / 255.0),
        ]
    }

    /// The entry's fields for this material: its finish and what it says after it.
    fn entry(self) -> Json {
        let mut fields = vec![(
            "finish".to_string(),
            Json::Str(format!("material.finish.{}", self.finish)),
        )];
        if let Some(side) = self.extra.mirror {
            fields.push((
                "mirror".into(),
                Json::Str(
                    if side == Mirror::Front {
                        "front"
                    } else {
                        "back"
                    }
                    .into(),
                ),
            ));
        }
        if let Some(glow) = self.extra.glow {
            fields.push(("glow".into(), Json::Str(hex_of(glow))));
        }
        if let Some(screen) = self.extra.screen {
            fields.push(("screen".into(), Json::Bool(screen)));
        }
        if let Some(r) = self.extra.roughness {
            fields.push(("roughness".into(), Json::Num(r)));
        }
        if let Some(m) = self.extra.metallic {
            fields.push(("metallic".into(), Json::Num(m)));
        }
        Json::Obj(fields)
    }
}

/// A finish of the base manifest (`material.finish.<name>`), as far as a build needs it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Finish {
    pub roughness: f64,
    pub metallic: f64,
    /// Whether it is drawn with a texture: its parts get UVs (`uv`).
    pub textured: bool,
}

/// The base manifest's finishes by name, handed in by the runner.
pub type Finishes = BTreeMap<String, Finish>;

/// A family's colours, in the order they were given.
#[derive(Clone, Debug, Default)]
pub struct Palette {
    pub colours: Vec<Mat>,
}

impl Palette {
    pub fn new(colours: &[Mat]) -> Palette {
        Palette {
            colours: colours.to_vec(),
        }
    }

    /// The material named `name`.
    pub fn get(&self, name: &str) -> Mat {
        match self.colours.iter().find(|m| m.name == name) {
            Some(m) => *m,
            None => panic!("no colour {name:?} in the palette"),
        }
    }

    pub fn has(&self, name: &str) -> bool {
        self.colours.iter().any(|m| m.name == name)
    }

    /// The palette with some colours changed (finishes stay) or added.
    pub fn with(&self, changes: &[Mat]) -> Palette {
        let mut out = self.clone();
        for c in changes {
            match out.colours.iter_mut().find(|m| m.name == c.name) {
                Some(m) => *m = *c,
                None => out.colours.push(*c),
            }
        }
        out
    }

    /// The palette with the colour of each (name, #rrggbb) changed, its finish kept.
    pub fn recoloured(&self, changes: &[(&str, &str)]) -> Palette {
        let mut out = self.clone();
        for (name, hex) in changes {
            match out.colours.iter_mut().find(|m| m.name == *name) {
                Some(m) => m.colour = rgb_of(hex),
                None => panic!("no colour {name:?} in the palette"),
            }
        }
        out
    }

    /// What is wrong with the palette against the game's `finishes`, if anything: a finish it hasn't, a screen
    /// without a glow, a roughness out of range.
    pub fn check(&self, finishes: &Finishes) -> Result<(), String> {
        for m in &self.colours {
            if !finishes.contains_key(m.finish) {
                let all: Vec<&str> = finishes.keys().map(String::as_str).collect();
                return Err(format!(
                    "{}: no finish {:?} (one of {})",
                    m.name,
                    m.finish,
                    all.join(", ")
                ));
            }
            for (k, v) in [
                ("roughness", m.extra.roughness),
                ("metallic", m.extra.metallic),
            ] {
                if v.is_some_and(|v| !(0.0..=1.0).contains(&v)) {
                    return Err(format!("{}: `{k}` is a number from 0 to 1", m.name));
                }
            }
            if m.extra.screen.is_some() && m.extra.glow.is_none() {
                return Err(format!("{}: a screen needs a `glow` colour", m.name));
            }
        }
        Ok(())
    }

    /// {material: {"finish": ...}} for the materials a model uses, sorted (the pack entry's `materials`).
    pub fn finishes(&self, names: &[&str]) -> Result<Json, String> {
        let mut names: Vec<&str> = names.to_vec();
        names.sort_unstable();
        names.dedup();
        let unknown: Vec<&str> = names.iter().copied().filter(|n| !self.has(n)).collect();
        if !unknown.is_empty() {
            return Err(format!(
                "materials not in the palette: {}",
                unknown.join(", ")
            ));
        }
        Ok(Json::Obj(
            names
                .iter()
                .map(|n| (n.to_string(), self.get(n).entry()))
                .collect(),
        ))
    }
}

/// A family's palette as a value with a field a colour: `palette! { M: body = ("#F3F3F1", gloss) }` makes `M`,
/// and `M.body` is that material. `M.palette()` is the list the family is given.
/// A material is named as its field, or as its `named(..)` says where the field cannot be the name.
#[macro_export]
macro_rules! palette {
    ($name:ident : $($field:ident = ($hex:expr, $finish:ident $(, $more:ident ( $($arg:expr),* ))* )),+ $(,)?) => {
        #[allow(non_camel_case_types, non_snake_case)]
        #[derive(Clone, Copy, Debug)]
        pub struct $name {
            $(pub $field: $crate::materials::Mat,)+
        }
        #[allow(non_upper_case_globals)]
        pub const $name: $name = $name {
            $($field: $crate::materials::Mat::new(stringify!($field), $hex, stringify!($finish))$(.$more($($arg),*))*,)+
        };
        impl $name {
            pub fn palette(&self) -> $crate::materials::Palette {
                $crate::materials::Palette::new(&[$(self.$field),+])
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    palette! { M: body = ("#F3F3F1", gloss), glass = ("#DCE6EA", gloss, mirror(Mirror::Front)), shade = ("#f2e6cc", fabric, glow("#FFD9A0"), screen()) }

    #[test]
    fn a_palette_names_its_materials_and_writes_their_finishes() {
        assert_eq!(M.body.colour, [0xF3, 0xF3, 0xF1]);
        assert_eq!(M.shade.finish, "fabric");
        let p = M.palette();
        assert_eq!(
            p.finishes(&["shade", "body", "glass", "body"])
                .unwrap()
                .dumps(),
            r##"{"body": {"finish": "material.finish.gloss"}, "glass": {"finish": "material.finish.gloss", "mirror": "front"}, "shade": {"finish": "material.finish.fabric", "glow": "#FFD9A0", "screen": true}}"##
        );
        assert!(p.finishes(&["wood"]).is_err());
        let q = p.recoloured(&[("body", "#102030")]);
        assert_eq!(
            (q.get("body").colour, q.get("body").finish),
            ([0x10, 0x20, 0x30], "gloss")
        );
    }

    palette! { K:
        body = ("#C49A6C", wood),
        crate_ = ("#A87547", wood, named("crate")), // (`crate`, `box`, `type` and `move` are Rust's)
        box_ = ("#C9B79C", matte, named("box")),
        type_ = ("#2B2C2E", metal, named("type"), roughness(0.4)),
        move_ = ("#F2E6CC", fabric, glow("#FFD9A0"), named("move")),
        crate_dark = ("#7C5432", wood),
    }

    #[test]
    fn a_material_whose_name_the_language_keeps_is_named_after_its_finish() {
        assert_eq!(
            (K.crate_.name, K.box_.name, K.type_.name, K.move_.name),
            ("crate", "box", "type", "move")
        );
        // as the others in every other way
        assert_eq!(K.crate_, Mat::new("crate", "#A87547", "wood"));
        assert_eq!(K.type_.extra.roughness, Some(0.4));
        assert_eq!(K.move_.extra.glow, Some([0xFF, 0xD9, 0xA0]));
        // in the palette under its name, in its place
        let p = K.palette();
        let names: Vec<&str> = p.colours.iter().map(|m| m.name).collect();
        assert_eq!(
            names,
            ["body", "crate", "box", "type", "move", "crate_dark"]
        );
        assert!(p.has("crate") && !p.has("crate_") && p.get("box") == K.box_);
        assert_eq!(
            p.finishes(&["crate", "move", "body"]).unwrap().dumps(),
            r##"{"body": {"finish": "material.finish.wood"}, "crate": {"finish": "material.finish.wood"}, "move": {"finish": "material.finish.fabric", "glow": "#FFD9A0"}}"##
        );
        assert!(p.finishes(&["crate_"]).is_err());
        assert_eq!(
            p.recoloured(&[("crate", "#102030")]).get("crate").colour,
            [0x10, 0x20, 0x30]
        );
    }

    #[test]
    fn colours_go_linear_as_gltf_wants_them() {
        // the cypress's bark: what Blender 5.2 wrote for it (as singles)
        let m = Mat::new("bark", "#8E7A6A", "bark").linear();
        assert_eq!(
            [m[0] as f32, m[1] as f32, m[2] as f32],
            [0.270_497_8, 0.194_617_84, 0.144_128_47]
        );
        assert_eq!(linear(0.0), 0.0);
        assert_eq!(linear(1.0), 1.0);
    }
}
