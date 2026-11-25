//! The washing line, whose washing is pegged out line by line, and the log store.

use super::common::{F, M, bent};
use artkit::far::{is_low, low};
use artkit::forms::rounded_rect;
use artkit::kit;
use artkit::materials::Mat;
use artkit::math::{Axis, P3, PI, TAU, cos, cosd, sin, sind};
use artkit::motion::grow_in_turn;
use artkit::parts::numbered;
use artkit::parts::{join, join_all, new_parts};
use artkit::rand::Random;
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fcyl, fextrude, fsweep, loft, revolve, ring, settle, soft, torus};

// --- the washing line (3x1): two T-posts, three lines; the washing is pegged out and stays out --------------
const LINE_Y: f64 = 1.76;
const LINES: [f64; 3] = [-0.26, 0.0, 0.26];

/// A piece of washing cut to `outline` [(across, down from the line), ...], hung at x on the line at depth z: a slab
/// `t` thick (thinner than the line: flattened at rest, the washing is inside it).
fn hang(name: &str, outline: &[(f64, f64)], x: f64, z: f64, m: Mat, t: f64) -> Obj {
    let pts: Vec<(f64, f64)> = outline.iter().map(|&(u, v)| (x + u, LINE_Y - v)).collect();
    fextrude!(F, name, &pts, t, (0, 0, z), m)
}

/// A thing hung on the line at x, depth z, `w` wide and `h` down, in material `m`, with two pegs: a T-shirt with
/// its sleeves and neck, trousers with their legs apart, a pair of socks, a towel with a woven band, a sheet with a
/// soft hem, a pillowcase.
fn garment(kind: &str, x: f64, z: f64, w: f64, h: f64, m: Mat) {
    let hw = w / 2.0;
    let pegs: Vec<f64> = match kind {
        "shirt" => {
            let body = w * 0.31;
            let out = [
                (-body, 0.0),
                (-body * 0.45, 0.0),
                (-body * 0.3, 0.04),
                (body * 0.3, 0.04),
                (body * 0.45, 0.0),
                (body, 0.0),
                (hw, h * 0.12),
                (hw - 0.02, h * 0.42),
                (body, h * 0.32),
                (body, h),
                (-body, h),
                (-body, h * 0.32),
                (-hw + 0.02, h * 0.42),
                (-hw, h * 0.12),
            ];
            hang(&numbered("shirt"), &out, x, z, m, 0.006);
            vec![x - body * 0.8, x + body * 0.8]
        }
        "trousers" => {
            let out = [
                (-hw, 0.0),
                (hw, 0.0),
                (hw, h * 0.2),
                (hw * 0.96, h),
                (hw * 0.12, h),
                (0.0, h * 0.34),
                (-hw * 0.12, h),
                (-hw * 0.96, h),
                (-hw, h * 0.2),
            ];
            hang(&numbered("trousers"), &out, x, z, m, 0.006);
            vec![x - w * 0.4, x + w * 0.4]
        }
        "socks" => {
            for s in [-1.0, 1.0] {
                let cx = x + s * w * 0.25;
                let sock = [
                    (-0.04, 0.0),
                    (0.04, 0.0),
                    (0.04, h * 0.68),
                    (0.095, h * 0.76),
                    (0.1, h * 0.9),
                    (0.085, h),
                    (-0.03, h),
                    (-0.04, h * 0.92),
                ];
                let sock: Vec<(f64, f64)> = if s > 0.0 {
                    sock.to_vec()
                } else {
                    sock.iter().map(|&(u, v)| (-u, v)).collect()
                };
                hang(&numbered("sock"), &sock, cx, z, m, 0.006);
                fbox!(
                    F,
                    numbered("sock_cuff"),
                    0.08,
                    0.04,
                    0.008,
                    (cx, LINE_Y - 0.025, z),
                    M.white
                );
            }
            vec![x - w * 0.25, x + w * 0.25]
        }
        "sheet" => {
            let mut out = vec![(-hw, 0.0), (hw, 0.0)];
            out.extend((0..13).map(|i| {
                (
                    hw - w * i as f64 / 12.0,
                    h + 0.012 * sin(PI * i as f64 / 3.0),
                )
            }));
            hang(&numbered("sheet"), &out, x, z, m, 0.006);
            fbox!(
                F,
                numbered("sheet_hem"),
                w,
                0.025,
                0.008,
                (x, LINE_Y - h + 0.03, z),
                M.white
            );
            vec![x - hw + 0.05, x, x + hw - 0.05]
        }
        _ => {
            // a towel, a pillowcase
            hang(
                &numbered(kind),
                &[(-hw, 0.0), (hw, 0.0), (hw, h), (-hw, h)],
                x,
                z,
                m,
                0.006,
            );
            let band = if m.name != M.white.name {
                M.white
            } else {
                M.sage
            };
            let towel = kind == "towel";
            let y = if towel { h * 0.82 } else { h * 0.15 };
            fbox!(
                F,
                numbered("band"),
                w,
                if towel { 0.045 } else { 0.02 },
                0.008,
                (x, LINE_Y - y, z),
                band
            );
            vec![x - hw + 0.05, x + hw - 0.05]
        }
    };
    for px in pegs {
        // a wooden peg: two legs split at the foot, astride the line
        hang(
            &numbered("peg"),
            &[
                (-0.008, -0.014),
                (0.008, -0.014),
                (0.009, 0.02),
                (0.0045, 0.05),
                (0.0, 0.036),
                (-0.0045, 0.05),
                (-0.009, 0.02),
            ],
            px,
            z,
            M.peg,
            0.012,
        );
    }
}

pub fn clothesline() -> Vec<Obj> {
    for x in [-1.32, 1.32] {
        // the post, a cap on its top; the arm across, capped; two braces bent up into it; a flange at its foot
        fsweep!(
            F,
            numbered("post"),
            &[(x, 0.0, 0), (x, LINE_Y + 0.045, 0)],
            0.03,
            M.posts,
            verts = low(12, 6)
        );
        fcyl!(
            F,
            numbered("post_cap"),
            0.034,
            0.012,
            (x, LINE_Y + 0.051, 0),
            M.posts,
            verts = low(12, 6),
            bevel = low(0.004, 0.0),
            bevel_segments = 1
        );
        fsweep!(
            F,
            numbered("arm"),
            &[(x, LINE_Y + 0.02, -0.343), (x, LINE_Y + 0.02, 0.343)],
            0.022,
            M.posts,
            verts = low(10, 5)
        );
        for s in [-1.0, 1.0] {
            fcyl!(
                F,
                numbered("arm_cap"),
                0.025,
                0.012,
                (x, LINE_Y + 0.02, s * 0.345),
                M.posts,
                rot = (90, 0, 0),
                verts = low(10, 5)
            );
            bent(
                &numbered("brace"),
                &[
                    (x, LINE_Y - 0.28, 0.02 * s),
                    (x, LINE_Y - 0.1, 0.12 * s),
                    (x, LINE_Y, s * 0.25),
                ],
                0.012,
                M.posts,
                0.06,
                low(6, 4),
                3,
                1.0,
            );
        }
        revolve!(
            numbered("foot"),
            &[
                (0.0, 0.0),
                (0.06, 0.0),
                (0.056, 0.01),
                (0.04, 0.03),
                (0.035, 0.06),
                (0.0, 0.06)
            ],
            at = F.at(x, 0, 0),
            material = M.posts,
            segments = low(12, 6)
        );
    }
    for z in LINES {
        fsweep!(
            F,
            numbered("line"),
            &[(-1.34, LINE_Y, z), (1.34, LINE_Y, z)],
            0.008,
            M.line,
            verts = low(8, 4)
        );
        if !is_low() {
            // a tensioner on each line by the right post
            fbox!(
                F,
                numbered("tensioner"),
                0.06,
                0.02,
                0.022,
                (1.2, LINE_Y, z),
                M.posts,
                bevel = 0.005,
                bevel_segments = 1
            );
        }
    }
    // the basket by the left post, woven, a rim and two handles, the rest of the washing folded in it
    let (bx, bz) = (-1.0, 0.24);
    let p = F.at(bx, 0, bz);
    let (px, pz) = (p.x, p.y);
    let rings = [(0.0, 0.22, 0.125), (0.02, 0.24, 0.142), (0.22, 0.25, 0.152)];
    let secs: Vec<Vec<P3>> = rings
        .iter()
        .map(|&(y, rx, rz)| ring!(rx, rz, z = y, verts = low(20, 8), square = 4, at = (px, pz)))
        .collect();
    loft!("basket", &secs, material = M.basket);
    let rim: Vec<(f64, f64, f64)> = rounded_rect(0.52, 0.322, 0.08, low(3, 1))
        .iter()
        .map(|&(u, v)| (bx + u, 0.235, bz + v))
        .collect();
    fsweep!(
        F,
        "basket_rim",
        &rim,
        0.014,
        M.basket,
        verts = low(6, 4),
        closed = true
    );
    for s in [-1.0, 1.0] {
        if !is_low() {
            torus!(
                numbered("basket_handle"),
                0.035,
                0.009,
                at = F.at(bx + s * 0.262, 0.23, bz),
                material = M.basket,
                rot = (90, 0, 90),
                major_segments = 8,
                minor_segments = 4
            );
        }
    }
    for (k, (m, y)) in [
        (M.folded, 0.25),
        (M.folded_pink, 0.278),
        (M.folded_sage, 0.302),
    ]
    .into_iter()
    .enumerate()
    {
        let k = k as f64;
        soft!(
            F,
            numbered("folded"),
            0.34 - 0.04 * k,
            0.026,
            0.22 - 0.02 * k,
            (bx + 0.01 * k, y, bz),
            m,
            round = 0.012,
            puff = 0.004,
            flat = 1,
            round_segments = 1
        );
    }
    if is_low() {
        // from afar: at rest, the washing in
        return settle!(&[join_all("clothesline")], centre = false);
    }
    join(&meshes(), "posts", Some(P3::from((0, 0, 0))));
    // The washing, pegged on the lines, a part a line: each line's origin is on it, so at rest its washing is
    // flattened into it, and `washing` lets it down, the front line first, then the middle, then the back (a third
    // of the clip each: played along with the use, a line at a time).
    let hung = [
        ("towel", -0.9, 0, 0.5, 0.6, M.towel),
        ("pillowcase", -0.3, 0, 0.42, 0.5, M.white),
        ("shirt", 0.42, 0, 0.62, 0.48, M.pink),
        ("trousers", -0.62, 1, 0.38, 0.74, M.denim),
        ("shirt", 0.18, 1, 0.5, 0.4, M.yellow),
        ("towel", 0.92, 1, 0.32, 0.56, M.white),
        ("socks", -1.0, 2, 0.26, 0.3, M.red),
        ("sheet", 0.0, 2, 0.9, 0.66, M.washing),
        ("towel", 0.86, 2, 0.4, 0.46, M.sage),
    ];
    let mut lines = Vec::new();
    for row in [2, 1, 0] {
        // LINES runs back to front; the front line goes first
        let washing = new_parts(|| {
            for &(kind, x, r, w, h, m) in &hung {
                if r == row {
                    garment(kind, x, LINES[r], w, h, m);
                }
            }
        });
        let letter = ["a", "b", "c"][lines.len()];
        let line = join(
            &washing,
            &format!("washing_{letter}"),
            Some(F.at(0, LINE_Y, LINES[row])),
        );
        line.edit(|o| o.scale.z = 0.001);
        lines.push(line);
    }
    let objs = settle!(centre = false);
    grow_in_turn(&lines, "washing", Some(1.5), Axis::Z);
    objs
}

// --- the log store (1x1): a black iron rack under a little roof, stacked with logs, ends to the front --------
const LS_Z: f64 = -0.24;

/// A log's cross-section (x, height about its middle): round (its bark lumpy), split in half or in a quarter.
fn log_section(kind: &str, r: f64, rnd: &mut Random, n: usize) -> Vec<(f64, f64)> {
    if kind == "round" {
        return (0..n)
            .map(|i| {
                let a = r * rnd.uniform(0.93, 1.05) * cos(TAU * i as f64 / n as f64);
                (
                    a,
                    r * rnd.uniform(0.93, 1.05) * sin(TAU * i as f64 / n as f64),
                )
            })
            .collect();
    }
    if kind == "half" {
        return (0..n - 1)
            .map(|i| {
                let (u, v) = (
                    r * cos(PI * i as f64 / (n - 2) as f64),
                    r * 0.15 + r * 0.85 * sin(PI * i as f64 / (n - 2) as f64),
                );
                (u, v - r * 0.45)
            })
            .collect();
    }
    let mut pts = vec![(0.0, 0.0)];
    pts.extend((0..n - 1).map(|i| {
        (
            r * 1.25 * cos(PI / 2.0 * i as f64 / (n - 2) as f64),
            r * 1.25 * sin(PI / 2.0 * i as f64 / (n - 2) as f64),
        )
    }));
    pts.iter()
        .map(|&(u, v)| (u - r * 0.5, v - r * 0.5))
        .collect()
}

pub fn firewood_stack() -> Vec<Obj> {
    let mut rnd = Random::new(11);
    // the rack: a base frame on four feet, two ends each one bent tube, rails along the top
    fbox!(
        F,
        "rack_base",
        0.9,
        0.04,
        0.36,
        (0, 0.06, LS_Z),
        M.iron,
        bevel = low(0.006, 0.0),
        bevel_segments = 1
    );
    for x in [-0.44, 0.44] {
        bent(
            &numbered("rack_end"),
            &[
                (x, 0.0, LS_Z - 0.16),
                (x, 1.0, LS_Z - 0.16),
                (x, 1.0, LS_Z + 0.16),
                (x, 0.0, LS_Z + 0.16),
            ],
            0.0175,
            M.iron,
            0.03,
            low(8, 4),
            3,
            1.0,
        );
        fbox!(
            F,
            numbered("rack_side"),
            0.035,
            0.035,
            0.36,
            (x, 0.08, LS_Z),
            M.iron
        );
        for z in [LS_Z - 0.16, LS_Z + 0.16] {
            fbox!(
                F,
                numbered("rack_foot"),
                0.06,
                0.008,
                0.05,
                (x, 0.004, z),
                M.iron,
                bevel = low(0.002, 0.0),
                bevel_segments = 1
            );
        }
    }
    for z in [LS_Z - 0.16, LS_Z + 0.16] {
        fsweep!(
            F,
            numbered("rack_top"),
            &[(-0.46, 1.0, z), (0.46, 1.0, z)],
            0.0175,
            M.iron,
            verts = low(8, 4)
        );
    }
    // the shelter: a pent roof of corrugated iron, sloping to the back, on two bearers
    for x in [-0.44, 0.44] {
        kit::board(
            &F,
            &numbered("bearer"),
            (x, 1.06, LS_Z + 0.17),
            (x, 1.0, LS_Z - 0.17),
            0.035,
            0.035,
            M.iron,
            (1, 0, 0),
            0.0,
        );
    }
    let n = low(14, 1);
    if is_low() {
        kit::board(
            &F,
            "roof_sheet",
            (0, 1.088, LS_Z + 0.185),
            (0, 1.03, LS_Z - 0.185),
            1.0,
            0.012,
            M.iron,
            (0, 1, 0),
            0.0,
        );
    } else {
        let wave: Vec<(f64, f64)> = (0..4 * n + 1)
            .map(|i| {
                (
                    -0.5 + i as f64 / (4 * n) as f64,
                    0.008 * sin(PI / 2.0 * i as f64),
                )
            })
            .collect();
        let mut prof: Vec<(f64, f64)> = wave.iter().map(|&(u, v)| (u, v + 0.003)).collect();
        prof.extend(wave.iter().rev().map(|&(u, v)| (u, v - 0.003)));
        let prof: Vec<(f64, f64)> = prof.iter().map(|&(u, v)| (-u, v)).collect();
        fsweep!(
            F,
            "roof_sheet",
            &[(0, 1.088, LS_Z + 0.185), (0, 1.03, LS_Z - 0.185)],
            0,
            M.iron,
            profile = prof,
            up = (1, 0, 0),
            angle = 60
        );
    }
    fbox!(
        F,
        "roof_drip",
        1.0,
        0.04,
        0.012,
        (0, 1.075, LS_Z + 0.185),
        M.iron
    );
    // the logs: six rows in brick bond, ends to the front, round and split, bark lumpy; their sawn ends pale with a
    // darker heart
    let barks = [M.bark, M.bark_dark, M.bark_light];
    let ends = [M.end_grain, M.end_grain_dark];
    for row in 0..6 {
        for col in 0..6 - (row % 2) {
            let x = -0.36 + col as f64 * 0.145 + (row % 2) as f64 * 0.0725;
            let y = 0.15 + row as f64 * 0.13;
            let r = 0.066 * rnd.uniform(0.92, 1.05);
            let spin = rnd.uniform(0.0, 360.0);
            let kind = ["round", "round", "half", "round", "quarter"][(row * 5 + col * 3) % 5];
            let nv = low(8, 5);
            // its own numbers: the far version draws fewer
            let sec = log_section(kind, r, &mut Random::new((row * 10 + col) as i128), nv);
            let (c, s_) = (cosd(spin), sind(spin));
            let sec: Vec<(f64, f64)> = sec
                .iter()
                .map(|&(u, v)| (u * c - v * s_, u * s_ + v * c))
                .collect();
            let prof: Vec<(f64, f64)> = sec.iter().map(|&(u, v)| (-u, v)).collect();
            fsweep!(
                F,
                numbered("log"),
                &[(x, y, LS_Z - 0.17), (x, y, LS_Z + 0.169)],
                0,
                barks[(row + col) % 3],
                profile = prof,
                up = (0, 0, 1),
                smooth = false
            );
            let face: Vec<(f64, f64)> = sec
                .iter()
                .map(|&(u, v)| (x + u * 0.94, y + v * 0.94))
                .collect();
            fextrude!(
                F,
                numbered("log_end"),
                &face,
                0.004,
                (0, 0, LS_Z + 0.171),
                ends[(row * 7 + col) % 2]
            );
            if kind == "round" && !is_low() {
                fcyl!(
                    F,
                    numbered("log_heart"),
                    r * 0.45,
                    0.002,
                    (x, y, LS_Z + 0.1735),
                    ends[(row * 7 + col + 1) % 2],
                    rot = (90, 0, 0),
                    verts = 6
                );
            }
        }
    }
    settle!(&[join_all("firewoodStack")], centre = false)
}
