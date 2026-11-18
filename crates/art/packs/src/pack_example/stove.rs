//! The chef's stove (2x1): a six-burner range with a double oven, under a chimney hood.

use super::common::{M, WALL, cut_mouth};
use artkit::forms::rounded_rect;
use artkit::kit;
use artkit::math::{Axis, P3, TAU, cos, cosd, radians, sin, sind};
use artkit::parts::{hinged, join, new_parts};
use artkit::place::{Facing, GameFrame};
use artkit::scene::Obj;
use artkit::{
    fbox, fcone, fcyl, fextrude, fsweep, grow, loft, revolve, ring, settle, soft, swing, torus,
};

const HOB: f64 = 0.92; // the hob's top
const FRONT: f64 = 0.15; // the carcass's front face
const FLAME: (f64, f64) = (0.0, 0.0); // the front middle burner, lit by `burner_on`
const KNOBS: [f64; 5] = [-0.65, -0.45, 0.25, 0.45, 0.65]; // the still knobs' x; the one at -0.25 turns

fn burners() -> Vec<(f64, f64)> {
    let mut out = Vec::new();
    for z in [-0.27, 0.0] {
        for x in [-0.5, 0.0, 0.5] {
            out.push((x, z));
        }
    }
    out
}

/// A gas burner on the hob: a flared head with its flame ports, a black enamel cap on it.
fn _burner(f: &GameFrame, name: &str, x: f64, z: f64) {
    revolve!(
        name,
        &[
            (0.0, 0.0),
            (0.064, 0.0),
            (0.064, 0.005),
            (0.052, 0.012),
            (0.047, 0.02),
            (0.0, 0.02)
        ],
        at = f.at(x, HOB, z),
        material = M.grate,
        segments = 14
    );
    revolve!(
        format!("{name}_cap"),
        &[
            (0.0, 0.019),
            (0.04, 0.019),
            (0.04, 0.024),
            (0.03, 0.031),
            (0.0, 0.033)
        ],
        at = f.at(x, HOB, z),
        material = M.cap,
        segments = 12
    );
}

/// A cast-iron grate over a pair of burners (the back and front one at `x`): a mitred frame standing on four
/// feet, a bar across the middle and fingers reaching in towards each burner.
fn _grate(f: &GameFrame, name: &str, x: f64) {
    let (y, z0, z1, hw) = (HOB + 0.026, -0.4, 0.13, 0.24);
    let bar = [(0.0, 0.0), (0.016, 0.0), (0.016, 0.016), (0.0, 0.016)]; // (inset, up): a square bar
    kit::moulding!(
        f,
        name,
        &[(x - hw, z0), (x + hw, z0), (x + hw, z1), (x - hw, z1)],
        &bar,
        M.grate,
        at = y,
        plane = Facing::Top
    );
    let prof = vec![
        (-0.007, -0.008),
        (0.007, -0.008),
        (0.007, 0.008),
        (-0.007, 0.008),
    ]; // a square bar, as the frame's
    fsweep!(
        f,
        format!("{name}_mid"),
        &[
            (x - hw + 0.008, y + 0.008, -0.135),
            (x + hw - 0.008, y + 0.008, -0.135)
        ],
        0.0,
        M.grate,
        profile = &prof,
        up = (0, 0, 1)
    );
    for bz in [-0.27, 0.0] {
        // four fingers round each burner, dipping at their tips
        for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let reach = if dx != 0 { hw - 0.008 } else { 0.12 };
            let (dxf, dzf) = (dx as f64, dz as f64);
            let a = (x + dxf * reach, y + 0.008, bz + dzf * reach);
            let b = (x + dxf * 0.075, y + 0.006, bz + dzf * 0.075);
            fsweep!(
                f,
                format!("{name}_finger{bz}{dx}{dz}"),
                &[a, b],
                0.0,
                M.grate,
                profile = &prof,
                up = (0, 0, 1),
                scale = [1.0, 0.8]
            );
        }
    }
    for fx in [x - hw + 0.008, x + hw - 0.008] {
        for fz in [z0 + 0.008, z1 - 0.008] {
            fbox!(
                f,
                format!("{name}_foot{fx}{fz}"),
                0.014,
                0.026,
                0.014,
                (fx, HOB + 0.013, fz),
                M.grate
            );
        }
    }
}

/// A range knob on the control strip: a chrome skirt, a black grip with a domed face, a chrome pointer.
fn _knob(f: &GameFrame, name: &str, x: f64, y: f64) {
    revolve!(
        format!("{name}_skirt"),
        &[
            (0.0, 0.0),
            (0.031, 0.0),
            (0.031, 0.003),
            (0.027, 0.008),
            (0.0, 0.008)
        ],
        at = f.at(x, y, FRONT + 0.012),
        material = M.chrome,
        segments = 12,
        rot = (90, 0, 0)
    );
    revolve!(
        name,
        &[
            (0.0, 0.006),
            (0.022, 0.006),
            (0.024, 0.012),
            (0.024, 0.03),
            (0.02, 0.036),
            (0.0, 0.038)
        ],
        at = f.at(x, y, FRONT + 0.012),
        material = M.knob,
        segments = 12,
        rot = (90, 0, 0)
    );
    fbox!(
        f,
        format!("{name}_mark"),
        0.005,
        0.018,
        0.004,
        (x, y + 0.01, FRONT + 0.05),
        M.chrome
    );
}

/// An oven door on its bottom hinge at the oven mouth: a steel frame round a black window (the kit's shaker), a
/// bar pull across its top on two stand-offs (the kit's pull).
fn _oven_door(f: &GameFrame, name: &str, x: f64, w: f64) {
    kit::shaker!(
        f,
        name,
        w,
        0.54,
        (x, 0.41, FRONT + 0.0125),
        0.025,
        M.body,
        rail = 0.085,
        panel = M.window,
        recess = 0.007
    );
    kit::pull!(
        f,
        format!("{name}_pull"),
        x,
        0.63,
        FRONT + 0.025,
        w - 0.16,
        M.chrome,
        r = 0.012,
        out = 0.045
    );
}

/// The left oven's inside, seen with its door down: dark enamel walls, two wire racks, a loaf on a tray.
fn _oven(f: &GameFrame, x: f64) {
    let (w, h, d) = (0.62, 0.42, 0.34);
    let (yc, back) = (0.42, FRONT - 0.34);
    fbox!(f, "oven_back", w, h, 0.01, (x, yc, back), M.dark);
    for s in [-1, 1] {
        let sf = s as f64;
        fbox!(
            f,
            format!("oven_side{s}"),
            0.01,
            h,
            d,
            (x + sf * w / 2.0, yc, back + d / 2.0),
            M.dark
        );
        fbox!(
            f,
            format!("oven_lid{s}"),
            w,
            0.01,
            d,
            (x, yc + sf * h / 2.0, back + d / 2.0),
            M.dark
        );
    }
    for (k, y) in [0.3, 0.46].into_iter().enumerate() {
        // wire racks: a bent rim and rods front to back
        kit::tube!(
            f,
            format!("rack{k}"),
            &kit::fillet!(
                &[
                    (x - 0.29, y, back + 0.02),
                    (x - 0.29, y, FRONT - 0.03),
                    (x + 0.29, y, FRONT - 0.03),
                    (x + 0.29, y, back + 0.02),
                ],
                0.02
            ),
            0.004,
            M.chrome,
            verts = 4,
            round = 1
        );
        for i in 0..7 {
            let rx = x - 0.24 + i as f64 * 0.08;
            fsweep!(
                f,
                format!("rack{k}_rod{i}"),
                &[(rx, y, back + 0.02), (rx, y, FRONT - 0.03)],
                0.003,
                M.chrome,
                verts = 4
            );
        }
    }
    fbox!(
        f,
        "tray",
        0.4,
        0.015,
        0.26,
        (x, 0.315, -0.02),
        M.dark,
        bevel = 0.004,
        bevel_segments = 1
    );
    soft!(
        f,
        "loaf",
        0.26,
        0.09,
        0.14,
        (x, 0.365, -0.02),
        M.crust,
        round = 0.04,
        puff = (0, 0.015, 0),
        flat = 2,
        round_segments = 1
    );
}

/// The splashback: metro tiles in running bond over a grout board.
fn _tiles(f: &GameFrame) {
    fbox!(
        f,
        "grout",
        1.6,
        0.62,
        0.01,
        (0, HOB + 0.07 + 0.31, WALL + 0.005),
        M.grout
    );
    let (tw, th) = (0.15, 0.075);
    for r in 0..8 {
        let y = HOB + 0.07 + 0.004 + th / 2.0 + r as f64 * (th + 0.003);
        let mut x = -0.8 - (tw + 0.003) / 2.0 * (r % 2) as f64;
        while x < 0.8 {
            let (a, b) = (x.max(-0.797), (x + tw).min(0.797));
            if b - a > 0.02 {
                fbox!(
                    f,
                    format!("tile{r}_{x:.3}"),
                    b - a,
                    th,
                    0.008,
                    ((a + b) / 2.0, y, WALL + 0.014),
                    M.tile
                );
            }
            x += tw + 0.003;
        }
    }
}

/// The hood in one piece, lofted through rounded rectangles: its band, the canopy rising to the chimney and the
/// chimney's two telescoped sections up to the ceiling; under it a baffle filter and two lamps.
fn _hood(f: &GameFrame) {
    let ring = |w: f64, d: f64, zc: f64, y: f64| -> Vec<P3> {
        let c = f.at(0, 0, zc);
        ring!(
            w / 2.0,
            d / 2.0,
            z = y,
            verts = 24,
            square = 12,
            at = (c.x, c.y)
        )
    };
    loft!(
        "hood",
        &[
            ring(1.18, 0.48, WALL + 0.24, 1.556),
            ring(1.2, 0.5, WALL + 0.25, 1.564),
            ring(1.2, 0.5, WALL + 0.25, 1.68),
            ring(1.16, 0.48, WALL + 0.24, 1.7),
            ring(0.38, 0.29, WALL + 0.145, 2.04),
            ring(0.36, 0.28, WALL + 0.14, 2.06),
            ring(0.36, 0.28, WALL + 0.14, 2.26),
            ring(0.37, 0.285, WALL + 0.1425, 2.26),
            ring(0.37, 0.285, WALL + 0.1425, 2.55)
        ],
        material = M.body,
        smooth = false
    );
    fbox!(
        f,
        "hood_filter",
        1.12,
        0.01,
        0.44,
        (0, 1.552, WALL + 0.24),
        M.dark
    );
    for i in 0..9 {
        // the baffles' folds
        fbox!(
            f,
            format!("baffle{i}"),
            0.008,
            0.008,
            0.42,
            (-0.48 + i as f64 * 0.12, 1.547, WALL + 0.24),
            M.chrome
        );
    }
    for x in [-0.38, 0.38] {
        fcyl!(
            f,
            format!("hood_lamp{x}"),
            0.03,
            0.006,
            (x, 1.548, -0.05),
            M.egg,
            verts = 12
        );
    }
}

/// The utensil rail, bent in one piece out from the wall and back, with a ladle, a slotted spatula, a whisk and a
/// copper pan on S-hooks.
fn _rail(f: &GameFrame) {
    let (z, y) = (WALL + 0.06, 1.36);
    kit::tube!(
        f,
        "rail",
        &kit::fillet!(
            &[
                (-0.62, y, WALL + 0.002),
                (-0.62, y, z),
                (0.62, y, z),
                (0.62, y, WALL + 0.002),
            ],
            0.03
        ),
        0.008,
        M.chrome,
        verts = 8,
        round = 1
    );
    for x in [-0.62, 0.62] {
        fcyl!(
            f,
            format!("rail_rose{x}"),
            0.018,
            0.008,
            (x, y, WALL + 0.004),
            M.chrome,
            rot = (90, 0, 0),
            verts = 12
        );
    }
    for x in [-0.42, -0.28, -0.14, 0.32] {
        // S-hooks: over the rail, down and up into a hook
        fsweep!(
            f,
            format!("hook{x}"),
            &[
                (x, y + 0.004, z - 0.012),
                (x, y + 0.016, z - 0.004),
                (x, y + 0.014, z + 0.01),
                (x, y - 0.01, z + 0.012),
                (x, y - 0.04, z + 0.004),
                (x, y - 0.05, z - 0.006),
                (x, y - 0.042, z - 0.014)
            ],
            0.0025,
            M.chrome,
            verts = 4
        );
    }
    let hz = z - 0.004;
    // the ladle: a handle with a hooked end and a turned bowl
    fsweep!(
        f,
        "ladle_handle",
        &[
            (-0.42, y - 0.03, hz + 0.004),
            (-0.42, y - 0.06, hz + 0.004),
            (-0.42, 1.12, hz + 0.008),
            (-0.42, 1.08, hz + 0.022)
        ],
        0.0,
        M.copper,
        profile = rounded_rect(0.014, 0.004, 0.002, 1),
        up = (0, 1, 0),
        resolution = 2
    );
    revolve!(
        "ladle_bowl",
        &[
            (0.0, 0.0),
            (0.035, 0.008),
            (0.045, 0.03),
            (0.046, 0.04),
            (0.04, 0.04),
            (0.03, 0.016),
            (0.0, 0.008)
        ],
        at = f.at(-0.42, 1.04, hz + 0.065),
        material = M.copper,
        segments = 14,
        rot = (-80, 0, 0)
    );
    // the spatula: a handle and a blade with three slots
    fsweep!(
        f,
        "spatula_handle",
        &[(-0.28, y - 0.035, hz), (-0.28, 1.16, hz + 0.004)],
        0.0,
        M.body,
        profile = rounded_rect(0.016, 0.008, 0.003, 1),
        up = (0, 1, 0)
    );
    let holes: Vec<Vec<(f64, f64)>> = [-0.018, 0.0, 0.018]
        .iter()
        .map(|&dx| {
            rounded_rect(0.008, 0.055, 0.004, 1)
                .iter()
                .map(|&(u, v)| (u + dx, v - 0.01))
                .collect()
        })
        .collect();
    fextrude!(
        f,
        "spatula_blade",
        &rounded_rect(0.07, 0.1, 0.012, 2),
        0.003,
        (-0.28, 1.105, hz + 0.004),
        M.body,
        holes = holes
    );
    // the whisk: a handle and four wire loops, each in its own plane round the handle's axis
    fsweep!(
        f,
        "whisk_handle",
        &[(-0.14, y - 0.035, hz), (-0.14, 1.23, hz)],
        0.009,
        M.body,
        verts = 10
    );
    for k in 0..4 {
        let a = radians((45 * k) as f64);
        let mut pts = Vec::new();
        for i in 0..12 {
            let s = TAU * i as f64 / 12.0;
            let u = 0.03 * sin(s) * (0.35 + 0.65 * (1.0 - cos(s)) / 2.0);
            pts.push((
                -0.14 + u * cos(a),
                1.23 - 0.13 * (1.0 - cos(s)) / 2.0,
                hz + u * sin(a),
            ));
        }
        fsweep!(
            f,
            format!("whisk_wire{k}"),
            &pts,
            0.0018,
            M.body,
            verts = 4,
            closed = true
        );
    }
    // the copper pan hanging by its handle, its bottom to the room
    revolve!(
        "pan_hang",
        &[
            (0.0, 0.0),
            (0.1, 0.0),
            (0.112, 0.004),
            (0.118, 0.04),
            (0.112, 0.04),
            (0.104, 0.008),
            (0.0, 0.006)
        ],
        at = f.at(0.32, 1.13, WALL + 0.046),
        material = M.copper,
        segments = 20,
        rot = (-90, 0, 0)
    );
    fsweep!(
        f,
        "pan_hang_handle",
        &[(0.32, 1.24, WALL + 0.035), (0.32, y - 0.035, WALL + 0.05)],
        0.0,
        M.copper,
        profile = rounded_rect(0.02, 0.008, 0.003, 1),
        up = (0, 1, 0),
        scale = [1.0, 0.8]
    );
}

/// On the hob: a stock pot with its lid and loop handles, a frying pan with an omelette; salt and pepper mills.
fn _cookware(f: &GameFrame) {
    let y = HOB + 0.043; // the grates' top
    revolve!(
        "stock_pot",
        &[
            (0.0, 0.0),
            (0.12, 0.0),
            (0.128, 0.008),
            (0.13, 0.19),
            (0.134, 0.196),
            (0.13, 0.2),
            (0.122, 0.2),
            (0.122, 0.012),
            (0.0, 0.012)
        ],
        at = f.at(-0.5, y, 0.0),
        material = M.pot,
        segments = 22
    );
    revolve!(
        "pot_lid",
        &[
            (0.0, 0.0),
            (0.132, 0.0),
            (0.13, 0.006),
            (0.1, 0.018),
            (0.04, 0.026),
            (0.0, 0.027)
        ],
        at = f.at(-0.5, y + 0.2, 0.0),
        material = M.pot,
        segments = 22
    );
    revolve!(
        "pot_lid_knob",
        &[
            (0.0, 0.0),
            (0.012, 0.0),
            (0.009, 0.012),
            (0.018, 0.02),
            (0.016, 0.03),
            (0.0, 0.032)
        ],
        at = f.at(-0.5, y + 0.226, 0.0),
        material = M.knob,
        segments = 10
    );
    for s in [-1, 1] {
        let sf = s as f64;
        let x = -0.5 + sf * 0.128;
        fsweep!(
            f,
            format!("pot_handle{s}"),
            &[
                (x, y + 0.17, -0.035),
                (x + sf * 0.03, y + 0.172, -0.03),
                (x + sf * 0.036, y + 0.175, 0.0),
                (x + sf * 0.03, y + 0.172, 0.03),
                (x, y + 0.17, 0.035)
            ],
            0.006,
            M.pot,
            verts = 6,
            resolution = 2
        );
    }
    revolve!(
        "frying_pan",
        &[
            (0.0, 0.0),
            (0.13, 0.0),
            (0.15, 0.04),
            (0.142, 0.04),
            (0.124, 0.008),
            (0.0, 0.008)
        ],
        at = f.at(0.5, y, -0.27),
        material = M.pan,
        segments = 22
    );
    // (its handle out to the right, along the back row: reaching to the front, it ran through what the cook sets
    // down on the front right burner, `SET_DOWN_SPOTS`)
    fsweep!(
        f,
        "pan_handle",
        &[
            (0.645, y + 0.032, -0.27),
            (0.72, y + 0.05, -0.27),
            (0.79, y + 0.06, -0.27)
        ],
        0.0,
        M.pan,
        profile = rounded_rect(0.022, 0.012, 0.005, 1),
        up = (0, 1, 0),
        scale = [0.9, 1.0, 1.15]
    );
    soft!(
        f,
        "omelette",
        0.17,
        0.016,
        0.15,
        (0.5, y + 0.016, -0.27),
        M.egg,
        round = 0.008,
        puff = 0.006,
        flat = 2,
        round_segments = 1
    );
    for (x, z, m) in [(0.72, -0.38, M.tile), (0.64, -0.39, M.knob)] {
        // mills: a waisted body, a knob on top
        revolve!(
            format!("mill{x}"),
            &[
                (0.0, 0.0),
                (0.024, 0.0),
                (0.026, 0.02),
                (0.018, 0.06),
                (0.024, 0.1),
                (0.022, 0.115),
                (0.008, 0.125),
                (0.006, 0.135),
                (0.0, 0.137)
            ],
            at = f.at(x, HOB, z),
            material = m,
            segments = 10
        );
    }
}

pub fn chef_stove() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let still = new_parts(|| {
        fbox!(f, "kick", 1.54, 0.08, 0.52, (0, 0.04, -0.15), M.dark);
        let carcass = fbox!(
            f,
            "carcass",
            1.6,
            0.82,
            FRONT - WALL,
            (0, 0.49, (FRONT + WALL) / 2.0),
            M.body,
            bevel = 0.01
        );
        // the left oven's mouth is cut into the carcass, so its door opens onto an inside
        cut_mouth(
            carcass,
            fbox!(
                f,
                "oven_cut",
                0.62,
                0.42,
                0.36,
                (-0.4, 0.42, FRONT - 0.17),
                M.body
            ),
        );
        _oven(f, -0.4);
        fbox!(
            f,
            "hob",
            1.62,
            0.03,
            FRONT - WALL + 0.02,
            (0, HOB - 0.015, (FRONT + WALL) / 2.0 + 0.01),
            M.dark,
            bevel = 0.006
        );
        fbox!(
            f,
            "upstand",
            1.62,
            0.07,
            0.04,
            (0, HOB + 0.035, WALL + 0.02),
            M.body,
            bevel = 0.006
        );
        fbox!(
            f,
            "strip",
            1.56,
            0.1,
            0.01,
            (0, 0.8, FRONT + 0.005),
            M.dark,
            bevel = 0.003,
            bevel_segments = 1
        );
        for x in KNOBS {
            _knob(f, &format!("knob{x}"), x, 0.8);
        }
        fbox!(
            f,
            "logo",
            0.1,
            0.02,
            0.006,
            (-0.06, 0.8, FRONT + 0.013),
            M.chrome,
            bevel = 0.002,
            bevel_segments = 1
        );
        fbox!(
            f,
            "lamp",
            0.03,
            0.03,
            0.006,
            (0.08, 0.8, FRONT + 0.013),
            M.egg
        );
        fbox!(
            f,
            "oven_split",
            0.02,
            0.56,
            0.02,
            (0, 0.41, FRONT + 0.005),
            M.dark
        );
        _oven_door(f, "right_door", 0.4, 0.76); // the right oven, still
        for (x, z) in burners() {
            _burner(f, &format!("burner{x}{z}"), x, z);
        }
        for x in [-0.5, 0.0, 0.5] {
            _grate(f, &format!("grate{x}"), x);
        }
        _tiles(f);
        _rail(f);
        _hood(f);
        _cookware(f);
    });
    join(&still, "body", Some(P3::from((0, 0, 0))));
    // The left oven door, on its bottom hinge: `oven_open` drops it 85 degrees to the front.
    let door = new_parts(|| _oven_door(f, "oven_door", -0.4, 0.76));
    let door = hinged(
        join(&door, "oven_door", None),
        f.at(-0.4, 0.14, FRONT + 0.0125),
    );
    // The front middle burner's knob, turned by `burner_on` as its flame grows.
    let k = new_parts(|| _knob(f, "burner_knob", -0.25, 0.8));
    let knob = hinged(
        join(&k, "burner_knob", None),
        f.at(-0.25, 0.8, FRONT + 0.03),
    );
    // The ring of gas flame round the front middle burner's cap, built full size and scaled to nothing.
    let (fx, fz) = FLAME;
    let fl = new_parts(|| {
        torus!(
            "flame_ring",
            0.046,
            0.008,
            at = f.at(fx, HOB + 0.026, fz),
            material = M.flame,
            major_segments = 18,
            minor_segments = 5
        );
        for k in 0..12 {
            let a = (30 * k) as f64;
            fcone!(
                f,
                format!("flame_tip{k}"),
                0.01,
                0.0,
                0.035,
                (fx + 0.05 * cosd(a), HOB + 0.043, fz + 0.05 * sind(a)),
                M.flame,
                verts = 6
            );
        }
    });
    let flame = join(&fl, "flame", Some(f.at(fx, HOB + 0.026, fz)));
    flame.set_scale((0.001, 0.001, 0.001));
    let objs = settle!(centre = false);
    swing!(door, "oven_open", 85.0, seconds = 1.2, axis = Axis::X);
    swing!(knob, "burner_on", 90.0, seconds = 0.5, axis = Axis::Y);
    for axis in [Axis::X, Axis::Y, Axis::Z] {
        grow!(flame, "burner_on", seconds = 0.5, axis = axis);
    }
    objs
}
