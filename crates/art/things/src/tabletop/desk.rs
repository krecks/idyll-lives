//! TT5 bath, desk and toys.

use super::M;
use super::common::{_place, disc, done, oval};
use artkit::forms::{Plane, rounded_rect, smooth_outline};
use artkit::json::float_repr;
use artkit::kit::fillet;
use artkit::kit::{Moulding, moulding};
use artkit::materials::Mat;
use artkit::math::{Axis, M4, V3, radians};
use artkit::parts::{hinged, join, new_parts};
use artkit::place::{Facing, GameFrame};
use artkit::scene::{Obj, meshes};
use artkit::{cylinder, join, rod, settle, soft_box, sweep, swing};

/// A bathroom set on a stone tray: a pump dispenser (its collar, the pump head and spout in one piece), a tumbler
/// with two toothbrushes (bristles on their heads), a bar of soap on drainer ridges.
pub fn soap_set() -> Vec<Obj> {
    slab!(
        "tray",
        &rounded_rect(0.26, 0.12, 0.02, 2),
        0.012,
        (0, 0, 0.006),
        M.soap_tray,
        plane = Plane::XY,
        bevel = 0.002
    );
    for i in 0..3 {
        box_!(
            format!("ridge{i}"),
            (0.006, 0.07, 0.004),
            (-0.11 + i as f64 * 0.02, 0.0, 0.013),
            M.soap_tray
        );
    }
    turned!(
        "dispenser",
        &[
            (0.028, 0.012),
            (0.031, 0.018),
            (0.031, 0.105),
            (0.026, 0.118),
            (0.016, 0.124)
        ],
        M.dispenser,
        at = (0.08, 0, 0),
        verts = 12
    );
    turned!(
        "pump_collar",
        &[(0.016, 0.122), (0.016, 0.132), (0.011, 0.135)],
        M.dark,
        at = (0.08, 0, 0),
        verts = 10
    );
    rod!(
        "pump",
        (0.08, 0, 0.134),
        (0.08, 0, 0.148),
        0.005,
        material = M.dark,
        verts = 6
    );
    bar!(
        "pump_head",
        &fillet(
            &[
                (0.08, 0.008, 0.154),
                (0.08, -0.008, 0.156),
                (0.08, -0.034, 0.154),
                (0.08, -0.036, 0.146)
            ],
            0.006,
            1
        ),
        rounded_rect(0.012, 0.01, 0.004, 1),
        M.dark,
        up = (1, 0, 0)
    );
    turned!(
        "tumbler",
        &[
            (0.026, 0.012),
            (0.03, 0.09),
            (0.027, 0.09),
            (0.024, 0.02),
            (0.0, 0.018)
        ],
        M.tumbler,
        at = (-0.02, 0, 0),
        verts = 12
    );
    for (i, (colour, top)) in [
        (M.blush, (-0.004, 0.012, 0.165)),
        (M.leaf_light, (-0.042, -0.012, 0.158)),
    ]
    .into_iter()
    .enumerate()
    {
        let base = V3::from((-0.02, 0.0, 0.03));
        let d = (V3::from(top) - base).normalized();
        bar!(
            format!("brush{i}"),
            &[base, base + d * 0.09, base + d * 0.105, V3::from(top)],
            oval(0.007, 0.01, 6),
            colour,
            up = (0, -1, 0),
            scale = [1.0, 0.75, 1.0, 1.0]
        );
        let bt = V3::from(top) - d * 0.012 + V3::from((0.0, -0.007, 0.0));
        box_!(
            format!("bristles{i}"),
            (0.008, 0.008, 0.02),
            bt,
            M.white,
            bevel = 0.002,
            segs = 1
        );
    }
    box_!(
        "soap",
        (0.07, 0.05, 0.022),
        (-0.09, 0, 0.026),
        M.soap,
        bevel = 0.008,
        segs = 1
    );
    done("soapSet", false)
}

/// Three folded towels stacked a little askew, each folded over on itself (two soft layers, the fold showing at the
/// front), with a woven band across its front.
pub fn towel_stack() -> Vec<Obj> {
    for (i, (colour, band, dy, turn)) in [
        (M.towel_a, M.towel_b, 0.0, 0.0),
        (M.towel_b, M.towel_a, -0.006, 3.0),
        (M.towel_c, M.towel_b, 0.004, -2.0),
    ]
    .into_iter()
    .enumerate()
    {
        let z = i as f64 * 0.05;
        let towel = new_parts(|| {
            for k in 0..2 {
                let k = k as f64;
                soft_box!(
                    format!("towel{i}_{k}"),
                    (0.26 - k * 0.004, 0.18 - k * 0.006, 0.0245),
                    at = (0.0, k * 0.003, 0.0125 + k * 0.024),
                    material = colour,
                    round = 0.011,
                    puff = 0.002,
                    round_segments = 1,
                    flat = 1
                );
            }
            box_!(
                format!("band{i}"),
                (0.2, 0.002, 0.007),
                (0, -0.0905, 0.03),
                band
            );
        });
        _place(
            &towel,
            M4::translation(V3::from((0.0, dy, z))) * M4::rotation(radians(turn), Axis::Z),
        );
    }
    done("towelStack", false)
}

/// A mirrored vanity tray with a brass gallery round it, three scent bottles: a faceted pink one with a brass
/// stopper, a round flacon on its edge with an atomizer bulb, and a tall blue one with a black cap.
pub fn perfume_tray() -> Vec<Obj> {
    slab!(
        "tray",
        &rounded_rect(0.268, 0.168, 0.004, 1),
        0.008,
        (0, 0, 0.004),
        M.perfume_tray,
        plane = Plane::XY
    );
    moulding(
        &GameFrame::new(0.0),
        "gallery",
        &[
            (0.138, 0.088),
            (-0.138, 0.088),
            (-0.138, -0.088),
            (0.138, -0.088),
        ],
        &[
            (0.0, 0.0),
            (0.0, 0.016),
            (0.003, 0.0175),
            (0.006, 0.016),
            (0.006, 0.0),
        ],
        M.brass,
        Moulding {
            at: 0.0,
            plane: Facing::Top,
            ..Moulding::default()
        },
    );
    turned!(
        "bottle_a",
        &[
            (0.03, 0.02, 0.008),
            (0.031, 0.021, 0.012),
            (0.031, 0.021, 0.094),
            (0.024, 0.017, 0.1),
            (0.01, 0.01, 0.103)
        ],
        M.perfume_a,
        at = (0.07, 0, 0),
        verts = 16,
        square = 6,
        angle = 30
    );
    rod!(
        "neck_a",
        (0.07, 0, 0.1),
        (0.07, 0, 0.112),
        0.008,
        material = M.brass,
        verts = 8
    );
    ball!(
        "cap_a",
        0.017,
        (0.07, 0, 0.126),
        M.brass,
        scale = (1, 0.75, 1),
        seg = 8,
        rings = 6
    );
    turned!(
        "bottle_b",
        &[
            (0.03, -0.012),
            (0.034, -0.007),
            (0.034, 0.007),
            (0.03, 0.012)
        ],
        M.perfume_b,
        at = (-0.02, -0.02, 0.044),
        rot = (90, 0, 0),
        verts = 14
    );
    rod!(
        "neck_b",
        (-0.02, -0.02, 0.076),
        (-0.02, -0.02, 0.09),
        0.007,
        material = M.brass,
        verts = 6
    );
    sweep!(
        "tube_b",
        &[
            (-0.02, -0.02, 0.088),
            (-0.03, -0.03, 0.096),
            (-0.045, -0.04, 0.084),
            (-0.05, -0.044, 0.07)
        ],
        0.0018,
        material = M.dark,
        verts = 4,
        resolution = 2
    );
    ball!(
        "bulb_b",
        0.011,
        (-0.052, -0.046, 0.062),
        M.dark,
        scale = (1, 1, 1.25),
        seg = 6,
        rings = 5
    );
    turned!(
        "bottle_c",
        &[(0.025, 0.008), (0.025, 0.074), (0.022, 0.08)],
        M.perfume_c,
        at = (-0.08, 0.03, 0),
        verts = 12
    );
    turned!(
        "cap_c",
        &[(0.015, 0.078), (0.015, 0.1), (0.012, 0.104)],
        M.dark,
        at = (-0.08, 0.03, 0),
        verts = 10
    );
    done("perfumeTray", false)
}

/// A pen standing in a pot from Blender point `a` up to `b`: its barrel, a cap with a clip; or a hexagonal pencil
/// sharpened to a point.
fn _pen(
    name: &str,
    a: (f64, f64, f64),
    b: (f64, f64, f64),
    material: Mat,
    cap: Option<Mat>,
    pencil: bool,
) {
    let (a, b) = (V3::from(a), V3::from(b));
    let d = (b - a).normalized();
    if pencil {
        sweep!(
            name,
            &[a, b - d * 0.014],
            0.0045,
            material = material,
            verts = 6
        );
        sweep!(
            format!("{name}_tip"),
            &[b - d * 0.014, b],
            0.0045,
            material = M.crumb,
            verts = 6,
            scale = [1.0, 0.15]
        );
        return;
    }
    sweep!(
        name,
        &[a, b - d * 0.03],
        0.0042,
        material = material,
        verts = 6
    );
    sweep!(
        format!("{name}_cap"),
        &[b - d * 0.032, b],
        0.0048,
        material = cap.unwrap_or(M.dark),
        verts = 6,
        scale = [1.0, 0.8]
    );
    let side = if d.z.abs() < 0.999 {
        d.cross(V3::from((0, 0, 1))).normalized()
    } else {
        V3::from((1, 0, 0))
    };
    rod!(
        format!("{name}_clip"),
        b - d * 0.026 + side.cross(d) * 0.005,
        b - d * 0.004 + side.cross(d) * 0.005,
        0.0012,
        material = M.chrome,
        verts = 3
    );
}

/// A desk organiser: a two-tier letter tray (shaped sides scooped at the front, a back, posts) with paper in each
/// tier, and a pen pot holding two pens with clipped caps and a sharpened pencil.
pub fn desk_organiser() -> Vec<Obj> {
    let side = smooth_outline(
        &[
            (-0.1, 0.0),
            (0.1, 0.0),
            (0.1, 0.03),
            (-0.035, 0.03),
            (-0.07, 0.013),
            (-0.1, 0.011),
        ],
        1,
        true,
    );
    for tier in 0..2 {
        let z = tier as f64 * 0.05;
        box_!(
            format!("shelf{tier}"),
            (0.16, 0.2, 0.006),
            (-0.06, 0.0, z + 0.003),
            M.organiser
        );
        box_!(
            format!("paper{tier}"),
            (0.14, 0.19, 0.008),
            (-0.06, 0.003, z + 0.01),
            M.paper
        );
        box_!(
            format!("sheet{tier}"),
            (0.138, 0.186, 0.0015),
            (-0.058, 0.0, z + 0.0148),
            M.paper,
            rot = (0, 0, 3 - tier * 5)
        );
        for s in [-1, 1] {
            let pts: Vec<(f64, f64)> = side.iter().map(|&(u, v)| (-u, v)).collect();
            slab!(
                format!("side{tier}{s}"),
                &pts,
                0.006,
                (-0.06 + s as f64 * 0.077, 0.0, z + 0.003),
                M.organiser,
                plane = Plane::YZ
            );
        }
        box_!(
            format!("back{tier}"),
            (0.16, 0.006, 0.03),
            (-0.06, 0.097, z + 0.018),
            M.organiser
        );
    }
    for s in [-1, 1] {
        box_!(
            format!("post{s}"),
            (0.006, 0.006, 0.05),
            (-0.06 + s as f64 * 0.077, -0.094, 0.04),
            M.organiser
        );
    }
    turned!(
        "pen_pot",
        &[(0.035, 0.0), (0.035, 0.1), (0.031, 0.1), (0.031, 0.09)],
        M.organiser,
        at = (0.09, 0.02, 0),
        verts = 12
    );
    _pen(
        "pen0",
        (0.09, 0.02, 0.01),
        (0.105, 0.02, 0.16),
        M.pen_red,
        None,
        false,
    );
    _pen(
        "pen1",
        (0.09, 0.02, 0.01),
        (0.078, 0.03, 0.158),
        M.pen_blue,
        None,
        false,
    );
    _pen(
        "pencil",
        (0.09, 0.02, 0.01),
        (0.09, 0.005, 0.16),
        M.pen_yellow,
        None,
        true,
    );
    done("deskOrganiser", false)
}

/// A closed laptop: an aluminium base with eased edges, a thumb notch at the front, ports on its side and a hinge
/// barrel at the back, a keyboard and a touchpad on its top; the lid a part of its own, hinged at its back edge, the
/// logo on it and the screen under it: it opens while the laptop is used (`lid_open`).
#[allow(clippy::approx_constant)] // (the lid's 0.318 m, not 1 / pi)
pub fn laptop_closed() -> Vec<Obj> {
    let body = new_parts(|| {
        slab!(
            "base",
            &rounded_rect(0.32, 0.22, 0.012, 2),
            0.011,
            (0, 0, 0.0055),
            M.laptop,
            plane = Plane::XY,
            bevel = 0.0025
        );
        cylinder!(
            "hinge",
            0.0055,
            0.26,
            at = (0, 0.106, 0.012),
            material = M.dark,
            verts = 8,
            rot = (0, 90, 0)
        );
        box_!("notch", (0.05, 0.003, 0.003), (0, -0.1095, 0.0115), M.dark);
        for (i, (y, w)) in [(-0.03, 0.012), (-0.005, 0.009), (0.02, 0.009)]
            .into_iter()
            .enumerate()
        {
            box_!(
                format!("port{i}"),
                (0.002, w, 0.004),
                (0.1601, y, 0.006),
                M.dark
            );
        }
        // the keyboard's well and the touchpad, hidden under the lid until it opens
        box_!(
            "keyboard",
            (0.27, 0.105, 0.0006),
            (0, 0.025, 0.0109),
            M.dark
        );
        box_!(
            "touchpad",
            (0.1, 0.06, 0.0004),
            (0, -0.068, 0.0109),
            M.charcoal
        );
    });
    join!(&body, "laptop", origin = (0, 0, 0));
    let lid = new_parts(|| {
        slab!(
            "lid",
            &rounded_rect(0.318, 0.218, 0.012, 2),
            0.008,
            (0, 0, 0.0155),
            M.laptop,
            plane = Plane::XY,
            bevel = 0.002
        );
        turned!(
            "logo",
            &[(0.015, 0.0195), (0.015, 0.0201)],
            M.logo,
            at = (0, 0.01, 0),
            verts = 12
        );
        box_!(
            "screen",
            (0.29, 0.18, 0.0005),
            (0, 0.006, 0.01175),
            M.laptop_screen
        ); // under the lid, facing the keyboard
    });
    let lid = hinged(join(&lid, "lid", None), (0.0, 0.109, 0.0115));
    settle!();
    swing!(lid, "lid_open", -105, seconds = 0.8, axis = Axis::X); // its front up and over, the screen to the front
    meshes()
}

/// A desk monitor on a stand (a rounded foot, a curved neck into the back of the housing), a slim bezel round the
/// dark screen with a chrome logo and a power light; a keyboard of separate keys on its rounded base, a mouse.
pub fn monitor() -> Vec<Obj> {
    slab!(
        "foot",
        &rounded_rect(0.2, 0.14, 0.04, 2),
        0.012,
        (0, 0.08, 0.006),
        M.charcoal,
        plane = Plane::XY,
        bevel = 0.003
    );
    bar!(
        "neck",
        &fillet(
            &[(0.0, 0.098, 0.01), (0.0, 0.112, 0.11), (0.0, 0.104, 0.22)],
            0.04,
            2
        ),
        rounded_rect(0.05, 0.022, 0.006, 1),
        M.charcoal,
        up = (1, 0, 0)
    );
    soft_box!(
        "housing",
        (0.46, 0.03, 0.25),
        at = (0, 0.098, 0.3),
        material = M.monitor,
        round = 0.012,
        puff = (0, 0.006, 0),
        round_segments = 1,
        flat = 2
    );
    let hole: Vec<(f64, f64)> = rounded_rect(0.52, 0.29, 0.002, 1)
        .into_iter()
        .map(|(u, v)| (u, v + 0.006))
        .collect();
    slab!(
        "bezel",
        &rounded_rect(0.54, 0.32, 0.008, 2),
        0.022,
        (0, 0.075, 0.3),
        M.monitor,
        bevel = 0.002,
        holes = vec![hole]
    );
    box_!("screen", (0.522, 0.004, 0.292), (0, 0.067, 0.306), M.screen);
    box_!(
        "chin_logo",
        (0.03, 0.002, 0.005),
        (0, 0.0635, 0.149),
        M.chrome
    );
    box_!(
        "power_light",
        (0.004, 0.002, 0.002),
        (0.24, 0.0635, 0.149),
        M.white
    );
    slab!(
        "keyboard",
        &rounded_rect(0.38, 0.12, 0.008, 2),
        0.012,
        (0, -0.114, 0.006), // (the set under 33 cm deep)
        M.monitor,
        plane = Plane::XY,
        bevel = 0.003
    );
    for row in 0..4 {
        for k in 0..13 {
            box_!(
                format!("key{row}_{k}"),
                (0.021, 0.02, 0.006),
                (
                    -0.153 + k as f64 * 0.0255,
                    -0.146 + row as f64 * 0.024,
                    0.015
                ),
                M.keys
            );
        }
    }
    for (x, w) in [
        (-0.153, 0.021),
        (-0.122, 0.03),
        (0.0, 0.15),
        (0.122, 0.03),
        (0.153, 0.021),
    ] {
        box_!(
            format!("key4_{}", float_repr(x)),
            (w, 0.018, 0.006),
            (x, -0.165, 0.015), // (on the board, not over its front edge)
            M.keys
        );
    }
    turned!(
        "mouse",
        &[
            (0.019, 0.032, 0.0),
            (0.0225, 0.0365, 0.008),
            (0.021, 0.034, 0.02),
            (0.013, 0.022, 0.028),
            (0.0, 0.0, 0.031)
        ],
        M.monitor,
        at = (0.25, -0.114, 0),
        verts = 12
    );
    box_!("wheel", (0.004, 0.01, 0.004), (0.25, -0.133, 0.029), M.keys);
    done("monitor", true)
}

/// A sitting teddy bear: a pear-shaped body with a pale tummy, a round head with a pale muzzle, nose and eyes,
/// cupped ears, arms hanging forward with round paws, legs out in front with pale foot pads, a red bow at the neck.
pub fn stuffed_toy() -> Vec<Obj> {
    turned!(
        "body",
        &[
            (0.045, 0.04, 0.0),
            (0.068, 0.062, 0.025),
            (0.075, 0.068, 0.07),
            (0.066, 0.06, 0.12),
            (0.042, 0.04, 0.152),
            (0.02, 0.02, 0.162)
        ],
        M.bear,
        verts = 10
    );
    ball!(
        "tummy",
        0.044,
        (0, -0.058, 0.072),
        M.bear_light,
        scale = (1, 0.32, 1.1),
        seg = 7,
        rings = 4
    );
    turned!(
        "head",
        &[
            (0.03, 0.03, 0.0),
            (0.055, 0.05, 0.015),
            (0.062, 0.058, 0.05),
            (0.055, 0.052, 0.085),
            (0.034, 0.032, 0.106),
            (0.0, 0.0, 0.114)
        ],
        M.bear,
        at = (0, -0.008, 0.152),
        verts = 10
    );
    ball!(
        "muzzle",
        0.026,
        (0, -0.062, 0.188),
        M.bear_light,
        scale = (1.1, 0.8, 0.8),
        seg = 8,
        rings = 5
    );
    ball!(
        "nose",
        0.009,
        (0, -0.083, 0.196),
        M.dark,
        scale = (1.3, 1, 0.9),
        seg = 5,
        rings = 3
    );
    for s in [-1, 1] {
        let k = s as f64;
        ball!(
            format!("eye{s}"),
            0.0065,
            (k * 0.023, -0.06, 0.216),
            M.dark,
            seg = 4,
            rings = 3
        );
        turned!(
            format!("ear{s}"),
            &[(0.023, 0.0), (0.02, 0.011), (0.0, 0.016)],
            M.bear,
            at = (k * 0.046, -0.002, 0.256),
            rot = (-90, 0, s * -15),
            verts = 8
        );
        disc(
            &format!("ear_in{s}"),
            0.014,
            (0.0, 0.0, 0.0),
            M.bear_light,
            6,
            None,
        )
        .set_matrix_world(
            M4::translation(V3::from((k * 0.046, -0.0026, 0.256)))
                * M4::rotation(radians((-s * 15) as f64), Axis::Z)
                * M4::rotation(radians(90.0), Axis::X),
        );
        sweep!(
            format!("arm{s}"),
            &[
                (k * 0.05, -0.008, 0.135),
                (k * 0.07, -0.03, 0.105),
                (k * 0.074, -0.055, 0.075)
            ],
            0.024,
            material = M.bear,
            verts = 6,
            scale = [0.8, 1.0, 0.95]
        );
        ball!(
            format!("paw{s}"),
            0.023,
            (k * 0.074, -0.057, 0.072),
            M.bear,
            seg = 6,
            rings = 4
        );
        sweep!(
            format!("leg{s}"),
            &[
                (k * 0.045, -0.02, 0.032),
                (k * 0.05, -0.08, 0.03),
                (k * 0.052, -0.126, 0.03)
            ],
            0.03,
            material = M.bear,
            verts = 7,
            scale = [0.9, 1.0, 1.0]
        );
        let pad = disc(
            &format!("pad{s}"),
            0.02,
            (0.0, 0.0, 0.0),
            M.bear_light,
            7,
            Some(0.017),
        );
        pad.set_matrix_world(
            M4::translation(V3::from((k * 0.052, -0.1265, 0.031)))
                * M4::rotation(radians(90.0), Axis::X),
        );
        ball!(
            format!("bow{s}"),
            0.016,
            (k * 0.018, -0.052, 0.152),
            M.red,
            scale = (1.3, 0.5, 0.9),
            rot = (0, s * 15, 0),
            seg = 5,
            rings = 3
        );
    }
    ball!("knot", 0.007, (0, -0.058, 0.152), M.red, seg = 5, rings = 3);
    done("stuffedToy", false)
}

/// A raised letter on a toy block: its outline and the holes in it.
/// An outline and the holes in it.
type Outline = (Vec<(f64, f64)>, Vec<Vec<(f64, f64)>>);

fn letter(c: char) -> Outline {
    match c {
        'A' => (
            vec![
                (-0.018, -0.02),
                (-0.009, -0.02),
                (-0.006, -0.012),
                (0.006, -0.012),
                (0.009, -0.02),
                (0.018, -0.02),
                (0.004, 0.02),
                (-0.004, 0.02),
            ],
            vec![vec![(-0.0035, -0.004), (0.0035, -0.004), (0.0, 0.008)]],
        ),
        'L' => (
            vec![
                (-0.014, -0.02),
                (0.014, -0.02),
                (0.014, -0.012),
                (-0.005, -0.012),
                (-0.005, 0.02),
                (-0.014, 0.02),
            ],
            vec![],
        ),
        'T' => (
            vec![
                (-0.004, -0.02),
                (0.004, -0.02),
                (0.004, 0.012),
                (0.016, 0.012),
                (0.016, 0.02),
                (-0.016, 0.02),
                (-0.016, 0.012),
                (-0.004, 0.012),
            ],
            vec![],
        ),
        'H' => (
            vec![
                (-0.016, -0.02),
                (-0.008, -0.02),
                (-0.008, -0.004),
                (0.008, -0.004),
                (0.008, -0.02),
                (0.016, -0.02),
                (0.016, 0.02),
                (0.008, 0.02),
                (0.008, 0.004),
                (-0.008, 0.004),
                (-0.008, 0.02),
                (-0.016, 0.02),
            ],
            vec![],
        ),
        'O' => (
            rounded_rect(0.032, 0.04, 0.013, 2),
            vec![rounded_rect(0.014, 0.022, 0.006, 2)],
        ),
        other => panic!("no letter {other}"),
    }
}

/// Wooden toy blocks, their edges rounded, a raised letter on two faces of each, and a triangular roof block on top
/// of the stack.
pub fn toy_blocks() -> Vec<Obj> {
    let b = 0.065;
    for (i, (x, y, z, colour, turn, letters)) in [
        (-0.05, 0.02, 0.0, M.block_a, 10.0, "AL"),
        (0.03, 0.03, 0.0, M.block_b, -15.0, "TH"),
        (-0.01, 0.02, 1.0, M.block_c, 30.0, "OA"),
        (0.08, -0.06, 0.0, M.block_d, 20.0, "HO"),
        (-0.07, -0.07, 0.0, M.block_e, 0.0, "LT"),
    ]
    .into_iter()
    .enumerate()
    {
        let block = new_parts(|| {
            let mut chars = letters.chars();
            box_!(
                format!("block{i}"),
                (b, b, b),
                (0.0, 0.0, b / 2.0),
                colour,
                bevel = 0.006,
                segs = 1
            );
            let (outline, holes) = letter(chars.next().expect("two letters"));
            slab!(
                format!("letter{i}a"),
                &outline,
                0.003,
                (0.0, -b / 2.0 - 0.0008, b / 2.0),
                M.white,
                holes = holes
            );
            let (outline, holes) = letter(chars.next().expect("two letters"));
            let flipped: Vec<(f64, f64)> = outline.iter().map(|&(u, v)| (-u, v)).collect();
            let holes: Vec<Vec<(f64, f64)>> = holes
                .iter()
                .map(|h| h.iter().map(|&(u, v)| (-u, v)).collect())
                .collect();
            slab!(
                format!("letter{i}b"),
                &flipped,
                0.003,
                (b / 2.0 + 0.0008, 0.0, b / 2.0),
                M.white,
                plane = Plane::YZ,
                holes = holes
            );
        });
        _place(
            &block,
            M4::translation(V3::from((x, y, z * b))) * M4::rotation(radians(turn), Axis::Z),
        );
    }
    // a roof on the top block: a triangular prism
    let roof = slab!(
        "roof",
        &[(-b / 2.0 - 0.003, 0.0), (b / 2.0 + 0.003, 0.0), (0.0, 0.07)],
        b,
        (0, 0, 0),
        M.block_d
    );
    roof.set_matrix_world(
        M4::translation(V3::from((-0.01, 0.02, 2.0 * b))) * M4::rotation(radians(30.0), Axis::Z),
    );
    done("toyBlocks", false)
}
