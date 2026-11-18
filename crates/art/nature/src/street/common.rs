//! What the street's props are built from.

use super::M;
use crate::garden_decor::common::fillet;
use crate::plants::common::options;
use artkit::far::{is_low, low};
use artkit::forms::{by, rounded_rect};
use artkit::materials::Mat;
use artkit::math::{P3, PI, TAU, atan2, cos, degrees, pow, sin};
use artkit::place::{Coord, Facing, GameFrame};
use artkit::py;
use artkit::rand::Random;
use artkit::scene::Obj;
use artkit::{fbox, fcyl, fextrude, fsweep, jitter, loft, revolve, ring};

options! {
    /// `scroll`'s options.
    Scroll { turns: f64 = 1.1, end: f64 = 0.25, t: f64 = 0.02, w: f64 = 0.03, steps: usize = 10, profile: Option<Vec<(f64, f64)>> = None }
}

// KIT: kit.scroll
/// A flat iron scroll in the depth-height plane at the game x of `start`: from `start` (x, height, depth) it
/// curls round `centre` (height, depth) by `turns`, its radius closing to `end` of where it began, tapering as
/// it goes: a bench arm's or a bracket's end.
pub fn scroll(
    frame: &GameFrame,
    name: &str,
    start: (f64, f64, f64),
    centre: (f64, f64),
    material: Mat,
    o: Scroll,
) -> Obj {
    let (x, y0, z0) = start;
    let (cy, cz) = centre;
    let r0 = py::hypot(y0 - cy, z0 - cz);
    let a0 = atan2(y0 - cy, z0 - cz);
    let sign = if o.turns > 0.0 { 1.0 } else { -1.0 };
    let mut pts = Vec::new();
    for i in 0..=o.steps {
        let k = i as f64 / o.steps as f64;
        let r = r0 * (1.0 - (1.0 - o.end) * k);
        let a = a0 + sign * TAU * o.turns.abs() * k;
        pts.push((x, cy + r * sin(a), cz + r * cos(a)));
    }
    let prof = o
        .profile
        .unwrap_or_else(|| rounded_rect(o.t, o.w, o.t.min(o.w) * 0.3, 1));
    fsweep!(
        frame,
        name,
        &pts,
        0.0,
        material,
        profile = prof,
        up = (1, 0, 0),
        scale = by(|f| 1.0 - 0.45 * f)
    )
}

options! {
    /// `bench_end`'s options.
    BenchEnd { arm: Option<f64> = None, t: f64 = 0.026, w: f64 = 0.045, foot: f64 = 0.06, steps: usize = 3, corner: usize = 1, scrolls: bool = true }
}

// KIT: kit.bench_end
/// A cast-iron bench end in one plane at game x, each member a flat bar `t` thick and `w` deep: the back leg
/// rising in one curve into the upright, leaning back to `top` (height, depth); the front leg, straight up and,
/// with an `arm` height, bent back into the armrest that meets the upright, its front curling down in a scroll;
/// the seat rail between them at `seat` (its top, under the slats); pad feet `foot` long at `front` and `back`
/// (their depths). `steps` rounds the curves (1 for a far version), `corner` the bars' edges (0: square),
/// `scrolls` the curls.
#[allow(clippy::too_many_arguments)]
pub fn bench_end(
    frame: &GameFrame,
    name: &str,
    x: f64,
    seat: f64,
    front: f64,
    back: f64,
    top: (f64, f64),
    material: Mat,
    o: BenchEnd,
) -> Vec<Obj> {
    let BenchEnd {
        arm,
        t,
        w,
        foot,
        steps,
        corner,
        scrolls,
    } = o;
    let prof = rounded_rect(
        t,
        w,
        if corner != 0 { t.min(w) * 0.3 } else { 0.0 },
        corner.max(1),
    );
    let (ty, tz) = top;
    let rail = seat - w / 2.0;
    let bz = back + 0.045; // where the upright passes the seat rail
    let mut made = Vec::new();
    // the back leg into the upright: evenly spaced points, curved through (no corner to overshoot)
    let pts = [
        (x, 0.0, back),
        (x, seat * 0.5, back + 0.03),
        (x, rail, bz),
        (x, rail + (ty - rail) * 0.5, (bz + tz) / 2.0 + 0.012),
        (x, ty, tz),
    ];
    made.push(fsweep!(
        frame,
        format!("{name}_back"),
        &pts,
        0.0,
        material,
        profile = &prof,
        up = (1, 0, 0),
        resolution = steps
    ));
    // the upright's depth at height y (above the seat), for the arm to meet
    let upright_z = |y: f64| {
        let f = (y - rail) / (ty - rail);
        bz + (tz - bz) * f + 0.012 * sin(PI * f)
    };
    let fz = front - 0.015;
    match arm {
        None => {
            let pts = [(x, 0.0, front), (x, rail, fz)];
            made.push(fsweep!(
                frame,
                format!("{name}_front"),
                &pts,
                0.0,
                material,
                profile = &prof,
                up = (1, 0, 0)
            ));
        }
        Some(arm) => {
            let pts = fillet(
                &[
                    (x, 0.0, front),
                    (x, rail, fz),
                    (x, arm - w / 2.0, fz - 0.005),
                    (x, arm - w / 2.0, upright_z(arm) + w / 2.0),
                ],
                0.05,
                steps.max(1),
            );
            made.push(fsweep!(
                frame,
                format!("{name}_front"),
                &pts,
                0.0,
                material,
                profile = &prof,
                up = (1, 0, 0)
            ));
            if scrolls {
                let r = 0.038;
                made.push(scroll(
                    frame,
                    &format!("{name}_scroll"),
                    (x, arm - w / 2.0, fz - 0.012),
                    (arm - w / 2.0 - r, fz - 0.012),
                    material,
                    Scroll {
                        turns: -1.2,
                        t,
                        w: w * 0.75,
                        steps: 4 * steps.max(1),
                        ..Scroll::default()
                    },
                ));
            }
        }
    }
    if scrolls {
        // a stretcher sagging between the legs, and a curl in the angle of the front leg and the seat rail
        made.push(fsweep!(
            frame,
            format!("{name}_stretcher"),
            &[
                (x, 0.13, front - 0.01),
                (x, 0.1, (front + back) / 2.0),
                (x, 0.13, back + 0.02)
            ],
            0.0,
            material,
            profile = &prof,
            up = (1, 0, 0),
            resolution = steps,
            scale = 0.75
        ));
        made.push(scroll(
            frame,
            &format!("{name}_bracket"),
            (x, rail - w * 0.6, fz - w * 0.4),
            (rail - w * 0.6 - 0.035, fz - 0.06),
            material,
            Scroll {
                turns: 0.85,
                t,
                w: w * 0.6,
                steps: 4 * steps.max(1),
                ..Scroll::default()
            },
        ));
    }
    made.push(fsweep!(
        frame,
        format!("{name}_rail"),
        &[(x, rail, fz + 0.01), (x, rail, bz - 0.01)],
        0.0,
        material,
        profile = &prof,
        up = (1, 0, 0)
    ));
    for z in [front, back] {
        made.push(fbox!(
            frame,
            format!("{name}_foot{}", artkit::json::float_repr(z)),
            t + 0.02,
            0.02,
            foot,
            (x, 0.01, z),
            material,
            bevel = if corner != 0 { 0.006 } else { 0.0 },
            bevel_segments = 1
        ));
    }
    made
}

// KIT: kit.taxi_word
/// TAXI in block letters `h` high, its middle at game (x, height) `centre`, standing `depth` proud of a face at
/// depth `z` that looks `facing` (+1: to the front, +z; -1: to the back), so it reads from that side: seen from
/// the front, the game's +x is on the left, so the word runs from +x there, from -x behind. The taxi rank's signs
/// and fares board, as the taxi's own roof sign (render/shared/cars.ts `paintTaxi`).
#[allow(clippy::too_many_arguments)]
pub fn taxi_word(
    frame: &GameFrame,
    name: &str,
    centre: (f64, f64),
    h: f64,
    z: f64,
    facing: i32,
    material: Mat,
    depth: f64,
) -> Vec<Obj> {
    let (cx, cy) = centre;
    let facing = facing as f64;
    let (s, w, gap) = (0.2 * h, 0.66 * h, 0.22 * h);
    let total = 4.0 * w + 3.0 * gap;
    let zz = z + facing * depth / 2.0;
    let lean = degrees(atan2(w / 2.0 - s / 2.0, h)); // an A's legs from upright
    let cross = degrees(atan2(w - s, h)); // an X's strokes from upright
    let mut out = Vec::new();
    for (i, ch) in "TAXI".chars().enumerate() {
        // (Seen from the side it faces, letter by letter left to right.)
        let x = cx + facing * (total / 2.0 - i as f64 * (w + gap) - w / 2.0);
        let n = format!("{name}_{ch}{i}");
        match ch {
            'T' => {
                out.push(fbox!(
                    frame,
                    format!("{n}_bar"),
                    w,
                    s,
                    depth,
                    (x, cy + h / 2.0 - s / 2.0, zz),
                    material
                ));
                out.push(fbox!(
                    frame,
                    format!("{n}_stem"),
                    s,
                    h - s,
                    depth,
                    (x, cy - s / 2.0, zz),
                    material
                ));
            }
            'A' => {
                for k in [-1, 1] {
                    let leg = py::hypot(w / 2.0 - s / 2.0, h);
                    out.push(fbox!(
                        frame,
                        format!("{n}_leg{k}"),
                        s,
                        leg,
                        depth,
                        (x + k as f64 * (w / 4.0 - s / 4.0), cy, zz),
                        material,
                        rot = (0, k as f64 * lean, 0)
                    ));
                }
                out.push(fbox!(
                    frame,
                    format!("{n}_bar"),
                    w * 0.42,
                    s * 0.8,
                    depth,
                    (x, cy - h * 0.12, zz),
                    material
                ));
            }
            'X' => {
                for k in [-1, 1] {
                    out.push(fbox!(
                        frame,
                        format!("{n}_stroke{k}"),
                        s,
                        py::hypot(w - s, h) * 0.97,
                        depth,
                        (x, cy, zz),
                        material,
                        rot = (0, k as f64 * cross, 0)
                    ));
                }
            }
            _ => out.push(fbox!(
                frame,
                format!("{n}_stem"),
                s,
                h,
                depth,
                (x, cy, zz),
                material
            )),
        }
    }
    out
}

// KIT: kit.chequer
/// A chequered band's dark squares (`size` a side, `rows` of them, the band's top at height `y`) from x0 to x1
/// along a face at depth `z` looking `facing` (+1 front, -1 back), `depth` proud of it: the light squares are
/// the face behind. The taxi's livery, on the taxi rank's signs and shelter.
#[allow(clippy::too_many_arguments)]
pub fn chequer(
    frame: &GameFrame,
    name: &str,
    x0: f64,
    x1: f64,
    y: f64,
    z: f64,
    size: f64,
    facing: i32,
    material: Mat,
    rows: usize,
    depth: f64,
) -> Vec<Obj> {
    let zz = z + facing as f64 * depth / 2.0;
    let mut out = Vec::new();
    for r in 0..rows {
        let mut k = 0;
        let mut x = x0;
        while x + size <= x1 + 1e-6 {
            if (k + r) % 2 == 0 {
                out.push(fbox!(
                    frame,
                    format!("{name}{r}_{k}"),
                    size,
                    size,
                    depth,
                    (x + size / 2.0, y - size / 2.0 - r as f64 * size, zz),
                    material
                ));
            }
            x += size;
            k += 1;
        }
    }
    out
}

/// A box in game axes; bevelled with one segment (half the triangles of two), none in a far version.
#[allow(clippy::too_many_arguments)]
pub fn _box(
    f: &GameFrame,
    name: impl AsRef<str>,
    w: f64,
    h: f64,
    d: f64,
    at: (impl Coord, impl Coord, impl Coord),
    material: Mat,
    bevel: f64,
) -> Obj {
    fbox!(
        f,
        name,
        w,
        h,
        d,
        at,
        material,
        bevel = low(bevel, 0.0),
        bevel_segments = 1
    )
}

// --- the planter and the flower tub: a concrete tub, a little tree or flowers ---------------------------------
/// The tub's section (half-width, height): a chamfered foot, the body tapering out to a groove, a rim proud of it
/// with chamfered edges, and inside down to under the soil.
// (0.318 is a half-width of the tub, not 1/pi.)
#[allow(clippy::approx_constant)]
const TUB: [(f64, f64); 14] = [
    (0.300, 0.0),
    (0.311, 0.011),
    (0.311, 0.03),
    (0.304, 0.036),
    (0.322, 0.30),
    (0.318, 0.306),
    (0.324, 0.312),
    (0.33, 0.36),
    (0.347, 0.367),
    (0.35, 0.41),
    (0.343, 0.42),
    (0.302, 0.42),
    (0.294, 0.41),
    (0.289, 0.37),
];
const TUB_FAR: [(f64, f64); 7] = [
    (0.300, 0.0),
    (0.311, 0.011),
    (0.33, 0.36),
    (0.35, 0.37),
    (0.35, 0.42),
    (0.298, 0.42),
    (0.289, 0.37),
];

/// The concrete tub (0.7 m square, 0.42 high) in one piece, hollow at the top, and its soil 2 cm below the rim;
/// `groove`: a groove round it under the rim.
pub fn _tub(groove: bool) {
    let n = low(if groove { 24 } else { 16 }, 12);
    let rows: Vec<(f64, f64)> = if is_low() {
        TUB_FAR.to_vec()
    } else if groove {
        TUB.to_vec()
    } else {
        [&TUB[..4], &TUB[7..]].concat()
    };
    let sections: Vec<Vec<P3>> = rows
        .iter()
        .map(|&(r, y)| ring!(r, r, z = y, verts = n, square = 9))
        .collect();
    loft!("tub", &sections, material = M.concrete);
    let soil = loft!(
        "soil",
        &[
            ring!(0.291, 0.291, z = 0.37, verts = n, square = 9),
            ring!(0.292, 0.292, z = 0.395, verts = n, square = 9)
        ],
        material = M.soil
    );
    if !is_low() {
        jitter!(soil, 0.004, size = 0.08, seed = 3.0);
    }
}

/// A flower open to the sky: `petals` rounded petals round a raised eye, at game (x, y, z), `r` across its
/// petals.
#[allow(clippy::too_many_arguments)]
pub fn _bloom(
    f: &GameFrame,
    name: &str,
    x: f64,
    y: f64,
    z: f64,
    r: f64,
    petals: usize,
    mat: Mat,
    eye: Mat,
    tilt: (f64, f64, f64),
    detail: usize,
) {
    let mut pts = Vec::new();
    let n = petals * detail;
    for i in 0..n {
        let a = TAU * i as f64 / n as f64;
        let k = pow((cos(petals as f64 * a / 2.0)).abs(), 0.7);
        let rr = r * (0.42 + 0.58 * k);
        pts.push((rr * cos(a), rr * sin(a)));
    }
    fextrude!(
        f,
        name,
        &pts,
        0.01,
        (x, y, z),
        mat,
        plane = Facing::Top,
        rot = tilt
    );
    fcyl!(
        f,
        format!("{name}_eye"),
        r * 0.3,
        0.014,
        (x, y + 0.002, z),
        eye,
        verts = 5,
        rot = tilt
    );
}

// --- the street-name sign: two green blades crossing on a pole -----------------------------------------------
/// Words of dark green lettering on a blade's white panel, on one face (+1 front, -1 back): each word a capital,
/// then small letters on the same baseline, a few with ascenders.
pub fn _lettering(
    f: &GameFrame,
    name: &str,
    length: f64,
    y: f64,
    along_x: bool,
    face: i32,
    rng: &mut Random,
) {
    let face = face as f64;
    let mut x = -length / 2.0;
    let mut k = 0;
    let mut start = true;
    while x < length / 2.0 - 0.04 {
        let w = rng.choice(&[0.03, 0.035, 0.04]);
        let h = rng.choice(&[0.06, 0.06, 0.07]); // (drawn as before, so the rest of the sign's numbers stay put)
        if rng.random() < 0.14 && k != 0 {
            x += 0.04; // a space between words
            start = true;
        }
        let h = if start {
            0.075
        } else if h > 0.065 {
            0.068
        } else {
            0.05
        };
        start = false;
        let c = x + w / 2.0;
        if along_x {
            _box(
                f,
                format!("{name}{k}"),
                w,
                h,
                0.004,
                (c, y - 0.0375 + h / 2.0, face * 0.019),
                M.sign,
                0.0,
            );
        } else {
            _box(
                f,
                format!("{name}{k}"),
                0.004,
                h,
                w,
                (face * 0.019, y - 0.0375 + h / 2.0, c),
                M.sign,
                0.0,
            );
        }
        x += w + 0.012;
        k += 1;
    }
}

/// A round steel post from a flanged foot bolted to the ground up to `top`: the foot's profile turned, a collar
/// where the post leaves it.
pub fn _post(f: &GameFrame, top: f64, foot: f64, r: f64) {
    let n = low(12, 6);
    let c = (r + 0.01).min(foot);
    let prof = [
        (0.0, 0.0),
        (foot, 0.0),
        (foot, 0.01),
        (c.max(foot - 0.012), 0.016),
        (c, 0.03),
        (c.min(r + 0.006), 0.09),
        (c, 0.1),
        (c, 0.115),
        (r, 0.125),
        (r, top),
        (0.0, top),
    ];
    let far = [
        (0.0, 0.0),
        (foot, 0.0),
        (c, 0.03),
        (r, 0.12),
        (r, top),
        (0.0, top),
    ];
    revolve!(
        "post",
        if !is_low() { &prof[..] } else { &far[..] },
        material = M.pole,
        segments = n
    );
    if !is_low() && foot > r + 0.03 {
        // the foot's four bolts
        for k in 0..4 {
            let a = TAU * (k as f64 + 0.5) / 4.0;
            fcyl!(
                f,
                format!("bolt{k}"),
                0.008,
                0.012,
                ((foot - 0.018) * cos(a), 0.014, (foot - 0.018) * sin(a)),
                M.pole,
                verts = 6
            );
        }
    }
}

/// A clamp band round the post at height y.
pub fn _band(f: &GameFrame, name: impl AsRef<str>, y: f64, r: f64, h: f64) {
    fcyl!(
        f,
        name,
        r,
        h,
        (0, y, 0),
        M.pole,
        verts = low(12, 6),
        bevel = low(0.004, 0.0),
        bevel_segments = 1
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use artkit::far::far_of;
    use artkit::parts::measure;
    use artkit::scene::{self, meshes};

    const F: GameFrame = GameFrame::new(0.0);

    #[test]
    fn taxi_reads_from_the_side_it_faces() {
        scene::reset();
        let front = taxi_word(&F, "word", (0.0, 2.0), 0.1, 0.04, 1, M.black, 0.004);
        // T (2 boxes), A (3), X (2), I (1), standing 4 mm proud of the face
        assert_eq!(front.len(), 8);
        let (lo, hi) = measure(&front);
        assert!((lo[1] - -0.044).abs() < 1e-6 && (hi[1] - -0.04).abs() < 1e-6);
        // seen from the front the game's +x (Blender's -x) is on the left: the T's bar is there
        assert!(front[0].location().x < 0.0);
        let back = taxi_word(&F, "back", (0.0, 2.0), 0.1, -0.04, -1, M.black, 0.004);
        assert!(back[0].location().x > 0.0);
    }

    #[test]
    fn a_chequer_leaves_every_other_square() {
        scene::reset();
        let squares = chequer(
            &F, "check", -0.21, 0.21, 2.0, 0.035, 0.03, 1, M.black, 2, 0.003,
        );
        assert_eq!(squares.len(), 14);
        let (lo, hi) = measure(&squares);
        assert!((hi[0] - lo[0] - 0.42).abs() < 1e-6 && (hi[2] - lo[2] - 0.06).abs() < 1e-6);
    }

    #[test]
    fn a_far_bench_end_has_no_scrolls() {
        scene::reset();
        let end = |o: BenchEnd| {
            bench_end(
                &F,
                "end",
                0.8,
                0.415,
                0.19,
                -0.21,
                (0.885, -0.272),
                M.iron,
                o,
            )
            .len()
        };
        let arm = BenchEnd {
            arm: Some(0.68),
            ..BenchEnd::default()
        };
        // back, front, its scroll, stretcher, bracket, rail and two feet
        assert_eq!(end(arm.clone()), 8);
        assert_eq!(
            end(BenchEnd {
                scrolls: false,
                ..arm
            }),
            5
        );
        // without an arm the front leg stops at the seat rail
        assert_eq!(end(BenchEnd::default()), 7);
    }

    #[test]
    fn a_far_tub_has_fewer_sides_and_smooth_soil() {
        scene::reset();
        _tub(true);
        let near: usize = meshes()
            .iter()
            .map(|o| o.read(|p| p.mesh.verts.len()))
            .sum();
        scene::reset();
        far_of(|| _tub(true));
        let far: usize = meshes()
            .iter()
            .map(|o| o.read(|p| p.mesh.verts.len()))
            .sum();
        assert_eq!((near, far), (24 * 14 + 24 * 2, 12 * 7 + 12 * 2));
    }
}
