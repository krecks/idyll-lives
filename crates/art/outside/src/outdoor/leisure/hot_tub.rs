use super::super::common::{F, M, side_slab};
use artkit::forms::{bisect, rounded_rect};
use artkit::json::float_repr;
use artkit::math::{Axis, P3, TAU, cos, cosd, sin, sind};
use artkit::motion::{PathKey, Spin, Swing, attach, last_frame, path, spin, swing};
use artkit::parts::{hinged, join, new_parts};
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fextrude, fsphere, fsweep, loft, revolve, ring, settle};

// --- the cedar hot tub: banded staves, a step, water; the cover folds back for a soak and the water bubbles ----
pub(super) const TUB_R: f64 = 0.92;
pub(super) const TUB_SEAT: f64 = 0.35; // the bench's top (`seats.hotTubCedar`) and its inner edge's radius
pub(super) const TUB_BENCH: f64 = 0.44;

pub fn hot_tub_cedar() -> Vec<Obj> {
    let tub = new_parts(|| {
        // the staves: one wall, a V between each of 32, the inside round; a floor; the rim round their tops
        let mut out = Vec::new();
        let n = 32;
        for k in 0..n {
            for (f, r) in [(0.0, TUB_R - 0.006), (0.12, TUB_R), (0.88, TUB_R)] {
                let a = TAU * (k as f64 + f) / n as f64;
                out.push((r * cos(a), r * sin(a)));
            }
        }
        let hole: Vec<(f64, f64)> = (0..n)
            .map(|k| {
                (
                    (TUB_R - 0.06) * cos(TAU * k as f64 / n as f64),
                    (TUB_R - 0.06) * sin(TAU * k as f64 / n as f64),
                )
            })
            .collect();
        fextrude!(
            F,
            "staves",
            &out,
            0.78,
            (0, 0.41, 0),
            M.cedar,
            plane = artkit::place::Facing::Top,
            holes = [hole]
        );
        revolve!(
            "floor",
            &[
                (0.0, 0.0),
                (TUB_R - 0.05, 0.0),
                (TUB_R - 0.05, 0.12),
                (0.0, 0.12)
            ],
            at = F.at(0, 0, 0),
            material = M.cedar,
            segments = 24,
            smooth = false
        );
        revolve!(
            "rim",
            &[
                (TUB_R - 0.075, 0.79),
                (TUB_R + 0.005, 0.79),
                (TUB_R + 0.01, 0.81),
                (TUB_R + 0.002, 0.826),
                (TUB_R - 0.07, 0.826),
                (TUB_R - 0.078, 0.81)
            ],
            at = F.at(0, 0, 0),
            material = M.cedar,
            segments = 32
        );
        for y in [0.2, 0.6] {
            // the bands, a lug where each is drawn tight
            let band: Vec<(f64, f64, f64)> = (0..32)
                .map(|k| {
                    (
                        (TUB_R + 0.004) * cos(TAU * k as f64 / 32.0),
                        y,
                        (TUB_R + 0.004) * sin(TAU * k as f64 / 32.0),
                    )
                })
                .collect();
            fsweep!(
                F,
                format!("band{}", float_repr(y)),
                &band,
                0,
                M.cedar_band,
                profile = rounded_rect(0.04, 0.008, 0.003, 1),
                up = (0, 1, 0),
                closed = true
            );
            fbox!(
                F,
                format!("lug{}", float_repr(y)),
                0.05,
                0.05,
                0.03,
                (0.62, y, 0.68),
                M.cedar_band,
                rot = (0, 0, 0),
                bevel = 0.006,
                bevel_segments = 1
            );
        }
        // the bench round the inside wall, under the water: the soakers' seat (`seats.hotTubCedar`)
        revolve!(
            "bench",
            &[
                (TUB_BENCH, 0.12),
                (TUB_BENCH, TUB_SEAT - 0.01),
                (TUB_BENCH + 0.01, TUB_SEAT),
                (TUB_R - 0.06, TUB_SEAT)
            ],
            at = F.at(0, 0, 0),
            material = M.cedar,
            segments = 32,
            smooth = false
        );
        // the step: a tread with a dark nosing on two shaped sides and a riser
        fbox!(
            F,
            "tread",
            0.6,
            0.03,
            0.29,
            (0, 0.185, 0.81),
            M.cedar,
            bevel = 0.006,
            bevel_segments = 1
        );
        fbox!(
            F,
            "tread_edge",
            0.6,
            0.022,
            0.03,
            (0, 0.188, 0.944),
            M.cedar_band,
            bevel = 0.005,
            bevel_segments = 1
        );
        fbox!(
            F,
            "riser",
            0.54,
            0.14,
            0.02,
            (0, 0.09, 0.93),
            M.cedar,
            bevel = 0.004,
            bevel_segments = 1
        );
        for s in [-1, 1] {
            side_slab(
                &format!("step_side{s}"),
                s as f64 * 0.285,
                &[(0.66, 0.0), (0.955, 0.0), (0.955, 0.17), (0.66, 0.17)],
                0.03,
                M.cedar,
                0.004,
            );
        }
    });
    join(&tub, "tub", Some(P3::from((0, 0, 0))));
    let water = new_parts(|| {
        revolve!(
            "water",
            &[
                (0.0, 0.69),
                (0.3, 0.688),
                (0.6, 0.692),
                (TUB_R - 0.06, 0.69),
                (TUB_R - 0.06, 0.67),
                (0.0, 0.67)
            ],
            at = F.at(0, 0, 0),
            material = M.water,
            segments = 32
        );
        for k in 0..14 {
            let a = 360.0 / 14.0 * k as f64 + ((k % 3) * 9) as f64;
            let r = 0.2 + 0.5 * ((k * 7) % 5) as f64 / 5.0;
            fsphere!(
                F,
                format!("bubble{k}"),
                0.025 + 0.01 * (k % 3) as f64,
                (r * cosd(a), 0.69, r * sind(a)),
                M.foam,
                scale = (1, 1, 0.4),
                segments = 8,
                rings = 4
            );
        }
        for k in 0..4 {
            // the jets' foam against the wall
            let a = (90 * k + 30) as f64;
            fsphere!(
                F,
                format!("jet{k}"),
                0.06,
                ((TUB_R - 0.11) * cosd(a), 0.69, (TUB_R - 0.11) * sind(a)),
                M.foam,
                scale = (1, 0.7, 0.3),
                segments = 8,
                rings = 4
            );
        }
    });
    let water = hinged(join(&water, "water", None), F.at(0, 0.68, 0));
    // The cover in two halves meeting at the seam across its middle (a fourth draw, as the shower's door, Q-05): one
    // lofted disc, each half cut from a copy of it, a hair apart at the seam, where a vinyl hinge strip joins them.
    let rings = [
        (TUB_R + 0.0, 0.84),
        (TUB_R + 0.02, 0.85),
        (TUB_R + 0.02, 0.895),
        (TUB_R + 0.005, 0.91),
        (TUB_R - 0.03, 0.912),
    ];
    // The half of the cover on side `s` of the seam (1: the front, game +z; -1: the back), cut from the disc: what
    // the cutting box (2.2 x 0.2 x 1.2 at game depth -0.597 s) covers goes, up to its face a hair past the seam.
    let half = |name: &str, s: i32| -> Obj {
        let sections: Vec<Vec<P3>> = rings
            .iter()
            .map(|&(r, y)| ring!(r, r, z = y, verts = 32))
            .collect();
        let disc = loft!(name, &sections, material = M.tub_cover, angle = 30);
        let sf = s as f32;
        let face = (sf * 0.597_f32 - sf * 0.6_f32) as f64;
        bisect(disc, (0.0, face, 0.876), (0, s, 0), true)
    };
    let back = new_parts(|| {
        half("cover", -1);
        fbox!(
            F,
            "cover_seam",
            1.84,
            0.008,
            0.02,
            (0, 0.912, -0.003),
            M.tub_cover,
            bevel = 0.003,
            bevel_segments = 1
        );
    });
    let front = new_parts(|| {
        half("cover_front", 1);
        for x in [-0.3, 0.3] {
            // the straps down to their clips, and the handles
            fbox!(
                F,
                format!("strap{}", float_repr(x)),
                0.05,
                0.06,
                0.008,
                (x, 0.865, TUB_R + 0.024),
                M.tub_cover,
                bevel = 0.002,
                bevel_segments = 1
            );
            fbox!(
                F,
                format!("clip{}", float_repr(x)),
                0.04,
                0.025,
                0.012,
                (x, 0.83, TUB_R + 0.028),
                M.tub_cover,
                bevel = 0.003,
                bevel_segments = 1
            );
            fsweep!(
                F,
                format!("handle{}", float_repr(x)),
                &[
                    (x - 0.05, 0.91, 0.6),
                    (x - 0.03, 0.919, 0.6),
                    (x + 0.03, 0.919, 0.6),
                    (x + 0.05, 0.91, 0.6)
                ],
                0.006,
                M.tub_cover,
                verts = 5,
                resolution = 2
            );
        }
    });
    // the back half's origin on its back edge, at the bottom; the front half's on the seam, at the top, about which
    // it folds over onto the back half
    let cover = hinged(join(&back, "cover", None), F.at(0, 0.84, -TUB_R - 0.02));
    let flap = hinged(join(&front, "cover_front", None), F.at(0, 0.912, 0));
    settle!(centre = false);
    attach(flap, cover);
    // Off for a soak: the front half folds back over the back half on the seam, then the two together are lifted,
    // turned down behind the tub and lowered to hang there on their folded edge, as on a cover lifter, the round
    // edge just off the ground (Blender offsets: +y is back).
    swing(
        flap,
        "cover_off",
        -180.0,
        Swing {
            seconds: 0.5,
            axis: Axis::X,
            ..Swing::default()
        },
    );
    let keys: [PathKey; 5] = [
        (0.0, (0.0, 0.0, 0.0), (0.0, 0.0, 0.0)),
        (0.5, (0.0, 0.0, 0.0), (0.0, 0.0, 0.0)),
        (0.8, (0.0, 0.0, 0.25), (0.0, 0.0, 0.0)),
        (1.1, (0.0, 0.16, 1.16), (90.0, 0.0, 0.0)),
        (1.5, (0.0, 0.16, 0.11), (90.0, 0.0, 0.0)),
    ];
    path(cover, "cover_off", &keys, false);
    // previewed hung up, at its end
    let end = last_frame(1.5);
    cover.edit(|o| {
        if let Some(t) = o.tracks.iter_mut().find(|t| t.name == "cover_off") {
            t.peak = Some(end);
        }
    });
    spin(
        water,
        "bubbles",
        Spin {
            turns: 1.0,
            seconds: 2.0,
            axis: Axis::Z,
        },
    );
    meshes()
}
