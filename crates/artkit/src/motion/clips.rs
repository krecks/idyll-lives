//! The clips of a scene: which objects have them, and which move away.

use super::*;

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
