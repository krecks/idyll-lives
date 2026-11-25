//! What the lamps are built from: the palette, the content's lights the bulbs are placed against, and the turned,
//! lofted and lathed pieces more than one lamp takes (canopies, cords, sockets, drum shades, rings).

use artkit::json::{Json, loads};
use artkit::kit;
use artkit::materials::Mat;
use artkit::math::{P3, V3, cos, cosd, radians, sin, sind};
use artkit::palette;
use artkit::place::{Coord, GameFrame};
use artkit::py;
use artkit::scene::{Obj, meshes};
use artkit::{fsphere, loft, revolve, ring, torus};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::sync::OnceLock;

palette! { M:
    // fittings
    brass = ("#B8925A", metal),
    iron = ("#2B2C2E", metal),         // black metal: cords, plates, cages, track heads
    white = ("#F2F2F0", satin),        // the fan, the track, the paper lantern's canopy
    bulb = ("#FFF1CF", gloss, glow("#FFD9A0")),   // bulbs and diffusers
    flame = ("#FFD27A", gloss, glow("#FFC46A")),
    // the ceiling lights
    dome = ("#F2F0EA", satin, glow("#FFD9A0")),   // the pendant's dome
    panel = ("#FBF4E4", satin, glow("#FFD9A0")),  // the ceiling light's frosted sides
    industrial = ("#2F4A3A", metal),   // the industrial pendant's enamel dome
    liner = ("#F2F0EA", satin, glow("#FFD9A0")),  // the white inside of a metal shade
    blade = ("#CDB08A", wood),
    paper = ("#FBF7EE", fabric, glow("#FFD9A0")),
    rib = ("#EDE6D6", fabric),
    // the wall lights
    shade = ("#F4EBD8", fabric, glow("#FFD9A0")),  // drum and cone shades
    shade_metal = ("#2B2C2E", metal),   // the swing-arm lamp's shade
    lens = ("#F4EEDC", glass, glow("#FFD9A0")),    // the bulkhead's lens
    // the floor lamps
    stand = ("#3A3530", metal),
    lamp_shade = ("#F2E6CC", fabric, glow("#FFD9A0")),
    trim = ("#E3D2B0", fabric),
    marble = ("#E7E3DC", gloss, roughness(0.28)),
    arc = ("#C9A06A", metal),
    arc_shade = ("#C9A06A", metal),
    walnut = ("#7A5234", wood),
    linen = ("#E7E0D4", fabric, glow("#FFD9A0")),
    bowl = ("#F2EBDD", glass, glow("#FFD9A0")),
    // the table lamps
    oak = ("#CDB08A", wood),
    ceramic = ("#E8E2D6", gloss),
    jar = ("#3E6E9A", gloss),
    stone = ("#E7E3DC", matte),
    square_shade = ("#EFE8DA", fabric, glow("#FFD9A0")),
    banker = ("#2F6B4F", gloss, glow("#4FB27A")),   // green glass: lit, it glows green, not cream
    porcelain = ("#F2F0EA", gloss),
    cap = ("#E9A07B", gloss, glow("#FFAA78")),      // coral glass: lit, it glows coral
    candle = ("#F4EBD8", satin),
    glass = ("#DCE6EA", glass),
}

pub const F: GameFrame = GameFrame::new(0.0);
pub use artkit::kit::WALL;
pub const CEILING: f64 = 2.8;

/// The content files whose objects' `light` the bulbs are placed against, in the script's order.
const CONTENT: [&str; 3] = [
    include_str!("../../../../../web/public/content/base.json"),
    include_str!("../../../../../web/public/content/furniture.json"),
    include_str!("../../../../../web/public/content/catalog2/lighting.json"),
];

/// {object id: its content `light`}: the bulbs are placed against it.
fn lights() -> &'static BTreeMap<String, Json> {
    static LIGHTS: OnceLock<BTreeMap<String, Json>> = OnceLock::new();
    LIGHTS.get_or_init(|| {
        let mut out = BTreeMap::new();
        for text in CONTENT {
            let doc = loads(text).expect("the content is JSON");
            if let Some(Json::Arr(objects)) = doc.get("objects") {
                for o in objects {
                    if let (Some(light), Some(id)) =
                        (o.get("light"), o.get("id").and_then(Json::str))
                    {
                        out.insert(id.to_string(), light.clone());
                    }
                }
            }
        }
        out
    })
}

const AT_TOLERANCE: f64 = 0.05; // how far across (m) a bulb may stand from the content's `light.at`

thread_local! {
    /// {object id: its bulb's material name}, for `check_at` once the model is built.
    static BULBS: RefCell<BTreeMap<String, &'static str>> = const { RefCell::new(BTreeMap::new()) };
}

/// Asserts a bulb at height `y` is within 25 cm of where the content puts the lamp's light; notes its material, so
/// that once the model is built and settled `check_at` finds it across.
pub fn near_light(id: &str, y: f64, material: Option<Mat>) {
    let want = lights()
        .get(id)
        .and_then(|l| l.get("height"))
        .unwrap_or_else(|| panic!("{id}: the content gives no light height"))
        .num()
        .expect("a light's number is a number");
    assert!(
        (y - want).abs() <= 0.25,
        "{id}: bulb at {y:.2} m, the content's light at {want:.2} m"
    );
    BULBS.with(|b| {
        b.borrow_mut()
            .insert(id.to_string(), material.unwrap_or(M.bulb).name)
    });
}

/// Asserts the built lamp's bulb (the middle of the faces in its material, a diffuser's with it) stands within
/// `AT_TOLERANCE` across of the content's `light.at`, for a lamp that gives one (the arc lamp's shade, the table lamp
/// on its table); the others' light is at the footprint's middle, as the wall lights' is on purpose. Returns every
/// mesh, the model's parts.
pub fn check_at(id: &str) -> Vec<Obj> {
    let at = lights().get(id).and_then(|l| l.get("at"));
    let bulb = BULBS.with(|b| b.borrow().get(id).copied());
    let (Some(Json::Arr(at)), Some(bulb)) = (at, bulb) else {
        return meshes();
    };
    let mut pts: Vec<V3> = Vec::new();
    for o in meshes() {
        let m = o.matrix_world();
        o.read(|p| {
            for (i, f) in p.mesh.faces.iter().enumerate() {
                if p.mesh
                    .mats
                    .get(f.mat as usize)
                    .is_some_and(|mat| mat.name == bulb)
                {
                    pts.extend(
                        p.mesh
                            .corners(i)
                            .iter()
                            .map(|&v| m * p.mesh.verts[v as usize]),
                    );
                }
            }
        });
    }
    assert!(
        !pts.is_empty(),
        "{id}: no faces in its bulb's material {bulb:?}"
    );
    let n = pts.len() as f64;
    let x = -py::sum(pts.iter().map(|p| p.x as f64)) / n; // game x
    let z = -py::sum(pts.iter().map(|p| p.y as f64)) / n; // game depth
    let (ax, az) = (
        at[0].num().expect("a light's number is a number"),
        at[1].num().expect("a light's number is a number"),
    );
    let off = py::hypot(x - ax, z - az);
    assert!(
        off <= AT_TOLERANCE,
        "{id}: bulb at ({x:.3}, {z:.3}), the content's light.at ({ax}, {az}): {:.1} cm off",
        off * 100.0
    );
    meshes()
}

/// The lamp's bulb at a game position (`near_light`, `check_at`).
#[allow(clippy::too_many_arguments)]
pub fn bulb<A: Coord, C: Coord>(
    id: &str,
    r: f64,
    at: (A, f64, C),
    material: Option<Mat>,
    scale: (f64, f64, f64),
    segments: usize,
    rings: usize,
) -> Obj {
    near_light(id, at.1, material);
    fsphere!(
        F,
        format!("{id}_bulb"),
        r,
        at,
        material.unwrap_or(M.bulb),
        scale = scale,
        segments = segments,
        rings = rings
    )
}

/// A pear-shaped bulb (globe below, neck above) whose globe's middle is at game `at` (`near_light`).
pub fn pear<A: Coord, C: Coord>(id: &str, r: f64, at: (A, f64, C), segments: usize) -> Obj {
    near_light(id, at.1, None);
    let prof = [
        (0.0, -r),
        (r * 0.55, -r * 0.85),
        (r * 0.9, -r * 0.45),
        (r, 0.0),
        (r * 0.9, r * 0.45),
        (r * 0.62, r * 0.9),
        (r * 0.45, r * 1.25),
        (r * 0.45, r * 1.55),
        (0.0, r * 1.55),
    ];
    revolve!(
        format!("{id}_bulb"),
        &prof,
        at = F.at(at.0, at.1, at.2),
        material = M.bulb,
        segments = segments
    )
}

/// A turned part: `profile` [(radius, up), ...] in metres from its foot at game point `at`, bottom first.
pub fn turned<A: Coord, B: Coord, C: Coord>(
    name: &str,
    profile: &[(f64, f64)],
    at: (A, B, C),
    material: Mat,
    segments: usize,
    rot: (f64, f64, f64),
) -> Obj {
    revolve!(
        name,
        profile,
        at = F.at(at.0, at.1, at.2),
        material = material,
        segments = segments,
        rot = rot
    )
}

/// A turned part, upright.
pub fn turn<A: Coord, B: Coord, C: Coord>(
    name: &str,
    profile: &[(f64, f64)],
    at: (A, B, C),
    material: Mat,
    segments: usize,
) -> Obj {
    turned(name, profile, at, material, segments, (0.0, 0.0, 0.0))
}

/// A level ring (a rolled rim, a wire, a bead) of radius `r` and thickness `t` round game point `at`.
#[allow(clippy::too_many_arguments)]
pub fn ring_at<A: Coord, B: Coord, C: Coord>(
    name: &str,
    r: f64,
    at: (A, B, C),
    t: f64,
    material: Mat,
    segments: usize,
    minor: usize,
) -> Obj {
    torus!(
        name,
        r,
        t,
        at = F.at(at.0, at.1, at.2),
        material = material,
        major_segments = segments,
        minor_segments = minor
    )
}

/// A body lofted through level rings [(height, half width, half depth, squareness), ...] round game (x, z), bottom
/// first: a rounded-square canopy, a lantern's roof, a banker's base.
pub fn loft_at(
    name: &str,
    x: impl Coord,
    z: impl Coord,
    rings: &[(f64, f64, f64, f64)],
    material: Mat,
    verts: usize,
) -> Obj {
    let b = F.at(x, 0, z);
    let (bx, by) = (b.x, b.y);
    let secs: Vec<Vec<P3>> = rings
        .iter()
        .map(|&(y, rx, rz, sq)| ring!(rx, rz, z = y, verts = verts, square = sq, at = (bx, by)))
        .collect();
    loft!(name, &secs, material = material, caps = true)
}

/// `lathe`'s options.
#[derive(Clone, Copy)]
pub struct Lathe {
    pub segments: usize,
    pub rot: (f64, f64, f64),
    pub top: bool,
    pub liner: Option<Mat>,
}

impl Default for Lathe {
    fn default() -> Lathe {
        Lathe {
            segments: 24,
            rot: (0.0, 0.0, 0.0),
            top: false,
            liner: None,
        }
    }
}

/// An old part's lathe: `profile` in fractions of `w` across and `h` high (-0.5 … 0.5), centred on game `at`,
/// closed if it doesn't end on the axis or where it starts. `top`: `at` is its top middle instead, its origin (to
/// tilt it about where it hangs). `liner` (a material): a thin shell of it just inside the profile's inner face, the
/// points after the axis (a metal shade's white inside).
#[allow(clippy::too_many_arguments)]
pub fn lathe<A: Coord, C: Coord>(
    name: &str,
    w: f64,
    h: f64,
    at: (A, f64, C),
    profile: &[(f64, f64)],
    material: Mat,
    o: Lathe,
) -> Vec<Obj> {
    let mut pts: Vec<(f64, f64)> = profile
        .iter()
        .map(|&(r, z)| (r * w, (z - if o.top { 0.5 } else { -0.5 }) * h))
        .collect();
    if pts[pts.len() - 1].0 > 1e-6 && pts[pts.len() - 1] != pts[0] {
        pts.push(pts[0]);
    }
    let (x, y, z) = at;
    let base = F.at(x, if o.top { y } else { y - h / 2.0 }, z);
    let mut out = vec![revolve!(
        name,
        &pts,
        at = base,
        material = material,
        segments = o.segments,
        rot = o.rot
    )];
    if let Some(liner) = o.liner {
        let k = (1..profile.len())
            .rev()
            .find(|&i| profile[i].0 < 1e-6)
            .expect("a lined lathe crosses its axis");
        let inner: Vec<(f64, f64)> = profile[k + 1..]
            .iter()
            .zip(&pts[k + 1..])
            .map(|(&(r, _), &(_, zz))| (r * w, zz))
            .collect();
        let mut band: Vec<(f64, f64)> = inner.iter().map(|&(r, z)| (r * 0.97, z)).collect();
        band.extend(inner.iter().rev().map(|&(r, z)| (r * 0.93, z)));
        band.push(band[0]);
        out.push(revolve!(
            format!("{name}_liner"),
            &band,
            at = base,
            material = liner,
            segments = o.segments,
            rot = o.rot
        ));
    }
    out
}

/// `drum`'s options.
#[derive(Clone, Copy)]
pub struct Drum {
    pub segments: usize,
    pub t: f64,
    pub rot: (f64, f64, f64),
    pub rims: Option<Mat>,
    pub rim: f64,
    pub spider: Option<Mat>,
}

impl Default for Drum {
    fn default() -> Drum {
        Drum {
            segments: 28,
            t: 0.006,
            rot: (0.0, 0.0, 0.0),
            rims: None,
            rim: 0.0045,
            spider: None,
        }
    }
}

/// An open drum or cone shade, `t` thick, its bottom at game `at`. `rims` (a material): a rolled rim round its top
/// and bottom edges, as a lampshade's binding is; `spider` (a material): the wire fitter inside its top, three spokes
/// to a washer, seen from above.
#[allow(clippy::too_many_arguments)]
pub fn drum<A: Coord, C: Coord>(
    name: &str,
    r_bottom: f64,
    r_top: f64,
    h: f64,
    at: (A, f64, C),
    material: Mat,
    o: Drum,
) {
    let Drum {
        segments,
        t,
        rot,
        rims,
        rim,
        spider,
    } = o;
    revolve!(
        name,
        &[
            (r_bottom, 0.0),
            (r_top, h),
            (r_top - t, h),
            (r_bottom - t, 0.0),
            (r_bottom, 0.0)
        ],
        at = F.at(at.0, at.1, at.2),
        material = material,
        segments = segments,
        smooth = segments > 4,
        rot = rot
    );
    let (x, y, z) = at;
    if let Some(rims) = rims {
        ring_at(
            &format!("{name}_rim_low"),
            r_bottom - t / 2.0,
            (x, y + rim * 0.6, z),
            rim,
            rims,
            segments,
            5,
        );
        ring_at(
            &format!("{name}_rim_high"),
            r_top - t / 2.0,
            (x, y + h - rim * 0.6, z),
            rim,
            rims,
            segments,
            5,
        );
    }
    if let Some(spider) = spider {
        let top = y + h - 0.012;
        let (xf, zf) = (x.f(), z.f());
        for a in [90, 210, 330] {
            let af = a as f64;
            kit::tube!(
                F,
                format!("{name}_spoke{a}"),
                &[
                    (xf + 0.016 * cosd(af), top, zf + 0.016 * sind(af)),
                    (
                        xf + (r_top - t) * cosd(af),
                        top,
                        zf + (r_top - t) * sind(af)
                    )
                ],
                0.0018,
                spider,
                verts = 4,
                round = 1
            );
        }
        ring_at(
            &format!("{name}_washer"),
            0.016,
            (x, top, z),
            0.003,
            spider,
            10,
            4,
        );
    }
}

/// A half tube, open below, along Blender x: a banker's lamp shade, `t` thick, coming 20 degrees down past the
/// axis on either side.
pub fn trough(name: &str, r: f64, length: f64, t: f64, at: P3, material: Mat) -> Obj {
    let (steps, below) = (12, 20.0);
    let mut verts: Vec<V3> = Vec::new();
    let n = steps + 1;
    for e in [-length / 2.0, length / 2.0] {
        for rr in [r, r - t] {
            for i in 0..n {
                let a = radians(-below + (180.0 + 2.0 * below) * i as f64 / steps as f64);
                verts.push(
                    P3 {
                        x: e,
                        y: rr * cos(a),
                        z: rr * sin(a),
                    }
                    .into(),
                );
            }
        }
    }
    let v = |e: usize, inner: usize, i: usize| (e * 2 * n + inner * n + i) as u32;
    let mut faces: Vec<Vec<u32>> = Vec::new();
    for i in 0..steps {
        faces.push(vec![v(0, 0, i), v(0, 0, i + 1), v(1, 0, i + 1), v(1, 0, i)]); // outside
        faces.push(vec![v(0, 1, i), v(1, 1, i), v(1, 1, i + 1), v(0, 1, i + 1)]); // inside
        for e in [0, 1] {
            faces.push(vec![v(e, 0, i), v(e, 1, i), v(e, 1, i + 1), v(e, 0, i + 1)]); // the ends
        }
    }
    for i in [0, steps] {
        faces.push(vec![v(0, 0, i), v(1, 0, i), v(1, 1, i), v(0, 1, i)]); // the lower edges
    }
    let obj = artkit::forms::object(name, verts, &faces, Some(material), true, 40.0, false);
    obj.set_location(at)
}

/// A ceiling canopy: a shallow turned cup against the ceiling, `r` across and `h` deep, a cord grip below.
pub fn canopy(name: &str, r: f64, h: f64, material: Mat, segments: usize, y: f64) -> Obj {
    turn(
        name,
        &[
            (0.0, 0.0),
            (r * 0.22, 0.0),
            (r * 0.25, h * 0.2),
            (r * 0.42, h * 0.32),
            (r * 0.82, h * 0.5),
            (r * 0.97, h * 0.72),
            (r, h * 0.85),
            (r, h),
            (0.0, h),
        ],
        (0, y - h, 0),
        material,
        segments,
    )
}

/// A ceiling canopy (brass, or `canopy`) and the cord down from it to `cord_bottom`, through a cord grip.
pub fn hang(id: &str, cord_bottom: f64, canopy_mat: Option<Mat>, canopy_r: f64, cord: Option<Mat>) {
    let canopy_mat = canopy_mat.unwrap_or(M.brass);
    canopy(
        &format!("{id}_canopy"),
        canopy_r,
        0.03,
        canopy_mat,
        20,
        CEILING,
    );
    kit::tube!(
        F,
        format!("{id}_cord"),
        &[(0, CEILING - 0.03, 0), (0, cord_bottom, 0)],
        0.0045,
        cord.unwrap_or(M.iron),
        verts = 6,
        round = 1
    );
}

/// A lamp socket from its foot at game `at`: a skirt, a body with two turned ribs, a cap.
pub fn socket<A: Coord, C: Coord>(
    name: &str,
    r: f64,
    h: f64,
    at: (A, f64, C),
    material: Mat,
    segments: usize,
) -> Obj {
    turn(
        name,
        &[
            (0.0, 0.0),
            (r * 0.82, 0.0),
            (r * 0.9, h * 0.08),
            (r * 0.9, h * 0.45),
            (r, h * 0.5),
            (r, h * 0.58),
            (r * 0.9, h * 0.62),
            (r * 0.9, h * 0.72),
            (r, h * 0.76),
            (r, h * 0.84),
            (r * 0.9, h * 0.88),
            (r * 0.7, h),
            (0.0, h),
        ],
        at,
        material,
        segments,
    )
}

pub const DOME: [(f64, f64); 10] = [
    (0.5, -0.5),
    (0.47, 0.0),
    (0.36, 0.3),
    (0.14, 0.47),
    (0.0, 0.5),
    (0.0, 0.42),
    (0.13, 0.4),
    (0.33, 0.24),
    (0.44, -0.06),
    (0.47, -0.5),
];
