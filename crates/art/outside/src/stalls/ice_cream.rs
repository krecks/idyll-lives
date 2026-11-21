//! The ice cream cart: a cream freezer on wheels across its 2 x 1 tiles, a parasol over it; its sliding lid opens as
//! the seller reaches in.

use super::common::{M, forms_band, striped, wheel};
use artkit::json::float_repr;
use artkit::materials::Mat;
use artkit::math::{P3, PI, TAU, V3, cos, cosd, radians, sin, sind, v3};
use artkit::motion::{Slide, slide};
use artkit::parts::{join, new_parts};
use artkit::place::GameFrame;
use artkit::scene::{Obj, meshes};
use artkit::{
    fbox, fcyl, fextrude, fsphere, fsweep, jitter, kit_fillet, loft, revolve, settle, soft, sphere,
    sweep,
};

// --- the ice cream cart: a cream freezer on wheels, a parasol over it ------------------------------------
const CART_X: f64 = -0.15; // the body's middle across
const TILT: f64 = 11.0; // the parasol leans forward this much, from its socket at the back of the top

/// A point `s` metres up the parasol's pole (game axes), from its socket at the back of the top.
fn pole_at(s: f64) -> (f64, f64, f64) {
    (CART_X, 0.97 + s * cosd(TILT), -0.2 + s * sind(TILT))
}

/// A parasol's canopy at the origin (Blender, its rim at z 0, its peak at h): gores sagging a little between the
/// ribs, in materials a and b by turns, a scalloped valance hanging from the rim, ribs under it, a finial. Built
/// upright; the caller tilts and places it.
fn parasol(name: &str, r: f64, h: f64, gores: usize, a: Mat, b: Mat) -> Vec<Obj> {
    let n = gores * 2;
    let mut rings: Vec<Vec<P3>> = Vec::new();
    for (t, k) in [(0.0, 1.0), (0.35, 0.68), (0.7, 0.32), (0.95, 0.06)] {
        let mut ring = Vec::new();
        for i in 0..n {
            let ang = TAU * i as f64 / n as f64;
            let sag = if i % 2 == 0 { 0.0 } else { 0.03 * (1.0 - t) };
            ring.push(P3 {
                x: (r * k - sag) * cos(ang),
                y: (r * k - sag) * sin(ang),
                z: h * t * (1.0 + 0.04 * (1.0 - t)),
            });
        }
        rings.push(ring);
    }
    let canopy = striped(
        loft!(
            format!("{name}_canopy"),
            &rings,
            material = a,
            smooth = false
        ),
        gores,
        a,
        b,
    );
    // the valance: a band of scallops round the rim, a tongue a gore
    let m = gores * 3;
    let (mut out, mut inn) = (Vec::new(), Vec::new());
    for i in 0..m {
        let ang = TAU * i as f64 / m as f64;
        let f = (i % 3) as f64 / 3.0;
        let d = 0.06 - 0.02 * (1.0 - sin(PI * f));
        out.push(v3(r * cos(ang), r * sin(ang), -d));
        inn.push(v3((r - 0.006) * cos(ang), (r - 0.006) * sin(ang), -d));
    }
    let top_o: Vec<V3> = (0..m)
        .map(|i| {
            v3(
                r * cos(TAU * i as f64 / m as f64),
                r * sin(TAU * i as f64 / m as f64),
                0.004,
            )
        })
        .collect();
    let top_i: Vec<V3> = (0..m)
        .map(|i| {
            v3(
                (r - 0.006) * cos(TAU * i as f64 / m as f64),
                (r - 0.006) * sin(TAU * i as f64 / m as f64),
                0.004,
            )
        })
        .collect();
    let val = forms_band(&format!("{name}_valance"), &top_o, &out, &inn, &top_i, a);
    striped(val, gores, a, b);
    let mut made = vec![canopy, val];
    for i in (0..n).step_by(2) {
        // the ribs under the canopy
        let ang = TAU * i as f64 / n as f64;
        made.push(sweep!(
            format!("{name}_rib{i}"),
            &[
                v3(0.02 * cos(ang), 0.02 * sin(ang), h * 0.82),
                v3((r - 0.01) * cos(ang), (r - 0.01) * sin(ang), -0.004)
            ],
            0.004,
            material = M.steel,
            verts = 4
        ));
    }
    made.push(revolve!(
        format!("{name}_finial"),
        &[
            (0.0, h * 0.95),
            (0.022, h * 0.95),
            (0.026, h * 0.95 + 0.03),
            (0.016, h * 0.95 + 0.05),
            (0.02, h * 0.95 + 0.07),
            (0.012, h * 0.95 + 0.083),
            (0.0, h * 0.95 + 0.086)
        ],
        material = M.steel,
        segments = 8
    ));
    made
}

#[allow(clippy::approx_constant)] // (the badge's 0.318 m depth, not 1 / pi)
pub fn ice_cream_cart() -> Vec<Obj> {
    let f = GameFrame::new(0.0);
    let still = new_parts(|| {
        // the body: a rounded cream box, two pink bands round it, a pink cone in a circle on its front
        soft!(
            f,
            "cart",
            1.3,
            0.62,
            0.62,
            (CART_X, 0.62, 0.0),
            M.cart,
            round = 0.04,
            round_segments = 2,
            flat = 1
        );
        for y in [0.5, 0.74] {
            soft!(
                f,
                format!("band{}", float_repr(y)),
                1.312,
                0.1,
                0.632,
                (CART_X, y, 0.0),
                M.stripe,
                round = 0.04,
                round_segments = 2,
                flat = 1
            );
        }
        fcyl!(
            f,
            "badge",
            0.1,
            0.008,
            (CART_X, 0.62, 0.312),
            M.stripe,
            verts = 20,
            rot = (90, 0, 0)
        );
        fextrude!(
            f,
            "badge_cone",
            &[(-0.035, 0.0), (0.035, 0.0), (0.0, -0.09)],
            0.006,
            (CART_X, 0.64, 0.318),
            M.wafer
        );
        fcyl!(
            f,
            "badge_scoop",
            0.04,
            0.006,
            (CART_X, 0.655, 0.318),
            M.cart,
            verts = 12,
            rot = (90, 0, 0)
        );
        // the chassis: a steel frame, mudguards over the wheels
        fbox!(
            f,
            "chassis",
            1.16,
            0.04,
            0.46,
            (CART_X, 0.3, 0.0),
            M.steel,
            bevel = 0.006,
            bevel_segments = 1
        );
        for x in [-0.62, 0.38] {
            for z in [-0.27, 0.27] {
                wheel(
                    &f,
                    &format!("wheel{}{}", float_repr(x), float_repr(z)),
                    x,
                    z,
                    0.17,
                    6,
                );
            }
            for z in [-0.3, 0.3] {
                fsweep!(
                    f,
                    format!("axle{}{}", float_repr(x), float_repr(z)),
                    &[(x, 0.17, 0.0), (x, 0.17, z)],
                    0.012,
                    M.steel,
                    verts = 6
                );
            }
        }
        // the push bar: one bent tube from the body out to the handle and back, a rubber grip on it
        let bar = [
            P3::from((0.5, 0.86, -0.22)),
            P3::from((0.82, 1.0, -0.22)),
            P3::from((0.82, 1.0, 0.22)),
            P3::from((0.5, 0.86, 0.22)),
        ];
        fsweep!(
            f,
            "bar",
            &kit_fillet!(&bar, 0.05),
            0.016,
            M.steel,
            verts = 8
        );
        fsweep!(
            f,
            "grip",
            &[(0.82, 1.0, -0.13), (0.82, 1.0, 0.13)],
            0.022,
            M.tyre,
            verts = 8
        );
        for z in [-0.22, 0.22] {
            fcyl!(
                f,
                format!("bar_mount{}", float_repr(z)),
                0.03,
                0.02,
                (0.495, 0.86, z),
                M.steel,
                verts = 8,
                rot = (0, 0, 90)
            );
        }
        // the steel deck round the freezer well, the lids' rails, the fixed lid on the right with a hinge strip
        fbox!(
            f,
            "deck_back",
            1.3,
            0.04,
            0.18,
            (CART_X, 0.95, -0.22),
            M.steel,
            bevel = 0.006
        );
        fbox!(
            f,
            "deck_front",
            1.3,
            0.04,
            0.02,
            (CART_X, 0.95, 0.30),
            M.steel,
            bevel = 0.006
        );
        for x in [-0.77, 0.47] {
            fbox!(
                f,
                format!("deck_end{}", float_repr(x)),
                0.06,
                0.04,
                0.42,
                (x, 0.95, 0.08),
                M.steel,
                bevel = 0.006,
                bevel_segments = 1
            );
        }
        for z in [-0.115, 0.275] {
            fbox!(
                f,
                format!("rail{}", float_repr(z)),
                1.18,
                0.06,
                0.03,
                (CART_X + 0.005, 0.96, z),
                M.steel,
                bevel = 0.004,
                bevel_segments = 1
            );
        }
        fbox!(
            f,
            "lid_fixed",
            0.59,
            0.02,
            0.36,
            (0.145, 0.98, 0.08),
            M.steel,
            bevel = 0.004
        );
        for k in 0..5 {
            // its pressed ribs
            fbox!(
                f,
                format!("lid_rib{k}"),
                0.5,
                0.006,
                0.012,
                (0.145, 0.991, 0.0 + 0.04 * k as f64),
                M.steel
            );
        }
        // the tubs in the well (seen with the lid open), sunk into the top, each heaped with scooped ice cream
        let creams = [
            M.cream_pink,
            M.cream_vanilla,
            M.cream_choc,
            M.cream_mint,
            M.cream_pink,
            M.cream_vanilla,
        ];
        for (k, x) in [-0.64, -0.445, -0.25, -0.05, 0.145, 0.34]
            .into_iter()
            .enumerate()
        {
            revolve!(
                format!("tub{k}"),
                &[
                    (0.0, 0.88),
                    (0.08, 0.88),
                    (0.088, 0.952),
                    (0.09, 0.958),
                    (0.077, 0.958),
                    (0.0, 0.93)
                ],
                at = f.at(x, 0, 0.08),
                material = M.steel,
                segments = 10
            );
            let cream = sphere!(
                format!("cream{k}"),
                0.07,
                at = f.at(x, 0.94, 0.08),
                material = creams[k],
                scale = (1, 1, 0.32),
                segments = 10,
                rings = 5
            );
            jitter!(cream, 0.008, size = 0.04, seed = k as f64 * 1.3);
        }
        // a scoop resting on the back of the deck
        fsweep!(
            f,
            "scoop_handle",
            &[(-0.5, 0.975, -0.24), (-0.36, 0.975, -0.22)],
            0.009,
            M.tyre,
            verts = 6
        );
        fsphere!(
            f,
            "scoop_bowl",
            0.022,
            (-0.335, 0.98, -0.22),
            M.steel,
            segments = 8,
            rings = 4,
            scale = (1, 1, 0.6)
        );
        // the sign: a waffle cone with a scoop on top, a drip round its rim, on the back of the top
        revolve!(
            "wafer",
            &[
                (0.0, 0.97),
                (0.008, 0.97),
                (0.065, 1.16),
                (0.07, 1.17),
                (0.0, 1.17)
            ],
            at = f.at(0.3, 0, -0.225),
            material = M.wafer,
            segments = 10
        );
        for k in 0..3i32 {
            fbox!(
                f,
                format!("waffle{k}"),
                0.004,
                0.16,
                0.004,
                (0.3 + 0.03 * (k - 1) as f64, 1.08, -0.225 + 0.05),
                M.cream_choc,
                rot = (-18, 0, (k - 1) * 20)
            );
        }
        let scoop = fsphere!(
            f,
            "scoop",
            0.08,
            (0.3, 1.2, -0.225),
            M.cream_pink,
            segments = 12,
            rings = 7
        );
        jitter!(scoop, 0.006, size = 0.05, seed = 2.0);
        let drip: Vec<(f64, f64, f64)> = (0..12)
            .map(|i| {
                let i = i as f64;
                (
                    0.3 + 0.075 * cos(TAU * i / 12.0),
                    1.18,
                    -0.225 + 0.075 * sin(TAU * i / 12.0),
                )
            })
            .collect();
        fsweep!(
            f,
            "drip",
            &drip,
            0.014,
            M.cream_pink,
            verts = 5,
            closed = true
        );
        // the parasol, leaning forward from a socket at the back of the top
        fcyl!(
            f,
            "socket",
            0.035,
            0.08,
            (CART_X, 1.0, -0.2),
            M.steel,
            verts = 10
        );
        fsweep!(
            f,
            "pole",
            &[pole_at(0.0), pole_at(1.38)],
            0.018,
            M.steel,
            verts = 8
        );
        fcyl!(
            f,
            "runner",
            0.026,
            0.06,
            pole_at(0.9),
            M.steel,
            verts = 8,
            rot = (TILT, 0.0, 0.0)
        );
    });
    let top = new_parts(|| {
        parasol("parasol", 0.492, 0.28, 16, M.parasol, M.parasol_white);
    });
    let canopy = join(&top, "canopy", Some(P3::from((0, 0, 0))));
    canopy.set_scale((1.5, 0.985, 1)); // (as wide and deep as the old parasol)
    canopy.set_rotation_euler((radians(TILT), 0.0, 0.0));
    let (px, py, pz) = pole_at(1.06);
    canopy.set_location(f.at(px, py, pz + 0.004));
    let body: Vec<Obj> = still.iter().copied().chain([canopy]).collect();
    join(&body, "body", Some(P3::from((0, 0, 0))));
    // the sliding lid over the left half of the well, on the rails; a recessed grip at its left end
    let parts = new_parts(|| {
        fbox!(
            f,
            "lid",
            0.59,
            0.02,
            0.42,
            (-0.445, 1.0, 0.08),
            M.steel,
            bevel = 0.004
        );
        for k in 0..5 {
            fbox!(
                f,
                format!("lid_rib{k}"),
                0.46,
                0.006,
                0.012,
                (-0.42, 1.011, 0.0 + 0.04 * k as f64),
                M.steel
            );
        }
        let grip = [
            P3::from((-0.70, 1.008, -0.02)),
            P3::from((-0.70, 1.035, -0.02)),
            P3::from((-0.70, 1.035, 0.18)),
            P3::from((-0.70, 1.008, 0.18)),
        ];
        fsweep!(
            f,
            "lid_grip",
            &kit_fillet!(&grip, 0.012),
            0.009,
            M.tyre,
            verts = 6
        );
    });
    let lid = join(&parts, "lid", None);
    settle!(centre = false);
    slide(
        lid,
        "lid_open",
        (-0.58, 0, 0),
        Slide {
            seconds: 1.0,
            ..Slide::default()
        },
    ); // to the right (game +x), over the fixed lid
    meshes()
}
