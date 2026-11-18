//! What the couch potato pack is built from: its palette, meshes from points, pleated curtains, speaker drivers,
//! screens, snacks, and the kit's parts it uses.

use artkit::forms::Sweep;
use artkit::materials::Mat;
use artkit::math::{P3, PI, V3, atan2, cosd, pow, sind, tan};
use artkit::mesh::Mesh;
use artkit::parts::turn;
use artkit::place::{Facing, GameFrame, Spot};
use artkit::py::fmod;
use artkit::scene::{self, Obj};
use artkit::{fbox, fextrude, fsphere, palette, revolve};
use std::cell::Cell;

use artkit::forms::rounded_rect;
use artkit::math::Axis;

palette! { M:
    // the cinema wall
    leather = ("#7A2E2E", satin),        // the twin recliners; plain, so the curtains take the textured draws
    cinema_frame = ("#5C6168", satin),   // the console, posts and speakers; its main part
    curtain = ("#8E2A3A", fabric),
    pelmet = ("#3A3D41", satin),
    // screens
    screen_back = ("#CDB08A", satin),    // the oak acoustic backs, plain
    slat = ("#B08E66", satin),
    bezel = ("#1B2A3A", gloss),
    picture = ("#3BA3E0", gloss, glow("#3BA3E0"), screen()),
    picture_b = ("#D6A64A", gloss, glow("#D6A64A"), screen()),
    graphite = ("#5C6168", metal),
    speaker_grey = ("#6E7277", satin),
    speaker_cone = ("#3A3D41", satin),
    soda = ("#C9483B", gloss),
    straw = ("#F2F2F0", satin),
    popcorn = ("#F6E7B8", matte),
    dark = ("#2B2C2E", satin),
    // the game console
    gamer = ("#3F6E9E", fabric),          // the gaming loveseat; its main part
    mustard = ("#D6A64A", satin),
    console = ("#E8E8E6", gloss),
    disc = ("#C9CCD0", chrome),
    led = ("#3BA3E0", gloss),
    box_red = ("#B33A32", satin),
    box_blue = ("#3E8EDE", satin),
    // the recliner
    plush = ("#C08A52", satin),           // the recliner's microsuede; its main part
    plinth = ("#6E5038", satin),
    teal = ("#4E7A7F", gloss),
    throw = ("#D9C7A6", satin),
    // the snack fridge
    mint = ("#9FD3C7", gloss),            // the fridge; its main part
    mint_trim = ("#7FB5A9", satin),
    lid = ("#E9E4DA", fabric),            // the padded lid
    inside = ("#F2F4F4", satin),
    chrome = ("#D0D3D6", chrome),
    menu = ("#F2A541", gloss),
    can_a = ("#C9483B", gloss),
    can_b = ("#3E8EDE", gloss),
    can_c = ("#9DAF8C", gloss),
    takeout = ("#F2EDE2", satin),
    bowl = ("#F2EDE2", gloss),
    crisps = ("#E9B54A", matte),
    // the workout game
    fit_stand = ("#E8E8E6", satin),       // the TV stand; its main part
    oak = ("#CDB08A", satin),
    fit_mat = ("#7FC4A4", fabric),
    mat_inner = ("#9FD8BB", fabric),
    towel = ("#E9B54A", fabric),
    tv_frame = ("#D5D6D4", satin),
    tv_screen = ("#4FC3A1", gloss, glow("#4FC3A1"), screen()),
    tv_floor = ("#3FA88A", gloss),
    tv_white = ("#F4F1EA", gloss),
    tv_orange = ("#F2A541", gloss),
    board = ("#3E8EDE", satin),
    grip = ("#2B2C2E", matte),
    bottle = ("#3E8EDE", gloss),
    weight = ("#C9483B", satin),
    // the lava lamp
    pouf = ("#5A4A7A", satin),            // the pouf; its main part (plain: the lamp's glass takes the extra draw)
    lamp_metal = ("#4E7A7F", metal),
    vessel = ("#E96A9A", glass, glow("#FF6FA8")),   // the lit glass: it and the wax glow at night (the lamp's `light`)
    wax = ("#F2914A", gloss, glow("#FFA060")),
    table_white = ("#E8E8E6", satin),
    rug = ("#E9B54A", satin),
}

thread_local! {
    static N: Cell<u32> = const { Cell::new(0) };
}

/// `stem` numbered: each part its own name.
pub fn name(stem: &str) -> String {
    N.with(|n| {
        n.set(n.get() + 1);
        format!("{stem}{}", n.get())
    })
}

// --- meshes from points (as pack_cheerful) --------------------------------------------------------------------

/// A mesh object from Blender-space `verts` and `faces`, normals made outward, smooth below `sharp` degrees.
pub fn obj(name: &str, verts: &[P3], faces: &[Vec<u32>], mats: &[Mat], sharp: f64) -> Obj {
    let mut mesh = Mesh::from_pydata(verts.iter().map(|&p| V3::from(p)).collect(), faces);
    for &m in mats {
        mesh.slot(m);
    }
    mesh.recalc_normals();
    let o = scene::add(name, mesh);
    o.edit(|p| p.mesh.shade_smooth(sharp));
    o
}

/// A polygon's middle, as Blender gives it: the mean of its corners.
fn centre(mesh: &Mesh, f: usize) -> V3 {
    let c = mesh.corners(f);
    let mut s = V3::ZERO;
    for &v in c {
        s += mesh.verts[v as usize];
    }
    let n = c.len() as f32;
    V3 {
        x: s.x / n,
        y: s.y / n,
        z: s.z / n,
    }
}

/// Gives the faces of `obj` whose middle (its own axes, Blender) passes `test` another material: a band, a stripe,
/// a surround, with no more triangles.
pub fn paint(obj: Obj, material: Mat, test: impl Fn(V3) -> bool) -> Obj {
    obj.edit(|p| {
        let k = p.mesh.slot(material);
        for f in 0..p.mesh.faces.len() {
            if test(centre(&p.mesh, f)) {
                p.mesh.faces[f].mat = k;
            }
        }
    });
    obj
}

/// A pleated curtain across `x0`..`x1` hanging in the plane at depth `z` from its heading at `y1` to its hem at
/// `y0`: its folds shallow under the heading tape, deepening as it falls; a turned-up hem thickens its foot.
#[allow(clippy::too_many_arguments)]
pub fn pleats(
    f: &GameFrame,
    name: &str,
    x0: f64,
    x1: f64,
    y0: f64,
    y1: f64,
    z: f64,
    folds: i32,
    amp: f64,
    mat: Mat,
    t: f64,
) -> Obj {
    let n = folds * 4;
    let h = y1 - y0;
    let rows = [
        (y0, 1.0, 1.5, 0.0),
        (y0 + 0.012, 1.0, 1.5, 0.02),
        (y0 + 0.04, 1.0, 1.0, 0.05),
        (y0 + h * 0.45, 0.9, 1.0, 0.25),
        (y1 - 0.06, 0.7, 1.0, 0.4),
        (y1 - 0.05, 0.4, 1.3, 0.42),
        (y1, 0.36, 1.3, 0.42),
    ];
    let (mut verts, mut m) = (Vec::new(), 0);
    for &(y, af, tf, phase) in &rows {
        let front: Vec<(f64, f64)> = (0..=n)
            .map(|k| {
                (
                    x0 + (x1 - x0) * k as f64 / n as f64,
                    amp * af
                        * artkit::math::sin(2.0 * PI * folds as f64 * k as f64 / n as f64 + phase)
                        + t * tf / 2.0,
                )
            })
            .collect();
        let mut ring = front.clone();
        ring.extend(front.iter().rev().map(|&(u, o)| (u, o - t * tf)));
        m = ring.len();
        verts.extend(ring.iter().map(|&(u, o)| f.at(u, y, z + o)));
    }
    let m = m as u32;
    let mut faces: Vec<Vec<u32>> = Vec::new();
    for r in 0..rows.len() as u32 - 1 {
        let (a, b) = (r * m, (r + 1) * m);
        faces.extend((0..m).map(|i| vec![a + i, a + (i + 1) % m, b + (i + 1) % m, b + i]));
    }
    let n = n as u32;
    for base in [0, (rows.len() as u32 - 1) * m] {
        faces.extend(
            (0..n).map(|k| vec![base + k, base + k + 1, base + m - 2 - k, base + m - 1 - k]),
        );
    }
    obj(name, &verts, &faces, &[mat], 70.0)
}

/// Turns a model built in place half a turn about the footprint's middle (each part about its own origin,
/// which moves round with it), as pack_bookworm turned the writer's bureau. A part's slides then go the other way.
pub fn turn_round(objs: &[Obj]) {
    for &o in objs {
        turn(o, 180.0, Axis::Z);
        o.edit(|p| {
            p.loc = V3 {
                x: -p.loc.x,
                y: -p.loc.y,
                z: p.loc.z,
            }
        });
    }
}

/// A speaker driver at (x, y) on a face at depth `z`, facing +z (`facing` 1) or -z: its flange, a rolled
/// surround and the cone dished in to a domed dust cap, one turned piece; the flange and surround in the dark.
#[allow(clippy::too_many_arguments)]
pub fn driver(
    f: &GameFrame,
    name: &str,
    x: f64,
    y: f64,
    z: f64,
    r: f64,
    facing: i32,
    segments: usize,
) -> Obj {
    let prof = [
        (r + 0.012, -0.008),
        (r + 0.012, 0.002),
        (r + 0.004, 0.004),
        (r - 0.002, 0.009),
        (r - 0.008, 0.004),
        (r * 0.36, -0.02),
        (r * 0.3, -0.016),
        (r * 0.15, -0.012),
        (0.0, -0.011),
    ];
    revolve!(
        name,
        &prof,
        at = f.at(x, y, z),
        material = M.speaker_cone,
        segments = segments,
        rot = (90 * facing, 0, 0)
    )
}

/// A screen held between two slim posts at depth `z`, its picture facing the seats behind it (-Z), the oak back
/// with slats on its other side (to the wall, once the model is turned round).
pub fn screen(f: &GameFrame, x0: f64, x1: f64, y0: f64, y1: f64, z: f64, stripe: Mat) {
    let (w, h) = (x1 - x0, y1 - y0);
    let (cx, cy) = ((x0 + x1) / 2.0, (y0 + y1) / 2.0);
    fextrude!(
        f,
        name("screen_back"),
        &rounded_rect(w, h, 0.012, 1),
        0.05,
        (cx, cy, z),
        M.screen_back,
        bevel = 0.008,
        bevel_segments = 1
    );
    fextrude!(
        f,
        name("bezel"),
        &rounded_rect(w - 0.03, h - 0.03, 0.01, 1),
        0.008,
        (cx, cy, z - 0.028),
        M.bezel,
        bevel = 0.002,
        bevel_segments = 1
    );
    fbox!(
        f,
        name("picture"),
        w * 0.82,
        h * 0.7,
        0.002,
        (cx, cy + h * 0.04, z - 0.0325),
        M.picture
    );
    fbox!(
        f,
        name("picture_b"),
        w * 0.3,
        h * 0.12,
        0.002,
        (cx - w * 0.2, cy - h * 0.28, z - 0.034),
        M.picture_b
    );
    fbox!(
        f,
        name("stripe"),
        w * 0.5,
        0.02,
        0.006,
        (cx, y0 + 0.06, z + 0.027),
        stripe
    );
    let n = 3.max(artkit::py::int(w / 0.06));
    for i in 1..n {
        fbox!(
            f,
            name("slat"),
            0.016,
            h - 0.05,
            0.006,
            (x0 + w * i as f64 / n as f64, cy, z + 0.026),
            M.slat
        );
    }
    for x in [x0 + 0.02, x1 - 0.02] {
        fbox!(
            f,
            name("post"),
            0.04,
            y1 + 0.02,
            0.04,
            (x, (y1 + 0.02) / 2.0, z + 0.01),
            M.graphite,
            bevel = 0.008,
            bevel_segments = 1
        );
        fextrude!(
            f,
            name("post_foot"),
            &rounded_rect(0.08, 0.12, 0.02, 2),
            0.02,
            (x, 0.01, z - 0.04),
            M.graphite,
            plane = Facing::Top,
            bevel = 0.005,
            bevel_segments = 1
        );
    }
}

/// A striped popcorn bucket standing on `y`, popped corn heaped over its rim.
#[allow(clippy::too_many_arguments)]
pub fn popcorn(f: &GameFrame, x: f64, y: f64, z: f64, r: f64, h: f64, stripe: Mat, n: i32) {
    let b = revolve!(
        name("bucket"),
        &[
            (0.0, 0.0),
            (r * 0.75, 0.0),
            (r, h),
            (r * 0.94, h),
            (r * 0.72, 0.006),
            (0.0, 0.006)
        ],
        at = f.at(x, y, z),
        material = M.straw,
        segments = 16
    );
    paint(b, stripe, |c| {
        fmod(atan2(c.y as f64, c.x as f64), PI / 4.0) < PI / 8.0
    });
    for k in 0..n {
        let a = (360 * k) as f64 / n as f64;
        let rr = if k % 2 != 0 { r * 0.55 } else { r * 0.3 };
        fsphere!(
            f,
            name("popcorn"),
            r * 0.32,
            (
                x + rr * cosd(a),
                y + h + 0.006 * (k % 3) as f64,
                z + rr * sind(a)
            ),
            M.popcorn,
            segments = 6,
            rings = 4
        );
    }
    fsphere!(
        f,
        name("popcorn"),
        r * 0.36,
        (x, y + h + 0.02, z),
        M.popcorn,
        segments = 6,
        rings = 4
    );
}

/// A soda cup standing on `y`: a tapered body, its domed lid and a straw through it.
pub fn cup(f: &GameFrame, x: f64, y: f64, z: f64, r: f64, h: f64, material: Mat) {
    revolve!(
        name("soda"),
        &[(0.0, 0.0), (r * 0.78, 0.0), (r, h), (0.0, h)],
        at = f.at(x, y, z),
        material = material,
        segments = 12
    );
    revolve!(
        name("soda_lid"),
        &[
            (0.0, 0.0),
            (r * 1.06, 0.0),
            (r * 1.06, 0.006),
            (r * 0.8, 0.014),
            (0.0, 0.016)
        ],
        at = f.at(x, y + h, z),
        material = M.straw,
        segments = 12
    );
    tube(
        f,
        &name("straw"),
        &[(x + 0.006, y + h, z), (x + 0.012, y + h + 0.1, z - 0.006)],
        0.0045,
        M.straw,
        Some(6),
        1,
    );
}

// --- the kit's parts ------------------------------------------------------------------------------------------

// KIT: kit.tube
/// A round tube of radius `r` through game points, its bends rounded (`round` steps between each pair of
/// points; 1 keeps them sharp), its ends capped. `verts` round: by default 8, and more for one over 13 mm in
/// radius, as many as keep its outline within a millimetre of the circle (`forms::sweep`).
pub fn tube<P: Spot>(
    f: &GameFrame,
    name: &str,
    points: &[P],
    r: f64,
    material: Mat,
    verts: Option<usize>,
    round: usize,
) -> Obj {
    f.sweep(
        name,
        points,
        r,
        material,
        Sweep {
            verts,
            resolution: round,
            least: 8,
            ..Sweep::default()
        },
    )
}

// KIT: kit.fillet
/// The path through `points` (any axes) with each inner corner rounded to about `radius` (`steps` points round
/// it, on a curve that never overshoots, however uneven the spacing: Catmull-Rom loops on a short leg beside a long
/// one). For handles, bent rails and draped cloth swept with `resolution` 1.
// (3.1415 is the script's own limit for a straight corner, not pi.)
#[allow(clippy::approx_constant)]
pub fn fillet(points: &[(f64, f64, f64)], radius: f64, steps: usize) -> Vec<(f64, f64, f64)> {
    let pts: Vec<V3> = points.iter().map(|&p| V3::from(p)).collect();
    let mut out = vec![pts[0]];
    for w in pts.windows(3) {
        let (a, p, c) = (w[0], w[1], w[2]);
        let (d1, d2) = (a - p, c - p);
        if d1.length() < 1e-9 || d2.length() < 1e-9 || d1.angle(d2) > 3.1415 {
            out.push(p);
            continue;
        }
        let t = (radius / 1e-6_f64.max(tan(d1.angle(d2) / 2.0)))
            .min(d1.length() * 0.5)
            .min(d2.length() * 0.5);
        let (p1, p2) = (p + d1.normalized() * t, p + d2.normalized() * t);
        for i in 0..=steps {
            let k = i as f64 / steps as f64;
            out.push(p1 * pow(1.0 - k, 2.0) + p * 2 * k * (1.0 - k) + p2 * k * k);
        }
    }
    out.push(pts[pts.len() - 1]);
    out.iter()
        .map(|p| (p.x as f64, p.y as f64, p.z as f64))
        .collect()
}

// KIT: kit.pull
/// A bar pull in one piece: out from a front at game depth `face` on two round stand-offs `length` apart
/// (across it, or `upright`), its bends rounded: a drawer's or a door's handle.
#[allow(clippy::too_many_arguments)]
pub fn pull(
    f: &GameFrame,
    name: &str,
    x: f64,
    y: f64,
    face: f64,
    length: f64,
    material: Mat,
    r: f64,
    out: f64,
    upright: bool,
    verts: usize,
) -> Obj {
    let (h, b) = (length / 2.0, 0.012_f64.min(out * 0.45));
    let ends = [
        (-h, face - 0.003),
        (-h, face + out),
        (h, face + out),
        (h, face - 0.003),
    ];
    let pts: Vec<(f64, f64, f64)> = ends
        .iter()
        .map(|&(u, z)| {
            if upright {
                (x, y + u, z)
            } else {
                (x + u, y, z)
            }
        })
        .collect();
    f.sweep(
        name,
        &fillet(&pts, b, 3),
        r,
        material,
        Sweep {
            verts: Some(verts),
            ..Sweep::default()
        },
    )
}
