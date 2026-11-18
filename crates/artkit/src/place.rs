//! Placing parts in the game's axes. The content and the old primitive parts wrote (x, height, depth) with the
//! front at +depth; Blender is (x, y, z) with the front at -y and the game's x mirrored. A `GameFrame` takes the
//! game's numbers as they are:
//!
//!   const F: GameFrame = GameFrame::new(0.094);
//!   fbox!(F, "door", 0.728, 1.146, 0.04, (0, 0.643, 0.195), M.body, bevel = 0.01);   // w, h, d at (x, y, z)
//!   soft!(F, "cushion", 0.6, 0.12, 0.55, (0, 0.45, 0), M.fabric, puff = 0.02);
//!
//! `shift` moves every depth forward (+z, to the front) by that much, a negative one back: a thing built against
//! the back of its tile (BACK = -0.455), its parts' depths centred on -shift, is drawn centred on the tile this way.
//! `depths` redraws a thing's numbers at another depth: pairs (game depth, drawn depth), ascending; depths between
//! two pairs are mapped linearly, beyond the ends moved as the end is. A box's back and front are mapped, so it
//! keeps its place between them; anything else moves by its middle (the 0.79-deep kitchen counters drawn 0.66
//! deep, as the old `shallow`).

use crate::forms::{self, Extrude, Plane, Sweep};
use crate::materials::Mat;
use crate::math::P3;
use crate::scene::Obj;
use crate::shapes::{self, Cube, Round, Sphere};
use crate::soft::{self, Puff, Sag, SoftBox};

/// Where a game outline is drawn (`GameFrame::extrude`): 'front' (x, height), thick front to back; 'side' (depth,
/// height), thick across; 'top' (x, depth), thick upwards.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Facing {
    #[default]
    Front,
    Side,
    Top,
}

impl crate::Arg<Plane> for Facing {
    fn arg(self) -> Plane {
        match self {
            Facing::Front => Plane::XZ,
            Facing::Side => Plane::YZ,
            Facing::Top => Plane::XY,
        }
    }
}

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

/// A game position as a script wrote one: (x, height, depth), each whole or not.
pub trait Spot: Copy {
    fn through(self, frame: &GameFrame) -> P3;
}
impl<A: Coord, B: Coord, C: Coord> Spot for (A, B, C) {
    fn through(self, frame: &GameFrame) -> P3 {
        frame.at(self.0, self.1, self.2)
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct GameFrame {
    pub shift: f64,
    pub depths: Option<Vec<(f64, f64)>>,
}

impl GameFrame {
    pub const fn new(shift: f64) -> GameFrame {
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

    /// A box of game width, height and depth at a game position.
    #[allow(clippy::too_many_arguments)] // (as the scripts called it)
    pub fn box_(
        &self,
        name: impl AsRef<str>,
        w: f64,
        h: f64,
        d: f64,
        at: (impl Coord, impl Coord, impl Coord),
        material: Mat,
        mut o: Cube,
    ) -> Obj {
        o.material = Some(material);
        if self.depths.as_ref().is_some_and(|d| !d.is_empty()) {
            let z = at.2.f();
            let (z0, z1) = (self.depth(z - d / 2.0), self.depth(z + d / 2.0));
            let d = z1 - z0;
            if o.bevel != 0.0 {
                o.bevel = o.bevel.min(d * 0.45);
            }
            o.at = P3 {
                x: at.0.negated(),
                y: -(z0 + z1) / 2.0 - self.shift,
                z: at.1.f(),
            };
            return shapes::cube(name, (w, d, h), o);
        }
        o.at = at.through(self);
        shapes::cube(name, (w, d, h), o)
    }

    /// An upright cylinder of radius and height at a game position.
    pub fn cyl(
        &self,
        name: impl AsRef<str>,
        r: f64,
        h: f64,
        at: impl Spot,
        material: Mat,
        mut o: Round,
    ) -> Obj {
        (o.at, o.material) = (at.through(self), Some(material));
        shapes::cylinder(name, r, h, o)
    }

    /// An upright cone, radius `r1` at the bottom and `r2` at the top, at a game position.
    #[allow(clippy::too_many_arguments)] // (as the scripts called it)
    pub fn cone(
        &self,
        name: impl AsRef<str>,
        r1: f64,
        r2: f64,
        h: f64,
        at: impl Spot,
        material: Mat,
        mut o: Round,
    ) -> Obj {
        (o.at, o.material) = (at.through(self), Some(material));
        shapes::cone(name, r1, r2, h, o)
    }

    /// A sphere at a game position.
    pub fn sphere(
        &self,
        name: impl AsRef<str>,
        r: f64,
        at: impl Spot,
        material: Mat,
        mut o: Sphere,
    ) -> Obj {
        (o.at, o.material) = (at.through(self), Some(material));
        shapes::sphere(name, r, o)
    }

    /// A tube or profile swept through game points (`forms::sweep`).
    pub fn sweep<P: Spot>(
        &self,
        name: impl AsRef<str>,
        points: &[P],
        radius: f64,
        material: Mat,
        mut o: Sweep,
    ) -> Obj {
        o.material = Some(material);
        let points: Vec<P3> = points.iter().map(|&p| p.through(self)).collect();
        forms::sweep(name, &points, radius, o)
    }

    /// A slab from a game outline (`forms::extrude`), drawn as its `plane` says (`Facing`).
    pub fn extrude(
        &self,
        name: impl AsRef<str>,
        outline: &[(f64, f64)],
        depth: f64,
        at: impl Spot,
        material: Mat,
        mut o: Extrude,
    ) -> Obj {
        let flip = |p: &(f64, f64)| match o.plane {
            Plane::XY => (-p.0, -p.1),
            Plane::XZ | Plane::YZ => (-p.0, p.1),
        };
        let outline: Vec<(f64, f64)> = outline.iter().map(flip).collect();
        o.holes = forms::Holes(
            o.holes
                .0
                .iter()
                .map(|h| h.iter().map(flip).collect())
                .collect(),
        );
        (o.at, o.material) = (at.through(self), Some(material));
        forms::extrude(name, &outline, depth, o)
    }
}

/// `GameFrame::soft` with its options by name, as the family scripts wrote them:
/// `soft!(frame, name, w, h, d, at, material, key = value, ...)`.
#[macro_export]
macro_rules! soft {
    ($frame:expr, $name:expr, $w:expr, $h:expr, $d:expr, $at:expr, $material:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        // (as Python: what is given by place first, then the options in the order written)
        let args = (&$frame, $name, $w, $h, $d, $at, $material);
        #[allow(unused_mut)]
        let mut o = $crate::soft::SoftBox::default();
        $(o.$k = $crate::Arg::arg($v);)*
        args.0.soft(args.1, args.2, args.3, args.4, args.5, args.6, o)
    }};
}

/// `GameFrame::box_` with its options by name: `fbox!(F, name, w, h, d, at, material, key = value, ...)`.
#[macro_export]
macro_rules! fbox {
    ($frame:expr, $name:expr, $w:expr, $h:expr, $d:expr, $at:expr, $material:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        // (as Python: what is given by place first, then the options in the order written)
        let args = (&$frame, $name, $w, $h, $d, $at, $material);
        #[allow(unused_mut)]
        let mut o = $crate::shapes::Cube::default();
        $(o.$k = $crate::Arg::arg($v);)*
        args.0.box_(args.1, $crate::forms::num(args.2), $crate::forms::num(args.3), $crate::forms::num(args.4), args.5, args.6, o)
    }};
}

/// `GameFrame::cyl` with its options by name: `fcyl!(F, name, r, h, at, material, key = value, ...)`.
#[macro_export]
macro_rules! fcyl {
    ($frame:expr, $name:expr, $r:expr, $h:expr, $at:expr, $material:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        // (as Python: what is given by place first, then the options in the order written)
        let args = (&$frame, $name, $r, $h, $at, $material);
        #[allow(unused_mut)]
        let mut o = $crate::shapes::Round::default();
        $(o.$k = $crate::Arg::arg($v);)*
        args.0.cyl(args.1, $crate::forms::num(args.2), $crate::forms::num(args.3), args.4, args.5, o)
    }};
}

/// `GameFrame::cone` with its options by name: `fcone!(F, name, r1, r2, h, at, material, key = value, ...)`.
#[macro_export]
macro_rules! fcone {
    ($frame:expr, $name:expr, $r1:expr, $r2:expr, $h:expr, $at:expr, $material:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        // (as Python: what is given by place first, then the options in the order written)
        let args = (&$frame, $name, $r1, $r2, $h, $at, $material);
        #[allow(unused_mut)]
        let mut o = $crate::shapes::Round::default();
        $(o.$k = $crate::Arg::arg($v);)*
        args.0.cone(args.1, $crate::forms::num(args.2), $crate::forms::num(args.3), $crate::forms::num(args.4), args.5, args.6, o)
    }};
}

/// `GameFrame::sphere` with its options by name: `fsphere!(F, name, r, at, material, key = value, ...)`.
#[macro_export]
macro_rules! fsphere {
    ($frame:expr, $name:expr, $r:expr, $at:expr, $material:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        // (as Python: what is given by place first, then the options in the order written)
        let args = (&$frame, $name, $r, $at, $material);
        #[allow(unused_mut)]
        let mut o = $crate::shapes::Sphere::default();
        $(o.$k = $crate::Arg::arg($v);)*
        args.0.sphere(args.1, $crate::forms::num(args.2), args.3, args.4, o)
    }};
}

/// `GameFrame::sweep` with its options by name: `fsweep!(F, name, points, radius, material, key = value, ...)` (a
/// radius of 0 where the script gave none: a `profile` is swept).
#[macro_export]
macro_rules! fsweep {
    ($frame:expr, $name:expr, $points:expr, $radius:expr, $material:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        // (as Python: what is given by place first, then the options in the order written)
        let args = (&$frame, $name, $points, $radius, $material);
        #[allow(unused_mut)]
        let mut o = $crate::forms::Sweep::default();
        $(o.$k = $crate::Arg::arg($v);)*
        args.0.sweep(args.1, args.2, $crate::forms::num(args.3), args.4, o)
    }};
}

/// `GameFrame::extrude` with its options by name: `fextrude!(F, name, outline, depth, at, material, key = value, ...)`;
/// `plane = Facing::Top` for the script's `plane='top'`.
#[macro_export]
macro_rules! fextrude {
    ($frame:expr, $name:expr, $outline:expr, $depth:expr, $at:expr, $material:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        // (as Python: what is given by place first, then the options in the order written)
        let args = (&$frame, $name, $outline, $depth, $at, $material);
        #[allow(unused_mut)]
        let mut o = $crate::forms::Extrude::default();
        $(o.$k = $crate::Arg::arg($v);)*
        args.0.extrude(args.1, args.2, $crate::forms::num(args.3), args.4, args.5, o)
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
