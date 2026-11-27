//! The playhouse: a board hut with a gable roof, its doorway in the front gable, a window in its side, corner posts
//! and painted trim.

use super::M;
use artkit::math::{cos, radians, sin, tan};
use artkit::parts::join_all;
use artkit::place::GameFrame;
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fextrude, settle};

pub fn playhouse() -> Vec<Obj> {
    let f = GameFrame::new(0.0);
    // the walls' outer faces (w × d), the eaves' height, the roof's pitch and its boards' overhang and thickness
    let (w, d, eaves) = (1.16, 1.40, 1.06);
    let (pitch, over, over_front, th) = (40.0, 0.06, 0.07, 0.035);
    let (sa, ca, ta) = (
        sin(radians(pitch)),
        cos(radians(pitch)),
        tan(radians(pitch)),
    );
    let half = w / 2.0;
    let apex = eaves + half * ta;
    // the doorway: 0.66 m wide, 1.05 m clear over the 0.06 m floor (the crawl-in clip is fitted to it)
    let (door, door_top) = (0.66, 0.06 + 1.05);
    fbox!(
        f,
        "floor",
        w,
        0.06,
        d,
        (0, 0.03, 0),
        M.ph_board,
        bevel = 0.006
    );
    // the gable walls, front and back, their tops under the roof; the front with its doorway
    let gable = |notch: bool| {
        let mut o = vec![(-half, 0.06)];
        if notch {
            o.extend([
                (-door / 2.0, 0.06),
                (-door / 2.0, door_top),
                (door / 2.0, door_top),
                (door / 2.0, 0.06),
            ]);
        }
        o.extend([(half, 0.06), (half, eaves), (0.0, apex), (-half, eaves)]);
        o
    };
    for (name, z, notch) in [
        ("front", d / 2.0 - 0.03, true),
        ("back", -d / 2.0 + 0.03, false),
    ] {
        fextrude!(
            f,
            name,
            &gable(notch),
            0.06,
            (0, 0, z),
            M.ph_board,
            bevel = 0.004,
            bevel_segments = 1
        );
    }
    // the side walls between them, up to the eaves
    for (i, sx) in [-1.0f64, 1.0].into_iter().enumerate() {
        fbox!(
            f,
            format!("side{i}"),
            0.06,
            eaves - 0.06,
            d - 0.12,
            (sx * (half - 0.03), (eaves + 0.06) / 2.0, 0),
            M.ph_board,
            bevel = 0.006
        );
    }
    // corner posts, 1.5 cm proud of both walls they meet (flush, their faces fought the boards' and streaked)
    for (i, (sx, sz)) in [(-1.0f64, -1.0f64), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)]
        .into_iter()
        .enumerate()
    {
        fbox!(
            f,
            format!("post{i}"),
            0.09,
            eaves - 0.06,
            0.09,
            (
                sx * (half - 0.03),
                (eaves + 0.06) / 2.0,
                sz * (d / 2.0 - 0.03)
            ),
            M.house_trim,
            bevel = 0.006
        );
    }
    // the doorway's painted casing: two jambs and a head on the front, 1.5 cm proud
    let zc = d / 2.0 + 0.0075;
    for (i, sx) in [-1.0f64, 1.0].into_iter().enumerate() {
        fbox!(
            f,
            format!("jamb{i}"),
            0.06,
            door_top - 0.06,
            0.015,
            (sx * (door / 2.0 + 0.03), (door_top + 0.06) / 2.0, zc),
            M.house_trim,
            bevel = 0.003
        );
    }
    fbox!(
        f,
        "door_head",
        door + 0.12,
        0.05,
        0.015,
        (0, door_top + 0.025, zc),
        M.house_trim,
        bevel = 0.003
    );
    // the window in the left side, seen from the front right (the front either side of the doorway is too narrow
    // for one): its glass, a frame and a cross bar
    let (wx, wy, ww, wh) = (-half, 0.68, 0.36, 0.30);
    fbox!(
        f,
        "window",
        0.020,
        wh,
        ww,
        (wx + 0.005, wy, 0),
        M.ph_glass,
        bevel = 0.004
    );
    let xf = wx - 0.008;
    for (i, sy) in [-1.0f64, 1.0].into_iter().enumerate() {
        fbox!(
            f,
            format!("sill{i}"),
            0.016,
            0.05,
            ww + 0.10,
            (xf, wy + sy * (wh / 2.0 + 0.025), 0),
            M.house_trim,
            bevel = 0.003
        );
        fbox!(
            f,
            format!("stile{i}"),
            0.016,
            wh,
            0.05,
            (xf, wy, sy * (ww / 2.0 + 0.025)),
            M.house_trim,
            bevel = 0.003
        );
    }
    fbox!(
        f,
        "bar_up",
        0.012,
        wh,
        0.025,
        (xf, wy, 0),
        M.house_trim,
        bevel = 0
    );
    fbox!(
        f,
        "bar_across",
        0.012,
        0.025,
        ww,
        (xf, wy, 0),
        M.house_trim,
        bevel = 0
    );
    // the roof: two board slabs from the ridge out past the eaves, a bargeboard under each at the front and the back,
    // and a ridge board over the join
    let run = half + over; // the ridge to the eaves' edge, level
    let len = run / ca + th * ta; // up the slope, reaching over the ridge so the slabs' tops meet
    let rd = d + 2.0 * over_front;
    // a point `s` down the slope from the ridge's underside on the side `sx`, `n` out of the roof
    let at = |sx: f64, s: f64, n: f64| (sx * (s * ca + n * sa), apex - s * sa + n * ca);
    for (i, sx) in [-1.0f64, 1.0].into_iter().enumerate() {
        let (x, y) = at(sx, (run / ca - th * ta) / 2.0, th / 2.0);
        fbox!(
            f,
            format!("roof{i}"),
            len,
            th,
            rd,
            (x, y, 0),
            M.ph_roof,
            rot = (0.0, -sx * pitch, 0.0),
            bevel = 0.006
        );
        let (x, y) = at(sx, run / ca / 2.0, -0.025);
        for (k, sz) in [-1.0f64, 1.0].into_iter().enumerate() {
            fbox!(
                f,
                format!("barge{i}{k}"),
                run / ca,
                0.05,
                0.025,
                (x, y, sz * (d / 2.0 + over_front - 0.0125)),
                M.house_trim,
                rot = (0.0, -sx * pitch, 0.0),
                bevel = 0.004
            );
        }
    }
    fbox!(
        f,
        "ridge",
        0.07,
        0.04,
        rd + 0.02,
        (0, apex + th / ca + 0.01, 0),
        M.house_trim,
        bevel = 0.006
    );
    join_all("playhouse");
    settle!();
    meshes()
}
