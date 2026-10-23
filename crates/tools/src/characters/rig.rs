//! The residents' skeleton: 33 bones, the names the runtime uses, and the joint positions of every
//! family (`male`, `female`) at every life stage.
//!
//! Conventions (see docs/characters-format.md):
//! - Babylon's left-handed space, metres, y up, the character faces +z, its left is -x;
//! - every rest local rotation is the identity, so a bone's frame at rest is model space and a
//!   joint's local translation is just the offset from its parent's joint;
//! - the rest pose is an A-pose: arms 45 degrees down, legs straight and vertical;
//! - the clips store the first `CLIP_BONES` bones (the eyes come last and are posed procedurally).
//!
//! `PROPORTIONS` holds the numbers each stage is built from (fractions of the height H unless
//! noted). Positions scale with H, then the stage's overrides (head size, hip height, shoulder and
//! hip width, arm length) apply.

use super::np::radians;
use super::{V3, v3};

pub const BONES: [&str; 33] = [
    "root", "pelvis", "spine_01", "spine_02", "spine_03", "neck_01", "Head",
    "clavicle_l", "upperarm_l", "lowerarm_l", "hand_l", "thumb_01_l", "thumb_02_l", "fingers_01_l", "fingers_02_l",
    "clavicle_r", "upperarm_r", "lowerarm_r", "hand_r", "thumb_01_r", "thumb_02_r", "fingers_01_r", "fingers_02_r",
    "thigh_l", "calf_l", "foot_l", "ball_l",
    "thigh_r", "calf_r", "foot_r", "ball_r",
    "eye_l", "eye_r",
];
pub const NB: usize = BONES.len();
pub const CLIP_BONES: usize = 31;
pub const FACE_BONES: [&str; 2] = ["eye_l", "eye_r"];

/// Bones the runtime looks up by name (characters.ts, reactions.ts, data.ts).
pub const RUNTIME_BONES: [&str; 22] = [
    "pelvis", "spine_01", "spine_02", "spine_03", "neck_01", "Head", "clavicle_l", "clavicle_r", "upperarm_l", "upperarm_r",
    "lowerarm_l", "lowerarm_r", "hand_l", "hand_r", "thigh_l", "thigh_r", "calf_l", "calf_r", "foot_l", "foot_r", "eye_l", "eye_r",
];

pub const FAMILIES: [&str; 2] = ["male", "female"];
pub const STAGES: [&str; 6] = ["baby", "child", "teen", "youngAdult", "adult", "elder"];
pub const BASE_STAGE: &str = "youngAdult";
/// The reference skeleton the clips are authored on (`sourcePelvis`, leg lengths for IK).
pub const REFERENCE: (&str, &str) = ("male", BASE_STAGE);

/// The index of bone `name`.
pub fn b(name: &str) -> usize {
    BONES.iter().position(|&n| n == name).unwrap_or_else(|| panic!("no bone {name}"))
}

pub fn head() -> usize {
    b("Head")
}

/// Each bone's parent (`PARENT` in rig.py), in `BONES` order.
const PARENT: [&str; NB] = [
    "", "root", "pelvis", "spine_01", "spine_02", "spine_03", "neck_01",
    "spine_03", "clavicle_l", "upperarm_l", "lowerarm_l", "hand_l", "thumb_01_l", "hand_l", "fingers_01_l",
    "spine_03", "clavicle_r", "upperarm_r", "lowerarm_r", "hand_r", "thumb_01_r", "hand_r", "fingers_01_r",
    "pelvis", "thigh_l", "calf_l", "foot_l",
    "pelvis", "thigh_r", "calf_r", "foot_r",
    "Head", "Head",
];

/// Parent index per bone (-1 for the root).
pub fn parents() -> Vec<i64> {
    PARENT.iter().map(|&p| if p.is_empty() { -1 } else { b(p) as i64 }).collect()
}

/// Per bone, its mirror's index (`_l` <-> `_r`; the rest map to themselves).
pub fn mirror_bones() -> [usize; NB] {
    std::array::from_fn(|i| {
        let n = BONES[i];
        match (n.strip_suffix("_l"), n.strip_suffix("_r")) {
            (Some(s), _) => b(&format!("{s}_r")),
            (_, Some(s)) => b(&format!("{s}_l")),
            _ => i,
        }
    })
}

/// Rendered heights before this rig (m): the stages must stay within 3% so cribs, kids' beds,
/// desks and seats still fit.
pub fn target_height(family: &str, stage: &str) -> f64 {
    let m = [0.663, 1.128, 1.512, 1.77, 1.755, 1.742];
    let f = [0.675, 1.094, 1.442, 1.67, 1.672, 1.661];
    let i = STAGES.iter().position(|&s| s == stage).expect("stage");
    if family == "male" { m[i] } else { f[i] }
}

/// A stage's numbers (fractions of H unless noted).
#[derive(Clone, Copy, Debug)]
pub struct Proportions {
    /// Height (m).
    pub h: f64,
    /// Head height (chin to crown).
    pub hh: f64,
    /// Thigh joints.
    pub hip: f64,
    /// Upperarm joint x (female 0.115).
    pub shoulder: f64,
    /// Thigh joint x (female 0.058).
    pub hip_w: f64,
    /// Upper arm length.
    pub upper: f64,
    /// Forearm length (elbow to wrist).
    pub lower: f64,
    /// Limb radius factor.
    pub girth: f64,
    /// Waist width factor.
    pub waist: f64,
    pub neck_z: f64,
    /// Extra belly depth (fraction of H).
    pub belly: f64,
}

/// Adult layout (fractions of H; left side, the right mirrors x).
pub const ADULT: Proportions = Proportions {
    h: 0.0,
    hh: 1.0 / 6.5,
    hip: 0.50,
    shoulder: 0.13,
    hip_w: 0.054,
    upper: 0.165,
    lower: 0.145,
    girth: 1.0,
    waist: 1.0,
    neck_z: -0.012,
    belly: 0.0,
};

/// The stage's numbers: `ADULT` with the family / stage overrides (H in metres).
pub fn proportions(family: &str, stage: &str) -> Proportions {
    let a = ADULT;
    let female = family == "female";
    match (stage, female) {
        ("baby", _) => Proportions { h: 0.67, hh: 1.0 / 3.4, hip: 0.38, shoulder: 0.12, hip_w: 0.075, upper: 0.12, lower: 0.105, girth: 1.35, belly: 0.03, ..a },
        ("child", false) => Proportions { h: 1.13, hh: 1.0 / 4.6, hip: 0.45, shoulder: 0.11, hip_w: 0.06, upper: 0.15, lower: 0.13, girth: 1.1, ..a },
        ("child", true) => Proportions { h: 1.09, hh: 1.0 / 4.6, hip: 0.45, shoulder: 0.11, hip_w: 0.062, upper: 0.15, lower: 0.13, girth: 1.1, ..a },
        ("teen", false) => Proportions { h: 1.51, hh: 1.0 / 6.0, hip: 0.50, shoulder: 0.12, hip_w: 0.054, upper: 0.162, lower: 0.142, girth: 0.95, ..a },
        ("teen", true) => Proportions { h: 1.44, hh: 1.0 / 6.0, hip: 0.50, shoulder: 0.105, hip_w: 0.058, upper: 0.162, lower: 0.142, girth: 0.95, ..a },
        ("youngAdult", false) => Proportions { h: 1.77, ..a },
        ("youngAdult", true) => Proportions { h: 1.67, shoulder: 0.115, hip_w: 0.058, ..a },
        ("adult", false) => Proportions { h: 1.76, waist: 1.08, belly: 0.006, ..a },
        ("adult", true) => Proportions { h: 1.66, shoulder: 0.115, hip_w: 0.058, waist: 1.08, belly: 0.006, ..a },
        ("elder", false) => Proportions { h: 1.74, hh: 1.0 / 6.4, hip: 0.49, shoulder: 0.13 * 0.95, waist: 1.12, belly: 0.01, neck_z: -0.002, ..a },
        ("elder", true) => Proportions { h: 1.64, hh: 1.0 / 6.4, hip: 0.49, shoulder: 0.115 * 0.95, hip_w: 0.058, waist: 1.12, belly: 0.01, neck_z: -0.002, ..a },
        _ => panic!("no stage {stage}"),
    }
}

/// Joint positions per bone (m), model space at rest (identity rotations).
pub fn joints(family: &str, stage: &str) -> [V3; NB] {
    let p = proportions(family, stage);
    let (h, hh) = (p.h, p.hh);
    let mut j = [V3::zeros(); NB];
    let mut set = |n: &str, v: V3| j[b(n)] = v;
    let chin = 1.0 - hh;
    let neck = chin - (if hh < 0.2 { 0.27 } else { 0.12 }) * hh;
    let hip = p.hip;
    let pelvis = hip + 0.035 * (hip / 0.5);
    set("root", v3(0.0, 0.0, 0.0));
    set("pelvis", v3(0.0, pelvis, 0.0));
    let span = neck - pelvis;
    set("spine_01", v3(0.0, pelvis + 0.15 * span, -0.005));
    set("spine_02", v3(0.0, pelvis + 0.37 * span, -0.01));
    set("spine_03", v3(0.0, pelvis + 0.61 * span, -0.01));
    set("neck_01", v3(0.0, neck, p.neck_z));
    set("Head", v3(0.0, chin + 0.06 * hh, 0.0));
    let sh_y = neck - 0.02 * (hh / ADULT.hh).powf(0.3);
    let d = v3(-radians(45.0).cos(), -radians(45.0).sin(), 0.0);
    for (s, sx) in [("l", 1.0), ("r", -1.0)] {
        let m = v3(sx, 1.0, 1.0);
        set(&format!("clavicle_{s}"), v3(-0.012 * sx, neck - 0.015, 0.01));
        let up = v3(-p.shoulder, sh_y, -0.005);
        let el = up + d * p.upper;
        let wr = el + d * p.lower;
        // Hands and feet are 1.15x real ones (the toy look), and relatively bigger on small people.
        let hs = (p.upper / ADULT.upper).powf(0.5);
        let k1 = wr + d * (0.055 * hs);
        let k2 = k1 + d * (0.03 * hs);
        // The thumb's base (at the wrist end of the palm's thumb side) and its knuckle off the
        // palm, in palm lengths along the hand, towards the palm and forward (the thumb side).
        let palm = v3(-d[1], d[0], 0.0);
        let t1 = wr + (d * 0.12 + palm * 0.12 + v3(0.0, 0.0, 0.28)) * (0.055 * hs);
        let t2 = wr + (d * 0.6 + palm * 0.25 + v3(0.0, 0.0, 0.45)) * (0.055 * hs);
        for (name, v) in [("upperarm", up), ("lowerarm", el), ("hand", wr), ("fingers_01", k1), ("fingers_02", k2), ("thumb_01", t1), ("thumb_02", t2)] {
            set(&format!("{name}_{s}"), v.component_mul(&m));
        }
        let hx = -p.hip_w * sx;
        let fs = (hip / ADULT.hip).powf(0.5);
        set(&format!("thigh_{s}"), v3(hx, hip, 0.0));
        set(&format!("calf_{s}"), v3(hx, hip * 0.56, 0.005));
        set(&format!("foot_{s}"), v3(hx, hip * 0.096, -0.01));
        set(&format!("ball_{s}"), v3(hx, 0.012 * fs, -0.01 + 0.095 * fs));
    }
    // Eyes: sphere centres, head-relative; their fronts about level with the face round them, so the
    // sockets (head.rs) are shallow.
    let eye_z = if family == "male" { 0.32 } else { 0.315 };
    for (s, sx) in [("l", 1.0), ("r", -1.0)] {
        set(&format!("eye_{s}"), v3(-0.17 * hh * sx, chin + 0.44 * hh, eye_z * hh));
    }
    j.map(|v| v * h)
}

/// The rest pose of a body: local translations, inverse bind matrices (column-major), joints.
pub struct Skeleton {
    pub joints: [V3; NB],
    pub rest_t: Vec<f64>,
    pub inverse_bind: Vec<f64>,
    pub height: f64,
    pub pelvis_scale: f64,
}

pub fn skeleton(family: &str, stage: &str) -> Skeleton {
    let j = joints(family, stage);
    let par = parents();
    let mut rest_t = Vec::with_capacity(NB * 3);
    let mut ibm = Vec::with_capacity(NB * 16);
    for bi in 0..NB {
        let t = if par[bi] >= 0 { j[bi] - j[par[bi] as usize] } else { j[bi] };
        rest_t.extend([t.x, t.y, t.z]);
        ibm.extend([1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, -j[bi].x, -j[bi].y, -j[bi].z, 1.0]);
    }
    let r = joints(REFERENCE.0, REFERENCE.1);
    let pv = b("pelvis");
    Skeleton { joints: j, rest_t, inverse_bind: ibm, height: proportions(family, stage).h, pelvis_scale: j[pv].y / r[pv].y }
}

/// The reference pelvis rest position (clip pelvis translations are relative to it).
pub fn source_pelvis() -> V3 {
    joints(REFERENCE.0, REFERENCE.1)[b("pelvis")]
}

/// Reference thigh and shin lengths (hip to knee, knee to ankle), for the clips' leg IK.
pub fn leg_lengths() -> (f64, f64) {
    let j = joints(REFERENCE.0, REFERENCE.1);
    ((j[b("calf_l")] - j[b("thigh_l")]).norm(), (j[b("foot_l")] - j[b("calf_l")]).norm())
}
