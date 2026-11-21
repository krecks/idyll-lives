//! Keys, channels and tracks: what a clip is made of.

use super::*;

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
    /// The channel's value at `frame`, as Blender evaluates the F-curve (single precision throughout).
    pub fn at(&self, frame: f64) -> f64 {
        eval(&self.beziers(), frame as f32) as f64
    }

    /// Each key's (left handle, key, right handle), (frame, value) each, as Blender's F-curve holds them.
    pub fn handles(&self) -> Vec<[[f32; 2]; 3]> {
        self.beziers().iter().map(|b| [b.h1, b.co, b.h2]).collect()
    }

    /// The keys as Blender holds them: each with the handles its F-curve works out for it (auto-clamped, smoothed
    /// for continuous acceleration, Blender's defaults) and its interpolation to the next.
    fn beziers(&self) -> Vec<Bez> {
        let back = back_amount(self.back) as f32;
        let mut b: Vec<Bez> = self
            .keys
            .iter()
            .map(|k| Bez {
                h1: [0.0; 2],
                co: [k.frame as f32, k.value],
                h2: [0.0; 2],
                ease: k.ease,
                back,
                locked: false,
            })
            .collect();
        let n = b.len();
        if n < 2 {
            return b;
        }
        for i in 0..n {
            let prev = (i > 0).then(|| b[i - 1].co);
            let next = (i + 1 < n).then(|| b[i + 1].co);
            calc_handles(&mut b[i], prev, next);
            // the first and last keys level, as the curve holds still beyond them
            if i == 0 || i == n - 1 {
                b[i].h1[1] = b[i].co[1];
                b[i].h2[1] = b[i].co[1];
                b[i].locked = true;
            }
        }
        smooth_handles(&mut b);
        b
    }
}
