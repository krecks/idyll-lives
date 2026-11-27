//! The rowing machine: the fan in its round cage at the front, a monitor on an arm, a rail on the floor, a sliding seat and a handle on its chain.
//! `stroke` (a loop, 2.7 s): the drive (0.9 s: the legs push the seat back, then the arms pull the handle to the
//! chest), then the slower recovery (the hands away, the body over, the seat sliding back up to the catch).

use super::M;
use artkit::math::{P3, atan2, cos, degrees, sin};
use artkit::motion::{self, Path};
use artkit::parts::{hinged, join, new_parts};
use artkit::place::GameFrame;
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fcyl, fsphere, settle, torus};

/// The machine's back edge (game depth): the housing on the wall's face in a 3-deep footprint.
const BACK: f64 = -1.43;
/// The rower's hips at the catch (game depth): the content's seat (`forward`), 1.0 m in front of the back.
const CATCH: f64 = BACK + 1.0;
/// The seat's top (game height): the content's `seat.height`.
const SEAT: f64 = 0.33;
/// Where the chain leaves the housing (game height, depth) and where the handle rests, at the catch: the handle
/// 0.36 m ahead of the hips and 0.25 m over the seat, where the resident's `row` clip puts the palms.
const PIVOT: (f64, f64) = (SEAT + 0.21, CATCH - 0.58);
const HANDLE: (f64, f64) = (SEAT + 0.25, CATCH - 0.36);
/// The footplate's slope (deg from level, rising to the housing) and its face's middle (game height, depth): the
/// soles of the `row` clip's feet, its ankles 0.34 m ahead of the hips and 0.02 m over the seat, the toes 40° up.
const PLATE_DEG: f64 = 40.0;
const PLATE: (f64, f64) = (SEAT - 0.011, CATCH - 0.431);
/// The machine's length (m), to the rear foot.
const LENGTH: f64 = 2.15;
/// The stroke's keys: (seconds, how far back the seat is, how far the handle is along its chain's line), metres.
const STROKE: [(f64, f64, f64); 6] = [
    (0.0, 0.0, 0.0),
    (0.6, 0.40, 0.44),
    (0.9, 0.45, 0.70),
    (1.25, 0.45, 0.48),
    (1.5, 0.45, 0.42),
    (2.7, 0.0, 0.0),
];

pub fn rowing_machine() -> Vec<Obj> {
    let f = GameFrame::new(0.0);
    let still = new_parts(|| still(&f));
    join(&still, "body", Some(P3::ZERO));
    let seat = new_parts(|| {
        fbox!(
            f,
            "seat",
            0.40,
            0.08,
            0.28,
            (0, SEAT - 0.04, CATCH + 0.04),
            M.pads,
            bevel = 0.02
        );
        fbox!(
            f,
            "carriage",
            0.30,
            0.06,
            0.22,
            (0, SEAT - 0.11, CATCH + 0.04),
            M.dark,
            bevel = 0.008
        );
    });
    // (named so it is the clip's first part: the file keys parts by name, and riders move with the first, `ride`)
    let seat = join(&seat, "carriage", None);
    let handle = new_parts(|| {
        fbox!(
            f,
            "handle",
            0.56,
            0.035,
            0.035,
            (0, HANDLE.0, HANDLE.1),
            M.chrome,
            bevel = 0.01
        );
        for (i, sx) in [-1.0f64, 1.0].into_iter().enumerate() {
            fbox!(
                f,
                format!("grip{i}"),
                0.09,
                0.045,
                0.045,
                (sx * 0.235, HANDLE.0, HANDLE.1),
                M.dark,
                bevel = 0.012
            );
        }
    });
    let handle = join(&handle, "handle", None);
    // the chain, built level back from the pivot and tilted up to the handle on its node, so it stretches along
    // its own length as the handle is pulled along the same line
    let (rise, run) = (HANDLE.0 - PIVOT.0, HANDLE.1 - PIVOT.1);
    let len = (rise * rise + run * run).sqrt();
    let tilt = atan2(rise, run);
    let chain = new_parts(|| {
        fbox!(
            f,
            "chain",
            0.02,
            0.02,
            len,
            (0, PIVOT.0, PIVOT.1 + len / 2.0),
            M.dark
        );
    });
    let chain = hinged(join(&chain, "chain", None), f.at(0, PIVOT.0, PIVOT.1));
    chain.set_rotation((-degrees(tilt), 0.0, 0.0));
    settle!();
    let seat_keys: Vec<_> = STROKE
        .iter()
        .map(|&(t, s, _)| (t, (0.0, -s, 0.0), (0.0, 0.0, 0.0)))
        .collect();
    motion::path(seat, "stroke", &seat_keys, false);
    let handle_keys: Vec<_> = STROKE
        .iter()
        .map(|&(t, _, h)| (t, (0.0, -h * cos(tilt), h * sin(tilt)), (0.0, 0.0, 0.0)))
        .collect();
    motion::path(handle, "stroke", &handle_keys, false);
    let chain_keys: Vec<_> = STROKE.iter().map(|&(t, _, h)| (t, 1.0 + h / len)).collect();
    motion::keys(chain, "stroke", Path::Scale, 1, &chain_keys);
    meshes()
}

/// The housing, the footplate, the rail and the rear foot.
fn still(f: &GameFrame) {
    // the fan's cage at the front: a drum across the machine, its mesh dark, a rim, a hub and six spokes on each
    // face; on a foot, the rail running into it
    let (r, wide) = (0.24, 0.28);
    let (cy, cz) = (0.06 + r, BACK + r);
    fcyl!(
        f,
        "cage",
        r - 0.01,
        wide - 0.01,
        (0, cy, cz),
        M.dark,
        rot = (0, 90, 0),
        verts = 24
    );
    for (i, sx) in [-1.0f64, 1.0].into_iter().enumerate() {
        let x = sx * wide / 2.0;
        torus!(
            format!("rim{i}"),
            r - 0.012,
            0.012,
            at = f.at(x, cy, cz),
            material = M.body,
            rot = (0, 90, 0),
            major_segments = 24,
            minor_segments = 4
        );
        fcyl!(
            f,
            format!("hub{i}"),
            0.045,
            0.02,
            (x, cy, cz),
            M.body,
            rot = (0, 90, 0),
            verts = 12
        );
        for k in 0..6 {
            let a = (30.0 + 60.0 * k as f64).to_radians();
            let mid = (r + 0.045) / 2.0;
            fbox!(
                f,
                format!("spoke{i}"),
                0.012,
                r - 0.045,
                0.022,
                (x, cy + mid * cos(a), cz + mid * sin(a)),
                M.body,
                rot = (a.to_degrees(), 0, 0),
                bevel = 0
            );
        }
    }
    fbox!(
        f,
        "cage_foot",
        0.50,
        0.06,
        0.10,
        (0, 0.03, cz),
        M.dark,
        bevel = 0.01
    );
    // the neck the chain leaves from, up off the cage's top, and the monitor on its arm, facing the rower
    fbox!(
        f,
        "neck",
        0.09,
        0.20,
        0.10,
        (0, PIVOT.0 - 0.06, PIVOT.1 - 0.04),
        M.body,
        bevel = 0.012
    );
    fbox!(
        f,
        "arm",
        0.03,
        0.30,
        0.03,
        (0, PIVOT.0 + 0.18, PIVOT.1 - 0.05),
        M.dark,
        bevel = 0.006
    );
    fbox!(
        f,
        "monitor",
        0.17,
        0.13,
        0.04,
        (0, PIVOT.0 + 0.38, PIVOT.1 - 0.03),
        M.dark,
        bevel = 0.01
    );
    fbox!(
        f,
        "screen",
        0.13,
        0.09,
        0.004,
        (0, PIVOT.0 + 0.385, PIVOT.1 - 0.008),
        M.screen,
        bevel = 0
    );
    // the footplate, sloped to the soles (its board behind its face), on a stretcher across the rails
    let a = PLATE_DEG.to_radians();
    let (sa, ca) = (sin(a), cos(a));
    fbox!(
        f,
        "footplate",
        0.40,
        0.30,
        0.03,
        (0, PLATE.0 - 0.015 * ca, PLATE.1 - 0.015 * sa),
        M.pads,
        bevel = 0.01,
        rot = (90.0 - PLATE_DEG, 0, 0)
    );
    fbox!(
        f,
        "stretcher",
        0.44,
        0.05,
        0.06,
        (0, 0.205, CATCH - 0.33),
        M.dark,
        bevel = 0.008
    );
    // the rail: two beams from the housing back to the rear foot, the carriage on them
    let rail = LENGTH - 1.0 + 0.6;
    for (i, sx) in [-1.0f64, 1.0].into_iter().enumerate() {
        fbox!(
            f,
            format!("rail{i}"),
            0.07,
            0.11,
            rail,
            (sx * 0.11, 0.135, CATCH - 0.60 + rail / 2.0),
            M.steel,
            bevel = 0.008
        );
    }
    // the rear foot, under the rail's end
    fbox!(
        f,
        "foot",
        0.34,
        0.08,
        0.12,
        (0, 0.04, BACK + LENGTH - 0.06),
        M.dark,
        bevel = 0.01
    );
    fsphere!(f, "pivot", 0.03, (0, PIVOT.0, PIVOT.1), M.chrome);
}
