//! Keys given as points, and the traits that let them be.

use super::*;

/// A point of `keys`: (seconds, value), eased as Blender eases by default, or (seconds, value, the ease from that
/// key to the next).
pub trait KeyPoint: Copy {
    fn key(self) -> (f64, f64, Ease);
}
impl<A: crate::math::Num, B: crate::math::Num> KeyPoint for (A, B) {
    fn key(self) -> (f64, f64, Ease) {
        (self.0.f(), self.1.f(), Ease::Smooth)
    }
}
impl<A: crate::math::Num, B: crate::math::Num> KeyPoint for (A, B, Ease) {
    fn key(self) -> (f64, f64, Ease) {
        (self.0.f(), self.1.f(), self.2)
    }
}

/// A clip `clip` keying `obj`'s `path`[`index`] (Blender axes) at `points`, (seconds, value, the ease from that key
/// to the next) each; the first at 0 s is rest. Values as Blender has them (metres, radians, scale). For a clip none
/// of the others make: water filling a bath from 0.5 s, a koi surfacing and diving.
pub fn keys(obj: Obj, clip: &str, path: Path, index: usize, points: &[impl KeyPoint]) -> Obj {
    let points: Vec<(f64, f64, Ease)> = points.iter().map(|p| p.key()).collect();
    assert!(
        !points.is_empty() && points[0].0 == 0.0,
        "keys {clip:?}: the first key is at 0 s (rest)"
    );
    let frames: Vec<(i32, f64, Ease)> = points.iter().map(|p| (frame(p.0), p.1, p.2)).collect();
    assert!(
        frames.windows(2).all(|w| w[1].0 > w[0].0),
        "keys {clip:?}: keys a frame or more apart, in order"
    );
    key_eased(obj, clip, path, index, &frames, OVERSHOOT)
}

/// Puts moving part `obj` inside moving part `parent`, where it is: it goes with `parent`'s clips and plays its own on
/// top (a fridge's lever turning on its door, a record spinning as it swings up). Call it after `parts::settle` (which
/// moves every part on its own) and before the clips; both stay their own parts, so `obj` still costs its own draw.
/// Its own clips turn it about its own origin, in `parent`'s frame.
pub fn attach(obj: Obj, parent: Obj) -> Obj {
    let world = obj.matrix_world();
    obj.edit(|o| o.parent = Some(parent));
    // (its place, turn and scale taken apart from where it is in the parent's frame)
    obj.set_matrix_world(world);
    obj
}
