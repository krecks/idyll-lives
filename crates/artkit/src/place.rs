//! Placing parts in the game's axes. The content and the old primitive parts wrote (x, height, depth) with the
//! front at +depth; Blender is (x, y, z) with the front at -y and the game's x mirrored. A `GameFrame` takes
//! the game's numbers as they are:
//!
//!   const F: GameFrame = GameFrame::new(0.094);
//!   fbox!(F, "door", 0.728, 1.146, 0.04, (0, 0.643, 0.195), M.body, bevel = 0.01);   // w, h, d at (x, y, z)
//!
//! `shift` moves every depth forward (+z, to the front) by that much, a negative one back: a thing built
//! against the back of its tile, its parts' depths centred on -shift, is drawn centred on the tile this way.
//! `depths` redraws a thing's numbers at another depth: pairs (game depth, drawn depth), ascending; depths
//! between two pairs are mapped linearly, beyond the ends moved as the end is. A box's back and front are
//! mapped, so it keeps its place between them; anything else moves by its middle.

use crate::forms::{self, Extrude, Plane, Sweep};
use crate::materials::Mat;
use crate::math::{Num, P3};
use crate::scene::Obj;
use crate::shapes::{self, Cube, Round, Sphere};

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

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GameFrame {
    pub shift: f64,
    pub depths: &'static [(f64, f64)],
}

impl GameFrame {
    pub const fn new(shift: f64) -> GameFrame {
        GameFrame { shift, depths: &[] }
    }

    /// With its depths redrawn: pairs (game depth, drawn depth), ascending.
    pub const fn depths(mut self, depths: &'static [(f64, f64)]) -> GameFrame {
        self.depths = depths;
        self
    }

    /// A game depth as drawn (through `depths`, if given).
    pub fn depth(&self, z: f64) -> f64 {
        let d = self.depths;
        if d.is_empty() {
            return z;
        }
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
    pub fn at(&self, x: impl Num, y: impl Num, z: impl Num) -> P3 {
        P3 {
            x: -x.f(),
            y: -self.depth(z.f()) - self.shift,
            z: y.f(),
        }
    }

    fn place(&self, at: impl Into<P3>) -> P3 {
        let at: P3 = at.into();
        self.at(at.x, at.y, at.z)
    }

    /// A box of game width, height and depth at a game position.
    #[allow(clippy::too_many_arguments)] // (as the scripts called it)
    pub fn box_(
        &self,
        name: impl AsRef<str>,
        w: f64,
        h: f64,
        d: f64,
        at: impl Into<P3>,
        material: Mat,
        mut o: Cube,
    ) -> Obj {
        let at: P3 = at.into();
        o.material = Some(material);
        if !self.depths.is_empty() {
            let (z0, z1) = (self.depth(at.z - d / 2.0), self.depth(at.z + d / 2.0));
            let d = z1 - z0;
            if o.bevel != 0.0 {
                o.bevel = o.bevel.min(d * 0.45);
            }
            o.at = P3 {
                x: -at.x,
                y: -(z0 + z1) / 2.0 - self.shift,
                z: at.y,
            };
            return shapes::cube(name, (w, d, h), o);
        }
        o.at = self.place(at);
        shapes::cube(name, (w, d, h), o)
    }

    /// An upright cylinder of radius and height at a game position.
    pub fn cyl(
        &self,
        name: impl AsRef<str>,
        r: f64,
        h: f64,
        at: impl Into<P3>,
        material: Mat,
        mut o: Round,
    ) -> Obj {
        (o.at, o.material) = (self.place(at), Some(material));
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
        at: impl Into<P3>,
        material: Mat,
        mut o: Round,
    ) -> Obj {
        (o.at, o.material) = (self.place(at), Some(material));
        shapes::cone(name, r1, r2, h, o)
    }

    /// A sphere at a game position.
    pub fn sphere(
        &self,
        name: impl AsRef<str>,
        r: f64,
        at: impl Into<P3>,
        material: Mat,
        mut o: Sphere,
    ) -> Obj {
        (o.at, o.material) = (self.place(at), Some(material));
        shapes::sphere(name, r, o)
    }

    /// A tube or profile swept through game points (`forms::sweep`).
    pub fn sweep<P: Into<P3> + Copy>(
        &self,
        name: impl AsRef<str>,
        points: &[P],
        radius: f64,
        material: Mat,
        mut o: Sweep,
    ) -> Obj {
        o.material = Some(material);
        let points: Vec<P3> = points.iter().map(|&p| self.place(p)).collect();
        forms::sweep(name, &points, radius, o)
    }

    /// A slab from a game outline (`forms::extrude`), drawn as its `plane` says (`Facing`).
    pub fn extrude(
        &self,
        name: impl AsRef<str>,
        outline: &[(f64, f64)],
        depth: f64,
        at: impl Into<P3>,
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
        (o.at, o.material) = (self.place(at), Some(material));
        forms::extrude(name, &outline, depth, o)
    }
}

/// `GameFrame::box_` with its options by name: `fbox!(F, name, w, h, d, at, material, key = value, ...)`.
#[macro_export]
macro_rules! fbox {
    ($frame:expr, $name:expr, $w:expr, $h:expr, $d:expr, $at:expr, $material:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        #[allow(unused_mut)]
        let mut o = $crate::shapes::Cube::default();
        $(o.$k = $crate::Arg::arg($v);)*
        $frame.box_($name, $crate::forms::num($w), $crate::forms::num($h), $crate::forms::num($d), $at, $material, o)
    }};
}

/// `GameFrame::cyl` with its options by name: `fcyl!(F, name, r, h, at, material, key = value, ...)`.
#[macro_export]
macro_rules! fcyl {
    ($frame:expr, $name:expr, $r:expr, $h:expr, $at:expr, $material:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        #[allow(unused_mut)]
        let mut o = $crate::shapes::Round::default();
        $(o.$k = $crate::Arg::arg($v);)*
        $frame.cyl($name, $crate::forms::num($r), $crate::forms::num($h), $at, $material, o)
    }};
}

/// `GameFrame::cone` with its options by name: `fcone!(F, name, r1, r2, h, at, material, key = value, ...)`.
#[macro_export]
macro_rules! fcone {
    ($frame:expr, $name:expr, $r1:expr, $r2:expr, $h:expr, $at:expr, $material:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        #[allow(unused_mut)]
        let mut o = $crate::shapes::Round::default();
        $(o.$k = $crate::Arg::arg($v);)*
        $frame.cone($name, $crate::forms::num($r1), $crate::forms::num($r2), $crate::forms::num($h), $at, $material, o)
    }};
}

/// `GameFrame::sphere` with its options by name: `fsphere!(F, name, r, at, material, key = value, ...)`.
#[macro_export]
macro_rules! fsphere {
    ($frame:expr, $name:expr, $r:expr, $at:expr, $material:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        #[allow(unused_mut)]
        let mut o = $crate::shapes::Sphere::default();
        $(o.$k = $crate::Arg::arg($v);)*
        $frame.sphere($name, $crate::forms::num($r), $at, $material, o)
    }};
}

/// `GameFrame::sweep` with its options by name: `fsweep!(F, name, points, radius, material, key = value, ...)` (a
/// radius of 0 where the script gave none: a `profile` is swept).
#[macro_export]
macro_rules! fsweep {
    ($frame:expr, $name:expr, $points:expr, $radius:expr, $material:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        #[allow(unused_mut)]
        let mut o = $crate::forms::Sweep::default();
        $(o.$k = $crate::Arg::arg($v);)*
        $frame.sweep($name, $points, $crate::forms::num($radius), $material, o)
    }};
}

/// `GameFrame::extrude` with its options by name: `fextrude!(F, name, outline, depth, at, material, key = value, ...)`;
/// `plane = Facing::Top` for the script's `plane='top'`.
#[macro_export]
macro_rules! fextrude {
    ($frame:expr, $name:expr, $outline:expr, $depth:expr, $at:expr, $material:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        #[allow(unused_mut)]
        let mut o = $crate::forms::Extrude::default();
        $(o.$k = $crate::Arg::arg($v);)*
        $frame.extrude($name, $outline, $crate::forms::num($depth), $at, $material, o)
    }};
}
