//! What the tabletop things are built from (Blender axes: x across, y to the back, the front at -y; z up; metres):
//! small spheres, cylinders and boxes, turned bodies, slabs, swept bars and flat faces.

use artkit::forms::{self, Holes, Plane, Scale};
use artkit::materials::Mat;
use artkit::math::{M4, P3, TAU, V3, cos, cosd, sin, sind};
use artkit::parts::{join_all, settle};
use artkit::scene::Obj;
use artkit::{cube, cylinder, extrude, loft, ring, sphere, sweep};

/// `ball`'s options.
#[derive(Clone, Debug)]
pub struct Ball {
    pub scale: P3,
    pub rot: P3,
    pub seg: usize,
    pub rings: usize,
}

impl Default for Ball {
    fn default() -> Ball {
        Ball {
            scale: P3::ONE,
            rot: P3::ZERO,
            seg: 10,
            rings: 7,
        }
    }
}

/// A low sphere: these are small, so few segments.
pub fn ball(name: impl AsRef<str>, r: f64, at: impl Into<P3>, material: Mat, o: Ball) -> Obj {
    sphere!(
        name,
        r,
        at = at.into(),
        material = material,
        scale = o.scale,
        rot = o.rot,
        segments = o.seg,
        rings = o.rings
    )
}

/// `ball` with its options by name: `ball!(name, r, at, material, key = value, ...)`.
macro_rules! ball {
    ($name:expr, $r:expr, $at:expr, $m:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        let args = ($name, $r, $at, $m);
        #[allow(unused_mut)]
        let mut o = $crate::tabletop::common::Ball::default();
        $(o.$k = artkit::Arg::arg($v);)*
        $crate::tabletop::common::ball(args.0, args.1, args.2, args.3, o)
    }};
}

/// `cyl`'s options.
#[derive(Clone, Debug)]
pub struct Cyl {
    pub verts: usize,
    pub rot: P3,
    pub bevel: f64,
}

impl Default for Cyl {
    fn default() -> Cyl {
        Cyl {
            verts: 16,
            rot: P3::ZERO,
            bevel: 0.0,
        }
    }
}

pub fn cyl(name: impl AsRef<str>, r: f64, h: f64, at: impl Into<P3>, material: Mat, o: Cyl) -> Obj {
    cylinder!(
        name,
        r,
        h,
        at = at.into(),
        material = material,
        verts = o.verts,
        rot = o.rot,
        bevel = o.bevel,
        bevel_segments = 1
    )
}

/// `cyl` with its options by name: `cyl!(name, r, h, at, material, key = value, ...)`.
macro_rules! cyl {
    ($name:expr, $r:expr, $h:expr, $at:expr, $m:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        let args = ($name, $r, $h, $at, $m);
        #[allow(unused_mut)]
        let mut o = $crate::tabletop::common::Cyl::default();
        $(o.$k = artkit::Arg::arg($v);)*
        $crate::tabletop::common::cyl(args.0, args.1, args.2, args.3, args.4, o)
    }};
}

/// `box_`'s options.
#[derive(Clone, Debug)]
pub struct Box_ {
    pub bevel: f64,
    pub rot: P3,
    pub segs: usize,
}

impl Default for Box_ {
    fn default() -> Box_ {
        Box_ {
            bevel: 0.0,
            rot: P3::ZERO,
            segs: 1,
        }
    }
}

/// The script's `box`.
pub fn box_(
    name: impl AsRef<str>,
    size: impl Into<P3>,
    at: impl Into<P3>,
    material: Mat,
    o: Box_,
) -> Obj {
    cube!(
        name,
        size.into(),
        at = at.into(),
        material = material,
        bevel = o.bevel,
        rot = o.rot,
        bevel_segments = o.segs
    )
}

/// `box_` with its options by name: `box_!(name, size, at, material, key = value, ...)`.
macro_rules! box_ {
    ($name:expr, $size:expr, $at:expr, $m:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        let args = ($name, $size, $at, $m);
        #[allow(unused_mut)]
        let mut o = $crate::tabletop::common::Box_::default();
        $(o.$k = artkit::Arg::arg($v);)*
        $crate::tabletop::common::box_(args.0, args.1, args.2, args.3, o)
    }};
}

/// (y, z) turned about the x axis by `degrees` (negative leans the top back, to +y).
pub fn tilt_x(y: f64, z: f64, degrees: f64) -> (f64, f64) {
    let (c, s) = (cosd(degrees), sind(degrees));
    (y * c - z * s, y * s + z * c)
}

/// Part of a tube along x, closed: the ring between `r_in` and `r_out` from angle `a0` to `a1` (degrees;
/// 0 is to the front, -y, and 90 straight up), `length` long, centred on `at`.
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn arc_shell(
    name: &str,
    r_out: f64,
    r_in: f64,
    length: f64,
    a0: f64,
    a1: f64,
    at: impl Into<P3>,
    material: Mat,
    segments: usize,
) -> Obj {
    let mut vs: Vec<V3> = Vec::new();
    let mut rings: Vec<[u32; 4]> = Vec::new();
    for i in 0..=segments {
        let a = artkit::math::radians(a0 + (a1 - a0) * i as f64 / segments as f64);
        let mut ring = [0u32; 4];
        let mut k = 0;
        for x in [-length / 2.0, length / 2.0] {
            for r in [r_out, r_in] {
                ring[k] = vs.len() as u32;
                vs.push(V3::from((x, -cos(a) * r, sin(a) * r)));
                k += 1;
            }
        }
        rings.push(ring); // [x0 out, x0 in, x1 out, x1 in]
    }
    let mut faces: Vec<Vec<u32>> = Vec::new();
    for w in rings.windows(2) {
        let (a, b) = (w[0], w[1]);
        faces.push(vec![a[0], b[0], b[2], a[2]]); // outside
        faces.push(vec![a[1], a[3], b[3], b[1]]); // inside
        faces.push(vec![a[0], a[1], b[1], b[0]]); // the end at -x
        faces.push(vec![a[2], b[2], b[3], a[3]]); // the end at +x
    }
    for ring in [rings[0], rings[rings.len() - 1]] {
        faces.push(vec![ring[0], ring[2], ring[3], ring[1]]);
    }
    // (made flat, its faces turned to look outwards)
    let obj = forms::object(name, vs, &faces, Some(material), false, 40.0, false);
    obj.set_location(at)
}

/// A still model: everything joined into one node, stood on the floor and centred. `mirror` flips it left
/// to right, for one built the other way round from its old part.
pub fn done(name: &str, mirror: bool) -> Vec<Obj> {
    let obj = join_all(name);
    if mirror {
        obj.edit(|o| {
            o.mesh.transform(&M4::diagonal(V3::new(-1.0, 1.0, 1.0)));
            o.mesh.flip_normals();
        });
    }
    settle(&[obj])
}

/// A row of a turned body: (r, z), or (rx, ry, z) for an oval or a rounded square.
pub trait Row: Copy {
    fn row(self) -> (f64, f64, f64);
}
impl Row for (f64, f64) {
    fn row(self) -> (f64, f64, f64) {
        (self.0, self.0, self.1)
    }
}
impl Row for (f64, f64, f64) {
    fn row(self) -> (f64, f64, f64) {
        self
    }
}

/// `turned`'s options.
#[derive(Clone, Debug)]
pub struct Turned {
    pub at: P3,
    pub verts: usize,
    pub rot: P3,
    pub square: f64,
    pub caps: bool,
    pub angle: Option<f64>,
}

impl Default for Turned {
    fn default() -> Turned {
        Turned {
            at: P3::ZERO,
            verts: 16,
            rot: P3::ZERO,
            square: 2.0,
            caps: true,
            angle: None,
        }
    }
}

/// A turned (lathed) body through rows of (r, z), or (rx, ry, z) for an oval or (`square`) a rounded square,
/// bottom first, lofted with its rims kept sharp (edges turning over `angle`; by default a little more than its
/// sides turn, so a body of few sides is still round): a radius of 0 closes it to a point there. Built about z,
/// then placed at `at` and turned `rot` (degrees), so it can lie along another axis.
pub fn turned<R: Row>(name: impl AsRef<str>, rows: &[R], material: Mat, o: Turned) -> Obj {
    let angle = o.angle.unwrap_or(if o.square <= 2.5 {
        let a = 360.0 / o.verts as f64 + 8.0;
        if a > 40.0 { a } else { 40.0 }
    } else {
        40.0
    });
    let secs: Vec<Vec<P3>> = rows
        .iter()
        .map(|&row| {
            let (rx, ry, z) = row.row();
            ring!(rx, ry, z = z, verts = o.verts, square = o.square)
        })
        .collect();
    loft!(
        name,
        &secs,
        material = material,
        caps = o.caps,
        at = o.at,
        rot = o.rot,
        angle = angle
    )
}

/// `turned` with its options by name: `turned!(name, rows, material, key = value, ...)`.
macro_rules! turned {
    ($name:expr, $rows:expr, $m:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        let args = ($name, $rows, $m);
        #[allow(unused_mut)]
        let mut o = $crate::tabletop::common::Turned::default();
        $(o.$k = artkit::Arg::arg($v);)*
        $crate::tabletop::common::turned(args.0, args.1, args.2, o)
    }};
}

/// A flat face looking up (a drink's surface, a slice's cut face): one polygon, `verts` corners (seen from
/// above only, so no underside).
pub fn disc(
    name: &str,
    r: f64,
    at: (f64, f64, f64),
    material: Mat,
    verts: usize,
    rx: Option<f64>,
) -> Obj {
    let (x, y, z) = at;
    let rx = rx.filter(|&v| v != 0.0).unwrap_or(r);
    let pts: Vec<V3> = (0..verts)
        .map(|i| {
            V3::from((
                x + rx * cos(TAU * i as f64 / verts as f64),
                y + r * sin(TAU * i as f64 / verts as f64),
                z,
            ))
        })
        .collect();
    forms::object(
        name,
        pts,
        &[(0..verts as u32).collect()],
        Some(material),
        false,
        40.0,
        false,
    )
}

/// `bar`'s options.
#[derive(Clone, Debug)]
pub struct Bar {
    pub scale: Scale,
    pub up: P3,
    pub resolution: usize,
    pub angle: f64,
}

impl Default for Bar {
    fn default() -> Bar {
        Bar {
            scale: Scale::One(1.0),
            up: P3::from((0, 0, 1)),
            resolution: 1,
            angle: 60.0,
        }
    }
}

/// A profile swept through Blender points, its corners as given (round them with `fillet` first).
pub fn bar<P: Into<V3> + Copy>(
    name: impl AsRef<str>,
    points: &[P],
    profile: Vec<(f64, f64)>,
    material: Mat,
    o: Bar,
) -> Obj {
    sweep!(
        name,
        points,
        0.0,
        profile = profile,
        material = material,
        scale = o.scale,
        up = o.up,
        resolution = o.resolution,
        angle = o.angle
    )
}

/// `bar` with its options by name: `bar!(name, points, profile, material, key = value, ...)`.
macro_rules! bar {
    ($name:expr, $points:expr, $profile:expr, $m:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        let args = ($name, $points, $profile, $m);
        #[allow(unused_mut)]
        let mut o = $crate::tabletop::common::Bar::default();
        $(o.$k = artkit::Arg::arg($v);)*
        $crate::tabletop::common::bar(args.0, args.1, args.2, args.3, o)
    }};
}

/// An oval profile `w` across its `u` axis and `t` across `v` (a flat handle, a spoon's stem).
pub fn oval(w: f64, t: f64, verts: usize) -> Vec<(f64, f64)> {
    (0..verts)
        .map(|i| {
            (
                w / 2.0 * cos(TAU * i as f64 / verts as f64),
                t / 2.0 * sin(TAU * i as f64 / verts as f64),
            )
        })
        .collect()
}

/// `slab`'s options.
#[derive(Clone, Debug, Default)]
pub struct Slab {
    pub plane: Plane,
    pub rot: P3,
    pub bevel: f64,
    pub holes: Holes,
}

/// `forms::extrude` with one bevel segment (small things: the corners only need catching the light).
pub fn slab(
    name: impl AsRef<str>,
    outline: &[(f64, f64)],
    depth: f64,
    at: impl Into<P3>,
    material: Mat,
    o: Slab,
) -> Obj {
    extrude!(
        name,
        outline,
        depth,
        at = at.into(),
        material = material,
        plane = o.plane,
        rot = o.rot,
        bevel = o.bevel,
        bevel_segments = 1,
        holes = o.holes
    )
}

/// `slab` with its options by name: `slab!(name, outline, depth, at, material, key = value, ...)`.
macro_rules! slab {
    ($name:expr, $outline:expr, $depth:expr, $at:expr, $m:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        let args = ($name, $outline, $depth, $at, $m);
        #[allow(unused_mut)]
        let mut o = $crate::tabletop::common::Slab::default();
        $(o.$k = artkit::Arg::arg($v);)*
        $crate::tabletop::common::slab(args.0, args.1, args.2, args.3, args.4, o)
    }};
}

/// Moves made parts by `matrix` (a part built upright at the origin, then leaned and lifted into place).
pub fn _place(objs: &[Obj], matrix: M4) {
    for &o in objs {
        o.set_matrix_world(matrix * o.matrix_world());
    }
}
