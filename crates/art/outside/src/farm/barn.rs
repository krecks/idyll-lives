//! The red barn: a gambrel-roofed barn in board-and-batten siding, with sliding doors, a hayloft, windows down
//! each side and a louvred cupola, and its far version.

use super::barn_doors::{barn_doors, barn_window};
use super::common::{F, M, P2, bar_on, eased, offset, prism, pts, shell, strut};
use artkit::parts::{join_all, settle};
use artkit::place::Coord;
use artkit::place::N;
use artkit::py;
use artkit::scene::Obj;
use artkit::shapes::AUTO_WIDTH;
use artkit::{cube, fbox, fcone, fsphere, fsweep, points, rod, sweep};

// --- the barn --------------------------------------------------------------------------------------------
pub(super) const BW: f64 = 2.8; // half the walls' width and depth
const BD: f64 = 2.75;
const EAVE: f64 = 3.0;
const KNEE: f64 = 4.3;
const KNEE_X: f64 = 1.75;
const RIDGE: f64 = 4.95;
const ROOF_T: f64 = 0.12; // roof thickness, half its depth (an overhang at the gables)
const ROOF_D: f64 = 2.92;

#[allow(non_snake_case)]
fn BARN_WALL() -> Vec<P2> {
    points![
        (-BW, 0),
        (BW, 0),
        (BW, EAVE),
        (KNEE_X, KNEE),
        (0, RIDGE),
        (-KNEE_X, KNEE),
        (-BW, EAVE),
    ]
    .to_vec()
}

/// The roof's underside: the walls' gambrel, carried 0.2 m past the eaves.
fn barn_roof_line() -> Vec<P2> {
    let (dx, dy) = (KNEE_X - BW, KNEE - EAVE);
    let k = py::hypot(dx, dy);
    let (ex, ey) = (BW - 0.2 * dx / k, EAVE - 0.2 * dy / k);
    points![
        (-ex, ey),
        (-KNEE_X, KNEE),
        (0, RIDGE),
        (KNEE_X, KNEE),
        (ex, ey),
    ]
    .to_vec()
}

/// The walls' top at `x`.
fn barn_height(x: f64) -> f64 {
    let x = x.abs();
    if x > KNEE_X {
        return EAVE + (KNEE - EAVE) * (BW - x) / (BW - KNEE_X);
    }
    KNEE + (RIDGE - KNEE) * (KNEE_X - x) / KNEE_X
}

/// Walls, plinth, roof, ridge cap: what the far version keeps too.
fn barn_shell(detail: bool) {
    prism("walls", &BARN_WALL(), -BD, BD, M.barn, 'z');
    cube!(
        "plinth",
        (2.0 * BW + 0.08, 2.0 * BD + 0.08, 0.3),
        at = (0, 0, 0.15),
        material = M.stone,
        bevel = if detail { 0.02 } else { 0.0 }
    );
    shell(
        "roof",
        &barn_roof_line(),
        ROOF_T,
        -ROOF_D,
        ROOF_D,
        M.barn_roof,
        'z',
    );
    if !detail {
        cube!(
            "ridge",
            (0.2, 2.0 * ROOF_D + 0.04, 0.08),
            at = (0, 0, RIDGE + ROOF_T + 0.02),
            material = M.barn_roof
        );
        return;
    }
    // a rolled ridge cap, the metal roof's standing seams from eave to ridge, and the plinth's dressed stones
    eased(sweep!(
        "ridge",
        &[
            F.at(0, RIDGE + ROOF_T + 0.01, -ROOF_D - 0.02),
            F.at(0, RIDGE + ROOF_T + 0.01, ROOF_D + 0.02)
        ],
        0.0,
        material = M.barn_roof,
        profile = [
            (0.035, 0.0),
            (0.0, 0.13),
            (-0.03, 0.1),
            (-0.03, -0.1),
            (0.0, -0.13)
        ],
        up = (0, 0, 1),
        smooth = false
    ));
    let mut seam = offset(&barn_roof_line(), ROOF_T - 0.005);
    for (end, nxt) in [(0, 1), (-1, -2)] {
        // stopped short of the eaves' edge
        let at = |i: i32| {
            if i < 0 {
                (seam.len() as i32 + i) as usize
            } else {
                i as usize
            }
        };
        let (end, nxt) = (at(end), at(nxt));
        let ((u0, v0), (u1, v1)) = (
            (seam[end].0.f(), seam[end].1.f()),
            (seam[nxt].0.f(), seam[nxt].1.f()),
        );
        let k = 0.1 / py::hypot(u1 - u0, v1 - v0);
        seam[end] = (N::F(u0 + (u1 - u0) * k), N::F(v0 + (v1 - v0) * k));
    }
    let n = 17;
    for i in 0..n {
        let w = -ROOF_D + 0.18 + (2.0 * ROOF_D - 0.36) * i as f64 / (n - 1) as f64;
        let points: Vec<(f64, f64, f64)> = seam.iter().map(|&(u, v)| (u.f(), v.f(), w)).collect();
        // a seam's fold a centimetre across, under the width a bevel would show on
        fsweep!(
            F,
            format!("seam{i}"),
            &points,
            0.0,
            M.barn_roof,
            profile = [(0.04, -0.005), (0.04, 0.005), (0.0, 0.012), (0.0, -0.012)],
            up = (1, 0, 0),
            smooth = false
        );
    }
    barn_stones();
}

/// Two courses of dressed stones proud of the plinth all round, their joints staggered.
fn barn_stones() {
    let (hx, hz) = (BW + 0.04, BD + 0.04);
    for (c, (y, h)) in [(0.075, 0.13), (0.215, 0.13)].into_iter().enumerate() {
        let c = c as i64;
        for (face, (length, run)) in [(2.0 * hx, 'x'), (2.0 * hz, 'z')].into_iter().enumerate() {
            let k = 1.max(py::round(length / 0.8));
            let step = length / (k as f64 + 0.5 * (c % 2) as f64);
            let starts: Vec<f64> = (0..k + (c % 2))
                .map(|i| {
                    -length / 2.0 + (if c % 2 != 0 { step / 2.0 } else { 0.0 }) + i as f64 * step
                })
                .collect();
            for (i, &a) in starts.iter().enumerate() {
                let b = (length / 2.0).min(a + step);
                let a = (-length / 2.0).max(a);
                let (m, l) = ((a + b) / 2.0, b - a - 0.02);
                for s in [-1, 1] {
                    let s = s as f64;
                    if run == 'x' {
                        fbox!(
                            F,
                            format!("stone{c}{face}{i}{}", s as i32),
                            l,
                            h,
                            0.03,
                            (m, y, s * (hz + 0.006)),
                            M.stone,
                            bevel = 0.01,
                            bevel_segments = 1
                        );
                    } else {
                        fbox!(
                            F,
                            format!("stone{c}{face}{i}{}", s as i32),
                            0.03,
                            h,
                            l,
                            (s * (hx + 0.006), y, m),
                            M.stone,
                            bevel = 0.01,
                            bevel_segments = 1
                        );
                    }
                }
            }
        }
    }
}

/// Where the battens stop: (u0, u1, y0, y1) round each door and window, per face.
fn barn_openings(face: &str) -> Vec<(f64, f64, f64, f64)> {
    match face {
        "front" => vec![(-1.38, 1.38, 0.0, 2.8), (-0.58, 0.58, 3.12, 4.18)],
        "back" => vec![(0.53, 1.67, 0.0, 2.27), (-0.47, 0.47, 3.23, 4.07)],
        _ => [-1.6, 0.0, 1.6]
            .iter()
            .map(|&c| (c - 0.5, c + 0.5, 1.22, 2.25))
            .collect(),
    }
}

/// The heights a batten at `u` runs over, from the plinth to `top`, broken round `openings`.
fn batten_spans(u: f64, top: f64, openings: &[(f64, f64, f64, f64)]) -> Vec<(f64, f64)> {
    let mut spans = vec![(0.3, top)];
    for &(u0, u1, y0, y1) in openings {
        if u0 - 0.03 < u && u < u1 + 0.03 {
            spans = spans
                .iter()
                .flat_map(|&(a, b)| {
                    [
                        (a, if y0 < b { y0 } else { b }),
                        (if y1 > a { y1 } else { a }, b),
                    ]
                })
                .filter(|piece| piece.1 - piece.0 > 0.08)
                .collect();
        }
    }
    spans
}

/// Board-and-batten siding: raised battens in the walls' colour, round the doors and windows.
fn barn_battens() {
    for (s, face) in [(1.0, "front"), (-1.0, "back")] {
        for i in 0..15 {
            let u = -2.45 + 0.35 * i as f64;
            for (k, (a, b)) in batten_spans(u, barn_height(u) - 0.05, &barn_openings(face))
                .into_iter()
                .enumerate()
            {
                bar_on(
                    &format!("batten_{face}{i}_{k}"),
                    [u, a, s * BD],
                    [u, b, s * BD],
                    [0.0, 0.0, s],
                    0.06,
                    0.06,
                    0.03,
                    AUTO_WIDTH,
                    M.barn,
                );
            }
        }
    }
    for s in [-1, 1] {
        for i in 0..15 {
            let u = -2.45 + 0.35 * i as f64;
            for (k, (a, b)) in batten_spans(u, EAVE - 0.05, &barn_openings("side"))
                .into_iter()
                .enumerate()
            {
                bar_on(
                    &format!("batten_side{s}{i}_{k}"),
                    [s as f64 * BW, a, u],
                    [s as f64 * BW, b, u],
                    [s as f64, 0.0, 0.0],
                    0.06,
                    0.06,
                    0.03,
                    AUTO_WIDTH,
                    M.barn,
                );
            }
        }
    }
}

fn barn_corners() {
    for sx in [-1, 1] {
        for sz in [-1, 1] {
            fbox!(
                F,
                format!("corner{sx}{sz}"),
                0.16,
                EAVE - 0.3,
                0.16,
                (
                    sx as f64 * (BW - 0.06),
                    0.3 + (EAVE - 0.3) / 2.0,
                    sz as f64 * (BD - 0.06)
                ),
                M.barn_trim
            );
        }
    }
    // fascia boards along the gable ends' roof edge
    for sz in [-1, 1] {
        let line = barn_roof_line();
        for i in 0..line.len() - 1 {
            strut(
                &format!("fascia{sz}{i}"),
                (line[i].0.f(), line[i].1.f()),
                (line[i + 1].0.f(), line[i + 1].1.f()),
                0.14,
                sz as f64 * (ROOF_D - 0.02),
                sz as f64 * (ROOF_D + 0.02),
                M.barn_trim,
                'z',
            );
        }
    }
}

/// A louvred cupola on the ridge with a weathervane on top.
fn cupola(detail: bool) {
    let y0 = RIDGE + 0.02;
    fbox!(F, "cupola", 0.7, 0.5, 0.7, (0, y0 + 0.25, 0), M.barn_trim);
    if detail {
        // its base saddled down onto the roof, a band under its roof, boards at its corners and a ball under the vane
        fbox!(
            F,
            "cupola_base",
            0.8,
            0.3,
            0.8,
            (0, y0 - 0.1, 0),
            M.barn_roof,
            bevel = 0.015,
            bevel_segments = 1
        );
        fbox!(
            F,
            "cupola_band",
            0.8,
            0.06,
            0.8,
            (0, y0 + 0.5, 0),
            M.barn_trim,
            bevel = 0.012,
            bevel_segments = 1
        );
        for sx in [-1, 1] {
            for sz in [-1, 1] {
                fbox!(
                    F,
                    format!("cupola_corner{sx}{sz}"),
                    0.06,
                    0.46,
                    0.06,
                    (sx as f64 * 0.345, y0 + 0.24, sz as f64 * 0.345),
                    M.barn_trim
                );
            }
        }
        fsphere!(
            F,
            "vane_ball",
            0.05,
            (0, y0 + 0.84, 0),
            M.iron,
            segments = 10,
            rings = 6
        );
        for s in [-1, 1] {
            for i in 0..4 {
                let y = y0 + 0.12 + i as f64 * 0.08;
                fbox!(
                    F,
                    format!("louvre_f{s}{i}"),
                    0.5,
                    0.04,
                    0.02,
                    (0, y, s as f64 * 0.355),
                    M.shadow
                );
                fbox!(
                    F,
                    format!("louvre_s{s}{i}"),
                    0.02,
                    0.04,
                    0.5,
                    (s as f64 * 0.355, y, 0),
                    M.shadow
                );
            }
        }
    }
    fcone!(
        F,
        "cupola_roof",
        0.62,
        0.02,
        0.3,
        (0, y0 + 0.65, 0),
        M.barn_roof,
        verts = 4,
        smooth = false,
        rot = (0, 0, 45)
    );
    // the weathervane: a rod, an arrow, and the four points
    rod!(
        "vane_rod",
        F.at(0, y0 + 0.75, 0),
        F.at(0, 6.0, 0),
        0.015,
        material = M.iron,
        verts = 6
    );
    if detail {
        cube!(
            "vane_shaft",
            (0.55, 0.015, 0.015),
            at = F.at(0, 5.9, 0),
            material = M.iron
        );
        eased(fcone!(
            F,
            "vane_head",
            0.06,
            0.0,
            0.14,
            (-0.32, 5.9, 0),
            M.iron,
            verts = 4,
            rot = (0, 90, 0),
            smooth = false
        ));
        cube!(
            "vane_tail",
            (0.14, 0.01, 0.12),
            at = F.at(0.24, 5.92, 0),
            material = M.iron
        );
        cube!(
            "vane_ns",
            (0.012, 0.3, 0.012),
            at = F.at(0, 5.82, 0),
            material = M.iron
        );
        cube!(
            "vane_ew",
            (0.3, 0.012, 0.012),
            at = F.at(0, 5.82, 0),
            material = M.iron
        );
    }
}

pub fn barn() -> Vec<Obj> {
    barn_shell(true);
    barn_battens();
    barn_corners();
    barn_doors(BD, 1, true);
    barn_doors(-BD, -1, true);
    for s in [-1, 1] {
        for (i, u) in [N::F(-1.6), N::I(0), N::F(1.6)].into_iter().enumerate() {
            barn_window(&format!("win{s}{i}"), (u, 1.72), 'x', s);
        }
    }
    cupola(true);
    settle(&[join_all("barn")])
}

/// The barn from afar: the walls, roof, plinth, the doors' white X and the cupola, a few hundred triangles.
pub fn barn_far() -> Vec<Obj> {
    barn_shell(false);
    barn_doors(BD, 1, false);
    for sx in [-1, 1] {
        for sz in [-1, 1] {
            fbox!(
                F,
                format!("corner{sx}{sz}"),
                0.16,
                EAVE - 0.3,
                0.16,
                (
                    sx as f64 * (BW - 0.06),
                    0.3 + (EAVE - 0.3) / 2.0,
                    sz as f64 * (BD - 0.06)
                ),
                M.barn_trim
            );
        }
    }
    for s in [-1, 1] {
        for u in [N::F(-1.6), N::I(0), N::F(1.6)] {
            let (sf, uf) = (s as f64, u.f());
            prism(
                &format!("win{s}{u}"),
                &pts(&[
                    (uf - 0.43, 1.29),
                    (uf + 0.43, 1.29),
                    (uf + 0.43, 2.15),
                    (uf - 0.43, 2.15),
                ]),
                sf * BW,
                sf * (BW + 0.03),
                M.barn_trim,
                'x',
            );
            prism(
                &format!("pane{s}{u}"),
                &pts(&[
                    (uf - 0.33, 1.39),
                    (uf + 0.33, 1.39),
                    (uf + 0.33, 2.05),
                    (uf - 0.33, 2.05),
                ]),
                sf * BW,
                sf * (BW + 0.04),
                M.shadow,
                'x',
            );
        }
    }
    cupola(false);
    settle(&[join_all("barn_far")])
}
