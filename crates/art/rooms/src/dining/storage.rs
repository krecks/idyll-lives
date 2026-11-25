//! K9: dining storage: the walnut sideboard, the painted buffet, the china cabinet and the bar cart. The
//! sideboard's, buffet's and china cabinet's doors are parts of their own, hinged; one of each swings open while a
//! resident puts clean things away there (they are `cupboard` stations), the others are still.

use super::common::{apron, rail, slab, turned};
use super::{F, M};
use artkit::forms::rounded_rect;
use artkit::json::float_repr;
use artkit::materials::Mat;
use artkit::math::{P3, PI, cos, sin};
use artkit::motion::Ease;
use artkit::parts::{hinged, join, new_parts, settle};
use artkit::place::Facing;
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fcone, fcyl, fextrude, fsphere, fsweep, revolve, swing};

pub(super) use artkit::kit::WALL;
pub(super) const LEG_H: f64 = 0.16; // a sideboard's legs
pub const DOOR_SECONDS: f64 = 0.9; // a storage door swings open in this long, while clean things are put away (a `cupboard`)

/// A cupboard in a cabinet: (x0, x1, y0, y1, depth).
pub(super) type Hollow = (f64, f64, f64, f64, f64);

/// A cabinet on four turned tapered legs (in brass shoes: `ferrule`), its back on the wall face, a top with a
/// moulded edge over it, a bead along the carcass's foot; returns the front's depth. `hollow`: (x0, x1, y0, y1,
/// depth), a cupboard open at the front behind a door that opens.
#[allow(clippy::too_many_arguments)]
pub(super) fn cabinet(
    w: f64,
    h: f64,
    d: f64,
    body: Mat,
    top: Mat,
    legs: Mat,
    top_t: f64,
    top_over: f64,
    hollow: Option<Hollow>,
    ferrule: Option<Mat>,
) -> f64 {
    let z = WALL + d / 2.0;
    let (yb, yt) = (LEG_H, h - top_t);
    match hollow {
        None => {
            fbox!(
                F,
                "carcass",
                w,
                yt - yb,
                d,
                (0, (yb + yt) / 2.0, z),
                body,
                bevel = 0.01
            );
        }
        Some((x0, x1, y0, y1, depth)) => {
            // solid either side (their sides one piece), the cupboard between: a back, a floor and a roof
            for (name, a, b) in [("carcass_l", -w / 2.0, x0), ("carcass_r", x1, w / 2.0)] {
                fbox!(
                    F,
                    name,
                    b - a,
                    yt - yb,
                    d,
                    ((a + b) / 2.0, (yb + yt) / 2.0, z),
                    body,
                    bevel = 0.01
                );
            }
            fbox!(
                F,
                "carcass_back",
                x1 - x0,
                yt - yb,
                d - depth,
                ((x0 + x1) / 2.0, (yb + yt) / 2.0, WALL + (d - depth) / 2.0),
                body
            );
            let front = WALL + d - depth / 2.0;
            fbox!(
                F,
                "carcass_floor",
                x1 - x0,
                y0 - yb,
                depth,
                ((x0 + x1) / 2.0, (yb + y0) / 2.0, front),
                body,
                bevel = 0.003
            );
            fbox!(
                F,
                "carcass_roof",
                x1 - x0,
                yt - y1,
                depth,
                ((x0 + x1) / 2.0, (y1 + yt) / 2.0, front),
                body,
                bevel = 0.003
            );
        }
    }
    slab(
        "top",
        w + 2.0 * top_over,
        d + top_over,
        h,
        top_t,
        top,
        0.012,
        None,
        3,
        (0.0, z + top_over / 2.0),
    );
    // a bead along the front's foot and round its sides
    fsweep!(
        F,
        "bead",
        &[
            (-w / 2.0 - 0.004, yb + 0.006, WALL + 0.02),
            (-w / 2.0 - 0.004, yb + 0.006, WALL + d + 0.004),
            (w / 2.0 + 0.004, yb + 0.006, WALL + d + 0.004),
            (w / 2.0 + 0.004, yb + 0.006, WALL + 0.02)
        ],
        0.0,
        body,
        profile = rounded_rect(0.014, 0.012, 0.005, 1),
        up = (0, 0, 1),
        angle = 50
    );
    let leg = [
        (0.0, 0.0),
        (0.0135, 0.0),
        (0.0145, 0.03),
        (0.019, LEG_H - 0.03),
        (0.022, LEG_H - 0.022),
        (0.022, LEG_H),
        (0.0, LEG_H),
    ];
    for x in [-w / 2.0 + 0.08, w / 2.0 - 0.08] {
        for zz in [WALL + 0.06, WALL + d - 0.06] {
            let at = format!("{}{}", float_repr(x), float_repr(zz));
            turned(&format!("leg{at}"), &leg, (x, 0, zz), legs, 12, 35.0);
            if let Some(ferrule) = ferrule {
                turned(
                    &format!("shoe{at}"),
                    &[
                        (0.0, 0.0),
                        (0.0138, 0.0),
                        (0.0148, 0.022),
                        (0.0146, 0.03),
                        (0.0, 0.03),
                    ],
                    (x, 0, zz),
                    ferrule,
                    12,
                    35.0,
                );
            }
        }
    }
    WALL + d
}

/// What is built on a door's face: (x0, x1, y0, y1, its face's depth).
pub(super) type OnDoor<'a> = &'a dyn Fn(f64, f64, f64, f64, f64);

/// A door from x0 to x1 (game), `y0` to `y1`, its back on the front face `zf`, its edges rounded: a part of its own,
/// hinged on its outer edge (one of them swings open while clean things are put away: `door_open`; the others
/// are still). `frame`: only a frame that wide (a glazed door), its inner edge moulded.
#[allow(clippy::too_many_arguments)]
pub(super) fn door(
    name: &str,
    x0: f64,
    x1: f64,
    y0: f64,
    y1: f64,
    zf: f64,
    material: Mat,
    hinge_left: bool,
    build: Option<OnDoor>,
    frame: f64,
) -> Obj {
    let parts = new_parts(|| {
        let (xm, ym, w, h) = ((x0 + x1) / 2.0, (y0 + y1) / 2.0, x1 - x0, y1 - y0);
        let outline = rounded_rect(w, h, 0.006, 1);
        if frame != 0.0 {
            let hole = rounded_rect(w - 2.0 * frame, h - 2.0 * frame, 0.004, 1);
            fextrude!(
                F,
                name,
                &outline,
                0.02,
                (xm, ym, zf + 0.01),
                material,
                holes = [hole],
                bevel = 0.004,
                bevel_segments = 1
            );
        } else {
            fextrude!(
                F,
                name,
                &outline,
                0.02,
                (xm, ym, zf + 0.01),
                material,
                bevel = 0.005,
                bevel_segments = 1
            );
        }
        if let Some(build) = build {
            build(x0, x1, y0, y1, zf + 0.02);
        }
    });
    let door = join(&parts, name, None);
    hinged(
        door,
        F.at(if hinge_left { x0 } else { x1 }, (y0 + y1) / 2.0, zf + 0.01),
    )
}

/// A mushroom knob on a face at depth `zf`, `l` long: a neck flaring into a domed head.
pub(super) fn knob(
    name: &str,
    x: f64,
    y: f64,
    zf: f64,
    material: Mat,
    r: f64,
    l: Option<f64>,
) -> Obj {
    let k = l.unwrap_or(r * 1.85) / 1.85;
    revolve!(
        name,
        &[
            (0.0, 0.0),
            (r * 0.6, 0.0),
            (r * 0.42, k * 0.5),
            (r * 0.5, k * 0.9),
            (r, k * 1.25),
            (r * 0.9, k * 1.6),
            (r * 0.5, k * 1.8),
            (0.0, k * 1.85)
        ],
        at = F.at(x, y, zf),
        material = material,
        segments = 10,
        rot = (90, 0, 0)
    )
}

/// A plate lying on a top at (x, y, z), or standing (`rot`, Blender degrees): a well and a rim.
#[allow(clippy::too_many_arguments)]
pub(super) fn plate(
    name: &str,
    x: f64,
    y: f64,
    z: f64,
    r: f64,
    material: Mat,
    segments: usize,
    rot: Option<P3>,
) -> Obj {
    let prof = [
        (0.0, 0.0),
        (r * 0.6, 0.0),
        (r * 0.64, r * 0.04),
        (r * 0.9, r * 0.1),
        (r, r * 0.14),
        (r * 0.98, r * 0.16),
        (r * 0.88, r * 0.12),
        (r * 0.62, r * 0.065),
        (0.0, r * 0.06),
    ];
    let obj = revolve!(
        name,
        &prof,
        at = F.at(x, y, z),
        material = material,
        segments = segments,
        rot = rot.unwrap_or((0, 0, 0).into())
    );
    obj.edit(|o| o.mesh.shade_smooth(35.0));
    obj
}

/// A shelf half way up a cupboard (`cabinet`'s `hollow`) and stacks of rimmed plates and bowls on its floor
/// and on the shelf; `zf` is the cabinet's front.
pub(super) fn crockery(cup: Hollow, zf: f64, shelf: Mat) {
    let (x0, x1, y0, y1, depth) = cup;
    let (xm, z) = ((x0 + x1) / 2.0, zf - depth / 2.0);
    let ym = (y0 + y1) / 2.0;
    fbox!(
        F,
        "cup_shelf",
        x1 - x0,
        0.018,
        depth - 0.02,
        (xm, ym, z - 0.01),
        shelf,
        bevel = 0.003
    );
    for (k, base) in [y0, ym + 0.009].into_iter().enumerate() {
        // a stack of plates, only the top one's well and rim showing; a stack of bowls, the top one open
        for i in 0..5 {
            fcyl!(
                F,
                format!("cup_plate{k}{i}"),
                0.1,
                0.012,
                (xm - 0.06, base + 0.006 + 0.014 * f64::from(i), z),
                if i % 3 != 0 {
                    M.porcelain
                } else {
                    M.plate_blue
                },
                verts = 14
            );
        }
        plate(
            &format!("cup_plate{k}5"),
            xm - 0.06,
            base + 0.07,
            z,
            0.1,
            M.porcelain,
            14,
            None,
        );
        for i in 0..2 {
            fcone!(
                F,
                format!("cup_bowl{k}{i}"),
                0.04,
                0.066,
                0.045,
                (xm + 0.11, base + 0.0225 + 0.02 * f64::from(i), z + 0.04),
                M.porcelain,
                verts = 12
            );
        }
        turned(
            &format!("cup_bowl{k}2"),
            &[
                (0.0, 0.0),
                (0.04, 0.0),
                (0.06, 0.03),
                (0.066, 0.045),
                (0.06, 0.045),
                (0.054, 0.03),
                (0.036, 0.008),
                (0.0, 0.008),
            ],
            (xm + 0.11, base + 0.04, z + 0.04),
            M.porcelain,
            12,
            35.0,
        );
    }
}

/// Walnut sideboard (catalog2, 2x1): four fluted doors with brass knobs on turned legs in brass shoes, a top with
/// a moulded edge; a vase and a bowl of lemons on top.
pub fn sideboard() -> Vec<Obj> {
    let (w, h, d) = (1.8, 0.8, 0.62);
    let (n, y0, y1) = (4, LEG_H + 0.02, h - 0.05);
    let dw = (w - 0.04) / f64::from(n);
    // the second door's cupboard (the door from x0 to x1), with plates and bowls in it
    let cx0 = -w / 2.0 + 0.02 + dw + 0.006;
    let cup = (cx0 + 0.02, cx0 + dw - 0.032, y0 + 0.02, y1 - 0.02, 0.45);
    let mut zf = 0.0;
    let still = new_parts(|| {
        zf = cabinet(
            w,
            h,
            d,
            M.body_walnut,
            M.top_sideboard,
            M.body_walnut,
            0.03,
            0.01,
            Some(cup),
            Some(M.brass),
        );
        crockery(cup, zf, M.body_walnut);
        turned(
            "vase",
            &[
                (0.0, 0.0),
                (0.05, 0.0),
                (0.052, 0.01),
                (0.068, 0.07),
                (0.066, 0.12),
                (0.048, 0.2),
                (0.033, 0.26),
                (0.031, 0.28),
                (0.04, 0.295),
                (0.038, 0.3),
                (0.03, 0.3),
                (0.026, 0.28),
                (0.0, 0.27),
            ],
            (-0.6, h, -0.3),
            M.cream,
            16,
            35.0,
        );
        turned(
            "bowl",
            &[
                (0.0, 0.0),
                (0.05, 0.0),
                (0.055, 0.008),
                (0.1, 0.045),
                (0.11, 0.06),
                (0.104, 0.062),
                (0.092, 0.05),
                (0.045, 0.016),
                (0.0, 0.016),
            ],
            (0.45, h, -0.2),
            M.cream,
            18,
            35.0,
        );
        for (i, (dx, dz, dy)) in [
            (-0.04, 0.0, 0.0),
            (0.035, -0.02, 0.0),
            (0.0, 0.04, 0.0),
            (0.0, 0.0, 0.035),
        ]
        .into_iter()
        .enumerate()
        {
            let i = i as i32;
            fsphere!(
                F,
                format!("lemon{i}"),
                0.03,
                (0.45 + dx, h + 0.045 + dy, -0.2 + dz),
                M.lemon,
                scale = (1, 1.25, 1),
                rot = (0, 25 * i, 40 * i),
                segments = 10,
                rings = 7
            );
        }
    });
    join(&still, "body", Some((0, 0, 0).into()));
    let mut doors = Vec::new();
    let flutes = |x0: f64, x1: f64, y0: f64, y1: f64, zf: f64| {
        // half-round reeds standing proud of the door, stopped short of its top and bottom
        let half: Vec<(f64, f64)> = (0..7)
            .map(|i| {
                let a = PI * f64::from(i) / 6.0;
                (0.006 * sin(a), 0.006 * cos(a))
            })
            .collect();
        for k in -3..4 {
            let x = (x0 + x1) / 2.0 + f64::from(k) * dw / 8.0;
            fsweep!(
                F,
                format!("flute{}{k}", float_repr(x0)),
                &[(x, y0 + 0.035, zf - 0.001), (x, y1 - 0.035, zf - 0.001)],
                0.0,
                M.flute,
                profile = &half,
                up = (0, -1, 0),
                angle = 60
            );
        }
    };
    for i in 0..n {
        let x0 = -w / 2.0 + 0.02 + dw * f64::from(i) + 0.006;
        let x1 = x0 + dw - 0.012;
        let left = i % 2 == 0;
        let on_door = |a: f64, b: f64, c: f64, e: f64, zf: f64| {
            knob(
                &format!("knob{}", float_repr(a)),
                if left { b - 0.05 } else { a + 0.05 },
                (c + e) / 2.0,
                zf,
                M.brass,
                0.014,
                Some(0.018),
            );
            flutes(a, b, c, e, zf);
        };
        doors.push(door(
            &format!("door{i}"),
            x0,
            x1,
            y0,
            y1,
            zf,
            M.body_walnut,
            left,
            Some(&on_door),
            0.0,
        ));
    }
    settle(&meshes()); // where the game places it, against the wall (its Model's centre)
    // the second door (hinged on its right) swings open while clean things are put away; one door only, as
    // each costs draws of its own
    swing!(
        doors[1],
        "door_open",
        -100.0,
        seconds = DOOR_SECONDS,
        ease = Ease::BackOut
    );
    meshes()
}

/// A bar pull `w` long across a face at depth `zf`: a rounded bar on two posts, bent from one piece.
pub(super) fn pull(name: &str, x: f64, y: f64, zf: f64, w: f64, material: Mat) -> Obj {
    rail(
        name,
        &[
            (x - w / 2.0, y, zf),
            (x - w / 2.0, y, zf + 0.022),
            (x + w / 2.0, y, zf + 0.022),
            (x + w / 2.0, y, zf),
        ],
        0.005,
        material,
        0.008,
        6,
        3,
        false,
    )
}

/// Painted buffet (catalog2, 2x1): a sage cabinet, three drawers with brass bar pulls over five panelled doors
/// with brass knobs, an oak top with a moulded edge, an arched apron between turned oak legs.
pub fn buffet() -> Vec<Obj> {
    let (w, h, d) = (1.84, 0.9, 0.62);
    let ym = h - 0.035 - 0.15;
    // the wide middle door's cupboard, with plates and bowls in it
    let cup = (-0.265, 0.265, LEG_H + 0.04, ym - 0.03, 0.45);
    let mut zf = 0.0;
    let still = new_parts(|| {
        zf = cabinet(
            w,
            h,
            d,
            M.body_sage,
            M.top_oak,
            M.legs_oak,
            0.035,
            0.02,
            Some(cup),
            None,
        );
        crockery(cup, zf, M.body_sage);
        apron(
            "apron",
            w - 0.24,
            0.035,
            0.012,
            (0, LEG_H + 0.002, zf - 0.008),
            M.body_sage,
            Facing::Front,
            0.012,
            12,
            0.003,
        );
        for x in [-0.6, 0.0, 0.6] {
            let xs = float_repr(x);
            fextrude!(
                F,
                format!("drawer{xs}"),
                &rounded_rect(0.56, 0.13, 0.006, 1),
                0.02,
                (x, ym + 0.075, zf + 0.01),
                M.body_sage,
                bevel = 0.005,
                bevel_segments = 1
            );
            fextrude!(
                F,
                format!("drawer_field{xs}"),
                &rounded_rect(0.5, 0.08, 0.004, 1),
                0.006,
                (x, ym + 0.075, zf + 0.022),
                M.body_sage,
                bevel = 0.0025,
                bevel_segments = 1
            );
            pull(
                &format!("pull{xs}"),
                x,
                ym + 0.075,
                zf + 0.025,
                0.12,
                M.brass,
            );
        }
        for x in [-0.3, 0.3] {
            fbox!(
                F,
                format!("stile{}", float_repr(x)),
                0.03,
                ym - LEG_H - 0.02,
                0.01,
                (x, (ym + LEG_H) / 2.0, zf + 0.005),
                M.body_sage
            );
        }
    });
    join(&still, "body", Some((0, 0, 0).into()));
    let panel = |x0: f64, x1: f64, y0: f64, y1: f64, zf: f64, left: bool| {
        // a raised field: a bevelled panel on the door's face, a fillet round it
        fextrude!(
            F,
            format!("panel{}", float_repr(x0)),
            &rounded_rect(x1 - x0 - 0.07, y1 - y0 - 0.08, 0.005, 1),
            0.01,
            ((x0 + x1) / 2.0, (y0 + y1) / 2.0, zf + 0.002),
            M.body_sage,
            bevel = 0.0045,
            bevel_segments = 1
        );
        knob(
            &format!("knob{}", float_repr(x0)),
            if left { x1 - 0.04 } else { x0 + 0.04 },
            y1 - 0.09,
            zf + 0.002,
            M.brass,
            0.016,
            None,
        );
    };
    let (y0, y1) = (LEG_H + 0.02, ym - 0.01);
    let doors = [
        (-0.89, -0.605, true),
        (-0.595, -0.31, false),
        (-0.285, 0.285, true),
        (0.31, 0.595, true),
        (0.605, 0.89, false),
    ];
    let parts: Vec<Obj> = doors
        .iter()
        .enumerate()
        .map(|(i, &(x0, x1, left))| {
            let on_door = |a: f64, b: f64, c: f64, e: f64, z: f64| panel(a, b, c, e, z, left);
            door(
                &format!("door{i}"),
                x0,
                x1,
                y0,
                y1,
                zf,
                M.body_sage,
                left,
                Some(&on_door),
                0.0,
            )
        })
        .collect();
    settle(&meshes()); // where the game places it, against the wall (its Model's centre)
    // the wide middle door (hinged on its left) swings open while clean things are put away
    swing!(
        parts[2],
        "door_open",
        100.0,
        seconds = DOOR_SECONDS,
        ease = Ease::BackOut
    );
    meshes()
}
