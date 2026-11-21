//! O2: the desk chairs, and the chairs the desks with their own chair are drawn with: a padded seat and back on
//! a gas lift and five castors.

use super::common::turned;
use super::{F, M};
use artkit::forms::rounded_rect;
use artkit::materials::Mat;
use artkit::math::{P3, PI, TAU, cos, degrees, exp, pow, radians, sin};
use artkit::parts::{hinged, join, new_parts};
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fcyl, fsweep, kit, loft, py, revolve, settle, soft, sphere, swing};

const LEG_R: f64 = 0.27; // how far a chair's legs reach out from the lift

/// `math.copysign(abs(x) ** k, x)`.
fn signed_pow(x: f64, k: f64) -> f64 {
    pow(x.abs(), k).copysign(x)
}

/// Rings for a contoured chair back `w` wide, `h` high and `t` thick, its middle at game `at`: its outline
/// rounding over the top and the foot, a lumbar swell low down, its sides wrapping forward (`dish`), narrower
/// at the foot by `taper`. Bottom first, closed to a point at each end (Blender points, for `loft`).
#[allow(clippy::too_many_arguments)]
fn contour_rings(
    w: f64,
    h: f64,
    t: f64,
    at: (f64, f64, f64),
    verts: usize,
    n: usize,
    dish: f64,
    lumbar: f64,
    square: f64,
    taper: f64,
) -> Vec<Vec<P3>> {
    let (x0, y0, z0) = at;
    let mut secs = Vec::new();
    for i in 0..=n {
        // rings spaced evenly round the outline's ends, not up its height
        let q = PI * (i as f64 / n as f64 - 0.5);
        let v = pow(sin(q).abs(), 0.5).copysign(q);
        let e = if 0 < i && i < n {
            pow(cos(q).abs(), 0.5)
        } else {
            0.0
        };
        let rx = w / 2.0 * e * (1.0 - taper * (1.0 - v) / 2.0);
        let rz = t / 2.0 * pow(py::max(0.0, 1.0 - pow(v.abs(), 8.0)), 0.125);
        let fwd = lumbar * exp(-pow((v + 0.35) / 0.35, 2.0));
        let mut ring = Vec::new();
        for k in 0..verts {
            let a = TAU * k as f64 / verts as f64;
            let (c, s) = (cos(a), sin(a));
            let px = rx * signed_pow(c, 2.0 / square);
            let pz = rz * signed_pow(s, 2.0 / square);
            ring.push(F.at(
                x0 + px,
                y0 + v * h / 2.0,
                z0 + pz + fwd + dish * pow(px / (w / 2.0), 2.0),
            ));
        }
        secs.push(ring);
    }
    secs
}

/// The game point on a contoured back's front at `v` up it (-1 foot .. 1 top) and `u` across (-1 .. 1).
#[allow(clippy::too_many_arguments)]
fn contour_front(
    w: f64,
    h: f64,
    t: f64,
    at: (f64, f64, f64),
    v: f64,
    u: f64,
    dish: f64,
    taper: f64,
) -> (f64, f64, f64) {
    let (lumbar, square) = (0.012, 4.0);
    let (x0, y0, z0) = at;
    let e = pow(py::max(0.0, 1.0 - pow(v.abs(), 4.0)), 0.25);
    let rx = w / 2.0 * e * (1.0 - taper * (1.0 - v) / 2.0);
    let rz = t / 2.0 * pow(py::max(0.0, 1.0 - pow(v.abs(), 8.0)), 0.125);
    let px = u * rx;
    let pz = rz * pow(py::max(0.0, 1.0 - pow(u.abs(), square)), 1.0 / square);
    let fwd = lumbar * exp(-pow((v + 0.35) / 0.35, 2.0));
    (
        x0 + px,
        y0 + v * h / 2.0,
        z0 + pz + fwd + dish * pow(px / (w / 2.0), 2.0),
    )
}

/// A closed loop round a contoured back's front (a frame's or a welt's path), `k` of the way out to its edge.
#[allow(clippy::too_many_arguments)]
fn outline(
    w: f64,
    h: f64,
    t: f64,
    at: (f64, f64, f64),
    taper: f64,
    dish: f64,
    n: usize,
    k: f64,
) -> Vec<(f64, f64, f64)> {
    let mut out = Vec::new();
    for i in 0..n {
        let q = TAU * i as f64 / n as f64;
        let (c, s) = (cos(q), sin(q));
        let (u, v) = (signed_pow(c, 0.5), signed_pow(s, 0.5));
        out.push(contour_front(w, h, t, at, v * k, u * k, dish, taper));
    }
    out
}

/// A twin-wheel castor under a leg's tip at (x, z), its wheels rolling along the leg (`a`, radians): a
/// socket, a rounded hood and two wheels either side of it (`lite`: one wide wheel, for a chair half under a desk).
fn castor(name: &str, x: f64, z: f64, a: f64, lite: bool) {
    if !lite {
        fcyl!(
            F,
            format!("{name}_socket"),
            0.009,
            0.03,
            (x, 0.058, z),
            M.castor,
            verts = 8
        );
    }
    sphere!(
        format!("{name}_hood"),
        0.024,
        at = F.at(x, 0.034, z),
        material = M.castor,
        scale = (1.0, 1.0, 0.75),
        segments = if lite { 8 } else { 10 },
        rings = if lite { 4 } else { 5 }
    );
    let w = if lite { 0.014 } else { 0.0065 };
    let tyre = [(0.0, -w), (0.02, -w), (0.025, 0.0), (0.02, w), (0.0, w)];
    let sides: &[i32] = if lite { &[0] } else { &[-1, 1] };
    for &s in sides {
        // the wheels' axle across the leg
        let (ox, oz) = (
            -sin(a) * 0.015 * f64::from(s),
            cos(a) * 0.015 * f64::from(s),
        );
        revolve!(
            format!("{name}_wheel{s}"),
            &tyre,
            at = F.at(x + ox, 0.025, z + oz),
            material = M.castor,
            segments = if lite { 8 } else { 12 },
            rot = (0, 90, degrees(a) - 90.0)
        );
    }
}

/// The still base: five swept legs with castors, the hub, and the gas lift up to under the seat.
fn chair_base(z: f64, seat_y: f64, frame: Mat, lite: bool) {
    let prof = rounded_rect(0.032, 0.042, 0.012, 1);
    for i in 0..5 {
        let a = radians(f64::from(i * 72 + 18));
        let (c, s) = (cos(a), sin(a));
        let pts = [
            (c * 0.03, 0.118, z + s * 0.03),
            (c * 0.17, 0.105, z + s * 0.17),
            (c * 0.25, 0.082, z + s * 0.25),
            (c * LEG_R, 0.078, z + s * LEG_R),
        ];
        let path = if lite {
            pts.to_vec()
        } else {
            kit::fillet!(&pts, 0.04, 2)
        };
        fsweep!(
            F,
            format!("leg{i}"),
            &path,
            0.0,
            frame,
            profile = &prof,
            up = (0, 0, 1),
            scale = [1.0, 0.85, 0.75, 0.72],
            angle = 50
        );
        castor(&format!("castor{i}"), c * LEG_R, z + s * LEG_R, a, lite);
    }
    revolve!(
        "hub",
        &[
            (0.0, 0.085),
            (0.05, 0.085),
            (0.056, 0.1),
            (0.05, 0.128),
            (0.034, 0.138),
            (0.0, 0.138)
        ],
        at = F.at(0, 0, z),
        material = frame,
        segments = if lite { 10 } else { 16 }
    );
    // the lift: a tapered plastic sleeve over the cylinder, its chrome piston up into the mechanism
    revolve!(
        "lift_sleeve",
        &[
            (0.0, 0.13),
            (0.031, 0.13),
            (0.029, 0.15),
            (0.025, 0.25),
            (0.021, 0.262),
            (0.0, 0.262)
        ],
        at = F.at(0, 0, z),
        material = M.plastic,
        segments = if lite { 10 } else { 14 }
    );
    let lift = seat_y - 0.13 - 0.25;
    fcyl!(
        F,
        "lift",
        0.016,
        lift,
        (0, 0.25 + lift / 2.0, z),
        M.chrome,
        verts = if lite { 10 } else { 12 }
    );
}

/// A T arm: a bar bent from under the seat out and up, a padded rest on its top.
#[allow(clippy::too_many_arguments)]
fn t_arm(name: &str, s: i32, z: f64, seat_y: f64, frame: Mat, pad: Mat, lite: bool) {
    let s = f64::from(s);
    let (x0, x1) = (s * 0.2, s * 0.268);
    let pts = [
        (x0, seat_y - 0.105, z - 0.03),
        (x1, seat_y - 0.105, z - 0.03),
        (x1, seat_y + 0.125, z - 0.02),
    ];
    fsweep!(
        F,
        format!("{name}_post"),
        &kit::fillet!(&pts, 0.035, if lite { 2 } else { 3 }),
        0.0,
        frame,
        profile = rounded_rect(0.03, 0.05, 0.01, 1),
        up = (0, 1, 0),
        angle = 50
    );
    soft!(
        F,
        format!("{name}_pad"),
        0.07,
        0.032,
        0.27,
        (s * 0.272, seat_y + 0.15, z),
        pad,
        round = 0.014,
        round_segments = if lite { 1 } else { 2 }
    );
}

/// A loop arm: a flat bar bent from under the seat's front up, back along the top and down under its back,
/// a padded rest along the top.
#[allow(clippy::too_many_arguments)]
fn loop_arm(name: &str, s: i32, z: f64, seat_y: f64, frame: Mat, pad: Mat, _lite: bool) {
    let s = f64::from(s);
    let x = s * 0.275;
    let pts = [
        (s * 0.2, seat_y - 0.095, z + 0.12),
        (x, seat_y - 0.095, z + 0.12),
        (x, seat_y + 0.13, z + 0.1),
        (x, seat_y + 0.13, z - 0.14),
        (x, seat_y - 0.095, z - 0.12),
        (s * 0.2, seat_y - 0.095, z - 0.12),
    ];
    fsweep!(
        F,
        format!("{name}_loop"),
        &kit::fillet!(&pts, 0.045, 3),
        0.0,
        frame,
        profile = rounded_rect(0.012, 0.04, 0.005, 1),
        up = (1, 0, 0),
        angle = 50
    );
    soft!(
        F,
        format!("{name}_pad"),
        0.068,
        0.034,
        0.26,
        (x, seat_y + 0.152, z - 0.015),
        pad,
        round = 0.015,
        round_segments = 2
    );
}

/// `chair_seat`'s options: the back's material (the seat's by default), the frame's (black steel) and the arms'
/// (the frame's); a `head`rest; `channels` stitched across (or stripes up a racing back); a mesh back in a frame
/// of `mesh_frame`; how far behind the seat the back stands, how far it leans (degrees), the arms ('t' or
/// 'loop'), and `lite` for a desk's own chair.
#[derive(Clone, Copy)]
pub struct Seat {
    pub back: Option<Mat>,
    pub frame: Option<Mat>,
    pub arm: Option<Mat>,
    pub head: bool,
    pub channels: Option<Mat>,
    pub mesh_frame: Option<Mat>,
    pub back_z: f64,
    pub lean: f64,
    pub arms: &'static str,
    pub lite: bool,
}

impl Default for Seat {
    fn default() -> Seat {
        Seat {
            back: None,
            frame: None,
            arm: None,
            head: false,
            channels: None,
            mesh_frame: None,
            back_z: 0.25,
            lean: 9.0,
            arms: "t",
            lite: false,
        }
    }
}

/// The swivelling part: the mechanism and its levers, a shell under a padded seat, the back on its spine (a
/// contoured pad; a mesh in a frame with `mesh_frame`; stitched across with `channels`, or striped up for a
/// `head`ed racing back), and the arms.
fn chair_seat(z: f64, seat_y: f64, back_h: f64, seat: Mat, o: Seat) {
    let back = o.back.unwrap_or(seat);
    let frame = o.frame.unwrap_or(M.frame);
    let arm = o.arm.unwrap_or(frame);
    let (head, channels, mesh_frame, back_z, lean, lite) =
        (o.head, o.channels, o.mesh_frame, o.back_z, o.lean, o.lite);
    let verts = if lite { 16 } else { 20 };
    fbox!(
        F,
        "mech",
        0.17,
        0.05,
        0.2,
        (0, seat_y - 0.125, z),
        M.plastic,
        bevel = 0.012,
        bevel_segments = 1
    );
    fsweep!(
        F,
        "lever",
        &kit::fillet!(
            &[
                (0.06, seat_y - 0.12, z + 0.04),
                (0.22, seat_y - 0.12, z + 0.05),
                (0.25, seat_y - 0.13, z + 0.09)
            ],
            0.03,
            2
        ),
        0.006,
        M.plastic,
        verts = 6
    );
    fbox!(
        F,
        "lever_tip",
        0.03,
        0.016,
        0.05,
        (0.25, seat_y - 0.13, z + 0.1),
        M.plastic,
        bevel = 0.007,
        bevel_segments = 1
    );
    fcyl!(
        F,
        "tension",
        0.03,
        0.04,
        (0, seat_y - 0.135, z + 0.11),
        M.plastic,
        rot = (90, 0, 0),
        verts = 12,
        bevel = 0.006,
        bevel_segments = 1
    );
    // the seat: a moulded shell, the cushion on it rounded over a waterfall front, piped round its top
    fbox!(
        F,
        "shell",
        0.47,
        0.03,
        0.45,
        (0, seat_y - 0.088, z + 0.01),
        M.plastic,
        bevel = 0.012,
        bevel_segments = 1
    );
    soft!(
        F,
        "seat",
        0.5,
        0.068,
        0.48,
        (0, seat_y - 0.044, z + 0.01),
        seat,
        round = 0.03,
        puff = 0.01,
        flat = 2,
        round_segments = if lite { 1 } else { 2 },
        piping = if lite { None } else { Some(0.005) }
    );
    // the back, built upright and leant back `lean` degrees about its middle
    let yb = seat_y + 0.07 + back_h / 2.0;
    let (bw, bt) = (0.48, if mesh_frame.is_some() { 0.03 } else { 0.07 });
    let z0 = z - back_z + if mesh_frame.is_some() { 0.0 } else { 0.003 };
    let at = (0.0, yb, z0);
    let rear = z0 - if mesh_frame.is_some() { 0.0125 } else { 0.034 }; // the back's rear face, upright
    // the spine: a bar from the mechanism back and up behind the back, to a plate on the back (which leans)
    let plate_y = yb - back_h / 2.0 + 0.08;
    let plate_z = rear - 0.01 + (back_h / 2.0 - 0.08) * sin(radians(lean));
    let sz = plate_z - 0.022;
    fsweep!(
        F,
        "spine",
        &kit::fillet!(
            &[
                (0.0, seat_y - 0.115, z - 0.06),
                (0.0, seat_y - 0.115, sz),
                (0.0, plate_y, sz)
            ],
            0.06,
            3
        ),
        0.0,
        frame,
        profile = rounded_rect(0.024, 0.07, 0.01, 1),
        up = (0, 1, 0),
        angle = 50
    );
    let parts = new_parts(|| {
        fbox!(
            F,
            "back_plate",
            0.12,
            0.14,
            0.02,
            (0, plate_y, rear - 0.01),
            frame,
            bevel = 0.008,
            bevel_segments = 1
        );
        if let Some(mf) = mesh_frame {
            // a moulded frame round a taut mesh, the mesh bowed to the back
            loft!(
                "back",
                &contour_rings(
                    bw - 0.03,
                    back_h - 0.03,
                    0.008,
                    at,
                    verts,
                    8,
                    0.025,
                    0.012,
                    4.0,
                    0.0
                ),
                material = back
            );
            fsweep!(
                F,
                "back_frame",
                &outline(bw - 0.012, back_h - 0.012, 0.0, at, 0.0, 0.025, 32, 1.0),
                0.0,
                mf,
                profile = rounded_rect(0.025, 0.035, 0.008, 1),
                closed = true,
                up = (0, 1, 0),
                angle = 60
            );
            fbox!(
                F,
                "lumbar",
                bw - 0.08,
                0.07,
                0.02,
                (0, yb - back_h * 0.18, z0 - 0.016),
                mf,
                bevel = 0.009,
                bevel_segments = 1
            );
        } else if channels.is_some() && !head {
            // a leather shell, padded in front with four stitched pillows, each as wide as the back where it is
            loft!(
                "back",
                &contour_rings(
                    bw,
                    back_h,
                    0.044,
                    (0.0, yb, z0 - 0.012),
                    verts,
                    12,
                    0.02,
                    0.0,
                    4.0,
                    0.08
                ),
                material = back
            );
            let n = 4;
            let span = back_h * 0.86;
            for k in 0..n {
                let v = -0.86 + (k as f64 + 0.5) * 1.72 / n as f64;
                let e = pow(py::max(0.0, 1.0 - pow(v.abs(), 4.0)), 0.25)
                    * (1.0 - 0.08 * (1.0 - v) / 2.0);
                let fwd = 0.012 * exp(-pow((v + 0.35) / 0.35, 2.0));
                soft!(
                    F,
                    format!("pillow{k}"),
                    bw * e - 0.03,
                    span / n as f64 + 0.006,
                    0.05,
                    (0, yb + v * back_h / 2.0, z0 + 0.012 + fwd),
                    back,
                    round = 0.022,
                    puff = 0.01,
                    flat = 2,
                    round_segments = 2
                );
            }
        } else {
            loft!(
                "back",
                &contour_rings(
                    bw,
                    back_h,
                    bt,
                    at,
                    verts,
                    if lite { 8 } else { 12 },
                    0.02,
                    0.012,
                    4.0,
                    0.08
                ),
                material = back
            );
            loft!(
                "back_shell",
                &contour_rings(
                    bw - 0.02,
                    back_h - 0.02,
                    0.044,
                    (0.0, yb, z0 - 0.012),
                    verts,
                    6,
                    0.02,
                    0.0,
                    4.0,
                    0.08
                ),
                material = M.plastic
            );
            if !lite {
                let welt = outline(bw, back_h, bt, at, 0.08, 0.02, 32, 0.9);
                fsweep!(F, "welt", &welt, 0.005, back, verts = 5, closed = true);
            }
        }
        if let (Some(ch), true) = (channels, head) {
            // a racing back: stripes up either side of its front
            for s in [-1, 1] {
                let pts: Vec<(f64, f64, f64)> = (0..9)
                    .map(|i| -0.8 + 1.6 * i as f64 / 8.0)
                    .map(|v| contour_front(bw, back_h, bt, at, v, f64::from(s) * 0.55, 0.02, 0.08))
                    .map(|(px, py, pz)| (px, py, pz - 0.001))
                    .collect();
                fsweep!(
                    F,
                    format!("stripe{s}"),
                    &pts,
                    0.0,
                    ch,
                    profile = rounded_rect(0.006, 0.03, 0.002, 1),
                    up = (0, 0, 1),
                    angle = 60
                );
            }
        }
        if head {
            soft!(
                F,
                "headrest",
                0.3,
                0.14,
                0.07,
                (0, yb + back_h / 2.0 + 0.1, z - back_z - 0.005),
                back,
                round = 0.03,
                puff = 0.008,
                round_segments = if lite { 1 } else { 2 }
            );
            for s in [-1, 1] {
                fcyl!(
                    F,
                    format!("head_post{s}"),
                    0.008,
                    0.1,
                    (
                        f64::from(s) * 0.08,
                        yb + back_h / 2.0 + 0.01,
                        z - back_z - 0.01
                    ),
                    M.chrome,
                    verts = 8
                );
            }
        }
    });
    turned(&parts, "back", (0, yb, z0), -lean, 0.0);
    for s in [-1, 1] {
        let arm_of = if o.arms == "loop" { loop_arm } else { t_arm };
        arm_of(&format!("arm{s}"), s, z, seat_y, frame, arm, lite);
    }
}

/// A desk's own chair (part of the desk, still): in its back half, facing the desk, its back short of
/// the wall.
pub fn desk_chair(seat: Mat, z: f64, back_h: f64, head: bool, channels: Option<Mat>) {
    let seat_y = 0.44;
    chair_base(z, seat_y, M.frame, true);
    chair_seat(
        z,
        seat_y,
        back_h,
        seat,
        Seat {
            head,
            channels,
            back_z: if head { 0.18 } else { 0.184 },
            lean: if head { 5.0 } else { 7.0 },
            lite: true,
            ..Seat::default()
        },
    );
}

fn chair(seat_y: f64, back_h: f64, z: f64, seat: Mat, o: Seat) -> Vec<Obj> {
    let frame = o.frame.unwrap_or(M.frame);
    let still = new_parts(|| chair_base(z, seat_y, frame, false));
    join(&still, "base", Some((0, 0, 0).into()));
    let parts = new_parts(|| {
        chair_seat(
            z,
            seat_y,
            back_h,
            seat,
            Seat {
                frame: Some(frame),
                ..o
            },
        )
    });
    let swivel = hinged(join(&parts, "seat", None), F.at(0, seat_y - 0.15, z));
    settle!(centre = false);
    swing!(swivel, "swivel", 15.0, seconds = 0.5);
    meshes()
}

pub fn chair_executive() -> Vec<Obj> {
    chair(
        0.49,
        0.78,
        0.05,
        M.leather,
        Seat {
            arm: Some(M.leather),
            channels: Some(M.leather2),
            arms: "loop",
            back_z: 0.254,
            ..Seat::default()
        },
    )
}

pub fn chair_task() -> Vec<Obj> {
    chair(
        0.48,
        0.5,
        0.05,
        M.fabric,
        Seat {
            back: Some(M.mesh),
            mesh_frame: Some(M.plastic),
            ..Seat::default()
        },
    )
}

pub fn office_chair() -> Vec<Obj> {
    chair(
        0.47,
        0.62,
        0.0,
        M.upholstery,
        Seat {
            back_z: 0.254,
            ..Seat::default()
        },
    )
}
