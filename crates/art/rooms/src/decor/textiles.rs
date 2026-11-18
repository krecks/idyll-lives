//! D3: curtains and rugs.

use super::common::{F, M, WALL, mesh};
use artkit::forms::Plane;
use artkit::kit;
use artkit::materials::Mat;
use artkit::math::{P3, PI, TAU, V3, cos, sin};
use artkit::parts::{hinged, join, join_all, new_parts};
use artkit::py;
use artkit::scene::{Obj, meshes};
use artkit::{cube, cylinder, extrude, revolve, settle, squash, sweep, torus};

/// A gathered curtain panel from game x `x_out` to `x_in` and height `y0` to `y1`, its pleats a sine round game depth
/// `z`, `t` thick: deepest at the hem, where a turned-up hem thickens it, easing up the drop and drawn flat into the
/// heading tape at the top; each fold drifting a little as it falls.
#[allow(clippy::too_many_arguments)]
fn pleats(
    name: &str,
    x_out: f64,
    x_in: f64,
    y0: f64,
    y1: f64,
    z: f64,
    material: Mat,
    waves: usize,
) -> Obj {
    let (amp, t) = (0.018, 0.008);
    let n = waves * 8;
    let xs: Vec<f64> = (0..=n)
        .map(|k| x_out + (x_in - x_out) * k as f64 / n as f64)
        .collect();
    let h = y1 - y0;
    let rows = [
        (y0, 1.0, 1.4, 0.0),
        (y0 + 0.012, 1.0, 1.4, 0.02),
        (y0 + 0.05, 1.0, 1.0, 0.05),
        (y0 + h * 0.35, 0.94, 1.0, 0.25),
        (y0 + h * 0.7, 0.84, 1.0, 0.4),
        (y1 - 0.075, 0.74, 1.0, 0.45),
        (y1 - 0.065, 0.42, 1.3, 0.45),
        (y1, 0.38, 1.3, 0.45),
    ];
    let mut verts: Vec<V3> = Vec::new();
    let mut m = 0;
    for &(y, af, tf, phase) in &rows {
        let front: Vec<(f64, f64)> = xs
            .iter()
            .enumerate()
            .map(|(k, &x)| {
                (
                    x,
                    z + t * tf / 2.0
                        + amp * af * sin(2.0 * PI * waves as f64 * k as f64 / n as f64 + phase),
                )
            })
            .collect();
        let back: Vec<(f64, f64)> = front.iter().map(|&(x, d)| (x, d - t * tf)).collect();
        let mut ring = front.clone();
        ring.extend(back.iter().rev());
        m = ring.len();
        verts.extend(ring.iter().map(|&(x, d)| V3::from(F.at(x, y, d))));
    }
    let mut faces: Vec<Vec<u32>> = Vec::new();
    for r in 0..rows.len() - 1 {
        let (a, b) = (r * m, (r + 1) * m);
        for i in 0..m {
            faces.push(vec![
                (a + i) as u32,
                (a + (i + 1) % m) as u32,
                (b + (i + 1) % m) as u32,
                (b + i) as u32,
            ]);
        }
    }
    let smooth = faces.len();
    for base in [0, (rows.len() - 1) * m] {
        // the bottom and top edges: a strip of quads across the cloth's thickness
        for k in 0..n {
            faces.push(vec![
                (base + k) as u32,
                (base + k + 1) as u32,
                (base + m - 2 - k) as u32,
                (base + m - 1 - k) as u32,
            ]);
        }
    }
    mesh(name, verts, &faces, material, smooth) // the cloth smooth, its hems flat
}

/// Two pleated panels on a rail, gathered open at the sides (rest). At night `close` draws them across: each widens
/// from its outer end until they meet in the middle. The rail ends in turned finials, on brackets from round wall
/// plates.
fn curtains_of(w: f64, top: f64, bottom: f64, gathered: f64) -> Vec<Obj> {
    let (rail_y, z) = (top + 0.05, WALL + 0.08);
    let still = new_parts(|| {
        kit::tube!(
            F,
            "rail",
            &[(-(w / 2.0 + 0.03), rail_y, z), (w / 2.0 + 0.03, rail_y, z)],
            0.012,
            M.rail,
            verts = 12,
            round = 1
        );
        for s in [-1, 1] {
            let sf = s as f64;
            revolve!(
                format!("finial{s}"),
                &[
                    (0.0, 0.0),
                    (0.014, 0.0),
                    (0.016, 0.006),
                    (0.011, 0.012),
                    (0.018, 0.022),
                    (0.024, 0.038),
                    (0.018, 0.054),
                    (0.006, 0.062),
                    (0.0, 0.064)
                ],
                at = F.at(sf * (w / 2.0 + 0.008), rail_y, z),
                material = M.rail,
                segments = 14,
                rot = (0, -s * 90, 0)
            );
            let bx = sf * (w / 2.0 - 0.02);
            kit::tube!(
                F,
                format!("bracket{s}"),
                &[
                    (bx, rail_y - 0.035, WALL + 0.008),
                    (bx, rail_y - 0.03, z - 0.03),
                    (bx, rail_y - 0.012, z - 0.004),
                    (bx, rail_y, z)
                ],
                0.006,
                M.rail,
                verts = 8,
                round = 2
            );
            revolve!(
                format!("bracket_plate{s}"),
                &[
                    (0.0, 0.0),
                    (0.022, 0.0),
                    (0.022, 0.004),
                    (0.016, 0.008),
                    (0.008, 0.01),
                    (0.0, 0.01)
                ],
                at = F.at(bx, rail_y - 0.035, WALL),
                material = M.rail,
                segments = 14,
                rot = (90, 0, 0)
            );
        }
    });
    join(&still, "body", Some(P3::ZERO));
    let mut panels = Vec::new();
    for s in [-1, 1] {
        let sf = s as f64;
        let x_out = sf * w / 2.0;
        let parts = new_parts(|| {
            let waves = 3.max(py::round(gathered / 0.06)) as usize;
            pleats(
                &format!("panel{s}"),
                x_out,
                x_out - sf * gathered,
                bottom,
                top + 0.02,
                z,
                M.linen,
                waves,
            );
            for k in 0..4 {
                // cloth tabs over the rail (the panel's own fabric: no draw call of their own)
                torus!(
                    format!("tab{s}_{k}"),
                    0.018,
                    0.006,
                    at = F.at(
                        x_out - sf * (0.02 + k as f64 * (gathered - 0.04) / 3.0),
                        rail_y,
                        z
                    ),
                    material = M.linen,
                    rot = (0, 90, 0),
                    major_segments = 12,
                    minor_segments = 5
                );
            }
        });
        let name = if s < 0 { "panel_left" } else { "panel_right" };
        panels.push(hinged(join(&parts, name, None), F.at(x_out, rail_y, z)));
    }
    settle!(centre = false, floor = false);
    for panel in panels {
        squash!(
            panel,
            "close",
            ((w / 2.0 + 0.01) / gathered, 1, 1),
            seconds = 1.2
        );
    }
    meshes()
}

/// The window-length pair.
pub fn curtains() -> Vec<Obj> {
    curtains_of(1.16, 2.45, 0.85, 0.2)
}

/// The wide pair.
pub fn curtains_wide() -> Vec<Obj> {
    curtains_of(2.32, 2.45, 0.85, 0.3)
}

/// The floor-length pair.
pub fn curtains_long() -> Vec<Obj> {
    curtains_of(2.32, 2.55, 0.04, 0.3)
}

// --- D3: rugs ---------------------------------------------------------------------------------------------
// Rugs lie flat (Blender axes, centred): a bound border 1.2 cm, the field in it to 1.6 cm and what is woven into it
// raised to at most 1.9 cm (the size's height), so the house's floor, drawn 1 cm up, hides none of it.

/// A motif woven into a rug: the outline `pts` (Blender x, y) from height `z0` to `z1`.
fn flat_shape(name: &str, pts: &[(f64, f64)], z0: f64, z1: f64, material: Mat) -> Obj {
    extrude!(
        name,
        pts,
        z1 - z0,
        at = (0, 0, (z0 + z1) / 2.0),
        material = material,
        plane = Plane::XY
    )
}

/// A diamond's outline round (cx, cy), `rx` and `ry` to its points; `steps` > 0 cuts its edges into that many
/// stairs, as a knotted rug's diagonals are.
fn diamond(cx: f64, cy: f64, rx: f64, ry: f64, steps: usize) -> Vec<(f64, f64)> {
    let pts = [(cx + rx, cy), (cx, cy + ry), (cx - rx, cy), (cx, cy - ry)];
    if steps == 0 {
        return pts.to_vec();
    }
    let far = |c: (f64, f64)| (c.0 - cx).abs() / rx + (c.1 - cy).abs() / ry;
    let mut out = Vec::new();
    for i in 0..4 {
        let ((px, py), (qx, qy)) = (pts[i], pts[(i + 1) % 4]);
        let (dx, dy) = ((qx - px) / steps as f64, (qy - py) / steps as f64);
        for j in 0..steps {
            let (ax, ay) = (px + dx * j as f64, py + dy * j as f64);
            out.push((ax, ay));
            // the stair's corner on the outside of the diagonal
            let (c1, c2) = ((ax, ay + dy), (ax + dx, ay));
            out.push(if far(c1) > far(c2) { c1 } else { c2 });
        }
    }
    out
}

/// A fringe of knotted tassels lying on the floor, one at each of `xs` across the rug's end at `y_edge`, `length` out
/// from it (`across`: along the end's normal Blender x, not y), each a wavy cord with a knot where it leaves the rug.
fn fringe(
    name: &str,
    xs: &[f64],
    y_edge: f64,
    length: f64,
    material: Mat,
    side: f64,
    across: bool,
) {
    for (i, &x) in xs.iter().enumerate() {
        let wob = 0.006 * sin(i as f64 * 2.3);
        let mut pts = vec![
            (x, y_edge - side * 0.004),
            (x + wob * 0.4, y_edge + side * length * 0.45),
            (x + wob, y_edge + side * length),
        ];
        if across {
            pts = pts.iter().map(|&(a, b)| (b, a)).collect();
        }
        let pts: Vec<(f64, f64, f64)> = pts.iter().map(|&(px, py)| (px, py, 0.0034)).collect();
        sweep!(
            format!("{name}{i}"),
            &pts,
            0.0026,
            material = material,
            verts = 4,
            resolution = 2,
            scale = [1.3, 1.0, 0.8]
        );
    }
}

/// A flat rug `sx` wide and `sz` deep: a border, a raised field and a line round it, 2 cm high.
fn rug_rect(name: &str, sx: f64, sz: f64, field: Mat, border: Mat, line: Mat) {
    cube!(
        format!("{name}_border"),
        (sx, sz, 0.012),
        at = (0, 0, 0.006),
        material = border,
        bevel = 0.005,
        bevel_segments = 3
    );
    cube!(
        format!("{name}_field"),
        (sx - 0.24, sz - 0.24, 0.006),
        at = (0, 0, 0.013),
        material = field,
        bevel = 0.002
    );
    let lw = 0.03;
    for s in [-1, 1] {
        let sf = s as f64;
        cube!(
            format!("{name}_line_x{s}"),
            (sx - 0.44, lw, 0.004),
            at = (0.0, sf * (sz - 0.44 - lw) / 2.0, 0.017),
            material = line,
            bevel = 0.0015
        );
        cube!(
            format!("{name}_line_z{s}"),
            (lw, sz - 0.44 - 2.0 * lw, 0.004),
            at = (sf * (sx - 0.44 - lw) / 2.0, 0.0, 0.017),
            material = line,
            bevel = 0.0015
        );
    }
}

/// A round rug: a darker bound border, a field with a thin ring, a garland of sixteen cream leaves round a stepped
/// star medallion with a field-coloured heart.
pub fn rug_round() -> Vec<Obj> {
    let d = 1.9;
    new_parts(|| {
        cylinder!(
            "border",
            d / 2.0,
            0.012,
            at = (0, 0, 0.006),
            material = M.rug_border,
            verts = 72,
            bevel = 0.005,
            bevel_segments = 3
        );
        cylinder!(
            "field",
            d / 2.0 - 0.12,
            0.006,
            at = (0, 0, 0.013),
            material = M.rug,
            verts = 72,
            bevel = 0.002
        );
        revolve!(
            "line",
            &[
                (d / 2.0 - 0.25, 0.015),
                (d / 2.0 - 0.22, 0.015),
                (d / 2.0 - 0.22, 0.019),
                (d / 2.0 - 0.25, 0.019),
                (d / 2.0 - 0.25, 0.015)
            ],
            material = M.rug_border,
            segments = 72,
            smooth = false
        );
        for k in 0..16 {
            let a = TAU * k as f64 / 16.0;
            let mut leaf: Vec<(f64, f64)> = (0..7)
                .map(|i| {
                    let t = i as f64 / 6.0;
                    (0.5 * 0.13 * (2.0 * t - 1.0), 0.03 * sin(PI * t))
                })
                .collect();
            let under: Vec<(f64, f64)> = leaf[1..leaf.len() - 1]
                .iter()
                .rev()
                .map(|&(u, v)| (u, -v))
                .collect();
            leaf.extend(under);
            let r = 0.52;
            let pts: Vec<(f64, f64)> = leaf
                .iter()
                .map(|&(u, v)| {
                    (
                        r * cos(a) + u * -sin(a) + v * cos(a),
                        r * sin(a) + u * cos(a) + v * sin(a),
                    )
                })
                .collect();
            flat_shape(&format!("leaf{k}"), &pts, 0.0155, 0.019, M.rug_accent);
        }
        let star: Vec<(f64, f64)> = (0..16)
            .map(|k| {
                let a = TAU * k as f64 / 16.0;
                let rr = if k % 2 == 0 { 0.3 } else { 0.2 };
                (rr * cos(a), rr * sin(a))
            })
            .collect();
        flat_shape("medallion", &star, 0.0155, 0.0175, M.rug_border);
        let heart: Vec<(f64, f64)> = (0..8)
            .map(|k| {
                (
                    0.11 * cos(TAU * k as f64 / 8.0 + PI / 8.0),
                    0.11 * sin(TAU * k as f64 / 8.0 + PI / 8.0),
                )
            })
            .collect();
        flat_shape("medallion_heart", &heart, 0.0165, 0.019, M.rug_accent);
        flat_shape(
            "medallion_eye",
            &diamond(0.0, 0.0, 0.05, 0.05, 0),
            0.0175,
            0.019,
            M.rug,
        );
    });
    join_all("rugRound");
    settle!();
    meshes()
}

/// A large plain rug: a bound border and a field with a line round it, a stepped diamond medallion and stepped corner
/// pieces in the border's colour, and a knotted fringe across each short end.
pub fn rug_large() -> Vec<Obj> {
    let (sx, sz) = (2.75, 1.85);
    new_parts(|| {
        rug_rect("rug", sx, sz, M.rug, M.rug_border, M.rug_border);
        flat_shape(
            "medallion",
            &diamond(0.0, 0.0, 0.42, 0.3, 5),
            0.0155,
            0.0175,
            M.rug_border,
        );
        flat_shape(
            "medallion_core",
            &diamond(0.0, 0.0, 0.2, 0.14, 3),
            0.0165,
            0.019,
            M.rug_accent,
        );
        flat_shape(
            "medallion_eye",
            &diamond(0.0, 0.0, 0.07, 0.05, 0),
            0.0175,
            0.019,
            M.rug,
        );
        for qx in [-1, 1] {
            for qy in [-1, 1] {
                let (fx, fy) = (qx as f64, qy as f64);
                let (cx, cy) = (fx * (sx / 2.0 - 0.25), fy * (sz / 2.0 - 0.25));
                let corner = [
                    (cx, cy),
                    (cx - fx * 0.26, cy),
                    (cx - fx * 0.26, cy - fy * 0.04),
                    (cx - fx * 0.14, cy - fy * 0.04),
                    (cx - fx * 0.14, cy - fy * 0.1),
                    (cx - fx * 0.04, cy - fy * 0.1),
                    (cx - fx * 0.04, cy - fy * 0.18),
                    (cx, cy - fy * 0.18),
                ];
                flat_shape(
                    &format!("corner{qx}{qy}"),
                    &corner,
                    0.0155,
                    0.019,
                    M.rug_border,
                );
            }
        }
        for s in [-1, 1] {
            let xs: Vec<f64> = (0..18).map(|i| -0.85 + 0.1 * i as f64).collect();
            fringe(
                &format!("fringe{s}_"),
                &xs,
                s as f64 * sx / 2.0,
                0.05,
                M.rug_accent,
                s as f64,
                true,
            );
        }
    });
    join_all("rugLarge");
    settle!();
    meshes()
}

/// A hall runner: a bound border, a field with a line round it, three stepped medallions down its length with cream
/// hearts, and a knotted fringe on each short end.
pub fn rug_runner() -> Vec<Obj> {
    new_parts(|| {
        rug_rect("rug", 0.8, 2.68, M.rug, M.rug_border, M.rug_border);
        for (k, y) in [-0.75, 0.0, 0.75].into_iter().enumerate() {
            flat_shape(
                &format!("medallion{k}"),
                &diamond(0.0, y, 0.16, 0.26, 4),
                0.0155,
                0.0175,
                M.rug_border,
            );
            flat_shape(
                &format!("heart{k}"),
                &diamond(0.0, y, 0.06, 0.1, 0),
                0.0165,
                0.019,
                M.rug_accent,
            );
        }
        for (k, y) in [-0.375, 0.375].into_iter().enumerate() {
            flat_shape(
                &format!("link{k}"),
                &diamond(0.0, y, 0.05, 0.05, 0),
                0.0155,
                0.0175,
                M.rug_border,
            );
        }
        for s in [-1, 1] {
            let xs: Vec<f64> = (0..9).map(|i| -0.32 + 0.08 * i as f64).collect();
            fringe(
                &format!("fringe{s}_"),
                &xs,
                s as f64 * 1.34,
                0.06,
                M.rug_border,
                s as f64,
                false,
            );
        }
    });
    join_all("rugRunner");
    settle!();
    meshes()
}

/// A patterned rug: a navy border with a cream zigzag running round it, a terracotta ground, a large stepped
/// medallion in three tiers with two smaller ones beside it, stepped corner pieces, and a knotted fringe on each short
/// end.
pub fn rug_pattern() -> Vec<Obj> {
    let (sx, sz) = (2.7, 1.86);
    new_parts(|| {
        cube!(
            "border",
            (sx, sz, 0.012),
            at = (0, 0, 0.006),
            material = M.rug_edge,
            bevel = 0.005,
            bevel_segments = 3
        );
        cube!(
            "ground",
            (sx - 0.26, sz - 0.26, 0.006),
            at = (0, 0, 0.013),
            material = M.rug_ground,
            bevel = 0.002
        );
        // the zigzag round the border: a band of teeth along each side
        for s in [-1, 1] {
            let sf = s as f64;
            for (along, length, at_edge) in [
                ("x", sx - 0.12, sz / 2.0 - 0.065),
                ("y", sz - 0.12, sx / 2.0 - 0.065),
            ] {
                let n = (length / 0.08) as usize;
                let pts: Vec<(f64, f64)> = (0..=n)
                    .map(|i| {
                        (
                            -length / 2.0 + length * i as f64 / n as f64,
                            0.018 * if i % 2 == 1 { 1.0 } else { -1.0 },
                        )
                    })
                    .collect();
                let mut band: Vec<(f64, f64)> = pts.iter().map(|&(u, v)| (u, v + 0.008)).collect();
                band.extend(pts.iter().rev().map(|&(u, v)| (u, v - 0.008)));
                let band: Vec<(f64, f64)> = if along == "x" {
                    band.iter().map(|&(u, v)| (u, sf * at_edge + v)).collect()
                } else {
                    band.iter().map(|&(u, v)| (sf * at_edge + v, u)).collect()
                };
                flat_shape(
                    &format!("zigzag_{along}{s}"),
                    &band,
                    0.0115,
                    0.0135,
                    M.rug_accent,
                );
            }
            cube!(
                format!("line_x{s}"),
                (sx - 0.46, 0.04, 0.004),
                at = (0.0, sf * (sz - 0.5) / 2.0, 0.017),
                material = M.rug_accent,
                bevel = 0.0015
            );
            cube!(
                format!("line_z{s}"),
                (0.04, sz - 0.54, 0.004),
                at = (sf * (sx - 0.5) / 2.0, 0.0, 0.017),
                material = M.rug_accent,
                bevel = 0.0015
            );
        }
        flat_shape(
            "medallion",
            &diamond(0.0, 0.0, 0.5, 0.42, 6),
            0.0155,
            0.0185,
            M.rug_motif,
        );
        flat_shape(
            "medallion_core",
            &diamond(0.0, 0.0, 0.3, 0.25, 4),
            0.0175,
            0.0205,
            M.rug_accent,
        );
        flat_shape(
            "medallion_eye",
            &diamond(0.0, 0.0, 0.12, 0.1, 2),
            0.0195,
            0.0225,
            M.rug_edge,
        );
        flat_shape(
            "medallion_dot",
            &diamond(0.0, 0.0, 0.04, 0.035, 0),
            0.0215,
            0.024,
            M.rug_motif,
        );
        for s in [-1, 1] {
            let sf = s as f64;
            flat_shape(
                &format!("side{s}"),
                &diamond(sf * 0.85, 0.0, 0.17, 0.17, 3),
                0.0155,
                0.0185,
                M.rug_motif,
            );
            flat_shape(
                &format!("side_core{s}"),
                &diamond(sf * 0.85, 0.0, 0.07, 0.07, 0),
                0.0175,
                0.0205,
                M.rug_accent,
            );
            for qy in [-1, 1] {
                let fy = qy as f64;
                let (cx, cy) = (sf * (sx / 2.0 - 0.27), fy * (sz / 2.0 - 0.27));
                let corner = [
                    (cx, cy),
                    (cx - sf * 0.24, cy),
                    (cx - sf * 0.24, cy - fy * 0.05),
                    (cx - sf * 0.12, cy - fy * 0.05),
                    (cx - sf * 0.12, cy - fy * 0.12),
                    (cx - sf * 0.05, cy - fy * 0.12),
                    (cx - sf * 0.05, cy - fy * 0.2),
                    (cx, cy - fy * 0.2),
                ];
                flat_shape(
                    &format!("corner{s}{qy}"),
                    &corner,
                    0.0155,
                    0.0185,
                    M.rug_motif,
                );
            }
            let xs: Vec<f64> = (0..18).map(|i| -0.85 + 0.1 * i as f64).collect();
            fringe(
                &format!("fringe{s}_"),
                &xs,
                sf * sx / 2.0,
                0.07,
                M.rug_accent,
                sf,
                true,
            );
        }
    });
    join_all("rugPattern");
    settle!();
    meshes()
}
