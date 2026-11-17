//! Placing parts in the game's axes. The content and the old primitive parts wrote (x, height, depth) with the
//! front at +depth; Blender is (x, y, z) with the front at -y and the game's x mirrored. A `GameFrame` takes the
//! game's numbers as they are:
//!
//!   let f = GameFrame::new(0.094);
//!   soft!(f, "cushion", 0.6, 0.12, 0.55, (0, 0.45, 0), M.fabric, puff = 0.02);   // w, h, d at (x, y, z)
//!
//! `shift` moves every depth forward (+z, to the front) by that much, a negative one back: a thing built against
//! the back of its tile (BACK = -0.455), its parts' depths centred on -shift, is drawn centred on the tile this way.
//! `depths` redraws a thing's numbers at another depth: pairs (game depth, drawn depth), ascending; depths between
//! two pairs are mapped linearly, beyond the ends moved as the end is. A box's back and front are mapped, so it
//! keeps its place between them; anything else moves by its middle (the 0.79-deep kitchen counters drawn 0.66
//! deep, as the old `shallow`).

use crate::materials::Mat;
use crate::math::P3;
use crate::scene::Obj;
use crate::soft::{self, Puff, Sag, SoftBox};

/// A game coordinate as a script wrote it, whole or not: the negation of a whole zero is a plain zero, of a float
/// zero a negative one, and a part's place keeps that sign.
pub trait Coord: Copy {
    fn f(self) -> f64;
    /// `-x`, as Python takes it.
    fn negated(self) -> f64;
}
impl Coord for f64 {
    fn f(self) -> f64 {
        self
    }
    fn negated(self) -> f64 {
        -self
    }
}
impl Coord for i32 {
    fn f(self) -> f64 {
        self as f64
    }
    fn negated(self) -> f64 {
        -self as f64
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct GameFrame {
    pub shift: f64,
    pub depths: Option<Vec<(f64, f64)>>,
}

impl GameFrame {
    pub fn new(shift: f64) -> GameFrame {
        GameFrame {
            shift,
            depths: None,
        }
    }

    /// A frame that redraws depths through `depths`.
    pub fn with_depths(shift: f64, depths: &[(f64, f64)]) -> GameFrame {
        GameFrame {
            shift,
            depths: Some(depths.to_vec()),
        }
    }

    /// A game depth as drawn (through `depths`, if given).
    pub fn depth(&self, z: f64) -> f64 {
        let d = match &self.depths {
            Some(d) if !d.is_empty() => d,
            _ => return z,
        };
        if z <= d[0].0 {
            return z + d[0].1 - d[0].0;
        }
        for w in d.windows(2) {
            let ((a, b), (c, e)) = (w[0], w[1]);
            if z <= c {
                return b + (z - a) * (e - b) / (c - a);
            }
        }
        let last = d[d.len() - 1];
        z + last.1 - last.0
    }

    /// A game position (x, height, depth) as a Blender position (front -Y, up +Z).
    pub fn at(&self, x: impl Coord, y: impl Coord, z: impl Coord) -> P3 {
        // (a whole depth as it is, without `depths`: its negation is a whole number too)
        let depth = if self.depths.as_ref().is_some_and(|d| !d.is_empty()) {
            -self.depth(z.f())
        } else {
            z.negated()
        };
        P3 {
            x: x.negated(),
            y: depth - self.shift,
            z: y.f(),
        }
    }

    /// A soft box (`soft::soft_box`) of game width, height and depth at a game position; a `puff` of three is game
    /// (x, height, depth) too, and so is a `sag` of three (a number sags its top towards the front). `o.at` is
    /// not read: `at` places it.
    #[allow(clippy::too_many_arguments)]
    pub fn soft(
        &self,
        name: impl AsRef<str>,
        w: f64,
        h: f64,
        d: f64,
        at: (impl Coord, impl Coord, impl Coord),
        material: impl Into<Option<Mat>>,
        mut o: SoftBox,
    ) -> Obj {
        if let Puff::Each(p) = o.puff {
            o.puff = Puff::Each([p[0], p[2], p[1]]);
        }
        if let Sag::Towards(s) = o.sag {
            o.sag = Sag::Towards([-s[0], -s[2], s[1]]);
        }
        o.at = self.at(at.0, at.1, at.2);
        o.material = material.into();
        soft::soft_box(name, (w, d, h), o)
    }
}

/// `GameFrame::soft` with its options by name, as the family scripts wrote them:
/// `soft!(frame, name, w, h, d, at, material, key = value, ...)`.
#[macro_export]
macro_rules! soft {
    ($frame:expr, $name:expr, $w:expr, $h:expr, $d:expr, $at:expr, $material:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        #[allow(unused_mut)]
        let mut o = $crate::soft::SoftBox::default();
        $(o.$k = $crate::Arg::arg($v);)*
        $frame.soft($name, $w, $h, $d, $at, $material, o)
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn depths_map_between_pairs_and_move_beyond_them() {
        let f = GameFrame::with_depths(0.1, &[(-0.4, -0.3), (0.4, 0.2)]);
        assert_eq!(f.depth(-0.5), -0.5 + -0.3 - -0.4);
        assert_eq!(
            f.depth(0.0),
            -0.3 + (0.0 - -0.4) * (0.2 - -0.3) / (0.4 - -0.4)
        );
        assert_eq!(f.depth(0.6), 0.6 + 0.2 - 0.4);
        let p = f.at(0.2, 1.0, 0.0);
        assert_eq!((p.x, p.y, p.z), (-0.2, -f.depth(0.0) - 0.1, 1.0));
        assert_eq!(GameFrame::new(0.0).depth(0.3), 0.3);
        // a whole zero stays a plain zero, a float one turns negative
        let f = GameFrame::new(0.0);
        let (whole, float) = (f.at(0, 0, 0), f.at(0.0, 0, 0.0));
        assert!(whole.x.is_sign_positive() && whole.y.is_sign_positive());
        assert!(float.x.is_sign_negative() && float.y.is_sign_negative());
    }
}
