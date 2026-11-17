//! Moving parts: a door, lid or drawer that moves while residents use the object (docs/design/objects.md,
//! "Moving parts"). Each part is its own part with its origin on the hinge (`parts::hinged`), and each clip is one
//! track on it, named for the pack's `motions`.
//!
//! A clip's first key (frame 1) is the rest pose: where the file has the part, and the game draws it closed. Only
//! the part's rotation, location or scale move. `squash` goes from rest to open (the game plays it back);
//! `stages` is a crop's growth, held where its stage is. The file's clips are sampled every frame (30 a second,
//! `scene::FPS`) from frame 1 as 0 s, and the game samples them 60 times a second (twice each key).
//!
//! The curves are Blender's: between two keys a clip eases as Blender's default keys do (slow out of each key and
//! into the next), or by one of its easings (`Ease`).

use crate::math::{M4, V3};
use crate::scene::{self, FPS, Obj};
use crate::shapes::options;

/// What a channel keys: the part's place, its turn (Euler XYZ, radians) or its scale.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Path {
    Location,
    Rotation,
    Scale,
}

/// How a clip eases from one key to the next.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Ease {
    /// Blender's default: slow out of each key and into the next.
    #[default]
    Smooth,
    /// An even speed (spins, scrolls, loops).
    Linear,
    /// Quick off, slowing into the end.
    Out,
    In,
    InOut,
    /// A snap off, a long glide in (a drawer let run).
    ExpoOut,
    /// Past the end and back: a door swung open, settling.
    BackOut,
    /// Dropping onto the end and bouncing (a lid let fall).
    BounceOut,
}

/// How far past its end `BackOut` goes, as a share of the move (4°: a door opened 100°).
pub const OVERSHOOT: f64 = 0.04;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Key {
    pub frame: i32,
    pub value: f32,
    /// From this key to the next.
    pub ease: Ease,
}

/// One keyed number: a coordinate of the part's place, turn or scale.
#[derive(Clone, Debug)]
pub struct Channel {
    pub path: Path,
    pub index: usize,
    pub keys: Vec<Key>,
    /// `BackOut`'s overshoot.
    pub back: f64,
}

/// One clip on one part.
#[derive(Clone, Debug)]
pub struct Track {
    pub name: String,
    pub channels: Vec<Channel>,
    /// The frame of its last key.
    pub end: i32,
    /// Where it is most open, for a picture of it.
    pub peak: Option<i32>,
}

/// The frame `seconds` into a clip: frame 1 is 0 s, halves rounded up (a clip is never a frame short).
pub fn frame(seconds: f64) -> i32 {
    1 + (seconds * FPS + 0.5).floor() as i32
}

/// The frame of a clip's last key, for a clip that ends `seconds` in (`frame`, at least the second).
pub fn last_frame(seconds: f64) -> i32 {
    frame(seconds).max(2)
}

impl Channel {
    /// The channel's value at `frame`, as Blender evaluates the curve.
    pub fn at(&self, frame: f64) -> f64 {
        let keys = &self.keys;
        let first = keys[0];
        if frame <= first.frame as f64 {
            return first.value as f64;
        }
        let last = keys[keys.len() - 1];
        if frame >= last.frame as f64 {
            return last.value as f64;
        }
        let i = keys
            .iter()
            .rposition(|k| k.frame as f64 <= frame)
            .unwrap_or(0);
        let (a, b) = (keys[i], keys[i + 1]);
        let (begin, change) = (a.value as f64, b.value as f64 - a.value as f64);
        let duration = (b.frame - a.frame) as f64;
        let time = frame - a.frame as f64;
        let u = time / duration;
        match a.ease {
            Ease::Linear => begin + change * u,
            Ease::Smooth => {
                // a Bézier whose handles are a third of the way to the next key: time runs evenly along it
                let (h0, h1) = (self.handle(i, true), self.handle(i + 1, false));
                let (p0, p3) = (a.value as f64, b.value as f64);
                let (p1, p2) = (p0 + h0 * duration / 3.0, p3 - h1 * duration / 3.0);
                let v = 1.0 - u;
                v * v * v * p0 + 3.0 * v * v * u * p1 + 3.0 * v * u * u * p2 + u * u * u * p3
            }
            Ease::In => change * u * u + begin,
            Ease::Out => -change * u * (u - 2.0) + begin,
            Ease::InOut => {
                let t = u * 2.0;
                if t < 1.0 {
                    change / 2.0 * t * t + begin
                } else {
                    -change / 2.0 * ((t - 1.0) * (t - 3.0) - 1.0) + begin
                }
            }
            Ease::ExpoOut => {
                // (2^-10 at the end made up for, so it lands on the key)
                let (floor, scale) = (0.000_976_562_5, 1.0 / (1.0 - 0.000_976_562_5));
                change * (1.0 - (crate::math::pow(2.0, -10.0 * u) - floor) * scale) + begin
            }
            Ease::BackOut => {
                let s = back_amount(self.back);
                let t = u - 1.0;
                change * (t * t * ((s + 1.0) * t + s) + 1.0) + begin
            }
            Ease::BounceOut => {
                let k = 7.5625;
                let t = u;
                let y = if t < 1.0 / 2.75 {
                    k * t * t
                } else if t < 2.0 / 2.75 {
                    let t = t - 1.5 / 2.75;
                    k * t * t + 0.75
                } else if t < 2.5 / 2.75 {
                    let t = t - 2.25 / 2.75;
                    k * t * t + 0.9375
                } else {
                    let t = t - 2.625 / 2.75;
                    k * t * t + 0.984375
                };
                change * y + begin
            }
        }
    }

    /// The slope of the curve at key `i` (a unit a frame), on its `right` or its left: level at the clip's ends
    /// and at a key that is a peak or a trough, else through its neighbours, held inside them.
    fn handle(&self, i: usize, right: bool) -> f64 {
        let keys = &self.keys;
        if i == 0 || i + 1 == keys.len() {
            return 0.0;
        }
        let (prev, cur, next) = (keys[i - 1], keys[i], keys[i + 1]);
        let (y0, y1, y2) = (prev.value as f64, cur.value as f64, next.value as f64);
        let (d1, d2) = (y0 - y1, y2 - y1);
        if (d1 <= 0.0 && d2 <= 0.0) || (d1 >= 0.0 && d2 >= 0.0) {
            return 0.0;
        }
        let (da, db) = (
            (cur.frame - prev.frame) as f64,
            (next.frame - cur.frame) as f64,
        );
        // the slope through the neighbours, each side weighed alike
        let slope = ((y1 - y0) / da + (y2 - y1) / db) / 2.0;
        // a handle stays within its neighbour's value
        let (reach, limit) = if right {
            (db / 3.0, y2 - y1)
        } else {
            (da / 3.0, y1 - y0)
        };
        let rise = slope * reach;
        if rise.abs() > limit.abs() {
            limit / reach
        } else {
            slope
        }
    }
}

/// Blender's `back` for a back ease-out that goes `overshoot` (a share of the move) past its end: the peak of
/// 1 + (s + 1)u³ + su² (u = t - 1), found by halving.
pub fn back_amount(overshoot: f64) -> f64 {
    let peak = |s: f64| {
        let u = -2.0 * s / (3.0 * (s + 1.0));
        (s + 1.0) * u * u * u + s * u * u
    };
    let (mut lo, mut hi) = (0.0, 10.0);
    for _ in 0..60 {
        let mid = (lo + hi) / 2.0;
        if peak(mid) < overshoot {
            lo = mid
        } else {
            hi = mid
        }
    }
    (lo + hi) / 2.0
}

/// A part's (place, turn, scale) in clip `clip` at `frame`: its rest pose but for what the clip keys.
pub fn pose(obj: Obj, clip: Option<&str>, frame: f64) -> (V3, V3, V3) {
    obj.read(|o| {
        let (mut loc, mut rot, mut scale) = (o.loc, o.rot, o.scale);
        if let Some(track) = clip.and_then(|c| o.tracks.iter().find(|t| t.name == c)) {
            for ch in &track.channels {
                let v = ch.at(frame) as f32;
                match ch.path {
                    Path::Location => loc[ch.index] = v,
                    Path::Rotation => rot[ch.index] = v,
                    Path::Scale => scale[ch.index] = v,
                }
            }
        }
        (loc, rot, scale)
    })
}

/// Keys `obj`'s `path`[`index`] at the frames of `keys`, (frame, value) each, in its clip `clip`: several
/// channels of one clip share the track. `ease` from each key to the next.
pub fn key_channel(
    obj: Obj,
    clip: &str,
    path: Path,
    index: usize,
    keys: &[(i32, f64)],
    ease: Ease,
    overshoot: f64,
) -> Obj {
    let keys: Vec<Key> = keys
        .iter()
        .map(|&(frame, value)| Key {
            frame,
            value: value as f32,
            ease,
        })
        .collect();
    let end = keys[keys.len() - 1].frame;
    obj.edit(|o| {
        let channel = Channel {
            path,
            index,
            keys,
            back: overshoot,
        };
        match o.tracks.iter_mut().find(|t| t.name == clip) {
            Some(track) => {
                track
                    .channels
                    .retain(|c| !(c.path == path && c.index == index));
                track.channels.push(channel);
                track.end = track.end.max(end);
            }
            None => o.tracks.push(Track {
                name: clip.to_string(),
                channels: vec![channel],
                end,
                peak: None,
            }),
        }
    });
    obj
}

fn rest(obj: Obj, path: Path, index: usize) -> f64 {
    obj.read(|o| match path {
        Path::Location => o.loc[index],
        Path::Rotation => o.rot[index],
        Path::Scale => o.scale[index],
    }) as f64
}

fn mark_peak(obj: Obj, clip: &str, frame: i32) -> Obj {
    obj.edit(|o| {
        if let Some(t) = o.tracks.iter_mut().find(|t| t.name == clip) {
            t.peak = Some(frame);
        }
    });
    obj
}

/// A scale: one number for every axis, or (x, y, z) Blender.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Factors(pub [f64; 3]);

impl From<f64> for Factors {
    fn from(k: f64) -> Factors {
        Factors([k; 3])
    }
}
impl From<i32> for Factors {
    fn from(k: i32) -> Factors {
        Factors([k as f64; 3])
    }
}
impl<A: crate::math::Num, B: crate::math::Num, C: crate::math::Num> From<(A, B, C)> for Factors {
    fn from(t: (A, B, C)) -> Factors {
        Factors([t.0.f(), t.1.f(), t.2.f()])
    }
}

options! {
    /// `squash`'s options.
    Squash { seconds: f64 = 0.6, back: bool = false, split: f64 = 0.5 }
}

/// A clip `clip` scaling `obj` about its own origin from rest to `scale` times its size: a number for every axis,
/// or (x, y, z) Blender. A cushion sinking (0.7 along Z), a bag flattening, a held instrument or weight hidden
/// while in the hands (0.001). The game plays a use's clip back on its own; `back` makes the clip come back to
/// rest by itself (a bounce, for a loop), at `split` of the way through (0.3: down quickly, up slowly, as a
/// cushion recovers).
pub fn squash(obj: Obj, clip: &str, scale: impl Into<Factors>, o: Squash) -> Obj {
    let factors = scale.into().0;
    let end = last_frame(o.seconds);
    let mid = (end - 1).min(2.max(1 + ((end - 1) as f64 * o.split).floor() as i32));
    let mut keyed = false;
    for (i, &k) in factors.iter().enumerate() {
        if k == 1.0 {
            continue;
        }
        let r = rest(obj, Path::Scale, i);
        let keys = if o.back {
            vec![(1, r), (mid, r * k), (end, r)]
        } else {
            vec![(1, r), (end, r * k)]
        };
        key_channel(obj, clip, Path::Scale, i, &keys, Ease::Smooth, OVERSHOOT);
        keyed = true;
    }
    assert!(keyed, "squash {clip:?}: a scale of 1 does nothing");
    mark_peak(obj, clip, if o.back { mid } else { end })
}

options! {
    /// `stages`' options: `sink` is (picked, growing, planted), metres the part goes down into the soil at each.
    Stages { seconds: f64 = 1.5, sink: Option<(f64, f64, f64)> = None }
}

/// A crop's growth clip `clip` (`motions.growth`) for `obj`: the scale of it at each growth stage, times its size
/// at rest, which is ripe. A number for every axis or (x, y, z) Blender; 0.001 hides it. The game holds the clip
/// where the stage is: ripe at rest, picked a third of the way, growing two thirds, planted at the end, and plays
/// it on from stage to stage. `sink`: so plants spread over a bed come up where they stand rather than shrink
/// towards its middle: scale them only in height and sink them, and what shows is each plant's top, in its place.
pub fn stages(
    obj: Obj,
    clip: &str,
    picked: impl Into<Factors>,
    growing: impl Into<Factors>,
    planted: impl Into<Factors>,
    o: Stages,
) -> Obj {
    let end = last_frame(o.seconds);
    let frames = [
        1.0,
        1.0 + (end - 1) as f64 / 3.0,
        1.0 + 2.0 * (end - 1) as f64 / 3.0,
        end as f64,
    ]
    .map(|f| crate::py::round(f) as i32);
    let each = [
        [1.0; 3],
        picked.into().0,
        growing.into().0,
        planted.into().0,
    ];
    let mut keyed = false;
    for i in 0..3 {
        if each.iter().any(|k| k[i] != 1.0) {
            let r = rest(obj, Path::Scale, i);
            let keys: Vec<(i32, f64)> = frames
                .iter()
                .zip(&each)
                .map(|(&f, k)| (f, r * k[i]))
                .collect();
            key_channel(obj, clip, Path::Scale, i, &keys, Ease::Smooth, OVERSHOOT);
            keyed = true;
        }
    }
    if let Some(sink) = o.sink.filter(|s| s.0 != 0.0 || s.1 != 0.0 || s.2 != 0.0) {
        let r = rest(obj, Path::Location, 2);
        let keys: Vec<(i32, f64)> = frames
            .iter()
            .zip([0.0, sink.0, sink.1, sink.2])
            .map(|(&f, d)| (f, r - d))
            .collect();
        key_channel(obj, clip, Path::Location, 2, &keys, Ease::Smooth, OVERSHOOT);
        keyed = true;
    }
    assert!(keyed, "stages {clip:?}: the same at every stage");
    obj
}

/// [(part, clip name, last frame)] for every clip on `objs`.
pub fn clips(objs: &[Obj]) -> Vec<(Obj, String, i32)> {
    let mut out = Vec::new();
    for &o in objs {
        o.read(|p| out.extend(p.tracks.iter().map(|t| (o, t.name.clone(), t.end))));
    }
    out
}

/// Whether a clip takes `obj` away from its rest: moved over 3 cm, turned over 8 degrees, or scaled to under half
/// or over twice (grown from nothing, hidden while held). A cushion squashed a little stays.
pub fn moves_away(obj: Obj) -> bool {
    let tracks: Vec<(String, i32)> =
        obj.read(|o| o.tracks.iter().map(|t| (t.name.clone(), t.end)).collect());
    let (loc0, rot0, scale0) = pose(obj, None, 1.0);
    let rest_turn = crate::math::object_matrix(V3::ZERO, rot0, crate::math::v3(1, 1, 1));
    for (clip, last) in tracks {
        let step = (last / 12).max(1);
        let mut f = 1;
        while f <= last {
            let (loc, rot, scale) = pose(obj, Some(&clip), f as f64);
            if (loc - loc0).length() > 0.03 {
                return true;
            }
            if turn_between(
                &rest_turn,
                &crate::math::object_matrix(V3::ZERO, rot, crate::math::v3(1, 1, 1)),
            ) > crate::math::radians(8.0)
            {
                return true;
            }
            let ratios: Vec<f64> = (0..3)
                .map(|i| scale[i] as f64 / (scale0[i] as f64).max(1e-9))
                .collect();
            let lowest = ratios.iter().copied().fold(f64::INFINITY, f64::min);
            let highest = ratios.iter().copied().fold(f64::NEG_INFINITY, f64::max);
            if lowest < 0.5
                || highest > 2.0
                || (0..3).map(|i| scale0[i]).fold(f32::INFINITY, f32::min) < 0.01
            {
                return true;
            }
            f += step;
        }
    }
    false
}

/// The angle of the turn from `a` to `b` (two turns, as matrices), radians.
fn turn_between(a: &M4, b: &M4) -> f64 {
    let (a, b) = (a.to_3x3().0, b.to_3x3().0);
    let mut trace = 0.0_f64;
    for (ca, cb) in a.iter().zip(&b) {
        for (x, y) in ca.iter().zip(cb) {
            trace += *x as f64 * *y as f64;
        }
    }
    crate::math::acos(((trace - 1.0) / 2.0).clamp(-1.0, 1.0))
}

/// Whether any of the scene's parts has a clip.
pub fn any_clips() -> bool {
    scene::with(|s| s.objects.iter().any(|o| o.alive_with_tracks()))
}
