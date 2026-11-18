//! Creatures that perch on what the families build.

use crate::forms::{self, Loft, Profile, Scale, Sweep};
use crate::materials::Mat;
use crate::math::{P3, TAU, V3, cos, radians, sin};
use crate::place::GameFrame;
use crate::scene::Obj;
use crate::shapes::{self, Sphere, options};

/// Degrees as a script gave them, whole or not, and their negation as Python takes it: a whole zero's is a plain
/// zero, a float zero's a negative one, and a part's turn keeps that sign.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Turn {
    pub degrees: f64,
    pub negated: f64,
}

impl crate::Arg<Turn> for f64 {
    fn arg(self) -> Turn {
        Turn {
            degrees: self,
            negated: -self,
        }
    }
}
impl crate::Arg<Turn> for i32 {
    fn arg(self) -> Turn {
        Turn {
            degrees: self as f64,
            negated: -self as f64,
        }
    }
}

options! {
    /// `songbird`'s options.
    Songbird { turn: Turn = Turn { degrees: 0.0, negated: -0.0 }, size: f64 = 1.0, head: f64 = 0.0, low: bool = false }
}

/// A small songbird (a robin, a tit, a sparrow) perched with its feet at game (x, y, z), facing the front turned
/// `turn` degrees towards +x: a lofted body tapering into a fanned tail, a `breast` patch, a pointed `beak`,
/// `eye`s, thin legs down to the perch. `size` 1 is a robin (14 cm long); `head` tips the head down (degrees:
/// drinking, pecking); `low` takes fewer sides and no eyes (a far version). About 350 triangles. Returns its
/// pieces.
#[allow(clippy::too_many_arguments)] // (as the scripts called it)
pub fn songbird(
    frame: &GameFrame,
    name: impl AsRef<str>,
    x: f64,
    y: f64,
    z: f64,
    body: Mat,
    breast: Mat,
    beak: Mat,
    eye: Option<Mat>,
    o: Songbird,
) -> Vec<Obj> {
    let name = name.as_ref();
    let (size, low) = (o.size, o.low);
    let (c, s) = (cos(radians(o.turn.degrees)), sin(radians(o.turn.degrees)));
    let dip = radians(o.head);

    // The game point `u` across (to the bird's right), `v` forward and `w` up from its feet (at size 1).
    let at = |u: f64, mut v: f64, mut w: f64| -> P3 {
        if v > 0.03 {
            // the head and beak tip down about the neck
            let (dv, dw) = (v - 0.03, w - 0.048);
            (v, w) = (
                0.03 + dv * cos(dip) + dw * sin(dip),
                0.048 - dv * sin(dip) + dw * cos(dip),
            );
        }
        let (u, v, w) = (u * size, v * size, w * size);
        frame.at(x + v * s + u * c, y + w, z + v * c - u * s)
    };
    // (forward, half width, half height, centre height): from the tail's root to the face
    let rows = [
        (-0.05, 0.010, 0.010, 0.040),
        (-0.035, 0.022, 0.024, 0.036),
        (-0.012, 0.029, 0.031, 0.034),
        (0.012, 0.027, 0.030, 0.038),
        (0.03, 0.02, 0.022, 0.048),
        (0.045, 0.02, 0.02, 0.058),
        (0.06, 0.016, 0.015, 0.06),
        (0.07, 0.007, 0.007, 0.059),
    ];
    let n = if low { 6 } else { 10 };
    let sections: Vec<Vec<P3>> = rows
        .iter()
        .map(|&(v, hw, hh, wc)| {
            (0..n)
                .map(|k| {
                    at(
                        hw * cos(TAU * k as f64 / n as f64),
                        v,
                        wc + hh * sin(TAU * k as f64 / n as f64),
                    )
                })
                .collect()
        })
        .collect();
    let mut made = vec![forms::loft(
        name,
        &sections,
        Loft {
            material: Some(body),
            ..Loft::default()
        },
    )];
    made.push(forms::sweep(
        format!("{name}_beak"),
        &[at(0.0, 0.066, 0.058), at(0.0, 0.088, 0.057)],
        0.0055 * size,
        Sweep {
            material: Some(beak),
            verts: Some(if low { 5 } else { 6 }),
            scale: Scale::List(vec![1.0, 0.15]),
            ..Sweep::default()
        },
    ));
    let breast_at = V3::from(at(0.0, 0.01, 0.034));
    made.push(shapes::sphere(
        format!("{name}_breast"),
        1.0,
        Sphere {
            at: breast_at.into(),
            material: Some(breast),
            rot: P3 {
                x: 0.0,
                y: 0.0,
                z: o.turn.negated,
            },
            scale: P3 {
                x: 0.026 * size,
                y: 0.027 * size,
                z: 0.025 * size,
            },
            segments: if low { 8 } else { 10 },
            rings: if low { 5 } else { 6 },
            ..Sphere::default()
        },
    ));
    // the tail: a thin fan, cocked a little up
    made.push(forms::sweep(
        format!("{name}_tail"),
        &[at(0.0, -0.04, 0.041), at(0.0, -0.088, 0.054)],
        0.0,
        Sweep {
            material: Some(body),
            profile: Profile(Some(vec![
                (-0.0018, -0.008),
                (0.0018, -0.008),
                (0.0018, 0.008),
                (-0.0018, 0.008),
            ])),
            up: P3 {
                x: 0.0,
                y: 0.0,
                z: 1.0,
            },
            scale: Scale::List(vec![0.9 * size, 1.8 * size]),
            ..Sweep::default()
        },
    ));
    if let Some(eye) = eye
        && !low
    {
        for k in [-1, 1] {
            made.push(shapes::sphere(
                format!("{name}_eye{k}"),
                0.0042 * size,
                Sphere {
                    at: at(k as f64 * 0.0148, 0.055, 0.062),
                    material: Some(eye),
                    segments: 6,
                    rings: 4,
                    ..Sphere::default()
                },
            ));
        }
    }
    for k in [-1, 1] {
        let k_ = k as f64;
        made.push(forms::sweep(
            format!("{name}_leg{k}"),
            &[
                at(k_ * 0.008, 0.0, 0.026),
                at(k_ * 0.009, 0.004, 0.0),
                at(k_ * 0.009, 0.016, 0.0),
            ],
            0.0016 * size,
            Sweep {
                material: Some(beak),
                verts: Some(4),
                ..Sweep::default()
            },
        ));
    }
    made
}
