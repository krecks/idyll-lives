//! What the couch potato pack is built from: its palette, pleated curtains, speaker drivers, screens and snacks.

use artkit::materials::Mat;
use artkit::math::{PI, atan2, cosd, sin, sind};
use artkit::parts::numbered;
use artkit::place::{Facing, GameFrame};
use artkit::py::fmod;
use artkit::scene::Obj;
use artkit::{fbox, fextrude, fsphere, kit, object_of, palette, revolve};

use artkit::forms::rounded_rect;

palette! { M:
    // the cinema wall
    leather = ("#7A2E2E", satin),        // the twin recliners; plain, so the curtains take the textured draws
    cinema_frame = ("#5C6168", satin),   // the console, posts and speakers; its main part
    curtain = ("#8E2A3A", fabric),
    pelmet = ("#3A3D41", satin),
    // screens
    screen_back = ("#CDB08A", satin),    // the oak acoustic backs, plain
    slat = ("#B08E66", satin),
    bezel = ("#1B2A3A", gloss),
    picture = ("#3BA3E0", gloss, glow("#3BA3E0"), screen()),
    picture_b = ("#D6A64A", gloss, glow("#D6A64A"), screen()),
    graphite = ("#5C6168", metal),
    speaker_grey = ("#6E7277", satin),
    speaker_cone = ("#3A3D41", satin),
    soda = ("#C9483B", gloss),
    straw = ("#F2F2F0", satin),
    popcorn = ("#F6E7B8", matte),
    dark = ("#2B2C2E", satin),
    // the game console
    gamer = ("#3F6E9E", fabric),          // the gaming loveseat; its main part
    mustard = ("#D6A64A", satin),
    console = ("#E8E8E6", gloss),
    disc = ("#C9CCD0", chrome),
    led = ("#3BA3E0", gloss),
    box_red = ("#B33A32", satin),
    box_blue = ("#3E8EDE", satin),
    // the recliner
    plush = ("#C08A52", satin),           // the recliner's microsuede; its main part
    plinth = ("#6E5038", satin),
    teal = ("#4E7A7F", gloss),
    throw = ("#D9C7A6", satin),
    // the snack fridge
    mint = ("#9FD3C7", gloss),            // the fridge; its main part
    mint_trim = ("#7FB5A9", satin),
    lid = ("#E9E4DA", fabric),            // the padded lid
    inside = ("#F2F4F4", satin),
    chrome = ("#D0D3D6", chrome),
    menu = ("#F2A541", gloss),
    can_a = ("#C9483B", gloss),
    can_b = ("#3E8EDE", gloss),
    can_c = ("#9DAF8C", gloss),
    takeout = ("#F2EDE2", satin),
    bowl = ("#F2EDE2", gloss),
    crisps = ("#E9B54A", matte),
    // the workout game
    fit_stand = ("#E8E8E6", satin),       // the TV stand; its main part
    oak = ("#CDB08A", satin),
    fit_mat = ("#7FC4A4", fabric),
    mat_inner = ("#9FD8BB", fabric),
    towel = ("#E9B54A", fabric),
    tv_frame = ("#D5D6D4", satin),
    tv_screen = ("#4FC3A1", gloss, glow("#4FC3A1"), screen()),
    tv_floor = ("#3FA88A", gloss),
    tv_white = ("#F4F1EA", gloss),
    tv_orange = ("#F2A541", gloss),
    board = ("#3E8EDE", satin),
    grip = ("#2B2C2E", matte),
    bottle = ("#3E8EDE", gloss),
    weight = ("#C9483B", satin),
    // the lava lamp
    pouf = ("#5A4A7A", satin),            // the pouf; its main part (plain: the lamp's glass takes the extra draw)
    lamp_metal = ("#4E7A7F", metal),
    vessel = ("#E96A9A", glass, glow("#FF6FA8")),   // the lit glass: it and the wax glow at night (the lamp's `light`)
    wax = ("#F2914A", gloss, glow("#FFA060")),
    table_white = ("#E8E8E6", satin),
    rug = ("#E9B54A", satin),
}

// --- meshes from points (as pack_cheerful) --------------------------------------------------------------------

/// A pleated curtain across `x0`..`x1` hanging in the plane at depth `z` from its heading at `y1` to its hem at
/// `y0`: its folds shallow under the heading tape, deepening as it falls; a turned-up hem thickens its foot.
#[allow(clippy::too_many_arguments)]
pub fn pleats(
    f: &GameFrame,
    name: &str,
    x0: f64,
    x1: f64,
    y0: f64,
    y1: f64,
    z: f64,
    folds: i32,
    amp: f64,
    mat: Mat,
    t: f64,
) -> Obj {
    let h = y1 - y0;
    // (the hem and the heading 11.5 mm thick at most: thinner than a hard edge's 12 mm faces)
    let rows = [
        (y0, 1.0, 1.15, 0.0),
        (y0 + 0.012, 1.0, 1.15, 0.02),
        (y0 + 0.04, 1.0, 1.0, 0.05),
        (y0 + h * 0.45, 0.9, 1.0, 0.25),
        (y1 - 0.06, 0.7, 1.0, 0.4),
        (y1 - 0.05, 0.4, 1.15, 0.42),
        (y1, 0.36, 1.15, 0.42),
    ];
    let (verts, faces) = kit::pleats!(f, &rows, x0, x1, z, folds, amp, t);
    // its side hems: a cord down each end, where the cloth's end is (the folds end where they start)
    for x in [x0, x1] {
        let down: Vec<(f64, f64, f64)> = rows
            .iter()
            .map(|&(y, af, _, phase)| (x, y, z + amp * af * sin(phase)))
            .collect();
        kit::tube!(
            f,
            numbered("side_hem"),
            &down,
            0.0065,
            mat,
            verts = 5,
            round = 1
        );
    }
    object_of!(name, &verts, &faces, &[mat], sharp = 70)
}

/// A speaker driver at (x, y) on a face at depth `z`, facing +z (`facing` 1) or -z: its flange, a rolled
/// surround and the cone dished in to a domed dust cap, one turned piece; the flange and surround in the dark.
#[allow(clippy::too_many_arguments)]
pub fn driver(
    f: &GameFrame,
    name: &str,
    x: f64,
    y: f64,
    z: f64,
    r: f64,
    facing: i32,
    segments: usize,
) -> Obj {
    let prof = [
        (r + 0.012, -0.008),
        (r + 0.012, 0.002),
        (r + 0.004, 0.004),
        // (the surround's crest rolled over in two turns, not one)
        (r - 0.0005, 0.0085),
        (r - 0.0035, 0.0085),
        (r - 0.008, 0.004),
        (r * 0.36, -0.02),
        (r * 0.3, -0.016),
        (r * 0.15, -0.012),
        (0.0, -0.011),
    ];
    revolve!(
        name,
        &prof,
        at = f.at(x, y, z),
        material = M.speaker_cone,
        segments = segments,
        rot = (90 * facing, 0, 0)
    )
}

/// A screen held between two slim posts at depth `z`, its picture facing the seats behind it (-Z), the oak back
/// with slats on its other side (to the wall, once the model is turned round).
pub fn screen(f: &GameFrame, x0: f64, x1: f64, y0: f64, y1: f64, z: f64, stripe: Mat) {
    let (w, h) = (x1 - x0, y1 - y0);
    let (cx, cy) = ((x0 + x1) / 2.0, (y0 + y1) / 2.0);
    fextrude!(
        f,
        numbered("screen_back"),
        &rounded_rect(w, h, 0.012, 1),
        0.05,
        (cx, cy, z),
        M.screen_back,
        bevel = 0.008,
        bevel_segments = 1
    );
    fextrude!(
        f,
        numbered("bezel"),
        &rounded_rect(w - 0.03, h - 0.03, 0.01, 1),
        0.008,
        (cx, cy, z - 0.028),
        M.bezel,
        bevel = 0.002,
        bevel_segments = 1
    );
    fbox!(
        f,
        numbered("picture"),
        w * 0.82,
        h * 0.7,
        0.002,
        (cx, cy + h * 0.04, z - 0.0325),
        M.picture
    );
    fbox!(
        f,
        numbered("picture_b"),
        w * 0.3,
        h * 0.12,
        0.002,
        (cx - w * 0.2, cy - h * 0.28, z - 0.034),
        M.picture_b
    );
    fbox!(
        f,
        numbered("stripe"),
        w * 0.5,
        0.02,
        0.006,
        (cx, y0 + 0.06, z + 0.027),
        stripe
    );
    let n = 3.max(artkit::py::int(w / 0.06));
    for i in 1..n {
        fbox!(
            f,
            numbered("slat"),
            0.016,
            h - 0.05,
            0.006,
            (x0 + w * i as f64 / n as f64, cy, z + 0.026),
            M.slat
        );
    }
    for x in [x0 + 0.02, x1 - 0.02] {
        fbox!(
            f,
            numbered("post"),
            0.04,
            y1 + 0.02,
            0.04,
            (x, (y1 + 0.02) / 2.0, z + 0.01),
            M.graphite,
            bevel = 0.008,
            bevel_segments = 1
        );
        fextrude!(
            f,
            numbered("post_foot"),
            &rounded_rect(0.08, 0.12, 0.02, 2),
            0.02,
            (x, 0.01, z - 0.04),
            M.graphite,
            plane = Facing::Top,
            bevel = 0.005,
            bevel_segments = 1
        );
    }
}

/// A striped popcorn bucket standing on `y`, popped corn heaped over its rim.
#[allow(clippy::too_many_arguments)]
pub fn popcorn(f: &GameFrame, x: f64, y: f64, z: f64, r: f64, h: f64, stripe: Mat, n: i32) {
    let b = revolve!(
        numbered("bucket"),
        &[
            (0.0, 0.0),
            (r * 0.75, 0.0),
            (r, h),
            (r * 0.94, h),
            (r * 0.72, 0.006),
            (0.0, 0.006)
        ],
        at = f.at(x, y, z),
        material = M.straw,
        segments = 16
    );
    kit::paint(b, stripe, |c| {
        fmod(atan2(c.y as f64, c.x as f64), PI / 4.0) < PI / 8.0
    });
    for k in 0..n {
        let a = (360 * k) as f64 / n as f64;
        let rr = if k % 2 != 0 { r * 0.55 } else { r * 0.3 };
        fsphere!(
            f,
            numbered("popcorn"),
            r * 0.32,
            (
                x + rr * cosd(a),
                y + h + 0.006 * (k % 3) as f64,
                z + rr * sind(a)
            ),
            M.popcorn,
            segments = 6,
            rings = 4
        );
    }
    fsphere!(
        f,
        numbered("popcorn"),
        r * 0.36,
        (x, y + h + 0.02, z),
        M.popcorn,
        segments = 6,
        rings = 4
    );
}

/// A soda cup standing on `y`: a tapered body, its domed lid and a straw through it.
pub fn cup(f: &GameFrame, x: f64, y: f64, z: f64, r: f64, h: f64, material: Mat) {
    revolve!(
        numbered("soda"),
        &[(0.0, 0.0), (r * 0.78, 0.0), (r, h), (0.0, h)],
        at = f.at(x, y, z),
        material = material,
        segments = 12
    );
    revolve!(
        numbered("soda_lid"),
        &[
            (0.0, 0.0),
            (r * 1.06, 0.0),
            (r * 1.06, 0.006),
            (r * 0.8, 0.014),
            (0.0, 0.016)
        ],
        at = f.at(x, y + h, z),
        material = M.straw,
        segments = 12
    );
    kit::tube!(
        f,
        numbered("straw"),
        &[(x + 0.006, y + h, z), (x + 0.012, y + h + 0.1, z - 0.006)],
        0.0045,
        M.straw,
        verts = 6,
        round = 1
    );
}
