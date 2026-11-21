//! OU3: the charcoal grill, the gas grill station and the fire pit.

use super::common::{F, M, bent, bottle, jitter, rod, slab, v};
use artkit::json::float_repr;
use artkit::kit;
use artkit::materials::Mat;
use artkit::math::{Axis, P3, TAU, V3, cos, cosd, pow, radians, sin, sind};
use artkit::motion::{Grow, Swing, grow, swing};
use artkit::parts::{hinged, join, new_parts};
use artkit::place::Facing;
use artkit::scene::{Obj, meshes};
use artkit::shapes::SHARP;
use artkit::{cylinder, fbox, fcyl, fextrude, fsphere, fsweep, loft, revolve, settle};

/// A convex outline of game (depth, height) points, run across from game x0 to x1: a hood's barrel.
fn across(name: &str, outline: &[(f64, f64)], x0: f64, x1: f64, material: Mat) -> Obj {
    let n = outline.len() as u32;
    let mut verts: Vec<V3> = outline
        .iter()
        .map(|&(z, y)| V3::from(F.at(x0, y, z)))
        .collect();
    verts.extend(outline.iter().map(|&(z, y)| V3::from(F.at(x1, y, z))));
    let mut faces = vec![(0..n).collect::<Vec<u32>>(), (n..2 * n).rev().collect()];
    faces.extend((0..n).map(|i| vec![i, (i + 1) % n, n + (i + 1) % n, n + i]));
    // (flat, as a mesh made from points is)
    artkit::forms::object(name, verts, &faces, Some(material), false, SHARP, false)
}

/// `n` glowing lumps of coal scattered over a disc of radius `r` at height `y`.
fn embers(prefix: &str, r: f64, y: f64, n: usize, seed: f64, material: Mat) {
    for k in 0..n {
        let a = artkit::py::fmod(k as f64 * 137.5 + seed, 360.0);
        let d = r * ((k as f64 + 0.5) / n as f64).sqrt();
        fsphere!(
            F,
            format!("{prefix}{k}"),
            0.022 + 0.008 * (k % 3) as f64,
            (d * cosd(a), y, d * sind(a)),
            material,
            scale = (1, 1, 0.6),
            segments = 6,
            rings = 4
        );
    }
}

/// A patty lying at game (x, y = its underside, z): a rounded disc, its edge a little ragged.
fn burger(name: &str, x: f64, y: f64, z: f64, r: f64, material: Mat) {
    revolve!(
        name,
        &[
            (0.0, 0.0),
            (r * 0.92, 0.0),
            (r, 0.006),
            (r * 0.98, 0.016),
            (r * 0.85, 0.022),
            (0.0, 0.023)
        ],
        at = F.at(x, y, z),
        material = material,
        segments = 10
    );
}

const KETTLE_R: f64 = 0.3; // the charcoal grill's bowl: its radius and its rim's height
const KETTLE_RIM: f64 = 0.79;
const TRAY: (f64, f64) = (0.333, 0.333); // its side tray: middle (game x, depth), radius, top
const TRAY_R: f64 = 0.165;
const TRAY_TOP: f64 = 0.7;
const UP: f64 = KETTLE_RIM - 0.99; // its parts' heights, drawn for a rim at 0.99 m, moved down to it

fn h(y: f64) -> f64 {
    y + UP
}

// The charcoal grill: a kettle on three legs, wheels on the back two, an ash catcher under the bowl; inside, a bed of
// red coals (aglow at night) under an empty grate, at a 57 cm kettle's working height; a round side tray hooked on at
// the front right. The lid is hinged at the back of the rim and swings up (`lid_open`) for the whole of a cook.
pub fn grill() -> Vec<Obj> {
    let still = new_parts(|| {
        let (y0, top) = (h(0.64), KETTLE_RIM);
        revolve!(
            "bowl",
            &[
                (0.0, y0),
                (0.15, y0 + 0.005),
                (0.24, y0 + 0.04),
                (0.29, h(0.76)),
                (KETTLE_R, h(0.86)),
                (KETTLE_R, top - 0.006),
                (KETTLE_R + 0.006, top - 0.002),
                (KETTLE_R + 0.004, top + 0.003),
                (KETTLE_R - 0.012, top + 0.003),
                (KETTLE_R - 0.014, h(0.86)),
                (0.276, h(0.77)),
                (0.226, y0 + 0.055),
                (0.14, y0 + 0.02),
                (0.0, y0 + 0.02)
            ],
            at = F.at(0, 0, 0),
            material = M.kettle,
            segments = 32
        );
        for (k, a) in [90.0, 210.0, 330.0].into_iter().enumerate() {
            // the bowl's dampers, under it
            let (c, s_) = (cosd(a + 60.0), sind(a + 60.0));
            fcyl!(
                F,
                format!("damper{k}"),
                0.035,
                0.006,
                (0.17 * c, y0 + 0.01, 0.17 * s_),
                M.grate,
                verts = 10
            );
        }
        for s in [-1, 1] {
            // the bowl's side handles, on their brackets
            let sf = s as f64;
            bent(
                &format!("side_handle{s}"),
                &[
                    (sf * (KETTLE_R - 0.005), h(0.93), -0.06),
                    (sf * (KETTLE_R + 0.014), h(0.93), -0.05),
                    (sf * (KETTLE_R + 0.014), h(0.93), 0.05),
                    (sf * (KETTLE_R - 0.005), h(0.93), 0.06),
                ],
                0.007,
                M.grate,
                0.012,
                6,
                3,
                1.0,
            );
        }
        // the coals, glowing, under the cooking grate (a ring and its bars) with food
        fcyl!(
            F,
            "coal_bed",
            0.24,
            0.03,
            (0, h(0.85), 0),
            M.coals,
            verts = 16
        );
        embers("coal", 0.21, h(0.875), 14, 20.0, M.coal_hot);
        let ring: Vec<(f64, f64, f64)> = (0..24)
            .map(|i| {
                (
                    (KETTLE_R - 0.025) * cos(TAU * i as f64 / 24.0),
                    h(0.965),
                    (KETTLE_R - 0.025) * sin(TAU * i as f64 / 24.0),
                )
            })
            .collect();
        fsweep!(
            F,
            "grate_ring",
            &ring,
            0.006,
            M.grate,
            verts = 5,
            closed = true
        );
        for k in -5..6 {
            let x = k as f64 * 0.05;
            let half = (pow(KETTLE_R - 0.025, 2.0) - x * x).max(0.0).sqrt();
            rod(
                &format!("grate_bar{k}"),
                (x, h(0.965), -half),
                (x, h(0.965), half),
                0.004,
                M.grate,
                5,
            );
        }
        // (no food on the grate at rest: the cook's own is drawn on it while they grill, `STATION_SPOTS`; the
        // burgers and sausages modelled here showed twice over it)
        // the side tray, hooked onto the bowl at its front right on two struts, where the cook sets down what they
        // carried out (`SET_DOWN_SPOTS`): a round enamel tray with a rolled rim, clear of the lid's swing
        let (tx, tz) = TRAY;
        revolve!(
            "tray",
            &[
                (0.0, TRAY_TOP - 0.012),
                (TRAY_R - 0.012, TRAY_TOP - 0.012),
                (TRAY_R, TRAY_TOP - 0.006),
                (TRAY_R, TRAY_TOP + 0.012),
                (TRAY_R - 0.006, TRAY_TOP + 0.014),
                (TRAY_R - 0.01, TRAY_TOP + 0.004),
                (TRAY_R - 0.016, TRAY_TOP),
                (0.0, TRAY_TOP)
            ],
            at = F.at(tx, 0, tz),
            material = M.kettle,
            segments = 20
        );
        let d = artkit::py::hypot(tx, tz);
        let (ux, uz) = (tx / d, tz / d);
        for (k, a) in [-24.0, 24.0].into_iter().enumerate() {
            // two struts in a V from the bowl's wall up under the tray
            let (c, s_) = (cosd(a), sind(a));
            let (wx, wz) = ((ux * c - uz * s_) * 0.29, (uz * c + ux * s_) * 0.29);
            rod(
                &format!("tray_strut{k}"),
                (wx, h(0.83), wz),
                (
                    tx - ux * 0.05 + (wx - ux * 0.29) * 0.6,
                    TRAY_TOP - 0.014,
                    tz - uz * 0.05 + (wz - uz * 0.29) * 0.6,
                ),
                0.006,
                M.grate,
                6,
            );
        }
        fbox!(
            F,
            "tray_clip",
            0.05,
            0.03,
            0.012,
            (ux * 0.302, h(0.93), uz * 0.302),
            M.grate,
            rot = (0, 0, -45),
            bevel = 0.003,
            bevel_segments = 1
        );
        // the legs into sockets under the bowl, a wire rack between them, the ash pan, the wheels on their axle
        let mut feet = Vec::new();
        for (k, a) in [90, 210, 330].into_iter().enumerate() {
            let (c, s_) = (cosd(a as f64), sind(a as f64));
            let foot_y = if a != 90 { 0.075 } else { 0.0 };
            fbox!(
                F,
                format!("socket{k}"),
                0.04,
                0.05,
                0.04,
                (c * 0.13, h(0.665), s_ * 0.13),
                M.grate,
                rot = (0, 0, a),
                bevel = 0.006,
                bevel_segments = 1
            );
            rod(
                &format!("leg{k}"),
                (c * 0.13, h(0.665), s_ * 0.13),
                (c * 0.31, foot_y, s_ * 0.31),
                0.014,
                M.grate,
                8,
            );
            let t = (0.2 - foot_y) / (h(0.665) - foot_y);
            feet.push((
                c * (0.31 + (0.13 - 0.31) * t),
                0.2,
                s_ * (0.31 + (0.13 - 0.31) * t),
            ));
        }
        fsweep!(F, "rack", &feet, 0.006, M.grate, verts = 5, closed = true);
        revolve!(
            "ash_catcher",
            &[
                (0.0, 0.0),
                (0.11, 0.0),
                (0.125, 0.04),
                (0.13, 0.045),
                (0.115, 0.045),
                (0.105, 0.008),
                (0.0, 0.008)
            ],
            at = F.at(0, 0.29, 0),
            material = M.grate,
            segments = 16
        );
        fcyl!(
            F,
            "foot_cap",
            0.02,
            0.02,
            (0, 0.01, 0.31),
            M.wheel,
            verts = 10
        );
        rod(
            "axle",
            (-0.3, 0.075, -0.155),
            (0.3, 0.075, -0.155),
            0.008,
            M.grate,
            6,
        );
        for s in [-1, 1] {
            // a tyre, a hub cap
            revolve!(
                format!("wheel{s}"),
                &[
                    (0.0, -0.015),
                    (0.06, -0.015),
                    (0.072, -0.012),
                    (0.075, 0.0),
                    (0.072, 0.012),
                    (0.06, 0.015),
                    (0.03, 0.017),
                    (0.0, 0.017)
                ],
                at = F.at(s as f64 * 0.29, 0.075, -0.155),
                material = M.wheel,
                segments = 16,
                rot = (0, -90 * s, 0)
            );
            fcyl!(
                F,
                format!("hub{s}"),
                0.025,
                0.006,
                (s as f64 * 0.307, 0.075, -0.155),
                M.grate,
                rot = (0, 90, 0),
                verts = 10
            );
        }
    });
    join(&still, "kettle", Some(P3::from((0, 0, 0))));
    let parts = new_parts(|| {
        revolve!(
            "lid",
            &[
                (0.0, h(1.215)),
                (0.16, h(1.19)),
                (0.25, h(1.125)),
                (KETTLE_R - 0.012, h(1.04)),
                (KETTLE_R - 0.012, KETTLE_RIM + 0.004),
                (KETTLE_R + 0.004, KETTLE_RIM + 0.004),
                (KETTLE_R + 0.004, h(1.045)),
                (0.26, h(1.135)),
                (0.165, h(1.205)),
                (0.0, h(1.23))
            ],
            at = F.at(0, 0, 0),
            material = M.kettle_lid,
            segments = 32
        );
        // the vent on top with its damper's tab; the thermometer on the front; the handle on two brackets under a shield
        fcyl!(
            F,
            "vent",
            0.045,
            0.008,
            (0, h(1.232), 0),
            M.grate,
            verts = 12
        );
        fbox!(
            F,
            "vent_tab",
            0.05,
            0.006,
            0.015,
            (0.05, h(1.236), 0),
            M.grate
        );
        cylinder!(
            "gauge_bezel",
            0.024,
            0.01,
            at = F.at(0, h(1.172), 0.19),
            material = M.handle_steel,
            rot = (50, 0, 0),
            verts = 12
        );
        cylinder!(
            "gauge",
            0.019,
            0.004,
            at = F.at(0, h(1.176), 0.194),
            material = M.gauge,
            rot = (50, 0, 0),
            verts = 12
        );
        for s in [-1, 1] {
            let sf = s as f64;
            rod(
                &format!("handle_post{s}"),
                (sf * 0.06, h(1.12), 0.24),
                (sf * 0.06, h(1.165), 0.275),
                0.007,
                M.grate,
                6,
            );
        }
        fbox!(
            F,
            "handle_shield",
            0.17,
            0.004,
            0.05,
            (0, h(1.152), 0.255),
            M.grate,
            rot = (-35, 0, 0)
        );
        rod(
            "handle",
            (-0.08, h(1.165), 0.277),
            (0.08, h(1.165), 0.277),
            0.013,
            M.handle_steel,
            10,
        );
    });
    let lid = hinged(join(&parts, "lid", None), F.at(0, KETTLE_RIM, -KETTLE_R));
    settle!(centre = false);
    swing(
        lid,
        "lid_open",
        -95.0,
        Swing {
            seconds: 0.9,
            axis: Axis::X,
            ..Swing::default()
        },
    );
    meshes()
}

// The gas grill station: a stainless cart on a plinth, two doors with bar handles and a control panel's knobs below a
// dark counter (its top at 0.9, the content's `surface`: the free right counter); a chopping board with a tray of
// patties, a bottle and a pepper mill on the left. The grill in the middle has a barrel hood hinged at the back that
// swings up (`hood_up`) for cook and feast, and for any recipe's grilling step there (`flip`).
const HOOD: (f64, f64, f64, f64) = (-0.39, 0.11, 0.905, 1.0); // the hood's back and front depth, its bottom and where its barrel starts

pub fn grill_station() -> Vec<Obj> {
    let (back, front, bottom, spring) = HOOD;
    let still = new_parts(|| {
        fbox!(
            F,
            "plinth",
            1.52,
            0.06,
            0.5,
            (0, 0.03, -0.16),
            M.firebox,
            bevel = 0.005,
            bevel_segments = 1
        );
        fbox!(
            F,
            "cart",
            1.6,
            0.8,
            0.58,
            (0, 0.46, -0.15),
            M.cart,
            bevel = 0.01,
            bevel_segments = 2
        );
        slab("top", 1.66, 0.66, 0.9, 0.04, M.worktop, 0.02, (0.0, -0.125));
        // the control panel, tilted back, its knobs and an igniter
        fbox!(
            F,
            "panel",
            0.96,
            0.1,
            0.014,
            (0, 0.8, 0.147),
            M.firebox,
            rot = (-12, 0, 0),
            bevel = 0.003,
            bevel_segments = 1
        );
        for (k, x) in [-0.3, -0.1, 0.1, 0.3].into_iter().enumerate() {
            kit::knob!(
                F,
                format!("knob{k}"),
                x,
                0.8,
                0.154,
                0.018,
                M.handle_steel,
                segments = 10
            );
        }
        fcyl!(
            F,
            "igniter",
            0.012,
            0.012,
            (0.42, 0.8, 0.158),
            M.burger,
            rot = (90, 0, 0),
            verts = 8
        );
        // two stainless doors, framed, with bar pulls; the hinges' knuckles at their outer edges
        for s in [-1, 1] {
            let sf = s as f64;
            kit::shaker!(
                F,
                format!("door{s}"),
                0.74,
                0.6,
                (sf * 0.39, 0.4, 0.147),
                0.014,
                M.cart,
                rail = 0.05,
                recess = 0.003
            );
            kit::pull!(
                F,
                format!("door_handle{s}"),
                sf * 0.39,
                0.64,
                0.154,
                0.44,
                M.handle_steel,
                r = 0.008,
                out = 0.026
            );
            for y in [0.17, 0.63] {
                fcyl!(
                    F,
                    format!("knuckle{s}{}", float_repr(y)),
                    0.006,
                    0.05,
                    (sf * 0.765, y, 0.15),
                    M.handle_steel,
                    verts = 6
                );
            }
        }
        // a towel bar along the right end
        bent(
            "towel_bar",
            &[
                (0.79, 0.72, -0.35),
                (0.818, 0.72, -0.35),
                (0.818, 0.72, 0.05),
                (0.79, 0.72, 0.05),
            ],
            0.008,
            M.handle_steel,
            0.015,
            6,
            3,
            1.0,
        );
        // the grill: a dark firebox let into the counter, its grate with food on it, a warming rack at the back
        fbox!(
            F,
            "firebox",
            0.86,
            0.012,
            0.46,
            (0, 0.898, -0.14),
            M.firebox
        );
        for k in -8..9 {
            rod(
                &format!("grate_bar{k}"),
                (k as f64 * 0.05, 0.909, -0.36),
                (k as f64 * 0.05, 0.909, 0.08),
                0.005,
                M.grate,
                5,
            );
        }
        for z in [-0.36, 0.08] {
            rod(
                &format!("grate_frame{}", float_repr(z)),
                (-0.42, 0.909, z),
                (0.42, 0.909, z),
                0.006,
                M.grate,
                5,
            );
        }
        // (no food on the grate: the hood hides it at rest, and while it's up the cook's own is drawn on it,
        // `STATION_SPOTS`; the burgers, sausages and corn modelled here showed twice over it)
        // on the left counter: a board with a tray of patties, a bottle and a pepper mill behind it; the right counter
        // is left free (the content's `surface`), where the cook chops and sets down what they carried out
        fbox!(
            F,
            "board",
            0.34,
            0.02,
            0.36,
            (-0.62, 0.91, -0.12),
            M.board,
            bevel = 0.006,
            bevel_segments = 1
        );
        fbox!(
            F,
            "tray",
            0.24,
            0.025,
            0.16,
            (-0.62, 0.9325, -0.12),
            M.tray,
            bevel = 0.008,
            bevel_segments = 1
        );
        for (k, x) in [-0.68, -0.6].into_iter().enumerate() {
            burger(&format!("patty{k}"), x, 0.945, -0.12, 0.035, M.burger);
        }
        bottle(
            "bottle",
            -0.74,
            0.9,
            -0.385,
            0.035,
            0.24,
            M.bottle,
            Some(M.handle_steel),
            10,
        );
        revolve!(
            "pepper_mill",
            &[
                (0.0, 0.0),
                (0.022, 0.0),
                (0.024, 0.02),
                (0.016, 0.06),
                (0.02, 0.1),
                (0.022, 0.12),
                (0.012, 0.14),
                (0.004, 0.15),
                (0.0, 0.15)
            ],
            at = F.at(-0.54, 0.9, -0.395),
            material = M.board,
            segments = 10
        );
    });
    join(&still, "cart", Some(P3::from((0, 0, 0))));
    let parts = new_parts(|| {
        // the barrel: an ellipse over its upright front and back, dark end caps, a bar handle, a thermometer
        let (mid, rz, ry) = ((back + front) / 2.0, (front - back) / 2.0, 0.2);
        let arc: Vec<(f64, f64)> = (0..181)
            .step_by(12)
            .map(|a| {
                (
                    mid + rz * cos(radians(a as f64)),
                    spring + ry * sin(radians(a as f64)),
                )
            })
            .collect();
        let mut outline = vec![(front, bottom)];
        outline.extend_from_slice(&arc);
        outline.push((back, bottom));
        across("hood", &outline, -0.45, 0.45, M.hood);
        for s in [-1, 1] {
            let sf = s as f64;
            let mut end: Vec<(f64, f64)> = outline[1..outline.len() - 1]
                .iter()
                .map(|&(z, y)| (z + if z > mid { 0.006 } else { -0.006 }, y + 0.006))
                .collect();
            end.push((back - 0.006, bottom));
            end.push((front + 0.006, bottom));
            across(
                &format!("hood_end{s}"),
                &end,
                sf * 0.45,
                sf * 0.462,
                M.firebox,
            );
        }
        fbox!(
            F,
            "hood_inside",
            0.86,
            0.004,
            rz * 2.0 - 0.04,
            (0, bottom - 0.002, mid),
            M.firebox
        );
        kit::pull!(
            F,
            "hood_handle",
            0,
            1.0,
            front,
            0.68,
            M.handle_steel,
            r = 0.012,
            out = 0.055
        );
        cylinder!(
            "gauge_bezel",
            0.038,
            0.016,
            at = F.at(0, 1.1, front + 0.035),
            material = M.handle_steel,
            rot = (70, 0, 0),
            verts = 16
        );
        cylinder!(
            "gauge",
            0.03,
            0.004,
            at = F.at(0, 1.103, front + 0.043),
            material = M.gauge,
            rot = (70, 0, 0),
            verts = 16
        );
    });
    let hood = hinged(join(&parts, "hood", None), F.at(0, spring, back));
    settle!(centre = false);
    swing(
        hood,
        "hood_up",
        -75.0,
        Swing {
            seconds: 0.9,
            axis: Axis::X,
            ..Swing::default()
        },
    );
    meshes()
}

// The fire pit: a stone bowl with a ring of capstones on its rim, an ash bed with embers that glow at night and a
// teepee of logs. The flames grow on the logs (`flames`) while a resident warms up by it; the light is the
// content's (`light`, at 0.5 m).
const PIT_TOP: f64 = 0.3;

/// A split log from game point a to b: rough bark, its sawn ends pale.
#[allow(clippy::too_many_arguments)] // (as the script called it)
fn log(
    name: &str,
    a: (f64, f64, f64),
    b: (f64, f64, f64),
    r: f64,
    material: Mat,
    end: Mat,
    seed: f64,
) {
    let pts = [
        a,
        ((a.0 + b.0) / 2.0, (a.1 + b.1) / 2.0, (a.2 + b.2) / 2.0),
        b,
    ];
    jitter(
        fsweep!(
            F,
            name,
            &pts,
            r,
            material,
            verts = 7,
            scale = [1.0, 0.95, 0.9]
        ),
        0.004,
        0.06,
        seed,
        1.0,
    );
    let d = (v(b) - v(a)).normalized();
    for (k, (p, rr)) in [(a, r), (b, r * 0.9)].into_iter().enumerate() {
        let at = v(p) + d * (if k == 1 { 0.001 } else { -0.001 });
        let cap = cylinder!(
            format!("{name}_end{k}"),
            rr * 0.85,
            0.004,
            at = at,
            material = end,
            verts = 7
        );
        cap.edit(|o| o.quat = Some(d.to_track_quat_zy()));
    }
}

pub fn fire_pit() -> Vec<Obj> {
    let still = new_parts(|| {
        // the bowl: two courses of stones round a dark wall, the ash bed inside
        revolve!(
            "bowl",
            &[
                (0.0, 0.0),
                (0.36, 0.0),
                (0.36, PIT_TOP - 0.02),
                (0.31, PIT_TOP - 0.02),
                (0.31, 0.2),
                (0.0, 0.2)
            ],
            at = F.at(0, 0, 0),
            material = M.coals,
            segments = 20,
            smooth = false
        );
        for (row, (y, hh, n, off)) in [(0.07, 0.14, 11, 0.0), (0.21, 0.13, 11, 0.5)]
            .into_iter()
            .enumerate()
        {
            for k in 0..n {
                let a = 360.0 * (k as f64 + off) / n as f64;
                let w = TAU * 0.39 / n as f64 - 0.012;
                let st = fbox!(
                    F,
                    format!("stone{row}_{k}"),
                    w,
                    hh - 0.008,
                    0.1,
                    (0.39 * cosd(a), y, 0.39 * sind(a)),
                    M.pit,
                    rot = (0.0, 0.0, a - 90.0),
                    bevel = 0.018,
                    bevel_segments = 1
                );
                jitter(st, 0.008, 0.07, (row * 11 + k) as f64, 1.0);
            }
        }
        fcyl!(F, "ash", 0.315, 0.1, (0, 0.255, 0), M.coals, verts = 20);
        embers("ember", 0.2, 0.31, 12, 40.0, M.ember);
        // the capstones: ten wedges round the rim, each a little askew (three pushed out a little: the old one's bounds)
        for i in 0..10 {
            let a = i * 36 + (if i % 2 == 1 { 7 } else { -4 });
            let o = match i {
                7 => 0.021,
                0 => 0.014,
                5 => 0.017,
                _ => 0.0,
            };
            let wedge = [
                (0.31 + o, -0.085),
                (0.435 + o, -0.112),
                (0.435 + o, 0.112),
                (0.31 + o, 0.085),
            ];
            let cap = fextrude!(
                F,
                format!("cap{i}"),
                &wedge,
                0.07,
                (0, 0.335, 0),
                M.pit_stones,
                plane = Facing::Top,
                bevel = 0.022,
                bevel_segments = 1,
                rot = (0, 0, a + (i % 3 - 1) * 5)
            );
            jitter(cap, 0.006, 0.08, (50 + i) as f64, 0.4);
        }
        // the teepee of logs, and one across under it
        for (k, a) in [20.0, 110.0, 200.0, 290.0].into_iter().enumerate() {
            let (c, s_) = (cosd(a), sind(a));
            log(
                &format!("log{k}"),
                (0.22 * c, 0.31, 0.22 * s_),
                (0.03 * c, 0.6, 0.03 * s_),
                0.035,
                M.log,
                M.log_end,
                k as f64 * 3.1,
            );
        }
        log(
            "log_across",
            (-0.18, 0.33, -0.08),
            (0.18, 0.33, 0.1),
            0.04,
            M.log,
            M.log_end,
            9.0,
        );
    });
    join(&still, "pit", Some(P3::from((0, 0, 0))));
    // the flames: tongues round the logs and a bright core, grown from the ash
    let base = 0.298; // under the ash's top
    let parts = new_parts(|| {
        for (k, (a, d, hh, r)) in [
            (0.0, 0.09, 0.3, 0.08),
            (51.0, 0.11, 0.22, 0.07),
            (103.0, 0.08, 0.34, 0.08),
            (154.0, 0.11, 0.24, 0.07),
            (206.0, 0.09, 0.32, 0.08),
            (257.0, 0.11, 0.2, 0.06),
            (309.0, 0.09, 0.28, 0.075),
        ]
        .into_iter()
        .enumerate()
        {
            flame(
                &format!("flame{k}"),
                d * cosd(a),
                base,
                d * sind(a),
                r,
                hh,
                a,
                M.flame,
                0.04,
            );
        }
        for (k, (x, z, hh, r)) in [
            (0.0, 0.0, 0.46, 0.085),
            (0.04, -0.03, 0.32, 0.06),
            (-0.04, 0.03, 0.36, 0.06),
        ]
        .into_iter()
        .enumerate()
        {
            flame(
                &format!("flame_core{k}"),
                x,
                base,
                z,
                r,
                hh,
                40.0 * k as f64,
                M.flame_core,
                0.015,
            );
        }
    });
    let flames = join(&parts, "flames", Some(F.at(0, base, 0)));
    // Shrunk to a point at rest, not just flattened: the night glow's copy of the faces, drawn a hair in front,
    // would show a flattened disc through the ash.
    flames.set_scale((0.001, 0.001, 0.001));
    settle!(centre = false);
    for axis in [Axis::X, Axis::Y, Axis::Z] {
        grow(flames, "flames", Grow { seconds: 0.8, axis });
    }
    meshes()
}

/// A tongue of flame from game (x, y, z), `r` across at its foot and `h` tall, flattened across the way it leans
/// (out along `a` degrees), swelling a little above its foot, twisting as it narrows to its tip.
#[allow(clippy::too_many_arguments)] // (as the script called it)
fn flame(
    name: &str,
    x: f64,
    y: f64,
    z: f64,
    r: f64,
    hh: f64,
    a: f64,
    material: Mat,
    lean: f64,
) -> Obj {
    let b = F.at(x, y, z);
    let (bx, by, bz) = (b.x, b.y, b.z);
    let out = V3::from((-cosd(a), -sind(a), 0)); // game (cos, sin) along x, depth as Blender
    let mut sections = Vec::new();
    for (t, k) in [
        (0.0, 0.8),
        (0.18, 1.0),
        (0.45, 0.78),
        (0.72, 0.42),
        (1.0, 0.0),
    ] {
        let c = V3::from((bx, by, 0)) + out * lean * t * t;
        let tw = radians(a + 50.0 * t);
        let mut lp = Vec::new();
        for i in 0..6 {
            let p = TAU * i as f64 / 6.0;
            let (u, w) = (r * k * cos(p), r * k * 0.6 * sin(p));
            lp.push(P3 {
                x: c.x as f64 + u * cos(tw) - w * sin(tw),
                y: c.y as f64 + u * sin(tw) + w * cos(tw),
                z: bz + hh * t,
            });
        }
        sections.push(lp);
    }
    loft!(name, &sections, material = material, angle = 80)
}
