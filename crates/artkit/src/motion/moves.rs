//! The moves a part can be given: swing, slide, squash, stages, grow, spin, scroll, rock, turns, path and wobble.
//!
//! Split from `motion.rs` (see `mod.rs`).

use super::*;

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
    /// `swing`'s options.
    Swing { seconds: f64 = 1.0, axis: crate::math::Axis = crate::math::Axis::Z, ease: Ease = Ease::Smooth, overshoot: f64 = OVERSHOOT }
}

/// A clip `clip` turning `obj` about its own `axis` (Blender) by `degrees` from rest. A door hinged on its right
/// edge (seen from the front) opens with a negative angle about Z. `ease`: `Ease::BackOut` swings `overshoot`
/// past open and settles back, as a door let go does; only where there's room past open (not a lid against a
/// wall, a bed onto the floor).
pub fn swing(obj: Obj, clip: &str, degrees: f64, o: Swing) -> Obj {
    let i = o.axis.index();
    obj.set_rotation_mode(RotationMode::XYZ);
    let r = rest(obj, Path::Rotation, i);
    let keys = [
        (1, r),
        (last_frame(o.seconds), r + crate::math::radians(degrees)),
    ];
    key_channel(obj, clip, Path::Rotation, i, &keys, o.ease, o.overshoot)
}

/// `motion::swing` with its options by name, as the family scripts wrote them: `swing!(obj, clip, degrees, key = value, ...)`.
#[macro_export]
macro_rules! swing {
    ($obj:expr, $clip:expr, $degrees:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        // (as Python: what is given by place first, then the options in the order written)
        let args = ($obj, $clip, $degrees);
        #[allow(unused_mut)]
        let mut o = $crate::motion::Swing::default();
        $(o.$k = $crate::Arg::arg($v);)*
        $crate::motion::swing(args.0, args.1, $crate::forms::num(args.2), o)
    }};
}

options! {
    /// `slide`'s options.
    Slide { seconds: f64 = 1.0, ease: Ease = Ease::Smooth }
}

/// A clip `clip` moving `obj` by `offset` (Blender x, y, z, metres) from rest: a drawer pulls out to the front
/// with a negative y. `ease`: how it eases (`Ease`).
pub fn slide(obj: Obj, clip: &str, offset: impl Into<crate::math::P3>, o: Slide) -> Obj {
    let offset = offset.into().array();
    for (i, &d) in offset.iter().enumerate() {
        if d != 0.0 {
            let r = rest(obj, Path::Location, i);
            let keys = [(1, r), (last_frame(o.seconds), r + d)];
            key_channel(obj, clip, Path::Location, i, &keys, o.ease, OVERSHOOT);
        }
    }
    obj
}

/// `motion::slide` with its options by name, as the family scripts wrote them: `slide!(obj, clip, offset, key = value, ...)`.
#[macro_export]
macro_rules! slide {
    ($obj:expr, $clip:expr, $offset:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        // (as Python: what is given by place first, then the options in the order written)
        let args = ($obj, $clip, $offset);
        #[allow(unused_mut)]
        let mut o = $crate::motion::Slide::default();
        $(o.$k = $crate::Arg::arg($v);)*
        $crate::motion::slide(args.0, args.1, args.2, o)
    }};
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

/// `motion::squash` with its options by name, as the family scripts wrote them: `squash!(obj, clip, scale, key = value, ...)`.
#[macro_export]
macro_rules! squash {
    ($obj:expr, $clip:expr, $scale:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        // (as Python: what is given by place first, then the options in the order written)
        let args = ($obj, $clip, $scale);
        #[allow(unused_mut)]
        let mut o = $crate::motion::Squash::default();
        $(o.$k = $crate::Arg::arg($v);)*
        $crate::motion::squash(args.0, args.1, args.2, o)
    }};
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

/// `motion::stages` with its options by name, as the family scripts wrote them: `stages!(obj, clip, picked = .., growing = .., planted = .., key = value, ...)`.
#[macro_export]
macro_rules! stages {
    ($obj:expr, $clip:expr, picked = $picked:expr, growing = $growing:expr, planted = $planted:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        // (as Python: what is given by place first, then the options in the order written)
        let args = ($obj, $clip, $picked, $growing, $planted);
        #[allow(unused_mut)]
        let mut o = $crate::motion::Stages::default();
        $(o.$k = $crate::Arg::arg($v);)*
        $crate::motion::stages(args.0, args.1, args.2, args.3, args.4, o)
    }};
}

/// A part hidden at rest: a thousandth of its size, out of sight.
pub const FLAT: f64 = 0.001;

/// Hides `obj` at rest by flattening it along `axis` (Blender) to `FLAT`: a screen rolled up in its case on the wall, a
/// fire's flames at the foot of the log bed. `grow` runs it out to full size when the object is used, and `squash`
/// from here too.
pub fn flat(obj: Obj, axis: Axis) -> Obj {
    obj.edit(|o| o.scale[axis.index()] = FLAT as f32);
    obj
}

options! {
    /// `grow`'s and `grow_in_turn`'s options.
    Grow { seconds: f64 = 0.4, axis: Axis = Axis::Z }
}

/// A clip `clip` scaling `obj` about its own origin along `axis` (Blender) from where it is (built small and hidden) to
/// full size (scale 1): a part that appears while a resident uses the object (a stream of water).
pub fn grow(obj: Obj, clip: &str, o: Grow) -> Obj {
    let i = o.axis.index();
    let r = rest(obj, Path::Scale, i);
    key_channel(
        obj,
        clip,
        Path::Scale,
        i,
        &[(1, r), (last_frame(o.seconds), 1.0)],
        Ease::Smooth,
        OVERSHOOT,
    )
}

/// `motion::grow` with its options by name, as the family scripts wrote them: `grow!(obj, clip, key = value, ...)`.
#[macro_export]
macro_rules! grow {
    ($obj:expr, $clip:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        // (as Python: what is given by place first, then the options in the order written)
        let args = ($obj, $clip);
        #[allow(unused_mut)]
        let mut o = $crate::motion::Grow::default();
        $(o.$k = $crate::Arg::arg($v);)*
        $crate::motion::grow(args.0, args.1, o)
    }};
}

/// The frames that share `end` out among `n` parts, each its turn: from frame 1 to `end`.
fn bounds(end: i32, n: usize) -> Vec<i32> {
    (0..=n)
        .map(|k| 1 + py::round((end - 1) as f64 * k as f64 / n as f64) as i32)
        .collect()
}

/// Keys with one key a frame: a key on the frame of the one before it is left out.
fn one_a_frame(keys: &[(i32, f64)]) -> Vec<(i32, f64)> {
    keys.iter()
        .enumerate()
        .filter(|&(n, kv)| n == 0 || kv.0 != keys[n - 1].0)
        .map(|(_, &kv)| kv)
        .collect()
}

/// A clip `clip` in which `objs` grow one after another, the first first, each in its share of `seconds` (1.5 by
/// default), along `axis` from where it is (built small and hidden) to full size: the washing pegged out line by line.
/// Played along with a use (`over`), they come one at a time through it; played back, the last goes first.
pub fn grow_in_turn(objs: &[Obj], clip: &str, seconds: Option<f64>, axis: Axis) -> Vec<Obj> {
    let i = axis.index();
    let end = last_frame(seconds.unwrap_or(1.5));
    let b = bounds(end, objs.len());
    for (k, &obj) in objs.iter().enumerate() {
        let r = rest(obj, Path::Scale, i);
        let keys = one_a_frame(&[(1, r), (b[k], r), (b[k + 1], 1.0), (end, 1.0)]);
        key_channel(obj, clip, Path::Scale, i, &keys, Ease::Smooth, OVERSHOOT);
    }
    if let Some(&last) = objs.last() {
        mark_peak(last, clip, end);
    }
    objs.to_vec()
}

options! {
    /// `spin`'s options.
    Spin { turns: f64 = 1.0, seconds: f64 = 1.0, axis: Axis = Axis::Z }
}

/// A clip `clip` turning `obj` about its own `axis` (Blender) by whole `turns` (negative: the other way) at an even
/// speed, ending where it started: a wheel, a record platter, a fan. Keep it under about one turn a second (the
/// game blends between its samples); a part that looks the same turned (a fan's blades) can turn by less, e.g. a
/// quarter turn with four blades, and loop.
pub fn spin(obj: Obj, clip: &str, o: Spin) -> Obj {
    let i = o.axis.index();
    obj.set_rotation_mode(RotationMode::XYZ);
    let r = rest(obj, Path::Rotation, i);
    let end = last_frame(o.seconds);
    key_channel(
        obj,
        clip,
        Path::Rotation,
        i,
        &[(1, r), (end, r + o.turns * 2.0 * PI)],
        Ease::Linear,
        OVERSHOOT,
    );
    mark_peak(obj, clip, py::round((1 + end) as f64 / 2.0) as i32)
}

/// `motion::spin` with its options by name, as the family scripts wrote them: `spin!(obj, clip, key = value, ...)`.
#[macro_export]
macro_rules! spin {
    ($obj:expr, $clip:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        // (as Python: what is given by place first, then the options in the order written)
        let args = ($obj, $clip);
        #[allow(unused_mut)]
        let mut o = $crate::motion::Spin::default();
        $(o.$k = $crate::Arg::arg($v);)*
        $crate::motion::spin(args.0, args.1, o)
    }};
}

/// A clip `clip` moving `obj` by `offset` (Blender x, y, z, metres) at an even speed, as a loop, in `seconds`
/// (1 by default): a treadmill's belt slats, a conveyor. The game jumps it home at the end of each round, unseen when
/// `offset` is the pattern's repeat (one slat on) and the part's ends are hidden under covers.
pub fn scroll(obj: Obj, clip: &str, offset: impl Into<P3>, seconds: Option<f64>) -> Obj {
    let offset: P3 = offset.into();
    let end = last_frame(seconds.unwrap_or(1.0));
    for (i, d) in offset.array().into_iter().enumerate() {
        if d != 0.0 {
            let r = rest(obj, Path::Location, i);
            key_channel(
                obj,
                clip,
                Path::Location,
                i,
                &[(1, r), (end, r + d)],
                Ease::Linear,
                OVERSHOOT,
            );
        }
    }
    mark_peak(obj, clip, py::round((1 + end) as f64 / 2.0) as i32)
}

options! {
    /// `rock`'s options.
    Rock { seconds: f64 = 1.0, axis: Axis = Axis::X }
}

/// A clip `clip` rocking `obj` about its own `axis` (Blender): out by `degrees`, back past rest to the other side
/// and home, as a pendulum (a rocking chair, a swing, the seesaw). It ends where it started at the speed it started
/// with, so it loops smoothly.
pub fn rock(obj: Obj, clip: &str, degrees: f64, o: Rock) -> Obj {
    let i = o.axis.index();
    obj.set_rotation_mode(RotationMode::XYZ);
    let r = rest(obj, Path::Rotation, i);
    let end = last_frame(o.seconds);
    let a = math::radians(degrees);
    let keys: Vec<(i32, f64)> = (1..=end)
        .map(|f| {
            (
                f,
                r + a * math::sin(2.0 * PI * (f - 1) as f64 / (end - 1) as f64),
            )
        })
        .collect();
    key_channel(obj, clip, Path::Rotation, i, &keys, Ease::Linear, OVERSHOOT);
    mark_peak(obj, clip, py::round(1.0 + (end - 1) as f64 / 4.0) as i32)
}

/// `motion::rock` with its options by name, as the family scripts wrote them: `rock!(obj, clip, degrees, key = value, ...)`.
#[macro_export]
macro_rules! rock {
    ($obj:expr, $clip:expr, $degrees:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        // (as Python: what is given by place first, then the options in the order written)
        let args = ($obj, $clip, $degrees);
        #[allow(unused_mut)]
        let mut o = $crate::motion::Rock::default();
        $(o.$k = $crate::Arg::arg($v);)*
        $crate::motion::rock(args.0, args.1, $crate::forms::num(args.2), o)
    }};
}

/// A clip `clip` in which `objs` take turns: each in its share of `seconds` (1 by default) rises by `offset`
/// (Blender x, y, z, metres) and settles back, the first one first; it ends where it starts, so it loops
/// (`motions.loops`, `idle`). A dance floor's two checkerboards of tiles lifting one after the other.
pub fn turns(objs: &[Obj], clip: &str, offset: impl Into<P3>, seconds: Option<f64>) -> Vec<Obj> {
    let offset: P3 = offset.into();
    let end = last_frame(seconds.unwrap_or(1.0));
    let b = bounds(end, objs.len());
    for (k, &obj) in objs.iter().enumerate() {
        let (lo, hi) = (b[k], b[k + 1]);
        for (i, d) in offset.array().into_iter().enumerate() {
            if d != 0.0 {
                let r = rest(obj, Path::Location, i);
                let keys = one_a_frame(&[
                    (1, r),
                    (lo, r),
                    (py::floordiv((lo + hi) as i64, 2) as i32, r + d),
                    (hi, r),
                    (end, r),
                ]);
                key_channel(obj, clip, Path::Location, i, &keys, Ease::Smooth, OVERSHOOT);
            }
        }
    }
    if let Some(&first) = objs.first() {
        mark_peak(first, clip, py::floordiv((b[0] + b[1]) as i64, 2) as i32);
    }
    objs.to_vec()
}

/// A key of a `path`: (seconds, offset from rest (Blender metres), turn from rest about the part's own X, Y and Z
/// (Blender degrees)).
pub type PathKey = (f64, (f64, f64, f64), (f64, f64, f64));

/// A clip `clip` taking `obj` along a path of `keys`: at each time, its offset from rest and its turn from rest; the
/// first key, at 0 s, is rest. For what moves along more than a line: the pool rack lifted off the balls, carried
/// over the rail and hung on the table's end; the robot vacuum's round (a loop: it ends at rest, its heading a whole
/// turn on, either way; `linear` keeps an even speed between the keys, else each eases).
pub fn path(obj: Obj, clip: &str, keys: &[PathKey], linear: bool) -> Obj {
    let zero = |p: (f64, f64, f64)| p.0 == 0.0 && p.1 == 0.0 && p.2 == 0.0;
    assert!(
        keys[0].0 == 0.0 && zero(keys[0].1) && zero(keys[0].2),
        "path {clip:?}: the first key is rest, at 0 s"
    );
    let frames: Vec<i32> = keys.iter().map(|k| frame(k.0)).collect();
    assert!(
        frames.windows(2).all(|w| w[1] > w[0]),
        "path {clip:?}: keys a frame or more apart, in order"
    );
    obj.set_rotation_mode(RotationMode::XYZ);
    let ease = if linear { Ease::Linear } else { Ease::Smooth };
    let at = |p: (f64, f64, f64), i: usize| [p.0, p.1, p.2][i];
    let mut moved = false;
    for i in 0..3 {
        for (path, rot, scale) in [
            (Path::Location, false, 1.0),
            (Path::Rotation, true, math::radians(1.0)),
        ] {
            let of = |k: &PathKey| at(if rot { k.2 } else { k.1 }, i);
            if keys.iter().any(|k| of(k) != 0.0) {
                let r = rest(obj, path, i);
                let channel: Vec<(i32, f64)> = frames
                    .iter()
                    .zip(keys)
                    .map(|(&f, k)| (f, r + of(k) * scale))
                    .collect();
                key_channel(obj, clip, path, i, &channel, ease, OVERSHOOT);
                moved = true;
            }
        }
    }
    assert!(moved, "path {clip:?}: every key at rest");
    mark_peak(obj, clip, frames[frames.len() / 2])
}

options! {
    /// `wobble`'s options.
    Wobble { seconds: f64 = 1.0, axis: Axis = Axis::X, swings: f64 = 3.0, delay: f64 = 0.0 }
}

/// A clip `clip` that knocks `obj` swinging about its own `axis` (Blender) and lets it die away: out by `degrees`,
/// back past rest, `swings` times, each smaller, ending at rest, `delay` seconds in. The give in what hangs or stands
/// loose (a towel on its rail, a lamp's shade, a pan on its hook) as something beside it moves: `attach` it to a part
/// that moves and give both clips the same name.
pub fn wobble(obj: Obj, clip: &str, degrees: f64, o: Wobble) -> Obj {
    let i = o.axis.index();
    obj.set_rotation_mode(RotationMode::XYZ);
    let r = rest(obj, Path::Rotation, i);
    let (start, end) = (frame(o.delay), last_frame(o.delay + o.seconds));
    let a = math::radians(degrees);
    let mut keys = vec![(1, r)];
    if start > 1 {
        keys.push((start, r));
    }
    let span = end - start;
    for f in start + 1..end {
        let t = (f - start) as f64 / span as f64;
        keys.push((
            f,
            r + a * math::exp(-3.0 * t) * math::sin(PI * o.swings * t),
        ));
    }
    keys.push((end, r));
    key_channel(obj, clip, Path::Rotation, i, &keys, Ease::Linear, OVERSHOOT);
    let swing = py::round(span as f64 / (2.0 * o.swings)).max(1) as i32;
    mark_peak(obj, clip, start + swing)
}

/// `motion::wobble` with its options by name, as the family scripts wrote them: `wobble!(obj, clip, degrees, key = value, ...)`.
#[macro_export]
macro_rules! wobble {
    ($obj:expr, $clip:expr, $degrees:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        // (as Python: what is given by place first, then the options in the order written)
        let args = ($obj, $clip, $degrees);
        #[allow(unused_mut)]
        let mut o = $crate::motion::Wobble::default();
        $(o.$k = $crate::Arg::arg($v);)*
        $crate::motion::wobble(args.0, args.1, $crate::forms::num(args.2), o)
    }};
}
