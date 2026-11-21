//! What is turned from a profile: a pot with its soil, a cupped bloom leaning on its stem.

use crate::far::{is_low, low};
use crate::forms::{Jitter, jitter};
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

options! {
    /// `vessel`'s options (`r_bottom` None: 0.72 of `r_top`; `material` None: the palette's `body`).
    Vessel { z0: f64 = 0.0, r_bottom: Option<f64> = None, lip: f64 = 0.012, foot: f64 = 0.0, saucer: f64 = 0.0, segments: usize = 20, soil: bool = true, belly: f64 = 0.0, material: Option<Mat> = None }
}

/// A pot standing at height `z0`, `h` tall, its rim `r_top` across the inside: a rolled lip `lip` thick round the top
/// (0: a plain, square-cut rim), a foot band `foot` high at the bottom, a body swelling by `belly`, a hollow top and
/// its soil (`soil_top`) a little below the rim, and a `saucer` under it (its overhang). Returns the soil's
/// height. Turned in the palette `p`'s `body` (or `material`), so the player colours it.
pub fn vessel(p: &Palette, r_top: f64, h: f64, o: Vessel) -> f64 {
    let (z0, lip, foot, saucer) = (o.z0, o.lip, o.foot, o.saucer);
    let segments = low(o.segments, 8.max(o.segments / 2));
    let mat = o.material.unwrap_or_else(|| p.get("body"));
    let rb = o.r_bottom.unwrap_or(r_top * 0.72);
    let top = z0 + h;
    let mut prof = vec![(0.0, z0), (rb - 0.006, z0), (rb, z0 + 0.006)];
    if foot != 0.0 {
        prof.extend([(rb + 0.004, z0 + foot * 0.6), (rb + 0.002, z0 + foot)]);
    }
    let body_top = top - if lip != 0.0 { lip * 2.2 } else { 0.0 };
    let mid = z0 + (body_top - z0) * 0.55;
    let rmid = rb + (r_top - rb) * 0.55 + o.belly;
    prof.push((rmid, mid));
    if lip != 0.0 {
        prof.extend([
            (r_top, body_top),
            (r_top + lip * 0.9, body_top + lip * 0.25),
            (r_top + lip, top - lip * 0.5),
            (r_top + lip * 0.6, top),
            (r_top - 0.004, top),
        ]);
    } else {
        prof.extend([(r_top + 0.008, top), (r_top - 0.004, top)]);
    }
    let inner = r_top - 0.008;
    prof.extend([(inner - 0.004, top - 0.04), (0.0, top - 0.04)]);
    revolve(
        "pot",
        &prof,
        Revolve {
            material: Some(mat),
            segments: Some(segments),
            ..Revolve::default()
        },
    );
    if saucer != 0.0 {
        let rs = rb + saucer;
        revolve(
            "saucer",
            &[
                (0.0, z0 - 0.002),
                (rs - 0.01, z0 - 0.002),
                (rs, z0 + 0.008),
                (rs + 0.004, z0 + 0.022),
                (rs - 0.006, z0 + 0.022),
                (rs - 0.012, z0 + 0.008),
                (0.0, z0 + 0.008),
            ],
            Revolve {
                material: Some(mat),
                segments: Some(segments),
                ..Revolve::default()
            },
        );
    }
    if o.soil {
        soil_top(
            Some(p),
            inner - 0.002,
            top - 0.028,
            SoilTop {
                segments,
                ..SoilTop::default()
            },
        );
    }
    top - 0.028
}

options! {
    /// `soil_top`'s options (`material` None: the palette's `soil`).
    SoilTop { segments: usize = 16, mound: f64 = 0.012, seed: f64 = 0.0, material: Option<Mat> = None }
}

/// A pot's soil: a shallow mound `r` across at height `z`, a little uneven, its edge tucked into the pot; in
/// `material`, or the palette `p`'s `soil`.
pub fn soil_top(p: Option<&Palette>, r: f64, z: f64, o: SoilTop) -> Obj {
    let mound = o.mound;
    let prof = [
        (0.0, z - 0.012),
        (r, z - 0.012),
        (r, z - 0.004),
        (r * 0.8, z + mound * 0.45),
        (r * 0.4, z + mound * 0.9),
        (0.0, z + mound),
    ];
    let material = o.material.unwrap_or_else(|| match p {
        Some(p) => p.get("soil"),
        None => panic!("soil_top: a material, or a palette with a soil"),
    });
    let obj = revolve(
        "soil",
        &prof,
        Revolve {
            material: Some(material),
            segments: Some(o.segments),
            ..Revolve::default()
        },
    );
    if !is_low() {
        jitter(
            obj,
            mound * 0.35,
            Jitter {
                size: r * 0.6,
                seed: o.seed + 3.1,
                up: 1.0,
            },
        );
    }
    obj
}
