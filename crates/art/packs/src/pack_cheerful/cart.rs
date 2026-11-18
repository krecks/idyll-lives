//! The fizz & sunshine mocktail cart (1x1): a brass bar cart under a striped canopy.

use super::common::{M, TOP, slab, tumbler};
use artkit::forms::{rounded_rect, smooth_outline};
use artkit::materials::Mat;
use artkit::math::{P3, PI, cos, cosd, pow, sin, sind};
use artkit::motion::{Squash, squash};
use artkit::parts::{hinged, join, new_parts, numbered};
use artkit::place::{Facing, GameFrame};
use artkit::scene::Obj;
use artkit::{
    fbox, fcone, fcyl, fextrude, fsphere, fsweep, jitter, kit, loft, object_of, revolve, ring,
    settle, torus,
};

const SHAKER: (f64, f64) = (0.06, 0.08); // the shaker's foot (x, depth), tipping over to the left

/// A syrup bottle standing on `y`: a lofted body with a shoulder and neck, a label band, a chrome pourer.
#[allow(clippy::too_many_arguments)]
fn bottle(
    f: &GameFrame,
    name: &str,
    x: f64,
    y: f64,
    z: f64,
    r: f64,
    h: f64,
    material: Mat,
    cap: Mat,
) {
    let P3 {
        x: bx,
        y: by,
        z: bz,
    } = f.at(x, y, z);
    let rows = [
        (0.0, 0.92),
        (0.02, 1.0),
        (0.62, 1.0),
        (0.76, 0.6),
        (0.84, 0.38),
        (0.95, 0.38),
    ];
    let secs: Vec<Vec<P3>> = rows
        .iter()
        .map(|&(t, k)| ring!(r * k, r * k, z = bz + h * t, verts = 10, at = (bx, by)))
        .collect();
    let b = loft!(name, &secs, material = material);
    kit::paint(b, M.porcelain, |c| {
        let up = c.z as f64 - bz;
        0.2 * h < up && up < 0.5 * h
    });
    revolve!(
        format!("{name}_pourer"),
        &[
            (0.0, 0.0),
            (r * 0.42, 0.0),
            (r * 0.42, 0.012),
            (r * 0.15, 0.03),
            (0.0, 0.032)
        ],
        at = (bx, by, bz + h * 0.95),
        material = cap,
        segments = 8
    );
}

pub fn mocktail_cart() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let (zb, zf) = (-0.29, 0.17);
    let still = new_parts(|| {
        for y in [0.3, 0.8] {
            fextrude!(
                f,
                numbered("shelf"),
                &rounded_rect(0.78, 0.46, 0.03, 2),
                0.03,
                (0.0, y, -0.06),
                M.mint,
                plane = Facing::Top,
                bevel = 0.008,
                bevel_segments = 1
            );
            let gal: Vec<(f64, f64, f64)> = rounded_rect(0.78, 0.46, 0.04, 2)
                .iter()
                .map(|&(x, z)| (x, y + 0.06, -0.06 + z))
                .collect();
            fsweep!(
                f,
                numbered("gallery"),
                &gal,
                0.006,
                M.brass,
                verts = 6,
                closed = true
            );
            for x in [-0.2, 0.0, 0.2] {
                for z in [zb, zf] {
                    kit::tube!(
                        f,
                        numbered("spindle"),
                        &[(x, y + 0.015, z), (x, y + 0.06, z)],
                        0.0035,
                        M.brass,
                        verts = 4,
                        round = 1
                    );
                }
            }
        }
        for x in [-0.39, 0.39] {
            for z in [zb, zf] {
                let y0 = if z == zf { 0.07 } else { 0.0 };
                kit::tube!(
                    f,
                    numbered("post"),
                    &[(x, y0, z), (x, 1.6, z)],
                    0.012,
                    M.brass,
                    verts = 8,
                    round = 1
                );
                revolve!(
                    numbered("finial"),
                    &[
                        (0.0, 0.0),
                        (0.014, 0.0),
                        (0.018, 0.008),
                        (0.012, 0.016),
                        (0.0, 0.022)
                    ],
                    at = f.at(x, 1.598, z),
                    material = M.brass,
                    segments = 8
                );
            }
            torus!(
                numbered("wheel"),
                0.055,
                0.016,
                at = f.at(x, 0.071, zf),
                material = M.tyre,
                rot = (0, 90, 0),
                major_segments = 16,
                minor_segments = 6
            );
            fcyl!(
                f,
                numbered("hub"),
                0.03,
                0.04,
                (x, 0.071, zf),
                M.brass,
                rot = (0, 90, 0),
                verts = 10
            );
            revolve!(
                numbered("cap"),
                &[
                    (0.0, 0.0),
                    (0.018, 0.0),
                    (0.016, 0.014),
                    (0.012, 0.02),
                    (0.0, 0.02)
                ],
                at = f.at(x, 0, zb),
                material = M.dark,
                segments = 8
            );
        }
        kit::tube!(
            f,
            "handle",
            &[(-0.39, 0.95, zb), (-0.39, 0.95, zf)],
            0.012,
            M.brass,
            verts = 8,
            round = 1
        );
        fsweep!(
            f,
            "handle_grip",
            &[(-0.39, 0.95, zb + 0.1), (-0.39, 0.95, zf - 0.1)],
            0.017,
            M.porcelain,
            verts = 8
        );
        // the striped canopy, arched front to back, with a scalloped valance at the front and sides
        for i in 0..6 {
            let m = if i % 2 != 0 { M.canopy } else { M.canopy_white };
            let x = -0.35 + i as f64 * 0.14;
            let pts: Vec<(f64, f64, f64)> = (0..9)
                .map(|k| {
                    let s = (2 * k) as f64 / 8.0 - 1.0;
                    (
                        x,
                        1.6 + 0.024 * (1.0 - pow(s, 2.0)) + 0.006,
                        -0.33 + 0.56 * k as f64 / 8.0,
                    )
                })
                .collect();
            fsweep!(
                f,
                numbered("canopy"),
                &pts,
                0,
                m,
                profile = rounded_rect(0.012, 0.142, 0.004, 1),
                up = (0, 0, 1)
            );
            let mut pts = vec![(-0.07, 1.605), (0.07, 1.605)];
            pts.extend((0..7).map(|k| {
                (
                    0.07 * cos(PI * k as f64 / 6.0),
                    1.56 - 0.07 * sin(PI * k as f64 / 6.0),
                )
            }));
            let pts: Vec<(f64, f64)> = pts.iter().map(|&(px, py)| (x + px, py)).collect();
            slab(f, &numbered("scallop"), &pts, 0.218, 0.23, m, 0.0);
        }
        for s in [-1.0, 1.0] {
            for i in 0..4 {
                let z = -0.26 + i as f64 * 0.14;
                let m = if i % 2 != 0 { M.canopy } else { M.canopy_white };
                let mut pts = vec![(z - 0.07, 1.605), (z + 0.07, 1.605)];
                pts.extend((0..7).map(|k| {
                    (
                        z + 0.07 * cos(PI * k as f64 / 6.0),
                        1.56 - 0.07 * sin(PI * k as f64 / 6.0),
                    )
                }));
                let mut verts: Vec<P3> = Vec::new();
                for j in [1.0, -1.0] {
                    verts.extend(
                        pts.iter()
                            .map(|&(zz, y)| f.at(s * 0.41 + s * 0.006 * j, y, zz)),
                    );
                }
                let nn = pts.len() as u32;
                let mut faces = vec![(0..nn).collect::<Vec<u32>>(), (nn..2 * nn).rev().collect()];
                faces.extend((0..nn).map(|k| vec![k, (k + 1) % nn, nn + (k + 1) % nn, nn + k]));
                object_of!(&numbered("scallop_side"), &verts, &faces, &[m]);
            }
        }
        // the top: a jug of juice, an ice bucket, a pineapple, citrus and three drinks
        let (jx, jz) = (-0.25, -0.17);
        let P3 {
            x: bx,
            y: by,
            z: bz,
        } = f.at(jx, TOP, jz);
        let secs: Vec<Vec<P3>> = [
            (0.0, 0.048),
            (0.01, 0.055),
            (0.08, 0.06),
            (0.15, 0.05),
            (0.19, 0.046),
            (0.2, 0.05),
        ]
        .iter()
        .map(|&(h, r)| ring!(r, r, z = bz + h, verts = 14, at = (bx, by)))
        .collect();
        loft!("jug", &secs, material = M.porcelain);
        fcyl!(
            f,
            "jug_juice",
            0.044,
            0.002,
            (jx, TOP + 0.201, jz),
            M.juice,
            verts = 12
        );
        fcone!(
            f,
            "jug_spout",
            0.02,
            0.008,
            0.03,
            (jx + 0.05, TOP + 0.195, jz),
            M.porcelain,
            rot = (0, -50, 0),
            verts = 8
        );
        fsweep!(
            f,
            "jug_handle",
            &kit::fillet!(
                &[
                    (jx - 0.055, TOP + 0.17, jz),
                    (jx - 0.095, TOP + 0.16, jz),
                    (jx - 0.09, TOP + 0.06, jz),
                    (jx - 0.058, TOP + 0.04, jz)
                ],
                0.02,
                steps = 2
            ),
            0,
            M.porcelain,
            profile = rounded_rect(0.016, 0.008, 0.003, 1),
            up = (0, 1, 0)
        );
        revolve!(
            "bucket",
            &[
                (0.0, 0.0),
                (0.06, 0.0),
                (0.08, 0.12),
                (0.075, 0.12),
                (0.055, 0.01),
                (0.0, 0.01)
            ],
            at = f.at(0.2, TOP, -0.17),
            material = M.chrome,
            segments = 16
        );
        for s in [-1.0, 1.0] {
            torus!(
                numbered("bucket_ring"),
                0.016,
                0.003,
                at = f.at(0.2 + s * 0.08, TOP + 0.095, -0.17),
                material = M.chrome,
                rot = (90, 0, 90),
                major_segments = 8,
                minor_segments = 3
            );
        }
        for (k, (dx, dz)) in [(-0.02, 0.0), (0.02, 0.015), (0.0, -0.025), (0.03, -0.01)]
            .into_iter()
            .enumerate()
        {
            let k = k as i32;
            fbox!(
                f,
                numbered("ice"),
                0.03,
                0.03,
                0.03,
                (0.2 + dx, TOP + 0.11 + 0.008 * (k % 2) as f64, -0.17 + dz),
                M.ice,
                rot = (10 * k, 20, 15 * k),
                bevel = 0.005,
                bevel_segments = 1
            );
        }
        // the pineapple: a lumpy body and a crown of long leaves
        let pine = fsphere!(
            f,
            "pineapple",
            0.055,
            (0.3, TOP + 0.08, 0.06),
            M.pineapple,
            scale = (1, 1, 1.45),
            segments = 12,
            rings = 8
        );
        jitter!(pine, 0.004, size = 0.02, seed = 4.0);
        let leaf = smooth_outline(
            &[
                (0.0, 0.0),
                (0.012, 0.03),
                (0.006, 0.1),
                (0.0, 0.13),
                (-0.006, 0.1),
                (-0.012, 0.03),
            ],
            1,
            true,
        );
        for k in 0..7 {
            let a = (51 * k) as f64;
            let lean = (18 + 10 * (k % 2)) as f64;
            fextrude!(
                f,
                numbered("crown"),
                &leaf,
                0.004,
                (0.3 + 0.008 * cosd(a), TOP + 0.155, 0.06 + 0.008 * sind(a)),
                M.leaf,
                rot = (lean * sind(a), lean * cosd(a), -a)
            );
        }
        for (x, z, r, m) in [
            (0.15, 0.1, 0.034, M.lemon),
            (0.21, 0.13, 0.03, M.lime),
            (0.17, 0.02, 0.038, M.orange),
        ] {
            fsphere!(
                f,
                numbered("citrus"),
                r,
                (x, TOP + r, z),
                m,
                scale = (1, 1, 0.95),
                segments = 10,
                rings = 6
            );
        }
        fcyl!(
            f,
            "lemon_half",
            0.034,
            0.006,
            (0.24, TOP + 0.003, 0.0),
            M.porcelain,
            verts = 10
        );
        fcyl!(
            f,
            "lemon_face",
            0.03,
            0.006,
            (0.24, TOP + 0.006, 0.0),
            M.lemon,
            verts = 10
        );
        tumbler(
            f,
            -0.1,
            0.1,
            M.drink_pink,
            None,
            0.11,
            0.034,
            None,
            Some(M.lemon),
        );
        tumbler(
            f,
            -0.22,
            0.06,
            M.drink_sun,
            None,
            0.11,
            0.034,
            Some(M.sky),
            None,
        );
        tumbler(
            f,
            -0.33,
            0.1,
            M.drink_teal,
            None,
            0.11,
            0.034,
            None,
            Some(M.lime),
        );
        // below: a little fridge drawer and three syrup bottles
        fextrude!(
            f,
            "fridge",
            &rounded_rect(0.3, 0.2, 0.03, 2),
            0.22,
            (-0.15, 0.415, -0.1),
            M.fridge,
            bevel = 0.01,
            bevel_segments = 1
        );
        fbox!(
            f,
            "fridge_seam",
            0.27,
            0.004,
            0.004,
            (-0.15, 0.49, 0.011),
            M.chrome
        );
        kit::pull!(
            f,
            "fridge_handle",
            -0.15,
            0.46,
            0.01,
            0.12,
            M.chrome,
            r = 0.005,
            out = 0.02
        );
        for (i, m) in [M.drink_pink, M.drink_teal, M.drink_sun]
            .into_iter()
            .enumerate()
        {
            bottle(
                f,
                &numbered("syrup"),
                0.12 + i as f64 * 0.09,
                0.315,
                -0.12,
                0.033,
                0.24,
                m,
                M.chrome,
            );
        }
    });
    join(&still, "body", Some(P3::ZERO));
    // The cocktail shaker, hinged on its foot's left edge: `shake` takes it off the cart (squashed away) while the
    // resident shakes held.py's `tool.shaker` in both hands (review I-kitchen-13).
    let (sx, sz) = SHAKER;
    let shaker = new_parts(|| {
        revolve!(
            "shaker",
            &[
                (0.0, 0.0),
                (0.034, 0.0),
                (0.036, 0.004),
                (0.042, 0.12),
                (0.044, 0.124),
                (0.038, 0.13),
                (0.036, 0.15),
                (0.022, 0.172),
                (0.014, 0.176),
                (0.016, 0.19),
                (0.012, 0.205),
                (0.0, 0.206)
            ],
            at = f.at(sx, TOP, sz),
            material = M.chrome,
            segments = 16
        );
        torus!(
            "shaker_band",
            0.04,
            0.004,
            at = f.at(sx, TOP + 0.06, sz),
            material = M.chrome,
            major_segments = 16,
            minor_segments = 3
        );
    });
    let shaker = hinged(join(&shaker, "shaker", None), f.at(sx - 0.036, TOP, sz));
    let objs = settle!(centre = false);
    squash(
        shaker,
        "shake",
        0.001,
        Squash {
            seconds: 0.2,
            ..Squash::default()
        },
    );
    objs
}
