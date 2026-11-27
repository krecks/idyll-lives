//! The rowing machine: a front housing with the fan, a rail on the floor, a sliding seat and a handle on its chain.
//! `stroke` (a loop, 2.7 s): the drive (0.9 s: the legs push the seat back, then the arms pull the handle to the
//! chest), then the slower recovery (the hands away, the body over, the seat sliding back up to the catch).

use super::M;
use artkit::math::{P3, atan2, cos, degrees, sin};
use artkit::motion::{self, Path};
use artkit::parts::{hinged, join, new_parts};
use artkit::place::GameFrame;
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fsphere, settle};

/// The machine's back edge (game depth): on the wall's face in a 2-deep footprint.
const BACK: f64 = -0.93;
/// Where the chain leaves the housing (game height, depth) and where the handle rests, at the catch.
const PIVOT: (f64, f64) = (0.46, BACK + 0.40);
const HANDLE: (f64, f64) = (0.50, BACK + 0.62);
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
            (0, 0.21, BACK + 0.98),
            M.pads,
            bevel = 0.02
        );
        fbox!(
            f,
            "carriage",
            0.30,
            0.06,
            0.22,
            (0, 0.14, BACK + 0.98),
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

/// The housing, the rail and the rear foot.
fn still(f: &GameFrame) {
    // the front housing: the fan in its cage, on the floor
    fbox!(
        f,
        "housing",
        0.52,
        0.44,
        0.34,
        (0, 0.22, BACK + 0.19),
        M.body,
        bevel = 0.03
    );
    fbox!(
        f,
        "cage",
        0.30,
        0.30,
        0.02,
        (0, 0.24, BACK + 0.37),
        M.dark,
        bevel = 0.01
    );
    fbox!(
        f,
        "footplate",
        0.40,
        0.26,
        0.05,
        (0, 0.30, BACK + 0.42),
        M.pads,
        bevel = 0.01
    );
    // the rail: two beams on the floor, back to the housing
    for (i, sx) in [-1.0f64, 1.0].into_iter().enumerate() {
        fbox!(
            f,
            format!("rail{i}"),
            0.07,
            0.11,
            1.34,
            (sx * 0.11, 0.055, BACK + 0.36 + 0.67),
            M.steel,
            bevel = 0.008
        );
    }
    // the rear foot
    fbox!(
        f,
        "foot",
        0.34,
        0.07,
        0.12,
        (0, 0.035, BACK + 1.66),
        M.dark,
        bevel = 0.01
    );
    fsphere!(f, "pivot", 0.03, (0, PIVOT.0, PIVOT.1), M.chrome);
}
