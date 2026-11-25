//! A bench end.

use super::*;
use crate::forms::{self, Profile, Scale, Sweep, rounded_rect};
use crate::json::float_repr;
use crate::kit::{Scroll, scroll};
use crate::materials::Mat;
use crate::math::{Num, P3, PI, sin};
use crate::place::{Coord, GameFrame};
use crate::py::min_of;
use crate::scene::Obj;
use crate::shapes::{Cube, options};

/// A cast-iron bench end in one plane at game x, each member a flat bar `t` thick and `w` deep: the back leg
/// rising in one curve into the upright, leaning back to `top` (height, depth); the front leg, straight up and,
/// with an `arm` height, bent back into the armrest that meets the upright, its front curling down in a scroll;
/// the seat rail between them at `seat` (its top, under the slats); pad feet `foot` long at `front` and `back`
/// (their depths). `steps` rounds the curves (1 for a far version), `corner` the bars' edges (0: square),
/// `scrolls` the curls.
#[allow(clippy::too_many_arguments)] // (as the scripts called it)
pub fn bench_end<X: Coord>(
    frame: &GameFrame,
    name: impl AsRef<str>,
    x: X,
    seat: f64,
    front: f64,
    back: f64,
    top: (impl Num, impl Num),
    material: Mat,
    o: BenchEnd,
) -> Vec<Obj> {
    let name = name.as_ref();
    let (t, w, steps) = (o.t, o.w, o.steps);
    let prof = rounded_rect(
        t,
        w,
        if o.corner != 0 {
            min_of(&[t, w]) * 0.3
        } else {
            0.0
        },
        o.corner.max(1),
    );
    let (ty, tz) = (top.0.f(), top.1.f());
    let rail = seat - w / 2.0;
    let bz = back + 0.045; // where the upright passes the seat rail
    let mut made = Vec::new();
    // a bar of the profile through game points at x
    let bar = |name: String, pts: &[P3], resolution: usize, scale: f64| {
        forms::sweep(
            name,
            pts,
            0.0,
            Sweep {
                material: Some(material),
                profile: Profile(Some(prof.clone())),
                up: P3 {
                    x: 1.0,
                    y: 0.0,
                    z: 0.0,
                },
                resolution,
                scale: Scale::One(scale),
                ..Sweep::default()
            },
        )
    };
    // the back leg into the upright: evenly spaced points, curved through (no corner to overshoot)
    let pts = [
        frame.at(x, 0.0, back),
        frame.at(x, seat * 0.5, back + 0.03),
        frame.at(x, rail, bz),
        frame.at(x, rail + (ty - rail) * 0.5, (bz + tz) / 2.0 + 0.012),
        frame.at(x, ty, tz),
    ];
    made.push(bar(format!("{name}_back"), &pts, steps, 1.0));

    // the upright's depth at height y (above the seat), for the arm to meet
    let upright_z = |y: f64| {
        let f = (y - rail) / (ty - rail);
        bz + (tz - bz) * f + 0.012 * sin(PI * f)
    };
    let fz = front - 0.015;
    match o.arm {
        None => {
            let pts = [frame.at(x, 0.0, front), frame.at(x, rail, fz)];
            made.push(bar(format!("{name}_front"), &pts, 1, 1.0));
        }
        Some(arm) => {
            let pts = fillet(
                &[
                    (x.f(), 0.0, front),
                    (x.f(), rail, fz),
                    (x.f(), arm - w / 2.0, fz - 0.005),
                    (x.f(), arm - w / 2.0, upright_z(arm) + w / 2.0),
                ],
                0.05,
                steps.max(1),
            );
            let pts: Vec<P3> = pts.iter().map(|p| frame.at(p.0, p.1, p.2)).collect();
            made.push(bar(format!("{name}_front"), &pts, 1, 1.0));
            if o.scrolls {
                let r = 0.038;
                made.push(scroll(
                    frame,
                    format!("{name}_scroll"),
                    (x, arm - w / 2.0, fz - 0.012),
                    (arm - w / 2.0 - r, fz - 0.012),
                    material,
                    Scroll {
                        turns: -1.2,
                        t,
                        w: w * 0.75,
                        steps: 4 * steps.max(1),
                        ..Scroll::default()
                    },
                ));
            }
        }
    }
    if o.scrolls {
        // a stretcher sagging between the legs, and a curl in the angle of the front leg and the seat rail
        let pts = [
            frame.at(x, 0.13, front - 0.01),
            frame.at(x, 0.1, (front + back) / 2.0),
            frame.at(x, 0.13, back + 0.02),
        ];
        made.push(bar(format!("{name}_stretcher"), &pts, steps, 0.75));
        made.push(scroll(
            frame,
            format!("{name}_bracket"),
            (x, rail - w * 0.6, fz - w * 0.4),
            (rail - w * 0.6 - 0.035, fz - 0.06),
            material,
            Scroll {
                turns: 0.85,
                t,
                w: w * 0.6,
                steps: 4 * steps.max(1),
                ..Scroll::default()
            },
        ));
    }
    let pts = [frame.at(x, rail, fz + 0.01), frame.at(x, rail, bz - 0.01)];
    made.push(bar(format!("{name}_rail"), &pts, 1, 1.0));
    for z in [front, back] {
        made.push(frame.box_(
            format!("{name}_foot{}", float_repr(z)),
            t + 0.02,
            0.02,
            o.foot,
            (x, 0.01, z),
            material,
            Cube {
                bevel: if o.corner != 0 { 0.006 } else { 0.0 },
                bevel_segments: 1,
                ..Cube::default()
            },
        ));
    }
    made
}

// --- cuts ------------------------------------------------------------------------------------------------------

options! {
    /// `cut`'s options. `lined`: the faces the cutter leaves in the body take the cutter's material (the boolean
    /// modifier's TRANSFER: an oven's enamel, a fridge's liner); without, the body's own, as `forms.cut` left them.
    /// `keep_cutter`: the cutter stays in the scene, for the build to take away itself (`forms.cut`'s own option,
    /// and a script that set the modifier up by hand).
    Cut { lined: bool = false, keep_cutter: bool = false }
}
