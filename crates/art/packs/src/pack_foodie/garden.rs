//! The windowsill herb garden (1x1): a slatted cedar trough on a stand, pots on the shelf below.

use super::common::{_basil, _leaf, _pot, _rosemary, _thyme, M, WALL};
use artkit::forms::rounded_rect;
use artkit::kit;
use artkit::math::{Axis, P3};
use artkit::parts::{join, new_parts, numbered};
use artkit::place::{Facing, GameFrame};
use artkit::scene::Obj;
use artkit::{fbox, fcyl, fextrude, fsweep, grow, jitter, revolve, settle, torus};

pub fn herb_garden() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let leg = rounded_rect(0.045, 0.045, 0.008, 1);
    let still = new_parts(|| {
        for x in [-0.38, 0.38] {
            for z in [-0.40, -0.06] {
                fsweep!(
                    f,
                    numbered("leg"),
                    &[(x, 0.0, z), (x, 0.615, z)],
                    0.0,
                    M.cedar_dark,
                    profile = &leg,
                    up = (1, 0, 0)
                );
            }
            fbox!(
                f,
                numbered("side_rail"),
                0.04,
                0.03,
                0.36,
                (x, 0.6, -0.23),
                M.cedar_dark,
                bevel = 0.004,
                bevel_segments = 1
            );
            fbox!(
                f,
                numbered("shelf_rail"),
                0.03,
                0.025,
                0.36,
                (x, 0.165, -0.23),
                M.cedar_dark
            );
        }
        for z in [-0.36, -0.23, -0.10] {
            fbox!(
                f,
                numbered("shelf_slat"),
                0.78,
                0.02,
                0.09,
                (0, 0.185, z),
                M.cedar,
                bevel = 0.004,
                bevel_segments = 1
            );
        }
        for z in [-0.40, -0.06] {
            fbox!(
                f,
                numbered("top_rail"),
                0.8,
                0.03,
                0.05,
                (0, 0.6, z),
                M.cedar_dark,
                bevel = 0.004,
                bevel_segments = 1
            );
        }
        // the trough: a dark inside, corner posts, three slats a side with a gap between, a moulded rim; soil, herbs
        let (y0, w, h, d) = (0.615, 0.84, 0.39, 0.38);
        let (z0, z1) = (WALL, WALL + d);
        let zc = (z0 + z1) / 2.0;
        fbox!(
            f,
            "trough_inside",
            w - 0.05,
            h - 0.06,
            d - 0.05,
            (0, y0 + (h - 0.06) / 2.0, zc),
            M.cedar_dark
        );
        for px in [-w / 2.0 + 0.025, w / 2.0 - 0.025] {
            for pz in [z0 + 0.025, z1 - 0.025] {
                fbox!(
                    f,
                    numbered("trough_post"),
                    0.05,
                    h,
                    0.05,
                    (px, y0 + h / 2.0, pz),
                    M.cedar_dark,
                    bevel = 0.006,
                    bevel_segments = 1
                );
            }
        }
        let sh = h / 3.0;
        for k in 0..3 {
            let y = y0 + sh * (k as f64 + 0.5);
            for z in [z0 + 0.01, z1 - 0.01] {
                fbox!(
                    f,
                    numbered("trough_slat"),
                    w - 0.02,
                    sh - 0.014,
                    0.02,
                    (0, y, z),
                    M.cedar,
                    bevel = 0.004,
                    bevel_segments = 1
                );
            }
            for x in [-w / 2.0 + 0.01, w / 2.0 - 0.01] {
                fbox!(
                    f,
                    numbered("trough_slat"),
                    0.02,
                    sh - 0.014,
                    d - 0.02,
                    (x, y, zc),
                    M.cedar,
                    bevel = 0.004,
                    bevel_segments = 1
                );
            }
        }
        kit::moulding!(
            f,
            "trough_rim",
            &[
                (-w / 2.0 + 0.01, z0 + 0.0145),
                (w / 2.0 - 0.01, z0 + 0.0145),
                (w / 2.0 - 0.01, z1 - 0.005),
                (-w / 2.0 + 0.01, z1 - 0.005)
            ],
            &[
                (0.03, 0.0),
                (-0.02, 0.0),
                (-0.02, 0.014),
                (-0.014, 0.02),
                (0.03, 0.02)
            ],
            M.cedar_dark,
            at = y0 + h,
            plane = Facing::Top
        );
        let soil = y0 + h - 0.03;
        let trough_soil = fbox!(
            f,
            "trough_soil",
            w - 0.06,
            0.02,
            d - 0.06,
            (0, soil - 0.01, zc),
            M.soil
        );
        jitter!(trough_soil, 0.004, size = 0.05);
        _basil(f, -0.26, zc, soil, 1.3);
        _basil(f, -0.18, zc + 0.08, soil, 0.9);
        _rosemary(f, 0.0, zc, soil, 1.0);
        _thyme(f, 0.27, zc, soil, 1.1, 1.0);
        _thyme(f, 0.2, zc + 0.09, soil, 0.7, 2.0);
        for x in [-0.26, 0.0, 0.27] {
            // chalk tags on the trough's front
            fbox!(
                f,
                numbered("tag"),
                0.09,
                0.05,
                0.006,
                (x, y0 + h - 0.08, z1 + 0.004),
                M.chalk,
                bevel = 0.002,
                bevel_segments = 1
            );
            fbox!(
                f,
                numbered("tag_line"),
                0.05,
                0.006,
                0.002,
                (x, y0 + h - 0.08, z1 + 0.008),
                M.soil
            );
        }
        // the shelf below: the sprig's pot on a saucer, a pot of basil and a trowel
        fcyl!(
            f,
            "saucer",
            0.085,
            0.01,
            (-0.2, 0.2, -0.24),
            M.terracotta,
            verts = 14
        );
        _pot(f, -0.2, -0.24, 0.205, 0.08, 0.14);
        let top = _pot(f, 0.18, -0.25, 0.195, 0.07, 0.13);
        _basil(f, 0.18, -0.25, top, 0.8);
        fextrude!(
            f,
            "trowel_blade",
            &[
                (0.0, 0.0),
                (0.03, 0.02),
                (0.028, 0.08),
                (0.0, 0.12),
                (-0.028, 0.08),
                (-0.03, 0.02)
            ],
            0.006,
            (0.34, 0.199, -0.12),
            M.steel,
            plane = Facing::Top,
            rot = (0, 0, -155),
            bevel = 0.002,
            bevel_segments = 1
        );
        fsweep!(
            f,
            "trowel_grip",
            &[(0.33, 0.204, -0.15), (0.29, 0.21, -0.24)],
            0.014,
            M.cedar_dark,
            verts = 8,
            scale = [0.8, 1.1]
        );
        // a woven basket of picked herbs on the floor in front, its handle arching over
        let bz = 0.236;
        revolve!(
            "basket",
            &[
                (0.0, 0.0),
                (0.09, 0.0),
                (0.12, 0.09),
                (0.11, 0.09),
                (0.08, 0.012),
                (0.0, 0.012)
            ],
            at = f.at(-0.2, 0, bz),
            material = M.wicker,
            segments = 16,
            scale = (1, 0.72, 1)
        );
        for y in [0.03, 0.06] {
            let r = 0.09 + y / 3.0 + 0.003;
            torus!(
                numbered("weave"),
                1.0,
                0.004,
                at = f.at(-0.2, y, bz),
                material = M.wicker,
                major_segments = 16,
                minor_segments = 3
            )
            .set_scale((r, r * 0.72, 1.0));
        }
        kit::tube!(
            f,
            "basket_handle",
            &[
                (-0.2, 0.085, bz - 0.084),
                (-0.2, 0.2, bz),
                (-0.2, 0.085, bz + 0.084)
            ],
            0.008,
            M.wicker,
            verts = 5,
            round = 4
        );
        _basil(f, -0.2, bz, 0.04, 0.8);
    });
    join(&still, "body", Some(P3::from((0, 0, 0))));
    // The sprig, a stem and leaves in the left pot, built full size and scaled to nothing on the soil: `sprig` grows it
    // while a resident tends the herbs.
    let (sx, sz, sy) = (-0.2, -0.24, 0.205 + 0.14 * 0.9 + 0.006);
    let sprig = new_parts(|| {
        fsweep!(
            f,
            "sprig_stem",
            &[
                (sx, sy, sz),
                (sx + 0.006, sy + 0.09, sz),
                (sx + 0.01, sy + 0.17, sz)
            ],
            0.005,
            M.rosemary,
            verts = 4,
            resolution = 2,
            scale = [1.0, 0.8, 0.5]
        );
        for (k, (dy, a)) in [
            (0.06, 0),
            (0.09, 140),
            (0.12, 260),
            (0.15, 30),
            (0.175, 200),
        ]
        .into_iter()
        .enumerate()
        {
            _leaf(
                f,
                &numbered("sprig_leaf"),
                sx + 0.005,
                sy + dy,
                sz,
                0.045 - 0.004 * k as f64,
                a as f64,
                25.0,
                M.basil_light,
            );
        }
    });
    let sprig = join(&sprig, "sprig", Some(f.at(sx, sy, sz)));
    sprig.set_scale((0.001, 0.001, 0.001));
    let objs = settle!(centre = false);
    for axis in [Axis::X, Axis::Y, Axis::Z] {
        grow!(sprig, "sprig", seconds = 0.9, axis = axis);
    }
    objs
}
