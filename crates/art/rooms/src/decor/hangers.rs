//! D2: the coat rack and the hanging plant.

use super::common::{F, M, heart};
use artkit::forms::{Plane, rounded_rect};
use artkit::kit;
use artkit::math::{Axis, P3, cos, radians, sin};
use artkit::parts::{hinged, join, join_all, new_parts, settle_all};
use artkit::scene::{Obj, meshes};
use artkit::{
    extrude, fsphere, fsweep, jitter, loft, revolve, ring, settle, sphere, sweep, swing, torus,
};

const COAT_AT: (f64, f64) = (0.1, 0.07); // where the coat hangs (game x, depth)
const COAT_HEM: f64 = 0.97; // its hem's height
const COAT_TURN: i32 = 30; // its turn

/// A coat hung by its collar: a body flaring a little to the hem, lapels and three buttons down its front, pocket
/// flaps, sleeves hanging at its sides with turned-back cuffs, a collar rolled round its neck.
fn coat() {
    let (c, turn) = (F.at(COAT_AT.0, 0, COAT_AT.1), radians(COAT_TURN as f64));
    // a point in the coat's own frame (Blender x across it, y through it, h up from the hem)
    let at = |x: f64, y: f64, h: f64| {
        (
            c.x + x * cos(turn) - y * sin(turn),
            c.y + x * sin(turn) + y * cos(turn),
            COAT_HEM + h,
        )
    };
    // lofted through flat oval rings (not a lathe squashed after its UVs were laid): the cloth's weave even all round
    let body: Vec<Vec<P3>> = [
        (0.125, 0.0),
        (0.13, 0.015),
        (0.12, 0.1),
        (0.108, 0.3),
        (0.112, 0.45),
        (0.105, 0.53),
        (0.075, 0.585),
        (0.035, 0.6),
    ]
    .iter()
    .map(|&(r, h)| ring!(r, r * 0.45, z = h, verts = 20))
    .collect();
    loft!(
        "coat",
        &body,
        material = M.coat,
        at = at(0.0, 0.0, 0.0),
        rot = (0, 0, COAT_TURN)
    );
    let collar: Vec<(f64, f64, f64)> = (0..11)
        .map(|k| {
            let a = radians((-120 + 24 * k) as f64);
            at(0.07 * sin(a), 0.035 * cos(a), 0.585 + 0.01 * cos(a))
        })
        .collect();
    sweep!(
        "coat_collar",
        &collar,
        0.014,
        material = M.coat,
        verts = 6,
        resolution = 2
    );
    for s in [-1, 1] {
        let sf = s as f64;
        extrude!(
            format!("lapel{s}"),
            &[
                (0.0, 0.0),
                (sf * 0.05, 0.11),
                (sf * 0.03, 0.135),
                (sf * 0.006, 0.13)
            ],
            0.006,
            at = at(sf * 0.006, -0.05, 0.43),
            material = M.coat,
            plane = Plane::XZ,
            rot = (-6, 0, COAT_TURN)
        );
        extrude!(
            format!("pocket{s}"),
            &rounded_rect(0.06, 0.022, 0.004, 1),
            0.006,
            at = at(sf * 0.065, -0.048, 0.2),
            material = M.coat,
            plane = Plane::XZ,
            rot = (0, 0, COAT_TURN + s * 18)
        );
        sweep!(
            format!("sleeve{s}"),
            &[
                at(sf * 0.1, -0.005, 0.53),
                at(sf * 0.118, -0.012, 0.36),
                at(sf * 0.122, -0.022, 0.15)
            ],
            0.027,
            material = M.coat,
            verts = 10,
            resolution = 2,
            scale = [1.0, 0.95, 0.9]
        );
        torus!(
            format!("cuff{s}"),
            0.024,
            0.006,
            at = at(sf * 0.122, -0.022, 0.16),
            material = M.coat,
            major_segments = 10,
            minor_segments = 4,
            rot = (0, 0, COAT_TURN)
        );
    }
    for k in 0..3 {
        sphere!(
            format!("button{k}"),
            0.007,
            at = at(0.0, -0.054 + 0.004 * k as f64, 0.36 - 0.09 * k as f64),
            material = M.brass,
            segments = 8,
            rings = 4
        );
    }
}

/// A turned oak coat stand: four legs curving down from a hub to round pads, a pole with beads under each ring of
/// brass S-hooks and a ball finial; a coat, a fringed scarf over a hook and a hat with a band.
pub fn coat_rack() -> Vec<Obj> {
    new_parts(|| {
        let prof = [
            (0.0, 0.0),
            (0.03, 0.0),
            (0.042, 0.02),
            (0.044, 0.05),
            (0.036, 0.07),
            (0.026, 0.085),
            (0.023, 0.12),
            (0.021, 1.2),
            (0.026, 1.215),
            (0.026, 1.235),
            (0.02, 1.25),
            (0.019, 1.39),
            (0.024, 1.405),
            (0.024, 1.425),
            (0.019, 1.44),
            (0.017, 1.54),
            (0.013, 1.565),
            (0.0, 1.57),
        ];
        revolve!(
            "pole",
            &prof,
            at = F.at(0, 0.17, 0),
            material = M.oak,
            segments = 14
        );
        fsphere!(
            F,
            "top",
            0.038,
            (0, 1.75, 0),
            M.oak,
            segments = 14,
            rings = 8
        );
        for k in 0..4 {
            let a = radians((45 + k * 90) as f64);
            let (c, s) = (cos(a), sin(a));
            let pts: Vec<(f64, f64, f64)> =
                [(0.02, 0.24), (0.08, 0.205), (0.15, 0.1), (0.19, 0.03)]
                    .iter()
                    .map(|&(r, h)| (r * c, h, r * s))
                    .collect();
            kit::tube!(
                F,
                format!("leg{k}"),
                &pts,
                0.016,
                M.oak,
                verts = 8,
                round = 3,
                scale = [1.25, 1.05, 0.85, 0.75]
            );
            revolve!(
                format!("foot{k}"),
                &[
                    (0.0, 0.0),
                    (0.022, 0.0),
                    (0.024, 0.008),
                    (0.016, 0.022),
                    (0.0, 0.024)
                ],
                at = F.at(0.2 * c, 0.0, 0.2 * s),
                material = M.oak,
                segments = 10
            );
        }
        for (k, y) in [1.62, 1.62, 1.62, 1.62, 1.44, 1.44].into_iter().enumerate() {
            let a = radians(
                (k * 90 + if k > 3 { 45 } else { 0 } + if k == 5 { 180 } else { 0 }) as f64,
            );
            let (c, s) = (cos(a), sin(a));
            let pts: Vec<(f64, f64, f64)> =
                [(0.015, 0.0), (0.06, 0.004), (0.1, 0.03), (0.117, 0.06)]
                    .iter()
                    .map(|&(r, h)| (r * c, y + h, r * s))
                    .collect();
            kit::tube!(
                F,
                format!("hook{k}"),
                &pts,
                0.0055,
                M.brass,
                verts = 6,
                round = 3
            );
            fsphere!(
                F,
                format!("hook_tip{k}"),
                0.011,
                (0.12 * c, y + 0.07, 0.12 * s),
                M.brass,
                segments = 8,
                rings = 6
            );
        }
        // a coat hanging from a hook on the right-front, a scarf over the lower left one, a hat on the top hook
        coat();
        let (sx, sy, sz) = (0.085, 1.5, -0.085);
        for (side, d) in [("front", 1.0), ("back", -1.0)] {
            fsweep!(
                F,
                format!("scarf_{side}"),
                &[
                    (sx, sy + 0.012, sz),
                    (sx + 0.02 * d, sy - 0.02, sz + 0.028 * d),
                    (sx + 0.035 * d, sy - 0.2, sz + 0.035 * d),
                    (sx + 0.03 * d, sy - 0.42 + 0.06 * d, sz + 0.03 * d)
                ],
                0,
                M.scarf,
                profile = rounded_rect(0.07, 0.012, 0.005, 1),
                up = (1, 0, -1),
                resolution = 3
            );
            let end = (sx + 0.03 * d, sy - 0.42 + 0.06 * d, sz + 0.03 * d);
            for i in 0..5 {
                let o = (i as f64 - 2.0) * 0.012 * 0.7;
                kit::tube!(
                    F,
                    format!("fringe_{side}{i}"),
                    &[
                        (end.0 + o, end.1, end.2 + o),
                        (end.0 + o, end.1 - 0.035, end.2 + o)
                    ],
                    0.0025,
                    M.scarf,
                    verts = 4,
                    round = 1
                );
            }
        }
        revolve!(
            "hat",
            &[
                (0.0, 0.0),
                (0.12, 0.0),
                (0.122, 0.006),
                (0.11, 0.012),
                (0.072, 0.012),
                (0.068, 0.03),
                (0.066, 0.07),
                (0.058, 0.088),
                (0.03, 0.092),
                (0.012, 0.084),
                (0.0, 0.086)
            ],
            at = F.at(-0.09, 1.684, -0.06),
            material = M.hat,
            segments = 24
        );
        torus!(
            "hat_band",
            0.069,
            0.007,
            at = F.at(-0.09, 1.703, -0.06),
            material = M.scarf,
            major_segments = 24,
            minor_segments = 4
        );
    });
    join_all("coatRack");
    settle_all()
}

// --- D2: the hanging plant --------------------------------------------------------------------------------
const CEILING: f64 = 2.8;

/// A pot with a rolled rim in a macramé hanger: a cord from the ceiling hook to a ring, a gathering knot, three
/// knotted strands that cross into a net round the pot and gather again in a tasselled knot below it; leafy, its
/// vines trailing with heart leaves. It sways when watered: the whole hanger swings about the hook.
pub fn hanging_plant() -> Vec<Obj> {
    let still = new_parts(|| {
        revolve!(
            "hook_plate",
            &[
                (0.0, 0.0),
                (0.008, 0.0),
                (0.012, 0.004),
                (0.03, 0.008),
                (0.03, 0.012),
                (0.0, 0.012)
            ],
            at = F.at(0, CEILING - 0.012, 0),
            material = M.cord,
            segments = 14
        );
    });
    join(&still, "body", Some(P3::ZERO));
    let parts = new_parts(|| {
        kit::tube!(
            F,
            "cord",
            &[(0, CEILING - 0.01, 0), (0, 1.84, 0)],
            0.005,
            M.cord,
            verts = 6,
            round = 1
        );
        torus!(
            "ring",
            0.016,
            0.004,
            at = F.at(0, 1.825, 0),
            material = M.cord,
            rot = (90, 0, 0),
            major_segments = 10,
            minor_segments = 4
        );
        fsphere!(
            F,
            "gather_knot",
            0.016,
            (0, 1.79, 0),
            M.cord,
            scale = (1, 1, 1.4),
            segments = 8,
            rings = 6
        );
        let corners: Vec<(f64, f64)> = (0..3)
            .map(|k| {
                (
                    cos(radians((90 + k * 120) as f64)),
                    sin(radians((90 + k * 120) as f64)),
                )
            })
            .collect();
        for (k, &(c, s)) in corners.iter().enumerate() {
            kit::tube!(
                F,
                format!("hanger{k}"),
                &[
                    (0.008 * c, 1.78, 0.008 * s),
                    (0.1 * c, 1.6, 0.1 * s),
                    (0.162 * c, 1.44, 0.162 * s)
                ],
                0.0045,
                M.cord,
                verts = 5,
                round = 2
            );
            for (i, h) in [1.62, 1.5].into_iter().enumerate() {
                let r = 0.008 + (0.162 - 0.008) * (1.78 - h) / 0.34;
                fsphere!(
                    F,
                    format!("knot{k}_{i}"),
                    0.009,
                    (r * c, h, r * s),
                    M.cord,
                    scale = (1, 1, 1.3),
                    segments = 6,
                    rings = 4
                );
            }
            // the strand splits: each half crosses to meet its neighbour's under the pot's belly
            let (c2, s2) = corners[(k + 1) % 3];
            let (mut mc, mut ms) = ((c + c2) / 2.0, (s + s2) / 2.0);
            let ml = artkit::py::hypot(mc, ms);
            (mc, ms) = (mc / ml, ms / ml);
            for (j, (ec, es)) in [(c, s), (c2, s2)].into_iter().enumerate() {
                let (dc, ds) = (ec + mc, es + ms);
                let dl = artkit::py::hypot(dc, ds);
                kit::tube!(
                    F,
                    format!("net{k}_{j}"),
                    &[
                        (0.162 * ec, 1.44, 0.162 * es),
                        (0.157 * dc / dl, 1.37, 0.157 * ds / dl),
                        (0.146 * mc, 1.3, 0.146 * ms)
                    ],
                    0.0035,
                    M.cord,
                    verts = 4,
                    round = 2
                );
            }
            fsphere!(
                F,
                format!("net_knot{k}"),
                0.008,
                (0.146 * mc, 1.3, 0.146 * ms),
                M.cord,
                segments = 6,
                rings = 4
            );
            kit::tube!(
                F,
                format!("strand{k}"),
                &[
                    (0.146 * mc, 1.3, 0.146 * ms),
                    (0.125 * mc, 1.235, 0.125 * ms),
                    (0.07 * mc, 1.19, 0.07 * ms),
                    (0.01 * mc, 1.165, 0.01 * ms)
                ],
                0.004,
                M.cord,
                verts = 4,
                round = 2
            );
        }
        fsphere!(
            F,
            "base_knot",
            0.02,
            (0, 1.155, 0),
            M.cord,
            scale = (1, 1, 1.3),
            segments = 8,
            rings = 6
        );
        for i in 0..6 {
            let a = radians((i * 60) as f64);
            kit::tube!(
                F,
                format!("tassel{i}"),
                &[
                    (0.006 * cos(a), 1.14, 0.006 * sin(a)),
                    (0.016 * cos(a), 1.06, 0.016 * sin(a))
                ],
                0.0035,
                M.cord,
                verts = 4,
                round = 1
            );
        }
        revolve!(
            "pot",
            &[
                (0.0, 0.0),
                (0.1, 0.0),
                (0.104, 0.006),
                (0.142, 0.12),
                (0.15, 0.2),
                (0.16, 0.205),
                (0.163, 0.218),
                (0.16, 0.24),
                (0.148, 0.24),
                (0.14, 0.215),
                (0.13, 0.13),
                (0.0, 0.13)
            ],
            at = F.at(0, 1.2, 0),
            material = M.pot_white,
            segments = 28
        );
        torus!(
            "pot_band",
            0.148,
            0.007,
            at = F.at(0, 1.33, 0),
            material = M.cord,
            major_segments = 28,
            minor_segments = 4
        );
        for (k, (dx, dy, dz, r)) in [
            (0.0, 1.5, 0.0, 0.17),
            (-0.12, 1.47, 0.06, 0.11),
            (0.12, 1.46, -0.05, 0.11),
            (0.04, 1.47, 0.13, 0.1),
            (-0.04, 1.46, -0.13, 0.1),
        ]
        .into_iter()
        .enumerate()
        {
            let l = sphere!(
                format!("leaves{k}"),
                r,
                at = F.at(dx, dy, dz),
                material = M.leaf,
                scale = (1, 1, 0.75),
                segments = 14,
                rings = 8
            );
            jitter!(l, 0.02, size = 0.05, seed = k as f64 * 0.9);
        }
        for (k, (x, z, length)) in [
            (0.18, 0.08, 0.45),
            (-0.17, -0.06, 0.38),
            (0.05, -0.17, 0.3),
            (-0.06, 0.16, 0.34),
        ]
        .into_iter()
        .enumerate()
        {
            let mut pts: Vec<(f64, f64, f64)> = (0..7)
                .map(|i| {
                    let t = i as f64 / 6.0;
                    (
                        x * (0.8 + 0.2 * t) + 0.012 * sin(7.0 * t + k as f64),
                        1.42 - length * t,
                        z * (0.8 + 0.2 * t),
                    )
                })
                .collect();
            let last = pts[pts.len() - 1];
            *pts.last_mut().expect("a vine has points") = (last.0, 1.42 - length + 0.042, last.2);
            fsweep!(
                F,
                format!("vine{k}"),
                &pts,
                0.004,
                M.leaf_light,
                verts = 4,
                resolution = 2
            );
            for (i, &p) in pts.iter().enumerate().skip(1) {
                heart(
                    &format!("vine{k}_leaf{i}"),
                    p,
                    0.05 - 0.004 * i as f64,
                    50 * ((i * 3 + k) % 7) as i64 - 150,
                    150 + 10 * (i % 2) as i64,
                    if (i + k) % 2 == 1 {
                        M.leaf
                    } else {
                        M.leaf_light
                    },
                );
            }
        }
    });
    let plant = hinged(join(&parts, "plant", None), F.at(0, CEILING, 0));
    settle!(floor = false);
    // one sway as it's watered: out (eased) and, played back after the hold, home; a rock (out, past rest and home)
    // played back would swing it twice
    swing!(plant, "sway", 5.0, seconds = 0.8, axis = Axis::X);
    meshes()
}
