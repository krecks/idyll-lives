//! What is turned from a profile: a pot with its soil, a cupped bloom leaning on its stem.

use crate::far::low;
use crate::materials::{Mat, Palette};
use crate::math::{Axis, M4, V3, p3, radians, v3};
use crate::scene::Obj;
use crate::shapes::{Revolve, Round, cylinder, options, revolve};

options! {
    /// `pot`'s options (`r_bottom` None: 0.72 of `r_top`; `rim` 0: none).
    Pot { z0: f64 = 0.0, r_bottom: Option<f64> = None, rim: f64 = 0.012, rim_h: f64 = 0.03, segments: usize = 24, soil: bool = true }
}

/// A pot standing at height `z0`, `h` tall, its top `r_top` across the rim's inside, in the palette `p`'s `body`
/// (its soil in `soil`): returns its soil height.
pub fn pot(p: &Palette, r_top: f64, h: f64, o: Pot) -> f64 {
    let (z0, rim, rim_h) = (o.z0, o.rim, o.rim_h);
    let segments = low(o.segments, 8.max(o.segments / 2));
    let rb = o.r_bottom.unwrap_or(r_top * 0.72);
    let top = z0 + h;
    let mut profile = vec![
        (0.0, z0),
        (rb, z0),
        (rb + 0.004, z0 + 0.012),
        (r_top, top - rim_h),
    ];
    if rim != 0.0 {
        profile.extend([(r_top + rim, top - rim_h), (r_top + rim, top)]);
    } else {
        profile.extend([(r_top, top)]);
    }
    let inner = r_top - 0.012;
    profile.extend([
        (inner, top),
        (inner - 0.006, top - 0.035),
        (0.0, top - 0.035),
    ]);
    revolve(
        "pot",
        &profile,
        Revolve {
            material: Some(p.get("body")),
            segments: Some(segments),
            ..Revolve::default()
        },
    );
    if o.soil {
        cylinder(
            "soil",
            inner - 0.004,
            0.01,
            Round {
                at: p3(0, 0, top - 0.03),
                material: Some(p.get("soil")),
                verts: segments,
                ..Round::default()
            },
        );
    }
    top - 0.025
}

options! {
    /// `_put`'s options.
    Put { yaw: f64 = 0.0, tilt: f64 = 0.0, lift: f64 = 0.0 }
}

/// Moves a part made at the origin (facing up) to `at`, its face leaning `tilt` degrees towards `yaw`.
pub fn _put(obj: Obj, at: impl Into<V3>, o: Put) -> Obj {
    let m = M4::translation(at.into())
        * M4::rotation(radians(-o.yaw), Axis::Z)
        * M4::rotation(radians(-o.tilt), Axis::X)
        * M4::translation(v3(0, 0, o.lift));
    obj.edit(|part| part.mesh.transform(&m));
    obj
}

options! {
    /// `cupped`'s options.
    Cupped { yaw: f64 = 0.0, tilt: f64 = 0.0, segments: usize = 6 }
}

/// A bloom turned from `profile` [(r, z)] (a tulip, a poppy, a lily's trumpet), leaning `tilt` towards `yaw`.
pub fn cupped(
    name: impl AsRef<str>,
    profile: &[(f64, f64)],
    at: impl Into<V3>,
    material: Mat,
    o: Cupped,
) -> Obj {
    _put(
        revolve(
            name,
            profile,
            Revolve {
                material: Some(material),
                segments: Some(low(o.segments, 3)),
                ..Revolve::default()
            },
        ),
        at,
        Put {
            yaw: o.yaw,
            tilt: o.tilt,
            ..Put::default()
        },
    )
}
