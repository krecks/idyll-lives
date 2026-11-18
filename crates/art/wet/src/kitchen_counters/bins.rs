//! The pedal bin and the recycling bins: lids hinged at their backs that lift while a plate is scraped or the
//! recycling sorted.

use super::M;
use super::common::tube;
use super::turned::lathe;
use artkit::forms::smooth_outline;
use artkit::materials::Mat;
use artkit::math::{Axis, P3, cos, radians, sin};
use artkit::parts::{hinged, join, new_parts, settle_all};
use artkit::place::{Facing, GameFrame};
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fcyl, fextrude, loft, ring, soft, swing};

// --- the pedal bin: 1x1, a stainless drum with a domed lid and a pedal (static) -----------------------
pub fn trash_can() -> Vec<Obj> {
    let f = &GameFrame::new(0.172);
    let still = new_parts(|| {
        // the drum, a little wider at the top, a rolled bead round its rim; a dark plastic foot ring
        lathe(
            f,
            "drum",
            &[
                (0.0, 0.02),
                (0.144, 0.02),
                (0.148, 0.59),
                (0.152, 0.6),
                (0.152, 0.612),
                (0.146, 0.618),
                (0.14, 0.618),
                (0.14, 0.45),
                (0.0, 0.45),
            ],
            0,
            0.0,
            -0.2,
            M.steel,
            24,
        );
        lathe(
            f,
            "base",
            &[
                (0.0, 0.0),
                (0.138, 0.0),
                (0.15, 0.006),
                (0.152, 0.03),
                (0.145, 0.034),
                (0.0, 0.034),
            ],
            0,
            0.0,
            -0.2,
            M.dark,
            24,
        );
        // the inner bucket's rim, seen when the lid lifts
        lathe(
            f,
            "bucket",
            &[
                (0.0, 0.46),
                (0.128, 0.46),
                (0.132, 0.6),
                (0.142, 0.614),
                (0.142, 0.62),
                (0.134, 0.62),
                (0.124, 0.47),
                (0.0, 0.47),
            ],
            0,
            0.0,
            -0.2,
            M.lid,
            24,
        );
        // the pedal, rounded, on its hinge block; the lid's hinge at the back
        let pedal = smooth_outline(
            &[(-0.05, -0.07), (0.05, -0.07), (0.045, 0.01), (-0.045, 0.01)],
            2,
            true,
        );
        fextrude!(
            f,
            "pedal",
            &pedal,
            0.016,
            (0, 0.035, 0),
            M.dark,
            plane = Facing::Top,
            bevel = 0.004,
            bevel_segments = 1
        );
        fbox!(
            f,
            "pedal_hinge",
            0.08,
            0.03,
            0.03,
            (0, 0.03, -0.07),
            M.dark,
            bevel = 0.006,
            bevel_segments = 1
        );
        fbox!(
            f,
            "hinge",
            0.08,
            0.03,
            0.04,
            (0, 0.625, -0.335),
            M.dark,
            bevel = 0.008,
            bevel_segments = 1
        );
    });
    join(&still, "body", Some((0, 0, 0).into()));
    // the lid, hinged at its back edge, lifting while a resident scrapes a plate into the bin
    let parts = new_parts(|| {
        lathe(
            f,
            "lid",
            &[
                (0.0, 0.617),
                (0.155, 0.617),
                (0.155, 0.632),
                (0.15, 0.638),
                (0.14, 0.652),
                (0.11, 0.669),
                (0.06, 0.681),
                (0.0, 0.685),
            ],
            0,
            0.0,
            -0.2,
            M.lid,
            24,
        );
    });
    let lid = hinged(join(&parts, "lid", None), f.at(0, 0.62, -0.355));
    settle_all();
    swing!(lid, "lid_open", -75, seconds = 0.5, axis = Axis::X);
    meshes()
}

// --- the recycling bins: 1x1, three coloured bins with lids, each lid hinged at its back; they lift one
// after another while a resident sorts the recycling (`lid0_open` .. `lid2_open`)
const BINS: [(f64, Mat); 3] = [(-0.31, M.bin1), (0.0, M.bin2), (0.31, M.bin3)];

pub fn recycling_bins() -> Vec<Obj> {
    let f = &GameFrame::new(0.212);
    let still = new_parts(|| {
        for (i, (x, m)) in BINS.into_iter().enumerate() {
            let P3 { x: bx, y: by, .. } = f.at(x, 0, -0.22);
            // the bin: a rounded tub narrowing to its foot, a rolled rim, hollow inside
            let secs = [
                ring!(0.132, 0.198, z = 0.0, verts = 16, square = 6, at = (bx, by)),
                ring!(
                    0.136,
                    0.202,
                    z = 0.015,
                    verts = 16,
                    square = 6,
                    at = (bx, by)
                ),
                ring!(
                    0.143,
                    0.208,
                    z = 0.58,
                    verts = 16,
                    square = 6,
                    at = (bx, by)
                ),
                ring!(0.145, 0.21, z = 0.6, verts = 16, square = 6, at = (bx, by)),
                ring!(0.145, 0.21, z = 0.62, verts = 16, square = 6, at = (bx, by)),
                ring!(
                    0.132,
                    0.197,
                    z = 0.62,
                    verts = 16,
                    square = 6,
                    at = (bx, by)
                ),
                ring!(
                    0.128,
                    0.193,
                    z = 0.08,
                    verts = 16,
                    square = 6,
                    at = (bx, by)
                ),
            ];
            loft!(format!("body{i}"), &secs, material = m);
            // a dark foot band, and the recycling arrows on its front
            loft!(
                format!("foot{i}"),
                &[
                    ring!(0.134, 0.2, z = 0.0, verts = 16, square = 6, at = (bx, by)),
                    ring!(
                        0.136,
                        0.202,
                        z = 0.03,
                        verts = 16,
                        square = 6,
                        at = (bx, by)
                    ),
                    ring!(0.0, 0.0, z = 0.03, verts = 16, at = (bx, by))
                ],
                material = M.lid
            );
            let tri: Vec<(f64, f64)> = (0..3)
                .map(|k| {
                    (
                        0.05 * cos(radians((90 + 120 * k) as f64)),
                        0.05 * sin(radians((90 + 120 * k) as f64)),
                    )
                })
                .collect();
            let inner: Vec<(f64, f64)> = tri.iter().map(|&(u, v)| (u * 0.55, v * 0.55)).collect();
            fextrude!(
                f,
                format!("emblem{i}"),
                &smooth_outline(&tri, 1, true),
                0.004,
                (x, 0.34, -0.012),
                M.porcelain,
                holes = vec![smooth_outline(&inner, 1, true)]
            );
            // the hand grip under the lid's front
            tube(
                f,
                format!("handle{i}"),
                &[
                    (x - 0.06, 0.6, -0.012),
                    (x - 0.06, 0.6, -0.002),
                    (x - 0.052, 0.6, 0.006),
                    (x + 0.052, 0.6, 0.006),
                    (x + 0.06, 0.6, -0.002),
                    (x + 0.06, 0.6, -0.012),
                ],
                0.009,
                M.lid,
                Some(8),
                1,
                1.0,
            );
        }
    });
    join(&still, "body", Some((0, 0, 0).into()));
    let mut lids = Vec::new();
    for (i, (x, m)) in BINS.into_iter().enumerate() {
        let parts = new_parts(|| {
            soft!(
                f,
                format!("lid{i}"),
                0.3,
                0.036,
                0.42,
                (x, 0.638, -0.23),
                m,
                round = 0.012,
                puff = (0, 0.004, 0),
                flat = 2
            );
            fbox!(
                f,
                format!("lid{i}_lip"),
                0.16,
                0.02,
                0.025,
                (x, 0.64, -0.0125),
                m,
                bevel = 0.008,
                bevel_segments = 2
            );
            for s in [-1, 1] {
                fcyl!(
                    f,
                    format!("lid{i}_hinge{s}"),
                    0.01,
                    0.06,
                    (x + s as f64 * 0.09, 0.635, -0.43),
                    m,
                    rot = (0, 90, 0),
                    verts = 8
                );
            }
        });
        lids.push(hinged(
            join(&parts, &format!("lid{i}"), None),
            f.at(x, 0.62, -0.44),
        ));
    }
    settle_all();
    for (i, &lid) in lids.iter().enumerate() {
        swing!(
            lid,
            &format!("lid{i}_open"),
            -80,
            seconds = 0.5,
            axis = Axis::X
        );
    }
    meshes()
}
