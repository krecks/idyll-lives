//! The chicken coop: a wooden house on braced legs in lap siding under a shingled gable roof, its ramp, windows,
//! vents, nest box and feeder, and its pop-hole hatch that slides shut at night; and its far version.

use super::common::{F, M, P2, frame, prism, pts, shell, strut};
use artkit::json::float_repr;
use artkit::parts::{join, join_all, new_parts, settle, settle_all};
use artkit::place::Coord;
use artkit::py;
use artkit::scene::{Obj, meshes};
use artkit::{cube, fbox, fcone, fcyl, rod, slide, sphere};

// --- the chicken coop ------------------------------------------------------------------------------------
const CF: f64 = 0.3; // the house's front and back (depth)
const CB: f64 = -0.8;
const CX: f64 = 0.75; // half its width
const CY0: f64 = 0.55;
const CEAVE: f64 = 1.45;
const CRIDGE: f64 = 1.78;
const CRZ: f64 = (CF + CB) / 2.0; // the ridge's depth
const HATCH: (f64, f64, f64) = (0.17, 0.6, 0.95); // the pop hole: half its width, its bottom and top
const HATCH_UP: f64 = 0.32; // how far the hatch stands up its runners by day, the hole open

fn coop_roof_line() -> Vec<P2> {
    let (dz, dy) = (CRZ - CF, CRIDGE - CEAVE);
    let k = py::hypot(dz, dy);
    let e = 0.16;
    pts(&[
        (CB + e * dz / k, CEAVE - e * dy / k),
        (CRZ, CRIDGE),
        (CF - e * dz / k, CEAVE - e * dy / k),
    ])
}

fn coop_body(detail: bool) {
    let wall = pts(&[
        (CB, CY0),
        (CF, CY0),
        (CF, CEAVE),
        (CRZ, CRIDGE),
        (CB, CEAVE),
    ]);
    prism("house", &wall, -CX, CX, M.coop, 'x');
    shell(
        "roof",
        &coop_roof_line(),
        0.05,
        -CX - 0.1,
        CX + 0.1,
        M.coop_roof,
        'x',
    );
    cube!(
        "ridge_cap",
        (2.0 * CX + 0.24, 0.1, 0.05),
        at = F.at(0, CRIDGE + 0.06, CRZ),
        material = M.coop_roof
    );
    for sx in [-1, 1] {
        for z in [CB + 0.1, CF - 0.1] {
            fbox!(
                F,
                format!("leg{sx}{}", float_repr(z)),
                0.08,
                CY0 + 0.02,
                0.08,
                (sx as f64 * (CX - 0.1), CY0 / 2.0, z),
                M.coop
            );
        }
    }
    fbox!(
        F,
        "floor",
        2.0 * CX + 0.04,
        0.05,
        CF - CB + 0.04,
        (0, CY0 + 0.01, (CF + CB) / 2.0),
        M.coop
    );
    // the ramp up to the pop hole, three planks with cleats across them
    let (a, b) = ((CF + 0.68, 0.0), (CF + 0.02, CY0 + 0.03));
    if !detail {
        strut("ramp", a, b, 0.035, -0.15, 0.15, M.coop, 'x');
        return;
    }
    for (k, (w0, w1)) in [(-0.15, -0.054), (-0.046, 0.046), (0.054, 0.15)]
        .into_iter()
        .enumerate()
    {
        strut(&format!("ramp{k}"), a, b, 0.035, w0, w1, M.coop, 'x');
    }
    for k in 1..6 {
        let t = k as f64 / 6.0;
        let (z, y) = (a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t);
        fbox!(
            F,
            format!("cleat{k}"),
            0.28,
            0.025,
            0.025,
            (0, y + 0.03, z),
            M.coop_trim
        );
    }
    // braces between the legs on each side
    for sx in [-1, 1] {
        let x = sx as f64 * (CX - 0.1);
        strut(
            &format!("brace{sx}"),
            (CB + 0.14, 0.1),
            (CF - 0.14, CY0 - 0.04),
            0.05,
            x - 0.015,
            x + 0.015,
            M.coop,
            'x',
        );
    }
}

/// Lap siding: boards a hand wide, each tilted out at its foot over the one below, round the pop hole and windows.
fn coop_siding() {
    let rows: Vec<f64> = (0..9).map(|k| CY0 + 0.02 + 0.112 * k as f64).collect();
    for (k, (&y0, &y1)) in rows.iter().zip(&rows[1..]).enumerate() {
        for (face, z, s) in [("f", CF, 1.0), ("b", CB, -1.0)] {
            let mut spans = vec![(-CX + 0.05, CX - 0.05)];
            if face == "f" && y0 < HATCH.1 + 0.36 {
                spans = vec![(-CX + 0.05, -0.245), (0.245, CX - 0.05)];
            }
            for (i, &(x0, x1)) in spans.iter().enumerate() {
                strut(
                    &format!("board_{face}{k}{i}"),
                    (z + s * 0.017, y0),
                    (z + s * 0.004, y1 + 0.006),
                    0.012,
                    x0,
                    x1,
                    M.coop,
                    'x',
                );
            }
        }
    }
    let mut gable = rows.clone();
    gable.extend((1..4).map(|k| CEAVE + 0.11 * k as f64));
    for (k, (&y0, &y1)) in gable.iter().zip(&gable[1..]).enumerate() {
        let f = if y1 > CEAVE {
            0.0f64.max((CRIDGE - if CRIDGE < y1 { CRIDGE } else { y1 }) / (CRIDGE - CEAVE))
        } else {
            1.0
        };
        let (d0, d1) = (CRZ - (CRZ - CB) * f + 0.03, CRZ + (CF - CRZ) * f - 0.03);
        if d1 - d0 < 0.08 {
            continue;
        }
        let mut spans = vec![(d0, d1)];
        if y1 > 0.86 && y0 < 1.24 {
            spans = vec![(d0, -0.39), (-0.01, d1)];
        }
        for sx in [-1, 1] {
            let x = sx as f64 * CX;
            for (i, &(w0, w1)) in spans.iter().enumerate() {
                strut(
                    &format!("board_s{sx}{k}{i}"),
                    (x + sx as f64 * 0.017, y0),
                    (x + sx as f64 * 0.004, y1 + 0.006),
                    0.012,
                    w0,
                    w1,
                    M.coop,
                    'z',
                );
            }
        }
    }
}

/// The roof in four courses of shingles a side, each course's tabs staggered over the last's joints.
fn coop_shingles() {
    let line = coop_roof_line();
    let f2 = |p: P2| (p.0.f(), p.1.f());
    for (side, (p0, p1)) in [(line[0], line[1]), (line[2], line[1])]
        .into_iter()
        .enumerate()
    {
        let ((z0, y0), (z1, y1)) = (f2(p0), f2(p1));
        let (dz, dy) = (z1 - z0, y1 - y0);
        let k = py::hypot(dz, dy);
        // out of the roof, up
        let (nz, ny) = if side == 0 {
            (-dy / k, dz / k)
        } else {
            (dy / k, -dz / k)
        };
        for c in 0..4usize {
            let (t0, t1) = (c as f64 / 4.0, 1.0f64.min((c + 1) as f64 / 4.0 + 0.05));
            let a = (z0 + dz * t0 + nz * 0.068, y0 + dy * t0 + ny * 0.068);
            let b = (z0 + dz * t1 + nz * 0.056, y0 + dy * t1 + ny * 0.056);
            let n = 7;
            let span = 2.0 * CX + 0.2;
            for i in 0..n + (c % 2) {
                let w0 = -span / 2.0 + span * (i as f64 - 0.5 * (c % 2) as f64) / n as f64;
                let w1 = w0 + span / n as f64;
                let (w0, w1) = (
                    (if w0 > -span / 2.0 { w0 } else { -span / 2.0 }) + 0.006,
                    (if w1 < span / 2.0 { w1 } else { span / 2.0 }) - 0.006,
                );
                strut(
                    &format!("shingle{side}{c}{i}"),
                    a,
                    b,
                    0.014,
                    w0,
                    w1,
                    M.coop_roof,
                    'x',
                );
            }
        }
    }
}

pub fn coop() -> Vec<Obj> {
    let body = new_parts(|| {
        coop_body(true);
        coop_siding();
        coop_shingles();
        // corner boards and the eave trim
        for sx in [-1, 1] {
            for z in [CB, CF] {
                fbox!(
                    F,
                    format!("corner{sx}{}", float_repr(z)),
                    0.06,
                    CEAVE - CY0,
                    0.06,
                    (
                        sx as f64 * (CX - 0.02),
                        (CY0 + CEAVE) / 2.0,
                        z + if z == CF { 0.02 } else { -0.02 }
                    ),
                    M.coop_trim
                );
            }
        }
        // the pop hole, dark behind the hatch, and the runners the hatch slides up in
        let (hw, h0, h1) = HATCH;
        fbox!(
            F,
            "pop_hole",
            2.0 * hw,
            h1 - h0,
            0.02,
            (0, (h0 + h1) / 2.0, CF + 0.005),
            M.coop_dark
        );
        for s in [-1, 1] {
            fbox!(
                F,
                format!("runner{s}"),
                0.04,
                0.72,
                0.03,
                (s as f64 * (hw + 0.03), h0 + 0.33, CF + 0.02),
                M.coop_trim
            );
        }
        fbox!(
            F,
            "runner_top",
            2.0 * hw + 0.1,
            0.04,
            0.03,
            (0, h0 + 0.69, CF + 0.02),
            M.coop_trim
        );
        // a window with wire mesh on each side, its sill, and a slatted vent high in each gable
        for s in [-1, 1] {
            let sf = s as f64;
            let x = sf * CX;
            prism(
                &format!("win{s}"),
                &pts(&[(-0.35, 0.9), (-0.05, 0.9), (-0.05, 1.2), (-0.35, 1.2)]),
                x,
                x + sf * 0.01,
                M.coop_dark,
                'x',
            );
            frame(
                &format!("win{s}_frame"),
                -0.35,
                0.9,
                -0.05,
                1.2,
                x,
                0.04,
                sf * 0.03,
                M.coop_trim,
                'x',
            );
            for k in 0..3 {
                let u = -0.35 + 0.075 * (k + 1) as f64;
                prism(
                    &format!("wire_v{s}{k}"),
                    &pts(&[
                        (u - 0.004, 0.9),
                        (u + 0.004, 0.9),
                        (u + 0.004, 1.2),
                        (u - 0.004, 1.2),
                    ]),
                    x,
                    x + sf * 0.016,
                    M.coop_trim,
                    'x',
                );
                let v = 0.9 + 0.075 * (k + 1) as f64;
                prism(
                    &format!("wire_h{s}{k}"),
                    &pts(&[
                        (-0.35, v - 0.004),
                        (-0.05, v - 0.004),
                        (-0.05, v + 0.004),
                        (-0.35, v + 0.004),
                    ]),
                    x,
                    x + sf * 0.016,
                    M.coop_trim,
                    'x',
                );
            }
            fbox!(
                F,
                format!("win_sill{s}"),
                0.05,
                0.025,
                0.38,
                (x + sf * 0.03, 0.865, -0.2),
                M.coop_trim,
                bevel = 0.006,
                bevel_segments = 1
            );
            prism(
                &format!("vent{s}"),
                &pts(&[
                    (CRZ - 0.12, 1.53),
                    (CRZ + 0.12, 1.53),
                    (CRZ + 0.12, 1.63),
                    (CRZ - 0.12, 1.63),
                ]),
                x,
                x + sf * 0.025,
                M.coop_dark,
                'x',
            );
            frame(
                &format!("vent{s}_frame"),
                CRZ - 0.12,
                1.53,
                CRZ + 0.12,
                1.63,
                x,
                0.03,
                sf * 0.035,
                M.coop_trim,
                'x',
            );
            for k in 0..2 {
                let v = 1.555 + 0.04 * k as f64;
                strut(
                    &format!("vent_slat{s}{k}"),
                    (CRZ - 0.11, v),
                    (CRZ + 0.11, v),
                    0.012,
                    x,
                    x + sf * 0.03,
                    M.coop_trim,
                    'x',
                );
            }
        }
        // the nest box on the back, its lid sloping like the roof on two hinges, its latch
        fbox!(F, "nest", 0.9, 0.36, 0.14, (0, 0.92, CB - 0.07), M.coop);
        strut(
            "nest_lid",
            (CB - 0.17, 1.08),
            (CB + 0.01, 1.16),
            0.035,
            -0.48,
            0.48,
            M.coop_roof,
            'x',
        );
        for x in [-0.3, 0.3] {
            strut(
                &format!("nest_hinge{}", float_repr(x)),
                (CB - 0.06, 1.155),
                (CB + 0.02, 1.19),
                0.008,
                x - 0.03,
                x + 0.03,
                M.iron,
                'x',
            );
        }
        fbox!(
            F,
            "nest_latch",
            0.06,
            0.04,
            0.02,
            (0, 1.0, CB - 0.15),
            M.iron
        );
        // a feeder hanging under the house
        fcyl!(
            F,
            "feeder",
            0.08,
            0.2,
            (0.3, 0.34, -0.3),
            M.trough,
            verts = 12
        );
        fcone!(
            F,
            "feeder_lid",
            0.095,
            0.02,
            0.06,
            (0.3, 0.47, -0.3),
            M.trough,
            verts = 12
        );
        fcyl!(
            F,
            "feeder_pan",
            0.13,
            0.03,
            (0.3, 0.225, -0.3),
            M.trough,
            verts = 14
        );
        rod!(
            "feeder_wire",
            F.at(0.3, 0.5, -0.3),
            F.at(0.3, CY0 - 0.015, -0.3),
            0.005,
            material = M.iron,
            verts = 4
        );
    });
    join(&body, "coop", Some((0, 0, 0).into()));
    // the pop-hole hatch: its own part, its origin the middle of its top edge. By day it stands up in its runners,
    // the hole open for the hens; at night `close` slides it down over the hole (`motions.night`).
    let (hw, h0, h1) = HATCH;
    let hatch_parts = new_parts(|| {
        fbox!(
            F,
            "hatch",
            2.0 * hw + 0.02,
            h1 - h0 + 0.02,
            0.025,
            (0, (h0 + h1) / 2.0, CF + 0.03),
            M.coop_trim,
            bevel = 0.005
        );
        for y in [h0 + 0.05, h1 - 0.05] {
            fbox!(
                F,
                format!("hatch_rail{}", float_repr(y)),
                2.0 * hw - 0.04,
                0.035,
                0.012,
                (0, y, CF + 0.048),
                M.coop_trim,
                bevel = 0.003,
                bevel_segments = 1
            );
        }
        strut(
            "hatch_brace",
            (-hw + 0.04, h0 + 0.07),
            (hw - 0.04, h1 - 0.07),
            0.03,
            CF + 0.042,
            CF + 0.054,
            M.coop_trim,
            'z',
        );
        sphere!(
            "hatch_knob",
            0.016,
            at = F.at(0, h0 + 0.06, CF + 0.065),
            material = M.iron,
            segments = 8,
            rings = 5
        );
    });
    let hatch = join(&hatch_parts, "hatch", Some(F.at(0, h1 + 0.01, CF + 0.03)));
    hatch.set_location_axis(2, hatch.location().z as f64 + HATCH_UP);
    settle_all();
    slide!(hatch, "close", (0, 0, -HATCH_UP), seconds = 1.0);
    meshes()
}

/// The coop from afar: the house, roof, legs and ramp, its trims, the dark pop hole with the hatch up, the windows
/// as dark panels in their frames, the nest box: a few hundred triangles (decimated, its legs and ramp went to
/// spikes).
pub fn coop_far() -> Vec<Obj> {
    coop_body(false);
    for sx in [-1, 1] {
        let sf = sx as f64;
        for z in [CB, CF] {
            fbox!(
                F,
                format!("corner{sx}{}", float_repr(z)),
                0.06,
                CEAVE - CY0,
                0.06,
                (
                    sf * (CX - 0.02),
                    (CY0 + CEAVE) / 2.0,
                    z + if z == CF { 0.02 } else { -0.02 }
                ),
                M.coop_trim
            );
        }
        let x = sf * CX;
        prism(
            &format!("win{sx}"),
            &pts(&[(-0.37, 0.88), (-0.03, 0.88), (-0.03, 1.22), (-0.37, 1.22)]),
            x,
            x + sf * 0.02,
            M.coop_trim,
            'x',
        );
        prism(
            &format!("pane{sx}"),
            &pts(&[(-0.33, 0.92), (-0.07, 0.92), (-0.07, 1.18), (-0.33, 1.18)]),
            x,
            x + sf * 0.025,
            M.coop_dark,
            'x',
        );
    }
    let (hw, h0, h1) = HATCH;
    fbox!(
        F,
        "pop_hole",
        2.0 * hw,
        h1 - h0,
        0.02,
        (0, (h0 + h1) / 2.0, CF + 0.005),
        M.coop_dark
    );
    fbox!(
        F,
        "hatch",
        2.0 * hw + 0.02,
        h1 - h0 + 0.02,
        0.025,
        (0, (h0 + h1) / 2.0 + HATCH_UP, CF + 0.03),
        M.coop_trim
    );
    fbox!(F, "nest", 0.9, 0.36, 0.14, (0, 0.92, CB - 0.07), M.coop);
    strut(
        "nest_lid",
        (CB - 0.17, 1.08),
        (CB + 0.01, 1.16),
        0.035,
        -0.48,
        0.48,
        M.coop_roof,
        'x',
    );
    settle(&[join_all("coop_far")])
}
