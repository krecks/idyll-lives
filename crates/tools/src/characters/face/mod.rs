//! The residents' faces: small feature shapes on a head that never deforms. Contract:
//!
//! - `build_face(family, stage, head)`: parts `eyes`, `lashes`, `brows`, `teeth`, `tongue` (pos,
//!   nor, idx, dense, fields, tone); `body`, the pieces merged into `body` (lids, lips), skinned
//!   to Head, with their tone; `morphs` per part and channel (dpos, dnor) (`body` indexes the
//!   merged pieces only); `follow`, the lip lines and their morphs (the beard copies them).
//!
//! - Eyes: a sphere per eye on its eye bone, its pole to the front so the rings run round the iris.
//!   Fields xy = offset from the iris centre on the eye's front (m), z = iris radius, w = pupil
//!   radius; the shader draws sclera, iris (the resident's eye colour), pupil and a catchlight.
//! - Lids: upper and lower skin over the eye's socket, from a seam on the face round the hollow carved
//!   for the eye (head.rs `sockets`) to their edges, almond-shaped curves on the eyeball meeting on its
//!   front at the corners, with a rounded rim; in between they lie on the face as it is without the
//!   socket, or on the eye's bulge where that stands out (`Face::lid_z`), so a closed lid is the face
//!   with the eye's round under it. build.rs shades them and gives them the tone of the face they lie
//!   on, the upper lid a crease (tone.y) above its edge; lashes a dark band along the upper rim (a flick
//!   at the outer corner on women). Blinks meet a little below the middle; squints raise the lower
//!   lid's middle most.
//! - Brows: flattened capsules along the brow line, on the skin, thickest at the inner end.
//! - Mouth: full upper and lower lip tubes meeting at the corners, the upper with a cupid's bow
//!   (tone.x tints them); between them a dark plate (tone.y = 1) with the tongue on it, and the
//!   upper and lower teeth. Everything inside the mouth stays between the lips' inner lines, so a
//!   closed mouth shows nothing of it.
//!
//! Every piece is made by one function of the face's parameters (`pieces`), with a fixed vertex
//! layout; a channel is the pieces made with its parameters minus the pieces at rest, so units and
//! visemes are authored as shapes (`unit_params`, `VISEME_PARAMS`), not as sums of deltas. Units
//! are at ARKit strength (face.ts uses them unscaled). Morphs are built at the base stage only and
//! shared by the other stages (the runtime adds them to every stage).

use std::collections::BTreeMap;
use std::sync::Arc;

use nalgebra::{DMatrix, DVector};

use super::contract::expressions;
use super::head::{HeadFrame, front_z, head_frame, smin};
use super::identity::Identity;
use super::kdtree::KdTree;
use super::meshops::smooth_normals;
use super::np::{PI, clip, interp, linspace, lstsq, radians, sign, smoothstep};
use super::prims::{OPEN, caps, ellipsoid, grid, loft, orient, rot, rot_x};
use super::rig::BASE_STAGE;
use super::{Mesh, V3, merge, skin, v3};

mod geometry;

/// The adult head height the millimetres below are measured on; other heads scale them.
const ADULT_HH: f64 = 0.2723;

/// Per family: lip half-heights (upper, lower) and depth (mm), how far the lips stand off the skin,
/// brow thickness, arch and taper, lash length and outer-corner flick, the lids' resting edges.
struct Style {
    lip_u: f64,
    lip_l: f64,
    lip_z: f64,
    proud: f64,
    brow_h: f64,
    brow_z: f64,
    arch: f64,
    brow_tip: f64,
    /// Where along the brow its arch peaks (the exponent of its run: 0.8 a third of the way out, 1.5
    /// two thirds).
    arch_at: f64,
    lash: f64,
    flick: f64,
    upper: f64,
    lower: f64,
    tilt: f64,
    /// Both lids' edges turned about the eye's axis (degrees; + the outer corner lower): a face of its own.
    cant: f64,
}

/// What a face of its own (identity.rs `Identity`, each of its numbers -1..1) takes on at 1: fuller lips
/// (of their heights, their depth and how far they stand off the skin), the eyes' outer corners higher
/// (degrees both lids' edges turn, `Style::cant`), and larger eyes open wider (degrees: the upper lid's edge higher, the
/// lower's lower).
const LIP_FULLNESS: f64 = 0.34;
const EYE_TILT: f64 = -7.0;
const EYE_OPEN: [f64; 2] = [-3.0, 2.0];

/// The family's style with a face of its own.
fn style_of(family: &str, id: &Identity) -> Style {
    let s = style(family);
    let full = 1.0 + LIP_FULLNESS * id.lip_fullness;
    Style {
        lip_u: s.lip_u * full,
        lip_l: s.lip_l * full,
        lip_z: s.lip_z * (1.0 + 0.5 * LIP_FULLNESS * id.lip_fullness),
        proud: s.proud * (1.0 + 0.5 * LIP_FULLNESS * id.lip_fullness),
        upper: s.upper + EYE_OPEN[0] * id.eye_size,
        lower: s.lower + EYE_OPEN[1] * id.eye_size,
        cant: EYE_TILT * id.eye_tilt,
        ..s
    }
}

fn style(family: &str) -> Style {
    if family == "male" {
        Style {
            lip_u: 2.8,
            lip_l: 3.6,
            lip_z: 3.0,
            proud: 1.5,
            brow_h: 3.5,
            brow_z: 1.5,
            arch: 0.6,
            brow_tip: 0.55,
            arch_at: 0.8,
            lash: 1.2,
            flick: 0.3,
            upper: 63.0,
            lower: 121.0,
            tilt: 0.0,
            cant: 0.0,
        }
    } else {
        Style {
            lip_u: 3.9,
            lip_l: 5.0,
            lip_z: 3.5,
            proud: 1.9,
            brow_h: 2.3,
            brow_z: 1.0,
            arch: 3.0,
            brow_tip: 0.3,
            arch_at: 1.5,
            lash: 2.0,
            flick: 1.6,
            upper: 61.0,
            lower: 120.0,
            tilt: -3.0,
            cant: 0.0,
        }
    }
}

/// How dark the upper lid's crease is (tone y).
const LID_CREASE: f64 = 0.7;
/// How much of the lip's colour the lips take (tone y, negative): the whole tube, less towards its edge
/// against the face and towards the line between the lips (which the occlusion shades).
const LIP_TINT: f64 = 0.9;

/// The lids reach out over the eye's socket to the face (`Face::seams`): their outer row lies
/// `LID_LIFT` mm in front of the face round the socket, `LID_MARGIN` eye radii past the hollow and at
/// least `LID_REACH` from the eye's axis; in between they lie on the face, or on a bulge round the
/// eyeball (`LID_BULGE` eye radii from its centre) where that stands out, blended over `LID_SOFT` eye
/// radii (`Face::lid_z`).
const LID_LIFT: f64 = 0.5;
const LID_MARGIN: f64 = 0.25;
const LID_REACH: f64 = 1.2;
const LID_BULGE: f64 = 1.1;
const LID_SOFT: f64 = 0.06;
/// Where the rows between the seam and the edge lie, from the seam (0) to the edge (1): upper, lower.
const LID_ROWS: ([f64; 2], f64) = ([0.45, 0.78], 0.5);
/// Past `LID_CORNER.0` (of 90 degrees round the eye from its middle) the lid edges turn towards each
/// other (`LID_CORNER.1` degrees times the square of how far past), so the lids meet on the eye's front at
/// about 63 degrees rather than past its sides, where only the hollow behind it would show; past that,
/// where they overlap by `LID_SHUT` degrees at rest, their edges lie on the face.
const LID_CORNER: (f64, f64) = (0.4, 200.0);
const LID_SHUT: (f64, f64) = (6.0, 14.0);

/// Lid columns: azimuth round the eye (degrees, 0 = front, + towards the outer corner).
fn lid_theta() -> Vec<f64> {
    linspace(-104.0, 104.0, 9)
}

fn lash_theta() -> Vec<f64> {
    linspace(-54.0, 58.0, 9)
}

/// Lip columns along the mouth (-1 = the resident's left corner), denser at the corners.
fn lip_s() -> Vec<f64> {
    linspace(-PI / 2.0, PI / 2.0, 11)
        .iter()
        .map(|v| v.sin())
        .collect()
}

/// The upper lip's cupid's bow: how much its top edge rises (0..1) along the mouth (-1..1), two
/// peaks under the philtrum's ridges with a dip between.
fn bow(s: f64) -> f64 {
    (-((s.abs() - 0.3) / 0.2).powi(2)).exp()
}

/// A tooth's width (m): what the lines between the teeth are spaced by (the shader's, tone.z).
const TOOTH: f64 = 0.0075;

fn teeth_s() -> Vec<f64> {
    linspace(-0.62, 0.62, 9)
}

fn tongue_s() -> Vec<f64> {
    linspace(-0.6, 0.6, 7)
}

/// How high the cupid's bow's peaks stand (of the upper lip's half-height).
const BOW: f64 = 0.22;

/// How far the lower lip drops with the jaw fully open (mm).
const JAW: f64 = 26.0;

/// Face parameters (all 0 at rest): per side pairs [left, right] or scalars (see `pieces`).
#[derive(Clone, Copy, Default)]
pub struct Params {
    blink: [f64; 2],
    wide: [f64; 2],
    squint: [f64; 2],
    cheek: [f64; 2],
    look_down: [f64; 2],
    look_up: [f64; 2],
    brow_down: [f64; 2],
    brow_inner: f64,
    brow_outer: [f64; 2],
    sneer: [f64; 2],
    smile: [f64; 2],
    frown: [f64; 2],
    dimple: [f64; 2],
    stretch: [f64; 2],
    press: [f64; 2],
    low_down: [f64; 2],
    up_up: [f64; 2],
    jaw: f64,
    jaw_fwd: f64,
    close: f64,
    funnel: f64,
    pucker: f64,
    left: f64,
    roll_l: f64,
    roll_u: f64,
    shrug_l: f64,
    shrug_u: f64,
    tuck: f64,
    tongue: f64,
}

impl Params {
    /// Sets parameter `name` (face.py's names); `side` None sets both sides of a pair.
    fn set(&mut self, name: &str, v: f64, side: Option<usize>) {
        let pair = |p: &mut [f64; 2]| match side {
            Some(s) => p[s] = v,
            None => *p = [v, v],
        };
        match name {
            "blink" => pair(&mut self.blink),
            "wide" => pair(&mut self.wide),
            "squint" => pair(&mut self.squint),
            "cheek" => pair(&mut self.cheek),
            "lookDown" => pair(&mut self.look_down),
            "lookUp" => pair(&mut self.look_up),
            "browDown" => pair(&mut self.brow_down),
            "browOuter" => pair(&mut self.brow_outer),
            "sneer" => pair(&mut self.sneer),
            "smile" => pair(&mut self.smile),
            "frown" => pair(&mut self.frown),
            "dimple" => pair(&mut self.dimple),
            "stretch" => pair(&mut self.stretch),
            "press" => pair(&mut self.press),
            "lowDown" => pair(&mut self.low_down),
            "upUp" => pair(&mut self.up_up),
            "browInner" => self.brow_inner = v,
            "jaw" => self.jaw = v,
            "jawFwd" => self.jaw_fwd = v,
            "close" => self.close = v,
            "funnel" => self.funnel = v,
            "pucker" => self.pucker = v,
            "left" => self.left = v,
            "rollL" => self.roll_l = v,
            "rollU" => self.roll_u = v,
            "shrugL" => self.shrug_l = v,
            "shrugU" => self.shrug_u = v,
            "tuck" => self.tuck = v,
            "tongue" => self.tongue = v,
            _ => panic!("no face parameter {name}"),
        }
    }

    fn of(kw: &[(&str, f64)]) -> Params {
        let mut p = Params::default();
        for &(k, v) in kw {
            p.set(k, v, None);
        }
        p
    }
}

/// The parameters of ARKit unit `c`.
fn unit_params(c: &str) -> Params {
    let (name, side) = match ["Left", "Right"]
        .iter()
        .enumerate()
        .find(|(_, s)| c.ends_with(*s) && c != "mouthLeft")
    {
        Some((k, s)) => (&c[..c.len() - s.len()], Some(k)),
        None => (c, None),
    };
    let key = match name {
        "eyeBlink" => "blink",
        "eyeWide" => "wide",
        "eyeSquint" => "squint",
        "cheekSquint" => "cheek",
        "eyeLookDown" => "lookDown",
        "eyeLookUp" => "lookUp",
        "browDown" => "browDown",
        "browInnerUp" => "browInner",
        "browOuterUp" => "browOuter",
        "noseSneer" => "sneer",
        "mouthSmile" => "smile",
        "mouthFrown" => "frown",
        "mouthDimple" => "dimple",
        "mouthStretch" => "stretch",
        "mouthPress" => "press",
        "mouthLowerDown" => "lowDown",
        "mouthUpperUp" => "upUp",
        "jawOpen" => "jaw",
        "jawForward" => "jawFwd",
        "mouthClose" => "close",
        "mouthFunnel" => "funnel",
        "mouthPucker" => "pucker",
        "mouthLeft" => "left",
        "mouthRollLower" => "rollL",
        "mouthRollUpper" => "rollU",
        "mouthShrugLower" => "shrugL",
        "mouthShrugUpper" => "shrugU",
        other => panic!("no unit {other}"),
    };
    let mut p = Params::default();
    match side {
        None => p.set(key, 1.0, None),
        Some(s) => {
            p.set(key, 0.0, None);
            p.set(key, 1.0, Some(s));
        }
    }
    p
}

/// Visemes (the voice engine's order) as mouth shapes. Clear, slightly exaggerated shapes read at
/// a distance: open vowels drop the jaw, the wide ones show the teeth, the round ones push the
/// lips out.
fn viseme_params(c: &str) -> Option<&'static [(&'static str, f64)]> {
    Some(match c {
        "sil" => &[],
        "PP" => &[
            ("press", 1.0),
            ("rollL", 0.25),
            ("rollU", 0.25),
            ("shrugL", 0.3),
        ],
        "FF" => &[("jaw", 0.07), ("tuck", 1.0), ("upUp", 0.35)],
        "TH" => &[("jaw", 0.16), ("tongue", 1.0), ("upUp", 0.15)],
        "DD" => &[("jaw", 0.22), ("upUp", 0.25), ("lowDown", 0.15)],
        "kk" => &[("jaw", 0.3), ("upUp", 0.1)],
        "CH" => &[
            ("jaw", 0.12),
            ("funnel", 0.65),
            ("upUp", 0.4),
            ("lowDown", 0.4),
        ],
        "SS" => &[
            ("jaw", 0.05),
            ("stretch", 0.5),
            ("upUp", 0.45),
            ("lowDown", 0.45),
        ],
        "nn" => &[("jaw", 0.16), ("upUp", 0.15)],
        "RR" => &[("jaw", 0.17), ("funnel", 0.45)],
        "aa" => &[("jaw", 0.62), ("upUp", 0.1)],
        "E" => &[("jaw", 0.34), ("stretch", 0.45), ("upUp", 0.25)],
        "I" => &[
            ("jaw", 0.2),
            ("stretch", 0.6),
            ("upUp", 0.35),
            ("lowDown", 0.25),
        ],
        "O" => &[("jaw", 0.45), ("funnel", 0.85)],
        "U" => &[("jaw", 0.16), ("pucker", 0.95)],
        _ => return None,
    })
}

/// A lip line, its half-height and depth per column.
type Lip = (Vec<V3>, Vec<f64>, Vec<f64>);

/// One face (a family at a stage): where its features are, and how they are shaped.
pub struct Face {
    h: Arc<HeadFrame>,
    st: Style,
    mm: f64,
    hw: f64,
    m0: (f64, f64),
    mcoef: Vec<f64>,
    mfloor: f64,
    /// The face round the eyes without their sockets (the head's `bare` triangles there), and the base
    /// stage's head and its face so (`lid_z`).
    bare: Vec<[V3; 3]>,
    base: (Arc<HeadFrame>, Vec<[V3; 3]>),
    /// Per eye, upper and lower lid: where each lid column meets the face (`seams`).
    seams: [[Vec<V3>; 2]; 2],
}

/// Those of a head's triangles (`tris`: its `bare` or its `tris`) round its eyes.
fn round_eyes(head: &HeadFrame, tris: &[[V3; 3]]) -> Vec<[V3; 3]> {
    let (e, r) = (head.eyes[1], head.eye_r);
    tris.iter()
        .filter(|t| {
            t.iter()
                .any(|p| (p.y - e.y).abs() < 3.0 * r && (p.x.abs() - e.x).abs() < 3.0 * r)
        })
        .copied()
        .collect()
}

fn dot7(a: &[f64; 7], c: &[f64]) -> f64 {
    let mut s = 0.0;
    for k in 0..7 {
        s += a[k] * c[k];
    }
    s
}

/// Open ends: the corners are thin enough not to show them.
fn lip_tube(lip: &Lip) -> (Vec<V3>, Vec<usize>) {
    let m = loft(&lip.0, &lip.1, &lip.2, 8, v3(0.0, 1.0, 0.0), OPEN);
    (m.pos, m.idx)
}

type Pieces = BTreeMap<&'static str, (Vec<V3>, Vec<usize>)>;

/// Every morphing piece of the face for parameters p: name -> (pos, idx).
fn pieces(face: &Face, p: &Params) -> Pieces {
    let mut out: Pieces = BTreeMap::new();
    for k in 0..2 {
        let (up, lo, lash) = face.lids(p, k);
        let eye = face.h.eyes[k];
        for (name, g) in [
            (["lidU0", "lidU1"][k], up),
            (["lidL0", "lidL1"][k], lo),
            (["lash0", "lash1"][k], lash),
        ] {
            let (pos, idx) = grid(&g, false);
            let outv: Vec<V3> = pos.iter().map(|q| q - eye).collect();
            let idx = orient(&pos, idx, &outv);
            out.insert(name, (pos, idx));
        }
        let b = face.brow(p, k);
        let m = loft(&b.0, &b.1, &b.2, 6, v3(0.0, 1.0, 0.0), caps(1));
        out.insert(["brow0", "brow1"][k], (m.pos, m.idx));
    }
    let (lips, plate, teeth, tongue) = face.mouth(p);
    out.insert("lipU", lip_tube(&lips[0]));
    out.insert("lipL", lip_tube(&lips[1]));
    let [tu, tl] = teeth;
    for (name, g) in [
        ("plate", plate),
        ("teethU", tu),
        ("teethL", tl),
        ("tongue", tongue),
    ] {
        let (pos, idx) = grid(&g, false);
        let fwd = vec![v3(0.0, 0.0, 1.0); pos.len()];
        let idx = orient(&pos, idx, &fwd);
        out.insert(name, (pos, idx));
    }
    out
}

/// Which pieces make which part (in order).
const PARTS: [(&str, &[&str]); 5] = [
    (
        "body",
        &["lidU0", "lidL0", "lidU1", "lidL1", "lipU", "lipL"],
    ),
    ("lashes", &["lash0", "lash1"]),
    ("brows", &["brow0", "brow1"]),
    ("tongue", &["plate", "tongue"]),
    ("teeth", &["teethU", "teethL"]),
];

fn smooth_n(name: &str, pos: &[V3], idx: &[usize]) -> Vec<V3> {
    let n = smooth_normals(pos, idx);
    // The lids' shading normals lean towards the face's (they are skin round the eye, not a ball).
    let f = match &name[..4.min(name.len())] {
        "lidU" => Some(v3(0.0, 0.45, 1.0)),
        "lidL" => Some(v3(0.0, 0.35, 1.0)),
        _ => None,
    };
    match f {
        Some(f) => n
            .iter()
            .map(|q| {
                let m = q * 0.2 + f * 0.8 / f.norm();
                m / m.norm()
            })
            .collect(),
        None => n,
    }
}

/// Per piece tone: lips tinted (y < 0: how much of the lip's colour) and blushed (x), the mouth plate
/// dark (y = 1), the lids' crease dark (y > 0), the tongue red, teeth shaded at the sides.
fn tone(name: &str, pos: &[V3], idx: &[usize], face: &Face) -> Vec<[f64; 4]> {
    let n = pos.len();
    let mut t = vec![[0.0; 4]; n];
    if name.starts_with("lidU") {
        // The crease: a soft line along the upper lid's row above the edge, fading at the corners.
        let k = lid_theta().len();
        for j in 0..k {
            t[2 * k + j][1] = LID_CREASE * (PI * j as f64 / (k - 1) as f64).sin().sqrt();
        }
    } else if name == "lipU" || name == "lipL" {
        // The lips' colour (y < 0), fading at the mouth's corners into the skin and towards the line
        // between them: there the lips' inner sides face each other in shade, which the occlusion (w)
        // bakes, so the mouth's shape (a smile, a frown) reads even when it is a few pixels wide. And
        // (z, lips only) how far the lip turns down at rest, which the shader shades: the lip's own
        // shape, not which way the head faces (lying down, bent over).
        let nn = smooth_normals(pos, idx);
        for i in 0..n {
            let s = clip((pos[i].x - face.h.mouth[0]).abs() / face.hw, 0.0, 1.0);
            t[i][0] = (if name == "lipU" { 0.75 } else { 0.85 }) * (1.0 - 0.35 * s.powf(3.0));
            let ny = nn[i].y * (if name == "lipU" { -1.0 } else { 1.0 });
            let inner = clip(ny, 0.0, 1.0).powf(1.5);
            t[i][1] = -LIP_TINT * (1.0 - 0.45 * s.powf(3.0)) * (1.0 - 0.55 * inner);
            t[i][2] = clip(-nn[i].y, 0.0, 1.0);
        }
    } else if name == "plate" {
        for v in &mut t {
            v[1] = 1.0;
        }
    } else if name == "tongue" {
        let k = tongue_s().len();
        for (i, v) in t.iter_mut().enumerate() {
            v[0] = 1.0;
            v[1] = [0.45, 0.15, 0.05][i / k];
        }
    } else if name.starts_with("teeth") {
        for i in 0..n {
            let s = (pos[i].x - face.h.mouth[0]).abs() / face.hw;
            let c = clip(s / 0.62, 0.0, 1.0);
            t[i][1] = 0.05 + 0.5 * (c * c);
            // (The teeth counted from the middle: the shader draws a line at every whole number.)
            t[i][2] = (pos[i].x - face.h.mouth[0]) / TOOTH;
        }
    }
    t
}

/// Morph deltas of a part per channel: (dpos, dnor) per vertex.
pub type Deltas = BTreeMap<&'static str, (Vec<V3>, Vec<V3>)>;

/// One shape's deltas per part (`PARTS` order).
type PartDeltas = Vec<(&'static str, (Vec<V3>, Vec<V3>))>;

/// The lip lines (the beard follows them).
pub struct Follow {
    pub pos: Vec<V3>,
    pub upper: Vec<bool>,
    pub morphs: Vec<(&'static str, Vec<V3>)>,
}

pub struct FaceOut {
    /// eyes, lashes, brows, tongue, teeth (in that order).
    pub parts: Vec<(String, Mesh)>,
    /// The pieces merged into `body`: the lids (skin_tone) then the lips.
    pub body: Vec<Mesh>,
    /// Per part (in the order a channel first moves it): channel -> deltas.
    pub morphs: Vec<(String, Deltas)>,
    pub follow: Option<Follow>,
}

/// The face of a base stage's head at rest, part by part (`PARTS` order; `body`: the lids and lips merged
/// into it): positions and normals, and how many of `body`'s are the lids' (its first). `id`: a face of
/// its own on its own head (identity.rs takes its difference from the shared face's; the layout is the
/// same).
#[allow(clippy::type_complexity)]
pub fn rest_parts(
    family: &str,
    head: Arc<HeadFrame>,
    id: &Identity,
) -> (Vec<(&'static str, Vec<V3>, Vec<V3>)>, usize) {
    let face = Face::new(family, head.clone(), head, id);
    let rest = pieces(&face, &Params::default());
    let parts = PARTS
        .iter()
        .map(|&(part, names)| {
            let (mut pos, mut nor) = (Vec::new(), Vec::new());
            for &nm in names {
                pos.extend(&rest[nm].0);
                nor.extend(smooth_n(nm, &rest[nm].0, &rest[nm].1));
            }
            (part, pos, nor)
        })
        .collect();
    let lids = PARTS[0]
        .1
        .iter()
        .filter(|n| n.starts_with("lid"))
        .map(|n| rest[n].0.len())
        .sum();
    (parts, lids)
}

pub fn build_face(family: &str, stage: &str, head: Arc<HeadFrame>) -> FaceOut {
    let face = Face::new(
        family,
        head,
        head_frame(family, BASE_STAGE),
        &Identity::NONE,
    );
    let rest = pieces(&face, &Params::default());
    let mut parts = vec![("eyes".to_string(), face.eyeballs())];
    let mut body = Vec::new();
    for (part, names) in PARTS {
        let mut meshes = Vec::new();
        for &name in names {
            let (pos, idx) = &rest[name];
            let n = pos.len();
            meshes.push(Mesh {
                pos: pos.clone(),
                idx: idx.clone(),
                nor: smooth_n(name, pos, idx),
                dense: skin(n, "Head"),
                tone: tone(name, pos, idx, &face),
                fields: vec![[0.0; 4]; n],
                skin_tone: part == "body" && name.starts_with("lid"),
                ..Mesh::default()
            });
        }
        if part == "body" {
            body.extend(meshes);
        } else {
            parts.push((part.to_string(), merge(&meshes.iter().collect::<Vec<_>>())));
        }
    }
    if stage != BASE_STAGE {
        return FaceOut {
            parts,
            body,
            morphs: Vec::new(),
            follow: None,
        };
    }

    // Morphs: each channel's pieces minus the rest.
    let rest_n: BTreeMap<&str, Vec<V3>> = rest
        .iter()
        .map(|(&k, (p, i))| (k, smooth_n(k, p, i)))
        .collect();
    let deltas = |pc: &Pieces, base: Option<&Pieces>| -> PartDeltas {
        let (base, base_n): (&Pieces, BTreeMap<&str, Vec<V3>>) = match base {
            Some(b) => (
                b,
                b.iter()
                    .map(|(&k, (p, i))| (k, smooth_n(k, p, i)))
                    .collect(),
            ),
            None => (&rest, rest_n.clone()),
        };
        PARTS
            .iter()
            .map(|&(part, names)| {
                let mut dp = Vec::new();
                let mut dn = Vec::new();
                for &nm in names {
                    dp.extend(pc[nm].0.iter().zip(&base[nm].0).map(|(a, b)| a - b));
                    dn.extend(
                        smooth_n(nm, &pc[nm].0, &pc[nm].1)
                            .iter()
                            .zip(&base_n[nm])
                            .map(|(a, b)| a - b),
                    );
                }
                (part, (dp, dn))
            })
            .collect()
    };
    let mut chans: Vec<(&str, PartDeltas)> = Vec::new();
    for c in expressions() {
        let d = if let Some(vp) = viseme_params(c) {
            if vp.is_empty() {
                Vec::new()
            } else {
                deltas(&pieces(&face, &Params::of(vp)), None)
            }
        } else if c == "mouthClose" {
            // A corrective (as in ARKit): the lips closing over an open jaw.
            let open = pieces(&face, &Params::of(&[("jaw", 1.0)]));
            deltas(
                &pieces(&face, &Params::of(&[("jaw", 1.0), ("close", 1.0)])),
                Some(&open),
            )
        } else {
            deltas(&pieces(&face, &unit_params(c)), None)
        };
        chans.push((c, d));
    }
    let mut morphs: Vec<(String, Deltas)> = Vec::new();
    for (c, d) in chans {
        for (part, (dp, dn)) in d {
            let mx = dp
                .iter()
                .map(|v| v.abs().max())
                .fold(f64::NEG_INFINITY, f64::max);
            if mx > 1e-6 {
                match morphs.iter_mut().find(|e| e.0 == part) {
                    Some(e) => {
                        e.1.insert(c, (dp, dn));
                    }
                    None => {
                        let mut m = Deltas::new();
                        m.insert(c, (dp, dn));
                        morphs.push((part.to_string(), m));
                    }
                }
            }
        }
    }
    // The lip lines (the beard follows them).
    let lip_n: Vec<usize> = PARTS[0].1.iter().map(|n| rest[n].0.len()).collect();
    let k0: usize = lip_n[..4].iter().sum();
    let mut lips = rest["lipU"].0.clone();
    lips.extend(&rest["lipL"].0);
    let upper: Vec<bool> = (0..lips.len()).map(|i| i < rest["lipU"].0.len()).collect();
    let mut fm = Vec::new();
    if let Some((_, bm)) = morphs.iter().find(|e| e.0 == "body") {
        for c in expressions() {
            if let Some((dp, _)) = bm.get(c) {
                fm.push((c, dp[k0..k0 + lips.len()].to_vec()));
            }
        }
    }
    FaceOut {
        parts,
        body,
        morphs,
        follow: Some(Follow {
            pos: lips,
            upper,
            morphs: fm,
        }),
    }
}

/// The beard's morphs: each vertex copies the morphs of the lip vertices nearest it (the moustache
/// the upper lip's, the rest the lower lip's), fading with distance, so it moves with the mouth.
pub fn follow_mouth(beard: &Mesh, follow: &Follow, head: &HeadFrame) -> Deltas {
    let pos = &beard.pos;
    let above: Vec<bool> = pos.iter().map(|p| p.y > head.mouth[1]).collect();
    let mut fade = vec![0.0; pos.len()];
    // Per vertex: its 6 lip vertices and their weights.
    let mut near: Vec<Vec<(usize, f64)>> = vec![Vec::new(); pos.len()];
    for want_upper in [true, false] {
        let ids: Vec<usize> = (0..follow.pos.len())
            .filter(|&i| follow.upper[i] == want_upper)
            .collect();
        let tree = KdTree::new(&ids.iter().map(|&i| follow.pos[i]).collect::<Vec<_>>());
        for (v, p) in pos.iter().enumerate() {
            if above[v] != want_upper {
                continue;
            }
            // The lips are mirror-symmetric, so a vertex on the midline has its neighbours in
            // equidistant pairs; a pair straddling the 6th place shares it (the Python took
            // whichever rounding noise put first, so the moustache's middle could slide sideways).
            let nb = tree.query_shared(p, 6, 1e-9);
            let wt: Vec<f64> = nb
                .iter()
                .map(|&(d, _, share)| 1.0 / (d + 1e-4) * share)
                .collect();
            let s = wt.iter().fold(0.0, |a, w| a + w);
            near[v] = nb
                .iter()
                .zip(&wt)
                .map(|(&(_, k, _), w)| (ids[k], w / s))
                .collect();
            fade[v] = clip(
                1.0 - (nb[0].0 - 0.015 * head.hh) / (0.32 * head.hh),
                0.0,
                1.0,
            )
            .powf(1.2);
        }
    }
    let mut out = Deltas::new();
    for (c, dp) in &follow.morphs {
        let acc: Vec<V3> = (0..pos.len())
            .map(|v| {
                let mut a = V3::zeros();
                for &(k, w) in &near[v] {
                    a += dp[k] * w;
                }
                a * fade[v]
            })
            .collect();
        let mx = acc
            .iter()
            .map(|v| v.abs().max())
            .fold(f64::NEG_INFINITY, f64::max);
        if mx > 1e-6 {
            out.insert(c, (acc, vec![V3::zeros(); pos.len()]));
        }
    }
    out
}
