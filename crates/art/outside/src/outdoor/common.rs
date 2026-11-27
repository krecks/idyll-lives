//! What the outdoor things are built from: the palette, the frame, boards and bars at any angle, and the cups,
//! bottles and tops more than one of them carries.

use artkit::forms::{Plane, rounded_rect};
use artkit::kit;
use artkit::materials::Mat;
use artkit::math::{M3, M4, P3, PI, V3, cos, radians, sin};
use artkit::mesh::Mesh;
use artkit::noise::noise;
use artkit::palette;
use artkit::parts::{join, own};
use artkit::place::{Coord, Facing, Spot};
use artkit::scene::{Obj, meshes};
use artkit::{cylinder, extrude, fcyl, fextrude, fsweep, loft, revolve, ring, settle};

pub use crate::common::{F, bent, rod};

palette! { M:
    iron = ("#2A2C30", metal),           // cast and wrought iron frames
    park_slats = ("#9A6E4A", wood),      // the park bench's slats
    bistro = ("#B5784A", wood),          // the bistro set's slats
    pine = ("#9C7A58", wood),            // the picnic table's planks
    pine_dark = ("#7E5F42", wood),       // its A-frames
    teak = ("#9A6B45", wood),            // the teak table's top, the bar's body
    teak_dark = ("#7E5636", wood),       // the teak table's legs and aprons
    bench_slats = ("#A9744E", wood),     // the garden bench's slats
    bar_top = ("#C9A06A", wood),         // the bar's counter
    steel = ("#8E959B", metal),          // the bar's foot rail
    garden_white = ("#F2F0EA", satin),   // the garden chair
    adirondack = ("#3E7A6E", satin),     // the Adirondack chair
    patio_green = ("#2F5A4E", metal),    // the patio chair's frame
    cushion = ("#E8E2D6", fabric),       // its seat cushion
    mosaic = ("#3E6E73", gloss),         // the round table's top
    tile = ("#E9C46A", gloss),           // its mosaic tiles
    tile_light = ("#F2EEE4", gloss),
    canopy = ("#E7DCC4", fabric),        // the parasol's canopy
    valance = ("#C9A979", fabric),       // its scalloped edge
    pole = ("#E7E3DC", satin),           // its pole
    base = ("#3A3D41", satin),           // its weighted base
    cloth = ("#D9483B", fabric),         // the picnic table's gingham cloth
    cloth_check = ("#F4EFE6", fabric),
    wicker = ("#C9A979", fabric),        // the picnic basket
    wicker_rim = ("#B8955F", fabric),
    handle = ("#8E6E52", wood),          // the basket's handle
    cup = ("#F2F1EE", gloss),            // cups and a jug on the tables
    jug = ("#9DC6CF", gloss),
    // OU3, the grills and the fire pit
    kettle = ("#2A2C30", gloss),         // the charcoal grill's enamelled bowl (Body)
    kettle_lid = ("#2A2C30", gloss),     // its lid (Lid)
    grate = ("#3A3D41", metal),          // cooking grates, legs, the kettle's vent and handle brackets
    handle_steel = ("#D0D3D6", chrome),  // handles and knobs
    wheel = ("#2A2C30", satin),
    coals = ("#3A3330", matte),          // a charcoal or ash bed
    ember = ("#D9542B", gloss, glow("#FF7A2E")),        // the fire pit's embers: red by day, aglow at night
    coal_hot = ("#D9542B", gloss, glow("#FF7A2E")),     // the kettle's coals, aglow at night (not through its lid)
    flame = ("#FF9A3C", gloss, glow("#FFB050")),        // the fire pit's flames, grown while warmed by
    flame_core = ("#FFD27A", gloss, glow("#FFE0A0")),
    burger = ("#6B3A26", satin),         // the grill station's patties waiting on their tray
    cart = ("#C3C7CB", metal),           // the grill station's stainless cart (Cart)
    hood = ("#C3C7CB", metal),           // its hood (Hood)
    worktop = ("#3A3D41", satin),        // its dark counter
    firebox = ("#1F2023", matte),        // inside the grill and under the hood
    gauge = ("#F2F1EE", gloss),          // the hood's thermometer face
    board = ("#CDB08A", wood),           // the chopping board
    tray = ("#E9E6E0", gloss),           // the tray of food
    bottle = ("#6FA6D9", gloss),
    bottle_green = ("#3E6B48", gloss),   // the bar's bottles and glasses
    bottle_amber = ("#8A5A2B", gloss),
    bottle_clear = ("#CFE3E6", gloss),
    pit = ("#8E8A84", matte),            // the fire pit's bowl (Bowl)
    pit_stones = ("#A19C94", matte),     // its capstones (Stones)
    log = ("#6B4A33", bark),
    log_end = ("#B89A72", matte),        // the logs' sawn ends
    // OU2
    lounger = ("#CDB08A", wood),         // the sun lounger's frame
    lounger_pad = ("#E8E2D6", fabric),   // its cushions (the back is upholstered all round)
    towel = ("#E9853B", fabric),         // a beach towel
    pillow = ("#FFFFFF", fabric),
    hammock_frame = ("#B08E66", wood),
    sling = ("#4E7A7F", fabric),         // the hammock's striped sling
    sling_stripe = ("#E8E2D6", fabric),
    porch_frame = ("#8E6E52", wood),     // the porch swing's and the swing set's A-frames
    porch_seat = ("#EDE6D8", satin),
    porch_cushion = ("#7FA3B8", fabric),
    chain = ("#8E959B", satin),          // chains: plain, so they share the seat's draw call
    swing_frame = ("#B08E66", wood),
    swing_seat = ("#D9483B", satin),
    cedar = ("#A9744E", wood),           // the hot tub's staves
    cedar_band = ("#5E4430", wood),      // its bands and steps' edges
    tub_cover = ("#5A5F66", satin),      // its insulated cover
    water = ("#5FB4C2", gloss),
    foam = ("#E6F4F6", gloss),           // bubbles on the water
    // the garden's built things: timber, roof sheet, panels, awning canvas
    struct_timber = ("#7E6247", satin),
    struct_roof = ("#4A4E52", satin),
    panel_glass = ("#22313D", gloss),
    panel_frame = ("#9AA0A6", metal),
    awning_stripe = ("#E7DFCC", satin),
    awning_stripe2 = ("#B8623F", satin),
}

/// A game position or direction as a Blender vector.
pub fn v(p: impl Spot) -> V3 {
    V3::from(p.through(&F))
}

/// Joins the model (but what it carries); `centred`: moves it onto its Model's `centre` (its back onto the wall face), else
/// leaves it where it was built. Its feet are cut flat on the floor first: a leg swept up from it at a slant dips
/// a few mm below, and settling would lift the whole model (its seat) by that.
pub fn finish(name: &str, centred: bool) -> Vec<Obj> {
    // (what it carries, `aside`, stays a part of its own)
    let obj = join(&own(&meshes()), name, Some(P3::ZERO));
    obj.edit(|o| {
        for v in &mut o.mesh.verts {
            if v.z < 0.0 {
                v.z = 0.0;
            }
        }
    });
    settle!(centre = centred);
    meshes()
}

/// `jitter`, but the same in every run: three noise() calls, offset.
pub fn jitter(obj: Obj, amount: f64, size: f64, seed: f64, up: f64) -> Obj {
    let world = obj.world_points();
    obj.edit(|o| {
        for (co, p) in o.mesh.verts.iter_mut().zip(&world) {
            let q = V3::from((
                p.x as f64 / size + seed,
                p.y as f64 / size - seed,
                p.z as f64 / size + seed * 0.5,
            ));
            let n = V3::from((
                noise(q),
                noise(q + V3::from((31.4, 0, 0))),
                noise(q + V3::from((0, 47.1, 0))),
            ));
            *co += V3::from((n.x as f64, n.y as f64, n.z as f64 * up)) * amount;
        }
    });
    obj
}

/// A flat or square bar (`w` across the path's normal, `h` the other way, corners rounded `r`) through game points,
/// its corners bent round `bend`: cast-iron frames, flat straps, rails. `up` (Blender) is the normal's direction.
pub struct Bar {
    pub bend: f64,
    pub steps: usize,
    pub up: P3,
    pub r: Option<f64>,
    pub resolution: usize,
    pub scale: f64,
    pub closed: bool,
}

impl Default for Bar {
    fn default() -> Bar {
        Bar {
            bend: 0.03,
            steps: 3,
            up: P3::from((1, 0, 0)),
            r: None,
            resolution: 1,
            scale: 1.0,
            closed: false,
        }
    }
}

pub fn bar(name: &str, pts: &[(f64, f64, f64)], w: f64, h: f64, material: Mat, o: Bar) -> Obj {
    let path = if o.bend != 0.0 && !o.closed {
        kit::fillet!(pts, o.bend, o.steps)
    } else {
        pts.to_vec()
    };
    let prof = rounded_rect(w, h, o.r.unwrap_or(w.min(h) * 0.3), 1);
    fsweep!(
        F,
        name,
        &path,
        0,
        material,
        profile = prof,
        up = o.up,
        resolution = o.resolution,
        scale = o.scale,
        angle = 60,
        closed = o.closed
    )
}

/// A board from game point `a` towards `b`, `t` thick, cut to `outline` [(across, along from a), ...] (a rounded or
/// pointed end, an arched top), its broad face towards game direction `face`: shaped slats, stringers, arms.
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn plank(
    name: &str,
    a: impl Spot,
    b: impl Spot,
    t: f64,
    material: Mat,
    outline: &[(f64, f64)],
    face: impl Spot,
    bevel: f64,
) -> Obj {
    let (a, b) = (v(a), v(b));
    let z = (b - a).normalized();
    let x = v(face).cross(z).normalized();
    let y = z.cross(x);
    let obj = extrude!(
        name,
        outline,
        t,
        material = material,
        plane = Plane::XZ,
        bevel = bevel,
        bevel_segments = 1
    );
    let rot = M3([x.array(), y.array(), z.array()]);
    obj.set_matrix_world(M4::translation(a) * rot.to_4x4())
}

/// A board's outline `w` across and `length` along, its far end rounded (`r`, a full round by default); `both`:
/// its near end too.
pub fn end_round(w: f64, length: f64, r: Option<f64>, steps: usize, both: bool) -> Vec<(f64, f64)> {
    let r = r.filter(|&r| r != 0.0).unwrap_or(w / 2.0).min(w / 2.0);
    let mut far: Vec<(f64, f64)> = (0..=steps)
        .map(|i| {
            (
                (w / 2.0 - r) + r * cos(PI / 2.0 * i as f64 / steps as f64),
                length - r + r * sin(PI / 2.0 * i as f64 / steps as f64),
            )
        })
        .collect();
    let back: Vec<(f64, f64)> = far.iter().rev().map(|&(u, v)| (-u, v)).collect();
    far.extend(back);
    if !both {
        let mut out = vec![(-w / 2.0, 0.0), (w / 2.0, 0.0)];
        out.extend(far);
        return out;
    }
    let mut near: Vec<(f64, f64)> = far.iter().map(|&(u, v)| (-u, length - v)).collect();
    near.extend(far);
    near
}

/// Round bolt heads at game points, domed out towards the face (`upward`: up, else forward on a back slat).
pub fn bolts(prefix: &str, points: &[(f64, f64, f64)], material: Mat, upward: bool) {
    for (k, p) in points.iter().enumerate() {
        cylinder!(
            format!("{prefix}{k}"),
            0.008,
            0.005,
            at = F.at(p.0, p.1, p.2),
            material = material,
            verts = 6,
            rot = if upward {
                P3::from((0, 0, 0))
            } else {
                P3::from((90, 0, 0))
            }
        );
    }
}

/// A board seen from the side at game x, `t` thick across: its outline [(depth, height), ...] (a leg cut level at
/// the foot and the top, a bearer or a cleat with its ends chamfered).
pub fn side_slab(
    name: &str,
    x: impl Coord,
    outline: &[(f64, f64)],
    t: f64,
    material: Mat,
    bevel: f64,
) -> Obj {
    fextrude!(
        F,
        name,
        outline,
        t,
        (x, 0, 0),
        material,
        plane = Facing::Side,
        bevel = bevel,
        bevel_segments = 1
    )
}

/// A cup standing at game (x, y, z): a turned body with a foot and a lip, hollow at the top, a loop handle.
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn cup(
    name: &str,
    x: f64,
    y: f64,
    z: f64,
    r: f64,
    h: f64,
    material: Mat,
    handle: bool,
    turn: f64,
    segments: usize,
) {
    let at = F.at(x, y, z);
    let rings = [
        (0.0, 0.0),
        (r * 0.78, 0.0),
        (r * 0.82, h * 0.08),
        (r * 0.95, h * 0.5),
        (r, h),
        (r * 0.9, h),
        (r * 0.86, h * 0.55),
        (r * 0.7, h * 0.18),
        (0.0, h * 0.18),
    ];
    revolve!(
        name,
        &rings,
        at = at,
        material = material,
        segments = segments
    );
    if handle {
        let a = radians(turn);
        let (hx, hz) = (cos(a), sin(a));
        let pts = [
            (x + hx * r * 0.9, y + h * 0.78, z + hz * r * 0.9),
            (x + hx * (r + 0.022), y + h * 0.72, z + hz * (r + 0.022)),
            (x + hx * (r + 0.02), y + h * 0.35, z + hz * (r + 0.02)),
            (x + hx * r * 0.92, y + h * 0.25, z + hz * r * 0.92),
        ];
        fsweep!(
            F,
            format!("{name}_handle"),
            &pts,
            0.005,
            material,
            verts = 5,
            resolution = 3
        );
    }
}

/// A jug standing at game (x, y, z): a belly, a waist, a flared lip with a pinched spout, a handle at the back.
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn jug(name: &str, x: f64, y: f64, z: f64, r: f64, h: f64, material: Mat) {
    let b = F.at(x, y, z);
    let (bx, by, bz) = (b.x, b.y, b.z);
    let shape = [
        (0.0, 0.85),
        (0.02, 0.92),
        (0.45, 1.0),
        (0.75, 0.92),
        (0.88, 0.7),
        (0.96, 0.78),
        (1.0, 0.86),
    ];
    // the lip's top edge rounded over in two rings, where the mouth's disc starts
    let lip = [
        (1.0 + 0.003 / h, 0.86 - 0.001 / r),
        (1.0 + 0.0045 / h, 0.86 - 0.004 / r),
    ];
    let mut sections = Vec::new();
    for (t, k) in shape.into_iter().chain(lip) {
        let mut lp = ring!(r * k, r * k, z = bz + h * t, verts = 14, at = (bx, by));
        if t >= 1.0 {
            // the spout: the lip pulled out towards -x
            lp = lp
                .iter()
                .map(|p| P3 {
                    x: p.x
                        + if p.x > bx && (p.y - by).abs() < r * 0.5 {
                            0.012
                        } else {
                            0.0
                        },
                    y: p.y,
                    z: p.z,
                })
                .collect();
        }
        sections.push(lp);
    }
    loft!(name, &sections, material = material);
    let pts = [
        (x + r * 0.8, y + h * 0.9, z),
        (x + r + 0.035, y + h * 0.82, z),
        (x + r + 0.03, y + h * 0.4, z),
        (x + r * 0.9, y + h * 0.2, z),
    ];
    fsweep!(
        F,
        format!("{name}_handle"),
        &pts,
        0,
        material,
        profile = rounded_rect(0.008, 0.016, 0.004, 1),
        up = (0, 1, 0),
        resolution = 3
    );
}

/// A top `w` by `d` (corners rounded `r`), its top at `y_top`, `t` thick, its edge moulded: `edge` [(inset,
/// fraction of the thickness up), ...] from the bottom (a rounded nose over a chamfer underneath). Lofted in one.
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn slab(
    name: &str,
    w: f64,
    d: f64,
    y_top: f64,
    t: f64,
    material: Mat,
    r: f64,
    at: (f64, f64),
) -> Obj {
    let edge = [
        (0.008, 0.0),
        (0.002, 0.3),
        (0.0, 0.6),
        (0.0015, 0.88),
        (0.005, 1.0),
    ];
    let corner = 3;
    let mut sections = Vec::new();
    for (inset, f) in edge {
        let lp = rounded_rect(
            w - 2.0 * inset,
            d - 2.0 * inset,
            (r - inset).max(0.002),
            corner,
        );
        sections.push(
            lp.iter()
                .map(|&(u, v)| F.at(at.0 + u, y_top - t + f * t, at.1 + v))
                .collect::<Vec<P3>>(),
        );
    }
    artkit::bevel::eased(loft!(name, &sections, material = material, angle = 50))
}

/// A bottle standing at game (x, y, z): a body, a rounded shoulder, a neck, a lip; `cap`: its cap's material.
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn bottle(
    name: &str,
    x: f64,
    y: f64,
    z: f64,
    r: f64,
    h: f64,
    material: Mat,
    cap: Option<Mat>,
    verts: usize,
) {
    let b = F.at(x, y, z);
    let (bx, by, bz) = (b.x, b.y, b.z);
    let rings = [
        (0.0, 0.92),
        (0.03, 1.0),
        (0.6, 1.0),
        (0.72, 0.8),
        (0.8, 0.38),
        (0.94, 0.34),
        (0.97, 0.4),
        (1.0, 0.36),
    ];
    let sections: Vec<Vec<P3>> = rings
        .iter()
        .map(|&(t, k)| ring!(r * k, r * k, z = bz + h * t, verts = verts, at = (bx, by)))
        .collect();
    loft!(name, &sections, material = material);
    if let Some(cap) = cap {
        fcyl!(
            F,
            format!("{name}_cap"),
            r * 0.4,
            h * 0.05,
            (x, y + h * 1.02, z),
            cap,
            verts = verts
        );
    }
}

/// A square leg, its corners eased, tapering from `top` across at height y1 to `foot` at y0 (the floor).
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn tapered_leg(
    name: &str,
    x: f64,
    z: f64,
    y0: f64,
    y1: f64,
    foot: f64,
    top: f64,
    material: Mat,
    verts: usize,
) -> Obj {
    let b = F.at(x, 0, z);
    let (bx, by) = (b.x, b.y);
    let rings = [(y0, foot), (y0 + 0.006, foot), (y1, top)];
    let sections: Vec<Vec<P3>> = rings
        .iter()
        .map(|&(y, s)| {
            ring!(
                s / 2.0,
                s / 2.0,
                z = y,
                verts = verts,
                square = 7,
                at = (bx, by)
            )
        })
        .collect();
    loft!(name, &sections, material = material)
}

/// A mesh of points and faces, its faces each on one of `materials` (`mats`: the index of each), smooth or flat,
/// its normals pointing out.
pub fn mesh_of(
    name: &str,
    verts: Vec<V3>,
    faces: &[Vec<u32>],
    materials: &[Mat],
    mats: &[u16],
    smooth: bool,
) -> Obj {
    let mut mesh = Mesh::from_pydata(verts, faces);
    for &m in materials {
        mesh.slot(m);
    }
    for (f, &m) in mesh.faces.iter_mut().zip(mats) {
        (f.mat, f.smooth) = (m, smooth);
    }
    mesh.recalc_normals();
    artkit::scene::add(name, mesh)
}
