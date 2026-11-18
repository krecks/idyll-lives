//! D2: the grandfather clock.

use super::common::{
    F, M, WALL, box_outline, clock_clips, clock_hands, disc, fan, fr, rect, turned_g,
};
use artkit::forms::rounded_rect;
use artkit::kit;
use artkit::math::{Axis, P3, TAU, atan2, cos, degrees, pow, radians, sin};
use artkit::motion::Ease;
use artkit::parts::{hinged, join, new_parts};
use artkit::place::Facing;
use artkit::py;
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fcyl, fextrude, revolve, rock, settle, swing, torus};

const GF_Z: f64 = WALL + 0.2; // the case's middle, game depth (0.4 deep, back on the wall)
const TRUNK_FRONT: f64 = WALL + 0.35; // the trunk's front face
const DIAL_Y: f64 = 1.77;

const ROMAN: [(i32, &str); 12] = [
    (1, "I"),
    (2, "II"),
    (3, "III"),
    (4, "IIII"),
    (5, "V"),
    (6, "VI"),
    (7, "VII"),
    (8, "VIII"),
    (9, "IX"),
    (10, "X"),
    (11, "XI"),
    (12, "XII"),
];

/// Roman hours round a dial centred at game (0, y), their feet towards the middle, `r` out and `h` tall, drawn in
/// strokes: I a bar, V and X two bars crossed or splayed.
fn numerals(y: f64, z: f64, r: f64, h: f64, material: artkit::materials::Mat) {
    let (stroke, gap) = (h * 0.13, h * 0.3);
    for (hour, text) in ROMAN {
        let a = hour * 30;
        let widths: Vec<f64> = text
            .chars()
            .map(|c| gap * if c != 'I' { 1.6 } else { 1.0 })
            .collect();
        let mut u = -py::sum(widths.iter().copied()) / 2.0;
        for (j, c) in text.chars().enumerate() {
            let mid = u + widths[j] / 2.0;
            u += widths[j];
            let bars: Vec<(i32, f64)> = match c {
                'I' => vec![(0, 0.0)],
                'V' => vec![(-14, -0.22), (14, 0.22)],
                _ => vec![(-22, 0.0), (22, 0.0)],
            };
            for (b, (tilt, off)) in bars.into_iter().enumerate() {
                // the stroke's middle on the ring, `mid` along it, turned to the hour and tilted in its letter (seen
                // from the front the game's +x is on the left: three o'clock is at -x)
                let (ca, sa) = (cos(radians(a as f64)), sin(radians(a as f64)));
                let (x0, y0) = (
                    -sa * r - ca * (mid + off * gap),
                    y + ca * r - sa * (mid + off * gap),
                );
                fbox!(
                    F,
                    format!("numeral{hour}_{j}_{b}"),
                    stroke,
                    h,
                    0.001,
                    (x0, y0, z),
                    material,
                    rot = (0, a + tilt, 0)
                );
            }
        }
    }
}

/// Python's `_dedupe`: consecutive points closer than `eps` (in x plus y) dropped, and the last if it closes on the
/// first.
fn dedupe(pts: &[(f64, f64)], eps: f64) -> Vec<(f64, f64)> {
    let mut out: Vec<(f64, f64)> = Vec::new();
    for &p in pts {
        if out
            .last()
            .is_none_or(|q: &(f64, f64)| (p.0 - q.0).abs() + (p.1 - q.1).abs() > eps)
        {
            out.push(p);
        }
    }
    if out.len() > 2
        && (out[0].0 - out[out.len() - 1].0).abs() + (out[0].1 - out[out.len() - 1].1).abs() <= eps
    {
        out.pop();
    }
    out
}

/// A walnut longcase clock: a moulded plinth and a base with a fielded panel, a waist moulding, a trunk with quarter
/// columns and a door whose arched window shows the pendulum, a neck moulding, a hood with turned brass columns, a
/// cornice and an arched pediment under three urn finials; a brass dial plate with corner spandrels, a cream chapter
/// ring with Roman hours and a minute track, winding holes and shaped hands.
pub fn grandfather_clock() -> Vec<Obj> {
    let dz = GF_Z + 0.18;
    let door_z = TRUNK_FRONT + 0.026;
    let still = new_parts(|| {
        kit::moulding!(
            F,
            "plinth",
            &box_outline(0.6, 0.4, GF_Z),
            &[
                (0.0, 0.0),
                (0.0, 0.03),
                (0.006, 0.036),
                (0.012, 0.046),
                (0.018, 0.054),
                (0.022, 0.06),
                (0.05, 0.06),
                (0.05, 0.0)
            ],
            M.walnut_dark,
            at = 0.0,
            plane = Facing::Top
        );
        fbox!(
            F,
            "base",
            0.56,
            0.38,
            0.38,
            (0, 0.25, GF_Z - 0.01),
            M.walnut,
            bevel = 0.01
        );
        fextrude!(
            F,
            "base_panel",
            &rounded_rect(0.36, 0.22, 0.03, 3),
            0.014,
            (0, 0.25, GF_Z + 0.185),
            M.walnut_dark,
            bevel = 0.006,
            bevel_segments = 2
        );
        kit::moulding!(
            F,
            "base_frame",
            &rect(0.4, 0.26, 0.25, 0.0),
            &[
                (0.0, 0.0),
                (0.0, 0.006),
                (0.006, 0.01),
                (0.014, 0.006),
                (0.016, 0.0)
            ],
            M.walnut,
            at = GF_Z + 0.18
        );
        kit::moulding!(
            F,
            "waist_moulding",
            &box_outline(0.58, 0.4, GF_Z),
            &[
                (0.07, 0.0),
                (0.0, 0.0),
                (0.0, 0.01),
                (0.006, 0.016),
                (0.012, 0.028),
                (0.03, 0.04),
                (0.07, 0.04)
            ],
            M.walnut_dark,
            at = 0.44,
            plane = Facing::Top
        );
        fbox!(
            F,
            "trunk",
            0.46,
            1.06,
            0.3,
            (0, 1.0, TRUNK_FRONT - 0.15),
            M.walnut,
            bevel = 0.008
        );
        fbox!(
            F,
            "trunk_hollow",
            0.32,
            0.8,
            0.004,
            (0, 1.0, TRUNK_FRONT + 0.002),
            M.hollow
        );
        for s in [-1, 1] {
            // quarter columns at the trunk's front corners, on brass bases and capitals
            let sf = s as f64;
            fcyl!(
                F,
                format!("quarter{s}"),
                0.016,
                0.96,
                (sf * 0.222, 1.0, TRUNK_FRONT - 0.006),
                M.walnut_dark,
                verts = 12
            );
            for (y, k) in [(0.52, 1), (1.48, -1)] {
                turned_g(
                    &format!("quarter_cap{s}{}", fr(y)),
                    &[
                        (0.0, 0.0),
                        (0.022, 0.0),
                        (0.022, 0.008),
                        (0.017, 0.016),
                        (0.0, 0.016),
                    ],
                    (sf * 0.222, y, TRUNK_FRONT - 0.006),
                    M.brass,
                    12,
                    k < 0,
                );
            }
        }
        kit::moulding!(
            F,
            "neck_moulding",
            &box_outline(0.58, 0.4, GF_Z),
            &[
                (0.07, 0.0),
                (0.03, 0.0),
                (0.012, 0.012),
                (0.006, 0.024),
                (0.0, 0.03),
                (0.0, 0.04),
                (0.07, 0.04)
            ],
            M.walnut_dark,
            at = 1.53,
            plane = Facing::Top
        );
        fbox!(
            F,
            "hood",
            0.54,
            0.44,
            0.38,
            (0, 1.79, GF_Z - 0.01),
            M.walnut,
            bevel = 0.01
        );
        for s in [-1, 1] {
            // turned brass columns either side of the hood door
            turned_g(
                &format!("column{s}"),
                &[
                    (0.0, 0.0),
                    (0.02, 0.0),
                    (0.02, 0.012),
                    (0.014, 0.022),
                    (0.016, 0.03),
                    (0.0125, 0.06),
                    (0.0115, 0.33),
                    (0.0145, 0.37),
                    (0.016, 0.39),
                    (0.02, 0.4),
                    (0.02, 0.412),
                    (0.0, 0.412),
                ],
                (s as f64 * 0.24, 1.584, GF_Z + 0.18),
                M.brass,
                12,
                false,
            );
        }
        kit::moulding!(
            F,
            "cornice",
            &box_outline(0.6, 0.4, GF_Z),
            &[
                (0.07, 0.0),
                (0.03, 0.0),
                (0.02, 0.008),
                (0.012, 0.02),
                (0.006, 0.034),
                (0.0, 0.042),
                (0.0, 0.05),
                (0.07, 0.05)
            ],
            M.walnut_dark,
            at = 2.01,
            plane = Facing::Top
        );
        fbox!(
            F,
            "cornice_top",
            0.48,
            0.006,
            0.28,
            (0, 2.057, GF_Z),
            M.walnut_dark
        );
        // the arched pediment over the cornice, its middle finial on the arch's crown
        let mut arch = vec![(0.29, 2.06), (0.29, 2.068)];
        arch.extend((10..171).step_by(20).map(|a| {
            let a = radians(a as f64);
            (0.29 * cos(a) * 1.0, 2.06 + 0.032 * pow(sin(a), 0.7) + 0.008)
        }));
        arch.extend([(-0.29, 2.068), (-0.29, 2.06)]);
        fextrude!(
            F,
            "pediment",
            &arch,
            0.03,
            (0, 0, GF_Z + 0.155),
            M.walnut_dark,
            bevel = 0.004,
            bevel_segments = 1
        );
        for (x, y, h) in [
            (-0.25, 2.06, 0.056),
            (0.25, 2.06, 0.056),
            (0.0, 2.098, 0.018),
        ] {
            turned_g(
                &format!("finial{}", fr(x)),
                &[
                    (0.0, 0.0),
                    (0.012, 0.0),
                    (0.016, 0.008 * h / 0.056),
                    (0.01, 0.016 * h / 0.056),
                    (0.022 * 1.0_f64.min(h / 0.03), 0.03 * h / 0.056),
                    (0.016, 0.042 * h / 0.056),
                    (0.004, 0.05 * h / 0.056),
                    (0.0, h),
                ],
                (x, y, GF_Z),
                M.brass,
                12,
                false,
            );
        }
        // the hood door's frame round the square dial plate
        fextrude!(
            F,
            "hood_door",
            &rounded_rect(0.44, 0.42, 0.01, 1),
            0.014,
            (0, DIAL_Y, GF_Z + 0.178),
            M.walnut,
            holes = [rounded_rect(0.37, 0.37, 0.006, 1)],
            bevel = 0.003,
            bevel_segments = 1
        );
        fbox!(
            F,
            "dial_plate",
            0.37,
            0.37,
            0.004,
            (0, DIAL_Y, dz + 0.002),
            M.brass
        );
        for sx in [-1, 1] {
            for sy in [-1, 1] {
                fan(
                    &format!("spandrel{sx}{sy}"),
                    sx as f64 * 0.125,
                    DIAL_Y + sy as f64 * 0.125,
                    dz + 0.004,
                    0.05,
                    degrees(atan2(sy as f64, sx as f64)),
                    M.brass,
                    0.004,
                );
            }
        }
        let round = |r: f64| -> Vec<(f64, f64)> {
            (0..40)
                .map(|k| {
                    let a = TAU * k as f64 / 40.0;
                    (r * cos(a), DIAL_Y + r * sin(a))
                })
                .collect()
        };
        fextrude!(
            F,
            "dial",
            &round(0.16),
            0.004,
            (0, 0, dz + 0.006),
            M.dial,
            holes = [round(0.112)]
        );
        disc("dial_centre", 0, DIAL_Y, dz + 0.004, 0.1, M.dial, 0.003, 32);
        torus!(
            "dial_ring",
            0.165,
            0.006,
            at = F.at(0, DIAL_Y, dz + 0.006),
            material = M.brass,
            rot = (90, 0, 0),
            major_segments = 40,
            minor_segments = 4
        );
        numerals(DIAL_Y, dz + 0.0085, 0.134, 0.03, M.ink);
        for k in 0..60 {
            let a = radians((k * 6) as f64);
            fbox!(
                F,
                format!("minute{k}"),
                0.0016,
                if k % 5 != 0 { 0.007 } else { 0.01 },
                0.001,
                (sin(a) * 0.153, DIAL_Y + cos(a) * 0.153, dz + 0.0085),
                M.ink,
                rot = (0, -k * 6, 0)
            );
        }
        for s in [-1, 1] {
            disc(
                &format!("winding{s}"),
                s as f64 * 0.05,
                DIAL_Y - 0.045,
                dz + 0.007,
                0.008,
                M.hollow,
                0.001,
                10,
            );
        }
        disc("arbor", 0, DIAL_Y, dz + 0.014, 0.01, M.brass, 0.006, 12);
    });
    join(&still, "body", Some(P3::ZERO));
    // The trunk door (a walnut frame round an arched window) is a part of its own: it swings open on its left edge
    // (seen from the front) while the clock is wound, the weights behind it. The pendulum, the hands and it take the
    // draws (five, as the pack entry's `draws` allows this one model).
    let leaf = new_parts(|| {
        let mut window = vec![(-0.13, 0.65), (0.13, 0.65)];
        window.extend((0..181).step_by(15).map(|a| {
            let a = radians(a as f64);
            (0.13 * cos(a), 1.22 + 0.13 * sin(a))
        }));
        window.push((-0.13, 0.65));
        let window = dedupe(&window, 1e-6);
        let hole: Vec<(f64, f64)> = window.iter().map(|&(u, v)| (u, v - 1.0)).collect();
        fextrude!(
            F,
            "door",
            &rounded_rect(0.36, 0.82, 0.01, 1),
            0.02,
            (0, 1.0, door_z),
            M.walnut,
            holes = [hole],
            bevel = 0.004,
            bevel_segments = 1
        );
        let bead: Vec<(f64, f64)> = window.iter().rev().copied().collect();
        kit::moulding!(
            F,
            "door_bead",
            &bead,
            &[
                (-0.008, 0.0),
                (-0.008, 0.006),
                (-0.004, 0.009),
                (0.0, 0.006),
                (0.0, 0.0)
            ],
            M.walnut_dark,
            at = door_z + 0.01
        );
        disc(
            "escutcheon",
            0.15,
            1.0,
            door_z + 0.01,
            0.012,
            M.brass,
            0.003,
            12,
        );
        fbox!(
            F,
            "keyhole",
            0.004,
            0.01,
            0.001,
            (0.15, 1.0, door_z + 0.0135),
            M.hollow
        );
    });
    let door = hinged(join(&leaf, "door", None), F.at(0.18, 1.0, door_z));
    let hands = clock_hands(
        DIAL_Y,
        dz,
        ("hour", 0.075, 0.014, 0.009, 0.0),
        ("minute", 0.115, 0.009, 0.011, 0.0),
    );
    // The pendulum: a brass rod and lens-shaped bob hung from just under the hood, in front of the dark hollow, seen
    // through the door's window; it swings ±4° about its top (`pendulum`, ambient), a beat each 0.8 s, as a pendulum
    // this long does.
    let pz = TRUNK_FRONT + 0.008;
    let pendulum = new_parts(|| {
        fbox!(F, "pendulum_rod", 0.01, 0.58, 0.004, (0, 1.12, pz), M.brass);
        fbox!(
            F,
            "pendulum_block",
            0.03,
            0.03,
            0.008,
            (0, 1.39, pz),
            M.brass,
            bevel = 0.003,
            bevel_segments = 1
        );
        revolve!(
            "pendulum_bob",
            &[
                (0.0, -0.008),
                (0.035, -0.006),
                (0.058, -0.0025),
                (0.065, 0.0),
                (0.058, 0.0025),
                (0.035, 0.006),
                (0.0, 0.008)
            ],
            at = F.at(0, 0.8, pz + 0.002),
            material = M.brass,
            segments = 24,
            rot = (90, 0, 0)
        );
        fcyl!(
            F,
            "rating_nut",
            0.008,
            0.012,
            (0, 0.72, pz),
            M.brass,
            verts = 8
        );
    });
    let pendulum = hinged(join(&pendulum, "pendulum", None), F.at(0, 1.41, pz));
    settle!(centre = false);
    clock_clips(hands);
    rock!(pendulum, "pendulum", 4.0, seconds = 1.6, axis = Axis::Y);
    swing!(
        door,
        "door_open",
        -100.0,
        seconds = 0.8,
        ease = Ease::BackOut
    );
    meshes()
}
