//! A pose at a frame, and the channel behind a key.

use super::*;

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
    let keys: Vec<(i32, f64, Ease)> = keys.iter().map(|&(f, v)| (f, v, ease)).collect();
    key_eased(obj, clip, path, index, &keys, overshoot)
}

/// Keys `obj`'s `path`[`index`] at the frames of `keys`, (frame, value, the ease from it to the next key) each, in
/// its clip `clip` (gamekit's `_clip`). `overshoot`: how far `BackOut` goes past the key it eases into, a share of
/// the move.
pub fn key_eased(
    obj: Obj,
    clip: &str,
    path: Path,
    index: usize,
    keys: &[(i32, f64, Ease)],
    overshoot: f64,
) -> Obj {
    let keys: Vec<Key> = keys
        .iter()
        .map(|&(frame, value, ease)| Key {
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

pub(crate) fn rest(obj: Obj, path: Path, index: usize) -> f64 {
    obj.read(|o| match path {
        Path::Location => o.loc[index],
        Path::Rotation => o.rot[index],
        Path::Scale => o.scale[index],
    }) as f64
}

pub(super) fn mark_peak(obj: Obj, clip: &str, frame: i32) -> Obj {
    obj.edit(|o| {
        if let Some(t) = o.tracks.iter_mut().find(|t| t.name == clip) {
            t.peak = Some(frame);
        }
    });
    obj
}
