//! BR2: the nightstands, the blanket chest, the laundry hamper and the bed-end bench.

use super::common::{
    DRAWER_SECONDS, F, M, PULL, Style, WALL, book, books, drawer, front, gap, mug, shell, top,
};
use artkit::forms::rounded_rect;
use artkit::json::float_repr;
use artkit::math::{Axis, P3, PI, TAU, cos, cosd, sin, sind};
use artkit::parts::{hinged, join, new_parts};
use artkit::place::Facing;
use artkit::scene::Obj;
use artkit::{
    fbox, fcone, fcyl, fextrude, fsphere, fsweep, jitter, loft, revolve, ring, rod, slide, soft,
    swing, torus,
};
use artkit::{kit, settle};

// --- BR2: the nightstand: oak, a drawer over an open niche, on four short tapered legs -----------------
fn classic_nightstand(decor: bool) {
    let front_z = -0.02;
    let (d, zc) = (front_z - WALL, (WALL + front_z) / 2.0);
    for sx in [-1, 1] {
        for z in [WALL + 0.035, front_z - 0.035] {
            fcone!(
                F,
                format!("leg{sx}{}", float_repr(z)),
                0.022,
                0.015,
                0.145,
                (sx as f64 * 0.22, 0.0725, z),
                M.oak,
                verts = 12
            );
        }
    }
    shell(0.52, 0.14, 0.535, WALL, front_z, M.oak, 0.02, "case");
    // a shaped apron under the niche, between the front legs
    let edge: Vec<(f64, f64)> = (0..13)
        .map(|i| {
            let u = 0.24 - 0.48 * i as f64 / 12.0;
            (
                u,
                0.1 + 0.025 * cos(PI * (0.24 - 0.48 * i as f64 / 12.0) / 0.48),
            )
        })
        .collect();
    let mut outline = vec![(0.24, 0.142), (-0.24, 0.142)];
    outline.extend(edge.iter().rev());
    fextrude!(
        F,
        "apron",
        &outline,
        0.02,
        (0, 0, front_z - 0.012),
        M.oak,
        bevel = 0.003,
        bevel_segments = 1
    );
    fbox!(
        F,
        "shelf",
        0.48,
        0.02,
        d - 0.02,
        (0, 0.37, zc + 0.01),
        M.oak,
        bevel = 0.003,
        bevel_segments = 1
    );
    fbox!(
        F,
        "drawer_case",
        0.48,
        0.135,
        d - 0.02,
        (0, 0.4575, zc - 0.01),
        M.oak
    );
    front(
        "drawer",
        0.48,
        0.14,
        0,
        0.45,
        front_z + 0.01,
        M.oak,
        Style::Plain,
        None,
    );
    kit::knob!(F, "knob", 0, 0.45, front_z + 0.02, 0.018, M.brass);
    top("top", 0.54, 0.025, 0.44, 0.56, WALL + 0.22, M.oak, 0.01);
    books("niche_book", -0.2, 0.16, -0.22, 4, 0.17, 0.15);
    if decor {
        book("book", 0.1, 0.56, -0.16, 0.16, 0.03, 0.22, M.teal);
        mug("mug", -0.12, 0.56, -0.12);
    }
}

/// The nightstand.
pub fn nightstand() -> Vec<Obj> {
    let still = new_parts(|| classic_nightstand(true));
    join(&still, "nightstand", Some(P3::ZERO));
    settle!(centre = false)
}

// --- the nightstand with a lamp: the same nightstand, a gourd lamp with a drum shade on it --------------
/// The nightstand with a lamp.
pub fn nightstand_lamp() -> Vec<Obj> {
    let still = new_parts(|| {
        classic_nightstand(false);
        let z = -0.26;
        revolve!(
            "lamp_foot",
            &[
                (0.0, 0.0),
                (0.05, 0.0),
                (0.052, 0.006),
                (0.044, 0.012),
                (0.0, 0.012)
            ],
            at = F.at(0, 0.56, z),
            material = M.brass,
            segments = 20
        );
        revolve!(
            "lamp_base",
            &[
                (0.0, 0.01),
                (0.045, 0.01),
                (0.068, 0.04),
                (0.08, 0.088),
                (0.066, 0.135),
                (0.03, 0.166),
                (0.022, 0.18),
                (0.0, 0.18)
            ],
            at = F.at(0, 0.56, z),
            material = M.porcelain,
            segments = 20
        );
        revolve!(
            "lamp_collar",
            &[
                (0.0, 0.0),
                (0.026, 0.0),
                (0.026, 0.012),
                (0.012, 0.022),
                (0.0, 0.022)
            ],
            at = F.at(0, 0.735, z),
            material = M.brass,
            segments = 12
        );
        fcyl!(
            F,
            "lamp_stem",
            0.008,
            0.13,
            (0, 0.80, z),
            M.brass,
            verts = 10
        );
        fcyl!(
            F,
            "lamp_socket",
            0.016,
            0.04,
            (0, 0.88, z),
            M.brass,
            verts = 12
        );
        fsphere!(
            F,
            "lamp_bulb",
            0.035,
            (0, 0.935, z),
            M.bulb,
            segments = 14,
            rings = 8,
            scale = (1, 1, 1.15)
        );
        // the drum shade, open at both ends (a ring of thickness), trimmed at both edges, on a harp and spider
        revolve!(
            "lamp_shade",
            &[
                (0.15, 0.0),
                (0.132, 0.22),
                (0.126, 0.22),
                (0.144, 0.0),
                (0.15, 0.0)
            ],
            at = F.at(0, 0.86, z),
            material = M.shade,
            segments = 28,
            smooth = false
        );
        for (k, (r, y)) in [(0.149, 0.864), (0.132, 1.076)].into_iter().enumerate() {
            let pts: Vec<(f64, f64, f64)> = (0..28)
                .map(|i| {
                    (
                        r * cos(TAU * i as f64 / 28.0),
                        y,
                        z + r * sin(TAU * i as f64 / 28.0),
                    )
                })
                .collect();
            fsweep!(
                F,
                format!("lamp_trim{k}"),
                &pts,
                0.004,
                M.shade,
                verts = 4,
                closed = true
            );
        }
        torus!(
            "lamp_ring",
            0.128,
            0.003,
            at = F.at(0, 1.075, z),
            material = M.brass,
            major_segments = 20,
            minor_segments = 4
        );
        for a in [0, 120, 240] {
            let af = a as f64;
            rod!(
                format!("lamp_spoke{a}"),
                F.at(0, 1.075, z),
                F.at(0.128 * cosd(af), 1.075, z + 0.128 * sind(af)),
                0.0025,
                material = M.brass,
                verts = 4
            );
        }
        for s in [-1, 1] {
            let sx = s as f64;
            fsweep!(
                F,
                format!("lamp_harp{s}"),
                &[
                    (0.0, 0.895, z),
                    (sx * 0.03, 0.93, z),
                    (sx * 0.035, 1.0, z),
                    (0.0, 1.075, z)
                ],
                0.0025,
                M.brass,
                verts = 4,
                resolution = 2
            );
        }
        fsphere!(
            F,
            "lamp_finial",
            0.012,
            (0, 1.08, z),
            M.brass,
            segments = 10,
            rings = 6,
            scale = (1, 1, 0.8)
        );
    });
    join(&still, "nightstandLamp", Some(P3::ZERO));
    settle!(centre = false)
}

// --- the modern nightstand: white, an oak top and legs, a drawer over an open niche of books -----------
// 0.47 m deep, as the classic one (it was 0.63, as deep as its tile and past the pillows: M-bedroom-02).
pub const NS_FRONT: f64 = 0.0; // the case's front
const NS_TOP: (f64, f64) = (0.46, -0.20); // the top's depth and middle

/// A round leg tapering from the case at height `h` over (x, z) to a foot splayed out by `out`.
fn splayed(name: &str, x: f64, z: f64, h: f64, r: f64, material: artkit::materials::Mat) -> Obj {
    let out: f64 = 0.016;
    let (sx, sz) = (out.copysign(x), out.copysign(z + 0.21));
    kit::tube!(
        F,
        name,
        &[(x, h, z), (x + sx, 0.002, z + sz)],
        r,
        material,
        verts = 10,
        round = 1,
        scale = [1.0, 0.65]
    )
}

/// The modern nightstand.
pub fn nightstand_modern() -> Vec<Obj> {
    let still = new_parts(|| {
        let (front_z, back) = (NS_FRONT, -0.42);
        for sx in [-1, 1] {
            for z in [-0.37, front_z - 0.05] {
                splayed(
                    &format!("leg{sx}{}", float_repr(z)),
                    sx as f64 * 0.23,
                    z,
                    0.13,
                    0.018,
                    M.oak,
                );
            }
        }
        shell(0.56, 0.12, 0.53, back, front_z, M.white, 0.02, "case");
        fbox!(
            F,
            "shelf",
            0.52,
            0.02,
            front_z - back - 0.02,
            (0, 0.37, (back + front_z) / 2.0 + 0.01),
            M.white
        );
        fbox!(
            F,
            "drawer_case",
            0.52,
            0.14,
            front_z - back - 0.04,
            (0, 0.45, (back + front_z) / 2.0 - 0.01),
            M.white
        );
        front(
            "drawer",
            0.52,
            0.135,
            0,
            0.445,
            front_z + 0.01,
            M.white,
            Style::Plain,
            None,
        );
        kit::pull!(
            F,
            "pull",
            0.0,
            0.485,
            front_z + 0.02,
            0.12,
            M.iron,
            r = 0.005,
            out = 0.016
        );
        top("top", 0.58, 0.03, NS_TOP.0, 0.56, NS_TOP.1, M.oak, 0.012);
        books("niche_book", -0.22, 0.14, -0.2, 5, 0.19, 0.16);
        soft!(
            F,
            "niche_box",
            0.14,
            0.08,
            0.16,
            (0.15, 0.18, -0.18),
            M.box_,
            round = 0.01,
            round_segments = 1
        );
        soft!(
            F,
            "niche_box_lid",
            0.146,
            0.02,
            0.166,
            (0.15, 0.225, -0.18),
            M.box_,
            round = 0.008,
            round_segments = 1
        );
    });
    join(&still, "nightstandModern", Some(P3::ZERO));
    settle!(centre = false)
}

// --- the drum nightstand: a terracotta drum with a drawer band, an oak top and a round foot ------------
/// The drum nightstand.
pub fn nightstand_round() -> Vec<Obj> {
    let z = -0.06;
    let still = new_parts(|| {
        revolve!(
            "foot",
            &[
                (0.0, 0.0),
                (0.19, 0.0),
                (0.2, 0.006),
                (0.2, 0.034),
                (0.19, 0.04),
                (0.0, 0.04)
            ],
            at = F.at(0, 0, z),
            material = M.clay2,
            segments = 36
        );
        // a drum swelling a little to its middle, its lower edge rounded under
        revolve!(
            "drum",
            &[
                (0.0, 0.04),
                (0.228, 0.04),
                (0.244, 0.05),
                (0.25, 0.075),
                (0.253, 0.2),
                (0.253, 0.34),
                (0.25, 0.52),
                (0.242, 0.54),
                (0.0, 0.54)
            ],
            at = F.at(0, 0, z),
            material = M.clay,
            segments = 36
        );
        fcyl!(
            F,
            "band",
            0.256,
            0.12,
            (0, 0.42, z),
            M.clay2,
            verts = 36,
            bevel = 0.003,
            bevel_segments = 1
        );
        for y in [0.358, 0.482] {
            torus!(
                format!("groove{}", float_repr(y)),
                0.256,
                0.004,
                at = F.at(0, y, z),
                material = M.plinth,
                major_segments = 36,
                minor_segments = 4
            );
        }
        // the drawer's front: the band's front third, between two seams
        for a in [-35, 35] {
            let af = a as f64;
            fbox!(
                F,
                format!("seam{a}"),
                0.004,
                0.118,
                0.006,
                (0.257 * sind(af), 0.42, z + 0.257 * cosd(af)),
                M.plinth,
                rot = (0, 0, -a)
            );
        }
        kit::knob!(F, "knob", 0, 0.42, z + 0.255, 0.0125, M.brass); // (2.5 cm: the drum stays a nightstand's depth)
        revolve!(
            "top",
            &[
                (0.0, 0.54),
                (0.26, 0.54),
                (0.268, 0.547),
                (0.27, 0.556),
                (0.268, 0.565),
                (0.26, 0.57),
                (0.0, 0.57)
            ],
            at = F.at(0, 0, z),
            material = M.oak,
            segments = 36
        );
    });
    join(&still, "nightstandRound", Some(P3::ZERO));
    settle!(centre = false)
}

// --- the three-drawer nightstand: walnut, three drawers with brass bars; the top one a part of its own ---
// 0.47 m deep, as the modern one; a midnight snack in the top drawer (the use `snack`).
/// The three-drawer nightstand.
pub fn nightstand_drawers() -> Vec<Obj> {
    let (front_z, back) = (NS_FRONT, -0.42);
    let still = new_parts(|| {
        fbox!(
            F,
            "plinth",
            0.52,
            0.05,
            front_z - back - 0.04,
            (0, 0.025, (back + front_z) / 2.0 - 0.01),
            M.walnut,
            bevel = 0.004,
            bevel_segments = 1
        );
        soft!(
            F,
            "case",
            0.56,
            0.52,
            front_z - back,
            (0, 0.31, (back + front_z) / 2.0),
            M.walnut,
            round = 0.01,
            round_segments = 2
        );
        top("top", 0.58, 0.03, NS_TOP.0, 0.6, NS_TOP.1, M.walnut, 0.012);
        for (i, y) in [0.335, 0.185].into_iter().enumerate() {
            front(
                &format!("drawer{i}"),
                0.52,
                0.135,
                0,
                y,
                front_z + 0.01,
                M.walnut,
                Style::Plain,
                None,
            );
            kit::pull!(
                F,
                &format!("pull{i}"),
                0.0,
                y + 0.02,
                front_z + 0.02,
                0.11,
                M.brass,
                r = 0.005,
                out = 0.016
            );
        }
        gap("drawer", 0.52, 0.135, 0, 0.485, front_z, M.plinth);
    });
    join(&still, "nightstandDrawers", Some(P3::ZERO));
    // the top drawer with snacks and a paperback in it, sliding out while a resident takes a snack
    let d = drawer(
        "drawer",
        0.52,
        0.135,
        0,
        0.485,
        front_z,
        0.36,
        M.walnut,
        &|| {
            kit::pull!(
                F,
                "pull_top",
                0.0,
                0.505,
                front_z + 0.02,
                0.11,
                M.brass,
                r = 0.005,
                out = 0.016
            );
        },
        Some(&drawer_snacks),
        Style::Plain,
    );
    let objs = settle!(centre = false);
    slide!(d, "drawer_open", (0, -PULL, 0), seconds = DRAWER_SECONDS);
    objs
}

/// A nightstand drawer's midnight snacks, bottom at `y` round (x, z): a biscuit tin, two chocolate bars and a
/// paperback.
fn drawer_snacks(x: f64, y: f64, z: f64) {
    revolve!(
        "tin",
        &[
            (0.0, 0.0),
            (0.068, 0.0),
            (0.07, 0.005),
            (0.07, 0.06),
            (0.072, 0.062),
            (0.072, 0.07),
            (0.0, 0.07)
        ],
        at = F.at(x - 0.12, y, z),
        material = M.tin,
        segments = 16
    );
    for (i, dx) in [0.03, 0.07].into_iter().enumerate() {
        let fi = i as f64;
        fbox!(
            F,
            format!("bar{i}"),
            0.05,
            0.015,
            0.16,
            (x + dx, y + 0.0075 + 0.015 * fi, z + 0.02 * fi),
            M.wrapper,
            bevel = 0.004,
            bevel_segments = 1
        );
    }
    book(
        "paperback",
        x + 0.18,
        y,
        z - 0.02,
        0.12,
        0.025,
        0.18,
        M.book2,
    );
}

// --- the blanket chest: a cedar box on a plinth with iron straps and side handles; the lid a part --------
/// The blanket chest.
pub fn cedar_chest() -> Vec<Obj> {
    let (w, d, y0, y1) = (0.9, 0.46, 0.06, 0.455);
    let t = 0.025;
    let still = new_parts(|| {
        // the plinth: a moulded band on bracket feet, cut away in an arch between them
        soft!(
            F,
            "plinth_band",
            0.94,
            0.025,
            0.5,
            (0, 0.0475, 0),
            M.cedar2,
            round = 0.008,
            round_segments = 1
        );
        for s in [-1, 1] {
            let curve: Vec<(f64, f64)> = (0..17)
                .map(|i| {
                    (
                        0.47 - 0.94 * i as f64 / 16.0,
                        0.012 + 0.02 * sin(PI * i as f64 / 16.0),
                    )
                })
                .collect();
            let mut arch = vec![
                (0.47, 0.0),
                (0.47, 0.036),
                (-0.47, 0.036),
                (-0.47, 0.0),
                (-0.39, 0.0),
            ];
            arch.extend(curve.iter().rev().filter(|p| p.0.abs() < 0.39));
            arch.push((0.39, 0.0));
            arch.reverse();
            fextrude!(
                F,
                format!("plinth_x{s}"),
                &arch,
                0.02,
                (0, 0, s as f64 * 0.24),
                M.cedar2,
                bevel = 0.003,
                bevel_segments = 1
            );
            let mut side = vec![
                (0.25, 0.0),
                (0.25, 0.036),
                (-0.25, 0.036),
                (-0.25, 0.0),
                (-0.17, 0.0),
                (-0.12, 0.022),
                (0.12, 0.022),
                (0.17, 0.0),
            ];
            side.reverse();
            fextrude!(
                F,
                format!("plinth_z{s}"),
                &side,
                0.02,
                (s as f64 * 0.46, 0, 0),
                M.cedar2,
                plane = Facing::Side,
                bevel = 0.003,
                bevel_segments = 1
            );
        }
        for s in [-1, 1] {
            let sf = s as f64;
            fbox!(
                F,
                format!("wall_z{s}"),
                w,
                y1 - y0,
                t,
                (0, (y0 + y1) / 2.0, sf * (d - t) / 2.0),
                M.cedar,
                bevel = 0.004,
                bevel_segments = 1
            );
            fbox!(
                F,
                format!("wall_x{s}"),
                t,
                y1 - y0,
                d - 2.0 * t,
                (sf * (w - t) / 2.0, (y0 + y1) / 2.0, 0),
                M.cedar,
                bevel = 0.004,
                bevel_segments = 1
            );
        }
        fbox!(
            F,
            "floor",
            w - 2.0 * t,
            0.02,
            d - 2.0 * t,
            (0, y0 + 0.01, 0),
            M.cedarin
        );
        // two raised panels in frames on the front
        for s in [-1, 1] {
            kit::shaker!(
                F,
                format!("front_panel{s}"),
                0.34,
                0.27,
                (s as f64 * 0.2, 0.255, d / 2.0 + 0.006),
                0.012,
                M.cedar2,
                rail = 0.035,
                panel = M.cedar,
                raised = true,
                recess = 0.003
            );
        }
        // iron corner brackets with rivets, and a drop handle each side on a plate
        for s in [-1, 1] {
            let sf = s as f64;
            for y in [0.09, 0.425] {
                fbox!(
                    F,
                    format!("corner_front{s}{}", float_repr(y)),
                    0.05,
                    0.05,
                    0.004,
                    (sf * (w / 2.0 - 0.025), y, d / 2.0 + 0.002),
                    M.iron,
                    bevel = 0.0015,
                    bevel_segments = 1
                );
                fbox!(
                    F,
                    format!("corner_side{s}{}", float_repr(y)),
                    0.004,
                    0.05,
                    0.05,
                    (sf * (w / 2.0 + 0.002), y, d / 2.0 - 0.025),
                    M.iron,
                    bevel = 0.0015,
                    bevel_segments = 1
                );
                fsphere!(
                    F,
                    format!("rivet{s}{}", float_repr(y)),
                    0.005,
                    (sf * (w / 2.0 - 0.02), y, d / 2.0 + 0.004),
                    M.iron,
                    segments = 6,
                    rings = 3,
                    scale = (1, 1, 0.5)
                );
            }
            let hx = sf * (w / 2.0 + 0.006);
            fbox!(
                F,
                format!("handle_plate{s}"),
                0.008,
                0.05,
                0.13,
                (hx, 0.36, 0),
                M.iron,
                bevel = 0.002,
                bevel_segments = 1
            );
            fsweep!(
                F,
                format!("handle{s}"),
                &kit::fillet(
                    &[
                        (sf * (w / 2.0 + 0.014), 0.365, -0.055),
                        (sf * (w / 2.0 + 0.024), 0.3, -0.055),
                        (sf * (w / 2.0 + 0.024), 0.3, 0.055),
                        (sf * (w / 2.0 + 0.014), 0.365, 0.055)
                    ],
                    0.014,
                    3
                ),
                0.006,
                M.iron,
                verts = 6
            );
        }
        // the lock: a brass escutcheon with a black keyhole, under the lid's front edge
        soft!(
            F,
            "latch",
            0.07,
            0.06,
            0.006,
            (0, 0.415, d / 2.0 + 0.003),
            M.brass,
            round = 0.002,
            round_segments = 1
        );
        fcyl!(
            F,
            "keyhole",
            0.006,
            0.004,
            (0, 0.42, d / 2.0 + 0.006),
            M.iron,
            rot = (90, 0, 0),
            verts = 8
        );
        fbox!(
            F,
            "keyhole_slot",
            0.004,
            0.012,
            0.004,
            (0, 0.411, d / 2.0 + 0.006),
            M.iron
        );
        let b = soft!(
            F,
            "blanket",
            w - 2.0 * t - 0.02,
            0.1,
            d - 2.0 * t - 0.02,
            (0, y0 + 0.08, 0),
            M.cloth3,
            round = 0.03,
            puff = 0.012,
            flat = 3
        );
        jitter!(b, 0.006, size = 0.1, seed = 4, up = 0.5);
    });
    join(&still, "cedarChest", Some(P3::ZERO));
    // the lid, hinged along its back edge: it lifts while a resident takes out a blanket (`lid_open`)
    let parts = new_parts(|| {
        // (its underside 2 mm below the edge band's, not in its plane: seen when the lid is up)
        soft!(
            F,
            "lid",
            0.94,
            0.047,
            0.5,
            (0, 0.4765, 0),
            M.cedar,
            round = 0.012,
            round_segments = 2
        );
        soft!(
            F,
            "lid_edge",
            0.96,
            0.012,
            0.52,
            (0, 0.461, 0),
            M.cedar2,
            round = 0.005,
            round_segments = 1
        );
        let b = 0.04;
        fextrude!(
            F,
            "lid_frame",
            &rounded_rect(0.84, 0.4, 0.01, 1),
            0.006,
            (0, 0.5, 0),
            M.cedar2,
            plane = Facing::Top,
            holes = [rounded_rect(0.74, 0.3, 0.01, 1)],
            bevel = 0.002,
            bevel_segments = 1
        );
        // the raised field inside it, chamfered up to its flat top (the chest's top, 0.505)
        let secs: Vec<Vec<P3>> = [
            (0.499, 0.369, 0.149),
            (0.5005, 0.369, 0.149),
            (0.505, 0.369 - b, 0.149 - b),
        ]
        .iter()
        .map(|&(y, k, k2)| {
            [(1.0, 1.0), (-1.0, 1.0), (-1.0, -1.0), (1.0, -1.0)]
                .iter()
                .map(|&(u, v)| F.at(u * k, y, v * k2))
                .collect()
        })
        .collect();
        loft!("lid_panel", &secs, material = M.cedar, smooth = false);
    });
    let lid = hinged(join(&parts, "lid", None), F.at(0, 0.455, -0.25));
    let objs = settle!(centre = false);
    swing!(lid, "lid_open", -95.0, seconds = 0.9, axis = Axis::X);
    objs
}

// --- the laundry hamper: a woven basket, wider at the top, with bands; the lid a part ----------------------
/// A braided band round the hamper at height `y`, `r` from its middle: a cord swelling and narrowing in turn.
fn braid(
    name: &str,
    r: f64,
    y: f64,
    z: f64,
    material: artkit::materials::Mat,
    n: usize,
    cord: f64,
) {
    let pts: Vec<(f64, f64, f64)> = (0..n)
        .map(|i| {
            (
                r * cos(TAU * i as f64 / n as f64),
                y,
                z + r * sin(TAU * i as f64 / n as f64),
            )
        })
        .collect();
    let scale: Vec<f64> = (0..n).map(|i| 1.0 + 0.3 * (i % 2) as f64).collect();
    fsweep!(
        F,
        name,
        &pts,
        cord,
        material,
        verts = 4,
        closed = true,
        scale = scale,
        angle = 100 // a cord is round: its four sides shade so
    );
}

/// The laundry hamper.
pub fn laundry_hamper() -> Vec<Obj> {
    let z = -0.2;
    let b0 = F.at(0, 0, z);
    let (bx, by) = (b0.x, b0.y);
    let rib = |y: f64, r: f64, k: f64| -> Vec<P3> {
        (0..48)
            .map(|i| {
                let a = TAU * i as f64 / 48.0;
                P3 {
                    x: bx + r * (1.0 + k * cos(24.0 * a)) * cos(a),
                    y: by + r * (1.0 + k * cos(24.0 * a)) * sin(a),
                    z: y,
                }
            })
            .collect()
    };
    let still = new_parts(|| {
        // the basket: woven staves wider at the top, its wall 14 mm thick, open, and a woven floor inside
        let mut outer = vec![(0.0, 0.196, 0.0), (0.02, 0.2, 0.012)];
        outer.extend([0.15, 0.3, 0.45].map(|y| (y, 0.2 + 0.031 * y / 0.56, 0.014)));
        outer.push((0.56, 0.231, 0.012));
        let mut secs: Vec<Vec<P3>> = outer.iter().map(|&(y, r, k)| rib(y, r, k)).collect();
        secs.push(rib(0.56, 0.217, 0.0));
        secs.push(rib(0.025, 0.188, 0.0));
        loft!("basket", &secs, material = M.wicker);
        for y in [0.04, 0.2, 0.4] {
            braid(
                &format!("band{}", float_repr(y)),
                0.2 + 0.031 * y / 0.56 + 0.004,
                y,
                z,
                M.wicker2,
                48,
                0.009,
            );
        }
        braid("rim", 0.222, 0.552, z, M.wicker2, 48, 0.01);
        // the washing in it, near the top, and a sleeve hanging out under the lid
        for (i, (x, zz, y, m)) in [
            (-0.05, -0.04, 0.44, M.cloth1),
            (0.07, 0.05, 0.46, M.cloth2),
            (0.0, 0.07, 0.42, M.cloth3),
        ]
        .into_iter()
        .enumerate()
        {
            let c = fsphere!(
                F,
                format!("washing{i}"),
                0.13,
                (x, y, z + zz),
                m,
                segments = 12,
                rings = 6,
                scale = (1, 0.9, 0.4)
            );
            jitter!(c, 0.012, size = 0.06, seed = i as f64, up = 1.0);
        }
        fsweep!(
            F,
            "sleeve",
            &kit::fillet(
                &[
                    (0.03, 0.5, z + 0.17),
                    (0.03, 0.556, z + 0.222),
                    (0.03, 0.548, z + 0.234),
                    (0.035, 0.4, z + 0.231)
                ],
                0.012,
                3
            ),
            0,
            M.cloth1,
            profile = rounded_rect(0.07, 0.01, 0.004, 1),
            up = (1, 0, 0)
        );
    });
    join(&still, "laundryHamper", Some(P3::ZERO));
    // the lid, hinged at its back: it lifts while a resident sorts the washing; a domed woven top, a loop to lift it
    let parts = new_parts(|| {
        let secs: Vec<Vec<P3>> = [
            (0.226, 0.56),
            (0.236, 0.565),
            (0.236, 0.585),
            (0.226, 0.598),
            (0.15, 0.606),
            (0.02, 0.608),
        ]
        .iter()
        .map(|&(r, y)| ring!(r, r, z = y, verts = 32, at = (bx, by)))
        .collect();
        loft!("lid", &secs, material = M.wicker2);
        braid("lid_rim", 0.229, 0.578, z, M.wicker2, 40, 0.0065);
        fsweep!(
            F,
            "lid_loop",
            &[
                (-0.03, 0.604, z),
                (-0.022, 0.624, z),
                (0.0, 0.629, z),
                (0.022, 0.624, z),
                (0.03, 0.604, z)
            ],
            0.006,
            M.wicker2,
            verts = 6,
            resolution = 2
        );
    });
    // hinged 4 cm in from its back edge, near the loop's height: the back lifts off the rim and comes forward as it
    // opens (turned about the rim's bottom edge, its back went into the wall)
    let lid = hinged(join(&parts, "lid", None), F.at(0, 0.624, z - 0.197));
    let objs = settle!(centre = false);
    swing!(lid, "lid_open", -80.0, seconds = 0.7, axis = Axis::X); // not past upright: it stands at the wall
    objs
}

// --- the bed-end bench: a buttoned olive cushion on a slim brass frame ------------------------------------
/// The bed-end bench.
pub fn bench_bed_end() -> Vec<Obj> {
    let still = new_parts(|| {
        // each end one bent tube, down, across and down; rails along it, a stretcher low between the ends
        for sx in [-1, 1] {
            let x = sx as f64 * 0.74;
            fsweep!(
                F,
                format!("end{sx}"),
                &kit::fillet(
                    &[
                        (x, 0.004, -0.16),
                        (x, 0.335, -0.16),
                        (x, 0.335, 0.16),
                        (x, 0.004, 0.16)
                    ],
                    0.03,
                    3
                ),
                0.012,
                M.brass,
                verts = 10
            );
            fsweep!(
                F,
                format!("end_low{sx}"),
                &[(x, 0.09, -0.16), (x, 0.09, 0.16)],
                0.007,
                M.brass,
                verts = 8
            );
            for sz in [-1, 1] {
                fcyl!(
                    F,
                    format!("glide{sx}{sz}"),
                    0.014,
                    0.006,
                    (x, 0.003, sz as f64 * 0.16),
                    M.brass,
                    verts = 10
                );
            }
        }
        for sz in [-1, 1] {
            let z = sz as f64 * 0.16;
            fsweep!(
                F,
                format!("rail{sz}"),
                &[(-0.74, 0.335, z), (0.74, 0.335, z)],
                0.01,
                M.brass,
                verts = 10
            );
        }
        fsweep!(
            F,
            "stretcher",
            &[(-0.74, 0.09, 0), (0.74, 0.09, 0)],
            0.007,
            M.brass,
            verts = 8
        );
        // the cushion: piped, buttoned in two staggered rows, puffed between (its top, the seat, at 0.47)
        soft!(
            F,
            "cushion",
            1.6,
            0.105,
            0.42,
            (0, 0.4075, 0),
            M.olive,
            round = 0.035,
            puff = 0.01,
            flat = 5,
            piping = 0.007
        );
        for (row, z) in [-0.09, 0.09].into_iter().enumerate() {
            for i in 0..6 - row {
                let x = -0.625 + 0.25 * i as f64 + 0.125 * row as f64;
                fsphere!(
                    F,
                    format!("button{row}{i}"),
                    0.014,
                    (x, 0.462, z),
                    M.olive2,
                    segments = 8,
                    rings = 5,
                    scale = (1, 1, 0.5)
                );
            }
        }
    });
    join(&still, "benchBedEnd", Some(P3::ZERO));
    settle!(centre = false)
}
