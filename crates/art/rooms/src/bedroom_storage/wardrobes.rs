//! BR3: the wardrobes and the armoire.

use super::common::{DOOR_SECONDS, F, M, Style, WALL, folded, front, hanging, shell};
use super::large::storage_box;
use artkit::forms::rounded_rect;
use artkit::json::float_repr;
use artkit::materials::Mat;
use artkit::math::P3;
use artkit::motion::Ease;
use artkit::parts::{hinged, join, new_parts};
use artkit::place::N;
use artkit::scene::Obj;
use artkit::{fbox, fcyl, fextrude, fsweep, loft, revolve, soft, swing};
use artkit::{kit, settle};

// --- the wardrobe: a white carcass, two birch doors with brass bars, a rail of shirts and shelves --------
/// A wardrobe's inside: a rail of clothes on hangers (the left half when `split`, with shelves of folded clothes
/// on the right) under a shelf.
#[allow(clippy::too_many_arguments)]
fn wardrobe_inside(w: f64, y0: f64, y1: f64, back: f64, front_z: f64, rail_y: f64, split: bool) {
    let (d, zc) = (front_z - back, (back + front_z) / 2.0);
    let hang_x1 = if split { -0.02 } else { w / 2.0 - 0.08 };
    let width = if split { w / 2.0 } else { w };
    let x = if split { N::F(-w / 4.0) } else { N::I(0) };
    fbox!(
        F,
        "hat_shelf",
        width - 0.04,
        0.02,
        d - 0.04,
        (x, rail_y + 0.12, zc - 0.01),
        M.white
    );
    fcyl!(
        F,
        "rail",
        0.012,
        width - 0.06,
        (x, rail_y, zc),
        M.chrome,
        verts = 10,
        rot = (0, 90, 0)
    );
    let n = if split { 5 } else { 3.max((w / 0.16) as usize) };
    hanging(
        "shirt",
        -w / 2.0 + 0.1,
        hang_x1 - 0.08,
        rail_y,
        zc,
        n,
        0.42_f64.min(d - 0.1),
        0.78,
    );
    // a box on the hat shelf, under the carcass's top and inside its side (it came through both)
    let bw = 0.3_f64.min(w / 2.0 - 0.06);
    storage_box(
        "folded_box",
        bw,
        0.13,
        0.3,
        (-w / 4.0, rail_y + 0.13, zc),
        M.box_,
    );
    if split {
        fbox!(
            F,
            "divider",
            0.02,
            y1 - y0 - 0.04,
            d - 0.02,
            (0, (y0 + y1) / 2.0, zc - 0.01),
            M.white
        );
        for (i, y) in [y0 + 0.45, y0 + 0.9, y0 + 1.35].into_iter().enumerate() {
            fbox!(
                F,
                format!("shelf{i}"),
                w / 2.0 - 0.04,
                0.02,
                d - 0.04,
                (w / 4.0, y, zc - 0.01),
                M.white
            );
            folded(
                &format!("stack{i}_"),
                w / 4.0 - 0.18,
                y + 0.01,
                zc,
                0.3,
                0.32,
                3,
                0.035,
            );
            folded(
                &format!("stackb{i}_"),
                w / 4.0 + 0.18,
                y + 0.01,
                zc,
                0.3,
                0.32,
                2 + i % 2,
                0.035,
            );
        }
    }
}

/// The knuckles of a door's hinges on its hinge line (x, z), on the still carcass.
fn hinges(name: &str, x: f64, ys: &[f64], z: f64) {
    for &y in ys {
        fcyl!(
            F,
            format!("{name}{}", float_repr(y)),
            0.006,
            0.06,
            (x, y, z),
            M.chrome,
            verts = 8
        );
    }
}

/// The wardrobe.
pub fn wardrobe() -> Vec<Obj> {
    let (back, front_z) = (WALL, 0.15);
    let still = new_parts(|| {
        fbox!(
            F,
            "plinth",
            1.80,
            0.06,
            0.54,
            (0, 0.03, (back + front_z) / 2.0 - 0.01),
            M.plinth,
            bevel = 0.004,
            bevel_segments = 1
        );
        shell(1.84, 0.06, 2.11, back, front_z, M.white, 0.025, "case");
        wardrobe_inside(1.84, 0.06, 2.11, back, front_z, 1.8, true);
        storage_box("box_big", 0.4, 0.3, 0.32, (-0.5, 2.11, -0.18), M.box_);
        storage_box("box_small", 0.32, 0.22, 0.3, (-0.08, 2.11, -0.18), M.box2);
        for s in [-1, 1] {
            hinges(
                &format!("hinge{s}_"),
                s as f64 * 0.913,
                &[0.3, 1.085, 1.86],
                front_z + 0.01,
            );
        }
    });
    join(&still, "wardrobe", Some(P3::ZERO));
    let mut doors = Vec::new();
    for s in [-1, 1] {
        let sf = s as f64;
        let parts = new_parts(|| {
            front(
                &format!("door{s}"),
                0.905,
                2.04,
                sf * 0.4575,
                1.085,
                front_z + 0.01,
                M.birch,
                Style::Shaker,
                Some(0.09),
            );
            kit::pull!(
                F,
                &format!("handle{s}"),
                sf * 0.06,
                1.1,
                front_z + 0.02,
                0.36,
                M.brass,
                r = 0.008,
                out = 0.018,
                upright = true
            );
        });
        let name = if s < 0 { "door_left" } else { "door_right" };
        doors.push(hinged(
            join(&parts, name, None),
            F.at(sf * 0.91, 1.085, front_z + 0.01),
        ));
    }
    let objs = settle!(centre = false);
    swing!(
        doors[0],
        "doors_open",
        100.0,
        seconds = DOOR_SECONDS,
        ease = Ease::BackOut
    );
    swing!(
        doors[1],
        "doors_open",
        -100.0,
        seconds = DOOR_SECONDS,
        ease = Ease::BackOut
    );
    objs
}

// --- the single wardrobe: white, one door hinged on the left with a brass bar, a rail of shirts ----------
// 0.60 m wide, its door 0.58 (it was 0.90, with a door as wide as a room's: M-bedroom-03).
const WS_W: f64 = 0.6;

/// The single wardrobe.
pub fn wardrobe_single() -> Vec<Obj> {
    let (back, front_z) = (WALL, 0.14);
    let w = WS_W;
    let still = new_parts(|| {
        fbox!(
            F,
            "plinth",
            w - 0.04,
            0.05,
            0.53,
            (0, 0.025, (back + front_z) / 2.0 - 0.01),
            M.plinth,
            bevel = 0.004,
            bevel_segments = 1
        );
        shell(w, 0.05, 2.04, back, front_z, M.white, 0.022, "case");
        wardrobe_inside(w, 0.05, 2.04, back, front_z, 1.74, false);
        storage_box("box_top", 0.3, 0.24, 0.3, (-0.1, 2.04, -0.2), M.box_);
        hinges(
            "hinge",
            -(w - 0.02) / 2.0,
            &[0.3, 1.045, 1.79],
            front_z + 0.01,
        );
    });
    join(&still, "wardrobeSingle", Some(P3::ZERO));
    let parts = new_parts(|| {
        front(
            "door",
            w - 0.02,
            1.97,
            0,
            1.045,
            front_z + 0.01,
            M.white,
            Style::Shaker,
            Some(0.08),
        );
        kit::pull!(
            F,
            "handle",
            w / 2.0 - 0.06,
            1.1,
            front_z + 0.02,
            0.44,
            M.brass,
            r = 0.008,
            out = 0.018,
            upright = true
        );
    });
    let door = hinged(
        join(&parts, "door", None),
        F.at(-(w - 0.02) / 2.0, 1.045, front_z + 0.01),
    );
    let objs = settle!(centre = false);
    swing!(
        door,
        "door_open",
        100.0,
        seconds = DOOR_SECONDS,
        ease = Ease::BackOut
    );
    objs
}

// --- the armoire: mahogany, a cornice, raised-panel doors with brass knobs, on bun feet -------------------
/// A crown moulding round the front and sides at height `y` (its bottom), its path `x` either side and at game depth
/// `front`: a cove rising and swelling out `out` to a flat top, its top's corners eased and its ends (on the wall's
/// face) chamfered. Its back edge reaches only 1 cm in from the path, inside the path's 1.2 cm corners: reaching
/// farther, it folded back on itself round each front corner (faces turned inside out).
#[allow(clippy::too_many_arguments)]
fn crown(name: &str, x: f64, front_z: f64, back: f64, y: f64, material: Mat, h: f64, out: f64) {
    // the cove as drawn for the 6.9 cm crown that reaches 5 cm, scaled to this one
    let (ku, kv) = (h / 0.069, out / 0.05);
    let mut prof: Vec<(f64, f64)> = [
        (0.0, 0.0),
        (0.0, -0.006),
        (0.012, -0.012),
        (0.024, -0.02),
        (0.036, -0.032),
        (0.046, -0.044),
        (0.054, -0.05),
    ]
    .iter()
    .map(|&(u, v)| (u * ku, v * kv))
    .collect();
    prof.extend([(h, -out), (h, 0.01), (0.0, 0.01)]);
    // swept about its middle (`c`), so the rings at its ends can shrink into it: a chamfer
    let c = (h / 2.0, (0.01 - out) / 2.0);
    let centred: Vec<(f64, f64)> = prof.iter().map(|&(u, v)| (v - c.1, u - c.0)).collect();
    // (eased with height as its v: what it leaves sharp is its bottom, on the cornice)
    let prof: Vec<(f64, f64)> = artkit::forms::eased(&centred, 0.004)
        .iter()
        .map(|&(v, u)| (u, v))
        .collect();
    let (x, front_z, y) = (x - c.1, front_z - c.1, y + c.0); // the path through its middle, out by -c.1
    let (z0, z1) = (back, back + 0.004);
    let mut path = vec![(-x, y, z0)];
    path.extend(kit::fillet(
        &[(-x, y, z1), (-x, y, front_z), (x, y, front_z), (x, y, z1)],
        0.012 - c.1,
        2,
    ));
    path.push((x, y, z0));
    let end = 1.0 - 0.004 / (h / 2.0).min((0.01 + out) / 2.0); // each end's ring 4 mm in all round
    fsweep!(
        F,
        name,
        &path,
        0,
        material,
        profile = prof,
        up = (0, 0, 1),
        angle = 30,
        scale = artkit::forms::by(move |w| if w <= 0.0 || w >= 1.0 { end } else { 1.0 })
    );
}

/// The armoire.
pub fn armoire() -> Vec<Obj> {
    let (back, front_z) = (WALL, 0.115);
    let still = new_parts(|| {
        for sx in [-1, 1] {
            for z in [-0.38, 0.08] {
                revolve!(
                    format!("foot{sx}{}", float_repr(z)),
                    &[
                        (0.0, 0.0),
                        (0.028, 0.0),
                        (0.044, 0.018),
                        (0.048, 0.045),
                        (0.04, 0.07),
                        (0.026, 0.085),
                        (0.026, 0.12),
                        (0.0, 0.12)
                    ],
                    at = F.at(sx as f64 * 0.8, 0, z),
                    material = M.mahogany2,
                    segments = 14
                );
            }
        }
        soft!(
            F,
            "base",
            1.74,
            0.1,
            0.59,
            (0, 0.17, (back + 0.16) / 2.0),
            M.mahogany2,
            round = 0.012,
            round_segments = 1
        );
        crown(
            "base_moulding",
            0.85,
            front_z + 0.005,
            back,
            0.22,
            M.mahogany2,
            0.03,
            0.018,
        );
        shell(1.7, 0.22, 1.97, back, front_z, M.mahogany, 0.03, "case");
        fbox!(
            F,
            "cornice",
            1.76,
            0.07,
            0.57,
            (0, 2.005, (back + 0.14) / 2.0),
            M.mahogany,
            bevel = 0.006,
            bevel_segments = 1
        );
        crown("crown", 0.88, 0.148, back, 2.04, M.mahogany2, 0.069, 0.05);
        // the face frame round the doors: rails above and below, stiles with a bead
        fbox!(
            F,
            "rail_top",
            1.7,
            0.07,
            0.02,
            (0, 1.935, front_z - 0.01),
            M.mahogany,
            bevel = 0.004,
            bevel_segments = 1
        );
        fbox!(
            F,
            "rail_low",
            1.7,
            0.04,
            0.02,
            (0, 0.23, front_z - 0.01),
            M.mahogany,
            bevel = 0.004,
            bevel_segments = 1
        );
        for s in [-1, 1] {
            let sf = s as f64;
            fbox!(
                F,
                format!("stile{s}"),
                0.04,
                1.69,
                0.02,
                (sf * 0.83, 1.09, front_z - 0.006),
                M.mahogany,
                bevel = 0.004,
                bevel_segments = 1
            );
            fcyl!(
                F,
                format!("stile_bead{s}"),
                0.006,
                1.6,
                (sf * 0.836, 1.09, front_z + 0.004),
                M.mahogany,
                verts = 6
            );
        }
        // inside: a shelf, a rail of clothes on hangers and folded linen above
        let zc = (back + front_z) / 2.0;
        fbox!(F, "shelf", 1.64, 0.02, 0.5, (0, 1.7, zc - 0.01), M.mahogany);
        fcyl!(
            F,
            "rail",
            0.012,
            1.6,
            (0, 1.6, zc),
            M.chrome,
            verts = 10,
            rot = (0, 90, 0)
        );
        hanging("shirt", -0.72, 0.72, 1.6, zc, 9, 0.4, 0.8);
        for (i, x) in [-0.5, 0.0, 0.5].into_iter().enumerate() {
            folded(&format!("linen{i}_"), x, 1.71, zc, 0.36, 0.34, 3, 0.035);
        }
    });
    join(&still, "armoire", Some(P3::ZERO));
    let mut doors = Vec::new();
    for s in [-1, 1] {
        let sf = s as f64;
        let parts = new_parts(|| {
            let x = sf * 0.4125;
            let holes: Vec<Vec<(f64, f64)>> = [0.34, -0.36]
                .iter()
                .map(|&dy| {
                    rounded_rect(0.58, 0.56, 0.004, 1)
                        .iter()
                        .map(|&(u, v)| (u, v + dy))
                        .collect()
                })
                .collect();
            fextrude!(
                F,
                format!("door{s}"),
                &rounded_rect(0.815, 1.66, 0.004, 1),
                0.025,
                (x, 1.08, front_z + 0.0125),
                M.mahogany,
                holes = holes,
                bevel = 0.004,
                bevel_segments = 1
            );
            for dy in [0.34, -0.36] {
                let b = 0.07;
                let secs: Vec<Vec<P3>> = [
                    (0.289, 0.279, front_z + 0.004),
                    (0.289, 0.279, front_z + 0.017),
                    (0.289 - b, 0.279 - b, front_z + 0.03),
                ]
                .iter()
                .map(|&(hw, hh, d)| {
                    [(1.0, 1.0), (-1.0, 1.0), (-1.0, -1.0), (1.0, -1.0)]
                        .iter()
                        .map(|&(u, v)| F.at(x + u * hw, 1.08 + dy + v * hh, d))
                        .collect()
                })
                .collect();
                loft!(
                    format!("panel{s}{}", float_repr(dy)),
                    &secs,
                    material = M.walnut,
                    smooth = false
                );
            }
            kit::knob!(
                F,
                format!("knob{s}"),
                sf * 0.09,
                1.08,
                front_z + 0.025,
                0.024,
                M.brass
            );
            if s > 0 {
                soft!(
                    F,
                    "escutcheon",
                    0.026,
                    0.05,
                    0.004,
                    (0.045, 1.0, front_z + 0.027),
                    M.brass,
                    round = 0.0015,
                    round_segments = 1
                );
                fbox!(
                    F,
                    "keyhole",
                    0.005,
                    0.016,
                    0.003,
                    (0.045, 1.0, front_z + 0.029),
                    M.mahogany2
                );
            }
        });
        let name = if s < 0 { "door_left" } else { "door_right" };
        doors.push(hinged(
            join(&parts, name, None),
            F.at(sf * 0.82, 1.08, front_z + 0.0125),
        ));
    }
    let objs = settle!(centre = false);
    swing!(
        doors[0],
        "doors_open",
        100.0,
        seconds = DOOR_SECONDS,
        ease = Ease::BackOut
    );
    swing!(
        doors[1],
        "doors_open",
        -100.0,
        seconds = DOOR_SECONDS,
        ease = Ease::BackOut
    );
    objs
}
