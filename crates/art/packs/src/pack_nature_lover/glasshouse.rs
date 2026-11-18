//! The fern glasshouse (2x2): a white-framed glass room, a bench for two, ferns, orchids and palms.

use super::common::{_m, _pot, _rosette, M};
use super::ferns::{_fern, _orchid, _palm};
use artkit::forms::{rect, smooth_outline};
use artkit::json::float_repr;
use artkit::math::{Axis, P3, atan2, cosd, degrees, sind};
use artkit::motion::Ease;
use artkit::parts::{hinged, join, new_parts};
use artkit::place::{Facing, GameFrame};
use artkit::py::{hypot, int};
use artkit::scene::Obj;
use artkit::{boulder, fbox, fextrude, fsweep, settle, soft, swing};

// width, depth, wall height, ridge height
const GW: f64 = 1.86;
const GD: f64 = 1.86;
const HW: f64 = 2.0;
const HR: f64 = 2.45;
const DOOR: f64 = 0.8; // the doorway's width, in the middle of the front
const VENT: (f64, f64) = (0.3, 0.55); // the roof vent: half its length (depth), its length down the slope

/// The white frame: corner posts, eaves, gable rafters, the ridge, glazing bars, the doorway, gutters along the
/// eaves.
fn _glasshouse_frame(f: &GameFrame, hx: f64, hz: f64, slope: f64, run: f64) {
    for x in [-hx, hx] {
        for z in [-hz, hz] {
            fbox!(
                f,
                format!("post{}{}", float_repr(x), float_repr(z)),
                0.05,
                HW,
                0.05,
                (x, HW / 2.0, z),
                M.frame
            );
        }
    }
    for z in [-hz, hz] {
        fbox!(
            f,
            format!("eave{}", float_repr(z)),
            GW,
            0.05,
            0.05,
            (0, HW, z),
            M.frame
        );
        for s in [-1, 1] {
            fbox!(
                f,
                format!("rafter{}{s}", float_repr(z)),
                run,
                0.05,
                0.05,
                (s as f64 * GW / 4.0, (HW + HR) / 2.0, z),
                M.frame,
                rot = (0, -s as f64 * slope, 0)
            );
        }
    }
    for x in [-hx, hx] {
        fbox!(
            f,
            format!("eave_side{}", float_repr(x)),
            0.05,
            0.05,
            GD,
            (x, HW, 0),
            M.frame
        );
        fsweep!(
            f,
            format!("gutter{}", float_repr(x)),
            &[
                (x * 1.015, HW + 0.01, -GD / 2.0),
                (x * 1.015, HW + 0.01, GD / 2.0)
            ],
            0.0,
            M.frame,
            profile = vec![
                (0.0, 0.02),
                (-0.02, 0.0),
                (0.0, -0.02),
                (0.003, -0.016),
                (-0.014, 0.0),
                (0.003, 0.016)
            ],
            up = (1, 0, 0)
        );
    }
    fbox!(f, "ridge", 0.06, 0.06, GD, (0, HR, 0), M.frame);
    for z in [-0.45, 0.0, 0.45] {
        for x in [-hx, hx] {
            fbox!(
                f,
                format!("bar{}{}", float_repr(z), float_repr(x)),
                0.03,
                HW - 0.15,
                0.03,
                (x, HW / 2.0 + 0.05, z),
                M.frame
            );
        }
        for s in [-1, 1] {
            fbox!(
                f,
                format!("roof_bar{}{s}", float_repr(z)),
                run,
                0.03,
                0.03,
                (s as f64 * GW / 4.0, (HW + HR) / 2.0 + 0.01, z),
                M.frame,
                rot = (0, -s as f64 * slope, 0)
            );
        }
    }
    for x in [-0.45, 0.0, 0.45] {
        fbox!(
            f,
            format!("bar_back{}", float_repr(x)),
            0.03,
            HW - 0.15,
            0.03,
            (x, HW / 2.0 + 0.05, -hz),
            M.frame
        );
    }
    fbox!(f, "back_rail", GW, 0.04, 0.04, (0, 0.95, -hz), M.frame);
    for s in [-1, 1] {
        let sf = s as f64;
        fbox!(
            f,
            format!("door_post{s}"),
            0.05,
            HW,
            0.05,
            (sf * (DOOR / 2.0 + 0.025), HW / 2.0, hz),
            M.frame
        );
        fbox!(
            f,
            format!("front_rail{s}"),
            (GW - DOOR) / 2.0 - 0.08,
            0.04,
            0.04,
            (sf * (GW + DOOR) / 4.0, 0.95, hz),
            M.frame
        );
        fbox!(
            f,
            format!("side_rail{s}"),
            0.04,
            0.04,
            GD - 0.08,
            (sf * hx, 0.95, 0),
            M.frame
        );
    }
    fbox!(f, "lintel", DOOR + 0.1, 0.05, 0.05, (0, 1.92, hz), M.frame);
}

/// The glass: back and side walls, the front either side of the door, the gable ends, the roof round its vent.
fn _glasshouse_glass(f: &GameFrame, hx: f64, hz: f64, slope: f64, run: f64) {
    let (vz, vl) = VENT;
    fbox!(
        f,
        "glass_back",
        GW - 0.08,
        HW - 0.15,
        0.012,
        (0, HW / 2.0 + 0.05, -hz),
        M.glass
    );
    for x in [-hx, hx] {
        fbox!(
            f,
            format!("glass_side{}", float_repr(x)),
            0.012,
            HW - 0.15,
            GD - 0.08,
            (x, HW / 2.0 + 0.05, 0),
            M.glass
        );
    }
    for s in [-1, 1] {
        fbox!(
            f,
            format!("glass_front{s}"),
            (GW - DOOR) / 2.0 - 0.08,
            HW - 0.15,
            0.012,
            (s as f64 * (GW + DOOR) / 4.0, HW / 2.0 + 0.05, hz),
            M.glass
        );
    }
    fbox!(
        f,
        "glass_transom",
        DOOR,
        0.06,
        0.012,
        (0, 1.97, hz),
        M.glass
    );
    for z in [-hz, hz] {
        // the gables: thin triangular slabs
        fextrude!(
            f,
            format!("glass_gable{}", float_repr(z)),
            &[
                (-GW / 2.0 + 0.05, HW + 0.025),
                (GW / 2.0 - 0.05, HW + 0.025),
                (0.0, HR - 0.02)
            ],
            0.012,
            (0, 0, z),
            M.glass
        );
    }
    fbox!(
        f,
        "glass_roof_l",
        run,
        0.012,
        GD - 0.04,
        (-GW / 4.0, (HW + HR) / 2.0 + 0.02, 0),
        M.glass,
        rot = (0, slope, 0)
    );
    for s in [-1, 1] {
        let d = (GD - 0.04) / 2.0 - vz;
        fbox!(
            f,
            format!("glass_roof_r{s}"),
            run,
            0.012,
            d,
            (GW / 4.0, (HW + HR) / 2.0 + 0.02, s as f64 * (vz + d / 2.0)),
            M.glass,
            rot = (0, -slope, 0)
        );
    }
    let low = run - vl; // the part of the slope below the vent
    let t = 1.0 - (low / 2.0) / run; // where its middle is, from the ridge
    fbox!(
        f,
        "glass_roof_r_low",
        low,
        0.012,
        2.0 * vz,
        (GW / 2.0 * t, HR - (HR - HW) * t + 0.02, 0),
        M.glass,
        rot = (0, -slope, 0)
    );
}

pub fn glasshouse() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let (hx, hz) = (GW / 2.0 - 0.03, GD / 2.0 - 0.03);
    let slope = degrees(atan2(HR - HW, GW / 2.0));
    let run = hypot(GW / 2.0, HR - HW);
    let still = new_parts(|| {
        fbox!(
            f,
            "base",
            GW,
            0.12,
            GD,
            (0, 0.06, 0),
            M.stone,
            bevel = 0.01,
            bevel_segments = 1
        );
        fbox!(
            f,
            "gravel",
            GW - 0.06,
            0.012,
            GD - 0.06,
            (0, 0.126, 0),
            M.gravel
        );
        for k in 0..4 {
            // stepping stones from the door to the bench
            boulder!(
                format!("step{k}"),
                0.11,
                at = f.at(0.05 * (k % 2) as f64, 0.124, 0.75 - k as f64 * 0.2),
                material = M.stone,
                seed = k as f64 * 1.7,
                subdivisions = 1,
                scale = (1.3, 1.0, 0.12)
            );
        }
        _glasshouse_frame(f, hx, hz, slope, run);
        _glasshouse_glass(f, hx, hz, slope, run);
        // the bench for two (seat top 0.45 at depth 0): slats on shaped ends, a slatted back, stuffed cushions
        for k in 0..4 {
            fbox!(
                f,
                format!("seat_slat{k}"),
                1.4,
                0.03,
                0.095,
                (0, 0.405, -0.17 + k as f64 * 0.1),
                M.cedar,
                bevel = 0.006,
                bevel_segments = 1
            );
        }
        for k in 0..3 {
            fbox!(
                f,
                format!("back_slat{k}"),
                1.4,
                0.07,
                0.025,
                (0, 0.56 + k as f64 * 0.1, -0.25 - k as f64 * 0.018),
                M.cedar,
                rot = (10, 0, 0),
                bevel = 0.006,
                bevel_segments = 1
            );
        }
        let end = smooth_outline(
            &[
                (-0.19, 0.0),
                (-0.13, 0.0),
                (-0.1, 0.06),
                (0.1, 0.06),
                (0.13, 0.0),
                (0.19, 0.0),
                (0.19, 0.39),
                (-0.17, 0.39),
                (-0.19, 0.39),
            ],
            1,
            true,
        );
        for x in [-0.64, 0.64] {
            let outline: Vec<(f64, f64)> = end.iter().map(|&(u, v)| (-0.02 + u, v)).collect();
            fextrude!(
                f,
                format!("bench_end{}", float_repr(x)),
                &outline,
                0.05,
                (x, 0, 0),
                M.cedar_dark,
                plane = Facing::Side,
                bevel = 0.006,
                bevel_segments = 1
            );
            fsweep!(
                f,
                format!("bench_back_post{}", float_repr(x)),
                &[(x, 0.39, -0.21), (x, 0.82, -0.29)],
                0.0,
                M.cedar_dark,
                profile = rect(0.04, 0.04),
                up = (0, 1, 0)
            );
        }
        for x in [-0.36, 0.36] {
            soft!(
                f,
                format!("cushion{}", float_repr(x)),
                0.62,
                0.03,
                0.38,
                (x, 0.435, -0.01),
                M.cushion,
                round = 0.014,
                puff = 0.008,
                flat = 2,
                round_segments = 1
            );
        }
        // the back shelf: orchids and ferns
        fbox!(
            f,
            "shelf",
            1.7,
            0.04,
            0.3,
            (0, 0.78, -0.72),
            M.cedar,
            bevel = 0.006,
            bevel_segments = 1
        );
        for x in [-0.72, 0.72] {
            fbox!(
                f,
                format!("shelf_end{}", float_repr(x)),
                0.04,
                0.66,
                0.26,
                (x, 0.45, -0.72),
                M.cedar_dark,
                bevel = 0.006,
                bevel_segments = 1
            );
        }
        for (i, x) in [-0.32, 0.18, 0.68].into_iter().enumerate() {
            let sy = _pot(
                f,
                &format!("orchid_pot{i}"),
                x,
                0.8,
                -0.66,
                0.12,
                0.1,
                Some(M.porcelain),
                true,
                None,
            );
            _orchid(
                f,
                &format!("orchid{i}"),
                x,
                sy,
                -0.66,
                _m(["orchid", "white", "orchid_b"][i]),
            );
        }
        for (k, x) in [-0.6, -0.07, 0.43].into_iter().enumerate() {
            let sy = _pot(
                f,
                &format!("fern_pot{k}"),
                x,
                0.8,
                -0.7,
                0.16,
                0.12,
                None,
                true,
                None,
            );
            _fern(
                f,
                &format!("fern{k}"),
                x,
                sy,
                -0.7,
                [M.green, M.green_dark, M.green_light][k],
                0.24,
                7,
                k as i128,
            );
        }
        // potted palms in the front corners, hanging baskets
        for s in [-1, 1] {
            let sy = _pot(
                f,
                &format!("palm_pot{s}"),
                s as f64 * 0.64,
                0.13,
                0.6,
                0.3,
                0.26,
                None,
                true,
                None,
            );
            _palm(f, &format!("palm{s}"), s as f64 * 0.64, sy, 0.6, s);
        }
        for x in [-0.68, 0.68] {
            for k in 0..3 {
                let a = (k * 120 + 30) as f64;
                fsweep!(
                    f,
                    format!("basket_chain{}{k}", float_repr(x)),
                    &[
                        (x, HW - 0.02, -0.3),
                        (x + 0.1 * cosd(a), 1.74, -0.3 + 0.1 * sind(a))
                    ],
                    0.003,
                    M.steel,
                    verts = 3
                );
            }
            let sy = _pot(
                f,
                &format!("basket{}", float_repr(x)),
                x,
                1.6,
                -0.3,
                0.2,
                0.14,
                None,
                true,
                None,
            );
            _rosette(
                f,
                &format!("basket_leaf{}", float_repr(x)),
                x,
                sy,
                -0.3,
                7,
                0.12,
                0.06,
                &[M.green, M.green_light],
                (15.0, 45.0),
                0.5,
                "oval",
                0.3,
                None,
                3,
                3,
                int(x * 10.0) as i128,
                0.0,
            );
            for k in 0..4 {
                let a = (k * 90 + 45) as f64;
                fsweep!(
                    f,
                    format!("trailing{}{k}", float_repr(x)),
                    &[
                        (x + 0.09 * cosd(a), 1.72, -0.3 + 0.09 * sind(a)),
                        (x + 0.13 * cosd(a), 1.6, -0.3 + 0.13 * sind(a)),
                        (
                            x + 0.12 * cosd(a + 20.0),
                            1.42,
                            -0.3 + 0.12 * sind(a + 20.0)
                        )
                    ],
                    0.012,
                    M.green_dark,
                    verts = 4,
                    resolution = 2,
                    scale = [1.0, 0.6]
                );
            }
        }
    });
    join(&still, "body", Some(P3::from((0, 0, 0))));
    // The door: a glass leaf hinged on the left post, swinging out; the roof vent: a glass panel hinged at
    // the ridge, lifting. `door_open` lets the resident in and shuts behind them (and lets them out), `vent_open`
    // airs it while they're in.
    let (vz, vl) = VENT;
    let door = new_parts(|| {
        fbox!(
            f,
            "door_glass",
            DOOR - 0.01,
            1.86,
            0.014,
            (0, 0.06 + 0.93, hz),
            M.glass
        );
    });
    let door = hinged(join(&door, "door", None), f.at(-DOOR / 2.0, 1.0, hz));
    let vent = new_parts(|| {
        let t = (vl / 2.0) / run; // its middle, from the ridge
        fbox!(
            f,
            "vent_glass",
            vl,
            0.014,
            2.0 * vz,
            (GW / 2.0 * t, HR - (HR - HW) * t + 0.025, 0),
            M.glass,
            rot = (0, -slope, 0)
        );
    });
    let vent = hinged(join(&vent, "vent", None), f.at(0.0, HR + 0.01, 0));
    let objs = settle!(centre = false);
    swing!(door, "door_open", 70.0, seconds = 0.9, ease = Ease::BackOut);
    swing!(vent, "vent_open", 25.0, seconds = 0.9, axis = Axis::Y);
    objs
}
