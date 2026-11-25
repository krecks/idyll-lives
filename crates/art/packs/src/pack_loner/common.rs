//! What the loner pack is built from: its palette, a spindle chair, a desk, a mug and a book.

use artkit::bevel::ease;
use artkit::forms::{rounded_rect, smooth_outline};
use artkit::kit;
use artkit::materials::Mat;
use artkit::math::{cosd, pow, sind};
use artkit::parts::numbered;
use artkit::place::{Facing, GameFrame};
use artkit::{fbox, fextrude, fsweep, palette, soft};

palette! { M:
    body = ("#8FA58A", satin),       // sage paint: the radio's case, the puzzle table's frame, the bonsai bench's frame
    shell = ("#8FA58A", fabric),     // the reading pod's upholstered shell and hood
    slate = ("#7E93A3", satin),      // the delivery hatch's cabinet
    oak = ("#C49A6C", satin),        // plain oak: the pod's foot, the radio desk and chair
    wood = ("#C49A6C", wood),        // textured oak: the puzzle table's top and chair, the hatch's ledge and cap
    teak = ("#B07A4E", satin),       // the bonsai bench's top and shelf
    cedar = ("#C98B58", wood),       // the sauna
    cedar_dark = ("#A86E44", satin), // the sauna's door handle, the bucket's ladle
    cushion = ("#E6DCC8", satin),    // oat cushions and linings
    headrest = ("#D8CCB4", satin),
    seatpad = ("#7E93A3", satin),    // the radio chair's slate pad
    chrome = ("#D0D3D6", chrome),
    steel = ("#C9CDD1", metal),      // the hatch door, the magnifier ring
    iron = ("#7E8389", metal),       // the sauna heater
    copper = ("#B87333", metal),
    dark = ("#3A3D41", satin),       // knobs, headphones, the lamp foot, the hatch's recess
    rig_face = ("#C9C3B0", satin),   // the receiver's front plate
    dial = ("#F2B84B", gloss),       // the receiver's dial window
    needle = ("#C8402F", gloss),
    cream = ("#F4EFE2", satin),      // paper, logbook, meter, plaque, towel
    mustard = ("#D6A64A", satin),
    terra = ("#C0714C", satin),      // terracotta pots, the mug
    glaze = ("#3E6F9E", gloss),      // the bonsai's pot, the puzzle box
    soil = ("#5C4433", matte),
    trunk = ("#6E5340", satin),      // the bonsai's trunk, plain (its leaves take the textured draw)
    leaf = ("#4F7F3E", foliage),
    sack = ("#C9B48C", matte),
    porcelain = ("#F2F2F0", gloss),
    lens = ("#DCE6EA", gloss),       // the magnifier's lens, plain (no glass draw)
    stones = ("#8C8A85", matte),
    glass = ("#DCE6E8", glass),      // the sauna's door
    lamp = ("#FFE2B0", gloss),
    kraft = ("#C9A27A", matte),      // the delivery bag
    screen = ("#1F3A40", gloss),
    screen_a = ("#7FD1C4", gloss),
    mat = ("#8E7F6A", matte),
    // the puzzle picture's bands: sky, sea, hills, fields
    sky_a = ("#9CC8E6", satin),
    sky_b = ("#7FB6DE", satin),
    sea_a = ("#3E7FAE", satin),
    sea_b = ("#4C8FB8", satin),
    hill_a = ("#6E9E58", satin),
    hill_b = ("#7FAF63", satin),
    field_a = ("#D9B45A", satin),
    field_b = ("#E3C46E", satin),
}

/// The puzzle picture's bands.
pub const BANDS: [(Mat, Mat); 4] = [
    (M.sky_a, M.sky_b),
    (M.sea_a, M.sea_b),
    (M.hill_a, M.hill_b),
    (M.field_a, M.field_b),
];
/// The wall's face in a 1-deep footprint, game depth.
pub use artkit::kit::WALL;
/// In a 2-deep one.
pub const WALL2: f64 = -0.93;

/// A spindle chair facing the front, its seat (top 0.45) centred on (0, z): the resident's seat. A shaped seat with
/// its pad, back posts swept up from the floor and raked back, tapered front legs, a bowed crest rail over three
/// spindles, side stretchers.
pub fn _chair(f: &GameFrame, z: f64, wood: Mat, pad: Mat, verts: usize) {
    fextrude!(
        f,
        numbered("chair_seat"),
        &smooth_outline(
            &[
                (-0.21, -0.2),
                (0.21, -0.2),
                (0.21, 0.17),
                (0.0, 0.2),
                (-0.21, 0.17)
            ],
            1,
            true
        ),
        0.04,
        (0, 0.41, z),
        wood,
        plane = Facing::Top,
        bevel = 0.008,
        bevel_segments = 1
    );
    soft!(
        f,
        numbered("chair_pad"),
        0.36,
        0.024,
        0.34,
        (0, 0.442, z + 0.01),
        pad,
        round = 0.012,
        puff = 0.006,
        round_segments = 1,
        flat = 2
    );
    for x in [-0.18, 0.18] {
        fsweep!(
            f,
            numbered("chair_leg"),
            &[(x, 0.39, z + 0.15), (x * 1.04, 0.0, z + 0.17)],
            0.018,
            wood,
            verts = verts,
            scale = [1.0, 0.75]
        );
        fsweep!(
            f,
            numbered("chair_post"),
            &[
                (x, 0.0, z - 0.165),
                (x, 0.42, z - 0.172),
                (x, 0.92, z - 0.193)
            ],
            0.018,
            wood,
            verts = verts,
            resolution = 2,
            scale = [0.8, 1.0, 0.85]
        );
        kit::tube!(
            f,
            numbered("stretcher"),
            &[(x, 0.14, z + 0.15), (x, 0.14, z - 0.17)],
            0.009,
            wood,
            verts = verts,
            round = 1
        );
    }
    let crest: Vec<(f64, f64, f64)> = [-0.2, -0.1, 0.0, 0.1, 0.2]
        .iter()
        .map(|&x| {
            (
                x,
                0.87 + 0.012 * (1.0 - pow(x / 0.2, 2.0)),
                z - 0.186 - 0.006 * (1.0 - pow(x / 0.2, 2.0)),
            )
        })
        .collect();
    let crest = fsweep!(
        f,
        numbered("chair_crest"),
        &crest,
        0.0,
        wood,
        profile = rounded_rect(0.09, 0.03, 0.012, 1),
        up = (0, 0, 1),
        resolution = 2
    );
    ease(crest, 60.0, 1);
    for x in [-0.09, 0.0, 0.09] {
        kit::tube!(
            f,
            numbered("spindle"),
            &[
                (x, 0.43, z - 0.175),
                (x, 0.83, z - 0.186 - 0.006 * (1.0 - pow(x / 0.2, 2.0)))
            ],
            0.009,
            wood,
            verts = verts,
            round = 1
        );
    }
}

/// A plain desk from depth z0 to z1, `h` high: its top, an apron at the back and the ends, tapered splayed legs.
#[allow(clippy::too_many_arguments)]
pub fn _desk(f: &GameFrame, z0: f64, z1: f64, w: f64, h: f64, top: Mat, legs: Mat, verts: usize) {
    let (zc, d) = ((z0 + z1) / 2.0, z1 - z0);
    fbox!(
        f,
        numbered("desk_top"),
        w,
        0.03,
        d,
        (0, h - 0.015, zc),
        top,
        bevel = 0.008,
        bevel_segments = 2
    );
    fbox!(
        f,
        numbered("desk_apron"),
        w - 0.08,
        0.07,
        0.02,
        (0, h - 0.065, z1 - 0.03),
        legs,
        bevel = 0.004,
        bevel_segments = 1
    );
    for x in [-w / 2.0 + 0.035, w / 2.0 - 0.035] {
        fbox!(
            f,
            numbered("desk_apron_side"),
            0.02,
            0.07,
            d - 0.08,
            (x, h - 0.065, zc),
            legs,
            bevel = 0.004,
            bevel_segments = 1
        );
        for z in [z0 + 0.035, z1 - 0.035] {
            fsweep!(
                f,
                numbered("desk_leg"),
                &[
                    (x, h - 0.03, z),
                    (
                        x + 0.012_f64.copysign(x),
                        0.0,
                        z + 0.01_f64.copysign(z - zc)
                    )
                ],
                0.02,
                legs,
                verts = verts,
                scale = [1.0, 0.65]
            );
        }
    }
}

/// A mug standing on `y`: hollow, its handle out at `turn` degrees (0: +x).
pub fn _mug(f: &GameFrame, x: f64, y: f64, z: f64, material: Mat, turn: f64) {
    let (r, h) = (0.038, 0.09);
    kit::turned!(
        f,
        numbered("mug"),
        x,
        y,
        z,
        &[
            (0.0, 0.0),
            (0.034, 0.0),
            (r, 0.006),
            (r, h),
            (0.032, h),
            (0.03, 0.012),
            (0.0, 0.012),
        ],
        material,
        segments = 14
    );
    let (c, s) = (cosd(turn), sind(turn));
    let pts: Vec<(f64, f64, f64)> = [
        (r - 0.003, 0.8),
        (r + 0.016, 0.82),
        (r + 0.024, 0.55),
        (r + 0.015, 0.27),
        (r - 0.003, 0.25),
    ]
    .iter()
    .map(|&(d, t)| (x + c * d, y + t * h, z + s * d))
    .collect();
    fsweep!(
        f,
        numbered("mug_handle"),
        &pts,
        0.0045,
        material,
        verts = 5,
        resolution = 2
    );
}

/// A closed book lying flat on `y`: its cover and the block of pages, a hair inside it but for the spine (-x).
#[allow(clippy::too_many_arguments)]
pub fn _book(
    f: &GameFrame,
    name: &str,
    x: f64,
    y: f64,
    z: f64,
    w: f64,
    h: f64,
    d: f64,
    cover: Mat,
    turn: f64,
) {
    fbox!(
        f,
        format!("{name}_cover"),
        w,
        h,
        d,
        (x, y + h / 2.0, z),
        cover,
        rot = (0.0, 0.0, turn),
        bevel = 0.002,
        bevel_segments = 1
    );
    let (c, s) = (cosd(turn), sind(turn));
    fbox!(
        f,
        format!("{name}_pages"),
        w - 0.008,
        h - 0.006,
        d + 0.002,
        (x + 0.004 * c, y + h / 2.0, z + 0.004 * s),
        M.cream,
        rot = (0.0, 0.0, turn)
    );
}
