//! H1: the acoustic guitar on its stand.

use super::M;
use super::common::{frets, guitar_stand, lean, off_stand, strings};
use super::guitars::{dots, figure8, inset, neck};
use artkit::forms::smooth_outline;
use artkit::math::P3;
use artkit::parts::{Settle, join, new_parts, settle_with};
use artkit::place::GameFrame;
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fcyl, fextrude, torus};

fn acoustic(fr: &GameFrame) -> Obj {
    let p = new_parts(|| {
        // a dreadnought: a 0.51 m body, 0.40 m across the lower bout, the neck meeting it at the 14th fret, a
        // 0.645 m scale from the saddle to the nut, 1.05 m long (review M-hobbies-03: it was 1.29 m, the scale 0.97)
        let (saddle, scale) = (0.222, 0.645);
        let nut = saddle + scale;
        let body = figure8((0.17, 0.2, 0.17), (0.38, 0.145, 0.13), 8, 1);
        fextrude!(
            fr,
            "body",
            &body,
            0.1,
            (0, 0, 0),
            M.acoustic,
            bevel = 0.01,
            bevel_segments = 2
        );
        // the top inside a cream binding
        fextrude!(
            fr,
            "binding",
            &inset(&body, 0.004),
            0.003,
            (0, 0, 0.0495),
            M.bone
        );
        fextrude!(
            fr,
            "top",
            &inset(&body, 0.009),
            0.003,
            (0, 0, 0.0505),
            M.spruce
        );
        fcyl!(
            fr,
            "hole",
            0.05,
            0.004,
            (0, 0.4, 0.0515),
            M.hole,
            rot = (90, 0, 0),
            verts = 24,
            smooth = false
        );
        torus!(
            "rosette",
            0.057,
            0.003,
            at = fr.at(0, 0.4, 0.052),
            material = M.tortoise,
            rot = (90, 0, 0),
            major_segments = 28,
            minor_segments = 4
        );
        torus!(
            "rosette_ring",
            0.064,
            0.0015,
            at = fr.at(0, 0.4, 0.052),
            material = M.bone,
            rot = (90, 0, 0),
            major_segments = 28,
            minor_segments = 3
        );
        let guard = [
            (0.045, 0.36),
            (0.1, 0.335),
            (0.118, 0.29),
            (0.1, 0.255),
            (0.068, 0.268),
            (0.048, 0.31),
        ];
        fextrude!(
            fr,
            "pickguard",
            &smooth_outline(&guard, 2, true),
            0.002,
            (0, 0, 0.0525),
            M.tortoise
        );
        // the bridge: a shaped block, its saddle and six pins
        let bridge = [
            (0.075, -0.012),
            (0.075, 0.006),
            (0.04, 0.014),
            (-0.04, 0.014),
            (-0.075, 0.006),
            (-0.075, -0.012),
            (-0.04, -0.016),
            (0.04, -0.016),
        ];
        fextrude!(
            fr,
            "bridge",
            &smooth_outline(&bridge, 1, true),
            0.008,
            (0, saddle - 0.004, 0.055),
            M.fretboard,
            bevel = 0.002,
            bevel_segments = 1
        );
        fbox!(fr, "saddle", 0.075, 0.003, 0.004, (0, saddle, 0.06), M.bone);
        for k in 0..6 {
            fcyl!(
                fr,
                format!("pin{k}"),
                0.0025,
                0.004,
                (-0.03 + 0.012 * k as f64, saddle - 0.011, 0.06),
                M.bone,
                rot = (90, 0, 0),
                verts = 6
            );
        }
        neck(fr, "neck", 0.48, nut, 0.0125, 0.05, 0.024, M.neck, 0.88, 8);
        fbox!(
            fr,
            "heel",
            0.05,
            0.05,
            0.05,
            (0, 0.525, -0.01),
            M.neck,
            bevel = 0.01
        );
        fbox!(
            fr,
            "fretboard",
            0.048,
            nut - 0.455,
            0.005,
            (0, (nut + 0.455) / 2.0, 0.015),
            M.fretboard
        );
        frets(fr, 17, scale, nut, 0.048, 0.0185);
        dots(
            fr,
            "dot",
            &[3, 5, 7, 9, 12, 15],
            scale,
            nut,
            0.0176,
            M.bone,
            0.0035,
        );
        fbox!(
            fr,
            "nut",
            0.048,
            0.004,
            0.004,
            (0, nut + 0.001, 0.0185),
            M.bone
        );
        let head = [
            (0.024, nut - 0.003),
            (0.038, nut + 0.165),
            (0.03, nut + 0.185),
            (0.0, nut + 0.18),
            (-0.03, nut + 0.185),
            (-0.038, nut + 0.165),
            (-0.024, nut - 0.003),
        ];
        fextrude!(
            fr,
            "head",
            &smooth_outline(&head, 1, true),
            0.016,
            (0, 0, -0.003),
            M.neck,
            bevel = 0.003,
            bevel_segments = 1
        );
        fbox!(
            fr,
            "head_plate",
            0.05,
            0.03,
            0.002,
            (0, nut + 0.15, 0.0055),
            M.hole,
            bevel = 0.001,
            bevel_segments = 1
        );
        for s in [-1, 1] {
            let sf = s as f64;
            for k in 0..3 {
                let y = nut + 0.045 + 0.045 * k as f64;
                fcyl!(
                    fr,
                    format!("peg{s}{k}"),
                    0.004,
                    0.024,
                    (sf * 0.05, y, -0.003),
                    M.fret,
                    rot = (0, 90, 0),
                    verts = 8
                );
                fbox!(
                    fr,
                    format!("peg_button{s}{k}"),
                    0.005,
                    0.018,
                    0.012,
                    (sf * 0.064, y, -0.003),
                    M.fret
                );
                fcyl!(
                    fr,
                    format!("post{s}{k}"),
                    0.004,
                    0.012,
                    (sf * 0.018, y, 0.008),
                    M.fret,
                    rot = (90, 0, 0),
                    verts = 6
                );
            }
        }
        strings(
            fr,
            6,
            0.0095,
            0.0075,
            saddle + 0.007,
            nut + 0.001,
            0.0625,
            0.0215,
        );
    });
    join(&p, "guitar", Some(P3::ZERO))
}

pub fn guitar_acoustic() -> Vec<Obj> {
    // a little forward, where the game places it
    let f = GameFrame::new(0.07);
    let still = new_parts(|| guitar_stand(&f, -0.01, 0.105, 0.79, -0.057, 0.1, 0.27, 0.06));
    join(&still, "stand", Some(P3::ZERO));
    let guitar = lean(&f, acoustic(&GameFrame::new(0.0)), 8.0, (0.0, 0.12, 0.06));
    settle_with(
        &meshes(),
        Settle {
            centre: false,
            floor: true,
        },
    );
    off_stand(guitar);
    meshes()
}
