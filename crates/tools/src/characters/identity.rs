//! A face per resident: the identity channels, morph channels after the expressions' that the runtime
//! sets once per resident (docs/characters-format.md; household.ts `Appearance.face`).
//!
//! A channel is one number of `Identity` (-1..1; all 0: the family's shared face). Its shape is a
//! difference: the base stage's head, face and lids built with that one number at 1 (head.rs and face.rs
//! read it where they shape the jaw, the chin, the cheeks, the nose, the brow, the eyes and the mouth)
//! minus the ones built with it at -1, halved, vertex by vertex (the topology is the same). The middle
//! difference, so the morph is as near the face built outright at -1 as at 1: a lid's seam slides over
//! the curved face, and a difference from the shared face alone would lift it off at the other end. What
//! only adds to the head (head.rs `Bump`) is the head with it minus the shared one, and its negative
//! weights take that shape away.
//!
//! The lids, the lashes, the brows, the lips, the mouth's plate and the teeth are made from the head's
//! frame, so they follow the skin; the beard, and any other hair that lies on the face, takes the skin's
//! shape where it lies (build.rs). The eyeballs turn on their bones, so a channel that moves or sizes
//! them says so beside its morphs (`Channel::eye`, rig.json `morphs.identity.eyes`), and the runtime
//! moves the bones.
//!
//! The cranium, the ears, the back of the head and the neck stay (`Mask`): hairlines, hair and collars
//! are fitted to the shared head. The check holds every hairstyle clear of the head at each channel's
//! two ends (check.rs).

use std::sync::Arc;

use super::body::{self, Surface};
use super::contract::IDENTITY;
use super::face::rest_parts;
use super::head::{HeadFrame, hairline_y, head_frame, head_frame_of};
use super::meshops::OnMesh;
use super::np::{PI, radians, smoothstep};
use super::rig::{BASE_STAGE, proportions};
use super::{Cache, V3};

/// A face of its own: how much more (or less) of each feature than the shared face has, -1..1.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Identity {
    pub jaw_width: f64,
    pub jaw_square: f64,
    pub chin_length: f64,
    pub chin_width: f64,
    pub cheekbones: f64,
    pub cheek_fullness: f64,
    pub nose_length: f64,
    pub nose_width: f64,
    pub nose_bridge: f64,
    pub brow_ridge: f64,
    pub eye_spacing: f64,
    pub eye_tilt: f64,
    pub eye_size: f64,
    pub mouth_width: f64,
    pub lip_fullness: f64,
    pub face_length: f64,
}

impl Identity {
    /// The shared face.
    pub const NONE: Identity = Identity {
        jaw_width: 0.0,
        jaw_square: 0.0,
        chin_length: 0.0,
        chin_width: 0.0,
        cheekbones: 0.0,
        cheek_fullness: 0.0,
        nose_length: 0.0,
        nose_width: 0.0,
        nose_bridge: 0.0,
        brow_ridge: 0.0,
        eye_spacing: 0.0,
        eye_tilt: 0.0,
        eye_size: 0.0,
        mouth_width: 0.0,
        lip_fullness: 0.0,
        face_length: 0.0,
    };

    /// The shared face with one channel (`contract::IDENTITY`) at `by`.
    pub fn of(channel: &str, by: f64) -> Identity {
        let mut id = Identity::NONE;
        *match channel {
            "id.jawWidth" => &mut id.jaw_width,
            "id.jawSquare" => &mut id.jaw_square,
            "id.chinLength" => &mut id.chin_length,
            "id.chinWidth" => &mut id.chin_width,
            "id.cheekbones" => &mut id.cheekbones,
            "id.cheekFullness" => &mut id.cheek_fullness,
            "id.noseLength" => &mut id.nose_length,
            "id.noseWidth" => &mut id.nose_width,
            "id.noseBridge" => &mut id.nose_bridge,
            "id.browRidge" => &mut id.brow_ridge,
            "id.eyeSpacing" => &mut id.eye_spacing,
            "id.eyeTilt" => &mut id.eye_tilt,
            "id.eyeSize" => &mut id.eye_size,
            "id.mouthWidth" => &mut id.mouth_width,
            "id.lipFullness" => &mut id.lip_fullness,
            "id.faceLength" => &mut id.face_length,
            other => panic!("no identity channel {other}"),
        } = by;
        id
    }
}

/// How far a life stage's face takes a resident's identity (the runtime scales the weights by it,
/// rig.json `faceScale`): the shapes are metres on the base stage's head, so a smaller head takes less,
/// and a baby's and a child's face, round and unformed, less again.
pub fn face_scale(family: &str, stage: &str) -> f64 {
    let hh = |s: &str| {
        let p = proportions(family, s);
        p.hh * p.h
    };
    let young = match stage {
        "baby" => 0.6,
        "child" => 0.8,
        _ => 1.0,
    };
    (hh(stage) / hh(BASE_STAGE)).min(1.0) * young
}

/// Shapes below these are left out of a channel (m; a normal's component): under a quantum of the
/// stored morphs, and the long tails of the smooth head's blends would move half the face by microns.
pub const TRIM: (f64, f64) = (4e-5, 0.008);

/// Where on the base stage's head a channel's shape is kept (1) and where it is left out (0): the face,
/// up to a little under the hairline, back to behind the jaw's corners, round the ears but not at them.
/// Fractions of the head's height: the fade under the hairline (from, to, below it), the ears' box
/// (from, to, away from it); degrees round the head: the fade towards its back.
pub struct Mask {
    h: Arc<HeadFrame>,
}

const UNDER_HAIRLINE: (f64, f64) = (0.12, 0.02);
const ROUND_EARS: (f64, f64) = (0.015, 0.09);
const TOWARDS_BACK: (f64, f64) = (100.0, 135.0);
/// The head's share of a skin vertex (body.rs regions) from which the shape fades in down the neck.
const NECK: (f64, f64) = (0.1, 0.7);

impl Mask {
    pub fn new(family: &str) -> Self {
        Mask {
            h: head_frame(family, BASE_STAGE),
        }
    }

    pub fn at(&self, p: V3) -> f64 {
        let h = &self.h;
        let hl = hairline_y(h, p);
        let up = 1.0
            - smoothstep(
                hl - UNDER_HAIRLINE.0 * h.hh,
                hl - UNDER_HAIRLINE.1 * h.hh,
                p.y,
            );
        let az = (p.x - h.centre.x).atan2(p.z - h.centre.z).abs();
        let back = 1.0 - smoothstep(radians(TOWARDS_BACK.0), radians(TOWARDS_BACK.1), az);
        debug_assert!(az <= PI);
        let mut ear: f64 = 1.0;
        for [c, half] in h.ears {
            let d = ((p - c).abs() - half).sup(&V3::zeros()).norm();
            ear = ear.min(smoothstep(ROUND_EARS.0 * h.hh, ROUND_EARS.1 * h.hh, d));
        }
        up * back * ear
    }

    /// For a point of the skin (or a sample of it) the head has `share` of.
    fn skin(&self, p: V3, share: f64) -> f64 {
        self.at(p) * smoothstep(NECK.0, NECK.1, share)
    }
}

/// One channel's shape on a family's base stage (position and normal deltas at 1).
pub struct Channel {
    pub name: &'static str,
    /// The body's skin (`body`'s first vertices, body.rs `Surface::pos`).
    pub skin: (Vec<V3>, Vec<V3>),
    /// SPIKE (`hd.body`): the same on the level-2 skin (`Surface::fine_pos`).
    pub skin_hd: (Vec<V3>, Vec<V3>),
    /// The face's parts (face.rs `PARTS`; `body`: the lids and lips, after the skin).
    pub parts: Vec<(&'static str, Vec<V3>, Vec<V3>)>,
    /// The right eyeball: how far its centre moves (m; the left's x mirrored) and how much larger it is (of 1).
    pub eye: [f64; 4],
    /// The head's vertex samples (`HeadFrame::moved`): what the hair is checked against.
    pub samples: (Vec<V3>, Vec<V3>),
    /// How many of the `body` part's vertices are the lids' (its first; the lips follow).
    pub lids: usize,
}

impl Channel {
    /// The largest move (m).
    pub fn reach(&self) -> f64 {
        self.parts
            .iter()
            .map(|p| &p.1)
            .chain([&self.skin.0])
            .flatten()
            .map(|d| d.norm())
            .fold(0.0, f64::max)
    }
}

static CHANNELS: Cache<String, Vec<Channel>> = Cache::new();

/// The identity channels of a family (`contract::IDENTITY` order).
pub fn channels(family: &str) -> Arc<Vec<Channel>> {
    CHANNELS.get(family.to_string(), || {
        let base = Built::new(family, &Identity::NONE);
        let mask = Mask::new(family);
        IDENTITY
            .iter()
            .map(|&name| channel(family, name, &base, &mask))
            .collect()
    })
}

/// A face (the shared one, or one channel at 1) as built: its body, head and the face's parts at rest.
struct Built {
    srf: Arc<Surface>,
    head: Arc<HeadFrame>,
    parts: Vec<(&'static str, Vec<V3>, Vec<V3>)>,
    /// The lids' vertices (the first of the `body` part's), and their shading normals (build.rs
    /// `lid_shading`: the skin's without its sockets, where they lie).
    lids: Vec<V3>,
}

impl Built {
    fn new(family: &str, id: &Identity) -> Self {
        let (srf, head) = if *id == Identity::NONE {
            (
                body::surface(family, BASE_STAGE),
                head_frame(family, BASE_STAGE),
            )
        } else {
            let srf = body::surface_of(family, BASE_STAGE, id);
            let head = head_frame_of(family, BASE_STAGE, id, &srf);
            (Arc::new(srf), Arc::new(head))
        };
        let (parts, n_lids) = rest_parts(family, head.clone(), id);
        let skin = OnMesh::new(&srf.unsocketed, &body::tris(family), srf.pos.len());
        let lids = parts[0].1[..n_lids]
            .iter()
            .map(|&q| {
                let (t, w) = skin.at(q);
                (srf.nor[t[0]] * w[0] + srf.nor[t[1]] * w[1] + srf.nor[t[2]] * w[2]).normalize()
            })
            .collect();
        Built {
            srf,
            head,
            parts,
            lids,
        }
    }
}

fn trim(d: &mut (Vec<V3>, Vec<V3>)) {
    for (p, n) in d.0.iter_mut().zip(d.1.iter_mut()) {
        if p.abs().max() < TRIM.0 {
            *p = V3::zeros();
        }
        if n.abs().max() < TRIM.1 {
            *n = V3::zeros();
        }
    }
}

/// The skin in an eye's socket (hidden under the lids) goes with the eyeball where a channel moves or
/// sizes it, rather than by the difference of two sockets carved in different places (a socket's wall is
/// steep: its difference is centimetres, and its negative a spike through the eye): fully where the
/// socket is carved deeper than `SOCKET.1` (m), not at all where less than `SOCKET.0`.
const SOCKET: (f64, f64) = (0.0001, 0.001);

fn channel(family: &str, name: &'static str, base: &Built, mask: &Mask) -> Channel {
    let hi = Built::new(family, &Identity::of(name, 1.0));
    // (At -1 a shape that only adds is the shared face: its difference is one-sided.)
    let low = Built::new(family, &Identity::of(name, -1.0));
    let one_sided = low.srf.pos == base.srf.pos && low.parts == base.parts;
    let (lo, k) = if one_sided { (base, 1.0) } else { (&low, 0.5) };
    let b = &base.srf;
    let ri_head = body::region("head");
    let e = (hi.head.eyes[1] - lo.head.eyes[1]) * k;
    let size = (hi.head.eye_r - lo.head.eye_r) * k / base.head.eye_r;
    let eye_moves = e.norm() > 1e-9 || size.abs() > 1e-9;
    // (SPIKE: the same shape on level 1's vertices and on level 2's, `hd.body`.)
    let shape = |pos: &[V3],
                 bare: &[V3],
                 region: &[[f64; body::NR]],
                 hp: &[V3],
                 lp: &[V3],
                 hn: &[V3],
                 ln: &[V3]| {
        let w: Vec<f64> = (0..pos.len())
            .map(|v| mask.skin(pos[v], region[v][ri_head]))
            .collect();
        let mut skin: (Vec<V3>, Vec<V3>) = (
            (0..pos.len())
                .map(|v| {
                    let d = (hp[v] - lp[v]) * k;
                    let carved = smoothstep(SOCKET.0, SOCKET.1, bare[v].z - pos[v].z);
                    if !eye_moves || carved == 0.0 {
                        return d * w[v];
                    }
                    let c = base.head.eyes[usize::from(pos[v].x > 0.0)];
                    let with_eye = V3::new(e.x * c.x.signum(), e.y, e.z) + (pos[v] - c) * size;
                    (d + (with_eye - d) * carved) * w[v]
                })
                .collect(),
            (0..pos.len())
                .map(|v| (hn[v] - ln[v]) * (k * w[v]))
                .collect(),
        );
        trim(&mut skin);
        skin
    };
    let skin = shape(
        &b.pos,
        &b.unsocketed,
        &b.region,
        &hi.srf.pos,
        &lo.srf.pos,
        &hi.srf.nor,
        &lo.srf.nor,
    );
    let skin_hd = shape(
        &b.fine_pos,
        &b.fine_bare,
        &b.fine_region,
        &hi.srf.fine_pos,
        &lo.srf.fine_pos,
        &hi.srf.fine_nor,
        &lo.srf.fine_nor,
    );
    let parts = base
        .parts
        .iter()
        .zip(hi.parts.iter().zip(&lo.parts))
        .map(|((part, bp, _), ((_, hp, hn), (_, lp, ln)))| {
            assert!(
                bp.len() == hp.len() && bp.len() == lp.len(),
                "identity: {part} differs"
            );
            let w: Vec<f64> = bp.iter().map(|&p| mask.at(p) * k).collect();
            let mut d: (Vec<V3>, Vec<V3>) = (
                (0..bp.len()).map(|i| (hp[i] - lp[i]) * w[i]).collect(),
                (0..bp.len())
                    .map(|i| {
                        // (The lids are shaded as the skin they lie on, build.rs.)
                        if *part == "body" && i < base.lids.len() {
                            (hi.lids[i] - lo.lids[i]) * w[i]
                        } else {
                            (hn[i] - ln[i]) * w[i]
                        }
                    })
                    .collect(),
            );
            trim(&mut d);
            (*part, d.0, d.1)
        })
        .collect();
    let nv = b.head_samples.len() - b.head_tri_ix.len();
    let ws: Vec<f64> = (0..nv)
        .map(|i| mask.skin(b.head_samples[i], b.head_share[i]) * k)
        .collect();
    let samples = (
        (0..nv)
            .map(|i| (hi.srf.head_samples[i] - lo.srf.head_samples[i]) * ws[i])
            .collect(),
        (0..nv)
            .map(|i| (hi.srf.head_normals[i] - lo.srf.head_normals[i]) * ws[i])
            .collect(),
    );
    Channel {
        name,
        skin,
        skin_hd,
        parts,
        eye: [e.x, e.y, e.z, size],
        samples,
        lids: base.lids.len(),
    }
}
