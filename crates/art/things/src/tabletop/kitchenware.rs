//! TT1 kitchenware.

use super::M;
use super::common::{arc_shell, done, oval, tilt_x};
use artkit::forms::{Plane, by, rounded_rect, smooth_outline};
use artkit::json::float_repr;
use artkit::kit::{Pull, fillet, pull};
use artkit::materials::Mat;
use artkit::math::{Axis, PI, TAU, cos, cosd, radians, sin, sind};
use artkit::parts::{hinged, join, new_parts};
use artkit::place::GameFrame;
use artkit::scene::{Obj, meshes};
use artkit::{
    cone, cylinder, grow, join, rock, rod, settle, slide, soft_box, spin, star, sweep, swing,
};

/// A cordless jug kettle on its power base: a body swelling to its shoulder, a beak spout, a D handle in one
/// piece with its switch, a water window; the lid a part of its own, hinged on the handle's side: it flips up while
/// the kettle is filled for tea (`lid_open`).
pub fn kettle() -> Vec<Obj> {
    let body = new_parts(|| {
        turned!(
            "base",
            &[(0.083, 0.0), (0.086, 0.004), (0.086, 0.013), (0.08, 0.018)],
            M.dark,
            verts = 16
        );
        turned!(
            "body",
            &[
                (0.074, 0.016),
                (0.079, 0.026),
                (0.077, 0.1),
                (0.068, 0.178),
                (0.054, 0.203),
                (0.049, 0.211)
            ],
            M.kettle,
            verts = 16
        );
        // the spout: a beak drawn out of the shoulder, flattening and narrowing to its lip
        bar!(
            "spout",
            &[
                (-0.045, 0.0, 0.172),
                (-0.078, 0.0, 0.19),
                (-0.1, 0.0, 0.205),
                (-0.108, 0.0, 0.212)
            ],
            oval(0.03, 0.044, 8),
            M.kettle,
            scale = [1.0, 0.8, 0.55, 0.45],
            resolution = 2
        );
        // the handle in one piece from the shoulder round to the hip, the switch at its foot
        bar!(
            "handle",
            &fillet(
                &[
                    (0.045, 0.0, 0.198),
                    (0.1, 0.0, 0.198),
                    (0.104, 0.0, 0.075),
                    (0.06, 0.0, 0.068)
                ],
                0.024,
                2
            ),
            oval(0.017, 0.026, 6),
            M.dark,
            up = (0, 1, 0)
        );
        box_!("switch", (0.016, 0.014, 0.008), (0.09, 0, 0.08), M.red);
        slab!(
            "window",
            &rounded_rect(0.02, 0.1, 0.009, 2),
            0.006,
            (0.043, -0.062, 0.1),
            M.glass_pale,
            rot = (0, 0, 35)
        );
    });
    join!(&body, "kettle", origin = (0, 0, 0));
    let lid = new_parts(|| {
        turned!(
            "lid",
            &[
                (0.049, 0.209),
                (0.05, 0.214),
                (0.046, 0.221),
                (0.03, 0.225),
                (0.0, 0.226)
            ],
            M.dark,
            verts = 14
        );
        turned!(
            "lid_knob",
            &[(0.009, 0.224), (0.011, 0.231), (0.009, 0.236), (0.0, 0.238)],
            M.dark,
            at = (-0.014, 0, 0),
            verts = 8
        );
    });
    let lid = hinged(join(&lid, "lid", None), (0.046, 0.0, 0.223));
    settle!();
    swing!(lid, "lid_open", 80, seconds = 0.4, axis = Axis::Y); // its spout side up, about the hinge by the handle
    meshes()
}

/// A slice of bread standing in a slot, facing the front: a crust round a paler crumb face, its top two soft
/// shoulders (only the top shows).
fn _slice(name: &str, at: (f64, f64, f64), material: Mat, inner: Mat, w: f64, h: f64) {
    let pts = [
        (-w / 2.0, -h / 2.0),
        (w / 2.0, -h / 2.0),
        (w / 2.0, h * 0.16),
        (w * 0.47, h * 0.4),
        (w * 0.33, h / 2.0),
        (0.0, h * 0.44),
        (-w * 0.33, h / 2.0),
        (-w * 0.47, h * 0.4),
        (-w / 2.0, h * 0.16),
    ];
    slab!(name, &pts, 0.014, at, material);
    let crumb: Vec<(f64, f64)> = pts
        .iter()
        .map(|&(u, v)| (u * 0.88, v * 0.88 - h * 0.03))
        .collect();
    slab!(format!("{name}_crumb"), &crumb, 0.0146, at, inner);
}

/// A two-slice toaster: a rounded body on a dark plinth, two slots with a slice of toast up in each (a crust round
/// its crumb), the lever in its guide, a browning dial and a cancel button. The toast and the lever are a part of
/// their own: `toast_down` presses them down into the slots while the toast is made, and they pop up as it ends.
pub fn toaster() -> Vec<Obj> {
    let body = new_parts(_toaster_body);
    join!(&body, "toaster", origin = (0, 0, 0));
    let carriage = new_parts(|| {
        for s in [-1, 1] {
            _slice(
                &format!("toast{s}"),
                (0.0, s as f64 * 0.035, 0.192),
                M.bread,
                M.crumb,
                0.13,
                0.075,
            );
        }
        box_!(
            "lever",
            (0.022, 0.03, 0.014),
            (-0.14, 0, 0.125),
            M.dark,
            bevel = 0.004
        );
    });
    let toast = join!(&carriage, "toast", origin = (0, 0, 0.125));
    settle!();
    slide!(toast, "toast_down", (0, 0, -0.058), seconds = 0.4); // the toast's tops just under the slots, the lever down
    meshes()
}

fn _toaster_body() {
    turned!(
        "plinth",
        &[
            (0.122, 0.068, 0.0),
            (0.125, 0.071, 0.004),
            (0.125, 0.071, 0.013)
        ],
        M.dark,
        verts = 16,
        square = 6
    );
    turned!(
        "body",
        &[
            (0.125, 0.07, 0.012),
            (0.13, 0.075, 0.022),
            (0.13, 0.075, 0.15),
            (0.125, 0.07, 0.166),
            (0.114, 0.06, 0.172)
        ],
        M.toaster,
        verts = 20,
        square = 5
    );
    for s in [-1, 1] {
        slab!(
            format!("slot{s}"),
            &rounded_rect(0.17, 0.024, 0.01, 2),
            0.006,
            (0.0, s as f64 * 0.035, 0.171),
            M.dark,
            plane = Plane::XY
        );
    }
    box_!("guide", (0.004, 0.012, 0.1), (-0.13, 0, 0.1), M.dark);
    turned!(
        "knob",
        &[(0.016, 0.0), (0.016, 0.006), (0.012, 0.011), (0.0, 0.012)],
        M.chrome,
        at = (-0.129, 0.03, 0.055),
        rot = (0, -90, 0),
        verts = 12
    );
    box_!(
        "pointer",
        (0.004, 0.003, 0.012),
        (-0.1415, 0.03, 0.062),
        M.dark
    );
    box_!(
        "cancel",
        (0.006, 0.016, 0.01),
        (-0.132, -0.03, 0.055),
        M.chrome,
        bevel = 0.002
    );
    for s in [-1, 1] {
        box_!(
            format!("trim{s}"),
            (0.2, 0.002, 0.008),
            (0.0, s as f64 * 0.0755, 0.05),
            M.chrome
        );
    }
}

/// A jug blender: a tapering base with a dial and two buttons, the collar, a soft-square jug (the smoothie its
/// lower half, the jug drawn opaque), its handle in one piece, the lid and its filler cap. The jug (with its lid and
/// handle) is a part of its own, seated on the collar: `whirr` shakes it to and fro on the base while it blends.
pub fn blender() -> Vec<Obj> {
    let body = new_parts(_blender_base);
    join!(&body, "blender", origin = (0, 0, 0));
    let jar = new_parts(_blender_jar);
    let jug = join!(&jar, "jug", origin = (0, 0, 0.12));
    settle!();
    rock!(jug, "whirr", 2.5, seconds = 0.2, axis = Axis::Z); // a quick twist each way, as the motor bites
    meshes()
}

fn _blender_base() {
    turned!(
        "base",
        &[
            (0.068, 0.068, 0.0),
            (0.07, 0.07, 0.006),
            (0.066, 0.066, 0.085),
            (0.06, 0.06, 0.1)
        ],
        M.blender_base,
        verts = 16,
        square = 4
    );
    turned!(
        "dial",
        &[(0.016, 0.0), (0.016, 0.005), (0.012, 0.008), (0.0, 0.009)],
        M.chrome,
        at = (0, -0.063, 0.05),
        rot = (90, 0, 0),
        verts = 12
    );
    for x in [-0.038, 0.038] {
        box_!(
            format!("button{}", float_repr(x)),
            (0.016, 0.008, 0.01),
            (x, -0.065, 0.05),
            M.chrome,
            bevel = 0.002
        );
    }
    turned!(
        "collar",
        &[(0.056, 0.098), (0.06, 0.104), (0.06, 0.118), (0.056, 0.122)],
        M.dark,
        verts = 16
    );
}

#[allow(clippy::approx_constant)] // (the jug's 0.318 m, not 1 / pi)
fn _blender_jar() {
    turned!(
        "smoothie",
        &[(0.05, 0.05, 0.12), (0.058, 0.058, 0.22)],
        M.smoothie,
        verts = 12,
        square = 3
    );
    turned!(
        "jug",
        &[(0.058, 0.058, 0.22), (0.068, 0.068, 0.318)],
        M.jug,
        verts = 12,
        square = 3
    );
    turned!(
        "lid",
        &[
            (0.068, 0.068, 0.316),
            (0.072, 0.072, 0.322),
            (0.072, 0.072, 0.334),
            (0.064, 0.064, 0.34)
        ],
        M.dark,
        verts = 12,
        square = 3
    );
    turned!(
        "cap",
        &[(0.022, 0.338), (0.022, 0.35), (0.018, 0.355), (0.0, 0.356)],
        M.dark,
        verts = 10
    );
    bar!(
        "handle",
        &fillet(
            &[
                (-0.05, 0.0, 0.302),
                (-0.098, 0.0, 0.302),
                (-0.098, 0.0, 0.18),
                (-0.05, 0.0, 0.18)
            ],
            0.022,
            2
        ),
        oval(0.018, 0.026, 6),
        M.dark,
        up = (0, 1, 0)
    );
}

/// A countertop ice cream maker: a black motor head between silver cheeks, standing over a clear tub of churned
/// ice cream with a chunky black side handle, on a black collar and a silver plinth. The dasher is a part of its
/// own: it turns in the tub while the machine churns (`whirr`).
pub fn ice_cream_maker() -> Vec<Obj> {
    let body = new_parts(_ice_cream_maker_body);
    join!(&body, "iceCreamMaker", origin = (0, 0, 0));
    // the tub is glass seen through, a draw and so a part of its own; it never moves: the ice cream, rim and
    // handle stay with the body
    let tub = new_parts(_ice_cream_maker_tub);
    join!(&tub, "tub", origin = (0, 0, 0.08));
    let parts = new_parts(_ice_cream_maker_dasher);
    let dasher = join!(&parts, "dasher", origin = (0, 0, 0.285)); // about its shaft
    settle!();
    spin!(dasher, "whirr", turns = 1, seconds = 1.2, axis = Axis::Z);
    meshes()
}

fn _ice_cream_maker_body() {
    turned!(
        "foot",
        &[
            (0.104, 0.104, 0.0),
            (0.11, 0.11, 0.006),
            (0.11, 0.11, 0.016),
            (0.106, 0.106, 0.02)
        ],
        M.dark,
        verts = 24,
        square = 2
    );
    turned!(
        "plinth",
        &[
            (0.1, 0.1, 0.018),
            (0.112, 0.112, 0.03),
            (0.112, 0.112, 0.046),
            (0.104, 0.104, 0.052)
        ],
        M.steel,
        verts = 24,
        square = 2
    );
    turned!(
        "collar",
        &[
            (0.09, 0.09, 0.05),
            (0.094, 0.094, 0.058),
            (0.094, 0.094, 0.078),
            (0.09, 0.09, 0.082),
            (0.086, 0.086, 0.082)
        ],
        M.dark,
        verts = 20,
        square = 3
    );
    // the tub's furniture: the ice cream in the glass (its lower two thirds, rolled 45° over the top so the arris
    // doesn't read as a drawn edge through the glass), the black rim the head seats on, and the chunky handle bolted
    // to the collar below and the rim above, clear of the glass
    turned!(
        "ice_cream",
        &[
            (0.068, 0.068, 0.082),
            (0.07, 0.07, 0.188),
            (0.058, 0.058, 0.2)
        ],
        M.icecream,
        verts = 20,
        square = 2
    );
    turned!(
        "rim",
        &[
            (0.076, 0.076, 0.264),
            (0.08, 0.08, 0.268),
            (0.08, 0.08, 0.274),
            (0.076, 0.076, 0.278),
            (0.07, 0.07, 0.278)
        ],
        M.dark,
        verts = 20,
        square = 2
    );
    box_!(
        "handleTop",
        (0.031, 0.05, 0.022),
        (0.0925, 0.0, 0.266),
        M.dark,
        bevel = 0.005
    );
    box_!(
        "handleFoot",
        (0.031, 0.05, 0.022),
        (0.0925, 0.0, 0.062),
        M.dark,
        bevel = 0.005
    );
    box_!(
        "grip",
        (0.018, 0.05, 0.204),
        (0.114, 0.0, 0.164),
        M.dark,
        bevel = 0.005
    );
    turned!(
        "headBase",
        &[
            (0.08, 0.08, 0.276),
            (0.086, 0.086, 0.282),
            (0.086, 0.086, 0.296),
            (0.082, 0.082, 0.3),
            (0.078, 0.078, 0.3)
        ],
        M.dark,
        verts = 20,
        square = 3
    );
    turned!(
        "head",
        &[
            (0.082, 0.054, 0.3),
            (0.086, 0.056, 0.312),
            (0.086, 0.056, 0.458),
            (0.082, 0.054, 0.47)
        ],
        M.steel,
        verts = 24,
        square = 8
    );
    box_!(
        "face",
        (0.118, 0.012, 0.15),
        (0, -0.052, 0.386),
        M.dark,
        bevel = 0.005
    );
    box_!(
        "screen",
        (0.076, 0.004, 0.014),
        (0, -0.059, 0.432),
        M.chrome,
        bevel = 0.002
    );
    turned!(
        "power",
        &[(0.009, 0.0), (0.009, 0.004), (0.006, 0.005), (0.0, 0.006)],
        M.red,
        at = (0, -0.059, 0.336),
        rot = (90, 0, 0),
        verts = 12
    );
    // the program grid: two columns of three, as on the machine
    for (i, z) in [0.402, 0.383, 0.364].into_iter().enumerate() {
        for (j, x) in [-0.022, 0.022].into_iter().enumerate() {
            box_!(
                format!("program{i}{j}"),
                (0.016, 0.004, 0.01),
                (x, -0.059, z),
                M.chrome,
                bevel = 0.001
            );
        }
    }
    // a low domed lid, its open skirt buried in the head's top: a lip, then a shallow dome
    turned!(
        "cap",
        &[
            (0.08, 0.052, 0.466),
            (0.088, 0.058, 0.473),
            (0.0, 0.0, 0.4825)
        ],
        M.dark,
        verts = 20,
        square = 8,
        caps = false
    );
}

fn _ice_cream_maker_tub() {
    // glass seen through: its own draw call, so nothing else is in this part
    turned!(
        "tub",
        &[(0.072, 0.072, 0.08), (0.076, 0.076, 0.268)],
        M.glass,
        verts = 20,
        square = 2,
        caps = false
    );
}

fn _ice_cream_maker_dasher() {
    rod!(
        "shaft",
        (0.0, 0.0, 0.285),
        (0.0, 0.0, 0.1),
        0.004,
        material = M.steel,
        verts = 6
    );
    // the blade's upper half stands above the ice cream, so the paddle shows through the glass
    box_!(
        "blade",
        (0.055, 0.006, 0.1),
        (0.0375, 0.0, 0.195),
        M.steel,
        bevel = 0.001
    );
}

/// A tilt-head stand mixer: a rounded base, the column rising into the head (lofted, swelling over the bowl to
/// its nose, a chrome band and hub there), the speed lever, the steel bowl with batter in it and the beater's
/// shaft. The beater on its shaft is a part of its own: `beat` turns it round in the bowl while the mixer runs.
pub fn stand_mixer() -> Vec<Obj> {
    let body = new_parts(_mixer_body);
    join!(&body, "standMixer", origin = (0, 0, 0));
    let parts = new_parts(|| {
        rod!(
            "shaft",
            (-0.07, 0, 0.232),
            (-0.07, 0, 0.125),
            0.006,
            material = M.steel,
            verts = 6
        );
        slab!(
            "beater",
            &[
                (-0.03, 0.03),
                (0.03, 0.03),
                (0.022, -0.01),
                (0.0, -0.03),
                (-0.022, -0.01)
            ],
            0.005,
            (-0.07, 0, 0.11),
            M.steel,
            holes = vec![vec![
                (-0.016, 0.022),
                (0.016, 0.022),
                (0.01, -0.004),
                (0.0, -0.016),
                (-0.01, -0.004)
            ]]
        );
        // a swirl of the batter round the shaft, turned with it, so the turning shows in the bowl
        let mut pts = Vec::new();
        for i in 0..13 {
            let f = i as f64 / 12.0;
            let a = TAU * 1.25 * f;
            let r = 0.012 + 0.05 * f;
            pts.push((-0.07 + r * cos(a), r * sin(a), 0.1305 - 0.004 * f * f));
        }
        sweep!(
            "swirl",
            &pts,
            0.0,
            material = M.bread,
            profile = [
                (-0.002, -0.003),
                (0.0008, -0.002),
                (0.0008, 0.002),
                (-0.002, 0.003)
            ],
            scale = by(|f| 1.0 - 0.5 * f)
        );
    });
    let beater = join!(&parts, "beater", origin = (-0.07, 0, 0.232)); // about its shaft
    settle!();
    spin!(beater, "beat", turns = 1, seconds = 0.75, axis = Axis::Z);
    meshes()
}

fn _mixer_body() {
    turned!(
        "base",
        &[
            (0.15, 0.085, 0.0),
            (0.15, 0.085, 0.028),
            (0.142, 0.078, 0.04)
        ],
        M.mixer,
        verts = 16,
        square = 5
    );
    turned!(
        "column",
        &[
            (0.05, 0.056, 0.035),
            (0.042, 0.05, 0.14),
            (0.047, 0.056, 0.25)
        ],
        M.mixer,
        at = (0.1, 0, 0),
        verts = 12,
        square = 3
    );
    // the head lies along x, its back over the column and its nose over the bowl (rows: up, across, along)
    turned!(
        "head",
        &[
            (0.024, 0.026, 0.0),
            (0.05, 0.052, 0.012),
            (0.064, 0.066, 0.05),
            (0.066, 0.068, 0.14),
            (0.06, 0.062, 0.21),
            (0.047, 0.049, 0.255),
            (0.028, 0.03, 0.282),
            (0.0, 0.0, 0.29)
        ],
        M.mixer,
        at = (0.145, 0, 0.29),
        rot = (0, -90, 0),
        verts = 12
    );
    turned!(
        "band",
        &[(0.0545, 0.0565, 0.226), (0.0505, 0.0525, 0.25)],
        M.chrome,
        at = (0.145, 0, 0.29),
        rot = (0, -90, 0),
        verts = 12,
        caps = false
    );
    turned!(
        "hub",
        &[(0.016, 0.0), (0.016, 0.004), (0.0, 0.006)],
        M.chrome,
        at = (-0.143, 0, 0.29),
        rot = (0, -90, 0),
        verts = 10
    );
    turned!(
        "speed",
        &[(0.009, 0.0), (0.008, 0.012), (0.0, 0.014)],
        M.chrome,
        at = (0.1, -0.052, 0.2),
        rot = (90, 0, 0),
        verts = 8
    );
    turned!(
        "bowl",
        &[
            (0.042, 0.04),
            (0.06, 0.052),
            (0.074, 0.075),
            (0.08, 0.12),
            (0.082, 0.148),
            (0.078, 0.152),
            (0.075, 0.122)
        ],
        M.steel,
        at = (-0.07, 0, 0),
        verts = 14
    );
    turned!(
        "batter",
        &[(0.0752, 0.12), (0.05, 0.128), (0.0, 0.132)],
        M.crumb,
        at = (-0.07, 0, 0),
        verts = 10
    );
}

/// A small espresso machine: a brushed-steel body rounded over its top, the head over the cup with its buttons
/// and light, the group and portafilter (a black handle on a steel basket, two spouts), a slotted grate over the drip
/// tray, a pressure gauge in a chrome bezel, the steam knob and a bent wand, the water tank at the back, a rail round
/// the cup warmer and a cup upturned on it. Two thin streams of coffee from the spouts into the cup are a part of their
/// own, built full length and flattened into the spouts' tips: `brew` runs them down into the cup.
pub fn espresso_machine_small() -> Vec<Obj> {
    let body = new_parts(_espresso_body);
    join!(&body, "espressoMachineSmall", origin = (0, 0, 0));
    let parts = new_parts(|| {
        for x in [-0.009, 0.009] {
            cylinder!(
                format!("stream{}", float_repr(x)),
                0.0022,
                0.071,
                at = (x, -0.105, 0.1015),
                material = M.coffee,
                verts = 6
            );
        }
    });
    let stream = join!(&parts, "coffee", origin = (0, -0.105, 0.137)); // the spouts' tips
    stream.set_scale_axis(2, 0.001); // hidden in the spouts at rest
    settle!();
    grow!(stream, "brew", seconds = 0.6);
    meshes()
}

fn _espresso_body() {
    turned!(
        "body",
        &[
            (0.11, 0.096, 0.0),
            (0.11, 0.096, 0.255),
            (0.104, 0.09, 0.267),
            (0.094, 0.08, 0.27)
        ],
        M.espresso,
        at = (0, 0.026, 0),
        verts = 20,
        square = 8
    );
    slab!(
        "head",
        &rounded_rect(0.085, 0.07, 0.022, 2),
        0.2,
        (0, -0.0925, 0.235),
        M.espresso,
        plane = Plane::YZ,
        bevel = 0.006
    );
    for (i, x) in [-0.06, -0.035].into_iter().enumerate() {
        turned!(
            format!("button{i}"),
            &[(0.008, 0.0), (0.008, 0.004), (0.0, 0.006)],
            M.chrome,
            at = (x, -0.134, 0.248),
            rot = (90, 0, 0),
            verts = 10
        );
    }
    turned!(
        "light",
        &[(0.004, 0.0), (0.0, 0.003)],
        M.red,
        at = (0.055, -0.134, 0.248),
        rot = (90, 0, 0),
        verts = 6
    );
    box_!(
        "tank",
        (0.18, 0.05, 0.22),
        (0, 0.145, 0.12),
        M.glass_pale,
        bevel = 0.008
    );
    box_!(
        "tank_lid",
        (0.184, 0.054, 0.012),
        (0, 0.145, 0.236),
        M.dark,
        bevel = 0.004
    );
    box_!(
        "drip_tray",
        (0.18, 0.13, 0.02),
        (0, -0.13, 0.01),
        M.dark,
        bevel = 0.005
    );
    let slots: Vec<Vec<(f64, f64)>> = (0..7)
        .map(|_| rounded_rect(0.008, 0.08, 0.003, 1))
        .collect();
    let slots: Vec<Vec<(f64, f64)>> = slots
        .iter()
        .enumerate()
        .map(|(i, s)| {
            s.iter()
                .map(|&(u, v)| (u + (i as i64 - 3) as f64 * 0.02, v))
                .collect()
        })
        .collect();
    slab!(
        "grate",
        &rounded_rect(0.16, 0.11, 0.008, 1),
        0.004,
        (0, -0.13, 0.021),
        M.steel,
        plane = Plane::XY,
        holes = slots
    );
    turned!(
        "group",
        &[(0.036, 0.17), (0.038, 0.19), (0.034, 0.2)],
        M.dark,
        at = (0, -0.105, 0),
        verts = 14
    );
    turned!(
        "basket",
        &[
            (0.03, 0.15),
            (0.034, 0.156),
            (0.034, 0.172),
            (0.038, 0.174),
            (0.038, 0.18)
        ],
        M.steel,
        at = (0, -0.105, 0),
        verts = 14
    );
    bar!(
        "portafilter",
        &[
            (0.0, -0.135, 0.172),
            (0.0, -0.18, 0.168),
            (0.0, -0.222, 0.162)
        ],
        oval(0.02, 0.024, 8),
        M.dark,
        scale = [0.8, 1.0, 1.12]
    );
    for x in [-0.009, 0.009] {
        cone!(
            format!("spout{}", float_repr(x)),
            0.005,
            0.003,
            0.014,
            at = (x, -0.105, 0.144),
            material = M.steel,
            verts = 6
        );
    }
    // the cup on the grate, its handle to the side
    turned!(
        "cup",
        &[
            (0.018, 0.023),
            (0.024, 0.026),
            (0.031, 0.066),
            (0.032, 0.07),
            (0.029, 0.07),
            (0.026, 0.045),
            (0.0, 0.04)
        ],
        M.porcelain,
        at = (0, -0.105, 0),
        verts = 12
    );
    super::common::disc("crema", 0.0285, (0.0, -0.105, 0.064), M.coffee, 12, None);
    bar!(
        "cup_handle",
        &fillet(
            &[
                (0.028, -0.105, 0.06),
                (0.044, -0.105, 0.058),
                (0.044, -0.105, 0.04),
                (0.027, -0.105, 0.036)
            ],
            0.008,
            1
        ),
        oval(0.005, 0.008, 4),
        M.porcelain,
        up = (0, 1, 0)
    );
    // the gauge on the body's front, over the tray
    turned!(
        "gauge",
        &[(0.026, 0.0), (0.026, 0.006), (0.02, 0.008)],
        M.chrome,
        at = (0.062, -0.07, 0.13),
        rot = (90, 0, 0),
        verts = 16
    );
    turned!(
        "gauge_face",
        &[(0.0195, 0.004), (0.0195, 0.0075)],
        M.paper,
        at = (0.062, -0.07, 0.13),
        rot = (90, 0, 0),
        verts = 16
    );
    box_!(
        "needle",
        (0.002, 0.002, 0.016),
        (0.062 - 0.004, -0.0782, 0.136),
        M.red,
        rot = (0, -30, 0)
    );
    turned!(
        "steam_knob",
        &[(0.016, 0.0), (0.016, 0.006), (0.013, 0.011), (0.0, 0.012)],
        M.dark,
        at = (-0.109, -0.03, 0.15),
        rot = (0, -90, 0),
        verts = 10
    );
    sweep!(
        "steam_wand",
        &fillet(
            &[
                (-0.09, -0.07, 0.215),
                (-0.1, -0.085, 0.215),
                (-0.1, -0.09, 0.09)
            ],
            0.015,
            2
        ),
        0.005,
        material = M.chrome,
        verts = 6
    );
    cone!(
        "steam_tip",
        0.005,
        0.0035,
        0.012,
        at = (-0.1, -0.09, 0.084),
        material = M.chrome,
        verts = 6
    );
    // the warmer's rail on four posts, an upturned cup inside it
    let rail: Vec<(f64, f64, f64)> = rounded_rect(0.17, 0.15, 0.025, 2)
        .into_iter()
        .map(|(x, y)| (x, 0.026 + y, 0.288))
        .collect();
    sweep!(
        "rail",
        &rail,
        0.0035,
        material = M.chrome,
        verts = 6,
        closed = true
    );
    for (i, &(x, y, _)) in rail.iter().skip(1).step_by(3).enumerate() {
        rod!(
            format!("post{i}"),
            (x, y, 0.268),
            (x, y, 0.288),
            0.003,
            material = M.chrome,
            verts = 6
        );
    }
    turned!(
        "top_cup",
        &[
            (0.024, 0.27),
            (0.027, 0.272),
            (0.021, 0.304),
            (0.012, 0.31),
            (0.0, 0.31)
        ],
        M.porcelain,
        at = (0.04, 0.06, 0),
        verts = 12
    );
}

/// A wooden knife block leaning back on its foot, its edges eased; six knives in it, each a riveted black handle
/// on a steel bolster, sized to its blade.
pub fn knife_block() -> Vec<Obj> {
    let (tilt, zc) = (-18.0, 0.123);
    box_!(
        "foot",
        (0.12, 0.2, 0.05),
        (0, 0, 0.025),
        M.knife_block,
        bevel = 0.006
    );
    let outline: Vec<(f64, f64)> = rounded_rect(0.18, 0.2, 0.014, 2)
        .into_iter()
        .map(|(u, v)| tilt_x(u, v, tilt))
        .collect();
    let block: Vec<(f64, f64)> = outline.iter().map(|&(y, z)| (y, z + zc)).collect();
    slab!(
        "block",
        &block,
        0.12,
        (0, 0, 0),
        M.knife_block,
        plane = Plane::YZ,
        bevel = 0.004
    );
    for (i, (x, y)) in [
        (-0.03, -0.05),
        (0.0, -0.05),
        (0.03, -0.05),
        (-0.022, 0.02),
        (0.022, 0.02),
        (0.0, 0.06),
    ]
    .into_iter()
    .enumerate()
    {
        let w = if i < 3 { 0.016 } else { 0.021 };
        let mut pts = Vec::new();
        for z in [0.098, 0.106, 0.15, 0.164] {
            let (dy, dz) = tilt_x(y, z, tilt);
            pts.push((x, dy, zc + dz));
        }
        let (dy, dz) = tilt_x(y, 0.104, tilt);
        box_!(
            format!("bolster{i}"),
            (w + 0.002, 0.024, 0.008),
            (x, dy, zc + dz),
            M.steel,
            rot = (tilt, 0.0, 0.0)
        );
        bar!(
            format!("handle{i}"),
            &pts,
            rounded_rect(0.024, w, 0.006, 1),
            M.dark,
            up = (0, -1, 0),
            scale = [0.95, 1.0, 1.08, 1.0]
        );
    }
    done("knifeBlock", false)
}

/// A glazed crock with a rolled lip and a band, holding a wooden spoon, a slotted spatula, a balloon whisk and a
/// ladle.
pub fn utensil_crock() -> Vec<Obj> {
    turned!(
        "crock",
        &[
            (0.056, 0.0),
            (0.06, 0.02),
            (0.058, 0.11),
            (0.06, 0.135),
            (0.062, 0.147),
            (0.0595, 0.152),
            (0.054, 0.15),
            (0.052, 0.12)
        ],
        M.crock,
        verts = 14
    );
    turned!(
        "band",
        &[(0.0597, 0.072), (0.0592, 0.094)],
        M.crock_band,
        verts = 14,
        caps = false
    );
    // the wooden spoon: one flat stem swelling into its bowl
    bar!(
        "spoon",
        &[
            (0.0, 0.0, 0.12),
            (0.038, -0.016, 0.24),
            (0.045, -0.018, 0.256),
            (0.053, -0.021, 0.284),
            (0.059, -0.023, 0.31)
        ],
        oval(0.006, 0.012, 6),
        M.wood,
        up = (0, -1, 0),
        scale = [1.0, 1.0, 0.9, 2.6, 0.8],
        resolution = 2,
        angle = 70
    );
    // the spatula: a black handle into a slotted head
    sweep!(
        "spatula",
        &[(0.0, 0.0, 0.12), (-0.033, 0.019, 0.25)],
        0.0055,
        material = M.dark,
        verts = 6
    );
    let slits: Vec<Vec<(f64, f64)>> = [-0.014, 0.0, 0.014]
        .into_iter()
        .map(|dx| {
            [
                (0.004, 0.02),
                (-0.004, 0.02),
                (-0.004, -0.02),
                (0.004, -0.02),
            ]
            .into_iter()
            .map(|(u, v)| (u + dx, v + 0.004))
            .collect()
        })
        .collect();
    slab!(
        "spatula_head",
        &rounded_rect(0.05, 0.065, 0.008, 1),
        0.004,
        (-0.04, 0.023, 0.282),
        M.dark,
        rot = (0, -9, -30),
        holes = slits
    );
    // the whisk: a steel handle and two wire loops across each other
    sweep!(
        "whisk",
        &[(0.0, 0.0, 0.12), (0.0, 0.03, 0.23)],
        0.0055,
        material = M.steel,
        verts = 6
    );
    for (k, phi) in [0.0, 90.0].into_iter().enumerate() {
        let (c, s) = (cosd(phi), sind(phi));
        let lp = [
            (0.0, 0.0),
            (0.014, 0.016),
            (0.022, 0.048),
            (0.014, 0.074),
            (0.0, 0.081),
            (-0.014, 0.074),
            (-0.022, 0.048),
            (-0.014, 0.016),
        ];
        let pts: Vec<(f64, f64, f64)> = lp
            .iter()
            .map(|&(u, v)| (u * c, 0.031 + u * s + v * 0.15, 0.228 + v))
            .collect();
        sweep!(
            format!("wire{k}"),
            &pts,
            0.0013,
            material = M.steel,
            verts = 4,
            closed = true
        );
    }
    // the ladle: a stem with a hook at its top, the bowl at its foot hidden; its bowl turned up at the top
    sweep!(
        "ladle",
        &[(0.0, 0.0, 0.12), (-0.02, -0.032, 0.245)],
        0.005,
        material = M.wood_dark,
        verts = 6
    );
    turned!(
        "ladle_bowl",
        &[
            (0.0, 0.0),
            (0.012, 0.003),
            (0.019, 0.011),
            (0.021, 0.02),
            (0.018, 0.021),
            (0.015, 0.011),
            (0.0, 0.007)
        ],
        M.wood_dark,
        at = (-0.023, -0.037, 0.242),
        rot = (-30, -15, 0),
        verts = 8
    );
    done("utensilCrock", false)
}

/// A wooden board with a hand hole, eased edges: a scored bloomer loaf, two vine tomatoes with their calyxes, two
/// slices cut, and a cook's knife (blade, bolster, black handle).
pub fn cutting_board() -> Vec<Obj> {
    let hole: Vec<(f64, f64)> = rounded_rect(0.05, 0.018, 0.009, 1)
        .into_iter()
        .map(|(u, v)| (u + 0.135, v))
        .collect();
    slab!(
        "board",
        &rounded_rect(0.34, 0.23, 0.03, 2),
        0.022,
        (0, 0, 0.011),
        M.board,
        plane = Plane::XY,
        bevel = 0.003,
        holes = vec![hole]
    );
    let (c, s) = (cosd(15.0), sind(15.0));
    turned!(
        "loaf",
        &[
            (0.018, 0.024, 0.0),
            (0.033, 0.043, 0.012),
            (0.04, 0.05, 0.04),
            (0.041, 0.052, 0.095),
            (0.04, 0.05, 0.15),
            (0.033, 0.043, 0.178),
            (0.018, 0.024, 0.19)
        ],
        M.bread,
        at = (0.06 - 0.095 * c, 0.02 - 0.095 * s, 0.063),
        rot = (0, 90, 15),
        verts = 10,
        square = 2.4
    );
    for (i, d) in [-0.042, 0.0, 0.042].into_iter().enumerate() {
        box_!(
            format!("score{i}"),
            (0.007, 0.046, 0.003),
            (0.06 + d * c, 0.02 + d * s, 0.1018),
            M.crumb,
            rot = (0, 0, 15 + 35)
        );
    }
    for (i, (x, y)) in [(-0.1, 0.05), (-0.125, -0.025)].into_iter().enumerate() {
        ball!(
            format!("tomato{i}"),
            0.03,
            (x, y, 0.047),
            M.red,
            scale = (1, 1, 0.85),
            seg = 8,
            rings = 5
        );
        star!(
            format!("calyx{i}"),
            (x, y, 0.071),
            0.014,
            5,
            M.leaf,
            inner = 0.35,
            depth = 0.003,
            cup = 0.2
        );
    }
    for (i, (x, y)) in [(-0.04, -0.07), (-0.012, -0.082)].into_iter().enumerate() {
        let z = 0.022 + i as f64 * 0.006;
        turned!(
            format!("slice{i}"),
            &[(0.027, z), (0.027, z + 0.007)],
            M.red,
            verts = 10,
            at = (x, y, 0)
        );
        super::common::disc(
            &format!("flesh{i}"),
            0.021,
            (x, y, z + 0.0072),
            M.tomato_flesh,
            8,
            None,
        );
    }
    // the knife, lying across the front, its handle to the right
    let a = -8.0;
    let (ca, sa) = (cosd(a), sind(a));
    let k = |u: f64, v: f64| (0.085 + u * ca - v * sa, -0.082 + u * sa + v * ca);
    let blade = [
        k(0.0, -0.011),
        k(-0.075, -0.011),
        k(-0.105, -0.003),
        k(-0.112, 0.0),
        k(-0.09, 0.007),
        k(0.0, 0.012),
    ];
    slab!(
        "blade",
        &blade,
        0.002,
        (0, 0, 0.023),
        M.steel,
        plane = Plane::XY
    );
    let (bx, by) = k(0.004, 0.0);
    box_!(
        "bolster",
        (0.008, 0.026, 0.012),
        (bx, by, 0.028),
        M.steel,
        rot = (0.0, 0.0, a)
    );
    let at = |(u, v): (f64, f64)| (u, v, 0.029);
    bar!(
        "knife_handle",
        &[at(k(0.008, 0.0)), at(k(0.04, 0.0)), at(k(0.068, 0.0))],
        rounded_rect(0.013, 0.018, 0.005, 1),
        M.dark,
        scale = [1.0, 1.08, 1.0]
    );
    done("cuttingBoard", false)
}

/// A roll-top bread bin: a painted body between two D-shaped wooden ends (their edges eased); the roll-top is a
/// part of its own, turning about the bin's axis, a wooden bar pull on it: it rolls back into the bin for a sandwich
/// (`roll_open`), showing a loaf on the bin's floor.
pub fn bread_bin() -> Vec<Obj> {
    let (axis, r) = (0.105, 0.095);
    let body = new_parts(|| {
        box_!(
            "base",
            (0.31, 0.19, axis),
            (0.0, 0.0, axis / 2.0),
            M.bread_bin,
            bevel = 0.008,
            segs = 1
        );
        let mut end = vec![(r + 0.006, 0.0)];
        end.extend((0..11).map(|i| {
            (
                (r + 0.006) * cos(PI * i as f64 / 10.0),
                axis + (r + 0.006) * sin(PI * i as f64 / 10.0),
            )
        }));
        end.push((-(r + 0.006), 0.0));
        for s in [-1, 1] {
            let pts: Vec<(f64, f64)> = end.iter().map(|&(u, v)| (-u, v)).collect();
            slab!(
                format!("end{s}"),
                &pts,
                0.014,
                (s as f64 * 0.163, 0.0, 0.0),
                M.wood,
                plane = Plane::YZ,
                bevel = 0.003
            );
        }
        box_!(
            "toe",
            (0.31, 0.012, 0.012),
            (0, -0.092, 0.006),
            M.wood,
            bevel = 0.003
        );
        // a loaf on the bin's floor (the base's top), under the roll-top: seen only when it is rolled back
        soft_box!(
            "loaf",
            (0.2, 0.1, 0.07),
            at = (-0.02, 0.0, axis + 0.035),
            material = M.bread,
            round = 0.028,
            flat = 1,
            round_segments = 1
        );
    });
    join!(&body, "bread_bin", origin = (0, 0, 0));
    let top = new_parts(|| {
        arc_shell(
            "roll_top",
            r,
            r - 0.008,
            0.312,
            0.0,
            180.0,
            (0.0, 0.0, axis),
            M.bread_bin,
            14,
        );
        let f = GameFrame::new(0.0);
        let a = radians(35.0);
        pull(
            &f,
            "roll_handle",
            0.0,
            axis + r * sin(a),
            r * cos(a) - 0.004,
            0.1,
            M.wood,
            Pull {
                r: 0.0065,
                out: 0.02,
                verts: 6,
                ..Pull::default()
            },
        );
    });
    let top = hinged(join(&top, "roll_top", None), (0.0, 0.0, axis));
    settle!();
    // rolled back over the top and down behind it, inside the bin (Blender -X lifts the front)
    swing!(top, "roll_open", -100, seconds = 0.6, axis = Axis::X);
    meshes()
}

/// A wooden spice rack: shaped ends, a back board and a round front rail, five glass jars (each the colour of its
/// spice) with paper labels and black screw lids.
pub fn spice_jars() -> Vec<Obj> {
    box_!(
        "tray",
        (0.3, 0.12, 0.012),
        (0, 0, 0.006),
        M.rack,
        bevel = 0.003,
        segs = 1
    );
    let end = smooth_outline(
        &[
            (-0.06, 0.0),
            (0.06, 0.0),
            (0.06, 0.07),
            (0.045, 0.076),
            (0.0, 0.05),
            (-0.045, 0.036),
            (-0.06, 0.03),
        ],
        1,
        true,
    );
    for s in [-1, 1] {
        let pts: Vec<(f64, f64)> = end.iter().map(|&(u, v)| (-u, v)).collect();
        slab!(
            format!("end{s}"),
            &pts,
            0.012,
            (s as f64 * 0.144, 0.0, 0.0),
            M.rack,
            plane = Plane::YZ
        );
    }
    box_!("back", (0.276, 0.008, 0.052), (0, 0.054, 0.036), M.rack);
    sweep!(
        "rail",
        &[(-0.144, -0.05, 0.03), (0.144, -0.05, 0.03)],
        0.004,
        material = M.rack,
        verts = 6
    );
    for (i, colour) in [M.spice_a, M.spice_b, M.spice_c, M.spice_d, M.spice_e]
        .into_iter()
        .enumerate()
    {
        let x = -0.11 + i as f64 * 0.055;
        turned!(
            format!("jar{i}"),
            &[(0.021, 0.012), (0.022, 0.08), (0.016, 0.092)],
            colour,
            at = (x, 0.0, 0.0),
            verts = 8
        );
        turned!(
            format!("label{i}"),
            &[(0.0226, 0.03), (0.0226, 0.06)],
            M.paper,
            at = (x, 0.0, 0.0),
            verts = 8,
            caps = false
        );
        turned!(
            format!("lid{i}"),
            &[(0.0175, 0.09), (0.0188, 0.093), (0.0188, 0.112)],
            M.dark,
            at = (x, 0.0, 0.0),
            verts = 8
        );
    }
    done("spiceJars", false)
}

/// A glass cookie jar with a shoulder and a neck, four cookies stacked inside it, a glazed lid with a turned
/// knob.
pub fn cookie_jar() -> Vec<Obj> {
    turned!(
        "jar",
        &[
            (0.072, 0.0),
            (0.079, 0.006),
            (0.08, 0.02),
            (0.08, 0.15),
            (0.068, 0.168),
            (0.064, 0.173),
            (0.064, 0.19)
        ],
        M.glass,
        verts = 16
    );
    for i in 0..4 {
        let a = (i * 72) as f64;
        let ring = if i != 0 { 0.022 } else { 0.0 };
        turned!(
            format!("cookie{i}"),
            &[(0.033, 0.0), (0.034, 0.005), (0.026, 0.011), (0.0, 0.012)],
            if i % 2 != 0 { M.cookie } else { M.cookie_dark },
            at = (ring * cosd(a), ring * sind(a), 0.008 + i as f64 * 0.03),
            rot = (12 * (-1_i32).pow(i as u32), 8, 0),
            verts = 8
        );
    }
    turned!(
        "lid",
        &[
            (0.062, 0.186),
            (0.069, 0.19),
            (0.07, 0.208),
            (0.066, 0.216),
            (0.05, 0.221),
            (0.0, 0.223)
        ],
        M.jar_lid,
        verts = 14
    );
    turned!(
        "knob",
        &[
            (0.01, 0.22),
            (0.009, 0.23),
            (0.02, 0.238),
            (0.017, 0.246),
            (0.0, 0.248)
        ],
        M.jar_lid,
        verts = 8
    );
    done("cookieJar", false)
}
