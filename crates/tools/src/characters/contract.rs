//! Names the runtime and the generator agree on (docs/characters-format.md): morph channels,
//! hairstyles, garments and the parts every body has. `check` holds the build to them.

/// The 15 visemes in the voice engine's order (crates/voice/src/visemes.rs = web/src/voice/visemes.ts).
pub const VISEMES: [&str; 15] = [
    "sil", "PP", "FF", "TH", "DD", "kk", "CH", "SS", "nn", "RR", "aa", "E", "I", "O", "U",
];

/// ARKit face units (the subset face.ts uses: `UNITS` and the names in `MOODS`).
pub const UNITS: [&str; 43] = [
    "browDownLeft",
    "browDownRight",
    "browInnerUp",
    "browOuterUpLeft",
    "browOuterUpRight",
    "cheekSquintLeft",
    "cheekSquintRight",
    "eyeBlinkLeft",
    "eyeBlinkRight",
    "eyeLookDownLeft",
    "eyeLookDownRight",
    "eyeLookUpLeft",
    "eyeLookUpRight",
    "eyeSquintLeft",
    "eyeSquintRight",
    "eyeWideLeft",
    "eyeWideRight",
    "jawForward",
    "jawOpen",
    "mouthClose",
    "mouthDimpleLeft",
    "mouthDimpleRight",
    "mouthFrownLeft",
    "mouthFrownRight",
    "mouthFunnel",
    "mouthLeft",
    "mouthLowerDownLeft",
    "mouthLowerDownRight",
    "mouthPressLeft",
    "mouthPressRight",
    "mouthPucker",
    "mouthRollLower",
    "mouthRollUpper",
    "mouthShrugLower",
    "mouthShrugUpper",
    "mouthSmileLeft",
    "mouthSmileRight",
    "mouthStretchLeft",
    "mouthStretchRight",
    "mouthUpperUpLeft",
    "mouthUpperUpRight",
    "noseSneerLeft",
    "noseSneerRight",
];

/// The visemes, then the units.
pub fn channels() -> Vec<&'static str> {
    VISEMES.iter().chain(UNITS.iter()).copied().collect()
}

/// Hairstyles (household.ts HAIR_STYLES): each but `none` is a part `hair.<style>`; `hair.beard`
/// is the facial hair of the families that have it.
pub const HAIR_STYLES: [&str; 12] = [
    "short",
    "crop",
    "layered",
    "tousled",
    "sideSwept",
    "afro",
    "bob",
    "long",
    "ponytail",
    "braid",
    "bun",
    "none",
];
pub const BEARD_FAMILIES: [&str; 1] = ["male"];

/// Garments per family, in the creator's order (outfit.ts TOPS / BOTTOMS / SHOE_PARTS).
pub fn garments(family: &str) -> &'static [&'static str] {
    if family == "male" {
        &[
            "top.tee",
            "top.vneck",
            "top.polo",
            "top.long",
            "top.shirt",
            "top.jacket",
            "top.suit",
            "bottom.trousers",
            "bottom.shorts",
            "bottom.suit",
            "shoes.sneakers",
            "shoes.trainers",
            "shoes.boots",
            "shoes.dress",
        ]
    } else {
        &[
            "top.tee",
            "top.tank",
            "top.blouse",
            "top.long",
            "top.shirt",
            "top.jacket",
            "bottom.trousers",
            "bottom.shorts",
            "bottom.capri",
            "bottom.skirt",
            "shoes.sneakers",
            "shoes.trainers",
            "shoes.boots",
            "shoes.dress",
        ]
    }
}

/// Parts every body has (the face parts are characters.ts FACE_PARTS).
pub const FACE_PARTS: [&str; 5] = ["eyes", "brows", "lashes", "teeth", "tongue"];

pub fn required_parts() -> Vec<String> {
    ["body", "lod.body"]
        .iter()
        .chain(FACE_PARTS.iter())
        .map(|s| s.to_string())
        .collect()
}

pub fn hair_parts(family: &str) -> Vec<String> {
    let mut out: Vec<String> = HAIR_STYLES
        .iter()
        .filter(|&&s| s != "none")
        .map(|s| format!("hair.{s}"))
        .collect();
    if BEARD_FAMILIES.contains(&family) {
        out.push("hair.beard".into());
    }
    out
}
