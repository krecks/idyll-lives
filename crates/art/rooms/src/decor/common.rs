//! What the decor is built from: the palette, the frame, flat shapes laid on pictures, frames and mounts, clock
//! hands, books, heart leaves and vines, a pedestal.

use artkit::forms::{Plane, smooth_outline};
use artkit::kit;
use artkit::materials::Mat;
use artkit::math::{Axis, PI, V3, cos, pow, radians, sin};
use artkit::mesh::Mesh;
use artkit::palette;
use artkit::parts::{hinged, join, new_parts, set_origin};
use artkit::place::{Coord, Facing, GameFrame};
use artkit::py;
use artkit::scene::{self, Obj};
use artkit::{extrude, fbox, fcyl, fextrude, fsweep, revolve, spin};

palette! { M:
    // frames and wall things
    frame_white = ("#F2F0EA", satin),
    frame_black = ("#2A2C30", satin),
    gilt = ("#B8925A", metal),
    gilt_dark = ("#8E6E3A", metal),
    oak = ("#CDB08A", wood),
    mount = ("#FBFAF6", matte),
    // the pictures: flat colour, matte
    paper = ("#F1EBDD", matte),
    sage = ("#9DAF8C", matte),
    olive = ("#8C9468", matte),
    sky = ("#A9C8DD", matte),
    rock = ("#6E7E8A", matte),
    rock_dark = ("#56656F", matte),
    snow = ("#F4F4F2", matte),
    lake = ("#6E95AE", matte),
    shore = ("#4E6B4A", matte),
    sun = ("#F6E7B0", matte),
    haze = ("#C9D3C8", matte),
    cloud = ("#EDEBE2", matte),
    hills = ("#8E9A7A", matte),
    field = ("#B59B5E", matte),
    trunk = ("#4A3A2C", matte),
    tree = ("#4E5E3A", matte),
    cottage = ("#E7E0D4", matte),
    roof = ("#8E3F2E", matte),
    canvas = ("#F4F2EE", matte),
    mustard = ("#D6A64A", matte),
    rust = ("#B5653F", matte),
    ink = ("#2A2C30", satin),
    // clocks
    clock_case = ("#C3C7CB", metal),
    face = ("#FBFAF6", satin),
    second = ("#D9483B", satin),
    walnut = ("#6B4630", satin),     // the grandfather clock's polished case (plain: its pendulum and door take draws 4 and 5)
    walnut_dark = ("#4E3222", satin),
    dial = ("#F4EBD8", gloss),
    brass = ("#B8925A", metal),
    hollow = ("#2A1C14", matte),     // the clock's trunk behind the pendulum
    // the shelf's things
    pot = ("#E8E6E1", gloss),
    leaf = ("#5E8F4E", foliage),
    leaf_light = ("#6E9A55", foliage),
    book_teal = ("#3E7A7A", satin),
    book_rust = ("#B5653F", satin),
    book_mustard = ("#D6A64A", satin),
    book_navy = ("#3E5470", satin),
    book_cream = ("#EFE5D3", satin),
    clay = ("#C98F6B", matte),
    // ornaments
    ceramic = ("#E3DDD2", satin),
    twig = ("#8E6E52", satin),
    plume = ("#D9C7A7", foliage),
    porcelain = ("#F4F4F0", gloss),
    blue = ("#2E4E8E", gloss),
    stone = ("#E7E3DC", matte),
    vase_blue = ("#3E6E9A", gloss),
    stem = ("#6E8B4E", foliage),
    bloom = ("#F4EFE6", foliage),
    marble = ("#EDEBE6", satin),
    concrete = ("#9A9893", matte),
    concrete_dark = ("#6E6C68", matte),
    bronze = ("#A47B45", metal),       // a golden cast bronze, bright on its edges (not the plinth's wood)
    bronze_dark = ("#55583E", metal),  // the mane and tail: a dark green patina
    coat = ("#3E5470", fabric),
    scarf = ("#B5653F", fabric),
    hat = ("#C9B79C", fabric),
    pot_white = ("#F2F0EA", satin),
    cord = ("#C9B79C", matte),
    // the fireplace
    surround = ("#E7E3DC", satin),
    hearth = ("#8E8A84", matte),
    firebox = ("#2A2624", matte),
    soot = ("#1B1716", matte),
    log = ("#6B4A33", satin),
    log_end = ("#B08A62", satin),     // a split log's pale cut end
    brick = ("#4A2E26", matte),       // the firebox's sooty brick lining
    grate = ("#2B2C2E", metal),
    flame = ("#FF9A3C", gloss),
    flame_core = ("#FFD27A", gloss),
    ember = ("#D9542B", gloss),
    picture = ("#3E7A7A", matte),
    candle = ("#EFE5D3", satin),
    // textiles
    linen = ("#E7E0D4", fabric),
    rail = ("#2B2C2E", metal),
    rug = ("#9DAF8C", fabric),
    rug_border = ("#7E9271", fabric),
    rug_ground = ("#B5653F", fabric),
    rug_edge = ("#3E5470", fabric),
    rug_motif = ("#E9C46A", fabric),
    rug_accent = ("#F2E6CC", fabric),
    // the wall fittings: blinds, hooks and the door's gadgets
    blind_slat = ("#EFE9DC", satin),
    blind_rail = ("#D6D0C2", satin),
    hook_board = ("#3E5C48", satin),
    gadget_white = ("#E9E9E4", satin),
    gadget_dark = ("#3A3E44", satin),
    gadget_chrome = ("#C9CDD2", chrome),
    cam_lens = ("#16202B", gloss),
    alarm_led = ("#C0392B", gloss),
    hall_glass = ("#AFC0C8", gloss),
}

pub const F: GameFrame = GameFrame::new(0.0);
pub use artkit::kit::WALL;

// --- flat shapes ------------------------------------------------------------------------------------------
/// A mesh object from Blender points and faces, its normals made outward; the first `smooth` faces shaded smooth,
/// the rest flat.
pub fn mesh(name: &str, verts: Vec<V3>, faces: &[Vec<u32>], material: Mat, smooth: usize) -> Obj {
    let mut m = Mesh::from_pydata(verts, faces);
    m.recalc_normals();
    for (i, f) in m.faces.iter_mut().enumerate() {
        f.smooth = i < smooth;
    }
    m.set_material(Some(material));
    scene::add(name, m)
}

/// A flat convex shape on a picture: `pts` (game x, height) at game depth `z`, `t` thick, facing the front.
pub fn flat(name: &str, pts: &[(f64, f64)], z: f64, material: Mat, t: f64) -> Obj {
    let n = pts.len();
    let mut verts: Vec<V3> = pts.iter().map(|&(x, y)| F.at(x, y, z).into()).collect();
    verts.extend(pts.iter().map(|&(x, y)| V3::from(F.at(x, y, z + t))));
    let mut faces: Vec<Vec<u32>> = vec![
        (0..n as u32).collect(),
        (n as u32..2 * n as u32).rev().collect(),
    ];
    for i in 0..n {
        faces.push(vec![
            i as u32,
            ((i + 1) % n) as u32,
            (n + (i + 1) % n) as u32,
            (n + i) as u32,
        ]);
    }
    mesh(name, verts, &faces, material, 0)
}

/// A flat shape of any outline (concave too) on a picture: `pts` (game x, height), its back at game depth `z`, `t`
/// thick; `passes` rounds its corners (Chaikin).
pub fn shape(name: &str, pts: &[(f64, f64)], z: f64, material: Mat, t: f64, passes: usize) -> Obj {
    let pts: Vec<(f64, f64)> = if passes > 0 {
        smooth_outline(pts, passes, true)
    } else {
        pts.to_vec()
    };
    fextrude!(F, name, &pts, t, (0, 0, z + t / 2.0), material)
}

/// Points round an ellipse (degrees from the right, anticlockwise), for `flat`.
pub fn ellipse(cx: f64, cy: f64, rx: f64, ry: f64, a0: f64, a1: f64, n: usize) -> Vec<(f64, f64)> {
    let count = n + if a1 - a0 >= 360.0 { 0 } else { 1 };
    (0..count)
        .map(|k| {
            let a = radians(a0 + (a1 - a0) * k as f64 / n as f64);
            (cx + rx * cos(a), cy + ry * sin(a))
        })
        .collect()
}

/// A whole ellipse of `n` points.
pub fn oval(cx: f64, cy: f64, rx: f64, ry: f64, n: usize) -> Vec<(f64, f64)> {
    ellipse(cx, cy, rx, ry, 0.0, 360.0, n)
}

/// A round flat disc facing the front, its back at game depth `z`.
#[allow(clippy::too_many_arguments)]
pub fn disc(
    name: &str,
    x: impl Coord,
    y: f64,
    z: f64,
    r: f64,
    material: Mat,
    t: f64,
    verts: usize,
) -> Obj {
    fcyl!(
        F,
        name,
        r,
        t,
        (x, y, z + t / 2.0),
        material,
        rot = (90, 0, 0),
        verts = verts,
        smooth = false
    )
}

/// A pointed leaf's outline from its stalk at (0, 0) to its tip at (0, length), `width` across, its tip bent `curl`
/// to the side.
pub fn leaf_outline(length: f64, width: f64, n: usize, curl: f64) -> Vec<(f64, f64)> {
    let side: Vec<(f64, f64)> = (1..n)
        .map(|k| {
            let t = k as f64 / n as f64;
            (
                width / 2.0 * pow(sin(PI * t), 0.8) + curl * t * t,
                length * t,
            )
        })
        .collect();
    let mut out = vec![(0.0, 0.0)];
    out.extend(&side);
    out.push((curl, length));
    out.extend(
        side.iter()
            .rev()
            .map(|&(u, v)| (-u + 2.0 * curl * v * v / pow(length, 2.0), v)),
    );
    out
}

/// A flat pointed leaf `w` across and `h` long on a picture, its stalk at game (x, y), turned `ang` degrees from
/// upright (anticlockwise as seen).
#[allow(clippy::too_many_arguments)]
pub fn leaf(
    name: &str,
    x: f64,
    y: f64,
    z: f64,
    w: f64,
    h: f64,
    ang: f64,
    material: Mat,
    t: f64,
) -> Obj {
    let a = radians(ang);
    let pts: Vec<(f64, f64)> = leaf_outline(h, w, 6, 0.0)
        .iter()
        .map(|&(u, v)| (x + u * cos(a) - v * sin(a), y + u * sin(a) + v * cos(a)))
        .collect();
    shape(name, &pts, z, material, t, 0)
}

// --- frames -----------------------------------------------------------------------------------------------
/// A w x h rectangle's corners round (x, yc), game (x, height), anticlockwise as seen from the front.
pub fn rect(w: f64, h: f64, yc: f64, x: f64) -> Vec<(f64, f64)> {
    vec![
        (x + w / 2.0, yc - h / 2.0),
        (x + w / 2.0, yc + h / 2.0),
        (x - w / 2.0, yc + h / 2.0),
        (x - w / 2.0, yc - h / 2.0),
    ]
}

/// A picture frame hanging from `y0`, its back on the wall: one moulding mitred round its four sides (`profile`
/// [(inset, out)], from the wall; by default a flat face with a rounded outer edge and a lip at the sight edge) and
/// a backing. Returns the opening's middle height, width and height, and the game depth of the backing's face.
#[allow(clippy::too_many_arguments)]
pub fn frame(
    w: f64,
    h: f64,
    y0: f64,
    depth: f64,
    border: f64,
    material: Mat,
    profile: Option<&[(f64, f64)]>,
) -> (f64, f64, f64, f64) {
    let bevel = 0.004;
    let yc = y0 + h / 2.0;
    let default = [
        (0.0, 0.0),
        (0.0, depth - bevel),
        (bevel, depth),
        (border - 0.008, depth),
        (border - 0.006, depth - 0.005),
        (border, depth - 0.007),
        (border, 0.0),
    ];
    let profile = artkit::forms::eased_loop(profile.unwrap_or(&default), 0.003);
    kit::moulding!(
        F,
        "frame",
        &rect(w, h, yc, 0.0),
        &profile,
        material,
        at = WALL
    );
    fbox!(
        F,
        "backing",
        w - border,
        h - border,
        0.006,
        (0, yc, WALL + 0.003),
        material
    );
    (yc, w - 2.0 * border, h - 2.0 * border, WALL + 0.006)
}

/// A white mount filling the opening, its window cut with a bevel, and the picture's paper behind it: the picture's
/// size and face.
pub fn mounted(yc: f64, iw: f64, ih: f64, z: f64, mount_w: f64, paper: Mat) -> (f64, f64, f64) {
    let (pw, ph) = (iw - 2.0 * mount_w, ih - 2.0 * mount_w);
    fbox!(
        F,
        "paper",
        pw + 0.01,
        ph + 0.01,
        0.002,
        (0, yc, z + 0.001),
        paper
    );
    kit::moulding!(
        F,
        "mount",
        &rect(iw, ih, yc, 0.0),
        &[
            (0.0, 0.0),
            (0.0, 0.004),
            (mount_w - 0.003, 0.004),
            (mount_w, 0.0005),
            (mount_w, 0.0)
        ],
        M.mount,
        at = z + 0.002
    );
    (pw, ph, z + 0.003)
}

/// A carved shell on a frame's face, its hinge at game (x, y), fanning `r` out towards `turn` degrees (game x,
/// height: 90 is up): seven ribs, scalloped.
#[allow(clippy::too_many_arguments)]
pub fn fan(
    name: &str,
    x: impl Coord,
    y: f64,
    z: f64,
    r: f64,
    turn: f64,
    material: Mat,
    t: f64,
) -> Obj {
    let mut pts = vec![(0.0, 0.0)];
    for i in 0..15 {
        let a = radians(turn - 90.0 + 180.0 * i as f64 / 14.0);
        let rr = r * (1.0 - 0.16 * (i % 2) as f64);
        pts.push((rr * cos(a), rr * sin(a)));
    }
    fextrude!(
        F,
        name,
        &pts,
        t,
        (x, y, z + t / 2.0),
        material,
        bevel = 0.002,
        bevel_segments = 1
    )
}

// --- clocks -----------------------------------------------------------------------------------------------
// The clocks' hands show the game's time (`motions.clock`, CLOCK): each hand its own part on the dial's middle, one
// turn clockwise from rest a clip, and at rest ten past ten (CLOCK_AT, minutes past twelve).
pub const CLOCK_AT: i64 = 10 * 60 + 10;

/// The pack's `clock`.
pub fn clock_json() -> String {
    format!(r#""clock": {{"hour": "hour_hand", "minute": "minute_hand", "at": {CLOCK_AT}}}"#)
}

/// A clock hand from its tail (below the arbor) to its tip (`length` up), `width` at its widest: a slim taper with
/// a round boss on the arbor, or (`spade`) a leaf swelling near the tip.
fn hand_outline(length: f64, width: f64, tail: f64, spade: bool) -> Vec<(f64, f64)> {
    let w = width / 2.0;
    let boss: Vec<(f64, f64)> = (0..8)
        .map(|k| {
            let a = radians((200 + 20 * k) as f64);
            (w * 1.3 * cos(a), w * 1.3 * sin(a))
        })
        .collect();
    let right: Vec<(f64, f64)> = if spade {
        vec![
            (w * 0.7, 0.0),
            (w * 0.55, length * 0.55),
            (w * 1.6, length * 0.7),
            (w * 1.1, length * 0.86),
            (0.0, length),
        ]
    } else {
        vec![
            (w * 0.8, 0.0),
            (w * 0.55, length * 0.75),
            (w * 0.3, length * 0.95),
            (0.0, length),
        ]
    };
    let mut out = if tail != 0.0 {
        vec![(-w * 0.6, -tail), (w * 0.6, -tail)]
    } else {
        Vec::new()
    };
    out.extend(&right);
    out.extend(right[..right.len() - 1].iter().rev().map(|&(u, v)| (-u, v)));
    out.extend(boss);
    out
}

/// A hand: (name, length, width, lift above the dial, tail).
pub type Hand = (&'static str, f64, f64, f64, f64);

/// The hour and minute hands on a dial centred at game (`x`, `y`, `z`), pointing at CLOCK_AT; returned hinged on
/// the dial's middle. (Seen from the front, the game's +x is on the left, so angles clockwise from twelve go towards
/// -x.)
pub fn clock_hands(y: f64, z: f64, hour: Hand, minute: Hand) -> [Obj; 2] {
    let x = 0.0;
    let angles = [
        py::fmod(-(CLOCK_AT as f64 / 2.0), 360.0),
        -((CLOCK_AT % 60 * 6) as f64),
    ];
    let mut out = Vec::new();
    for ((name, length, width, lift, tail), angle) in [hour, minute].into_iter().zip(angles) {
        let parts = new_parts(|| {
            fextrude!(
                F,
                name,
                &hand_outline(length, width, tail, name == "hour"),
                0.002,
                (x, y, z + lift),
                M.ink,
                rot = (0.0, -angle, 0.0)
            );
        });
        out.push(hinged(
            join(&parts, &format!("{name}_hand"), None),
            F.at(x, y, z),
        ));
    }
    [out[0], out[1]]
}

/// Each hand once round clockwise (seen from the front); the game holds them at the time.
pub fn clock_clips(hands: [Obj; 2]) {
    for hand in hands {
        let name = hand.name();
        spin!(hand, &name, turns = 1.0, seconds = 12.0, axis = Axis::Y);
    }
}

// --- books, leaves --------------------------------------------------------------------------------------
/// A book standing on a shelf top at height `y`, its spine to the front at game x `x`, `t` thick: its boards and
/// spine in `cover`, a band at the spine's head and foot, the pages' cream edge showing at the top.
#[allow(clippy::too_many_arguments)]
pub fn book(
    name: &str,
    x: f64,
    y: f64,
    z: f64,
    t: f64,
    h: f64,
    depth: f64,
    cover: Mat,
    lean: f64,
) -> Obj {
    let parts = new_parts(|| {
        fbox!(
            F,
            format!("{name}_pages"),
            t - 0.003,
            h - 0.006,
            depth - 0.006,
            (x, y + h / 2.0, z - 0.002),
            M.book_cream
        );
        for s in [-1, 1] {
            fbox!(
                F,
                format!("{name}_board{s}"),
                0.0025,
                h,
                depth,
                (x + s as f64 * (t / 2.0 - 0.00125), y + h / 2.0, z),
                cover
            );
        }
        fbox!(
            F,
            format!("{name}_spine"),
            t,
            h,
            0.004,
            (x, y + h / 2.0, z + depth / 2.0 - 0.002),
            cover,
            bevel = 0.0012,
            bevel_segments = 1
        );
        for s in [-1, 1] {
            fbox!(
                F,
                format!("{name}_band{s}"),
                t + 0.0004,
                0.004,
                0.001,
                (
                    x,
                    y + h / 2.0 + s as f64 * (h / 2.0 - 0.018),
                    z + depth / 2.0 + 0.0002
                ),
                M.gilt
            );
        }
    });
    let b = join(&parts, name, None);
    if lean != 0.0 {
        let pivot = F.at(x + t / 2.0, y, z);
        set_origin(b, pivot);
        b.set_rotation_euler_axis(1, radians(lean));
    }
    b
}

/// A row of books standing on a shelf top at `y`, from `x0` to `x1`; the last one leaning on them.
#[allow(clippy::too_many_arguments)]
pub fn books(x0: f64, x1: f64, y: f64, z: f64, depth: f64, seed: usize) {
    let colours = [
        M.book_teal,
        M.book_rust,
        M.book_mustard,
        M.book_navy,
        M.book_cream,
    ];
    let (mut x, mut k) = (x0, 0);
    while x < x1 - 0.06 {
        let t = 0.025 + 0.012 * ((k * 7 + seed) % 3) as f64;
        let h = 0.17 + 0.025 * ((k * 5 + seed) % 4) as f64;
        book(
            &format!("book{seed}_{k}"),
            x + t / 2.0,
            y,
            z,
            t,
            h,
            depth - 0.01 * (k % 2) as f64,
            colours[(k + seed) % 5],
            0.0,
        );
        (x, k) = (x + t + 0.002, k + 1);
    }
    book(
        &format!("book{seed}_lean"),
        x + 0.085,
        y,
        z,
        0.03,
        0.19,
        depth,
        colours[(k + seed) % 5],
        22.0,
    );
}

/// A heart-shaped leaf `size` long, its stalk at game point `at`, turned `turn` degrees about the up axis and tipped
/// `tilt` degrees down from upright (a trailing plant's leaf).
pub fn heart(
    name: &str,
    at: (f64, f64, f64),
    size: f64,
    turn: i64,
    tilt: i64,
    material: Mat,
) -> Obj {
    let mut pts = vec![(0.0, 0.0)];
    pts.extend((1..8).map(|k| {
        let a = PI * k as f64 / 8.0;
        (
            size * 0.5 * sin(a) * (1.0 - 0.3 * cos(a)),
            size * (0.45 - 0.45 * cos(a)) + size * 0.1 * pow(sin(a), 2.0),
        )
    }));
    pts.push((0.0, size));
    let mirror: Vec<(f64, f64)> = pts[1..pts.len() - 1]
        .iter()
        .rev()
        .map(|&(u, v)| (-u, v))
        .collect();
    pts.extend(mirror);
    let pts: Vec<(f64, f64)> = pts
        .iter()
        .map(|&(u, v)| (u, v - size * 0.08 * sin(PI * v / size)))
        .collect();
    extrude!(
        name,
        &pts,
        0.003,
        at = F.at(at.0, at.1, at.2),
        material = material,
        plane = Plane::XZ,
        rot = (tilt as i32, 0, turn as i32)
    )
}

/// A trailing vine through game points, a heart leaf at every few of its points.
pub fn vine(name: &str, pts: &[(f64, f64, f64)], material: Mat, leaves: usize, seed: usize) {
    fsweep!(F, name, pts, 0.0028, material, verts = 4, resolution = 2);
    for (i, &p) in pts.iter().enumerate().skip(1) {
        if i % leaves == 0 || i == pts.len() - 1 {
            heart(
                &format!("{name}_leaf{i}"),
                p,
                0.034 - 0.004 * (i % 3) as f64,
                40 * ((i * 7 + seed) % 9) as i64 - 160,
                75 + 5 * (i % 3) as i64,
                if (i + seed) % 2 == 1 {
                    M.leaf
                } else {
                    M.leaf_light
                },
            );
        }
    }
}

/// A w x d rectangle's corners round game (x, depth z), for a level moulding (`Facing::Top`).
pub fn box_outline(w: f64, d: f64, z: f64) -> Vec<(f64, f64)> {
    let x = 0.0;
    vec![
        (x - w / 2.0, z - d / 2.0),
        (x + w / 2.0, z - d / 2.0),
        (x + w / 2.0, z + d / 2.0),
        (x - w / 2.0, z + d / 2.0),
    ]
}

/// A turned part, `profile` [(radius, up)] from its foot at game point `at` (`flip`: hanging down from it).
pub fn turned_g(
    name: &str,
    profile: &[(f64, f64)],
    at: (f64, f64, f64),
    material: Mat,
    segments: usize,
    flip: bool,
) -> Obj {
    let prof: Vec<(f64, f64)> = profile
        .iter()
        .map(|&(r, z)| (r, if flip { -z } else { z }))
        .collect();
    revolve!(
        name,
        &prof,
        at = F.at(at.0, at.1, at.2),
        material = material,
        segments = segments
    )
}

/// A square pedestal `h` tall and `w` wide on the footprint's middle: a moulded foot and cap a little wider, the
/// shaft between them with a raised fillet framing a sunk panel on each face.
pub fn pedestal(h: f64, w: f64, material: Mat) {
    let m = w / 2.0 + 0.02;
    kit::moulding!(
        F,
        "plinth_foot",
        &box_outline(2.0 * m, 2.0 * m, 0.0),
        &[
            (0.0, 0.0),
            (0.0, 0.022),
            (0.004, 0.028),
            (0.01, 0.034),
            (0.02, 0.04),
            (0.03, 0.045),
            (0.04, 0.045),
            (0.04, 0.0)
        ],
        material,
        at = 0.0,
        plane = Facing::Top
    );
    fbox!(
        F,
        "plinth",
        w,
        h - 0.07,
        w,
        (0, 0.04 + (h - 0.07) / 2.0, 0),
        material,
        bevel = 0.004,
        bevel_segments = 1
    );
    kit::moulding!(
        F,
        "plinth_cap",
        &box_outline(2.0 * m, 2.0 * m, 0.0),
        &artkit::forms::eased_loop(
            &[
                (0.04, 0.0),
                (0.02, 0.0),
                (0.012, 0.005),
                (0.006, 0.013),
                (0.0, 0.018),
                (0.0, 0.03),
                (0.04, 0.03)
            ],
            0.003
        ),
        material,
        at = h - 0.03,
        plane = Facing::Top
    );
    fbox!(
        F,
        "plinth_lid",
        2.0 * m - 0.07,
        0.004,
        2.0 * m - 0.07,
        (0, h - 0.002, 0),
        material
    );
    let (fh, fw) = (h - 0.2, w - 0.08);
    let fr_ = artkit::forms::rounded_rect(fw, fh, 0.004, 1);
    let hole = artkit::forms::rounded_rect(fw - 0.024, fh - 0.024, 0.003, 1);
    let y = 0.04 + (h - 0.07) / 2.0;
    for s in [-1, 1] {
        let sf = s as f64;
        fextrude!(
            F,
            format!("fillet_z{s}"),
            &fr_,
            0.008,
            (0, y, sf * (w / 2.0 + 0.002)),
            material,
            holes = [hole.clone()]
        );
        fextrude!(
            F,
            format!("fillet_x{s}"),
            &fr_,
            0.008,
            (sf * (w / 2.0 + 0.002), y, 0),
            material,
            holes = [hole.clone()],
            plane = Facing::Side
        );
    }
}
