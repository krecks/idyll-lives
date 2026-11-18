//! The grain silo: a galvanised drum on a concrete pad under a red domed cap, its ladder, walkway, door and chute,
//! and its far version.

use super::common::{F, M, fr};
use artkit::math::{TAU, cos, cosd, degrees, radians, sin, sind};
use artkit::parts::{join_all, settle};
use artkit::scene::Obj;
use artkit::{
    cube, cylinder, fbox, fcyl, fsweep, kit_fillet, kit_tube, revolve, rod, sweep, torus,
};

// --- the silo --------------------------------------------------------------------------------------------
const SILO_R: f64 = 1.2;
const SILO_TOP: f64 = 6.0;

/// The concrete pad, the corrugated drum (stiffening rings every 0.6 m) and the domed cap.
fn silo_drum(detail: bool) {
    revolve!(
        "pad",
        &[
            (0.0, 0.0),
            (1.35, 0.0),
            (1.35, 0.2),
            (1.3, 0.24),
            (0.0, 0.24)
        ],
        material = M.concrete,
        segments = if detail { 32 } else { 16 }
    );
    let mut prof = vec![(0.0, 0.24), (SILO_R, 0.24)];
    if detail {
        // stiffening rings every 0.6 m, the sheet between them corrugated (three waves a bay)
        for k in 0..10 {
            let y0 = 0.24 + 0.6 * k as f64;
            for w in 0..3 {
                let w = w as f64;
                prof.extend([
                    (SILO_R + 0.02, y0 + 0.1 + 0.15 * w + 0.0375),
                    (SILO_R, y0 + 0.1 + 0.15 * w + 0.1125),
                ]);
            }
            let y = y0 + 0.6;
            if y < SILO_TOP - 0.1 {
                prof.extend([
                    (SILO_R, y - 0.05),
                    (SILO_R + 0.03, y - 0.02),
                    (SILO_R + 0.03, y + 0.02),
                    (SILO_R, y + 0.05),
                ]);
            }
        }
        prof.retain(|p| p.1 < SILO_TOP - 0.03);
    }
    prof.extend([
        (SILO_R, SILO_TOP),
        (SILO_R + 0.04, SILO_TOP),
        (SILO_R + 0.04, SILO_TOP + 0.06),
        (0.0, SILO_TOP + 0.06),
    ]);
    revolve!(
        "drum",
        &prof,
        material = M.silo,
        segments = if detail { 24 } else { 16 },
        smooth = detail
    );
    let mut cap = vec![(0.0, SILO_TOP + 0.04), (SILO_R + 0.06, SILO_TOP + 0.04)];
    let rings = if detail { 7 } else { 4 };
    for i in 1..rings + 1 {
        let a = radians(90.0 * i as f64 / rings as f64);
        cap.push((
            (SILO_R + 0.06) * cos(a) * 1.0,
            SILO_TOP + 0.04 + 0.62 * sin(a),
        ));
    }
    revolve!(
        "cap",
        &cap,
        material = M.silo_cap,
        segments = if detail { 32 } else { 16 }
    );
    revolve!(
        "vent",
        &[
            (0.0, SILO_TOP + 0.6),
            (0.16, SILO_TOP + 0.6),
            (0.16, SILO_TOP + 0.72),
            (0.24, SILO_TOP + 0.74),
            (0.2, SILO_TOP + 0.84),
            (0.06, SILO_TOP + 0.9),
            (0.0, SILO_TOP + 0.9)
        ],
        material = M.silo,
        segments = if detail { 16 } else { 8 }
    );
}

pub fn silo() -> Vec<Obj> {
    silo_drum(true);
    // the cap's seams, from its eave over the dome to the vent, and a band round its eave
    for i in 0..12 {
        let t = TAU * i as f64 / 12.0;
        let mut pts = Vec::new();
        for k in 0..7 {
            let e = radians(90.0 * k as f64 / 6.0);
            let r = (SILO_R + 0.06) * cos(e) + 0.012;
            if r < 0.2 {
                break;
            }
            pts.push((
                r * cos(t),
                r * sin(t),
                SILO_TOP + 0.05 + 0.62 * sin(e) + 0.005,
            ));
        }
        sweep!(
            format!("cap_seam{i}"),
            &pts,
            0.014,
            material = M.silo_cap,
            verts = 4,
            resolution = 2
        );
    }
    torus!(
        "cap_eave",
        SILO_R + 0.06,
        0.025,
        at = (0, 0, SILO_TOP + 0.05),
        material = M.silo_cap,
        major_segments = 24,
        minor_segments = 4
    );
    // anchor brackets round the foot, bolted into the pad
    for i in 0..12 {
        let t = TAU * (i as f64 + 0.5) / 12.0;
        cube!(
            format!("anchor{i}"),
            (0.05, 0.08, 0.14),
            at = ((SILO_R + 0.03) * cos(t), (SILO_R + 0.03) * sin(t), 0.31),
            material = M.steel,
            rot = (0, 0, degrees(t)),
            bevel = 0.008,
            bevel_segments = 1
        );
    }
    // the ladder up the front: its rails in one bent piece each, over the eave and up the cap to the walkway
    let z = SILO_R + 0.15;
    for x in [-0.2, 0.2] {
        kit_tube!(
            F,
            format!("rail{}", fr(x)),
            &kit_fillet!(
                &[
                    (x, 0.3, z),
                    (x, SILO_TOP + 0.05, z),
                    (x, SILO_TOP + 0.4, z - 0.25)
                ],
                0.12
            ),
            0.02,
            M.steel,
            verts = 6,
            round = 1
        );
        for y in [1.0, 3.0, 5.0] {
            fbox!(
                F,
                format!("stay{}{}", fr(x), fr(y)),
                0.03,
                0.04,
                0.15,
                (x, y, z - 0.075),
                M.steel,
                bevel = 0.006,
                bevel_segments = 1
            );
            fcyl!(
                F,
                format!("stay_bolt{}{}", fr(x), fr(y)),
                0.018,
                0.04,
                (x, y - 0.03, z - 0.14),
                M.steel,
                verts = 6
            );
        }
    }
    for i in 0..18 {
        rod!(
            format!("rung{i}"),
            F.at(-0.2, 0.6 + i as f64 * 0.3, z),
            F.at(0.2, 0.6 + i as f64 * 0.3, z),
            0.013,
            material = M.steel,
            verts = 6
        );
    }
    // the walkway round the top: posts, a top rail and a mid rail
    for (k, h) in [0.3, 0.16].into_iter().enumerate() {
        torus!(
            format!("walkway{k}"),
            SILO_R + 0.05,
            if k == 0 { 0.02 } else { 0.014 },
            at = (0, 0, SILO_TOP + h),
            material = M.steel,
            major_segments = 32,
            minor_segments = 4
        );
    }
    for i in 0..8 {
        let t = 22.5 + 45.0 * i as f64;
        cylinder!(
            format!("post{i}"),
            0.016,
            0.32,
            at = (1.25 * cosd(t), 1.25 * sind(t), SILO_TOP + 0.15),
            material = M.steel,
            verts = 6
        );
    }
    // the roof hatch at the ladder's head, on its hinge
    cube!(
        "roof_hatch",
        (0.42, 0.36, 0.05),
        at = F.at(0, SILO_TOP + 0.36, SILO_R - 0.45),
        material = M.silo_cap,
        rot = (degrees(0.5), 0, 0),
        bevel = 0.012,
        bevel_segments = 1
    );
    // a framed door at the foot with its handle, and the unloading chute: one bent tube out and down into a hopper
    fbox!(
        F,
        "hatch",
        0.04,
        0.7,
        0.6,
        (-SILO_R - 0.01, 0.75, 0),
        M.steel,
        bevel = 0.01,
        bevel_segments = 1
    );
    for y in [0.42, 1.08] {
        fbox!(
            F,
            format!("hatch_hinge{}", fr(y)),
            0.03,
            0.05,
            0.14,
            (-SILO_R - 0.035, y, -0.27),
            M.iron,
            bevel = 0.005,
            bevel_segments = 1
        );
    }
    let hx = -SILO_R - 0.03;
    fsweep!(
        F,
        "hatch_handle",
        &kit_fillet!(
            &[
                (hx, 0.66, 0.18),
                (hx - 0.05, 0.66, 0.18),
                (hx - 0.05, 0.84, 0.18),
                (hx, 0.84, 0.18)
            ],
            0.02
        ),
        0.012,
        M.iron,
        verts = 6
    );
    kit_tube!(
        F,
        "chute",
        &kit_fillet!(
            &[(0.95, 0.85, -0.52), (1.32, 0.55, -0.72), (1.32, 0.3, -0.72)],
            0.12
        ),
        0.11,
        M.steel,
        verts = 10,
        round = 1
    );
    fbox!(
        F,
        "hopper",
        0.26,
        0.2,
        0.26,
        (1.32, 0.2, -0.72),
        M.steel,
        bevel = 0.02,
        bevel_segments = 1
    );
    settle(&[join_all("silo")])
}

/// The silo from afar: pad, plain drum, cap and vent; the ladder a flat strip.
pub fn silo_far() -> Vec<Obj> {
    silo_drum(false);
    fbox!(
        F,
        "ladder",
        0.44,
        SILO_TOP - 0.3,
        0.04,
        (0, 0.3 + (SILO_TOP - 0.3) / 2.0, SILO_R + 0.15),
        M.steel
    );
    settle(&[join_all("silo_far")])
}
