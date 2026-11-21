//! What a model is written for, and how its animation is interpolated.

use super::*;

impl Target {
    pub(super) fn name(self) -> &'static str {
        match self {
            Target::Translation => "translation",
            Target::Rotation => "rotation",
            Target::Scale => "scale",
        }
    }

    /// Numbers a key.
    pub fn width(self) -> usize {
        if self == Target::Rotation { 4 } else { 3 }
    }
}

/// How a channel goes from one key to the next.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Interpolation {
    Linear,
    Step,
    /// Each key an in-tangent, its value and an out-tangent.
    CubicSpline,
}

impl Interpolation {
    pub(super) fn name(self) -> &'static str {
        match self {
            Interpolation::Linear => "LINEAR",
            Interpolation::Step => "STEP",
            Interpolation::CubicSpline => "CUBICSPLINE",
        }
    }
}

/// One part's place, turn or scale over a clip: its keys' times (s) and values, `target.width()` a key.
#[derive(Clone, Debug, PartialEq)]
pub struct Channel {
    pub node: usize,
    pub target: Target,
    pub interpolation: Interpolation,
    pub times: Vec<f32>,
    pub values: Vec<f32>,
}

/// A clip.
#[derive(Clone, Debug, PartialEq)]
pub struct Animation {
    pub name: String,
    pub channels: Vec<Channel>,
}

impl Animation {
    /// Its length: its last key's time (s).
    pub fn length(&self) -> f32 {
        self.channels
            .iter()
            .filter_map(|c| c.times.last().copied())
            .fold(0.0, f32::max)
    }
}
