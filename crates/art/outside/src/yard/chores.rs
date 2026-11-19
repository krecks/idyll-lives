//! The letterbox on its post, the hose reel cart and the push lawnmower: the yard's chores.

use super::common::{F, M, bent, board, rod};
use artkit::far::{is_low, low};
use artkit::forms::{Plane, smooth_outline};
use artkit::materials::Mat;
use artkit::math::{Axis, P3, V3, cosd, sind};
use artkit::motion::{Swing, swing};
use artkit::parts::{hinged, join, join_all, new_parts, numbered};
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fcyl, fextrude, fsweep, loft, revolve, ring, settle};

/// An end of the letterbox: a flat bottom `2 * half` wide at `bottom`, straight sides up to `side`, a round top;
/// (game x, height), the round in `n` steps.
fn arch(half: f64, side: f64, bottom: f64, n: usize) -> Vec<(f64, f64)> {
    let mut pts = vec![(-half, bottom), (half, bottom)];
    pts.extend((0..=n).map(|k| {
        let a = 180.0 * k as f64 / n as f64;
        (half * cosd(a), side + half * sind(a))
    }));
    pts
}

/// A polygon's corners cut `r` back along each side, to be rounded by `smooth_outline`.
fn cut_corners(pts: &[(f64, f64)], r: f64) -> Vec<(f64, f64)> {
    let n = pts.len();
    let mut out = Vec::with_capacity(2 * n);
    for i in 0..n {
        let (p, a, b) = (pts[i], pts[(i + n - 1) % n], pts[(i + 1) % n]);
        for q in [a, b] {
            let (dx, dy) = (q.0 - p.0, q.1 - p.1);
            let k = r / (dx * dx + dy * dy).sqrt();
            out.push((p.0 + dx * k, p.1 + dy * k));
        }
    }
    out
}

/// A wheel on an axle across game x at (x, y, z), `r` in radius and `w` wide: a rounded tyre round a hub dished on
/// its outer side (`out`: -1 the left, game -x; 1 the right) with a boss on the axle.
fn wheel(x: f64, y: f64, z: f64, r: f64, w: f64, out: f64, hub: Mat) {
    let h = w / 2.0;
    let rim = r * 0.68;
    // (a profile's +z turned by (0, 90, 0) faces game -x)
    let rot = (0, if out < 0.0 { 90 } else { -90 }, 0);
    revolve!(
        numbered("tyre"),
        &[
            (rim, h),
            (r - 0.012, h),
            (r - 0.004, h * 0.7),
            (r, h * 0.3),
            (r, -h * 0.3),
            (r - 0.004, -h * 0.7),
            (r - 0.012, -h),
            (rim, -h)
        ],
        at = F.at(x, y, z),
        material = M.tyre,
        segments = low(20, 10),
        rot = rot
    );
    revolve!(
        numbered("hub"),
        &[
            (0.0, -h + 0.003),
            (rim, -h),
            (rim, h),
            (rim - 0.008, h - 0.005),
            (0.022, h - 0.005),
            (0.018, h + 0.004),
            (0.0, h + 0.004)
        ],
        at = F.at(x, y, z),
        material = hub,
        segments = low(20, 10),
        rot = rot
    );
}

// --- the letterbox (1x1): a rounded steel box on a post, its door drops open, its flag goes up -----------------
const LB_Y: f64 = 1.07; // the box's floor
const LB_HALF: f64 = 0.1; // half its width, the round top's radius
const LB_SIDE: f64 = 0.13; // its straight sides; the top 0.23 up
const LB_BACK: f64 = -0.24;
const LB_FRONT: f64 = 0.225; // the box's front edge, where the door shuts
const LB_DOOR: f64 = 0.012;
const LB_FLAG: (f64, f64) = (LB_Y + 0.11, -0.16); // the flag's pivot on the right side (height, depth)

pub fn letterbox() -> Vec<Obj> {
    let n = low(16, 6);
    let bevel = low(0.004, 0.0);
    // the post set in the ground, the board the box sits on, a brace under its front
    fbox!(
        F,
        "post",
        0.09,
        LB_Y - 0.025,
        0.09,
        (0, (LB_Y - 0.025) / 2.0, -0.07),
        M.mail_post,
        bevel = low(0.006, 0.0),
        bevel_segments = 1
    );
    fbox!(
        F,
        "board",
        0.15,
        0.025,
        0.42,
        (0, LB_Y - 0.0125, -0.02),
        M.mail_post,
        bevel = bevel,
        bevel_segments = 1
    );
    board(
        "brace",
        (0, 0.84, -0.04),
        (0, LB_Y - 0.03, 0.13),
        0.04,
        0.04,
        M.mail_post,
        (1, 0, 0),
        bevel,
    );
    // the box: a tunnel of sheet steel, its back set in
    let inner = arch(LB_HALF - 0.006, LB_SIDE, 0.006, n);
    fextrude!(
        F,
        "box",
        &arch(LB_HALF, LB_SIDE, 0.0, n),
        LB_FRONT - LB_BACK,
        (0, LB_Y, (LB_BACK + LB_FRONT) / 2.0),
        M.mailbox,
        holes = [inner.clone()],
        bevel = low(0.003, 0.0),
        bevel_segments = 1
    );
    fextrude!(
        F,
        "box_back",
        &inner,
        0.008,
        (0, LB_Y, LB_BACK + 0.004),
        M.mailbox
    );
    if !is_low() {
        // the post inside, seen when the door drops
        fbox!(
            F,
            numbered("letter"),
            0.15,
            0.004,
            0.22,
            (0.005, LB_Y + 0.008, -0.03),
            M.letter_kraft,
            rot = (0, 0, 4)
        );
        fbox!(
            F,
            numbered("letter"),
            0.16,
            0.004,
            0.11,
            (-0.006, LB_Y + 0.012, 0.13),
            M.letter,
            rot = (0, 0, -8)
        );
    }
    // the flag's pivot
    fcyl!(
        F,
        "flag_pivot",
        0.012,
        0.008,
        (LB_HALF + 0.004, LB_FLAG.0, LB_FLAG.1),
        M.steel,
        rot = (0, 90, 0),
        verts = low(12, 6)
    );
    if !is_low() {
        join(&meshes(), "letterbox", Some(P3::from((0, 0, 0))));
    }
    // The door: the box's end a little larger, a latch at its top to pull it down by, hinged along its foot.
    let door = new_parts(|| {
        fextrude!(
            F,
            "door",
            &arch(LB_HALF + 0.004, LB_SIDE, -0.004, n),
            LB_DOOR,
            (0, LB_Y, LB_FRONT + LB_DOOR / 2.0),
            M.mailbox,
            bevel = bevel,
            bevel_segments = 1
        );
        fbox!(
            F,
            "latch",
            0.04,
            0.026,
            0.012,
            (0, LB_Y + LB_SIDE + 0.08, LB_FRONT + LB_DOOR + 0.005),
            M.steel,
            bevel = low(0.003, 0.0),
            bevel_segments = 1
        );
    });
    // The flag: an arm lying along the right side, pointing to the front, the flag at its end.
    let flag = new_parts(|| {
        let x = LB_HALF + 0.0105;
        let (y, z) = LB_FLAG;
        fbox!(
            F,
            "flag_arm",
            0.005,
            0.022,
            0.2,
            (x, y, z + 0.09),
            M.mail_flag
        );
        fbox!(
            F,
            "flag",
            0.005,
            0.065,
            0.075,
            (x, y + 0.0215, z + 0.1525),
            M.mail_flag
        );
        fcyl!(
            F,
            "flag_rivet",
            0.006,
            0.004,
            (x + 0.004, y, z),
            M.steel,
            rot = (0, 90, 0),
            verts = low(8, 5)
        );
    });
    if is_low() {
        return settle!(&[join_all("letterbox")]);
    }
    let door = hinged(
        join(&door, "door", None),
        F.at(0, LB_Y - 0.004, LB_FRONT + LB_DOOR),
    );
    let flag = hinged(
        join(&flag, "flag", None),
        F.at(LB_HALF + 0.0105, LB_FLAG.0, LB_FLAG.1),
    );
    let objs = settle!();
    // the door's top drops to the front; the flag's arm turns up
    swing(
        door,
        "door_open",
        95.0,
        Swing {
            seconds: 1.0,
            axis: Axis::X,
            ..Swing::default()
        },
    );
    swing(
        flag,
        "flag_up",
        -90.0,
        Swing {
            seconds: 0.6,
            axis: Axis::X,
            ..Swing::default()
        },
    );
    objs
}

// --- the hose reel (1x1): a cart of tube on two wheels, a drum of green hose, the spray gun hooked on --------------
const HR_X: f64 = 0.215; // the frame's sides, game x
const HR_HUB: (f64, f64) = (0.44, 0.0); // the drum's axle (height, depth)
const HR_AXLE: (f64, f64) = (0.11, -0.13); // the wheels' axle
const HR_FOOT: (f64, f64) = (0.025, 0.2); // the foot bar
const HR_TOP: (f64, f64) = (0.83, -0.19); // the handle

pub fn hose_reel() -> Vec<Obj> {
    let tv = low(10, 6);
    let side = |sx: f64, (y, z): (f64, f64)| (sx * HR_X, y, z);
    // the frame: one tube up a front leg, over the top as the handle and down the other (the drum's axle on it
    // half way); a strut from each wheel up to the drum, a bar along the bottom, the foot bar across the front
    bent(
        "frame",
        &[
            side(-1.0, HR_FOOT),
            side(-1.0, HR_TOP),
            side(1.0, HR_TOP),
            side(1.0, HR_FOOT),
        ],
        0.0125,
        M.reel,
        0.06,
        tv,
        low(4, 2),
    );
    for sx in [-1.0, 1.0] {
        rod(
            &numbered("strut"),
            side(sx, HR_AXLE),
            side(sx, HR_HUB),
            0.011,
            M.reel,
            tv,
        );
        rod(
            &numbered("bottom"),
            side(sx, HR_AXLE),
            side(sx, HR_FOOT),
            0.011,
            M.reel,
            tv,
        );
        // a rubber foot under the foot bar
        fcyl!(
            F,
            numbered("foot"),
            0.02,
            0.018,
            (sx * 0.16, 0.009, HR_FOOT.1),
            M.tyre,
            verts = low(10, 6)
        );
        wheel(sx * 0.255, HR_AXLE.0, HR_AXLE.1, 0.11, 0.045, sx, M.drum);
    }
    rod(
        "foot_bar",
        side(-1.0, HR_FOOT),
        side(1.0, HR_FOOT),
        0.011,
        M.reel,
        tv,
    );
    rod(
        "axle",
        (-0.27, HR_AXLE.0, HR_AXLE.1),
        (0.27, HR_AXLE.0, HR_AXLE.1),
        0.008,
        M.steel,
        low(8, 5),
    );
    // the handle's grip
    fcyl!(
        F,
        "grip",
        0.018,
        0.3,
        (0, HR_TOP.0, HR_TOP.1),
        M.tyre,
        rot = (0, 90, 0),
        verts = low(12, 6)
    );
    // the drum: two flanges on a hub through the frame, the hose wound between them coil by coil
    let (hy, hz) = HR_HUB;
    for sx in [-1.0, 1.0] {
        revolve!(
            numbered("flange"),
            &[
                (0.0, -0.005),
                (0.161, -0.005),
                (0.1645, -0.0035),
                (0.166, 0.0),
                (0.1645, 0.0035),
                (0.161, 0.005),
                (0.0, 0.005)
            ],
            at = F.at(sx * 0.188, hy, hz),
            material = M.drum,
            segments = low(24, 12),
            rot = (0, 90, 0)
        );
    }
    fcyl!(
        F,
        "hub",
        0.024,
        0.47,
        (0.005, hy, hz),
        M.drum,
        rot = (0, 90, 0),
        verts = low(12, 6),
        bevel = low(0.004, 0.0),
        bevel_segments = 1
    );
    let coils = low(12, 6);
    let half = 0.183;
    let pitch = 2.0 * half / coils as f64;
    let mut wound = vec![(0.06, -half), (0.127, -half)];
    for k in 0..coils {
        wound.push((0.136, -half + pitch * (k as f64 + 0.5)));
        wound.push((0.127, -half + pitch * (k + 1) as f64));
    }
    wound.push((0.06, half));
    revolve!(
        "hose_wound",
        &wound,
        at = F.at(0, hy, hz),
        material = M.hose,
        segments = low(20, 10),
        rot = (0, 90, 0),
        angle = 80.0
    );
    // the crank on the right: an arm off the hub, a knob to turn it by
    board(
        "crank",
        (0.244, hy, hz),
        (0.244, hy - 0.095, hz + 0.045),
        0.026,
        0.008,
        M.reel,
        (1, 0, 0),
        0.0,
    );
    fcyl!(
        F,
        "crank_knob",
        0.013,
        0.042,
        (0.269, hy - 0.09, hz + 0.043),
        M.nozzle,
        rot = (0, 90, 0),
        verts = low(10, 6)
    );
    // the hose off the drum's front, looping down and up into the spray gun on the left front leg
    fsweep!(
        F,
        "hose",
        &[
            (-0.09, 0.4, 0.13),
            (-0.13, 0.27, 0.2),
            (-0.18, 0.13, 0.23),
            (-0.25, 0.11, 0.19),
            (-0.268, 0.16, 0.1),
            (-0.255, 0.215, 0.068),
            (-0.25, 0.235, 0.064)
        ],
        0.009,
        M.hose,
        verts = low(8, 5),
        resolution = low(4, 2)
    );
    // the spray gun, its barrel up: a yellow body, a dark head, its grip out to the front, a clip to the leg
    fcyl!(
        F,
        "gun",
        0.014,
        0.11,
        (-0.25, 0.29, 0.064),
        M.nozzle,
        verts = low(12, 6),
        bevel = low(0.003, 0.0),
        bevel_segments = 1
    );
    fcyl!(
        F,
        "gun_head",
        0.017,
        0.03,
        (-0.25, 0.358, 0.064),
        M.engine,
        verts = low(12, 6)
    );
    fbox!(
        F,
        "gun_grip",
        0.022,
        0.07,
        0.026,
        (-0.25, 0.27, 0.092),
        M.nozzle,
        rot = (12, 0, 0),
        bevel = low(0.004, 0.0),
        bevel_segments = 1
    );
    fbox!(
        F,
        "gun_clip",
        0.034,
        0.02,
        0.02,
        (-0.232, 0.32, 0.06),
        M.engine
    );
    settle!(&[join_all("hoseReel")])
}

// --- the lawnmower (1x2): a petrol rotary mower, its deck and engine at the back of the footprint, its handle ---
// rising to the front, where the resident stands to push it; a grass box between them
const MW_DECK: f64 = -0.45; // the deck's middle, depth
/// How far forward (+z) the built mower is moved: the resident pushing it stands on the tile in front of the
/// footprint, so the grip ends at about +0.97.
const MW_SHIFT: f64 = 0.263;

pub fn lawn_mower() -> Vec<Obj> {
    let d = F.at(0, 0, MW_DECK);
    // the deck: a pressed steel shell, its skirt straight, its top domed
    let n = low(40, 16);
    let deck: Vec<Vec<P3>> = [
        (0.03, 0.23, 0.25, 6.0),
        (0.115, 0.23, 0.25, 6.0),
        (0.14, 0.218, 0.238, 5.5),
        (0.172, 0.18, 0.2, 4.5),
        (0.19, 0.14, 0.155, 3.5),
    ]
    .iter()
    .map(|&(y, hx, hz, sq)| ring!(hx, hz, z = y, verts = n, square = sq, at = (d.x, d.y)))
    .collect();
    loft!("deck", &deck, material = M.mower, angle = 40);
    // four wheels, the back pair bigger, on stubs from the deck
    for sx in [-1.0, 1.0] {
        for (z, r) in [(-0.62, 0.09), (-0.29, 0.1)] {
            wheel(sx * 0.255, r, z, r, 0.035, sx, M.mower_hub);
            if !is_low() {
                fcyl!(
                    F,
                    numbered("stub"),
                    0.009,
                    0.03,
                    (sx * 0.23, r, z),
                    M.steel,
                    rot = (0, 90, 0),
                    verts = 8
                );
            }
        }
    }
    // the engine: a cast base on the deck, the black shroud, the starter's cap, a yellow fuel cap, the air filter on
    // the right and the silencer on the left
    let ez = MW_DECK - 0.01;
    revolve!(
        "engine_base",
        &[
            (0.0, 0.0),
            (0.125, 0.0),
            (0.125, 0.022),
            (0.11, 0.032),
            (0.0, 0.032)
        ],
        at = F.at(0, 0.18, ez),
        material = M.motor,
        segments = low(24, 10)
    );
    let e = F.at(0, 0, ez);
    let shroud: Vec<Vec<P3>> = [
        (0.2, 0.13, 4.0),
        (0.37, 0.125, 4.0),
        (0.403, 0.108, 3.2),
        (0.42, 0.086, 2.6),
    ]
    .iter()
    .map(|&(y, r, sq)| {
        ring!(
            r,
            r,
            z = y,
            verts = low(32, 12),
            square = sq,
            at = (e.x, e.y)
        )
    })
    .collect();
    loft!("shroud", &shroud, material = M.engine, angle = 40);
    fcyl!(
        F,
        "starter",
        0.07,
        0.03,
        (0, 0.425, ez),
        M.motor,
        verts = low(24, 10),
        bevel = low(0.006, 0.0),
        bevel_segments = 1
    );
    fcyl!(
        F,
        "fuel_cap",
        0.022,
        0.03,
        (-0.085, 0.405, ez + 0.085),
        M.nozzle,
        verts = low(12, 6),
        bevel = low(0.004, 0.0),
        bevel_segments = 1
    );
    fbox!(
        F,
        "air_filter",
        0.05,
        0.12,
        0.14,
        (0.15, 0.31, ez),
        M.engine,
        bevel = low(0.012, 0.0),
        bevel_segments = 2
    );
    fbox!(
        F,
        "silencer",
        0.05,
        0.08,
        0.13,
        (-0.15, 0.255, ez - 0.02),
        M.motor,
        bevel = low(0.015, 0.0),
        bevel_segments = 2
    );
    // the chute at the deck's back, into the grass box
    fbox!(
        F,
        "chute",
        0.3,
        0.11,
        0.075,
        (0, 0.165, -0.205),
        M.engine,
        bevel = low(0.012, 0.0),
        bevel_segments = 1
    );
    // the handle: one tube from a bracket on each back corner of the deck, up past the folding knobs to the grip
    let lo = (0.205, 0.16, -0.215);
    let fold = (0.215, 0.56, 0.23);
    let top = (0.215, 0.975, 0.69);
    let at = |sx: f64, p: (f64, f64, f64)| (sx * p.0, p.1, p.2);
    bent(
        "handle",
        &[
            at(-1.0, lo),
            at(-1.0, fold),
            at(-1.0, top),
            at(1.0, top),
            at(1.0, fold),
            at(1.0, lo),
        ],
        0.0125,
        M.mower_handle,
        0.07,
        low(10, 6),
        low(4, 2),
    );
    for sx in [-1.0, 1.0] {
        fbox!(
            F,
            numbered("bracket"),
            0.012,
            0.1,
            0.06,
            (sx * 0.205, 0.13, -0.215),
            M.mower_handle,
            bevel = low(0.003, 0.0),
            bevel_segments = 1
        );
        fcyl!(
            F,
            numbered("fold_knob"),
            0.022,
            0.03,
            (sx * 0.24, fold.1, fold.2),
            M.engine,
            rot = (0, 90, 0),
            verts = low(12, 6)
        );
    }
    rod(
        "cross_bar",
        at(-1.0, fold),
        at(1.0, fold),
        0.009,
        M.mower_handle,
        low(8, 5),
    );
    fcyl!(
        F,
        "grip",
        0.017,
        0.28,
        (0, top.1, top.2),
        M.grip_tape,
        rot = (0, 90, 0),
        verts = low(12, 6)
    );
    if !is_low() {
        // the bail that stops the engine, let go: turned away from the grip
        bent(
            "bail",
            &[
                (-0.2, 0.908, 0.616),
                (-0.19, 0.985, 0.57),
                (0.19, 0.985, 0.57),
                (0.2, 0.908, 0.616),
            ],
            0.006,
            M.mower_handle,
            0.03,
            6,
            3,
        );
    }
    // the grass box between the handle's tubes: a bag deepest at the chute, tapering to its back, under a hard
    // top with a carrying handle
    let bag = [(-0.165, 0.085), (0.2, 0.15), (0.215, 0.37), (-0.17, 0.4)];
    fextrude!(
        F,
        "bag",
        &smooth_outline(&cut_corners(&bag, 0.04), low(2, 0), true),
        0.36,
        (0, 0, 0),
        M.grass_box,
        plane = Plane::YZ,
        bevel = low(0.02, 0.0),
        bevel_segments = 2
    );
    let rise = |z: f64| 0.4 + (z + 0.17) * (0.37 - 0.4) / (0.215 + 0.17);
    fextrude!(
        F,
        "bag_top",
        &[
            (-0.185, rise(-0.185) - 0.02),
            (0.228, rise(0.228) - 0.02),
            (0.228, rise(0.228) + 0.014),
            (-0.185, rise(-0.185) + 0.014)
        ],
        0.375,
        (0, 0, 0),
        M.engine,
        plane = Plane::YZ,
        bevel = low(0.008, 0.0),
        bevel_segments = 1
    );
    bent(
        "bag_handle",
        &[
            (0.0, rise(-0.08) + 0.01, -0.08),
            (0.0, rise(-0.05) + 0.055, -0.05),
            (0.0, rise(0.09) + 0.055, 0.09),
            (0.0, rise(0.12) + 0.01, 0.12),
        ],
        0.01,
        M.engine,
        0.025,
        low(8, 5),
        low(3, 1),
    );
    // forward in its footprint, so the grip ends a few cm in from the front edge, where the pusher's hands are
    let mower = join_all("lawnMower");
    mower.edit(|o| o.mesh.translate(V3::from(F.at(0, 0, MW_SHIFT))));
    settle!(&[mower], centre = false)
}
