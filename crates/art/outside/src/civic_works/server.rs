//! The server rack: 1x1, a tall glass-doored cabinet of servers with blinking lights.

use super::common::{F, WALL, own_palette};
use artkit::far::{is_low, low};
use artkit::forms::{Holes, rounded_rect};
use artkit::motion::{Ease, Path, keys};
use artkit::parts::{Settle, join, new_parts, settle_with};
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fextrude, kit_tube};

own_palette! { SERVER:
    body = ("#222528", satin),        // the cabinet's black steel
    unit = ("#7A8088", satin),        // the servers' grey bezels
    bay = ("#2E3236", satin),         // their drive bays
    glass = ("#B9C8CE", glass),       // the door's glass
    screen = ("#4FA8D8", gloss, glow("#7FC8FF")),
    led_green = ("#4BE07A", gloss, glow("#5CFF8A")),
    led_blue = ("#4F8CFF", gloss, glow("#6FA4FF")),
    led_amber = ("#FFB23A", gloss, glow("#FFC25A")),
    cable_blue = ("#3E7BC8", satin), cable_yellow = ("#E8C23A", satin), cable_red = ("#C83A3A", satin),
}

pub const SR_W: f64 = 0.6; // the cabinet: back on the wall's face, door to the front
pub const SR_H: f64 = 2.0;
pub const SR_D: f64 = 0.86;
pub const SR_U: f64 = 0.04445; // a rack unit
pub const SR_FACE: f64 = WALL + SR_D - 0.06; // the servers' faces, behind the door

/// One unit in the rack at height y (its bottom), `units` high: a bezel, what its kind shows on its face (drive
/// bays, ports, a screen), and its LEDs: kept on `leds`, the ones that blink on `blink` (lists of parts).
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn server(
    name: &str,
    y: f64,
    units: usize,
    m: &SERVER,
    kind: &str,
    leds: &mut Vec<Obj>,
    blink: &mut Vec<Obj>,
) {
    let h = units as f64 * SR_U - 0.003;
    let w = 0.482;
    let yc = y + h / 2.0;
    fbox!(
        F,
        name,
        w,
        h,
        0.03,
        (0, yc, SR_FACE - 0.015),
        if kind != "blank" { m.unit } else { m.dark },
        bevel = if kind != "blank" && !is_low() {
            0.002
        } else {
            0.0
        },
        bevel_segments = 1
    );
    if kind == "blank" || is_low() {
        return;
    }
    let face = SR_FACE;
    if kind == "server" {
        let bays = if units == 2 { 8 } else { 4 };
        for k in 0..bays {
            fbox!(
                F,
                format!("{name}_bay{k}"),
                0.046,
                h - 0.016,
                0.004,
                (-0.2 + k as f64 * 0.05, yc, face + 0.001),
                m.bay
            );
        }
        fbox!(
            F,
            format!("{name}_tag"),
            0.03,
            h - 0.02,
            0.004,
            (0.215, yc, face + 0.001),
            m.white
        );
        for k in 0..3 {
            let led = if k < 2 { m.led_green } else { m.led_blue };
            let part = fbox!(
                F,
                format!("{name}_led{k}"),
                0.01,
                0.008,
                0.004,
                (0.19, y + 0.012 + 0.012 * k as f64, face + 0.002),
                led
            );
            if !(k + units).is_multiple_of(2) {
                &mut *blink
            } else {
                &mut *leds
            }
            .push(part);
        }
    } else if kind == "switch" {
        for k in 0..12 {
            fbox!(
                F,
                format!("{name}_port{k}"),
                0.014,
                0.012,
                0.004,
                (-0.19 + k as f64 * 0.022, yc, face + 0.001),
                m.dark
            );
            if k % 2 == 0 {
                let part = fbox!(
                    F,
                    format!("{name}_led{k}"),
                    0.008,
                    0.006,
                    0.004,
                    (-0.19 + k as f64 * 0.022, yc + 0.013, face + 0.002),
                    if k % 3 != 0 { m.led_amber } else { m.led_green }
                );
                if k % 4 != 0 { &mut *blink } else { &mut *leds }.push(part);
            }
        }
    } else if kind == "patch" {
        let cables = [
            m.cable_blue,
            m.cable_yellow,
            m.cable_blue,
            m.cable_red,
            m.cable_blue,
            m.cable_yellow,
        ];
        for k in 0..12 {
            fbox!(
                F,
                format!("{name}_port{k}"),
                0.014,
                0.012,
                0.004,
                (-0.19 + k as f64 * 0.022, yc, face + 0.001),
                m.dark
            );
        }
        for (k, c) in cables.into_iter().enumerate() {
            // patch leads looping down and off to the side
            let x = -0.19 + k as f64 * 0.044;
            kit_tube!(
                F,
                format!("{name}_lead{k}"),
                &[
                    (x, yc, face + 0.01),
                    (x, yc - 0.01, face + 0.035),
                    (x + 0.02, yc - 0.06, face + 0.035),
                    (0.23, yc - 0.05 - 0.004 * k as f64, face + 0.02)
                ],
                0.0035,
                c,
                verts = 4,
                round = 2
            );
        }
    } else if kind == "ups" {
        fbox!(
            F,
            format!("{name}_screen"),
            0.08,
            0.04,
            0.004,
            (0.12, yc + 0.02, face + 0.001),
            m.screen
        );
        for k in 0..4 {
            fbox!(
                F,
                format!("{name}_vent{k}"),
                0.25,
                0.006,
                0.004,
                (-0.08, yc - 0.03 + k as f64 * 0.02, face + 0.001),
                m.dark
            );
        }
        leds.push(fbox!(
            F,
            format!("{name}_led"),
            0.008,
            0.008,
            0.004,
            (0.2, yc - 0.03, face + 0.002),
            m.led_green
        ));
    }
}

pub fn server_rack() -> Vec<Obj> {
    let m = &SERVER;
    let (z0, z1) = (WALL, WALL + SR_D);
    let zc = (z0 + z1) / 2.0;
    let black = m.body;
    let (mut leds, mut blink): (Vec<Obj>, Vec<Obj>) = (Vec::new(), Vec::new());
    let still = new_parts(|| {
        // the cabinet: a plinth, its sides, roof and back, the rails the servers hang on
        fbox!(
            F,
            "plinth",
            SR_W - 0.02,
            0.06,
            SR_D - 0.04,
            (0, 0.03, zc),
            m.dark
        );
        for s in [-1, 1] {
            let sf = s as f64;
            fbox!(
                F,
                format!("side{s}"),
                0.02,
                SR_H - 0.06,
                SR_D,
                (sf * (SR_W / 2.0 - 0.01), (SR_H + 0.06) / 2.0, zc),
                black,
                bevel = low(0.004, 0.0),
                bevel_segments = 1
            );
            fbox!(
                F,
                format!("rail{s}"),
                0.02,
                SR_H - 0.14,
                0.02,
                (sf * 0.25, SR_H / 2.0, SR_FACE - 0.01),
                m.steel
            );
            for k in 0..low(40, 0) {
                fbox!(
                    F,
                    format!("rail_hole{s}{k}"),
                    0.006,
                    0.008,
                    0.002,
                    (sf * 0.25, 0.1 + k as f64 * SR_U, SR_FACE + 0.0005),
                    m.dark
                );
            }
        }
        fbox!(
            F,
            "roof",
            SR_W,
            0.03,
            SR_D,
            (0, SR_H - 0.015, zc),
            black,
            bevel = low(0.004, 0.0),
            bevel_segments = 1
        );
        fbox!(
            F,
            "back",
            SR_W - 0.04,
            SR_H - 0.09,
            0.02,
            (0, SR_H / 2.0, z0 + 0.01),
            black
        );
        fbox!(
            F,
            "inside",
            SR_W - 0.04,
            SR_H - 0.1,
            0.004,
            (0, SR_H / 2.0, z0 + 0.3),
            m.dark
        );
        if !is_low() {
            for k in 0..6 {
                // the roof's vents and its cable brush
                fbox!(
                    F,
                    format!("roof_vent{k}"),
                    0.4,
                    0.004,
                    0.012,
                    (0, SR_H + 0.0005, zc + 0.2 - k as f64 * 0.03),
                    m.dark
                );
            }
            fbox!(
                F,
                "brush",
                0.3,
                0.004,
                0.06,
                (0, SR_H + 0.0005, z0 + 0.1),
                m.plastic
            );
        }
        // the units, from the floor up: a UPS, storage, servers, a switch and a patch panel, blanks between
        let mut y = 0.08;
        for (k, (units, kind)) in [
            (3, "ups"),
            (2, "blank"),
            (4, "server"),
            (2, "server"),
            (2, "server"),
            (1, "blank"),
            (2, "server"),
            (2, "server"),
            (2, "server"),
            (1, "blank"),
            (1, "server"),
            (1, "server"),
            (1, "server"),
            (2, "blank"),
            (1, "switch"),
            (1, "patch"),
            (1, "switch"),
            (1, "patch"),
            (4, "blank"),
            (2, "server"),
            (1, "blank"),
        ]
        .into_iter()
        .enumerate()
        {
            server(
                &format!("unit{k}"),
                y,
                units,
                m,
                kind,
                &mut leds,
                &mut blink,
            );
            y += units as f64 * SR_U;
        }
        // the door: a frame round its glass, hinged on the left, a swing handle on the right
        let dz = z1 - 0.012;
        let frame = rounded_rect(SR_W, SR_H - 0.07, 0.01, 1);
        let hole: Vec<(f64, f64)> = rounded_rect(SR_W - 0.1, SR_H - 0.26, 0.02, low(2, 1))
            .into_iter()
            .collect();
        fextrude!(
            F,
            "door",
            &frame
                .iter()
                .map(|&(u, v)| (u, v + SR_H / 2.0 + 0.035))
                .collect::<Vec<_>>(),
            0.024,
            (0, 0, dz),
            black,
            holes = Holes(vec![
                hole.iter()
                    .map(|&(u, v)| (u, v + SR_H / 2.0 + 0.06))
                    .collect()
            ]),
            bevel = low(0.004, 0.0),
            bevel_segments = 1
        );
        fbox!(
            F,
            "door_glass",
            SR_W - 0.09,
            SR_H - 0.25,
            0.006,
            (0, SR_H / 2.0 + 0.06, dz),
            m.glass
        );
        fbox!(
            F,
            "handle_plate",
            0.03,
            0.2,
            0.01,
            (-0.255, 1.05, z1 + 0.005),
            m.plastic,
            bevel = low(0.004, 0.0),
            bevel_segments = 1
        );
        fbox!(
            F,
            "handle",
            0.022,
            0.15,
            0.022,
            (-0.255, 1.05, z1 + 0.018),
            m.chrome,
            bevel = low(0.006, 0.0),
            bevel_segments = 1
        );
        for (k, y) in [0.25, 1.75].into_iter().enumerate() {
            fbox!(
                F,
                format!("hinge{k}"),
                0.012,
                0.07,
                0.014,
                (0.293, y, z1 - 0.01),
                m.steel
            );
        }
        if !is_low() {
            fbox!(
                F,
                "label",
                0.12,
                0.03,
                0.002,
                (0, SR_H - 0.06, z1 + 0.001),
                m.white
            );
        }
    });
    let body: Vec<Obj> = if !is_low() {
        still
            .iter()
            .copied()
            .filter(|o| !blink.contains(o))
            .collect()
    } else {
        still
    };
    join(
        &body,
        if is_low() {
            "serverRack_far"
        } else {
            "serverRack"
        },
        Some((0, 0, 0).into()),
    );
    let back_on_the_wall = Settle {
        centre: false,
        floor: true,
    };
    if is_low() {
        settle_with(&meshes(), back_on_the_wall);
        return meshes();
    }
    let lights = join(&blink, "leds", Some(F.at(0, 1.0, SR_FACE)));
    settle_with(&meshes(), back_on_the_wall);
    // the blinking LEDs: back into their bezels for a moment, now and then
    let rest = lights.location().y as f64;
    let d = 0.006;
    let lin = Ease::Linear;
    keys(
        lights,
        "blink",
        Path::Location,
        1,
        &[
            (0.0, rest, lin),
            (0.5, rest, lin),
            (0.55, rest + d, lin),
            (0.85, rest + d, lin),
            (0.9, rest, lin),
            (1.3, rest, lin),
            (1.35, rest + d, lin),
            (1.5, rest + d, lin),
            (1.55, rest, lin),
            (2.2, rest, lin),
        ],
    );
    meshes()
}
